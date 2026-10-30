//! The Gold Saucer's lotteries and arcade machines, as retail played them in Bob's Chronofoil capture deaf7e28
//! (2026-10-29). The rules and numbers are in `gold_saucer.rs`; docs/src/fork/gold-saucer.md tells the whole story.
//!
//! * **The Mini Cactpot** (LotteryDaily, 0x250000): ActorControlSelf 109 (DirectorEvent) [28, done today] and [30,
//!   tickets left, the price], then scene 0 (0x2000) with [0x11, price] the first time ever, [0x13, price, MGP, day,
//!   tickets left] when there are tickets, [0x12] when there aren't. Buying is a yield (5): the MGP, then
//!   `LotteryDailyBoard` (the cell showing). Scratching is client trigger 808 [8, row, column], answered with 109 [2,
//!   row, column, number]; the line chosen is 808 [9, line]: 109 [3, row, column, number] for all nine cells, the MGP
//!   won (LogMessage 4765, an InventoryActionAck, the MGP's slot), the Gold Saucer flags (914), then 109 [4, MGP, sum].
//!   The scene returns [0] to buy another ([30, ...] and scene 0 again with 0x10) or nothing to leave.
//! * **The Jumbo Cactpot** (LotteryWeekly, 0x260000): 109 [29], then scene 9 with the week, the three tickets (10000
//!   for none) and the prizes. Each ticket is a yield (6) with the number: the MGP, the flags, LogMessage 4735 (the
//!   number, the price), then resume 20 [1, 0, the three tickets]. A drawn week's tickets are paid when the player next
//!   comes (not recorded on retail: Kawari tells each with LogMessage 4765, as the other prizes.)
//! * **Arcade machines** (GoldSaucerArcadeMachine, 0x24xxxx): scene 14 (0x2001); the coin is yield 7 (1 MGP), answered
//!   with resume 21 [1, the deadline]. Monster Toss, The Moogle's Paw, Crystal Tower Striker and Cuff-a-Cur send the
//!   result as yield 8 [tier]: 109 [1, the sheet's MGP], resume 22 [1, tier, that MGP twice, the tier's text, 1]; the
//!   MGP comes after the event finishes (LogMessage 4765, ActorControlSelf 910.) The Finer Miner and Out on a Limb
//!   are staged: yield 9 [choice] sets them up (resume 23), each swing is yield 10 [position] (resume 24), doubling
//!   down is yield 11 (resume 25), and a won round is paid when the player stops.

use async_trait::async_trait;
use kawari::{
    common::{
        ContainerType, DirectorEvent, DirectorTrigger, INVENTORY_ACTION_ACK_SHOP, ObjectTypeId,
        timestamp_secs,
    },
    ipc::zone::{ActorControlCategory, SceneFlags, ServerZoneIpcData, ServerZoneIpcSegment},
};

use crate::{
    Event, EventHandler, ZoneConnection,
    gold_saucer::{
        self, ARCADE_COST, ArcadeTier, JUMBO_CACTPOT_COSTS, JUMBO_CACTPOT_NO_TICKET,
        JUMBO_CACTPOT_PRIZES, JUMBO_CACTPOT_TICKETS, MGP_CAP, MINI_CACTPOT_COST,
        MINI_CACTPOT_PAYOUTS, MiniCactpotTicket, StagedGame,
    },
    inventory::{CurrencyKind, CurrencyStorage},
    lua::LuaPlayer,
    zone_connection::{
        COUNTER_GOLD_SAUCER_MGP, COUNTER_JUMBO_CACTPOT_MGP, COUNTER_MINI_CACTPOT_MGP,
    },
};

/// The MGP the player won (LogMessage 4765: "You obtain ... MGP.")
const LOG_MGP_OBTAINED: u32 = 4765;
/// A Jumbo Cactpot ticket bought (LogMessage 4735: its number and price.)
const LOG_JUMBO_TICKET: u32 = 4735;

