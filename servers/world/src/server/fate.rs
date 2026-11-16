use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use kawari::{
    common::{
        FATE_TIME_LIMIT, FateRule, FateState, HandlerId, HandlerType, ObjectId, Position,
        timestamp_secs,
    },
    config::get_config,
    ipc::zone::{
        ActorControlCategory, ActorControlSelf, DisplayFlag, ServerZoneIpcData,
        ServerZoneIpcSegment,
    },
};
use mlua::{Function, UserData, UserDataMethods};
use parking_lot::Mutex;

use crate::{
    ClientId, FromServer, GameData,
    lua::{KawariLua, KawariLuaState},
    server::{
        actor::NetworkedActor,
        instance::{Instance, QueuedTaskData},
        network::{DestinationNetwork, NetworkState},
    },
};

#[derive(Debug, Clone, Default)]
pub struct FateInstance {
    /// Index into the Fate Excel sheet.
    pub fate_id: u32,
    /// When this FATE was started.
    pub start_timestamp: u32,
    /// The current state of the FATE.
    pub fate_state: FateState,
    pub data: FateData,
    /// The start NPC for this FATE, if applicable.
    pub motivation_npc: Option<(ObjectId, Position)>,
    /// Current progress of this FATE.
    pub progress: u32,
    /// A FATE that is one enemy to defeat (`fate:spawn_boss`): its layout ID. The FATE's progress is the share of
    /// its health taken (`boss_progress`), told every `BOSS_PROGRESS_EVERY`.
    pub boss: Option<u32>,
    /// The boss has been seen on the field (gone after that, it was defeated.)
    pub boss_seen: bool,
    /// When the boss's progress is next told.
    pub next_boss_progress: Option<Instant>,
}

/// A boss FATE's FateUpdate comes every 1.33 s while it runs, changed or not (FATE 586 in Bob's Chronofoil capture
/// 651981e7 of 2026-11-16: 229 of them over the 306 s its enemy lived, 1.3 to 1.4 s apart.)
pub const BOSS_PROGRESS_EVERY: Duration = Duration::from_millis(1330);
/// After the boss falls: the FATE's state goes to 5 with its Defeated (2.0 s after the death, 0.1 s more), to 7 and
/// 10 a second later, and to 0 6.5 s after that (the same capture; FATE 588's two ends there had the same four.)
pub const BOSS_ENDING_AFTER: Duration = Duration::from_millis(2100);
pub const BOSS_ENDED_AFTER: Duration = Duration::from_millis(3100);
pub const FATE_GONE_AFTER: Duration = Duration::from_millis(6500);

/// A boss FATE's progress: the share of the boss's health taken, in whole percent, rounded down (retail's FateUpdates
/// against its enemy's health at the time: 210 of 229 exactly, the other 19 one above, a hit's result a moment
/// ahead of the health seen.) 100 only when it has none left.
pub fn boss_progress(health: u32, max_health: u32) -> u32 {
    if health == 0 || max_health == 0 {
        return 100;
    }
    let taken = max_health.saturating_sub(health) as u64;
    ((taken * 100 / max_health as u64) as u32).min(99)
}

impl FateInstance {
    pub fn new(fate_id: u32, lua: KawariLua, game_data: &mut GameData) -> Option<Self> {
        let fate_rule = game_data.get_fate_rule(fate_id).unwrap_or_default();
        let fate_state = match fate_rule {
            FateRule::Invalid => return None,
            FateRule::Gathering => FateState::Preparing,
            _ => FateState::Running,
        };

        // Setup Lua state
        let script_name;
        {
            let state = lua.0.app_data_ref::<KawariLuaState>().unwrap();
            script_name = state.fate_scripts.get(&fate_id).cloned();
        }

        let mut data = FateData::default();

        if let Some(script) = &script_name {
            let file = std::fs::read(script).unwrap();

            if let Err(err) = lua.0.load(file).set_name("@".to_string() + script).exec() {
                tracing::warn!(
                    "Syntax error in {}: {:?} the FATE won't be scripted!",
                    script,
                    err
                );
            } else {
                data.lua = lua;

                // Call into the onSetup function before returning, as we need the flag to be initialized before any players change zones.
                data.setup();
            }
        } else {
            tracing::warn!("FATE {fate_id} isn't scripted yet!");
        }

        Some(Self {
            fate_id,
            fate_state,
            data,
            ..Default::default()
        })
    }

