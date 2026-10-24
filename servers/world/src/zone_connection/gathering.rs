//! A gatherer at a node: the window, each gather and the actions used there (`crate::gathering` has the rules.)

use std::collections::{HashMap, HashSet};

use kawari::ipc::zone::{
    ActorControlCategory, LiveEventType, SceneFlags, ServerZoneIpcData, ServerZoneIpcSegment,
};

use crate::{
    Event, GatheringItemFlag, ItemInfoQuery, ToServer, ZoneConnection,
    gathering::{self, GatheringAction, PHASE_SLOTS},
    inventory::Item,
    lua::LuaPlayer,
};

/// "The item was added to your inventory" sort of LogMessage, as upstream had it.
const LOG_ITEM_OBTAINED: u32 = 789;

/// How long a used-up timed point stays gone: four Eorzea hours, longer than any window (two or three) and no longer
/// than the shortest gap to the next one (the 01:00/05:00/09:00 points'.)
const TIMED_POINT_GONE_SECONDS: u64 = 4 * 175;

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

/// A gatherer's view of the nodes: the client's phase table, what's left at each point, and what's been used at
/// the node they're at.
#[derive(Debug, Clone)]
pub struct GatheringState {
    /// One phase per slot, as sent at login (random, as retail's are.)
    pub phases: [u8; PHASE_SLOTS],
    /// Gathering attempts left at each point they've touched (a point not here has its full integrity.)
    remaining: HashMap<u32, u8>,
    /// Each point's phase slot, from its spawns (SpawnObject args1's second byte.)
    slots: HashMap<u32, u8>,
    /// Points whose hidden items Luck has shown.
    revealed: HashSet<u32>,
    /// Timed points used up, and when (Unix seconds): gone until their window has passed.
    exhausted_timed: HashMap<u32, u64>,
    /// The node's own statuses on the gatherer and their params, until the window closes.
    buffs: HashMap<u16, u16>,
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
            buffs: HashMap::new(),
        }
    }
}