/// What's being played right now at the Gold Saucer, kept on the connection while its event lasts.
#[derive(Debug, Clone)]
pub enum GoldSaucerGame {
    MiniCactpot(MiniCactpotTicket),
    /// A simple arcade machine's play, with what it pays once the event finishes.
    Arcade {
        payout: u32,
    },
    Staged(StagedGame),
    /// A GATE joined at its entrance, on its way to its duty through this ContentEntry event.
    GateEntry {
        content_entry: u32,
    },
}

fn flags() -> SceneFlags {
    SceneFlags::HIDE_HOTBAR
}

fn director_event(event: &Event, id: u32, args: [u32; 4]) -> ActorControlCategory {
    ActorControlCategory::DirectorEvent {
        handler_id: event.id,
        event: DirectorEvent::Unknown {
            id,
            arg1: args[0],
            arg2: args[1],
            arg3: args[2],
            arg4: args[3],
        },
    }
}

impl ZoneConnection {
    /// The player's MGP.
    pub fn mgp(&mut self) -> u32 {
        self.player_data
            .inventory
            .currency
            .get_item_for_id(CurrencyKind::MGP)
            .quantity
            .max(0) as u32
    }

    /// Takes or gives MGP (up to the cap) and sends its slot.
    pub async fn change_mgp(&mut self, amount: i64) {
        self.set_mgp_by(amount).await;
        // MGP won counts towards What Happens in the Saucer.
        if amount > 0 {
            self.add_to_achievement_counter(COUNTER_GOLD_SAUCER_MGP, amount as u32)
                .await;
        }
    }

    /// An MGP Voucher used (ItemAction type 3800, its MGP in Data[0]: 100 for the voucher from Scratch It Rich): one
    /// fewer, the log line and the MGP. Not won at an attraction, so it doesn't count towards What Happens in the
    /// Saucer. Retail's use wasn't recorded.
    pub async fn redeem_mgp_voucher(&mut self, item_id: u32, mgp: u32) {
        if self.player_data.inventory.count_in_bags(item_id) == 0 {
            tracing::warn!("MGP voucher {item_id} used, but there's none in the bags");
            return;
        }
        self.item_used_up(item_id).await;
        self.actor_control_self(ActorControlCategory::LogMessageActorNumber {
            log_message: LOG_MGP_OBTAINED,
            actor_id: self.player_data.character.actor_id,
            value: mgp,
        })
        .await;
        self.set_mgp_by(mgp as i64).await;
    }

    /// The MGP changed by `amount` (up to the cap), and its slot sent.
    async fn set_mgp_by(&mut self, amount: i64) {
        let slot = self
            .player_data
            .inventory
            .currency
            .get_item_for_id(CurrencyKind::MGP);
        // A slot with nothing in it still has to say it holds MGP.
        slot.item_id = CurrencyKind::MGP as u32;
        slot.quantity = (slot.quantity as i64 + amount).clamp(0, MGP_CAP as i64) as i32;
        self.send_inventory_slot(
            ContainerType::Currency,
            CurrencyStorage::get_slot_for_id(CurrencyKind::MGP),
        )
        .await;
    }

    /// MGP won: retail's log line (LogMessage 4765 with the player and the MGP), then the MGP.
    pub async fn win_mgp(&mut self, mgp: u32) {
        if mgp == 0 {
            return;
        }
        self.actor_control_self(ActorControlCategory::LogMessageActorNumber {
            log_message: LOG_MGP_OBTAINED,
            actor_id: self.player_data.character.actor_id,
            value: mgp,
        })
        .await;
        self.change_mgp(mgp as i64).await;
    }

    /// MGP won, with the InventoryActionAck retail sent between the log line and the MGP after a Mini Cactpot line and
    /// Air Force One.
    pub async fn win_mgp_acknowledged(&mut self, mgp: u32) {
        self.actor_control_self(ActorControlCategory::LogMessageActorNumber {
            log_message: LOG_MGP_OBTAINED,
            actor_id: self.player_data.character.actor_id,
            value: mgp,
        })
        .await;
        self.send_inventory_ack(u32::MAX, INVENTORY_ACTION_ACK_SHOP, 0)
            .await;
        self.change_mgp(mgp as i64).await;
    }