    /// Returns the packet that updates the start timestamp and time limit for this FATE.
    pub fn create_unk_fate_packet(&self) -> ServerZoneIpcSegment {
        ServerZoneIpcSegment::new(ServerZoneIpcData::UnkFate {
            fate_id: self.fate_id,
            unk1: 0,
            start_timestamp: self.start_timestamp,
            unk3: 0,
            time_limit: FATE_TIME_LIMIT.as_secs() as u32,
            unk5: 0,
        })
    }

    /// Returns the ActorControl that updates the state for this FATE.
    pub fn create_fate_init_ac(&self) -> ActorControlCategory {
        ActorControlCategory::FateInit {
            fate_id: self.fate_id,
            fate_state: self.fate_state,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LuaFateTask {
    SpawnBattleNpc {
        id: u32,
    },
    SpawnMotivationNpc {
        id: u32,
    },
    SetProgress {
        progress: u32,
    },
    /// The FATE's one enemy (`fate:spawn_boss`): spawned as `SpawnBattleNpc`, and the FATE follows its health.
    SpawnBoss {
        id: u32,
    },
}

// TODO: Maybe collapse into FateData?
#[derive(Default, Debug)]
pub struct LuaFate {
    pub tasks: Vec<LuaFateTask>,
}

impl UserData for LuaFate {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("spawn_bnpc", |_, this, id: u32| {
            this.tasks.push(LuaFateTask::SpawnBattleNpc { id });
            Ok(())
        });
        methods.add_method_mut("spawn_motivation_npc", |_, this, id: u32| {
            this.tasks.push(LuaFateTask::SpawnMotivationNpc { id });
            Ok(())
        });
        // The FATE's one enemy, by its layout ID: the FATE's progress is the health it has lost, and it's over when
        // the enemy falls.
        methods.add_method_mut("spawn_boss", |_, this, id: u32| {
            this.tasks.push(LuaFateTask::SpawnBoss { id });
            Ok(())
        });
        methods.add_method_mut("set_progress", |_, this, progress: u32| {
            this.tasks.push(LuaFateTask::SetProgress { progress });
            Ok(())
        });
    }
}

#[derive(Debug, Default, Clone)]
pub struct FateData {
    /// Lua state for this FATE.
    pub lua: KawariLua,
    pub tasks: Vec<LuaFateTask>,
}

impl FateData {
    pub fn setup(&mut self) {
        let mut run_script = || {
            let mut lua_director = self.create_lua_fate();
            let err = self.lua.0.scope(|scope| {
                let data = scope.create_userdata_ref_mut(&mut lua_director)?;

                let func: Function = self.lua.0.globals().get("onSetup")?;

                func.call::<()>(data)?;

                Ok(())
            });
            self.apply_lua_fate(lua_director);
            err
        };
        if let Err(err) = run_script() {
            tracing::warn!("Syntax error during onSetup: {err:?}");
        }
    }

    fn create_lua_fate(&self) -> LuaFate {
        LuaFate { tasks: Vec::new() }
    }

    fn apply_lua_fate(&mut self, lua: LuaFate) {
        self.tasks.extend_from_slice(&lua.tasks);
    }

    pub fn on_actor_death(&mut self) {
        // Not every FATE script cares who dies.
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
            let mut lua_fate = self.create_lua_fate();
            let err = self.lua.0.scope(|scope| {
                let data = scope.create_userdata_ref_mut(&mut lua_fate)?;

                let func: Function = self.lua.0.globals().get("onActorDeath")?;

                func.call::<()>(data)?;

                Ok(())
            });
            self.apply_lua_fate(lua_fate);
            err
        };
        if let Err(err) = run_script() {
            tracing::warn!("Syntax error during onActorDeath: {err:?}");
        }
    }
}

/// Perform any queued FATE tasks
pub fn fate_tick(
    network: Arc<Mutex<NetworkState>>,
    instance: &mut Instance,
    game_data: Arc<Mutex<GameData>>,
) {
    let mut fates = instance.fates.clone();
    let mut fates_that_updated = Vec::new();
    for fate in &mut fates {
        let tasks = fate.data.tasks.clone();
        let fate_id = fate.fate_id;

        for task in &tasks {
            if let LuaFateTask::SpawnBoss { id } = task {
                fate.boss = Some(*id);
            }
            match task {
                LuaFateTask::SpawnBattleNpc { id } | LuaFateTask::SpawnBoss { id } => {
                    if let Some(mut npc) = instance.zone.get_battle_npc(*id) {
                        npc.common.fate_id = fate_id as u16;
                        npc.common.handler_id = HandlerId::new(HandlerType::Fate, 65535);
                        let config = get_config();
                        let mut game_data = game_data.lock();
                        instance.insert_npc(
                            ObjectId(fastrand::u32(..)),
                            npc,
                            &config,
                            &mut game_data,
                        );
                    } else {
                        tracing::warn!(
                            "Failed to find bnpc {id} for SpawnBattleNpc, it won't spawn!"
                        );
                    }
                }
                LuaFateTask::SpawnMotivationNpc { id } => {
                    if let Some(mut npc) = instance.zone.get_battle_npc(*id) {
                        npc.common.fate_id = fate_id as u16;
                        npc.common.handler_id = HandlerId::new(HandlerType::Fate, 65535);
                        npc.common.display_flags = DisplayFlag::FATE_START_NPC;
                        let config = get_config();
                        let mut game_data = game_data.lock();
                        instance.insert_npc(
                            ObjectId(fastrand::u32(..)),
                            npc,
                            &config,
                            &mut game_data,
                        );
                    } else {
                        tracing::warn!(
                            "Failed to find bnpc {id} for SpawnBattleNpc, it won't spawn!"
                        );
                    }

                    if let Some((object_id, position)) = instance.find_npc(*id) {
                        // TODO: consolidate this code with the one in Instance
                        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorControlSelf(
                            ActorControlSelf {
                                category: ActorControlCategory::SetupMotivationNpc {
                                    fate_id,
                                    motivation_npc: object_id,
                                    unk1: 2175,
                                    x: position.0.x,
                                    y: position.0.y,
                                    z: position.0.z,
                                },
                            },
                        ));

                        let mut network = network.lock();
                        network.send_to_instance(
                            ObjectId::default(),
                            instance,
                            FromServer::PacketSegment(ipc, ObjectId::default()),
                            DestinationNetwork::ZoneClients,
                        );

                        // Set the motivation NPC for new players spawning in
                        fate.motivation_npc = Some((object_id, position));
                    } else {
                        tracing::warn!(
                            "Failed to find bnpc {id} for SetMotivationNpc, it won't work!"
                        );
                    }
                }
                LuaFateTask::SetProgress { progress } => {
                    fate.progress = *progress;
                    fates_that_updated.push((fate_id, *progress));
                }
            }
        }
    }

