//! Air Force One (RideShooting, InstanceContent 62000-62005): the client flies the course and shoots, and tells the
//! server its score as the ride ends (`RideShootingScore`); the server answers with the result. As retail played it in
//! InstanceContent 62005 (Bob's Chronofoil captures deaf7e28 and 2c98f9e8, 2026-10-29, scores 4980 and 4930):
//! * The opening (EnterTerritory, scene 1) had flags 0x0C002001 and `[0, 0, 0, 5, 1793225644, 0, 0, 0, 0, 0, 3600,
//!   262152]`: the fifth was the same both times, an hour apart (see `OPENING_SCENE_VALUE`.) The duty asked its course
//!   first (client trigger 808 `[0]`, answered with `[0, its GFateRideShooting row]`, `server/director.rs`.)
//! * The score's answer: `RideShootingEnd` (the same 40 bytes both times), DirectorEvent `[0x40000002, 0]`,
//!   `[0x40000007, 0]` and `[0x40000007, 1]`, then a GameProgress event (argument 7) playing scene 11 (0x0C002001) with
//!   `[0, score, rank, 300, golds, 100, silvers, 50, bronzes, -20, ruby bombs, MGP]`, the MGP (LogMessage 4765, an
//!   InventoryActionAck, the slot) and Enemy at the GATE I. The player left from the result (client trigger 819.)

use kawari::{
    common::{DirectorEvent, HandlerId, HandlerType, ObjectTypeId, ObjectTypeKind},
    ipc::zone::{
        ActorControlCategory, EventType, SceneFlags, ServerZoneIpcData, ServerZoneIpcSegment,
    },
};

use crate::{
    Event, EventHandler, ZoneConnection, lua::LuaPlayer, zone_connection::COUNTER_GATES_COMPLETED,
};

/// The opening scene's flags for Air Force One (retail's, both runs.)
pub const OPENING_SCENE_FLAGS: u32 = 0x0C00_2001;
/// The opening scene's fifth parameter in Air Force One: 1793225644 both times, an hour apart (2026-10-28 22:14:04 UTC
/// as a time; what it is isn't known. Kawari's duties send 14400 there.)
pub const OPENING_SCENE_VALUE: u32 = 1_793_225_644;

/// `RideShootingEnd` as retail sent it, both times.
const RIDE_SHOOTING_END: [u8; 40] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x32, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x08, 0x0b, 0x2e,
    0x18, 0x0d, 0x10, 0x11, 0x14, 0x17, 0x35, 0x43, 0x56, 0x3a, 0x1e, 0x02, 0, 0, 0, 0, 0, 0,
];

/// The result: a GameProgress event with this argument, playing this scene.
const RESULT_EVENT_ARG: u32 = 7;
const RESULT_SCENE: u16 = 11;
const RESULT_SCENE_FLAGS: u32 = 0x0C00_2001;
/// The DirectorEvents before it.
const RESULT_EVENTS: [(u32, u32); 3] = [(0x4000_0002, 0), (0x4000_0007, 0), (0x4000_0007, 1)];

/// The ranks by the lowest score that makes them, and what they pay (the public wikis: Perfect 5000, S 4000, A 1500,
/// B 500, C below; retail paid 3,000 for both S runs.) The scene's rank is the place in this list (S was 1.)
const RANKS: [(u32, u32); 5] = [
    (5000, 4000),
    (4000, 3000),
    (1500, 1500),
    (500, 500),
    (0, 100),
];

/// The rank (its place in `RANKS`) and its MGP for a score.
pub fn ride_shooting_rank(score: u32) -> (u32, u32) {
    let rank = RANKS
        .iter()
        .position(|(lowest, _)| score >= *lowest)
        .unwrap_or(RANKS.len() - 1);
    (rank as u32, RANKS[rank].1)
}