    /// The Gold Saucer flags (ActorControlSelf 914), as retail sent them at every zone change.
    pub async fn send_gold_saucer_flags(&mut self) {
        let flags = self
            .player_data
            .unlock
            .gold_saucer
            .flags(timestamp_secs() as u64);
        self.actor_control_self(ActorControlCategory::SetGoldSaucerFlags { flags })
            .await;
    }
}

// --- The Mini Cactpot ---

/// The Mini Cactpot broker (LotteryDaily.)
#[derive(Debug, Default)]
pub struct MiniCactpotEventHandler;

impl MiniCactpotEventHandler {
    const SCENE: u16 = 0;
    const YIELD_BUY: u8 = 5;
    const TRIGGER_SCRATCH: u32 = 8;
    const TRIGGER_LINE: u32 = 9;

    /// The broker's scene: `greeting` when the conversation begins (retail's 0x13), not after a ticket (0x10.)
    async fn offer(event: &Event, connection: &mut ZoneConnection, greeting: bool) {
        let now = timestamp_secs() as u64;
        let state = &mut connection.player_data.unlock.gold_saucer;
        state.refresh_mini_cactpot(now);
        let left = state.mini_cactpot_left() as u32;
        let played = state.mini_cactpot_played;
        let day = state.mini_cactpot_day;
        connection
            .actor_control_self(director_event(event, 30, [left, MINI_CACTPOT_COST, 0, 0]))
            .await;
        // The first number looks like flags on 0x10: 1 a greeting, 2 the game explained already. After it the price,
        // the MGP the player has, the day and the tickets left. The first talk ever is retail's `[0x11, 10]`: the
        // explanation, which ends the talk without a ticket (retail's player came back for one; Bob's bundles
        // 20261030-020833 and 20261030-102912, with the MGP and the rest sent after 0x11 or not, the same.)
        let params = if left == 0 {
            vec![0x12]
        } else if !played {
            vec![0x11, MINI_CACTPOT_COST]
        } else {
            let mgp = connection.mgp();
            let kind = if greeting { 0x13 } else { 0x10 };
            vec![kind, MINI_CACTPOT_COST, mgp, day, left]
        };
        connection
            .event_scene(event, Self::SCENE, flags(), params)
            .await;
    }

    async fn buy(event: &Event, connection: &mut ZoneConnection, player: &mut LuaPlayer) {
        let now = timestamp_secs() as u64;
        connection
            .player_data
            .unlock
            .gold_saucer
            .refresh_mini_cactpot(now);
        if connection
            .player_data
            .unlock
            .gold_saucer
            .mini_cactpot_left()
            == 0
            || connection.mgp() < MINI_CACTPOT_COST
        {
            tracing::warn!("Mini Cactpot: no ticket left or too little MGP");
            player.finish_event();
            return;
        }
        connection.change_mgp(-(MINI_CACTPOT_COST as i64)).await;
        let state = &mut connection.player_data.unlock.gold_saucer;
        state.mini_cactpot_bought += 1;
        let day = state.mini_cactpot_day;

        let ticket = MiniCactpotTicket::new();
        let shown = ticket.first_shown();
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::LotteryDailyBoard {
            handler_id: event.id,
            payouts_len: MINI_CACTPOT_PAYOUTS.len() as u8,
            last_sum: 23,
            day,
            row: (shown / 3) as u32,
            column: (shown % 3) as u32,
            number: ticket.numbers[shown] as u32,
            payouts: MINI_CACTPOT_PAYOUTS,
        });
        connection.send_ipc_self(ipc).await;
        connection.gold_saucer_game = Some(GoldSaucerGame::MiniCactpot(ticket));
    }
}