    for (fate, new_fate) in instance.fates.iter_mut().zip(fates) {
        fate.data.tasks.clear();
        fate.motivation_npc = new_fate.motivation_npc;
        fate.progress = new_fate.progress;
        fate.boss = new_fate.boss;
    }
    fates_that_updated.extend(boss_tick(instance, Instant::now()));

    send_fate_updates(&mut network.lock(), instance, fates_that_updated);
}

/// Boss FATEs (`fate:spawn_boss`): each one's progress from its boss's health, when it's time to tell it again or
/// the boss has fallen. Returns the FATEs to tell, with their progress.
pub fn boss_tick(instance: &mut Instance, now: Instant) -> Vec<(u32, u32)> {
    let mut updates = Vec::new();
    for index in 0..instance.fates.len() {
        let fate = &instance.fates[index];
        let Some(layout) = fate.boss else {
            continue;
        };
        if fate.fate_state != FateState::Running || fate.progress >= 100 {
            continue;
        }
        let health = instance
            .find_npc(layout)
            .and_then(|(id, _)| instance.find_actor(id))
            .map(|actor| {
                let common = actor.get_common_spawn();
                (common.health_points, common.max_health_points)
            });
        let fate = &mut instance.fates[index];
        let fallen = match health {
            Some((health, _)) => {
                fate.boss_seen = true;
                health == 0
            }
            None => fate.boss_seen,
        };
        if !fallen && fate.next_boss_progress.is_some_and(|at| now < at) {
            continue;
        }
        fate.next_boss_progress = Some(now + BOSS_PROGRESS_EVERY);
        fate.progress = match health {
            _ if fallen => 100,
            Some((health, max_health)) => boss_progress(health, max_health),
            None => 0,
        };
        updates.push((fate.fate_id, fate.progress));
    }
    updates
}

