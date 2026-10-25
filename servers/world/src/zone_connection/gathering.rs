//! A gatherer at a node: the window, each gather and the actions used there (`crate::gathering` has the rules.)
//!
//! The packets and their order are retail's, from Bob's Chronofoil capture of 2026-10-24 (a level 58 Miner and a level
//! 60 Botanist at normal, unspoiled and ephemeral nodes; docs/src/fork/gathering.md.)

use std::collections::{HashMap, HashSet, VecDeque};

use kawari::{
    common::{ObjectTypeId, ObjectTypeKind},
    ipc::zone::{
        ActionRequest, ActionType, ActorControlCategory, EventLogMessage, LiveEventType,
        SceneFlags, ServerZoneIpcData, ServerZoneIpcSegment,
    },
};

use crate::{
    Event, GatheringItemFlag, ItemInfoQuery, ToServer, ZoneConnection,
    gamedata::{GatheringPointBonusInfo, GatheringPointItem},
    gathering::{self, GatheringAction, PHASE_SLOTS},
    inventory::Item,
    lua::LuaPlayer,
};

/// The LogMessages after a gather, each with the item and how many (retail.)
const LOG_GATHERED: [u32; 2] = [1050, 1054];
/// How many used-up points each player keeps hidden: when another is used up, the oldest comes back (retail.)
const EXHAUSTED_POINTS_KEPT: usize = 2;
/// The ActorControlSelf retail sent with a point coming back: [0x30000 | point, 0, count, count, 1].
const ACTOR_CONTROL_POINT_BACK: u32 = 109;

/// How long a used-up timed point stays gone: four Eorzea hours, longer than any window (two or three) and no longer
/// than the shortest gap to the next one (the 01:00/05:00/09:00 points'.)
const TIMED_POINT_GONE_SECONDS: u64 = 4 * 175;

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

/// A gather rolled when the item was picked, given when its animation (scene 1) ends.
#[derive(Debug, Clone, Copy)]
struct PendingGather {
    slot: usize,
    item: GatheringPointItem,
    amount: u32,
    boon: bool,
}

/// The node the gatherer is at.
#[derive(Debug, Clone, Copy, Default)]
struct NodeVisit {
    /// Gathering attempts made here.
    gathers: u32,
    /// Gathers in a row (the chain; its EXP bonus and packet 618.)
    chain: u32,
    /// The slot gathered last (the window's tail.)
    last_slot: u8,
    pending: Option<PendingGather>,
}

/// A gatherer's view of the nodes: the client's phase table, what's left at each point, and what's been used at
/// the node they're at.
#[derive(Debug, Clone)]
pub struct GatheringState {
    /// One phase per slot, as sent at login (random, as retail's are.)
    pub phases: [u8; PHASE_SLOTS],
    /// Gathering attempts left at each point they've touched (a point not here has its full integrity.)
    remaining: HashMap<u32, u8>,
    /// Each point's phase slot and how many positions it has, from its spawns (SpawnObject args1's bytes 1 and 3.)
    slots: HashMap<u32, (u8, u8)>,
    /// Points whose hidden items Luck has shown.
    revealed: HashSet<u32>,
    /// Timed points used up, and when (Unix seconds): gone until their window has passed.
    exhausted_timed: HashMap<u32, u64>,
    /// Normal points used up, oldest first.
    exhausted: VecDeque<u32>,
    /// The node's own statuses on the gatherer and their params, until the window closes.
    buffs: HashMap<u16, u16>,
    visit: Option<NodeVisit>,
}

impl Default for GatheringState {
    fn default() -> Self {
        let mut phases = [0u8; PHASE_SLOTS];
        for phase in &mut phases {
            *phase = fastrand::u8(0..6);
        }
        Self {
            phases,
            remaining: HashMap::new(),
            slots: HashMap::new(),
            revealed: HashSet::new(),
            exhausted_timed: HashMap::new(),
            exhausted: VecDeque::new(),
            buffs: HashMap::new(),
            visit: None,
        }
    }
}

impl GatheringState {
    /// A point's spawn was sent: remember its phase slot and positions.
    pub fn saw_point(&mut self, point: u32, args1: u32) {
        let bytes = args1.to_le_bytes();
        self.slots.insert(point, (bytes[1], bytes[3]));
    }