#[async_trait]
impl EventHandler for MiniCactpotEventHandler {
    async fn on_talk_connection(
        &self,
        event: &Event,
        _target_id: ObjectTypeId,
        connection: &mut ZoneConnection,
        _player: &mut LuaPlayer,
    ) -> bool {
        let now = timestamp_secs() as u64;
        connection
            .player_data
            .unlock
            .gold_saucer
            .refresh_mini_cactpot(now);
        let done = connection
            .player_data
            .unlock
            .gold_saucer
            .mini_cactpot_left()
            == 0;
        connection
            .actor_control_self(director_event(event, 28, [done as u32, 0, 0, 0]))
            .await;
        Self::offer(event, connection, true).await;
        // Retail's broker explained the game the first time only (0x11), whether or not a ticket was bought then.
        connection
            .player_data
            .unlock
            .gold_saucer
            .mini_cactpot_played = true;
        true
    }

    async fn on_yield(
        &self,
        event: &Event,
        connection: &mut ZoneConnection,
        scene: u16,
        yield_id: u8,
        _results: &[i32],
        player: &mut LuaPlayer,
    ) {
        if scene == Self::SCENE && yield_id == Self::YIELD_BUY {
            Self::buy(event, connection, player).await;
        }
    }

    async fn on_trigger(
        &self,
        event: &Event,
        connection: &mut ZoneConnection,
        trigger: &DirectorTrigger,
        _player: &mut LuaPlayer,
    ) {
        let DirectorTrigger::Unknown { id, unk1, unk2, .. } = *trigger else {
            return;
        };
        let Some(GoldSaucerGame::MiniCactpot(ticket)) = connection.gold_saucer_game.as_mut() else {
            return;
        };
        match id {
            Self::TRIGGER_SCRATCH => {
                let cell = (unk1 * 3 + unk2) as usize;
                let Some(number) = ticket.scratch(cell) else {
                    tracing::warn!("Mini Cactpot: can't scratch {unk1}, {unk2}");
                    return;
                };
                connection
                    .actor_control_self(director_event(event, 2, [unk1, unk2, number as u32, 0]))
                    .await;
            }
            Self::TRIGGER_LINE => {
                let numbers = ticket.numbers;
                let Some((sum, mgp)) = ticket.line(unk1 as usize) else {
                    return;
                };
                connection.gold_saucer_game = None;
                for (cell, number) in numbers.iter().enumerate() {
                    connection
                        .actor_control_self(director_event(
                            event,
                            3,
                            [(cell / 3) as u32, (cell % 3) as u32, *number as u32, 0],
                        ))
                        .await;
                }
                connection
                    .actor_control_self(ActorControlCategory::LogMessageActorNumber {
                        log_message: LOG_MGP_OBTAINED,
                        actor_id: connection.player_data.character.actor_id,
                        value: mgp,
                    })
                    .await;
                connection
                    .send_inventory_ack(u32::MAX, INVENTORY_ACTION_ACK_SHOP, 0)
                    .await;
                connection.change_mgp(mgp as i64).await;
                connection
                    .add_to_achievement_counter(COUNTER_MINI_CACTPOT_MGP, mgp)
                    .await;
                connection.send_gold_saucer_flags().await;
                connection
                    .actor_control_self(director_event(event, 4, [mgp, sum, 0, 0]))
                    .await;
                tracing::info!("Mini Cactpot: line {unk1} summed {sum}, {mgp} MGP");
            }
            _ => tracing::warn!("Mini Cactpot: unknown trigger {id}"),
        }
    }

    async fn on_return(
        &self,
        event: &Event,
        connection: &mut ZoneConnection,
        scene: u16,
        results: &[i32],
        player: &mut LuaPlayer,
    ) {
        connection.gold_saucer_game = None;
        // [0] buys another ticket.
        if scene == Self::SCENE
            && results.first() == Some(&0)
            && connection
                .player_data
                .unlock
                .gold_saucer
                .mini_cactpot_left()
                > 0
        {
            Self::offer(event, connection, false).await;
            return;
        }
        player.finish_event();
    }
}

// --- The Jumbo Cactpot ---

/// The Jumbo Cactpot broker (LotteryWeekly.)
#[derive(Debug, Default)]
pub struct JumboCactpotEventHandler;

