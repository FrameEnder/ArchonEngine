use async_trait::async_trait;
use kawari::{
    common::HandlerType,
    ipc::zone::{EventType, SceneFlags},
};

use crate::{Event, EventHandler, ToServer, ZoneConnection, lua::LuaPlayer};

/// The "duty complete" fanfare when winning a quest battle.
pub const QUEST_BATTLE_SCENE_COMPLETE: u16 = 60001;
/// Played after `QUEST_BATTLE_SCENE_COMPLETE`, before the player is sent back.
pub const QUEST_BATTLE_SCENE_FINISH: u16 = 6;

/// For content director events.
/// This is mostly a dummy struct, most of the logic exists in the global server state.
#[derive(Debug)]
pub struct ContentEventHandler;

impl Default for ContentEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ContentEventHandler {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl EventHandler for ContentEventHandler {
    async fn on_return(
        &self,
        event: &Event,
        connection: &mut ZoneConnection,
        scene: u16,
        _results: &[i32],
        player: &mut LuaPlayer,
    ) {
        if event.event_type == EventType::EnterTerritory {
            connection
                .handle
                .send(ToServer::ReadyToCommence(
                    connection.player_data.character.actor_id,
                ))
                .await;
        }

        // After winning a quest battle (see FromServer::QuestBattleComplete), play its last scene and send them back.
        if event.id.handler_type() == HandlerType::QuestBattle {
            if scene == QUEST_BATTLE_SCENE_COMPLETE {
                // Some quest battles carry on a little after "Duty Complete" (e.g. lines after the boss falls), their
                // director sends the players back later.
                if connection.quest_battle_stay {
                    player.finish_event();
                    return;
                }
                player.play_scene(
                    QUEST_BATTLE_SCENE_FINISH,
                    SceneFlags::HIDE_HOTBAR | SceneFlags::NO_DEFAULT_CAMERA,
                    Vec::new(),
                );
                return;
            } else if scene == QUEST_BATTLE_SCENE_FINISH {
                player.finish_event();
                connection
                    .handle
                    .send(ToServer::LeaveContent(
                        connection.id,
                        connection.player_data.character.actor_id,
                        connection.old_zone_id,
                        connection.old_position,
                        connection.old_rotation,
                    ))
                    .await;
                return;
            }
        }

        player.finish_event();
    }
}