/// Tells everyone in the instance these FATEs' progress, and queues the end of those at 100.
pub fn send_fate_updates(
    network: &mut NetworkState,
    instance: &mut Instance,
    fates_that_updated: Vec<(u32, u32)>,
) {
    for (fate_id, progress) in fates_that_updated {
        network.send_to_instance(
            ObjectId::default(),
            instance,
            FromServer::ActorControlSelf(ActorControlCategory::FateUpdate {
                fate_id,
                progress,
                param: 0,
            }),
            DestinationNetwork::ZoneClients,
        );

        if progress == 100 {
            // A boss FATE ends as retail's did: state 5 with the boss's Defeated, 7 and 10 a second later.
            let is_boss = instance
                .fates
                .iter()
                .any(|fate| fate.fate_id == fate_id && fate.boss.is_some());
            if is_boss {
                instance.insert_task(
                    ClientId::default(),
                    ObjectId::default(),
                    BOSS_ENDING_AFTER,
                    QueuedTaskData::FateState {
                        fate_id,
                        state: FateState::Ending,
                    },
                );
                instance.insert_task(
                    ClientId::default(),
                    ObjectId::default(),
                    BOSS_ENDED_AFTER,
                    QueuedTaskData::EndFate { fate_id },
                );
                instance.insert_task(
                    ClientId::default(),
                    ObjectId::default(),
                    BOSS_ENDED_AFTER + FATE_GONE_AFTER,
                    QueuedTaskData::FateState {
                        fate_id,
                        state: FateState::Unk0,
                    },
                );
                continue;
            }
            // TODO: Introduce the slight delay seen in retail (higher for turn-in FATEs)
            instance.insert_task(
                ClientId::default(),
                ObjectId::default(),
                Duration::ZERO,
                QueuedTaskData::EndFate { fate_id },
            );
        }
    }
}

pub fn inform_fate_spawn(network: &mut NetworkState, from_actor_id: ObjectId, fate: &FateInstance) {
    let send_motivation_npc = |network: &mut NetworkState| {
        if let Some((object_id, position)) = fate.motivation_npc {
            network.send_to_by_actor_id(
                from_actor_id,
                FromServer::ActorControlSelf(ActorControlCategory::SetupMotivationNpc {
                    fate_id: fate.fate_id,
                    motivation_npc: object_id,
                    unk1: 2175,
                    x: position.0.x,
                    y: position.0.y,
                    z: position.0.z,
                }),
                DestinationNetwork::ZoneClients,
            );
        }
    };

    // TODO: maybe only for newly spawned fates?
    network.send_to_by_actor_id(
        from_actor_id,
        FromServer::ActorControlSelf(ActorControlCategory::FateInit {
            fate_id: fate.fate_id,
            fate_state: FateState::Unk1,
        }),
        DestinationNetwork::ZoneClients,
    );

    send_motivation_npc(network);

    network.send_to_by_actor_id(
        from_actor_id,
        FromServer::ActorControlSelf(ActorControlCategory::CreateFateContext {
            fate_id: fate.fate_id,
            is_bonus: 0,
        }),
        DestinationNetwork::ZoneClients,
    );

    if fate.fate_state == FateState::Running {
        network.send_to_by_actor_id(
            from_actor_id,
            FromServer::PacketSegment(fate.create_unk_fate_packet(), from_actor_id),
            DestinationNetwork::ZoneClients,
        );
    }

    network.send_to_by_actor_id(
        from_actor_id,
        FromServer::ActorControlSelf(fate.create_fate_init_ac()),
        DestinationNetwork::ZoneClients,
    );

    // Yes, retail does send it twice.
    send_motivation_npc(network);

    network.send_to_by_actor_id(
        from_actor_id,
        FromServer::ActorControlSelf(ActorControlCategory::FateUpdateTargetableStatus {
            fate_id: fate.fate_id,
        }),
        DestinationNetwork::ZoneClients,
    );
}

