use std::sync::Arc;

use async_trait::async_trait;
use parking_lot::Mutex;

use kawari::{
    common::{CharacterMode, DirectorTrigger, HandlerId, HandlerType, ObjectId, ObjectTypeId},
    config::get_config,
    ipc::zone::{Condition, EventType, SceneFlags},
};

use crate::{
    ArcadeMachineEventHandler, ContentEntryEventHandler, ContentEventHandler, CraftingEventHandler,
    FateEventHandler, FishingEventHandler, GameData, GateEventHandler, GatheringEventHandler,
    GimmickAccessorEventHandler, GimmickBillEventHandler, GimmickRectEventHandler,
    GrandCompanyShopEventHandler, InclusionShopEventHandler, JumboCactpotEventHandler,
    LuaEventHandler, MaterializeEventHandler, MiniCactpotEventHandler, ShopEventHandler,
    SpecialShopEventHandler, ZoneConnection,
};

use super::lua::LuaPlayer;

#[derive(Debug)]
pub struct Event {
    pub id: HandlerId,
    pub event_type: EventType,
    pub event_arg: u32,
    /// The condition set for this event.
    pub condition: Condition,
    /// The actor associated with this event. This is usually an NPC for Talk events, otherwise the player who initiated it.
    pub actor_id: ObjectTypeId,
}

/// Abstract event handler that can be implemented in Lua or Rust.
#[allow(unused)]
#[async_trait]
pub trait EventHandler: std::fmt::Debug + Send + Sync {
    async fn on_enter_territory(&self, event: &Event, player: &mut LuaPlayer) {}

    async fn on_enter_trigger(&self, event: &Event, player: &mut LuaPlayer, arg: u32) {}

    /// An event action started with `event_action` finished casting.
    async fn on_event_action_cast(&self, event: &Event, player: &mut LuaPlayer) {}

    async fn on_talk(&self, event: &Event, target_id: ObjectTypeId, player: &mut LuaPlayer) {}

    /// Talked to, with the connection at hand, for handlers that tell the client more than a scene first (the Gold
    /// Saucer's lotteries.) True if it was handled, so `on_talk` isn't run.
    async fn on_talk_connection(
        &self,
        event: &Event,
        target_id: ObjectTypeId,
        connection: &mut ZoneConnection,
        player: &mut LuaPlayer,
    ) -> bool {
        false
    }

    /// A client trigger 808 (DirectorTrigger) for this event's handler: the Mini Cactpot's scratches and line.
    async fn on_trigger(
        &self,
        event: &Event,
        connection: &mut ZoneConnection,
        trigger: &DirectorTrigger,
        player: &mut LuaPlayer,
    ) {
    }

    /// A key item of this quest was used on `target_id` (the event was started with type EventItem.)
    async fn on_event_item(
        &self,
        event: &Event,
        item_id: u32,
        target_id: ObjectTypeId,
        player: &mut LuaPlayer,
    ) {
    }

    /// The player used an emote on `target_id` (the event was started with type Emote, its argument the emote.)
    async fn on_emote(
        &self,
        event: &Event,
        emote: u32,
        target_id: ObjectTypeId,
        player: &mut LuaPlayer,
    ) {
    }

    /// An action (its Action row) was used on the event's NPC or object (an ActionResult event.)
    async fn on_action_result(
        &self,
        event: &Event,
        action: u32,
        target_id: ObjectTypeId,
        player: &mut LuaPlayer,
    ) {
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
    }

    /// The default implementation will always finish the event.
    async fn on_return(
        &self,
        event: &Event,
        connection: &mut ZoneConnection,
        scene: u16,
        results: &[i32],
        player: &mut LuaPlayer,
    ) {
        // TODO: Check back later and see if we're OK with this somewhat magical behavior
        player.finish_event();
    }

    /// Condition to set (or unset) for this handler.
    fn condition(&self) -> Condition {
        Condition::OccupiedInQuestEvent
    }

    /// Other conditions set while this handler's event runs (and unset when it finishes.)
    fn extra_conditions(&self) -> Vec<Condition> {
        Vec::new()
    }

    /// CharacterMode to set (or unset) for this handler.
    fn character_mode(&self) -> CharacterMode {
        CharacterMode::Normal
    }

    /// Whether we should send the DisableEventPosRollback ActorControl.
    fn disable_event_pos_rollback(&self) -> bool {
        false
    }

    /// Whether this event goes on when an event nested in it finishes (retail's Jonathas plays his menu again after his
    /// shop closes), rather than finishing with it. Either way it's told with scene 255.
    fn continues_after_nested(&self) -> bool {
        false
    }
}

/// Dummied out EventHandler, which can be useful for certain handlers that don't need any logic.
#[derive(Debug)]
pub struct DummyEventHandler {}

