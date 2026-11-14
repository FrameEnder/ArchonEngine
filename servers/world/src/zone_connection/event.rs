//! Handling all things related to the event system.

use crate::{
    Event, ToServer, ZoneConnection,
    event::{EventHandler, dispatch_event},
    lua::LuaPlayer,
};
use kawari::{
    common::{CharacterMode, HandlerId, HandlerType, ObjectTypeId},
    config::get_config,
    ipc::zone::{
        ActorControlCategory, Condition, EventResume, EventScene, EventStart, EventType,
        SceneFlags, ServerZoneIpcData, ServerZoneIpcSegment,
    },
};

/// How long after an event to see whether it moved the zone's layer filter on: long enough for a zone change it began
/// to have sent its PrepareZoning (retail warped at once, with the EventFinish.)
const LAYER_FILTER_CHECK_AFTER: std::time::Duration = std::time::Duration::from_millis(500);

impl ZoneConnection {
    /// Starts a scene for the current event.
    pub async fn event_scene(
        &mut self,
        event: &Event,
        scene: u16,
        mut scene_flags: SceneFlags,
        params: Vec<u32>,
    ) {
        let config = get_config();
        if config.tweaks.always_allow_skipping {
            scene_flags.remove(SceneFlags::DISABLE_SKIP);
        }

        // A cutscene puts them in it before it plays, a plain scene takes them out (retail sent the ActorControl 519,
        // 504 and SetOnlineStatus right before the cutscene's EventScene.)
        self.set_watching_cutscene(scene_flags.contains(SceneFlags::CONDITION_CUTSCENE))
            .await;

        let scene = EventScene {
            actor_id: event.actor_id,
            handler_id: event.id,
            scene,
            scene_flags,
            params,
            ..Default::default()
        };
        if let Some(ipc) = scene.package() {
            self.send_ipc_self(ipc).await;
        } else {
            tracing::error!(
                "Unable to package event scene {}, scene {:?}, scene_flags {scene_flags}!",
                event.id,
                scene
            );
        }
    }

    /// Into (`watching`) or out of a cutscene, as retail does around every scene with
    /// `SceneFlags::CONDITION_CUTSCENE` (Bob's Chronofoil captures 71045ef2, 689b15da and 09ecf8e7: every such scene, 41
    /// times in 71045ef2): the condition OccupiedInCutSceneEvent, ActorControl 519 `[the player, 3]` (or `1` when it's
    /// over) to their party from them, or ActorControlSelf alone, and Viewing Cutscene in their online statuses (their
    /// SetOnlineStatus, and ActorControl 504 with the icon to everyone around: 15, unless one ranked above it shows.)
    /// Nothing is sent when nothing changes.
    pub async fn set_watching_cutscene(&mut self, watching: bool) {
        if self.watching_cutscene == watching {
            return;
        }
        self.watching_cutscene = watching;
        if watching {
            self.conditions
                .set_condition(Condition::OccupiedInCutSceneEvent);
        } else {
            self.conditions
                .remove_condition(Condition::OccupiedInCutSceneEvent);
        }
        self.send_conditions().await;
        self.handle
            .send(ToServer::CutsceneState(
                self.player_data.character.actor_id,
                watching,
            ))
            .await;
        self.update_online_status().await;
    }

    /// Finishes the current event, including resetting any conditions set during the start of said event.
    pub async fn event_finish(&mut self, events: &mut Vec<(Box<dyn EventHandler>, Event)>) {
        if let Some(event) = events.pop() {
            // A trade window's items not taken by the event stay where they are.
            self.player_data.inventory.handed_in.clear();
            let event_type = event.1.event_type;
            let event_arg = event.1.event_arg;
            let event_id = event.1.id;

            // Back to normal from what the event put them in (crafting, gathering); an ordinary event put them in none.
            if event.0.character_mode() != CharacterMode::Normal {
                self.set_character_mode(CharacterMode::Normal, 0, true)
                    .await;
            }

            // Remove the condition given at the start of the event
            self.conditions.remove_condition(event.1.condition);
            for extra in event.0.extra_conditions() {
                self.conditions.remove_condition(extra);
            }
            self.send_conditions().await;
            // Out of a cutscene with the event (retail's ActorControl 519 came right after this Condition.)
            self.set_watching_cutscene(false).await;

            // sent event finish
            {
                let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::EventFinish {
                    handler_id: event_id,
                    event_type,
                    result: 1,
                    arg: event_arg,
                });
                self.send_ipc_self(ipc).await;
            }
        }

