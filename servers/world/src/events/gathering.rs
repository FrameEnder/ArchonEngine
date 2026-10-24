use async_trait::async_trait;
use bitflags::bitflags;
use kawari::{
    common::{CharacterMode, ObjectTypeId},
    ipc::zone::{ActorControlCategory, Condition, LiveEventType, SceneFlags},
};

use crate::{Event, EventHandler, ZoneConnection, lua::LuaPlayer};

/// What the client sends from the node's window (scene 0's yields `[point, slot or action, what]`; Bob's capture of
/// 2026-10-24.)
const CLIENT_QUIT: i32 = 1;
const CLIENT_GATHER: i32 = 2;
const CLIENT_ACTION: i32 = 3;
const CLIENT_AFTER_GATHER: i32 = 4;
const CLIENT_AFTER_ACTION: i32 = 5;
const CLIENT_OPENED: i32 = 6;
const CLIENT_CLOSING: i32 = 11;

/// For gathering events: the node's window and what's done there (`zone_connection/gathering.rs`.)
#[derive(Debug)]
pub struct GatheringEventHandler {}

impl GatheringEventHandler {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for GatheringEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Default)]
pub struct GatheringItemFlag(u8);

bitflags! {
    impl GatheringItemFlag : u8 {
        /// Sharp Vision is up.
        const GATHERING_CHANCE_BONUS = 1;
        /// The node's boon bonus or a Gift is up.
        const BOON_CHANCE_BONUS = 2;
        const UNK1 = 4;
        const HIDDEN = 8;
        const RARE = 16;
        const BONUS = 32;
        const NOT_GATHERED_YET = 64;
        const UNK2 = 128;
    }
}

impl std::fmt::Debug for GatheringItemFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}

#[async_trait]
impl EventHandler for GatheringEventHandler {
    async fn on_talk(&self, _event: &Event, _target_id: ObjectTypeId, player: &mut LuaPlayer) {
        // The window needs the connection (what's left, the gatherer's stats), so it's sent from there.
        player.open_gathering_window();
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
        let point = event.id.event_id();
        let result = |i: usize| results.get(i).copied().unwrap_or_default();

        if scene == 1 {
            // The gather's animation ended.
            connection.finish_gather(event).await;
            return;
        }

        match result(2) {
            CLIENT_GATHER => match connection.gather(event, result(1).max(0) as usize).await {
                Some(animation) => {
                    player.play_scene(1, SceneFlags::NO_DEFAULT_CAMERA, animation.to_vec())
                }
                None => connection.refresh_gathering_window(event).await,
            },
            CLIENT_OPENED | CLIENT_AFTER_GATHER | CLIENT_AFTER_ACTION => {
                connection.refresh_gathering_window(event).await;
            }
            CLIENT_ACTION => {
                connection
                    .request_gathering_action(result(1).max(0) as u32)
                    .await;
            }
            CLIENT_CLOSING => {
                connection.close_gathering_window(event).await;
            }
            CLIENT_QUIT => {
                let used_up = connection.gathering_point_used_up(point);
                connection.leave_gathering_point().await;
                // Retail cleared the tool (LiveEvent 39 with nothing.)
                connection
                    .broadcast_actor_control(ActorControlCategory::LiveEvent {
                        event: LiveEventType::SetOffHand {
                            model_id: 0,
                            unk1: 0,
                            unk2: 0,
                        },
                    })
                    .await;
                if used_up {
                    connection.gathering_point_exhausted(point).await;
                }
                player.finish_event();
            }
            _ => {
                tracing::info!("Unknown gathering yield {results:?}");
            }
        }
    }

    fn condition(&self) -> Condition {
        Condition::ExecutingGatheringAction
    }

    /// Retail set 63 too while at a node (Kawari calls it WaitingToVisitOtherWorld.)
    fn extra_conditions(&self) -> Vec<Condition> {
        vec![Condition::WaitingToVisitOtherWorld]
    }

    fn character_mode(&self) -> CharacterMode {
        CharacterMode::Gathering
    }
}