impl GatheringState {
    /// A point's spawn was sent: remember its phase slot.
    pub fn saw_point(&mut self, point: u32, args1: u32) {
        self.slots.insert(point, args1.to_le_bytes()[1]);
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

impl ZoneConnection {
    /// Sends the phase table (at login.)
    pub async fn send_gathering_phases(&mut self) {
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::GatheringPointPhases {
            phases: self.gathering.phases,
        });
        self.send_ipc_self(ipc).await;
    }

    /// The node's window (scene 0): `first` when it opens, otherwise a refresh. Params: 0 or 7, the event, a value
    /// retail sent (0x80000688, unknown), [count, ?, remaining, ?], then eight items of six words each: the
    /// GatheringItem, [?, ?, level, ?], [chance, HQ chance (255: none since Endwalker), yield, flags], then three words
    /// nobody has worked out (upstream's recorded scene; docs/src/fork/gathering.md.)
    pub async fn send_gathering_window(&mut self, event: &Event, first: bool) {
        let point = event.id.event_id();
        let (items, count) = {
            let mut game_data = self.game_data.lock();
            let (_, _, count, _, _) = game_data.get_gathering_point(point);
            let items = game_data.get_gathering_point_items(point);
            let items: Vec<_> = items
                .iter()
                .map(|item| (*item, game_data.gathering_base_stats(item.level_row)))
                .collect();
            (items, count)
        };
        let remaining = self.gathering.remaining(point, count);
        let parameters = self.base_parameters();

        let mut params = vec![
            if first { 0 } else { 7 },
            point,
            0x8000_0688,
            u32::from_le_bytes([count, 0, remaining, 0]),
        ];
        for (item, (base_gathering, _)) in items {
            let mut flags = GatheringItemFlag::from_bits_retain(4);
            let hidden = item.hidden && !self.gathering.revealed.contains(&point);
            if hidden {
                flags.insert(GatheringItemFlag::HIDDEN);
            }
            if item.gathering_id > 0
                && !self
                    .player_data
                    .quest
                    .gathered_gathering_items
                    .contains(item.gathering_id as u32)
            {
                flags.insert(GatheringItemFlag::NOT_GATHERED_YET);
            }
            let chance = if item.gathering_id <= 0 || hidden {
                0
            } else {
                gathering::gathering_chance(
                    parameters.gathering,
                    base_gathering,
                    parameters.perception,
                    item.perception_required as u32,
                    self.gathering_chance_bonus(),
                )
            };
            let yield_ =
                self.gathering_yield(item.item_id as u32, parameters.gathering, base_gathering);
            params.extend([
                item.gathering_id.max(0) as u32,
                u32::from_le_bytes([0, 0, item.level, 0]),
                u32::from_le_bytes([chance as u8, 255, yield_.min(255) as u8, flags.bits()]),
                0,
                0,
                0,
            ]);
        }

        self.event_scene(event, 0, SceneFlags::NO_DEFAULT_CAMERA, params)
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

    /// How many of an item one gather gives, before the boon: 1, King's Yield's, Bountiful Yield's, and for crystals
    /// Twelve's Bounty's.
    fn gathering_yield(&self, item_id: u32, gathering: u32, base_gathering: u32) -> u32 {
        let buffs = &self.gathering.buffs;
        let mut amount = 1 + buffs
            .get(&gathering::STATUS_KINGS_YIELD)
            .copied()
            .unwrap_or(0) as u32;
        if buffs.contains_key(&gathering::STATUS_BOUNTIFUL_YIELD) {
            amount += 1;
        }
        if buffs.contains_key(&gathering::STATUS_BOUNTIFUL_YIELD_II) {
            amount += gathering::bountiful_bonus(gathering, base_gathering);
        }
        if gathering::is_crystal(item_id) && buffs.contains_key(&gathering::STATUS_TWELVES_BOUNTY) {
            let level = self.current_level(&self.game_data.lock());
            amount += if level >= 71 { 3 } else { 2 };
        }
        amount
    }

    /// The gatherer gathers the item in `slot` (the window's yield.) Returns whether the point is now used up.
    pub async fn gather(&mut self, event: &Event, slot: usize) -> bool {
        let point = event.id.event_id();
        let (items, count, base_stats) = {
            let mut game_data = self.game_data.lock();
            let (_, _, count, _, _) = game_data.get_gathering_point(point);
            let items = game_data.get_gathering_point_items(point);
            let base = items
                .get(slot)
                .map(|item| game_data.gathering_base_stats(item.level_row))
                .unwrap_or_default();
            (items, count, base)
        };
        let remaining = self.gathering.remaining(point, count);
        let Some(item) = items.get(slot).copied() else {
            return remaining == 0;
        };
        if remaining == 0 || item.gathering_id <= 0 {
            return remaining == 0;
        }
        let (base_gathering, base_perception) = base_stats;
        let parameters = self.base_parameters();
        let level = self.current_level(&self.game_data.lock()) as u8;

        let chance = gathering::gathering_chance(
            parameters.gathering,
            base_gathering,
            parameters.perception,
            item.perception_required as u32,
            self.gathering_chance_bonus(),
        );
        let gathered = fastrand::u32(0..100) < chance;
        if gathered {
            let mut amount =
                self.gathering_yield(item.item_id as u32, parameters.gathering, base_gathering);
            let boon_chance = (gathering::boon_chance(parameters.perception, base_perception)
                + self
                    .gathering
                    .buffs
                    .get(&gathering::STATUS_GIFT_I)
                    .copied()
                    .unwrap_or(0) as u32
                + self
                    .gathering
                    .buffs
                    .get(&gathering::STATUS_GIFT_II)
                    .copied()
                    .unwrap_or(0) as u32)
                .min(100);
            if fastrand::u32(0..100) < boon_chance {
                amount += 1;
                if self
                    .gathering
                    .buffs
                    .contains_key(&gathering::STATUS_TIDINGS)
                {
                    amount += 1;
                }
            }

            let info = self
                .game_data
                .lock()
                .get_item_info(ItemInfoQuery::ById(item.item_id as u32));
            if let Some(info) = info {
                self.player_data
                    .inventory
                    .add_obtained_item(Item::new(&info, amount as i32));
            }
            self.send_inventory().await;

            // EXP: a stand-in until retail's is recorded: GatheringExp for the item's level, as much again the first
            // time (upstream gave 96, and 516 more the first time.)
            let exp = self.game_data.lock().get_gathering_exp(item.level as u32);
            let first_time = !self
                .player_data
                .quest
                .gathered_gathering_items
                .contains(item.gathering_id as u32);
            if first_time {
                self.record_in_gathering_log(item.gathering_id as u32, item.item_id as u32)
                    .await;
            }
            self.add_exp_with_bonus(if first_time { exp * 2 } else { exp }, 0)
                .await;

            self.actor_control_self(ActorControlCategory::LogMessage {
                log_message: LOG_ITEM_OBTAINED,
                id: item.item_id as u32,
            })
            .await;
        }

        // Each attempt costs one, gathered or not, gives back GP and uses up the next-gather statuses.
        let remaining = remaining - 1;
        self.gathering.remaining.insert(point, remaining);
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

        remaining == 0
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

    /// The point was used up: it moves to its next position (its phase slot steps on, which the client shows at
    /// once) with its integrity back, as normal nodes come back elsewhere on retail. A timed point is gone until its
    /// next window (its integrity stays at 0.)
    pub async fn gathering_point_exhausted(&mut self, point: u32) {
        let (base_id, level, count, bonus, point_type) =
            self.game_data.lock().get_gathering_point(point);
        self.gathering.remaining.remove(&point);
        if matches!(point_type, 2 | 4 | 5) {
            self.gathering.exhausted_timed.insert(point, now());
            return;
        }
        if let Some(slot) = self.gathering.slots.get(&point).copied()
            && (slot as usize) < PHASE_SLOTS
        {
            let phase = (self.gathering.phases[slot as usize] + 1) % 6;
            self.gathering.phases[slot as usize] = phase;
            let ipc =
                ServerZoneIpcSegment::new(ServerZoneIpcData::GatheringPointPhase { slot, phase });
            self.send_ipc_self(ipc).await;
        }
        self.actor_control_self(ActorControlCategory::SetupGatheringPoint {
            id: point,
            base_id: base_id as u32,
            level: level as u32,
            count: count as u32,
            remaining_count: count as u32,
            unk1: 0,
            gathering_point_bonus: bonus,
        })
        .await;
    }

    /// The node's window closed: its statuses come off.
    pub async fn leave_gathering_point(&mut self) {
        let actor_id = self.player_data.character.actor_id;
        for (status, _) in std::mem::take(&mut self.gathering.buffs) {
            self.lose_effect(status, 0, actor_id).await;
        }
    }

    /// A gatherer's action was used (its script queued it; the action pipeline already charged the GP.) At a node it
    /// changes what the next gathers give and the window is sent again; away from one, Prospect, Truth and Sneak
    /// toggle their status.
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
                let count = self.game_data.lock().get_gathering_point(point).2;
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
            self.gain_effect(status, param, 0.0).await;
        }
        self.send_gathering_window(event, false).await;
    }
}
