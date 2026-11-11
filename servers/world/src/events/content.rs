use async_trait::async_trait;
use kawari::{
    common::{HandlerType, ObjectTypeId},
    ipc::zone::{EventType, SceneFlags},
};

use crate::{Event, EventHandler, ToServer, ZoneConnection, lua::LuaPlayer};

/// A quest battle's start, played as its opening ends: EventStart 0x0d (GameProgress) on the battle with no argument,
/// then this scene with `QUEST_BATTLE_SCENE_FLAGS`, the client answering about 1.03 s later and the event finished then.
/// The fight is on from the scene's start (DirectorVars and HaterList at once; the first blow 0.05-2 s in): Bob's
/// Chronofoil captures, every one of 60 quest battle runs (19 battles, 2026-10-14 to 11-09).
pub const QUEST_BATTLE_SCENE_START: u16 = 60000;
/// The flags retail plays `QUEST_BATTLE_SCENE_START` and `QUEST_BATTLE_SCENE_COMPLETE` with.
pub const QUEST_BATTLE_SCENE_FLAGS: u32 = 0x40000;
/// The "duty complete" fanfare when winning a quest battle.
pub const QUEST_BATTLE_SCENE_COMPLETE: u16 = 60001;
/// Played after `QUEST_BATTLE_SCENE_COMPLETE`, before the player is sent back, unless the director picks another
/// (`complete_quest_battle`'s third argument; retail plays 6 in Leia's Legacy and 11 in To Guard a Guardian, 9 in
/// Spirithold Broken.)
pub const QUEST_BATTLE_SCENE_FINISH: u16 = 6;
/// The flags `QUEST_BATTLE_SCENE_FINISH` (or the director's pick) is played with, unless the director gives others:
/// HIDE_HOTBAR | NO_DEFAULT_CAMERA, as retail plays the last scene of Chasing Shadows, To Guard a Guardian and Leia's
/// Legacy. Spirithold Broken's is a cutscene (0xF8402EFB). From Bob's Chronofoil captures, 2026-10-14 and 16.
pub const QUEST_BATTLE_SCENE_FINISH_FLAGS: u32 = 0x2001;
/// A quest battle's answer to talking to someone in it (an event NPC the director brought out, whose handler is the
/// battle): the client's script logs EVENT_QIB_NOT_TALK, "now isn't the time" (Like Mother, Like Daughter's
/// ClsCnj005Btl_00440 has it, for Sylphie standing by.) Without a scene the event never ended.
pub const QUEST_BATTLE_SCENE_NOT_TALK: u16 = 10001;

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

/// Plays `QUEST_BATTLE_SCENE_NOT_TALK` in the talk event the player started.
pub fn play_not_talk(player: &mut LuaPlayer) {
    // Retail played it with 0x2000 (HIDE_HOTBAR) when Bob talked to In the Eyes of Gods and Men's onlooker (Chronofoil
    // capture 130088ec, three times); Kawari's has NO_DEFAULT_CAMERA too, which a message scene doesn't mind.
    player.play_scene(
        QUEST_BATTLE_SCENE_NOT_TALK,
        SceneFlags::HIDE_HOTBAR | SceneFlags::NO_DEFAULT_CAMERA,
        Vec::new(),
    );
}

#[async_trait]
impl EventHandler for ContentEventHandler {
    async fn on_talk(&self, event: &Event, _target_id: ObjectTypeId, player: &mut LuaPlayer) {
        if event.id.handler_type() == HandlerType::QuestBattle {
            play_not_talk(player);
        }
    }

    /// In a quest battle, the director decides: an event object it brought out (a rosary to pick up) is its own to
    /// answer, with no scene (retail's EventStart was followed at once by the director's event action, Bob's Chronofoil
    /// captures 130088ec and 3c56e09f); anything else is `play_not_talk` (`ToServer::QuestBattleTalk`.)
    async fn on_talk_connection(
        &self,
        event: &Event,
        target_id: ObjectTypeId,
        connection: &mut ZoneConnection,
        _player: &mut LuaPlayer,
    ) -> bool {
        if event.id.handler_type() != HandlerType::QuestBattle {
            return false;
        }
        connection
            .handle
            .send(ToServer::QuestBattleTalk(
                connection.player_data.character.actor_id,
                target_id.object_id,
            ))
            .await;
        true
    }

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
            // A quest battle's opening is followed by its start, as retail's always was (`QUEST_BATTLE_SCENE_START`);
            // its return is a GameProgress one, below.
            if event.id.handler_type() == HandlerType::QuestBattle {
                start_quest_battle(player, event.id.0);
                return;
            }
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
                    connection
                        .quest_battle_finish_scene
                        .unwrap_or(QUEST_BATTLE_SCENE_FINISH),
                    SceneFlags::from_bits_retain(connection.quest_battle_finish_flags),
                    Vec::new(),
                );
                return;
            } else if connection.quest_battle_finish_scene == Some(scene) {
                connection.quest_battle_finish_scene = None;
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
            } else if event.event_type == EventType::GameProgress {
                // One of the director's own scenes (`play_scene`), it carries on from here.
                player.finish_event();
                connection
                    .handle
                    .send(ToServer::DirectorSceneFinished(
                        connection.player_data.character.actor_id,
                        scene,
                    ))
                    .await;
                return;
            }
        }

        player.finish_event();
    }
}