impl JumboCactpotEventHandler {
    const SCENE: u16 = 9;
    const YIELD_BUY: u8 = 6;
    const RESUME_BOUGHT: u8 = 20;

    fn tickets(connection: &ZoneConnection) -> [u32; JUMBO_CACTPOT_TICKETS] {
        let now = timestamp_secs() as u64;
        let mut tickets = [JUMBO_CACTPOT_NO_TICKET; JUMBO_CACTPOT_TICKETS];
        for (slot, ticket) in connection
            .player_data
            .unlock
            .gold_saucer
            .jumbo_tickets_now(now)
            .iter()
            .enumerate()
            .take(JUMBO_CACTPOT_TICKETS)
        {
            tickets[slot] = *ticket as u32;
        }
        tickets
    }

    /// A drawn week's tickets, paid (not seen on retail, which hands them in one at a time at the broker.)
    async fn pay_drawn(connection: &mut ZoneConnection) {
        let now = timestamp_secs() as u64;
        let Some((week, tickets)) = connection
            .player_data
            .unlock
            .gold_saucer
            .take_drawn_jumbo_tickets(now)
        else {
            return;
        };
        let winning = gold_saucer::jumbo_cactpot_number(week);
        for ticket in tickets {
            let prize = gold_saucer::jumbo_cactpot_prize(ticket, winning);
            tracing::info!(
                "Jumbo Cactpot week {week}: ticket {ticket:04} against {winning:04}, {prize} MGP"
            );
            connection.win_mgp(prize).await;
            connection
                .add_to_achievement_counter(COUNTER_JUMBO_CACTPOT_MGP, prize)
                .await;
        }
    }
}

#[async_trait]
impl EventHandler for JumboCactpotEventHandler {
    async fn on_talk_connection(
        &self,
        event: &Event,
        _target_id: ObjectTypeId,
        connection: &mut ZoneConnection,
        _player: &mut LuaPlayer,
    ) -> bool {
        Self::pay_drawn(connection).await;
        connection
            .actor_control_self(director_event(event, 29, [0; 4]))
            .await;
        let now = timestamp_secs() as u64;
        let week = gold_saucer::week(now);
        let tickets = Self::tickets(connection);
        let last_winning = gold_saucer::jumbo_cactpot_number(week.saturating_sub(1)) as u32;
        // Retail's 21 parameters: 0, the week, the three tickets, last week's winning number, then numbers about the
        // participants and their bonus that aren't understood (sent as retail had them), and the five prizes.
        let mut params = vec![0, week];
        params.extend(tickets);
        params.extend([last_winning, 1, 29, 10125, 26, 7, 26, 24, 30, 0, 22]);
        params.extend(JUMBO_CACTPOT_PRIZES);
        connection
            .event_scene(event, Self::SCENE, flags(), params)
            .await;
        true
    }

    async fn on_yield(
        &self,
        event: &Event,
        connection: &mut ZoneConnection,
        scene: u16,
        yield_id: u8,
        results: &[i32],
        player: &mut LuaPlayer,
    ) {
        if scene != Self::SCENE || yield_id != Self::YIELD_BUY {
            return;
        }
        let now = timestamp_secs() as u64;
        let number = results.first().copied().unwrap_or(-1);
        let bought = connection
            .player_data
            .unlock
            .gold_saucer
            .jumbo_tickets_now(now)
            .len();
        let Some(price) = JUMBO_CACTPOT_COSTS.get(bought).copied() else {
            player.finish_event();
            return;
        };
        if !(0..10000).contains(&number) || connection.mgp() < price {
            tracing::warn!("Jumbo Cactpot: can't buy {number} for {price} MGP");
            player.finish_event();
            return;
        }
        connection.change_mgp(-(price as i64)).await;
        let state = &mut connection.player_data.unlock.gold_saucer;
        if state.jumbo_week != gold_saucer::week(now) {
            state.jumbo_week = gold_saucer::week(now);
            state.jumbo_tickets.clear();
        }
        state.jumbo_tickets.push(number as u16);
        connection.send_gold_saucer_flags().await;
        connection
            .actor_control_self(ActorControlCategory::LogMessageValues {
                log_message: LOG_JUMBO_TICKET,
                value1: number as u32,
                value2: price,
            })
            .await;
        let mut params = vec![1, 0];
        params.extend(Self::tickets(connection));
        connection
            .resume_event(event.id.0, Self::SCENE, Self::RESUME_BOUGHT, params)
            .await;
    }
}