/// The result scene's parameters: the targets' worth (RideShootingTargetType: gold, silver, bronze, the ruby bomb) and
/// how many of each were hit. The client sends the silvers, bronzes and ruby bombs; the golds are what's left of the
/// score (5 in both of retail's runs: 4980 = 5 × 300 + 14 × 100 + 42 × 50 - 20.)
pub fn ride_shooting_result(score: u32, hits: &[u8; 6], worth: &[i32]) -> Vec<u32> {
    let worth: Vec<i32> = (0..4)
        .map(|i| worth.get(i).copied().unwrap_or([300, 100, 50, -20][i]))
        .collect();
    let others: i32 = (0..3).map(|i| hits[i] as i32 * worth[i + 1]).sum();
    let golds = if worth[0] > 0 {
        ((score as i32 - others) / worth[0]).max(0)
    } else {
        0
    };
    let (rank, mgp) = ride_shooting_rank(score);
    vec![
        0,
        score,
        rank,
        worth[0] as u32,
        golds as u32,
        worth[1] as u32,
        hits[0] as u32,
        worth[2] as u32,
        hits[1] as u32,
        worth[3] as u32,
        hits[2] as u32,
        mgp,
    ]
}

impl ZoneConnection {
    /// The Air Force One duty the player is in, if any: its director.
    pub fn ride_shooting_director(&self) -> Option<HandlerId> {
        let director = self
            .content_handler_id
            .filter(|id| id.handler_type() == HandlerType::InstanceContent)?;
        self.game_data
            .lock()
            .ride_shooting_course(director.event_id() as u16)
            .map(|_| director)
    }

    /// The client's score as Air Force One ends: the result, once (the client sends it until it leaves.)
    pub async fn ride_shooting_score(
        &mut self,
        score: u16,
        hits: [u8; 6],
        events: &mut Vec<(Box<dyn EventHandler>, Event)>,
        lua_player: &mut LuaPlayer,
    ) {
        if self.ride_shooting_scored {
            return;
        }
        let Some(director) = self.ride_shooting_director() else {
            tracing::warn!("An Air Force One score ({score}) outside of Air Force One");
            return;
        };
        self.ride_shooting_scored = true;

        let worth = self.game_data.lock().ride_shooting_scores();
        let params = ride_shooting_result(score as u32, &hits, &worth);
        let mgp = *params.last().unwrap_or(&0);
        tracing::info!(
            "Air Force One: {score} points ({} golds, {:?}), {mgp} MGP",
            params[4],
            &hits[..3]
        );

        self.send_ipc_self(ServerZoneIpcSegment::new(
            ServerZoneIpcData::RideShootingEnd {
                unk: RIDE_SHOOTING_END,
            },
        ))
        .await;
        for (id, arg) in RESULT_EVENTS {
            self.actor_control_self(ActorControlCategory::DirectorEvent {
                handler_id: director,
                event: DirectorEvent::Unknown {
                    id,
                    arg1: arg,
                    arg2: 0,
                    arg3: 0,
                    arg4: 0,
                },
            })
            .await;
        }

        let started = self
            .start_event(
                ObjectTypeId {
                    object_id: self.player_data.character.actor_id,
                    object_type: ObjectTypeKind::None,
                },
                director,
                EventType::GameProgress,
                RESULT_EVENT_ARG,
                events,
                lua_player,
            )
            .await;
        if started && let Some((_, event)) = events.last() {
            let event = Event {
                id: event.id,
                event_type: event.event_type,
                event_arg: event.event_arg,
                condition: event.condition,
                actor_id: event.actor_id,
            };
            self.event_scene(
                &event,
                RESULT_SCENE,
                SceneFlags::from_bits_retain(RESULT_SCENE_FLAGS),
                params,
            )
            .await;
        }

        self.win_mgp_acknowledged(mgp).await;
        self.add_to_achievement_counter(COUNTER_GATES_COMPLETED, 1)
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retail_results() {
        let worth = [300, 100, 50, -20];
        // deaf7e28 and 2c98f9e8's scene 11.
        assert_eq!(
            ride_shooting_result(4980, &[14, 42, 1, 0, 0, 0], &worth),
            vec![
                0,
                4980,
                1,
                300,
                5,
                100,
                14,
                50,
                42,
                (-20i32) as u32,
                1,
                3000
            ]
        );
        assert_eq!(
            ride_shooting_result(4930, &[14, 41, 1, 0, 0, 0], &worth),
            vec![
                0,
                4930,
                1,
                300,
                5,
                100,
                14,
                50,
                41,
                (-20i32) as u32,
                1,
                3000
            ]
        );
        assert_eq!(ride_shooting_rank(5000), (0, 4000));
        assert_eq!(ride_shooting_rank(499), (4, 100));
    }
}
