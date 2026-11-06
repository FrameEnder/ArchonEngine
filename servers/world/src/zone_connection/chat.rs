//! Chat and command handling.

use std::str::FromStr;

use binrw::BinRead;

use mlua::Function;
use physis::equipment::EquipSlot;
use physis::sestring::SeString;

use crate::{
    Event, EventHandler, ItemInfoQuery, MessageInfo, ZoneConnection,
    inventory::{Item, Storage},
    lua::{KawariLuaState, LuaPlayer},
};
use kawari::{
    common::{ERR_INVENTORY_ADD_FAILED, HandlerId, HandlerType, ObjectTypeId, ObjectTypeKind},
    config::get_config,
    ipc::zone::{
        ActorControlCategory, ChatMessage, Condition, Conditions, EventType, GameMasterRank,
        SceneFlags, ServerNoticeFlags, ServerNoticeMessage, ServerZoneIpcData,
        ServerZoneIpcSegment,
    },
};

impl ZoneConnection {
    pub async fn send_message(&mut self, message: MessageInfo) {
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ChatMessage(ChatMessage {
            sender_account_id: message.sender_account_id,
            sender_actor_id: message.sender_actor_id,
            sender_world_id: message.sender_world_id,
            sender_name: message.sender_name,
            channel: message.channel,
            message: message.message,
            ..Default::default()
        }));
        self.send_ipc_self(ipc).await;
    }

    pub async fn send_notice(&mut self, message: &str) {
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ServerNoticeMessage(
            ServerNoticeMessage {
                flags: ServerNoticeFlags::CHAT_LOG,
                message: message.to_string(),
            },
        ));

        self.send_ipc_self(ipc).await;
    }

    pub async fn run_gm_command(
        &mut self,
        command: u32,
        arg0: i32,
        arg1: i32,
        arg2: i32,
        arg3: i32,
        name: String,
        lua_player: &mut LuaPlayer,
    ) {
        let lua = self.lua.lock();
        let state = lua.0.app_data_ref::<KawariLuaState>().unwrap();

        if let Some(command_script) = state.gm_command_scripts.get(&command) {
            let config = get_config();
            let file_name = config.filesystem.locate_script_file(command_script);

            let run_script = || -> mlua::Result<()> {
                lua.0.scope(|scope| {
                    let connection_data = scope
                    .create_userdata_ref_mut(lua_player)?;
                    /* TODO: Instead of panicking we ought to send a message to the player
                     * and the console log, and abandon execution. */
                    lua.0.load(
                        std::fs::read(&file_name).unwrap_or_else(|_| panic!("Failed to load script file {}!", file_name)),
                    )
                    .set_name("@".to_string() + &file_name)
                    .exec()?;

                    let required_rank = lua.0.globals().get("required_rank");
                    if let Err(error) = required_rank {
                        tracing::info!("Script is missing required_rank! Unable to run command, sending error to user. Additional information: {}", error);
                        let func: Function =
                        lua.0.globals().get("onCommandRequiredRankMissingError")?;
                        func.call::<()>((error.to_string(), connection_data))?;
                        return Ok(());
                    }

                    /* Reset state for future commands. Without this it'll stay set to the last value
                     * and allow other commands that omit required_rank to run, which is undesirable. */
                    lua.0.globals().set("required_rank", mlua::Value::Nil)?;

                    if self.player_data.character.gm_rank as u8 >= required_rank? {
                        let func: Function =
                        lua.0.globals().get("onCommand")?;
                        func.call::<()>((connection_data, [arg0, arg1, arg2, arg3], name))?;

                        /* `command_sender` is an optional variable scripts can define to identify themselves in print messages.
                         * It's okay if this global isn't set. We also don't care what its value is, just that it exists.
                         * This is reset -after- running the command intentionally. Resetting beforehand will never display the command's identifier.
                         */
                        let command_sender: Result<mlua::prelude::LuaValue, mlua::prelude::LuaError> = lua.0.globals().get("command_sender");
                        if command_sender.is_ok() {
                            lua.0.globals().set("command_sender", mlua::Value::Nil)?;
                        }
                        Ok(())
                    } else {
                        tracing::info!("User with account_id {} tried to invoke GM command {} with insufficient privileges!",
                                       self.player_data.character.service_account_id, command);
                        let func: Function =
                        lua.0.globals().get("onCommandRequiredRankInsufficientError")?;
                        func.call::<()>(connection_data)?;
                        Ok(())
                    }
                })
            };

            if let Err(err) = run_script() {
                tracing::warn!("Lua error in {file_name}: {:?}", err);
            }
        } else {
            tracing::warn!(
                "Received unknown GM command {command} with args: arg0: {arg0} arg1: {arg1} arg2: {arg2} arg3: {arg3}!"
            );
        }
    }

    /// Returns true if the debug command is handled, otherwise false.
    pub async fn process_debug_commands(
        &mut self,
        chat_message: &SeString, // TODO: Replace this with an SEString
        events: &mut Vec<(Box<dyn EventHandler>, Event)>,
    ) -> bool {
        // `!mark` is for anyone testing, GM or not.
        if let Some(text) = mark_text(&chat_message.to_string()) {
            self.mark(&text).await;
            return true;
        }

        if self.player_data.character.gm_rank == GameMasterRank::NormalUser {
            tracing::info!("Rejecting debug command because the user is not GM!");
            return true;
        }

        // TODO: Ensure the message has no SEString macros (e.g. auto-translate phrases)?
        let chat_message = chat_message.to_string();

        let parts: Vec<&str> = chat_message.split(' ').collect();

        match parts[0] {
            "!equip" => {
                if let Some((_, name)) = chat_message.split_once(' ') {
                    let item_info;
                    {
                        let mut game_data = self.game_data.lock();
                        item_info =
                            game_data.get_item_info(ItemInfoQuery::ByName(name.to_string()));
                    }
                    if let Some(item_info) = item_info {
                        let equip_slot = EquipSlot::from(&item_info.equip_category);

                        // EquipSlot::from returns EquipSlot::Waist if the item isn't equipment, since belts are no longer part of the game.
                        if equip_slot == EquipSlot::Waist {
                            self.send_notice(&format!("[equip] The found item, {:#?}, isn't equipment! If the wrong item was found, try being more specific with its name, or consider using //gm item instead if you can't get the desired item from this command.", item_info.name).to_string()).await;
                            return true;
                        };

                        let slot = self
                            .player_data
                            .inventory
                            .equipped
                            .get_slot_mut(equip_slot as u16);
                        *slot = Item::new(&item_info, 1);

                        self.send_inventory().await;
                        self.inform_equip().await;
                    } else {
                        self.send_notice(&format!("[equip] No items named {name:#?} were found! If you know its item id instead, consider using //gm item.").to_string()).await;
                    }
                } else {
                    self.send_notice("[equip] Usage: !equip <item name>").await;
                }

                true
            }
            "!item" => {
                if let Some((_, name)) = chat_message.split_once(' ') {
                    let mut result = None;
                    {
                        let mut game_data = self.game_data.lock();

                        if let Some(item_info) =
                            game_data.get_item_info(ItemInfoQuery::ByName(name.to_string()))
                        {
                            result = self
                                .player_data
                                .inventory
                                .add_in_next_free_slot(Item::new(&item_info, 1));
                        }
                    }

                    if result.is_some() {
                        self.send_inventory().await;
                    } else {
                        tracing::error!(ERR_INVENTORY_ADD_FAILED);
                        self.send_notice(ERR_INVENTORY_ADD_FAILED).await;
                    }
                }

                true
            }
            "!reload" => {
                self.reload_scripts().await;
                self.send_notice("Scripts reloaded!").await;
                true
            }
            "!directorscene" => {
                // Plays a scene of the current content director (e.g. a quest battle's), the way its victory fanfare
                // (60001) is played. For finding the director's other generic scenes.
                let scene: u16 = parts.get(1).and_then(|x| x.parse().ok()).unwrap_or(0);
                let flags: u32 = parts
                    .get(2)
                    .and_then(|x| u32::from_str_radix(x.trim_start_matches("0x"), 16).ok())
                    .unwrap_or(0x40000);
                let Some(handler_id) = self.content_handler_id else {
                    self.send_notice("There's no content director here.").await;
                    return true;
                };
                let actor_id = ObjectTypeId {
                    object_id: self.player_data.character.actor_id,
                    object_type: ObjectTypeKind::None,
                };
                self.start_event(
                    actor_id,
                    handler_id,
                    EventType::GameProgress,
                    1,
                    events,
                    &LuaPlayer::default(),
                )
                .await;
                if let Some((_, event)) = events.last() {
                    self.event_scene(
                        event,
                        scene,
                        SceneFlags::from_bits_retain(flags),
                        Vec::new(),
                    )
                    .await;
                }
                true
            }
            "!forkopcodes" => {
                // Which of the fork's own opcodes are known for this game version, and how to find the rest.
                for line in crate::fork_opcodes::status_lines() {
                    self.send_notice(&line).await;
                }
                true
            }
            "!finishevent" => {
                self.event_finish(events).await;
                self.send_notice("Current event forcefully finished.").await;
                true
            }
            "!condition" => {
                if let Some((_, condition_name)) = chat_message.split_once(' ') {
                    if let Ok(condition) = Condition::from_str(condition_name) {
                        self.conditions.set_condition(condition);
                        self.send_conditions().await;
                        self.send_notice(&format!("Condition {condition:?} set!"))
                            .await;
                    } else {
                        self.send_notice(&format!("Unknown condition {condition_name}"))
                            .await;
                    }
                }

                true
            }
            "!clearconditions" => {
                self.conditions = Conditions::default();
                self.send_conditions().await;
                self.send_notice("Conditions cleared!").await;

                true
            }
            "!acs" => {
                let parts: Vec<&str> = chat_message.split(' ').collect();
                if parts.len() > 1 {
                    self.actor_control_self(ActorControlCategory::Unknown {
                        category: parts.get(1).cloned().unwrap().parse().unwrap_or_default(),
                        param1: parts
                            .get(2)
                            .cloned()
                            .unwrap_or_default()
                            .parse()
                            .unwrap_or_default(),
                        param2: parts
                            .get(3)
                            .cloned()
                            .unwrap_or_default()
                            .parse()
                            .unwrap_or_default(),
                        param3: parts
                            .get(4)
                            .cloned()
                            .unwrap_or_default()
                            .parse()
                            .unwrap_or_default(),
                        param4: parts
                            .get(5)
                            .cloned()
                            .unwrap_or_default()
                            .parse()
                            .unwrap_or_default(),
                        param5: parts
                            .get(6)
                            .cloned()
                            .unwrap_or_default()
                            .parse()
                            .unwrap_or_default(),
                    })
                    .await;
                }

                true
            }
            "!yell" => {
                if let Some((_, npc_yell_id)) = chat_message.split_once(' ') {
                    let npc_yell_id = npc_yell_id.parse().unwrap_or_default();

                    let name_id;
                    {
                        let mut game_data = self.game_data.lock();
                        name_id = game_data
                            .get_npc_yell_name_id(npc_yell_id)
                            .unwrap_or_default();
                    }

                    self.send_ipc_self(ServerZoneIpcSegment::new(ServerZoneIpcData::NpcYell {
                        object_id: ObjectTypeId::default(),
                        name_id,
                        npc_yell_id,
                        param1: 0,
                        param2: 0,
                        param3: 0,
                        param4: 0,
                    }))
                    .await;
                }

                true
            }
            "!gate" => {
                if let Some((_, id)) = chat_message.split_once(' ')
                    && let Ok(id) = id.parse()
                {
                    let handler_id = HandlerId::new(HandlerType::GoldSaucer, 119); // TODO: unsure how this is decided
                    let row;
                    let npc_yell_id = 2450; // <?:96>The limited-time event “<?:9>” is now underway <?:9>. All guests are encouraged to participate!
                    let name_id;
                    {
                        let mut game_data = self.game_data.lock();
                        let Some(gate_row) = game_data.get_gate(id) else {
                            return true;
                        };
                        row = gate_row;
                        let Some(yell_name_id) = game_data.get_npc_yell_name_id(2450) else {
                            return true;
                        };
                        name_id = yell_name_id;
                    }

                    self.send_ipc_self(ServerZoneIpcSegment::new(ServerZoneIpcData::NpcYell {
                        object_id: ObjectTypeId::default(),
                        name_id,
                        npc_yell_id,
                        param1: row.Unknown11 as u32, // TODO: Rename to Location
                        param2: row.Type as u32,
                        param3: 0,
                        param4: 0,
                    }))
                    .await;

                    self.actor_control_self(ActorControlCategory::InitDirector {
                        handler_id,
                        content_id: 2, // TODO: ditto
                        flags: 0,
                    })
                    .await;
                }

                true
            }
            "!settime" => {
                if let Some((_, val)) = chat_message.split_once(' ') {
                    // TODO: Figure out how UTC is converted to Eorzean time and make this friendly by allowing for strings such as "6:30PM" or "18:30"
                    // TODO: Write the GM command equivalent which would just set the time offset directly as an i64/u64 (whichever this actually is)
                    let val = val.parse::<i64>().unwrap_or_default();
                    self.set_eorzean_time(val).await;
                }

                true
            }
            "!emptypacket" => {
                let parts: Vec<&str> = chat_message.split(' ').collect();
                if parts.len() == 3 {
                    self.send_arbitrary_packet(
                        parts.get(1).unwrap().parse().unwrap_or_default(),
                        vec![0; parts.get(2).unwrap().parse().unwrap_or_default()],
                    )
                    .await;
                }

                true
            }
            "!rawpacket" => {
                // Sends an IPC packet with the given opcode and contents, written as hex, e.g. !rawpacket 643 00000000...
                let parts: Vec<&str> = chat_message.split(' ').collect();
                if parts.len() == 3 {
                    let hex = parts[2];
                    let data: Option<Vec<u8>> = (0..hex.len())
                        .step_by(2)
                        .map(|i| {
                            hex.get(i..i + 2)
                                .and_then(|b| u8::from_str_radix(b, 16).ok())
                        })
                        .collect();
                    match (parts[1].parse(), data) {
                        (Ok(op_code), Some(data)) => {
                            self.send_arbitrary_packet(op_code, data).await
                        }
                        _ => self.send_notice("Usage: !rawpacket <opcode> <hex>").await,
                    }
                }

                true
            }
            "!rawsweep" => {
                // Sends the same IPC data with every opcode in a range that Kawari doesn't know yet, to find which one
                // the client reacts to. e.g. !rawsweep 0 1100 00000000
                let parts: Vec<&str> = chat_message.split(' ').collect();
                let data: Option<Vec<u8>> = parts.get(3).and_then(|hex| {
                    (0..hex.len())
                        .step_by(2)
                        .map(|i| {
                            hex.get(i..i + 2)
                                .and_then(|b| u8::from_str_radix(b, 16).ok())
                        })
                        .collect()
                });
                match (
                    parts.get(1).and_then(|x| x.parse::<u16>().ok()),
                    parts.get(2).and_then(|x| x.parse::<u16>().ok()),
                    data,
                ) {
                    (Some(from), Some(to), Some(data)) => {
                        let mut sent = 0;
                        for op_code in from..=to {
                            let known = kawari::opcodes::ServerZoneIpcType::read_le(
                                &mut std::io::Cursor::new(op_code.to_le_bytes()),
                            )
                            .is_ok_and(|x| {
                                !matches!(x, kawari::opcodes::ServerZoneIpcType::Unknown(_))
                            });
                            if !known {
                                self.send_arbitrary_packet(op_code, data.clone()).await;
                                sent += 1;
                            }
                        }
                        self.send_notice(&format!(
                            "Sent {sent} unknown opcodes from {from} to {to}."
                        ))
                        .await;
                    }
                    _ => self.send_notice("Usage: !rawsweep <from> <to> <hex>").await,
                }

                true
            }
            "!spectator" => {
                self.actor_control_self(ActorControlCategory::InitializeSpectatorManager {
                    row_id: 0,
                    unk2: 0,
                    unk3: 0,
                })
                .await;
                // This ACS has to be delayed, otherwise it doesn't take effect.
                self.hide_spectator_ui = true;
                true
            }
            _ => false,
        }
    }

    /// `!mark <what happened>`: notes the moment in the packet capture (a `note` line, `capture.rs`) and the server
    /// log, with where the player is and their quests' steps, so a diagnostics bundle shows exactly when something went
    /// wrong (Bob, 2026-11-05: "sometimes Rive goes toward the boulder" had to be found by reading a whole capture.)
    /// The player is told whether the capture is on: without it only the log has the mark.
    pub async fn mark(&mut self, text: &str) {
        let position = self.player_data.volatile.position.0;
        let quests: Vec<String> = self
            .player_data
            .quest
            .active
            .0
            .iter()
            .filter(|quest| quest.id != 0)
            .map(|quest| format!("{}:{}", quest.id, quest.sequence))
            .collect();
        let note = format!(
            "MARK {text} | zone {} at ({:.2}, {:.2}, {:.2}) facing {:.3} | quests {}",
            self.player_data.volatile.zone_id,
            position.x,
            position.y,
            position.z,
            self.player_data.volatile.rotation,
            if quests.is_empty() {
                "none".to_string()
            } else {
                quests.join(" ")
            }
        );
        tracing::info!("{} {note}", self.player_data.character.name);
        self.capture.note(&note);
        let reply = if self.capture.is_enabled() {
            "Marked in the packet capture."
        } else {
            "Marked in the server log only: packet capture is off (ArchonEngine ControlPanel → Diagnostics turns it on)."
        };
        self.send_notice(reply).await;
    }
}

/// The text of a `!mark` command (empty for a bare `!mark`), or None for anything else.
fn mark_text(message: &str) -> Option<String> {
    let rest = message.strip_prefix("!mark")?;
    if !(rest.is_empty() || rest.starts_with(' ')) {
        return None;
    }
    Some(rest.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::mark_text;

    #[test]
    fn mark_text_reads_the_command() {
        assert_eq!(
            mark_text("!mark Rive went the other way").as_deref(),
            Some("Rive went the other way")
        );
        assert_eq!(mark_text("!mark").as_deref(), Some(""));
        assert_eq!(mark_text("!marker"), None);
        assert_eq!(mark_text("!equip 1"), None);
    }
}