// --- The arcade machines ---

/// An arcade machine (GoldSaucerArcadeMachine): its row is the event's ID.
#[derive(Debug, Default)]
pub struct ArcadeMachineEventHandler;

impl ArcadeMachineEventHandler {
    const SCENE: u16 = 14;
    const YIELD_COIN: u8 = 7;
    const YIELD_RESULT: u8 = 8;
    const YIELD_CHOICE: u8 = 9;
    const YIELD_SWING: u8 = 10;
    const YIELD_DOUBLE_DOWN: u8 = 11;
    const RESUME_STARTED: u8 = 21;
    const RESUME_RESULT: u8 = 22;
    const RESUME_SET_UP: u8 = 23;
    const RESUME_SWUNG: u8 = 24;
    const RESUME_DOUBLED: u8 = 25;
    /// The Finer Miner's and Out on a Limb's icons for the four tiers (retail's resume 24.)
    const FINER_MINER_ICONS: [u32; 4] = [4785, 4786, 4787, 4788];
    const OUT_ON_A_LIMB_ICONS: [u32; 4] = [4790, 4791, 4792, 4793];

    fn staged(machine: u32) -> bool {
        matches!(machine, 5 | 6)
    }

    /// The MGP won, after the event finished (ActorControlSelf 910 with the machine's source, as retail.)
    async fn pay(connection: &mut ZoneConnection, machine: u32, mgp: u32) {
        if mgp == 0 {
            return;
        }
        connection.win_mgp(mgp).await;
        connection
            .actor_control_self(ActorControlCategory::GoldSaucerMgpWon {
                source: gold_saucer::arcade_source(machine),
                mgp,
            })
            .await;
    }
}

#[async_trait]
impl EventHandler for ArcadeMachineEventHandler {
    async fn on_talk(&self, _event: &Event, _target_id: ObjectTypeId, player: &mut LuaPlayer) {
        player.play_scene(
            Self::SCENE,
            SceneFlags::HIDE_HOTBAR | SceneFlags::NO_DEFAULT_CAMERA,
            Vec::new(),
        );
    }