impl DummyEventHandler {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl EventHandler for DummyEventHandler {}

/// A DefaultTalk (an NPC's line) the client asks the server for, e.g. a guard with character data flags 0x4. Retail
/// answers with scene 0, flags 0x2001 (HIDE_HOTBAR | NO_DEFAULT_CAMERA), no parameters; the client shows the line and
/// returns the scene, and the event finishes (Bob's Chronofoil capture of 2026-10-21, the Horizon guards' DefaultTalks
/// 1838 and 1839). Without the scene the client waited for one forever. A DefaultTalk nested in another event (SwitchTalk)
/// has its scene played by that event's script instead: `on_talk` only runs for the client's own talk.
#[derive(Debug)]
pub struct DefaultTalkEventHandler {}

#[async_trait]
impl EventHandler for DefaultTalkEventHandler {
    async fn on_talk(&self, _event: &Event, _target_id: ObjectTypeId, player: &mut LuaPlayer) {
        player.play_scene(
            0,
            SceneFlags::HIDE_HOTBAR | SceneFlags::NO_DEFAULT_CAMERA,
            Vec::new(),
        );
    }
}

/// Extracts the script id from a given CustomTalk `name`. For example, "CmnDefBeginnerGuide_00327" will return 327.
fn extract_script_id(name: &str) -> Result<u32, std::num::ParseIntError> {
    name[name.len() - 5..].parse()
}

/// Creates the proper folder name from a given script id. For example, 327 will return 003.
fn folder_from_script_id(id: u32) -> String {
    format!("{:03}", (id / 100))
}

/// Where a quest's script lives, such as `events/quest/038/XxaFst034_03854.lua`.
pub fn quest_script_path(
    game_data: &Arc<Mutex<GameData>>,
    handler_id: HandlerId,
) -> Option<String> {
    let script_name = game_data.lock().get_quest_name(handler_id.0);
    let Ok(script_id) = extract_script_id(&script_name) else {
        tracing::error!(
            "Unable to obtain script id from script {script_name}, the event cannot continue!"
        );
        return None;
    };

    let script_folder = folder_from_script_id(script_id);
    let path = format!("events/quest/{script_folder}/{script_name}.lua");
    let config = get_config();
    if std::fs::exists(config.filesystem.locate_script_file(&path)).unwrap_or_default() {
        return Some(path);
    }

    // Otherwise a script for the same quest number under another name: one written from a retail capture before
    // the client's script (and so its name) was dumped, e.g. a quest newer than the references.
    let suffix = format!("_{script_id:05}.lua");
    let folders = config
        .filesystem
        .additional_resource_paths
        .iter()
        .map(|x| format!("{x}/scripts/events/quest/{script_folder}"))
        .chain(std::iter::once(format!(
            "resources/scripts/events/quest/{script_folder}"
        )));
    for folder in folders {
        let Ok(entries) = std::fs::read_dir(folder) else {
            continue;
        };
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.ends_with(&suffix) {
                return Some(format!("events/quest/{script_folder}/{file_name}"));
            }
        }
    }

    Some(path)
}