    /// What's left at a point whose full integrity is `count`.
    pub fn remaining(&self, point: u32, count: u8) -> u8 {
        if self
            .exhausted_timed
            .get(&point)
            .is_some_and(|at| now() < at + TIMED_POINT_GONE_SECONDS)
        {
            return 0;
        }
        self.remaining.get(&point).copied().unwrap_or(count)
    }
}

/// What the window needs about a node.
struct Node {
    items: [(GatheringPointItem, (u32, u32)); 8],
    /// Its integrity, with the node's bonus.
    count: u8,
    flags: u32,
    gathering_type: u8,
    /// The node's bonus, if the gatherer meets it.
    bonus: Option<GatheringPointBonusInfo>,
}

impl Node {
    fn bonus_of(&self, types: [u8; 2]) -> u32 {
        self.bonus
            .filter(|bonus| types.contains(&bonus.bonus_type))
            .map_or(0, |bonus| bonus.bonus_value as u32)
    }

    fn boon_bonus(&self) -> u32 {
        self.bonus_of([22, 23])
    }

    fn yield_bonus(&self) -> u32 {
        self.bonus_of([14, 17])
    }
}

impl ZoneConnection {
    /// Sends the phase table (at login.)
    pub async fn send_gathering_phases(&mut self) {
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::GatheringPointPhases {
            phases: self.gathering.phases,
        });
        self.send_ipc_self(ipc).await;
    }

    fn gathering_node(&mut self, point: u32) -> Node {
        let parameters = self.base_parameters();
        let mut game_data = self.game_data.lock();
        let (_, _, count, bonus_ids, point_type) = game_data.get_gathering_point(point);
        let gathering_type = game_data.get_gathering_type(point);
        let items = game_data
            .get_gathering_point_items(point)
            .map(|item| (item, game_data.gathering_base_stats(item.level_row)));
        // The first bonus the gatherer meets (retail's nodes had one.)
        let bonus = bonus_ids.iter().find_map(|id| {
            game_data.get_gathering_point_bonus(*id).filter(|bonus| {
                gathering::bonus_met(
                    bonus.condition,
                    bonus.condition_value,
                    parameters.gathering,
                    parameters.perception,
                    parameters.gp,
                )
            })
        });
        let mut node = Node {
            items,
            count,
            flags: gathering::window_flags(gathering_type, point_type),
            gathering_type,
            bonus,
        };
        node.count = count.saturating_add(node.bonus_of([18, 19]) as u8);
        node
    }

    /// The node's window (scene 0, 60 params): the mode (`gathering::WINDOW_*`), `p1` (the point, or the slot gathered
    /// after a gather, or 0xFFFFFFFF after an action), the node's flags, [count, 0, left, 0], eight items of six words
    /// ([GatheringItem], [100, 0, level row (u16)], [chance, boon (255: none), yield, flags], base Gathering << 16, base
    /// Perception, 0), then [0, the bonus' value, 0, 0, attempts made, 0, 0x8000 | 0x100 with the bonus | last slot, 0].
    pub async fn send_gathering_window(&mut self, event: &Event, mode: u32, p1: u32) {
        let point = event.id.event_id();
        let node = self.gathering_node(point);
        let remaining = self.gathering.remaining(point, node.count);
        let parameters = self.base_parameters();
        let level = self.current_level(&self.game_data.lock()) as u8;
        let visit = self.gathering.visit.unwrap_or_default();

        let mut params = vec![
            mode,
            p1,
            node.flags,
            u32::from_le_bytes([node.count, 0, remaining, 0]),
        ];
        for (item, (base_gathering, base_perception)) in node.items {
            if item.gathering_id <= 0 {
                params.extend([0; 6]);
                continue;
            }
            let crystal = gathering::is_crystal(item.item_id as u32);
            let mut flags = GatheringItemFlag::empty();
            let hidden = item.hidden && !self.gathering.revealed.contains(&point);
            if hidden {
                flags.insert(GatheringItemFlag::HIDDEN);
            }
            if !crystal
                && !self
                    .player_data
                    .quest
                    .gathered_gathering_items
                    .contains(item.gathering_id as u32)
            {
                flags.insert(GatheringItemFlag::NOT_GATHERED_YET);
            }
            if self
                .gathering
                .buffs
                .contains_key(&gathering::STATUS_SHARP_VISION)
            {
                flags.insert(GatheringItemFlag::GATHERING_CHANCE_BONUS);
            }
            let chance = if hidden {
                0
            } else {
                gathering::gathering_chance(
                    parameters.gathering,
                    base_gathering,
                    parameters.perception,
                    item.perception_required as u32,
                    level,
                    item.level,
                    self.gathering_chance_bonus(),
                )
            };
            let boon = if crystal || item.no_boon {
                255
            } else {
                let boon_bonus = node.boon_bonus() + self.gift_bonus();
                if boon_bonus > 0 {
                    flags.insert(GatheringItemFlag::BOON_CHANCE_BONUS);
                }
                self.boon_chance(parameters.perception, base_perception, boon_bonus)
            };
            let yield_ = self.gathering_yield(&node, &item, parameters.gathering, base_gathering);
            params.extend([
                item.gathering_id as u32,
                100 | (item.level_row as u32) << 16,
                u32::from_le_bytes([
                    chance as u8,
                    boon as u8,
                    yield_.min(255) as u8,
                    flags.bits(),
                ]),
                base_gathering << 16,
                base_perception & 0xFFFF,
                0,
            ]);
        }
        let mut last = 0x8000 | visit.last_slot as u32;
        let bonus_value = node.bonus.map_or(0, |bonus| {
            last |= 0x100;
            bonus.bonus_value as u32
        });
        params.extend([0, bonus_value, 0, 0, visit.gathers, 0, last, 0]);

        self.event_scene(event, 0, SceneFlags::NO_DEFAULT_CAMERA, params)
            .await;
    }

    /// The window as the client asked for it after an action or a gather: a refresh, or the used-up one.
    pub async fn refresh_gathering_window(&mut self, event: &Event) {
        let point = event.id.event_id();
        let count = self.gathering_node(point).count;
        let mode = if self.gathering.remaining(point, count) == 0 {
            gathering::WINDOW_USED_UP
        } else {
            gathering::WINDOW_REFRESH
        };
        self.send_gathering_window(event, mode, point).await;
    }

    /// The window closing (the client's 11): scene 0 `[8, point, the node's flags' low half, count]`.
    pub async fn close_gathering_window(&mut self, event: &Event) {
        let point = event.id.event_id();
        let node = self.gathering_node(point);
        self.event_scene(
            event,
            0,
            SceneFlags::NO_DEFAULT_CAMERA,
            vec![8, point, node.flags & 0xFFFF, node.count as u32],
        )
        .await;
    }

    /// The gatherer opened a node.
    pub async fn open_gathering_point(&mut self, event: &Event) {
        let point = event.id.event_id();
        self.gathering.visit = Some(NodeVisit::default());
        self.send_gathering_window(event, gathering::WINDOW_OPEN, point)
            .await;
    }

    /// The gathering chance the node's statuses add: Sharp Vision's, and Clear Vision's for the next gather.
    fn gathering_chance_bonus(&self) -> u32 {
        let buffs = &self.gathering.buffs;
        buffs
            .get(&gathering::STATUS_SHARP_VISION)
            .copied()
            .unwrap_or(0) as u32
            + buffs
                .get(&gathering::STATUS_CLEAR_VISION)
                .copied()
                .unwrap_or(0) as u32
    }

    /// The Gifts' boon chance.
    fn gift_bonus(&self) -> u32 {
        let buffs = &self.gathering.buffs;
        buffs.get(&gathering::STATUS_GIFT_I).copied().unwrap_or(0) as u32
            + buffs.get(&gathering::STATUS_GIFT_II).copied().unwrap_or(0) as u32
    }

    fn boon_chance(&self, perception: u32, base_perception: u32, bonus: u32) -> u32 {
        (gathering::boon_chance(perception, base_perception) + bonus).min(100)
    }

    /// How many of an item one gather gives, before the boon: 1, the node's yield bonus, King's Yield's, Bountiful
    /// Yield's, and for crystals Twelve's Bounty's.
    fn gathering_yield(
        &self,
        node: &Node,
        item: &GatheringPointItem,
        gathering: u32,
        base_gathering: u32,
    ) -> u32 {
        let buffs = &self.gathering.buffs;
        let mut amount = 1 + buffs
            .get(&gathering::STATUS_KINGS_YIELD)
            .copied()
            .unwrap_or(0) as u32;
        if item.bonus_yield {
            amount += node.yield_bonus();
        }
        if buffs.contains_key(&gathering::STATUS_BOUNTIFUL_YIELD) {
            amount += 1;
        }
        if buffs.contains_key(&gathering::STATUS_BOUNTIFUL_YIELD_II) {
            amount += gathering::bountiful_bonus(gathering, base_gathering);
        }
        let item_id = item.item_id as u32;
        if gathering::is_crystal(item_id) && buffs.contains_key(&gathering::STATUS_TWELVES_BOUNTY) {
            let level = self.current_level(&self.game_data.lock());
            amount += if level >= 71 { 3 } else { 2 };
        }
        amount
    }

    /// The gatherer picked the item in `slot` (the window's yield 2): rolls the gather and returns scene 1's params
    /// (the animation, which shows the boon), or None if nothing can be gathered there.
    pub async fn gather(&mut self, event: &Event, slot: usize) -> Option<[u32; 2]> {
        let point = event.id.event_id();
        let node = self.gathering_node(point);
        let remaining = self.gathering.remaining(point, node.count);
        let (item, (base_gathering, base_perception)) = node.items.get(slot).copied()?;
        if remaining == 0 || item.gathering_id <= 0 {
            return None;
        }
        let parameters = self.base_parameters();

        let chance = gathering::gathering_chance(
            parameters.gathering,
            base_gathering,
            parameters.perception,
            item.perception_required as u32,
            self.current_level(&self.game_data.lock()) as u8,
            item.level,
            self.gathering_chance_bonus(),
        );
        let gathered = fastrand::u32(0..100) < chance;
        let mut amount = 0;
        let mut boon = false;
        if gathered {
            amount = self.gathering_yield(&node, &item, parameters.gathering, base_gathering);
            if !gathering::is_crystal(item.item_id as u32) && !item.no_boon {
                let chance = self.boon_chance(
                    parameters.perception,
                    base_perception,
                    node.boon_bonus() + self.gift_bonus(),
                );
                boon = fastrand::u32(0..100) < chance;
            }
            if boon {
                amount += 1;
                if self
                    .gathering
                    .buffs
                    .contains_key(&gathering::STATUS_TIDINGS)
                {
                    amount += 1;
                }
            }
        }

        // Each attempt costs one, gathered or not.
        self.gathering.remaining.insert(point, remaining - 1);
        let visit = self.gathering.visit.get_or_insert_default();
        visit.pending = Some(PendingGather {
            slot,
            item,
            amount,
            boon,
        });
        Some(gathering::gather_animation(node.gathering_type, boon))
    }

    /// The gather's animation ended (scene 1's yield): the item, the gathering log, EXP, the chain, the window (mode 4),
    /// the log lines and GP, as retail sent them.
    pub async fn finish_gather(&mut self, event: &Event) {
        let Some(visit) = self.gathering.visit.as_mut() else {
            return;
        };
        let Some(pending) = visit.pending.take() else {
            return;
        };
        visit.gathers += 1;
        visit.last_slot = pending.slot as u8;
        let chain = if pending.amount > 0 { visit.chain } else { 0 };
        visit.chain = if pending.amount > 0 {
            visit.chain + 1
        } else {
            0
        };

        let item = pending.item;
        let level = self.current_level(&self.game_data.lock()) as u8;
        if pending.amount > 0 {
            let info = self
                .game_data
                .lock()
                .get_item_info(ItemInfoQuery::ById(item.item_id as u32));
            if let Some(info) = info {
                self.player_data
                    .inventory
                    .add_obtained_item(Item::new(&info, pending.amount as i32));
            }
            self.send_inventory().await;

            let first_time = !gathering::is_crystal(item.item_id as u32)
                && !self
                    .player_data
                    .quest
                    .gathered_gathering_items
                    .contains(item.gathering_id as u32);
            if first_time {
                self.record_in_gathering_log(item.gathering_id as u32, item.item_id as u32)
                    .await;
            }

            let gathering_exp = self.game_data.lock().get_gathering_exp(level as u32);
            let exp = gathering::gather_exp(gathering_exp, level, item.level);
            let bonus = if pending.boon { 100 } else { 0 } + gathering::chain_bonus(chain);
            self.add_exp_with_bonus(exp, bonus).await;
            if first_time {
                self.add_exp(gathering::first_gather_exp(gathering_exp))
                    .await;
            }
            if chain > 0 {
                let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::GatheringChainCount {
                    count: chain,
                });
                self.send_ipc_self(ipc).await;
            }
        }

        // Every attempt wears the gear; one that gathered something bonds it too, ten times more at an unspoiled node
        // (`crate::spiritbond`; retail sent the worn pieces' slots after each gather.)
        let (_, _, _, _, point_type) = self
            .game_data
            .lock()
            .get_gathering_point(event.id.event_id());
        self.use_gear(crate::GearUse::Gather {
            node_level: item.level,
            rare: point_type == 2,
            gathered: pending.amount > 0,
        })
        .await;

        self.send_gathering_window(event, gathering::WINDOW_GATHERED, pending.slot as u32)
            .await;

        if pending.amount > 0 {
            for message_type in LOG_GATHERED {
                let message = EventLogMessage {
                    handler_id: event.id,
                    message_type,
                    params: vec![item.item_id as u32, pending.amount],
                };
                if let Some(ipc) = message.package() {
                    self.send_ipc_self(ipc).await;
                }
            }
        }

        // GP back, and the next-gather statuses are used up.
        self.handle
            .send(ToServer::RestoreResourcePoints(
                self.player_data.character.actor_id,
                gathering::gp_per_gather(level),
            ))
            .await;
        for status in [
            gathering::STATUS_CLEAR_VISION,
            gathering::STATUS_BOUNTIFUL_YIELD,
            gathering::STATUS_BOUNTIFUL_YIELD_II,
        ] {
            if self.gathering.buffs.remove(&status).is_some() {
                let actor_id = self.player_data.character.actor_id;
                self.lose_effect(status, 0, actor_id).await;
            }
        }
    }

    async fn record_in_gathering_log(&mut self, gathering_id: u32, item_id: u32) {
        let (value, index) = kawari::common::value_to_flag_byte_index_value_quests(gathering_id);
        self.player_data.quest.gathered_gathering_items.data[index as usize] ^= value;
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::RecordGatheringLog {
            index: index as u8,
            value: self.player_data.quest.gathered_gathering_items.data[index as usize],
        });
        self.send_ipc_self(ipc).await;
        self.actor_control_self(ActorControlCategory::LiveEvent {
            event: LiveEventType::RecordInGatheringLog { item_id },
        })
        .await;
    }

    /// Whether the point the gatherer is at is used up.
    pub fn gathering_point_used_up(&mut self, point: u32) -> bool {
        let count = self.gathering_node(point).count;
        self.gathering.remaining(point, count) == 0
    }

    /// The point was used up. A normal point stays hidden (the client hides points with nothing left) until two more
    /// have been used up; then it comes back with its integrity, at another of its positions (GatheringPointPhase) and
    /// with ActorControlSelf 109, as retail did. A timed point is gone until its next window.
    pub async fn gathering_point_exhausted(&mut self, point: u32) {
        let point_type = self.game_data.lock().get_gathering_point(point).4;
        if matches!(point_type, 2 | 4 | 5) {
            self.gathering.remaining.remove(&point);
            self.gathering.exhausted_timed.insert(point, now());
            return;
        }
        self.gathering.exhausted.retain(|p| *p != point);
        self.gathering.exhausted.push_back(point);
        while self.gathering.exhausted.len() > EXHAUSTED_POINTS_KEPT {
            let Some(back) = self.gathering.exhausted.pop_front() else {
                break;
            };
            self.gathering_point_back(back).await;
        }
    }

    async fn gathering_point_back(&mut self, point: u32) {
        self.gathering.remaining.remove(&point);
        if let Some((slot, positions)) = self.gathering.slots.get(&point).copied()
            && (slot as usize) < PHASE_SLOTS
            && positions > 1
        {
            // A new phase that shows another position (retail's were 0–5.)
            let old = self.gathering.phases[slot as usize];
            let mut phase = fastrand::u8(0..6);
            while phase % positions == old % positions {
                phase = fastrand::u8(0..6);
            }
            self.gathering.phases[slot as usize] = phase;
            let ipc =
                ServerZoneIpcSegment::new(ServerZoneIpcData::GatheringPointPhase { slot, phase });
            self.send_ipc_self(ipc).await;
        }
        let count = self.gathering_node(point).count as u32;
        self.actor_control_self(ActorControlCategory::Unknown {
            category: ACTOR_CONTROL_POINT_BACK,
            param1: kawari::common::HandlerId::new(
                kawari::common::HandlerType::GatheringPoint,
                point as u16,
            )
            .0,
            param2: 0,
            param3: count,
            param4: count,
            param5: 1,
        })
        .await;
    }

    /// The node's window closed (the client's 1): the chain ends (packet 618 with 0), the node's statuses come off.
    pub async fn leave_gathering_point(&mut self) {
        if self.gathering.visit.take().is_some() {
            let ipc =
                ServerZoneIpcSegment::new(ServerZoneIpcData::GatheringChainCount { count: 0 });
            self.send_ipc_self(ipc).await;
        }
        let actor_id = self.player_data.character.actor_id;
        for (status, _) in std::mem::take(&mut self.gathering.buffs) {
            self.lose_effect(status, 0, actor_id).await;
        }
    }

    /// An action picked in the node's window (the client's `[point, action, 3]`; it sends no ActionRequest there):
    /// handed to the action pipeline as if it had, which charges the GP, shows it and runs its script.
    pub async fn request_gathering_action(&mut self, action_id: u32) {
        let actor_id = self.player_data.character.actor_id;
        let request = ActionRequest {
            action_id,
            action_type: ActionType::Action,
            target: ObjectTypeId {
                object_id: actor_id,
                object_type: ObjectTypeKind::None,
            },
            ..Default::default()
        };
        self.handle
            .send(ToServer::ActionRequest(self.id, actor_id, request))
            .await;
    }

    /// A gatherer's action was used (its script queued it; the action pipeline already charged the GP.) At a node it
    /// changes what the next gathers give and the window is sent (mode 5, which the client answers with 5 for the
    /// refresh); away from one, Prospect, Truth and Sneak toggle their status.
    pub async fn use_gathering_action(
        &mut self,
        action_id: u32,
        player: &mut LuaPlayer,
        event: Option<&Event>,
    ) {
        if let Some(status) = gathering::field_action_status(action_id) {
            let actor_id = self.player_data.character.actor_id;
            if player.status_effects.get(status).is_some() {
                self.lose_effect(status, 0, actor_id).await;
            } else {
                self.gain_effect(status, 0, 0.0).await;
            }
            return;
        }
        let Some(action) = GatheringAction::from_id(action_id) else {
            return;
        };
        let Some(event) = event
            .filter(|event| event.id.handler_type() == kawari::common::HandlerType::GatheringPoint)
        else {
            return;
        };
        let point = event.id.event_id();
        match action {
            GatheringAction::SolidReason | GatheringAction::WiseToTheWorld => {
                let count = self.gathering_node(point).count;
                let remaining = self.gathering.remaining(point, count);
                self.gathering
                    .remaining
                    .insert(point, remaining.saturating_add(1));
                let actor_id = self.player_data.character.actor_id;
                if action == GatheringAction::WiseToTheWorld {
                    self.lose_effect(gathering::STATUS_EUREKA_MOMENT, 0, actor_id)
                        .await;
                } else if self.current_level(&self.game_data.lock()) >= 90 && fastrand::bool() {
                    // Enhanced Solid Reason: half the time, Eureka Moment (the trait's Value is 50.)
                    self.gain_effect(gathering::STATUS_EUREKA_MOMENT, 0, 0.0)
                        .await;
                }
            }
            GatheringAction::Luck => {
                self.gathering.revealed.insert(point);
            }
            _ => {}
        }
        if let Some((status, mut param)) = action.status() {
            // Gifts and Sharp Vision add up with what's there.
            if matches!(action, GatheringAction::SharpVision(_)) {
                param = param.max(self.gathering.buffs.get(&status).copied().unwrap_or(0));
            }
            if action == GatheringAction::BountifulYieldII {
                // Bountiful Yield II replaces the first one.
                self.gathering
                    .buffs
                    .remove(&gathering::STATUS_BOUNTIFUL_YIELD);
            }
            self.gathering.buffs.insert(status, param);
            // Retail's node statuses all had param 0 (what they do is the server's.)
            self.gain_effect(status, 0, 0.0).await;
        }
        self.send_gathering_window(event, gathering::WINDOW_AFTER_ACTION, u32::MAX)
            .await;
    }
}