    async fn on_yield(
        &self,
        event: &Event,
        connection: &mut ZoneConnection,
        scene: u16,
        yield_id: u8,
        results: &[i32],
        player: &mut LuaPlayer,
    ) {
        if scene != Self::SCENE {
            return;
        }
        let machine = event.id.event_id();
        let Some(row) = connection.game_data.lock().get_arcade_machine(event.id.0) else {
            tracing::warn!("No GoldSaucerArcadeMachine row {}", event.id.0);
            player.finish_event();
            return;
        };
        let now = timestamp_secs() as u64;
        let result = results.first().copied().unwrap_or(0).max(0) as u32;
        match yield_id {
            Self::YIELD_COIN => {
                if connection.mgp() < ARCADE_COST {
                    player.finish_event();
                    return;
                }
                connection.change_mgp(-(ARCADE_COST as i64)).await;
                let deadline = (now + row.timeout) as u32;
                let params = if Self::staged(machine) {
                    vec![1, deadline, 2000]
                } else {
                    vec![1, deadline]
                };
                connection
                    .resume_event(event.id.0, scene, Self::RESUME_STARTED, params)
                    .await;
            }
            Self::YIELD_RESULT => {
                let tier = result.min(3) as ArcadeTier;
                let shown = row.values[tier as usize] as u32;
                let payout = gold_saucer::arcade_payout(machine, tier).unwrap_or(shown);
                connection.gold_saucer_game = Some(GoldSaucerGame::Arcade { payout });
                connection
                    .actor_control_self(director_event(event, 1, [shown, 0, 0, 0]))
                    .await;
                connection
                    .resume_event(
                        event.id.0,
                        scene,
                        Self::RESUME_RESULT,
                        vec![1, tier as u32, shown, shown, row.texts[tier as usize], 1],
                    )
                    .await;
            }
            Self::YIELD_CHOICE => {
                let game = StagedGame::new(machine, result.min(2));
                let deadline = (now + row.timeout) as u32;
                connection
                    .resume_event(
                        event.id.0,
                        scene,
                        Self::RESUME_SET_UP,
                        vec![
                            1,
                            game.hp as u32,
                            game.swings as u32,
                            game.gauge,
                            deadline,
                            5,
                            2000,
                            game.stake(),
                            row.timeout as u32,
                        ],
                    )
                    .await;
                connection.gold_saucer_game = Some(GoldSaucerGame::Staged(game));
            }
            Self::YIELD_SWING => {
                let Some(GoldSaucerGame::Staged(game)) = connection.gold_saucer_game.as_mut()
                else {
                    return;
                };
                let ranges = [row.ranges[1], row.ranges[2], row.ranges[3]];
                let tier = game.swing(result.min(99) as u8, ranges, row.values);
                let icons = if machine == 5 {
                    Self::FINER_MINER_ICONS
                } else {
                    Self::OUT_ON_A_LIMB_ICONS
                };
                let (over, round, hp) = (game.over, game.round, game.hp);
                let (stake, next) = if over {
                    (game.stake(), game.next())
                } else {
                    (0, 0)
                };
                if over {
                    connection
                        .actor_control_self(director_event(event, 1, [hp as u32, 0, 0, 0]))
                        .await;
                }
                connection
                    .resume_event(
                        event.id.0,
                        scene,
                        Self::RESUME_SWUNG,
                        vec![
                            1,
                            tier as u32,
                            hp as u32,
                            row.texts[tier as usize],
                            icons[tier as usize],
                            over as u32,
                            (round > 0) as u32,
                            stake,
                            next,
                        ],
                    )
                    .await;
            }
            Self::YIELD_DOUBLE_DOWN => {
                let Some(GoldSaucerGame::Staged(game)) = connection.gold_saucer_game.as_mut()
                else {
                    return;
                };
                if !game.can_double_down() {
                    return;
                }
                game.double_down();
                let (hp, gauge, stake) = (game.hp, game.gauge, game.stake());
                // Retail: Out on a Limb's clock went on (36 s left), The Finer Miner's started again.
                let left = if machine == 6 {
                    row.timeout as u32 / 2
                } else {
                    row.timeout as u32
                };
                connection
                    .resume_event(
                        event.id.0,
                        scene,
                        Self::RESUME_DOUBLED,
                        vec![
                            1,
                            hp as u32,
                            (now as u32).saturating_add(left),
                            gauge,
                            stake,
                            (machine == 5) as u32,
                            left,
                        ],
                    )
                    .await;
            }
            _ => tracing::warn!("Arcade machine {machine}: unknown yield {yield_id}"),
        }
    }

    async fn on_return(
        &self,
        event: &Event,
        connection: &mut ZoneConnection,
        _scene: u16,
        _results: &[i32],
        player: &mut LuaPlayer,
    ) {
        let machine = event.id.event_id();
        let payout = match connection.gold_saucer_game.take() {
            Some(GoldSaucerGame::Arcade { payout }) => payout,
            Some(GoldSaucerGame::Staged(game)) if game.won => game.stake(),
            _ => 0,
        };
        player.finish_event();
        // Retail paid after EventFinish; the finish is queued, so the payout follows it.
        player.queue_mgp_win(machine, payout);
    }
}

impl ZoneConnection {
    /// An arcade machine's payout, after its event finished (see `ArcadeMachineEventHandler::pay`.)
    pub async fn pay_arcade(&mut self, machine: u32, mgp: u32) {
        ArcadeMachineEventHandler::pay(self, machine, mgp).await;
    }
}

// --- The GATEs' entrances ---