/// Finds and creates the relevant `EventHandler` for this event.
pub fn dispatch_event(
    handler_id: HandlerId,
    base_id: Option<u32>,
    game_data: Arc<Mutex<GameData>>,
    connection: &ZoneConnection,
    actor_id: ObjectId,
) -> Option<Box<dyn EventHandler>> {
    let generic_lua_event = |path: &str| -> Option<Box<dyn EventHandler>> {
        if let Some(event) = LuaEventHandler::new(handler_id, base_id, path, game_data.clone()) {
            Some(Box::new(event))
        } else {
            tracing::warn!("{path} was not found!");
            None
        }
    };

    match handler_id.handler_type() {
        HandlerType::Quest => {
            let script_path = quest_script_path(&game_data, handler_id)?;
            generic_lua_event(&script_path)
        }
        HandlerType::Shop => Some(Box::new(ShopEventHandler::new())),
        HandlerType::Warp => {
            let warp_name;
            {
                let mut game_data = game_data.lock();
                warp_name = game_data.get_warp_logic_name(handler_id.0);
            }

            if warp_name.is_empty() {
                generic_lua_event("events/generic/Warp.lua")
            } else {
                let script_path = format!("events/warp/{warp_name}.lua");
                generic_lua_event(&script_path)
            }
        }
        HandlerType::GatheringPoint => Some(Box::new(GatheringEventHandler::new())),
        HandlerType::Aetheryte => {
            let aetheryte_id = handler_id.event_id();

            // Aetherytes and Aethernet shards are handled by different event scripts
            let is_aetheryte;
            {
                let mut game_data = game_data.lock();
                is_aetheryte = game_data.is_aetheryte(aetheryte_id);
            }

            if is_aetheryte {
                generic_lua_event("events/generic/Aetheryte.lua")
            } else {
                generic_lua_event("events/generic/AethernetShard.lua")
            }
        }
        // Mostly played by the client itself; the server is asked for one by battle NPCs with a talk (guards) and nested
        // in SwitchTalk.
        HandlerType::DefaultTalk => Some(Box::new(DefaultTalkEventHandler {})),
        HandlerType::Craft => Some(Box::new(CraftingEventHandler::new())),
        HandlerType::Materialize => Some(Box::new(MaterializeEventHandler)),
        HandlerType::GuildLeveAssignment => generic_lua_event("events/generic/Levemete.lua"),
        HandlerType::CustomTalk => {
            let script_name;
            {
                let mut game_data = game_data.lock();
                script_name = game_data.get_custom_talk_name(handler_id.0);
            }
            let Ok(script_id) = extract_script_id(&script_name) else {
                tracing::error!(
                    "Unable to obtain script id from script {script_name}, the event cannot continue!"
                );
                return None;
            };

            let script_folder = folder_from_script_id(script_id);
            let script_path = format!("events/custom/{script_folder}/{script_name}.lua");

            generic_lua_event(&script_path)
        }
        HandlerType::GimmickAccessor => Some(Box::new(GimmickAccessorEventHandler::new())),
        HandlerType::GimmickBill => Some(Box::new(GimmickBillEventHandler::new())),
        HandlerType::GimmickRect => Some(Box::new(GimmickRectEventHandler::new())),
        HandlerType::ChocoboTaxiStand => generic_lua_event("events/generic/Chocobokeep.lua"),
        HandlerType::Opening => {
            let script_name;
            {
                let mut game_data = game_data.lock();
                script_name = game_data.get_opening_name(handler_id.0);
            }

            generic_lua_event(&format!("events/quest/opening/{script_name}.lua"))
        }
        HandlerType::ExitRange => generic_lua_event("events/generic/ExitRange.lua"),
        HandlerType::Fishing => Some(Box::new(FishingEventHandler::new())),
        HandlerType::SwitchTalk => generic_lua_event("events/generic/SwitchTalk.lua"),
        // The Gold Saucer's arcade machines and lotteries (`events/gold_saucer.rs`.)
        HandlerType::GoldSaucerArcadeMachine => Some(Box::new(ArcadeMachineEventHandler)),
        HandlerType::LotteryDaily => Some(Box::new(MiniCactpotEventHandler)),
        HandlerType::LotteryWeekly => Some(Box::new(JumboCactpotEventHandler)),
        HandlerType::GoldSaucerTalk => generic_lua_event("events/generic/GoldSaucerTalk.lua"),
        // A GATE's entrance, an event on the GATE's director, and the ContentEntry it leads to (`gate.rs`.)
        HandlerType::GoldSaucer => Some(Box::new(GateEventHandler)),
        HandlerType::ContentEntry => Some(Box::new(ContentEntryEventHandler)),
        HandlerType::Verminion => generic_lua_event("events/generic/Verminion.lua"),
        HandlerType::DpsChallengeOfficer => {
            generic_lua_event("events/generic/DpsChallengeOfficer.lua")
        }
        HandlerType::TopicSelect => generic_lua_event("events/generic/TopicSelect.lua"),
        HandlerType::PreHandler => generic_lua_event("events/generic/PreHandler.lua"),
        HandlerType::Description => Some(Box::new(DummyEventHandler::new())), // Same reason as DefaultTalk
        // A duty's entrance (its InstanceContent's guide): one scene the client plays itself.
        HandlerType::InstanceContentGuide => {
            generic_lua_event("events/generic/InstanceContentGuide.lua")
        }
        HandlerType::InclusionShop => Some(Box::new(InclusionShopEventHandler::new())),
        HandlerType::EventGimmickPathMove => {
            generic_lua_event("events/generic/GimmickPathMove.lua")
        }
        HandlerType::SpecialShop => Some(Box::new(SpecialShopEventHandler::new())),
        // A quartermaster's seal exchange (`events/gc_shop.rs`.)
        HandlerType::GrandCompanyShop => Some(Box::new(GrandCompanyShopEventHandler::new())),
        HandlerType::HousingAethernet => generic_lua_event("events/generic/HousingAethernet.lua"),
        HandlerType::InstanceContent => Some(Box::new(ContentEventHandler::new())),
        HandlerType::PublicContent => Some(Box::new(ContentEventHandler::new())),
        HandlerType::QuestBattle => Some(Box::new(ContentEventHandler::new())),
        HandlerType::Fate => Some(Box::new(FateEventHandler {
            fate_id: *connection.fate_motivation_npcs.get(&actor_id).unwrap(),
        })),
        _ => None,
    }
}

// Ensure that the script names we retrieve can be turned into sensible ids and folders.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_script_id() {
        // First ensure it succeeded.
        match extract_script_id("CmnDefBeginnerGuide_00327") {
            Ok(id) => assert_eq!(id, 327),
            Err(_) => assert!(false), // We don't care about the error, anything going wrong is an automatic test failure
        }

        // Next, fail intentionally with a bogus string and ensure it indeed failed.
        let fail_case = extract_script_id("CmnDefBeginnerGuide_XYZAB");
        assert!(fail_case.is_err());
    }

    #[test]
    fn test_folder_from_script_id() {
        assert_eq!(folder_from_script_id(327), "003");
    }
}