pub fn inform_fate_spawn_globally(
    instance: &mut Instance,
    network: &mut NetworkState,
    fate: &FateInstance,
) {
    for actor in instance
        .actors
        .iter()
        .filter(|x| matches!(x.1, NetworkedActor::Player { .. }))
        .map(|x| *x.0)
    {
        inform_fate_spawn(network, actor, fate);
    }
}

/// Move a FATE from Preparing to Running. For a version that networks to other players, use `start_fate`.
pub fn internally_start_fate(instance: &mut Instance, fate_id: u32) {
    {
        let Some(fate) = instance.find_fate_by_id_mut(fate_id) else {
            return;
        };

        fate.fate_state = FateState::Running;
        fate.start_timestamp = timestamp_secs();
    }

    instance.insert_task(
        ClientId::default(),
        ObjectId::default(),
        FATE_TIME_LIMIT,
        QueuedTaskData::EndFate { fate_id },
    );
}

/// Move a FATE from Preparing to Running.
pub fn start_fate(network: &mut NetworkState, instance: &mut Instance, fate_id: u32) {
    internally_start_fate(instance, fate_id);

    let fate_init_ac;
    let fate_unk_packet;
    let motivation_npc;
    {
        let Some(fate) = instance.find_fate_by_id_mut(fate_id) else {
            return;
        };

        fate_init_ac = fate.create_fate_init_ac();
        fate_unk_packet = fate.create_unk_fate_packet();
        motivation_npc = fate.motivation_npc.map(|x| x.0);
    }

    network.send_to_instance(
        ObjectId::default(),
        instance,
        FromServer::ActorControlSelf(fate_init_ac),
        DestinationNetwork::ZoneClients,
    );

    network.send_to_instance(
        ObjectId::default(),
        instance,
        FromServer::PacketSegment(fate_unk_packet, Default::default()),
        DestinationNetwork::ZoneClients,
    );

    if let Some(motivation_npc) = motivation_npc {
        network.send_to_instance(
            ObjectId::default(),
            instance,
            FromServer::ActorControlSelf(ActorControlCategory::SetCollectionPointNpc {
                fate_id,
                motivation_npc,
            }),
            DestinationNetwork::ZoneClients,
        );
    }
}

/// Move a FATE to Ended.
pub fn ended_fate(fate: &mut FateInstance) {
    fate.fate_state = FateState::Ended;
}

/// Move a FATE to Unk10.
pub fn unk10_fate(fate: &mut FateInstance) {
    fate.fate_state = FateState::Unk10;
}

/// Spawns a random FATE from the candiate pool.
pub fn start_next_fate(
    network: &mut NetworkState,
    game_data: &mut GameData,
    lua: &KawariLua,
    instance: &mut Instance,
) {
    // Remove any FATEs that already spawned.
    let mut candidate_fates = instance.candiate_fates.clone();
    candidate_fates.retain(|x| instance.fates.iter().find(|y| y.fate_id == *x).is_none());
    if let Some(fate_id) = fastrand::choice(candidate_fates) {
        spawn_fate(network, game_data, lua, instance, fate_id);
    }
}

/// Spawns the specified FATE.
pub fn spawn_fate(
    network: &mut NetworkState,
    game_data: &mut GameData,
    lua: &KawariLua,
    instance: &mut Instance,
    fate_id: u32,
) {
    let Some(new_instance) = FateInstance::new(fate_id, lua.clone(), game_data) else {
        return;
    };
    instance.fates.push(new_instance);
    let fate = instance.fates.last().unwrap().clone();
    inform_fate_spawn_globally(instance, network, &fate);

    if fate.fate_state == FateState::Running {
        start_fate(network, instance, fate_id);
    }
}