/// A GATE's entrance (`gate.rs`): an event on the GATE's own director (GoldSaucer), which the client starts at the
/// entrance it shows. As retail played them (Bob's Chronofoil captures deaf7e28 and 2c98f9e8): scene 0 (0x2000) with
/// `[the kind's number (Air Force One 498, 501 once played), 0, MGP]`; joining is its yield `[1]`. An in-zone GATE
/// answers with DirectorEvent `[5, 1, start, registration window]` and the event finishes; an instanced one resumes the
/// scene with nothing, and when it returns plays the GATE's ContentEntry event, nested (scene 0), which puts the duty in
/// the Duty Finder (`ContentEntryEventHandler`.)
#[derive(Debug, Default)]
pub struct GateEventHandler;

impl GateEventHandler {
    const SCENE: u16 = 0;
    const JOIN: i32 = 1;
    /// The registration's DirectorEvent (see `zone_connection/gate.rs`.)
    const EVENT_JOINED: u32 = 5;

    fn gate(event: &Event, connection: &ZoneConnection) -> Option<crate::gate::Gate> {
        let now = timestamp_secs() as u64;
        connection
            .current_gate()
            .filter(|gate| gate.director() == event.id && gate.registering(now))
    }
}

#[async_trait]
impl EventHandler for GateEventHandler {
    async fn on_talk_connection(
        &self,
        event: &Event,
        _target_id: ObjectTypeId,
        connection: &mut ZoneConnection,
        player: &mut LuaPlayer,
    ) -> bool {
        let Some(gate) = Self::gate(event, connection) else {
            tracing::info!("GATE entrance {}: no GATE taking registrations", event.id);
            player.finish_event();
            return true;
        };
        let first = if connection.joined_gate(&gate) {
            gate.kind.entrance_played
        } else {
            gate.kind.entrance
        };
        let mgp = connection.mgp();
        connection
            .event_scene(event, Self::SCENE, flags(), vec![first, 0, mgp])
            .await;
        true
    }

    async fn on_yield(
        &self,
        event: &Event,
        connection: &mut ZoneConnection,
        scene: u16,
        _yield_id: u8,
        results: &[i32],
        player: &mut LuaPlayer,
    ) {
        let gate = Self::gate(event, connection);
        match gate {
            Some(gate)
                if results.first() == Some(&Self::JOIN) && !connection.joined_gate(&gate) =>
            {
                if gate.kind.instanced {
                    let content = connection
                        .game_data
                        .lock()
                        .gate_content(gate.row.kind, gate.place);
                    let Some((content_entry, _)) = content else {
                        tracing::warn!("GATE row {} has no duty to go to", gate.row.row);
                        player.finish_event();
                        return;
                    };
                    connection.gold_saucer_game = Some(GoldSaucerGame::GateEntry { content_entry });
                    connection
                        .resume_event(event.id.0, scene, 0, Vec::new())
                        .await;
                } else {
                    connection.gate_joined = Some(gate.slot);
                    connection
                        .actor_control_self(director_event(
                            event,
                            Self::EVENT_JOINED,
                            [1, gate.start as u32, gate.kind.registration as u32, 0],
                        ))
                        .await;
                    player.finish_event();
                }
            }
            _ => player.finish_event(),
        }
    }

    async fn on_return(
        &self,
        _event: &Event,
        connection: &mut ZoneConnection,
        scene: u16,
        _results: &[i32],
        player: &mut LuaPlayer,
    ) {
        // Its ContentEntry finished: this one finishes with it.
        if scene == 255 {
            return;
        }
        if let Some(GoldSaucerGame::GateEntry { content_entry }) =
            connection.gold_saucer_game.take()
        {
            player.queued_tasks.push(crate::lua::LuaTask::StartEvent {
                event_id: content_entry,
                event_type: kawari::ipc::zone::EventType::Nest,
                event_arg: 0,
            });
            player.play_scene(
                crate::ContentEntryEventHandler::SCENE,
                SceneFlags::HIDE_HOTBAR,
                Vec::new(),
            );
            return;
        }
        player.finish_event();
    }
}