        if let Some(event) = events.last() {
            self.event_handler_id = Some(event.1.id);
        } else {
            self.event_handler_id = None;
            // Whether the event moved the zone's layer filter on, once it's had its effect (a zone change it began
            // cancels this with its PrepareZoning: the new zone's ZoneInit has its key.)
            if crate::layer_filters::has_filters(self.player_data.volatile.zone_id as u16) {
                self.layer_filter_check =
                    Some(std::time::Instant::now() + LAYER_FILTER_CHECK_AFTER);
            }
        }
    }

    /// Starts a new event. This can be nested, depending on the event type you chose. Returns true if the event was successfully found and scripted, otherwise flase.
    pub async fn start_event(
        &mut self,
        actor_id: ObjectTypeId,
        event_id: HandlerId,
        event_type: EventType,
        event_arg: u32,
        events: &mut Vec<(Box<dyn EventHandler>, Event)>,
        lua_player: &LuaPlayer,
    ) -> bool {
        let old_event_handler_id = self.event_handler_id;
        self.event_handler_id = Some(event_id);

        // Retail sheathes the player's weapon first, for every event.
        self.sheathe_weapon_for_event().await;

        // tell the client the event has started
        {
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::EventStart(EventStart {
                target_id: actor_id,
                handler_id: event_id,
                event_type,
                event_arg,
                flags: event_start_flags(event_id, event_type),
                ..Default::default()
            }));
            self.send_ipc_self(ipc).await;
        }

        // Find the ENpc or EObj base ID which is used in some scripts (or the battle NPC's a key item was used on.)
        let base_override = self.event_base_override.take();
        let base_id = base_override.or(lua_player
            .zone_data
            .cached_eobj_base_ids
            .get(&actor_id.object_id.0)
            .copied()
            .or(lua_player
                .zone_data
                .cached_npc_base_ids
                .get(&actor_id.object_id)
                .copied()));

        // call into the event dispatcher, get the event
        let handler = dispatch_event(
            event_id,
            base_id,
            self.game_data.clone(),
            self,
            actor_id.object_id,
        );

        if let Some(handler) = handler {
            let condition = handler.condition();
            let character_mode = handler.character_mode();

            self.conditions.toggle_condition(condition, true);
            for extra in handler.extra_conditions() {
                self.conditions.toggle_condition(extra, true);
            }
            self.send_conditions().await;

            // Only an event of its own (crafting, gathering) changes the player's mode. Retail sends no SetMode for an
            // ordinary event (a talk, a walk-in: Bob's Chronofoil captures 130088ec and 1fac2416), and a player who
            // rode into a walk-in event stayed on their mount through it. Kawari's SetMode Normal at every event's
            // start and end left the client mounted in Normal mode: it let him cast from the saddle, and talking to an
            // NPC no longer got him off first (Bob's bundle 20261110-120801.)
            if character_mode != CharacterMode::Normal {
                self.set_character_mode(character_mode, 0, true).await;
            }

            if handler.disable_event_pos_rollback() {
                self.actor_control_self(ActorControlCategory::DisableEventPosRollback {
                    handler_id: event_id,
                })
                .await;
            }

            events.push((
                handler,
                Event {
                    id: event_id,
                    event_type,
                    event_arg,
                    condition,
                    actor_id,
                },
            ));

            true
        } else {
            tracing::warn!("Event {:?} isn't scripted yet! Ignoring...", event_id);

            // give control back to the player so they aren't stuck
            {
                let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::EventFinish {
                    handler_id: event_id,
                    event_type,
                    result: 1,
                    arg: event_arg,
                });
                self.send_ipc_self(ipc).await;
            }

            self.send_notice(&format!(
                "Event {event_id} tried to start, but it doesn't have a script associated with it!"
            ))
            .await;

            self.event_handler_id = old_event_handler_id;

            // We have to send this again to ensure the client doesn't get stuck.
            self.send_conditions().await;

            false
        }
    }

    /// The client's yield 232: before the scene goes on, take the player off whatever they ride. The server answers
    /// it, not the event's script: off the mount if riding (`dismount_by_event`), then the scene resumed with nothing,
    /// as retail answered all six in Bob's and Connor's Chronofoil captures (five mounted: SetMode 1 and
    /// PlayDismountAnimation, then EventResume; one on foot: the resume.) Every Little Thing She Does Is Magitek's
    /// reaper comes off this way at its third event range (scene 27), a chocobo before 1190's and 1192's scenes. True
    /// if it was this yield.
    pub async fn answer_dismount_yield(
        &mut self,
        handler_id: HandlerId,
        scene: u16,
        yield_id: u8,
    ) -> bool {
        if yield_id != DISMOUNT_YIELD {
            return false;
        }
        self.dismount_by_event().await;
        self.resume_event(handler_id.0, scene, yield_id, Vec::new())
            .await;
        true
    }

    /// Resumes the current event.
    pub async fn resume_event(
        &mut self,
        event_id: u32,
        scene: u16,
        resume_id: u8,
        params: Vec<u32>,
    ) {
        let scene = EventResume {
            handler_id: HandlerId(event_id),
            scene,
            resume_id,
            params,
        };
        if let Some(ipc) = scene.package() {
            self.send_ipc_self(ipc).await;
        } else {
            tracing::error!(
                "Unable to package resume event {event_id}, scene {:?}!",
                scene
            );
        }
    }
}

/// The yield a client makes to be taken off its mount before a scene goes on (`answer_dismount_yield`.)
pub const DISMOUNT_YIELD: u8 = 232;

/// EventStart's flags byte. Retail sets it to 1 for the content directors' (InstanceContent, PublicContent,
/// QuestBattle) entering and progress events and a quest's entering event, 0 for every other event (Bob's Chronofoil
/// captures of 2026-10-21 and the release captures: 228 such events, none otherwise). What it does isn't known; a
/// quest battle's opening scene came without the UI hidden or a skip button while it was 0 (along with its scene
/// flags, see `QUEST_BATTLE_OPENING_FLAGS`.)
fn event_start_flags(event_id: HandlerId, event_type: EventType) -> u8 {
    let entering = event_type == EventType::EnterTerritory;
    let progress = event_type == EventType::GameProgress;
    let handler = event_id.handler_type();
    if (handler.is_content_director() && (entering || progress))
        || (handler == HandlerType::Quest && entering)
    {
        1
    } else {
        0
    }
}