/// The opening's event ends, and the quest battle's start (`QUEST_BATTLE_SCENE_START`) plays in an event of its own.
fn start_quest_battle(player: &mut LuaPlayer, handler: u32) {
    player.finish_event();
    player.start_event(handler, EventType::GameProgress, 0);
    player.play_scene(
        QUEST_BATTLE_SCENE_START,
        SceneFlags::from_bits_retain(QUEST_BATTLE_SCENE_FLAGS),
        Vec::new(),
    );
}

/// A ContentEntry event (0x2C0000): the question before a duty that's entered on the spot rather than queued for, the
/// GATEs' (Air Force One's 117, Leap of Faith's 91: Bob's Chronofoil capture 2c98f9e8.) Retail played its scene 0
/// (0x2000, no parameters), which returned `[2, 0]` to go; the duty (`GameData::content_entry_duty`) was then put in
/// the Duty Finder (ContentFinderUpdate, then ContentFinderFound once the events finished), and the player entered it
/// from there.
#[derive(Debug, Default)]
pub struct ContentEntryEventHandler;

impl ContentEntryEventHandler {
    pub const SCENE: u16 = 0;
    const ENTER: i32 = 2;
}

#[async_trait]
impl EventHandler for ContentEntryEventHandler {
    async fn on_talk(&self, _event: &Event, _target_id: ObjectTypeId, player: &mut LuaPlayer) {
        player.play_scene(Self::SCENE, SceneFlags::HIDE_HOTBAR, Vec::new());
    }

    async fn on_return(
        &self,
        event: &Event,
        connection: &mut ZoneConnection,
        _scene: u16,
        results: &[i32],
        player: &mut LuaPlayer,
    ) {
        if results.first() == Some(&Self::ENTER) {
            let duty = connection.game_data.lock().content_entry_duty(event.id.0);
            match duty {
                Some(duty) => {
                    // A GATE's: it's been played once this is answered.
                    if let Some(gate) = connection.current_gate() {
                        let entry = connection
                            .game_data
                            .lock()
                            .gate_content(gate.row.kind, gate.place)
                            .map(|(entry, _)| entry);
                        if entry == Some(event.id.0) {
                            connection.gate_joined = Some(gate.slot);
                        }
                    }
                    connection
                        .register_for_content(
                            [duty, 0, 0, 0, 0],
                            kawari::ipc::zone::DutyFinderSetting::default(),
                        )
                        .await;
                }
                None => tracing::warn!("ContentEntry {} has no duty Kawari knows", event.id),
            }
        }
        player.finish_event();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::LuaTask;

    /// Bob asked why Kawari didn't play retail's start of a quest battle: every one of 60 runs in his Chronofoil
    /// captures had EventStart 0x0d on the battle, with no argument, and scene 60000 with flags 0x40000 as the opening
    /// ended (Notorious Biggs' 7.3 s in, 689b15da), after the opening's EventFinish.
    #[test]
    fn a_quest_battle_starts_as_retail_s_did() {
        let mut player = LuaPlayer::default();
        start_quest_battle(&mut player, 0x8006003E);
        let tasks = &player.queued_tasks;
        assert!(matches!(tasks[0], LuaTask::FinishEvent));
        assert!(matches!(
            tasks[1],
            LuaTask::StartEvent {
                event_id: 0x8006003E,
                event_type: EventType::GameProgress,
                event_arg: 0,
            }
        ));
        assert!(matches!(
            &tasks[2],
            LuaTask::PlayScene { scene: 60000, scene_flags, params }
                if scene_flags.bits() == 0x40000 && params.is_empty()
        ));
    }
}
