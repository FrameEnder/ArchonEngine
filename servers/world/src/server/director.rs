// TODO: Restore DirectorTrigger handling, I removed it after the refactor...

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use kawari::{
    common::{
        ContentDirectorEvent, DirectorEvent, DirectorTrigger, DutyOption, EOBJ_EXIT, EOBJ_SHORTCUT,
        EventState, GimmickAccessorType, GimmickRectEvent, HandlerId, HandlerType,
        InstanceContentDirectorEvent, LeveDirectorEvent, LogMessageType, ObjectId, ObjectTypeId,
        ObjectTypeKind, Position, PublicContentType, TerritoryIntendedUse, WARP_DELAY, WarpType,
        timestamp_secs,
    },
    config::get_config,
    ipc::chat::ChatChannelType,
    ipc::zone::{
        ActorControlCategory, ActorControlSelf, ActorSetPos, ChatMessage, DutyFinderSetting,
        EventLogMessage, MapEffects, PrepareZoning, ServerZoneIpcData, ServerZoneIpcSegment,
        WalkInEvent,
    },
};
use mlua::{Function, LuaSerdeExt, UserData, UserDataMethods, Value};
use parking_lot::Mutex;

use crate::{
    ClientId, FromServer, GameData, QUEST_BATTLE_SCENE_FINISH, QUEST_BATTLE_SCENE_FINISH_FLAGS,
    ToServer,
    lua::KawariLua,
    server::{
        WorldServer,
        actor::NetworkedActor,
        effect::gain_effect_instance,
        instance::{Instance, QueuedTaskData},
        network::{DestinationNetwork, NetworkState},
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum LuaDirectorTask {
    HideEObj {
        base_id: u32,
    },
    ShowEObj {
        base_id: u32,
    },
    DeleteEObj {
        base_id: u32,
    },
    SpawnEObj {
        base_id: u32,
        position: Option<Position>,
    },
    SendVariables,
    AbandonDuty {
        actor_id: ObjectId,
    },
    BeginEventAction {
        actor_id: ObjectId,
        target: ObjectId,
        action_id: u32,
    },
    FinishGimmickEvent {
        actor_id: ObjectId,
    },
    LogMessage {
        id: u32,
        params: Vec<u32>,
    },
    SpawnBattleNpc {
        id: u32,
    },
    /// Remove a BNpc (by layout ID) without it dying, e.g. the background fighters before a quest battle's cutscene.
    DespawnBattleNpc {
        layout_id: u32,
    },
    GainEffect {
        actor_id: ObjectId,
        id: u16,
        param: u16,
        duration: f32,
    },
    SetBGM {
        id: u32,
    },
    SealBossWall {
        actor_id: ObjectId,
        id: u32,
        place_name: u32,
        time_until: u32,
    },
    SpawnBoss {
        bnpc_id: u32,
        wall_id: u32,
        line_id: u32,
        place_name: u32,
    },
    SpawnTreasure {
        id: u32,
    },
    VariantVoteRoute {
        npc_route: u32,
    },
    PlayCutscene {
        cutscene_id: u32,
    },
    UpdateShortcut {
        poprange_id: u32,
    },
    UseShortcut {
        actor_id: ObjectId,
    },
    CompleteDuty,
    /// Win a quest battle, which also moves its quest to this sequence. "Duty Complete" shows straight away; unless
    /// `stay`, the players are then sent back, otherwise they stay until `LeaveQuestBattle`.
    CompleteQuestBattle {
        sequence: u8,
        stay: bool,
        /// The scene played before they're sent back, see `QUEST_BATTLE_SCENE_FINISH`.
        finish_scene: u16,
        /// Its scene flags, see `QUEST_BATTLE_SCENE_FINISH_FLAGS`.
        finish_flags: u32,
        /// Quest variables (index 0-5, value) set in the same journal update as the sequence (Lurkers in the Grotto's
        /// ITEM0, UI8BH, as retail sets it on winning.)
        variables: Vec<(u8, u8)>,
    },
    /// Play one of the director's scenes for every player (e.g. a cutscene halfway through a quest battle), with
    /// these scene flags. The script's `onSceneFinished(director, scene)` is called when a player finishes it.
    PlayScene {
        scene: u16,
        flags: u32,
    },
    /// Send the players of a won quest battle back (see `CompleteQuestBattle`'s `stay`.)
    LeaveQuestBattle,
    /// Move every player to a position in the same instance through a loading screen, as retail does after To Guard
    /// a Guardian's cutscene (PrepareZoning to the same territory, then ActorSetPos; the client then finishes zoning
    /// and gets ZoneIn). A cutscene that ends this way leaves the client waiting for it.
    WarpPlayers {
        position: Position,
        rotation: f32,
    },
    /// Lose a quest battle, which sends the players back to where they entered it.
    FailQuestBattle,
    /// Make a BNpc use an action on another BNpc (by layout ID), or on the player if there's no target.
    UseAction {
        layout_id: u32,
        action_id: u32,
        target: Option<u32>,
    },
    /// A BNpc says a line of quest text (by its key) in /say, e.g. an ally's battle chatter.
    Say {
        layout_id: u32,
        key: String,
    },
    /// Someone says the quest battle's quest's `number`th battle talk line (1-based, in key order), see
    /// `GameData::get_quest_battle_talk`. It comes from the BNpc at `layout_id`, under the name `name_id` (BNpcName) if
    /// given, for speakers that aren't there, like the masked mage.
    SayBattleTalk {
        layout_id: u32,
        name_id: Option<u32>,
        number: usize,
    },
    /// Shows the battle talk window (DirectorPopupMessage), as retail does in quest battles: `text_id` is the
    /// content's text (InstanceContentTextData; one outside the content's range crashes the client, so use recorded
    /// IDs), `name_id` the speaker's BNpcName and `icon` the portrait. It comes from the BNpc at `layout_id` if
    /// it's there.
    BattleTalk {
        layout_id: u32,
        name_id: u32,
        text_id: u32,
        icon: u32,
        duration_ms: u32,
    },
    /// Has an NPC yell a line from the NpcYell sheet: a speech bubble over them, and the line in chat.
    Yell {
        layout_id: u32,
        npc_yell_id: u32,
    },
    /// Change the director's sequence, which the client's script for it reacts to (e.g. `OnSequence001`.)
    SetSequence {
        sequence: u8,
    },
    /// How close (in yalms) a BNpc gets to its target before attacking: melee (3) or a caster's range (15.)
    SetAttackRange {
        layout_id: u32,
        yalms: f32,
    },
    /// Keep a BNpc within `yalms` of every one of its anchors (BNpcs by layout ID; `None` is the player): it heads back
    /// to the middle of them when it's further from any, whatever it's fighting (a support near the tank and the
    /// player.) No anchors or no distance stops it.
    KeepNear {
        layout_id: u32,
        anchors: Vec<Option<u32>>,
        yalms: Option<f32>,
    },
    /// Have a BNpc stand behind the enemy another BNpc (the tank) is fighting, `yalms` beyond it: out of its frontal
    /// cones and cleaves, where a support belongs. `None` stops it.
    StandBehind {
        layout_id: u32,
        tank: u32,
        yalms: Option<f32>,
    },
    /// Make a BNpc hate another BNpc (by layout ID), or the player if there's no target, by this much enmity.
    SetHate {
        layout_id: u32,
        target: Option<u32>,
        amount: u32,
    },
    MapEffect {
        index: u32,
        timeline_id: u32,
    },
}

/// One actor's numbers for a director's `onUpdate` (see `DirectorData::update`.)
#[derive(Debug, Clone, Copy)]
pub struct ActorInfo {
    pub hp: u32,
    pub max_hp: u32,
    pub position: glam::Vec3A,
    /// Who it's targeting: an NPC's layout ID, or the player.
    pub target_layout: Option<u32>,
    pub target_is_player: bool,
}

// TODO: Maybe collapse into DirectorData?
#[derive(Default, Debug)]
pub struct LuaDirector {
    pub data: [u8; 10],
    pub tasks: Vec<LuaDirectorTask>,
    pub bosses: HashMap<u32, DirectorBoss>,
}

impl UserData for LuaDirector {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("hide_eobj", |_, this, base_id: u32| {
            this.tasks.push(LuaDirectorTask::HideEObj { base_id });
            Ok(())
        });
        methods.add_method_mut("show_eobj", |_, this, base_id: u32| {
            this.tasks.push(LuaDirectorTask::ShowEObj { base_id });
            Ok(())
        });
        methods.add_method_mut("delete_eobj", |_, this, base_id: u32| {
            this.tasks.push(LuaDirectorTask::DeleteEObj { base_id });
            Ok(())
        });
        methods.add_method_mut(
            "spawn_eobj",
            |lua, this, (base_id, position): (u32, Value)| {
                let position: Option<Position> = lua.from_value(position).ok();
                this.tasks
                    .push(LuaDirectorTask::SpawnEObj { base_id, position });
                Ok(())
            },
        );
        methods.add_method_mut("set_data", |_, this, (index, data): (u8, u8)| {
            this.data[index as usize] = data;
            Ok(())
        });
        methods.add_method("data", |_, this, index: u8| Ok(this.data[index as usize]));
        methods.add_method_mut(
            "event_action",
            |_, this, (action_id, actor_id, target): (u32, u32, u32)| {
                this.tasks.push(LuaDirectorTask::BeginEventAction {
                    actor_id: ObjectId(actor_id),
                    target: ObjectId(target),
                    action_id,
                });
                Ok(())
            },
        );
        methods.add_method_mut("finish_gimmick", |_, this, actor_id: u32| {
            this.tasks.push(LuaDirectorTask::FinishGimmickEvent {
                actor_id: ObjectId(actor_id),
            });
            Ok(())
        });
        methods.add_method_mut("log_message", |_, this, (id, params): (u32, Vec<u32>)| {
            this.tasks.push(LuaDirectorTask::LogMessage { id, params });
            Ok(())
        });
        methods.add_method_mut("spawn_bnpc", |_, this, id: u32| {
            this.tasks.push(LuaDirectorTask::SpawnBattleNpc { id });
            Ok(())
        });
        methods.add_method_mut("despawn_bnpc", |_, this, layout_id: u32| {
            this.tasks
                .push(LuaDirectorTask::DespawnBattleNpc { layout_id });
            Ok(())
        });
        methods.add_method_mut(
            "gain_effect",
            |_, this, (actor_id, id, param, duration): (u32, u16, u16, f32)| {
                this.tasks.push(LuaDirectorTask::GainEffect {
                    actor_id: ObjectId(actor_id),
                    id,
                    param,
                    duration,
                });
                Ok(())
            },
        );
        methods.add_method_mut("set_bgm", |_, this, id: u32| {
            this.tasks.push(LuaDirectorTask::SetBGM { id });
            Ok(())
        });
        methods.add_method_mut(
            "spawn_boss",
            |_, this, (bnpc_id, wall_id, line_id, place_name): (u32, u32, u32, u32)| {
                this.tasks.push(LuaDirectorTask::SpawnBoss {
                    bnpc_id,
                    wall_id,
                    line_id,
                    place_name,
                });
                Ok(())
            },
        );
        methods.add_method_mut("spawn_treasure", |_, this, id: u32| {
            this.tasks.push(LuaDirectorTask::SpawnTreasure { id });
            Ok(())
        });
        methods.add_method_mut("variant_vote_route", |_, this, npc_route: u32| {
            this.tasks
                .push(LuaDirectorTask::VariantVoteRoute { npc_route });
            Ok(())
        });
        methods.add_method_mut("play_cutscene", |_, this, cutscene_id: u32| {
            this.tasks
                .push(LuaDirectorTask::PlayCutscene { cutscene_id });
            Ok(())
        });
        methods.add_method_mut("update_shortcut", |_, this, poprange_id: u32| {
            // Show the shortcut object because it's hidden by default
            this.tasks.push(LuaDirectorTask::ShowEObj {
                base_id: EOBJ_SHORTCUT,
            });
            this.tasks
                .push(LuaDirectorTask::UpdateShortcut { poprange_id });
            Ok(())
        });
        methods.add_method_mut("complete_duty", |_, this, _: ()| {
            // Show the exit object
            this.tasks
                .push(LuaDirectorTask::ShowEObj { base_id: EOBJ_EXIT });
            this.tasks.push(LuaDirectorTask::CompleteDuty {});
            Ok(())
        });
        methods.add_method_mut(
            "complete_quest_battle",
            |_,
             this,
             (sequence, stay, finish_scene, finish_flags, variables): (
                u8,
                Option<bool>,
                Option<u16>,
                Option<u32>,
                Option<mlua::Table>,
            )| {
                // `variables` is a table of quest variable index (0-5) to value, e.g. { [1] = 0x10 }.
                let mut set = Vec::new();
                if let Some(variables) = variables {
                    for pair in variables.pairs::<u8, u8>() {
                        set.push(pair?);
                    }
                    set.sort();
                }
                this.tasks.push(LuaDirectorTask::CompleteQuestBattle {
                    sequence,
                    stay: stay.unwrap_or(false),
                    finish_scene: finish_scene.unwrap_or(QUEST_BATTLE_SCENE_FINISH),
                    finish_flags: finish_flags.unwrap_or(QUEST_BATTLE_SCENE_FINISH_FLAGS),
                    variables: set,
                });
                Ok(())
            },
        );
        methods.add_method_mut("play_scene", |_, this, (scene, flags): (u16, u32)| {
            this.tasks.push(LuaDirectorTask::PlayScene { scene, flags });
            Ok(())
        });
        methods.add_method_mut(
            "warp_players",
            |_, this, (x, y, z, rotation): (f32, f32, f32, f32)| {
                this.tasks.push(LuaDirectorTask::WarpPlayers {
                    position: Position(glam::Vec3A::new(x, y, z)),
                    rotation,
                });
                Ok(())
            },
        );
        methods.add_method_mut("leave_quest_battle", |_, this, _: ()| {
            this.tasks.push(LuaDirectorTask::LeaveQuestBattle);
            Ok(())
        });
        methods.add_method_mut(
            "use_action",
            |_, this, (layout_id, action_id, target): (u32, u32, Option<u32>)| {
                this.tasks.push(LuaDirectorTask::UseAction {
                    layout_id,
                    action_id,
                    target,
                });
                Ok(())
            },
        );
        methods.add_method_mut("say", |_, this, (layout_id, key): (u32, String)| {
            this.tasks.push(LuaDirectorTask::Say { layout_id, key });
            Ok(())
        });
        methods.add_method_mut(
            "say_battle_talk",
            |_, this, (layout_id, number, name_id): (u32, usize, Option<u32>)| {
                this.tasks.push(LuaDirectorTask::SayBattleTalk {
                    layout_id,
                    name_id,
                    number,
                });
                Ok(())
            },
        );
        methods.add_method_mut(
            "battle_talk",
            |_,
             this,
             (layout_id, name_id, text_id, icon, duration_ms): (
                u32,
                u32,
                u32,
                u32,
                Option<u32>,
            )| {
                this.tasks.push(LuaDirectorTask::BattleTalk {
                    layout_id,
                    name_id,
                    text_id,
                    icon,
                    // Retail shows every line for 4 seconds.
                    duration_ms: duration_ms.unwrap_or(4000),
                });
                Ok(())
            },
        );
        methods.add_method_mut("yell", |_, this, (layout_id, npc_yell_id): (u32, u32)| {
            this.tasks.push(LuaDirectorTask::Yell {
                layout_id,
                npc_yell_id,
            });
            Ok(())
        });
        methods.add_method_mut("set_sequence", |_, this, sequence: u8| {
            this.tasks.push(LuaDirectorTask::SetSequence { sequence });
            Ok(())
        });
        methods.add_method_mut("fail_quest_battle", |_, this, _: ()| {
            this.tasks.push(LuaDirectorTask::FailQuestBattle);
            Ok(())
        });
        methods.add_method_mut(
            "set_hate",
            |_, this, (layout_id, target, amount): (u32, Option<u32>, Option<u32>)| {
                this.tasks.push(LuaDirectorTask::SetHate {
                    layout_id,
                    target,
                    amount: amount.unwrap_or(1),
                });
                Ok(())
            },
        );
        methods.add_method_mut(
            "set_attack_range",
            |_, this, (layout_id, yalms): (u32, f32)| {
                this.tasks
                    .push(LuaDirectorTask::SetAttackRange { layout_id, yalms });
                Ok(())
            },
        );
        methods.add_method_mut(
            "keep_near",
            |_, this, (layout_id, anchors, yalms): (u32, mlua::Value, Option<f32>)| {
                // One anchor (a layout ID, or nil / "player" for the player) or a list of them.
                let one = |value: &mlua::Value| match value {
                    mlua::Value::Integer(layout) => Some(Some(*layout as u32)),
                    mlua::Value::Number(layout) => Some(Some(*layout as u32)),
                    mlua::Value::Nil => Some(None),
                    mlua::Value::String(text) if text.to_str().is_ok_and(|t| t == "player") => {
                        Some(None)
                    }
                    _ => None,
                };
                let anchors = match &anchors {
                    mlua::Value::Table(list) => list
                        .sequence_values::<mlua::Value>()
                        .filter_map(|value| value.ok())
                        .filter_map(|value| one(&value))
                        .collect(),
                    value => one(value).into_iter().collect(),
                };
                this.tasks.push(LuaDirectorTask::KeepNear {
                    layout_id,
                    anchors,
                    yalms,
                });
                Ok(())
            },
        );
        methods.add_method_mut(
            "stand_behind",
            |_, this, (layout_id, tank, yalms): (u32, u32, Option<f32>)| {
                this.tasks.push(LuaDirectorTask::StandBehind {
                    layout_id,
                    tank,
                    yalms,
                });
                Ok(())
            },
        );
        methods.add_method_mut("map_effect", |_, this, (index, timeline_id): (u32, u32)| {
            this.tasks
                .push(LuaDirectorTask::MapEffect { index, timeline_id });
            Ok(())
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DirectorBossState {
    /// The boss arena is open and nothing has happened yet.
    Open,
    /// The boss arena is closing soon, because the boss was aggravated.
    Aggro,
    /// The boss arena is closed.
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DirectorBoss {
    state: DirectorBossState,
    actor_id: ObjectId,
    wall_id: u32,
    line_id: u32,
    place_name: u32,
}

#[derive(Debug, Default)]
pub struct DirectorData {
    pub id: HandlerId,
    /// The director's sequence (DirectorVars' first byte.)
    pub flag: u8,
    pub data: [u8; 10],
    /// Lua state for this director.
    pub lua: KawariLua,
    pub tasks: Vec<LuaDirectorTask>,
    /// List of alive bosses and their data.
    pub bosses: HashMap<u32, DirectorBoss>,
    /// What the shortcut is currently pointing to.
    pub shortcut_poprange_id: Option<u32>,
    /// For quest battles, the quest it belongs to.
    pub quest_battle_quest_id: u32,
}

impl DirectorData {
    pub fn setup(&mut self) {
        let mut run_script = || {
            let mut lua_director = self.create_lua_director();
            let err = self.lua.0.scope(|scope| {
                let data = scope.create_userdata_ref_mut(&mut lua_director)?;

                let func: Function = self.lua.0.globals().get("onSetup")?;

                func.call::<()>(data)?;

                Ok(())
            });
            self.apply_lua_director(lua_director);
            err
        };
        if let Err(err) = run_script() {
            tracing::warn!("Syntax error during onSetup: {err:?}");
        }
    }

    /// Called every server tick for scripts that define `onUpdate`, with the health (in percent) of every living NPC
    /// by its layout ID, the player's, and (fourth) everyone's details: `info[layout] = { hp, max_hp, x, y, z, target }`
    /// (`target`: the layout ID it's after, or "player"), and
    /// `info.player` the same for the player, for scripts that need real numbers or distances (content/lib/Battle.lua.)
    pub fn update(
        &mut self,
        npc_health: &HashMap<u32, u32>,
        player_health: Option<u32>,
        info: &HashMap<String, ActorInfo>,
    ) {
        let Ok(func) = self.lua.0.globals().get::<Function>("onUpdate") else {
            return;
        };

        let mut lua_director = self.create_lua_director();
        let err = self.lua.0.scope(|scope| {
            let data = scope.create_userdata_ref_mut(&mut lua_director)?;
            let health = self
                .lua
                .0
                .create_table_from(npc_health.iter().map(|(k, v)| (*k, *v)))?;
            let details = self.lua.0.create_table()?;
            for (key, actor) in info {
                let entry = self.lua.0.create_table()?;
                entry.set("hp", actor.hp)?;
                entry.set("max_hp", actor.max_hp)?;
                entry.set("x", actor.position.x)?;
                entry.set("y", actor.position.y)?;
                entry.set("z", actor.position.z)?;
                if let Some(target) = actor.target_layout {
                    entry.set("target", target)?;
                } else if actor.target_is_player {
                    entry.set("target", "player")?;
                }
                match key.parse::<u32>() {
                    Ok(layout_id) => details.set(layout_id, entry)?,
                    Err(_) => details.set(key.as_str(), entry)?,
                }
            }

            func.call::<()>((data, health, player_health, details))?;

            Ok(())
        });
        self.apply_lua_director(lua_director);
        if let Err(err) = err {
            tracing::warn!("Syntax error during onUpdate: {err:?}");
        }
    }

    /// Called when the player is ready for the content to begin, after its opening cutscene.
    pub fn commence(&mut self) {
        let mut run_script = || {
            let mut lua_director = self.create_lua_director();
            let err = self.lua.0.scope(|scope| {
                let data = scope.create_userdata_ref_mut(&mut lua_director)?;

                let func: Function = self.lua.0.globals().get("onCommence")?;

                func.call::<()>(data)?;

                Ok(())
            });
            self.apply_lua_director(lua_director);
            err
        };
        if let Err(err) = run_script() {
            tracing::warn!("Syntax error during onCommence: {err:?}");
        }
    }

    /// Called when a player finishes one of the director's scenes, for scripts that define `onSceneFinished`.
    pub fn scene_finished(&mut self, scene: u16) {
        let Ok(func) = self.lua.0.globals().get::<Function>("onSceneFinished") else {
            return;
        };

        let mut lua_director = self.create_lua_director();
        let err = self.lua.0.scope(|scope| {
            let data = scope.create_userdata_ref_mut(&mut lua_director)?;
            func.call::<()>((data, scene))?;
            Ok(())
        });
        self.apply_lua_director(lua_director);
        if let Err(err) = err {
            tracing::warn!("Syntax error during onSceneFinished: {err:?}");
        }
    }

    pub fn gimmick_accessor(&mut self, actor_id: ObjectId, id: u32, params: &[i32]) {
        let mut run_script = || {
            let mut lua_director = self.create_lua_director();
            let err = self.lua.0.scope(|scope| {
                let data = scope.create_userdata_ref_mut(&mut lua_director)?;

                let func: Function = self.lua.0.globals().get("onGimmickAccessor")?;

                func.call::<()>((data, actor_id.0, id, params))?;

                Ok(())
            });
            self.apply_lua_director(lua_director);
            err
        };
        if let Err(err) = run_script() {
            tracing::warn!("Syntax error during onGimmickAccessor: {err:?}");

            // Finish the gimmick event so the client at least gets unstuck
            self.tasks
                .push(LuaDirectorTask::FinishGimmickEvent { actor_id });
        }
    }

    pub fn event_action_cast(&mut self, actor_id: ObjectId, target: ObjectId) {
        let mut run_script = || {
            let mut lua_director = self.create_lua_director();
            let err = self.lua.0.scope(|scope| {
                let data = scope.create_userdata_ref_mut(&mut lua_director)?;

                let func: Function = self.lua.0.globals().get("onEventActionCast")?;

                func.call::<()>((data, actor_id.0, target.0))?;

                Ok(())
            });
            self.apply_lua_director(lua_director);
            err
        };
        if let Err(err) = run_script() {
            tracing::warn!("Syntax error during onEventActionCast: {err:?}");
        }
    }

    /// Called when a player in this instance dies, if the script has `onPlayerDeath`.
    pub fn on_player_death(&mut self) {
        if self
            .lua
            .0
            .globals()
            .get::<Function>("onPlayerDeath")
            .is_err()
        {
            return;
        }

        let mut run_script = || {
            let mut lua_director = self.create_lua_director();
            let err = self.lua.0.scope(|scope| {
                let data = scope.create_userdata_ref_mut(&mut lua_director)?;

                let func: Function = self.lua.0.globals().get("onPlayerDeath")?;

                func.call::<()>(data)?;

                Ok(())
            });
            self.apply_lua_director(lua_director);
            err
        };
        if let Err(err) = run_script() {
            tracing::warn!("Syntax error during onPlayerDeath: {err:?}");
        }
    }

    pub fn on_actor_death(&mut self, bnpc_id: u32, position: Position) {
        if let Some(boss) = self.bosses.get(&bnpc_id) {
            self.unseal_boss_wall(boss.wall_id, boss.line_id, boss.place_name);
        }

        // Most directors (and FATEs) don't care who dies.
        if self
            .lua
            .0
            .globals()
            .get::<Function>("onActorDeath")
            .is_err()
        {
            return;
        }

        let mut run_script = || {
            let mut lua_director = self.create_lua_director();
            let err = self.lua.0.scope(|scope| {
                let data = scope.create_userdata_ref_mut(&mut lua_director)?;

                let func: Function = self.lua.0.globals().get("onActorDeath")?;

                func.call::<()>((data, bnpc_id, position))?;

                Ok(())
            });
            self.apply_lua_director(lua_director);
            err
        };
        if let Err(err) = run_script() {
            tracing::warn!("Syntax error during onActorDeath: {err:?}");
        }
    }

    pub fn build_var_segment(&self) -> ServerZoneIpcSegment {
        ServerZoneIpcSegment::new(ServerZoneIpcData::DirectorVars {
            handler_id: self.id,
            flag: self.flag,
            branch: 0,
            data: self.data,
            unk1: 0,
            unk2: 0,
            unk3: 0,
            unk4: 0,
        })
    }

    fn create_lua_director(&self) -> LuaDirector {
        LuaDirector {
            data: self.data,
            tasks: Vec::new(),
            bosses: self.bosses.clone(),
        }
    }

    fn apply_lua_director(&mut self, lua: LuaDirector) {
        if self.data != lua.data {
            self.data = lua.data;
            self.tasks.push(LuaDirectorTask::SendVariables {});
        }
        if self.bosses != lua.bosses {
            self.bosses = lua.bosses;
        }
        self.tasks.extend_from_slice(&lua.tasks);
    }

    /// Actually insert tasks to seal the boss wall.
    pub fn seal_boss_wall(&mut self, id: u32, place_name: u32) {
        if let Some(boss) = self.bosses.iter_mut().find(|x| x.1.wall_id == id) {
            self.tasks.push(LuaDirectorTask::LogMessage {
                id: LogMessageType::BossRoomSealed as u32,
                params: vec![place_name],
            });
            self.tasks.push(LuaDirectorTask::ShowEObj { base_id: id });
            boss.1.state = DirectorBossState::Closed;
        }
    }

    /// Actually insert tasks to unseal the boss wall.
    pub fn unseal_boss_wall(&mut self, wall_id: u32, line_id: u32, place_name: u32) {
        self.tasks
            .push(LuaDirectorTask::HideEObj { base_id: wall_id });
        self.tasks
            .push(LuaDirectorTask::HideEObj { base_id: line_id });
        self.tasks.push(LuaDirectorTask::LogMessage {
            id: LogMessageType::BossRoomUnsealed as u32,
            params: vec![place_name],
        });
    }

    pub fn on_actor_aggro(&mut self, id: u32) {
        if let Some(boss) = self.bosses.get_mut(&id)
            && boss.state == DirectorBossState::Open
        {
            // TODO: is there times that are longer than 15 secs?
            self.tasks.push(LuaDirectorTask::LogMessage {
                id: LogMessageType::BossRoomSealing as u32,
                params: vec![boss.place_name, 15],
            });
            self.tasks.push(LuaDirectorTask::SealBossWall {
                actor_id: boss.actor_id,
                id: boss.wall_id,
                place_name: boss.place_name,
                time_until: 15,
            });
            boss.state = DirectorBossState::Aggro;
        }
    }

    pub fn on_gimmick_rect(&mut self, id: u32) {
        // Skip if the fucntion isn't defined'
        if !self
            .lua
            .0
            .globals()
            .contains_key("onGimmickRect")
            .unwrap_or_default()
        {
            return;
        }

        let mut run_script = || {
            let mut lua_director = self.create_lua_director();
            let err = self.lua.0.scope(|scope| {
                let data = scope.create_userdata_ref_mut(&mut lua_director)?;

                let func: Function = self.lua.0.globals().get("onGimmickRect")?;

                func.call::<()>((data, id))?;

                Ok(())
            });
            self.apply_lua_director(lua_director);
            err
        };
        if let Err(err) = run_script() {
            tracing::warn!("Syntax error during onGimmickRect: {err:?}");
        }
    }

    pub fn get_debug_shortcut(&self, id: u32) -> u32 {
        let run_script = || {
            self.lua.0.scope(|_| {
                let func: Function = self.lua.0.globals().get("getDebugShortcut")?;

                func.call::<u32>(id)
            })
        };
        match run_script() {
            Ok(pop_range_id) => pop_range_id,
            Err(err) => {
                tracing::warn!("Syntax error during getDebugShortcut: {err:?}");

                0
            }
        }
    }

    #[allow(unused)] // TODO: temporary after refactor
    pub fn variant_vote(&mut self, vote: u32) {
        let mut run_script = || {
            let mut lua_director = self.create_lua_director();
            let err = self.lua.0.scope(|scope| {
                let data = scope.create_userdata_ref_mut(&mut lua_director)?;

                let func: Function = self.lua.0.globals().get("onVariantVote")?;

                func.call::<()>((data, vote))?;

                Ok(())
            });
            self.apply_lua_director(lua_director);
            err
        };
        if let Err(err) = run_script() {
            tracing::warn!("Syntax error during onVariantVote: {err:?}");
        }
    }

    pub fn is_gimmick_functional(&self, id: u32) -> bool {
        let run_script = || {
            self.lua.0.scope(|_| {
                let func: Function = self.lua.0.globals().get("isGimmickFunctional")?;

                func.call::<bool>(id)
            })
        };
        match run_script() {
            Ok(value) => value,
            Err(err) => {
                tracing::warn!("Syntax error during isGimmickFunctional: {err:?}");

                true
            }
        }
    }
}

/// Calls the director's `onUpdate` with the health of every living NPC.
fn update_director(instance: &mut Instance) {
    let npc_health: HashMap<u32, u32> = instance
        .actors
        .values()
        .filter_map(|actor| match actor {
            NetworkedActor::Npc { spawn, .. }
                if spawn.common.health_points > 0 && spawn.common.max_health_points > 0 =>
            {
                Some((
                    spawn.common.layout_id,
                    spawn.common.health_points * 100 / spawn.common.max_health_points,
                ))
            }
            _ => None,
        })
        .collect();
    // The player's health too (in percent), e.g. for allies that heal them.
    let player_health = instance.actors.values().find_map(|actor| match actor {
        NetworkedActor::Player { spawn, .. } if spawn.common.max_health_points > 0 => {
            Some(spawn.common.health_points * 100 / spawn.common.max_health_points)
        }
        _ => None,
    });
    // Everyone's real numbers, position and target: living NPCs by layout ID, and "player".
    let layout_of: HashMap<ObjectId, u32> = instance
        .actors
        .iter()
        .filter_map(|(id, actor)| match actor {
            NetworkedActor::Npc { spawn, .. } => Some((*id, spawn.common.layout_id)),
            _ => None,
        })
        .collect();
    let players: Vec<ObjectId> = instance
        .actors
        .iter()
        .filter(|(_, actor)| matches!(actor, NetworkedActor::Player { .. }))
        .map(|(id, _)| *id)
        .collect();
    let mut info = HashMap::new();
    for actor in instance.actors.values() {
        let key = match actor {
            NetworkedActor::Npc { spawn, .. } if spawn.common.health_points > 0 => {
                spawn.common.layout_id.to_string()
            }
            NetworkedActor::Player { .. } => "player".to_string(),
            _ => continue,
        };
        let common = actor.get_common_spawn();
        let target = common.target_id.object_id;
        info.entry(key).or_insert(ActorInfo {
            hp: common.health_points,
            max_hp: common.max_health_points,
            position: common.position.0,
            target_layout: layout_of.get(&target).copied(),
            target_is_player: players.contains(&target),
        });
    }
    if let Some(director) = instance.directors.first_mut() {
        director.update(&npc_health, player_health, &info);
    }
}

/// Perform any queued director tasks
pub fn director_tick(
    network: Arc<Mutex<NetworkState>>,
    instance: &mut Instance,
    game_data: Arc<Mutex<GameData>>,
) {
    // Let the script see how the fight is going, if it wants to
    let wants_update = instance
        .directors
        .first()
        .is_some_and(|x| x.lua.0.globals().get::<Function>("onUpdate").is_ok());
    if wants_update {
        update_director(instance);
    }

    let tasks = if let Some(director) = &instance.directors.first() {
        director.tasks.clone()
    } else {
        return;
    };

    let mut bosses = if let Some(director) = &instance.directors.first() {
        director.bosses.clone()
    } else {
        return;
    };

    let director_id = instance.directors.first().as_ref().unwrap().id;

    for task in &tasks {
        match task {
            LuaDirectorTask::HideEObj { base_id } => {
                let Some(actor_id) = instance.find_object_by_eobj_id(*base_id) else {
                    tracing::warn!("Failed to find eobj {base_id} for HideEObj, it won't despawn!");
                    continue;
                };

                let state = EventState::OFF | EventState::UNK2 | EventState::UNK3;

                let mut network = network.lock();
                network.send_ac_in_range_instance(
                    instance,
                    actor_id,
                    ActorControlCategory::SetEventState { state },
                );

                // Update invisibility flags for next spawn
                if let Some(NetworkedActor::Object { object, .. }) =
                    instance.find_actor_mut(actor_id)
                {
                    object.event_state = state;
                }
            }
            LuaDirectorTask::ShowEObj { base_id } => {
                let Some(actor_id) = instance.find_object_by_eobj_id(*base_id) else {
                    tracing::warn!("Failed to find eobj {base_id} for ShowEObj, it won't despawn!");
                    continue;
                };

                let state = EventState::empty();

                let mut network = network.lock();
                network.send_ac_in_range_instance(
                    instance,
                    actor_id,
                    ActorControlCategory::SetEventState { state },
                );

                // Update invisibility flags for next spawn
                if let Some(NetworkedActor::Object { object, .. }) =
                    instance.find_actor_mut(actor_id)
                {
                    object.event_state = state;
                }
            }
            LuaDirectorTask::DeleteEObj { base_id } => {
                let Some(actor_id) = instance.find_object_by_eobj_id(*base_id) else {
                    tracing::warn!(
                        "Failed to find eobj {base_id} for DeleteEObj, it won't despawn!"
                    );
                    continue;
                };

                let mut network = network.lock();
                network.remove_actor(instance, actor_id);
            }
            LuaDirectorTask::SpawnEObj { base_id, position } => {
                if let Some(mut object) = instance.zone.get_event_object(*base_id) {
                    if let Some(position) = position {
                        object.position = *position;
                    }
                    instance.insert_object(object.entity_id, object, String::default()); // TODO: insert layer name
                } else {
                    tracing::warn!("Failed to find eobj {base_id} for SpawnEObj, it won't spawn!");
                }
            }
            LuaDirectorTask::SendVariables => {
                let vars = if let Some(director) = &instance.directors.first() {
                    director.build_var_segment()
                } else {
                    panic!("There's no way this could've happened!");
                };

                let mut network = network.lock();
                for id in instance
                    .actors
                    .iter()
                    .filter(|x| matches!(x.1, NetworkedActor::Player { .. }))
                    .map(|x| *x.0)
                {
                    let Some((handle, _)) = network.get_by_actor_mut(id) else {
                        continue;
                    };

                    let msg = FromServer::PacketSegment(vars.clone(), id);
                    let _ = handle.send(msg.clone()); // TODO: use result
                }
            }
            LuaDirectorTask::AbandonDuty { actor_id } => {
                let mut network = network.lock();
                network.send_to_by_actor_id(
                    *actor_id,
                    FromServer::LeaveContent,
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::BeginEventAction {
                actor_id,
                target,
                action_id,
            } => {
                let act = ActorControlCategory::EventAction {
                    unk1: 1,
                    id: *action_id,
                };

                let mut network = network.lock();
                network.send_to_by_actor_id(
                    *actor_id,
                    FromServer::ActorControlTarget(
                        *actor_id,
                        ObjectTypeId {
                            object_id: *target,
                            object_type: ObjectTypeKind::None,
                        },
                        act,
                    ),
                    DestinationNetwork::ZoneClients,
                );

                // TODO: set OccupiedInEvent?

                let cast_time = game_data.lock().get_event_action_cast_time(*action_id);
                instance.insert_task(
                    ClientId::default(),
                    *actor_id,
                    Duration::from_secs(cast_time as u64),
                    QueuedTaskData::CastEventAction { target: *target },
                );
            }
            LuaDirectorTask::FinishGimmickEvent { actor_id } => {
                let mut network = network.lock();
                network.send_to_by_actor_id(
                    *actor_id,
                    FromServer::FinishEvent,
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::LogMessage { id, params } => {
                let ipc = EventLogMessage {
                    handler_id: director_id,
                    message_type: *id,
                    params: params.to_vec(),
                }
                .package()
                .unwrap();

                let mut network = network.lock();
                network.send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::PacketSegment(ipc, ObjectId::default()), // TODO: how do we just send it from the player?
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::SpawnBattleNpc { id } => {
                if let Some(mut npc) = instance.zone.get_battle_npc(*id) {
                    npc.common.handler_id = director_id;
                    let config = get_config();
                    let mut game_data = game_data.lock();
                    instance.insert_npc(ObjectId(fastrand::u32(..)), npc, &config, &mut game_data);
                } else {
                    tracing::warn!("Failed to find bnpc {id} for SpawnBattleNpc, it won't spawn!");
                }
            }
            LuaDirectorTask::DespawnBattleNpc { layout_id } => {
                if let Some((actor_id, _)) = instance.find_npc(*layout_id) {
                    network.lock().remove_actor(instance, actor_id);
                }
            }
            LuaDirectorTask::GainEffect {
                actor_id,
                id,
                param,
                duration,
            } => {
                gain_effect_instance(
                    network.clone(),
                    ClientId::default(),
                    instance,
                    *actor_id,
                    *id,
                    *param,
                    *duration,
                    ObjectId::default(),
                    false, // Don't need to inform players here
                );
            }
            LuaDirectorTask::SetBGM { id } => {
                let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorControlSelf(
                    ActorControlSelf {
                        category: ActorControlCategory::DirectorEvent {
                            handler_id: director_id,
                            event: DirectorEvent::ContentDirector(ContentDirectorEvent::SetBGM {
                                bgm: *id,
                            }),
                        },
                    },
                ));

                let mut network = network.lock();
                network.send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::PacketSegment(ipc, ObjectId::default()), // TODO: how do we just send it from the player?
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::SealBossWall {
                actor_id,
                id,
                place_name,
                time_until,
            } => {
                instance.insert_task(
                    ClientId::default(),
                    *actor_id,
                    Duration::from_secs(*time_until as u64),
                    QueuedTaskData::SealBossWall {
                        id: *id,
                        place_name: *place_name,
                    },
                );
            }
            LuaDirectorTask::SpawnBoss {
                bnpc_id,
                wall_id,
                line_id,
                place_name,
            } => {
                if let Some(mut npc) = instance.zone.get_battle_npc(*bnpc_id) {
                    npc.common.handler_id = director_id;

                    let actor_id = ObjectId(fastrand::u32(..));
                    let config = get_config();
                    let mut game_data = game_data.lock();
                    instance.insert_npc(actor_id, npc, &config, &mut game_data);
                    bosses.insert(
                        *bnpc_id,
                        DirectorBoss {
                            state: DirectorBossState::Open,
                            actor_id,
                            wall_id: *wall_id,
                            line_id: *line_id,
                            place_name: *place_name,
                        },
                    );
                } else {
                    tracing::warn!("Failed to find bnpc {bnpc_id} for SpawnBoss, it won't spawn!");
                }
            }
            LuaDirectorTask::SpawnTreasure { id } => {
                if let Some(mut treasure) = instance.zone.get_treasure(*id) {
                    treasure.handler_id = director_id;

                    let actor_id = ObjectId(fastrand::u32(..));
                    instance.insert_treasure(actor_id, treasure);
                } else {
                    tracing::warn!(
                        "Failed to find treasure {id} for SpawnTreasure, it won't spawn!"
                    );
                }
            }
            LuaDirectorTask::VariantVoteRoute { .. } => {
                // TODO: Restore, this was removed after the refactor
                // let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorControlSelf(
                //     ActorControlSelf {
                //         category: ActorControlCategory::DirectorEvent {
                //             handler_id: director_id,
                //             event: DirectorEvent::VariantVoteRoute {
                //                 votes_needed: 1, // TODO: set to the number of players in the instance
                //                 npc_route: *npc_route,
                //             },
                //         },
                //     },
                // ));
                //
                // let mut network = network.lock();
                // network.send_to_instance(
                //     ObjectId::default(),
                //     instance,
                //     FromServer::PacketSegment(ipc, ObjectId::default()), // TODO: how do we just send it from the player?
                //     DestinationNetwork::ZoneClients,
                // );
            }
            LuaDirectorTask::CompleteQuestBattle {
                sequence,
                stay,
                finish_scene,
                finish_flags,
                variables,
            } => {
                let quest_id = instance.directors.first().unwrap().quest_battle_quest_id;
                let mut network = network.lock();
                network.send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::QuestBattleComplete(
                        director_id,
                        quest_id,
                        *sequence,
                        *stay,
                        *finish_scene,
                        *finish_flags,
                        variables.clone(),
                    ),
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::PlayScene { scene, flags } => {
                let mut network = network.lock();
                network.send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::PlayDirectorScene(director_id, *scene, *flags),
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::WarpPlayers { position, rotation } => {
                let players: Vec<ObjectId> = instance
                    .actors
                    .iter()
                    .filter(|(_, actor)| matches!(actor, NetworkedActor::Player { .. }))
                    .map(|(id, _)| *id)
                    .collect();
                let mut network = network.lock();
                for actor_id in players {
                    // As retail sent it: a normal warp, the character hidden, the usual fade.
                    let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::PrepareZoning(
                        PrepareZoning {
                            territory_type_id: instance.zone.id,
                            warp_type: WarpType::Normal,
                            hide_character: 1,
                            fade_out_delay: 1,
                            ..Default::default()
                        },
                    ));
                    network.send_to_by_actor_id(
                        actor_id,
                        FromServer::PacketSegment(ipc, actor_id),
                        DestinationNetwork::ZoneClients,
                    );
                    let Some(client_id) = network
                        .clients
                        .iter()
                        .find(|(_, (handle, _))| handle.actor_id == actor_id)
                        .map(|(id, _)| *id)
                    else {
                        continue;
                    };
                    let segment =
                        ServerZoneIpcSegment::new(ServerZoneIpcData::ActorSetPos(ActorSetPos {
                            position: *position,
                            rotation: *rotation,
                            warp_type: WarpType::Normal,
                            arg: 2,
                            ..Default::default()
                        }));
                    instance.insert_task(
                        client_id,
                        actor_id,
                        WARP_DELAY,
                        QueuedTaskData::PacketSegment { segment },
                    );
                }
            }
            LuaDirectorTask::LeaveQuestBattle => {
                let mut network = network.lock();
                network.send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::LeaveQuestBattle(director_id),
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::UseAction {
                layout_id,
                action_id,
                target,
            } => {
                let Some((actor_id, _)) = instance.find_npc(*layout_id) else {
                    tracing::warn!("Failed to find bnpc {layout_id} for UseAction!");
                    continue;
                };
                let target_id = match target {
                    Some(target) => instance.find_npc(*target).map(|(id, _)| id),
                    None => instance
                        .actors
                        .iter()
                        .find(|(_, actor)| matches!(actor, NetworkedActor::Player { .. }))
                        .map(|(id, _)| *id),
                };
                let Some(target_id) = target_id else {
                    continue;
                };
                crate::server::npc_behavior::npc_use_action(
                    &network, &game_data, instance, actor_id, *action_id, target_id,
                );
            }
            LuaDirectorTask::Say { .. } | LuaDirectorTask::SayBattleTalk { .. } => {
                let (layout_id, name_override) = match task {
                    LuaDirectorTask::Say { layout_id, .. } => (*layout_id, None),
                    LuaDirectorTask::SayBattleTalk {
                        layout_id, name_id, ..
                    } => (*layout_id, *name_id),
                    _ => unreachable!(),
                };
                // Someone who isn't around (e.g. the golem once it's gone) still gets their line said.
                let actor_id = instance
                    .find_npc(layout_id)
                    .map(|(actor_id, _)| actor_id)
                    .unwrap_or_default();
                let name_id = name_override.unwrap_or_else(|| {
                    instance
                        .find_actor(actor_id)
                        .map(|actor| actor.get_common_spawn().name_id)
                        .unwrap_or_default()
                });
                let quest_id = instance
                    .directors
                    .first()
                    .map(|director| director.quest_battle_quest_id)
                    .unwrap_or_default();
                let (name, text) = {
                    let mut game_data = game_data.lock();
                    let text = match task {
                        LuaDirectorTask::Say { key, .. } => game_data.get_quest_text(key),
                        LuaDirectorTask::SayBattleTalk { number, .. } => game_data
                            .get_quest_battle_talk(quest_id)
                            .get(number.saturating_sub(1))
                            .map(|(_, text)| text.clone()),
                        _ => unreachable!(),
                    };
                    // BNpcName 0 is a speaker whose name isn't given yet, shown as "???" like on retail.
                    let name = if name_id == 0 {
                        Some("???".to_string())
                    } else {
                        game_data.get_bnpc_name(name_id)
                    };
                    (name, text)
                };
                let Some(text) = text else {
                    tracing::warn!("Couldn't find the quest text for {task:?}!");
                    continue;
                };

                let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ChatMessage(ChatMessage {
                    sender_actor_id: actor_id,
                    channel: ChatChannelType::Say,
                    // BNpcName is lower case ("short-tempered thaumaturge"); retail shows "Short-tempered Thaumaturge".
                    sender_name: capitalize_words(&name.unwrap_or_default()),
                    message: text,
                    ..Default::default()
                }));
                let mut network = network.lock();
                network.send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::PacketSegment(ipc, actor_id),
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::BattleTalk {
                layout_id,
                name_id,
                text_id,
                icon,
                duration_ms,
            } => {
                let actor_id = instance
                    .find_npc(*layout_id)
                    .map(|(actor_id, _)| actor_id)
                    .unwrap_or_default();
                let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::DirectorPopupMessage {
                    actor_id: ObjectTypeId {
                        object_id: actor_id,
                        object_type: ObjectTypeKind::None,
                    },
                    handler_id: director_id,
                    npc_name: *name_id,
                    text_data_id: *text_id,
                    duration_msecs: *duration_ms,
                    icon: *icon,
                    kind: 2,
                    style: 0,
                    params_count: 0,
                    param1: 0,
                    param2: 0,
                });
                let mut network = network.lock();
                network.send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::PacketSegment(ipc, actor_id),
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::Yell {
                layout_id,
                npc_yell_id,
            } => {
                let Some((actor_id, _)) = instance.find_npc(*layout_id) else {
                    continue;
                };
                let name_id = game_data
                    .lock()
                    .get_npc_yell_name_id(*npc_yell_id)
                    .unwrap_or_default();
                let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::NpcYell {
                    object_id: ObjectTypeId {
                        object_id: actor_id,
                        object_type: ObjectTypeKind::None,
                    },
                    name_id,
                    npc_yell_id: *npc_yell_id,
                    param1: 0,
                    param2: 0,
                    param3: 0,
                    param4: 0,
                });
                let mut network = network.lock();
                network.send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::PacketSegment(ipc, actor_id),
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::SetSequence { sequence } => {
                let vars = if let Some(director) = instance.directors.first_mut() {
                    director.flag = *sequence;
                    director.build_var_segment()
                } else {
                    continue;
                };

                let mut network = network.lock();
                for id in instance
                    .actors
                    .iter()
                    .filter(|x| matches!(x.1, NetworkedActor::Player { .. }))
                    .map(|x| *x.0)
                {
                    let Some((handle, _)) = network.get_by_actor_mut(id) else {
                        continue;
                    };
                    let _ = handle.send(FromServer::PacketSegment(vars.clone(), id));
                }
            }
            LuaDirectorTask::FailQuestBattle => {
                let mut network = network.lock();
                network.send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::QuestBattleFailed,
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::SetAttackRange { layout_id, yalms } => {
                let Some((actor_id, _)) = instance.find_npc(*layout_id) else {
                    tracing::warn!("Failed to find bnpc {layout_id} for SetAttackRange!");
                    continue;
                };
                if let Some(NetworkedActor::Npc { timeline, .. }) =
                    instance.find_actor_mut(actor_id)
                {
                    timeline.attack_range = yalms.max(0.5);
                }
            }
            LuaDirectorTask::KeepNear {
                layout_id,
                anchors,
                yalms,
            } => {
                let player = instance
                    .actors
                    .iter()
                    .find(|(_, actor)| matches!(actor, NetworkedActor::Player { .. }))
                    .map(|(id, _)| *id);
                let anchor_ids: Vec<ObjectId> = anchors
                    .iter()
                    .filter_map(|anchor| match anchor {
                        Some(anchor) => instance.find_npc(*anchor).map(|(id, _)| id),
                        None => player,
                    })
                    .collect();
                let Some((actor_id, _)) = instance.find_npc(*layout_id) else {
                    tracing::warn!("Failed to find bnpc {layout_id} for KeepNear!");
                    continue;
                };
                if let Some(NetworkedActor::Npc { placement, .. }) =
                    instance.find_actor_mut(actor_id)
                {
                    match yalms {
                        Some(yalms) if !anchor_ids.is_empty() => {
                            placement.anchors = anchor_ids;
                            placement.within = *yalms;
                        }
                        _ => placement.anchors.clear(),
                    }
                }
            }
            LuaDirectorTask::StandBehind {
                layout_id,
                tank,
                yalms,
            } => {
                let tank_id = instance.find_npc(*tank).map(|(id, _)| id);
                let Some((actor_id, _)) = instance.find_npc(*layout_id) else {
                    tracing::warn!("Failed to find bnpc {layout_id} for StandBehind!");
                    continue;
                };
                if let Some(NetworkedActor::Npc { placement, .. }) =
                    instance.find_actor_mut(actor_id)
                {
                    placement.behind = tank_id.zip(*yalms);
                }
            }
            LuaDirectorTask::SetHate {
                layout_id,
                target,
                amount,
            } => {
                let target_id = match target {
                    Some(target) => instance.find_npc(*target).map(|(id, _)| id),
                    None => instance
                        .actors
                        .iter()
                        .find(|(_, actor)| matches!(actor, NetworkedActor::Player { .. }))
                        .map(|(id, _)| *id),
                };
                let Some(target_id) = target_id else {
                    tracing::warn!(
                        "Failed to find {target:?} for SetHate, {layout_id} won't attack!"
                    );
                    continue;
                };
                let Some((actor_id, _)) = instance.find_npc(*layout_id) else {
                    tracing::warn!("Failed to find bnpc {layout_id} for SetHate!");
                    continue;
                };

                // Picked up on the next NPC tick, the same way as being attacked.
                if let Some(actor) = instance.find_actor_mut(actor_id) {
                    actor.add_enmity(target_id, *amount);
                }
            }
            LuaDirectorTask::PlayCutscene { cutscene_id } => {
                let mut network = network.lock();
                network.send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::PlayDirectorCutscene(*cutscene_id, director_id),
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::UpdateShortcut { poprange_id } => {
                instance.directors.first_mut().unwrap().shortcut_poprange_id = Some(*poprange_id);
            }
            LuaDirectorTask::UseShortcut { actor_id } => {
                instance.insert_task(
                    ClientId::default(),
                    *actor_id,
                    Duration::from_secs(0),
                    QueuedTaskData::WarpToPopRange {
                        id: instance
                            .directors
                            .first()
                            .as_ref()
                            .unwrap()
                            .shortcut_poprange_id
                            .unwrap(),
                    },
                );
            }
            LuaDirectorTask::CompleteDuty => {
                // The clock stops: no more time-left re-sends.
                instance.duty_clock = None;
                let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorControlSelf(
                    ActorControlSelf {
                        category: ActorControlCategory::DirectorEvent {
                            handler_id: director_id,
                            event: DirectorEvent::ContentDirector(
                                ContentDirectorEvent::InstanceContent(
                                    InstanceContentDirectorEvent::DutyCompleted {
                                        arg1: 0,
                                        arg2: 0,
                                        arg3: 0,
                                        arg4: 0,
                                    },
                                ),
                            ),
                        },
                    },
                ));

                let mut network = network.lock();
                network.send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::PacketSegment(ipc, ObjectId::default()), // TODO: how do we just send it from the player?
                    DestinationNetwork::ZoneClients,
                );

                // TODO: mark duty as completed for each player
            }
            LuaDirectorTask::MapEffect { index, timeline_id } => {
                let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::DirectorMapEffect {
                    handler_id: director_id,
                    state: 1,
                    timeline_id: *timeline_id as u16,
                    index: *index as u8,
                });

                let mut network = network.lock();
                network.send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::PacketSegment(ipc, ObjectId::default()), // TODO: how do we just send it from the player?
                    DestinationNetwork::ZoneClients,
                );
            }
        }
    }

    if let Some(director) = &mut instance.directors.first_mut() {
        director.tasks.clear();
        director.bosses = bosses;
    }
}

/// Process director-related messages.
pub fn handle_director_messages(
    data: Arc<Mutex<WorldServer>>,
    network: Arc<Mutex<NetworkState>>,
    game_data: Arc<Mutex<GameData>>,
    msg: &ToServer,
) -> bool {
    match msg {
        ToServer::GimmickAccessor(id, from_actor_id, from_object_id, params) => {
            let mut data = data.lock();
            let Some(instance) = data.find_actor_instance_mut(*from_actor_id) else {
                tracing::warn!("Somehow failed to find an instance for actor?");
                return true;
            };

            let Some(base_id) = instance.find_base_id_by_actor_id(*from_object_id) else {
                tracing::warn!("Somehow failed to find base id from actor id {from_object_id}!");
                return true;
            };

            let Some(director) = &mut instance.directors.first_mut() else {
                tracing::warn!("Expected a director when recieving a GimmickAccessor?");
                return true;
            };

            let (gimmick_accessor_type, gimmick_param0, gimmick_param1, gimmick_param2);
            {
                let mut game_data = game_data.lock();
                (
                    gimmick_accessor_type,
                    gimmick_param0,
                    gimmick_param1,
                    gimmick_param2,
                ) = game_data.lookup_gimmick_accessor(*id).unwrap();
            }

            match gimmick_accessor_type {
                GimmickAccessorType::Shortcut => {
                    director.tasks.push(LuaDirectorTask::FinishGimmickEvent {
                        actor_id: *from_actor_id,
                    });
                    director.tasks.push(LuaDirectorTask::UseShortcut {
                        actor_id: *from_actor_id,
                    });
                }
                GimmickAccessorType::DutyExit
                | GimmickAccessorType::DeepDungeonExit
                | GimmickAccessorType::LeapOfFaithExit => {
                    // 0 means the user hit "Yes" in the prompt
                    if params[0] == 0 {
                        director.tasks.push(LuaDirectorTask::AbandonDuty {
                            actor_id: *from_actor_id,
                        });
                    }
                    director.tasks.push(LuaDirectorTask::FinishGimmickEvent {
                        actor_id: *from_actor_id,
                    });
                }
                GimmickAccessorType::GimmickPath => {
                    let pop_range_id = gimmick_param2;
                    let path_id = gimmick_param1;
                    let speed = 150; // TODO: dunno if this is defined somewhere

                    // TODO: Needs an EventAction before this happens I think
                    if let Some((position, rotation)) =
                        instance.zone.cached_pop_ranges.get(&pop_range_id)
                    {
                        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::WalkInEvent(
                            WalkInEvent {
                                path_id,
                                rotation: *rotation,
                                speed,
                                position: *position,
                                ..Default::default()
                            },
                        ));
                        let mut network = network.lock();
                        network.send_to_by_actor_id(
                            *from_actor_id,
                            FromServer::PacketSegment(ipc, *from_actor_id),
                            DestinationNetwork::ZoneClients,
                        );
                    }

                    director.tasks.push(LuaDirectorTask::FinishGimmickEvent {
                        actor_id: *from_actor_id,
                    });
                }
                GimmickAccessorType::Talk => {
                    // This handled by the client, we just need to finish it
                    director.tasks.push(LuaDirectorTask::FinishGimmickEvent {
                        actor_id: *from_actor_id,
                    });
                }
                GimmickAccessorType::FalliblePrompt => {
                    // 0 means the user hit "Yes" in the prompt
                    if params[0] == 0 {
                        if director.is_gimmick_functional(base_id) {
                            director.gimmick_accessor(*from_actor_id, base_id, params);
                            return true;
                        } else {
                            director.tasks.push(LuaDirectorTask::LogMessage {
                                id: gimmick_param1,
                                params: Vec::default(),
                            });
                        }
                    }

                    director.tasks.push(LuaDirectorTask::FinishGimmickEvent {
                        actor_id: *from_actor_id,
                    });
                }
                GimmickAccessorType::Message => {
                    if director.is_gimmick_functional(base_id) {
                        director.gimmick_accessor(*from_actor_id, base_id, params);
                    } else {
                        director.tasks.push(LuaDirectorTask::LogMessage {
                            id: gimmick_param0,
                            params: Vec::default(),
                        });
                        director.tasks.push(LuaDirectorTask::FinishGimmickEvent {
                            actor_id: *from_actor_id,
                        });
                    }
                }
                GimmickAccessorType::InfalliblePrompt => {
                    // 0 means the user hit "Yes" in the prompt
                    if params[0] == 0 {
                        director.gimmick_accessor(*from_actor_id, base_id, params);
                    } else {
                        director.tasks.push(LuaDirectorTask::FinishGimmickEvent {
                            actor_id: *from_actor_id,
                        });
                    }
                }
                _ => {
                    director.gimmick_accessor(*from_actor_id, base_id, params);
                }
            }

            true
        }
        ToServer::ReadyDirectorData(from_actor_id) => {
            let mut data = data.lock();
            let Some(instance) = data.find_actor_instance_mut(*from_actor_id) else {
                tracing::warn!("Somehow failed to find an instance for actor?");
                return true;
            };

            let mut network = network.lock();
            for director in &instance.directors {
                let flags = if instance
                    .content_settings
                    .unwrap_or_default()
                    .contains(DutyFinderSetting::EXPLORER_MODE)
                    && director.id.handler_type().is_content_director()
                {
                    1
                } else {
                    0
                };

                network.send_to_by_actor_id(
                    *from_actor_id,
                    FromServer::ActorControlSelf(ActorControlCategory::InitDirector {
                        handler_id: director.id,
                        content_id: if director.id.handler_type().is_content_director() {
                            instance.content_id
                        } else {
                            0xFFFF
                        },
                        flags,
                    }),
                    DestinationNetwork::ZoneClients,
                );
            }

            // Sync to new max level
            if let Some(synced_level) = instance.synced_level {
                network.send_to_by_actor_id(
                    *from_actor_id,
                    FromServer::SyncMaxLevel(synced_level, instance.synced_item_level),
                    DestinationNetwork::ZoneClients,
                );
            }

            true
        }
        ToServer::EnterTerritoryEvent(from_actor_id) => {
            let mut data = data.lock();
            let Some(instance) = data.find_actor_instance_mut(*from_actor_id) else {
                tracing::warn!("Somehow failed to find an instance for actor?");
                return true;
            };

            let mut network = network.lock();
            for director in &instance.directors {
                if director.id.handler_type().is_content_director() {
                    // Start the initial cutscene
                    network.send_to_by_actor_id(
                        *from_actor_id,
                        FromServer::PlayInitialCutscene(
                            instance.duration.unwrap_or_default().as_secs() as u32,
                            DutyOption::from_content_flags(
                                instance.content_settings.unwrap_or_default(),
                            )
                            .bits(),
                        ),
                        DestinationNetwork::ZoneClients,
                    );
                }
            }

            true
        }
        ToServer::DirectorSceneFinished(from_actor_id, scene) => {
            let mut data = data.lock();
            let Some(instance) = data.find_actor_instance_mut(*from_actor_id) else {
                return true;
            };
            if let Some(director) = instance.directors.first_mut() {
                director.scene_finished(*scene);
            }

            true
        }
        ToServer::ReadyToCommence(from_actor_id) => {
            let mut data = data.lock();
            let Some(instance) = data.find_actor_instance_mut(*from_actor_id) else {
                tracing::warn!("Somehow failed to find an instance for actor?");
                return true;
            };

            // Quest battles don't have the InstanceContent duty start, their script decides what happens.
            if let Some(director) = instance.directors.first_mut()
                && director.id.handler_type() == HandlerType::QuestBattle
            {
                director.commence();
                return true;
            }

            // TODO: wait for all players to be ready, not just the first one!

            // Have the director commence the duty
            let ipc =
                ServerZoneIpcSegment::new(ServerZoneIpcData::ActorControlSelf(ActorControlSelf {
                    category: ActorControlCategory::DirectorEvent {
                        handler_id: instance.directors.first().unwrap().id,
                        event: DirectorEvent::ContentDirector(
                            ContentDirectorEvent::InstanceContent(
                                InstanceContentDirectorEvent::DutyCommence {
                                    time_limit: instance.duration.unwrap_or_default().as_secs()
                                        as u32,
                                },
                            ),
                        ),
                    },
                }));
            let mut network = network.lock();
            network.send_to_by_actor_id(
                *from_actor_id,
                FromServer::PacketSegment(ipc, *from_actor_id),
                DestinationNetwork::ZoneClients,
            );

            let entrance_actor_id;
            let state = EventState::OFF | EventState::UNK2 | EventState::UNK3;

            {
                // Find the spawned entrance circle
                let Some(actor_id) = instance.find_entrance_circle() else {
                    tracing::warn!("Failed to find entrance circle, it won't despawn!");
                    return true;
                };
                entrance_actor_id = actor_id;

                // Update invisibility flags for next spawn
                if let Some(NetworkedActor::Object { object, .. }) =
                    instance.find_actor_mut(entrance_actor_id)
                {
                    object.event_state = state;
                    object.not_targetable = true;
                }
            }

            // Make the entrance circle invisible.
            let msg = FromServer::ActorControl(
                entrance_actor_id,
                ActorControlCategory::SetEventState { state },
            );

            network.send_to_by_actor_id(*from_actor_id, msg, DestinationNetwork::ZoneClients);

            // This begins the countdown
            let ipc =
                ServerZoneIpcSegment::new(ServerZoneIpcData::ActorControlSelf(ActorControlSelf {
                    category: ActorControlCategory::DirectorEvent {
                        handler_id: instance.directors.first().unwrap().id,
                        event: DirectorEvent::ContentDirector(
                            ContentDirectorEvent::SetDutyTimeRemaining {
                                time_remaining: instance.duration.unwrap_or_default().as_secs()
                                    as u32,
                            },
                        ),
                    },
                }));
            network.send_to_by_actor_id(
                *from_actor_id,
                FromServer::PacketSegment(ipc, *from_actor_id),
                DestinationNetwork::ZoneClients,
            );
            if instance.duty_clock.is_none() {
                let now = Instant::now();
                instance.duty_clock = Some((now, now));
            }

            true
        }
        _ => false,
    }
}

/// How often a duty's time left is re-sent (retail: every 300 s, Bob's Chronofoil captures.)
pub const DUTY_CLOCK_RESYNC: Duration = Duration::from_secs(300);

/// Re-sends a running duty's time left to its players every `DUTY_CLOCK_RESYNC`, as retail does.
pub fn resync_duty_clock(network: &mut NetworkState, instance: &mut Instance) {
    let (Some((started, last)), Some(duration), Some(director)) = (
        instance.duty_clock,
        instance.duration,
        instance.directors.first(),
    ) else {
        return;
    };
    let now = Instant::now();
    if now.duration_since(last) < DUTY_CLOCK_RESYNC {
        return;
    }
    instance.duty_clock = Some((started, now));
    let remaining = duration
        .saturating_sub(now.duration_since(started))
        .as_secs() as u32;
    let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorControlSelf(ActorControlSelf {
        category: ActorControlCategory::DirectorEvent {
            handler_id: director.id,
            event: DirectorEvent::ContentDirector(ContentDirectorEvent::SetDutyTimeRemaining {
                time_remaining: remaining,
            }),
        },
    }));
    let players: Vec<ObjectId> = instance
        .actors
        .iter()
        .filter(|(_, actor)| matches!(actor, NetworkedActor::Player { .. }))
        .map(|(id, _)| *id)
        .collect();
    for player in players {
        network.send_to_by_actor_id(
            player,
            FromServer::PacketSegment(ipc.clone(), player),
            DestinationNetwork::ZoneClients,
        );
    }
}

/// Handles an incoming director trigger from a client.
pub fn handle_director_trigger(
    network: Arc<Mutex<NetworkState>>,
    game_data: Arc<Mutex<GameData>>,
    from_actor_id: ObjectId,
    handler_id: HandlerId,
    trigger: &DirectorTrigger,
) {
    if handler_id.handler_type() == HandlerType::GimmickRect {
        let ipc =
            ServerZoneIpcSegment::new(ServerZoneIpcData::ActorControlSelf(ActorControlSelf {
                category: ActorControlCategory::DirectorEvent {
                    handler_id,
                    event: DirectorEvent::GimmickRect(GimmickRectEvent::Activate { arg1: 1 }),
                },
            }));
        let mut network = network.lock();
        network.send_to_by_actor_id(
            from_actor_id,
            FromServer::PacketSegment(ipc, from_actor_id),
            DestinationNetwork::ZoneClients,
        );
    }
    if handler_id.handler_type() == HandlerType::GoldSaucer
        && let DirectorTrigger::Unknown { id, .. } = trigger
        && *id == 0
    {
        let gate_id = 12; // TODO: GATE hardcoded
        let row;
        {
            let mut game_data = game_data.lock();
            let Some(gate_row) = game_data.get_gate(gate_id) else {
                return;
            };
            row = gate_row;
        }

        let mut network = network.lock();
        network.send_to_by_actor_id(
            from_actor_id,
            FromServer::ActorControlSelf(ActorControlCategory::DirectorEvent {
                handler_id,
                event: DirectorEvent::Unknown {
                    id: 9,
                    arg1: gate_id,
                    arg2: 4, // 4 for AFO, 12 for Cliffhanger?
                    arg3: 0,
                    arg4: 0,
                },
            }),
            DestinationNetwork::ZoneClients,
        );
        network.send_to_by_actor_id(
            from_actor_id,
            FromServer::ActorControlSelf(ActorControlCategory::DirectorEvent {
                handler_id,
                event: DirectorEvent::Unknown {
                    id: 6,
                    arg1: row.GFATEParams[0].LGBPopRange,
                    arg2: timestamp_secs() + 1000, // TODO: hardcoded duration
                    arg3: 0,
                    arg4: 0,
                },
            }),
            DestinationNetwork::ZoneClients,
        );
        network.send_to_by_actor_id(
            from_actor_id,
            FromServer::ActorControlSelf(ActorControlCategory::DirectorEvent {
                handler_id,
                event: DirectorEvent::Unknown {
                    id: 11,
                    arg1: 3,
                    arg2: 0,
                    arg3: 0,
                    arg4: 0,
                },
            }),
            DestinationNetwork::ZoneClients,
        );
    }

    // TODO: send triggers to Lua etc.
}

pub fn send_director_vars(
    data: Arc<Mutex<WorldServer>>,
    network: Arc<Mutex<NetworkState>>,
    game_data: Arc<Mutex<GameData>>,
    from_actor_id: ObjectId,
) {
    let mut data = data.lock();
    let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
        tracing::warn!("Somehow failed to find an instance for actor?");
        return;
    };

    let mut network = network.lock();
    for director in &instance.directors {
        if director.id.handler_type().is_content_director() {
            network.send_to_by_actor_id(
                from_actor_id,
                FromServer::PacketSegment(director.build_var_segment(), from_actor_id),
                DestinationNetwork::ZoneClients,
            );

            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::UnkDirector1 {
                unk: [
                    0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 38, 0, 0, 0,
                ],
            });
            network.send_to_by_actor_id(
                from_actor_id,
                FromServer::PacketSegment(ipc, from_actor_id),
                DestinationNetwork::ZoneClients,
            );

            let mut game_data = game_data.lock();
            if let Some(map_effects) = game_data.get_map_effects(instance.content_id as u32) {
                let mut states = Vec::new();
                for (i, layout_id) in map_effects.iter().enumerate() {
                    // A layout ID of zero means the effect should be skipped.
                    if *layout_id != 0 {
                        states.resize(i + 1, 0);
                        states[i] = 4; // 4 means to play it, I guess?
                    }
                }

                let ipc = MapEffects {
                    handler_id: director.id,
                    unk_flag: 5,
                    states,
                    ..Default::default()
                }
                .package()
                .unwrap();
                network.send_to_by_actor_id(
                    from_actor_id,
                    FromServer::PacketSegment(ipc, from_actor_id),
                    DestinationNetwork::ZoneClients,
                );
            }

            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::UnkDirector2 {
                unk: [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
            });
            network.send_to_by_actor_id(
                from_actor_id,
                FromServer::PacketSegment(ipc, from_actor_id),
                DestinationNetwork::ZoneClients,
            );
        }

        if director.id.handler_type() == HandlerType::PublicContent {
            let content_type;
            {
                let mut game_data = game_data.lock();
                content_type = game_data
                    .find_public_content_type(director.id.event_id())
                    .unwrap_or_default();
            }

            if content_type == PublicContentType::OccultCrescent {
                // Setup the panel
                let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::OccultCrescentSetup {
                    unk1: [
                        243, 152, 1, 0, 136, 182, 2, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 48, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 137, 21, 36, 0, 0,
                        0, 0, 0, 0, 0, 2, 5, 0, 1, 0, 0, 4, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 1, 223, 63, 0, 0, 0, 0, 0, 0, 0, 0,
                    ],
                });
                network.send_to_by_actor_id(
                    from_actor_id,
                    FromServer::PacketSegment(ipc, from_actor_id),
                    DestinationNetwork::ZoneClients,
                );

                // Setup duty actions
                network.send_to_by_actor_id(
                    from_actor_id,
                    FromServer::ActorControlSelf(ActorControlCategory::UnkDutyActions { unk1: 24 }),
                    DestinationNetwork::ZoneClients,
                );
                network.send_to_by_actor_id(
                    from_actor_id,
                    FromServer::ActorControlSelf(ActorControlCategory::EnableDutyActions {
                        enabled: true,
                    }),
                    DestinationNetwork::ZoneClients,
                );
                network.send_to_by_actor_id(
                    from_actor_id,
                    FromServer::ActorControlSelf(ActorControlCategory::SetDutyActions {
                        action_ids: [41588, 41589, 41590, 0, 0],
                    }),
                    DestinationNetwork::ZoneClients,
                );
            }
        }

        if director.id.handler_type() == HandlerType::TreasureHunt {
            network.send_to_by_actor_id(
                from_actor_id,
                FromServer::PacketSegment(director.build_var_segment(), from_actor_id),
                DestinationNetwork::ZoneClients,
            );
        }

        if director.id.handler_type().is_leve_director() {
            network.send_to_by_actor_id(
                from_actor_id,
                FromServer::PacketSegment(director.build_var_segment(), from_actor_id),
                DestinationNetwork::ZoneClients,
            );

            // Send a dummy commence for now
            // TODO: I think this can be extrapolated from Excel
            network.send_to_by_actor_id(
                from_actor_id,
                FromServer::ActorControlSelf(ActorControlCategory::DirectorEvent {
                    handler_id: director.id,
                    event: DirectorEvent::LeveDirector(LeveDirectorEvent::DutyCommence {
                        time_limit: 1200,
                        start_timestamp: timestamp_secs(),
                    }),
                }),
                DestinationNetwork::ZoneClients,
            );
        }

        if instance.zone.intended_use == TerritoryIntendedUse::CrystallineConflictCustomMatch {
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::SpectatorList {
                object_ids: [
                    from_actor_id,
                    ObjectId::default(),
                    ObjectId::default(),
                    ObjectId::default(),
                    ObjectId::default(),
                    ObjectId::default(),
                    ObjectId::default(),
                    ObjectId::default(),
                ],
                unk1: 1,
            });
            network.send_to_by_actor_id(
                from_actor_id,
                FromServer::PacketSegment(ipc, from_actor_id),
                DestinationNetwork::ZoneClients,
            );
        }
    }
}

/// Capitalizes the first letter of each word, the way retail shows NPC names in chat.
fn capitalize_words(name: &str) -> String {
    name.split(' ')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
