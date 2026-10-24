//! Executing status effect related functions.

use std::{sync::Arc, time::Duration};

use mlua::Function;
use parking_lot::Mutex;

use crate::{
    ClientId, FromServer, PlayerData, StatusEffects, ToServer,
    lua::{KawariLua, KawariLuaState, LuaPlayer, LuaZone},
    server::{
        WorldServer,
        instance::{Instance, QueuedTaskData},
        network::{DestinationNetwork, NetworkState},
    },
    zone_connection::BaseParameters,
};
use kawari::{
    common::ObjectId,
    ipc::zone::{
        ActorControlCategory, ServerZoneIpcData, ServerZoneIpcSegment, StatusEffect,
        StatusEffectList,
    },
};

/// Process status effect-related messages.
pub fn handle_effect_messages(
    data: Arc<Mutex<WorldServer>>,
    network: Arc<Mutex<NetworkState>>,
    lua: Arc<Mutex<KawariLua>>,
    msg: &ToServer,
) -> bool {
    match msg {
        ToServer::GainEffect(
            from_id,
            from_actor_id,
            effect_id,
            effect_param,
            effect_duration,
            effect_source_actor_id,
        ) => {
            gain_effect(
                network.clone(),
                data.clone(),
                *from_id,
                *from_actor_id,
                *effect_id,
                *effect_param,
                *effect_duration,
                *effect_source_actor_id,
                true,
            );

            true
        }
        ToServer::LoseEffect(
            from_id,
            from_actor_id,
            effect_id,
            effect_param,
            effect_source_actor_id,
        ) => {
            remove_effect(
                network.clone(),
                data.clone(),
                lua.clone(),
                *from_id,
                *from_actor_id,
                *effect_id,
                *effect_param,
                *effect_source_actor_id,
            );

            true
        }
        ToServer::SetCraftingState(_from_id, from_actor_id, cp, statuses) => {
            set_crafting_state(network.clone(), data.clone(), *from_actor_id, *cp, statuses);
            true
        }
        _ => false,
    }
}

/// A crafter's CP and synthesis statuses, sent in one StatusEffectList (the only packet with CP, see
/// `update_actor_hp_mp`.) Statuses not in the list are taken off.
fn set_crafting_state(
    network: Arc<Mutex<NetworkState>>,
    data: Arc<Mutex<WorldServer>>,
    from_actor_id: ObjectId,
    cp: u16,
    statuses: &[(u16, u16)],
) {
    let mut data = data.lock();
    let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
        return;
    };
    let Some(actor) = instance.find_actor_mut(from_actor_id) else {
        return;
    };

    let common = actor.get_common_spawn_mut();
    common.resource_points = cp.min(common.max_resource_points);

    if let Some(effects) = actor.status_effects_mut() {
        for id in crate::crafting::CRAFTING_STATUSES {
            match statuses.iter().find(|(status, _)| *status == id) {
                Some((_, param)) => {
                    if effects.get(id).map(|effect| effect.param) != Some(*param) {
                        // Steps, not seconds: these never run out by time (duration 0.)
                        effects.add(id, *param, 0.0, from_actor_id);
                    }
                }
                None => effects.remove(id),
            }
        }
        effects.reset_dirty();
    }

    send_effects_list(network, instance, from_actor_id);
}

pub fn send_effects_list(
    network: Arc<Mutex<NetworkState>>,
    instance: &Instance,
    from_actor_id: ObjectId,
) {
    let Some(actor) = instance.find_actor(from_actor_id) else {
        return;
    };

    let Some(status_effects) = actor.status_effects() else {
        return;
    };
    let common_spawn = actor.get_common_spawn();

    let mut statuses = [StatusEffect::default(); 30];
    let status_data = status_effects.data();
    let count = status_data.len().min(statuses.len());
    statuses[..count].copy_from_slice(&status_data[..count]);

    let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::StatusEffectList(StatusEffectList {
        statuses,
        classjob_id: common_spawn.class_job,
        level: common_spawn.level,
        flags: common_spawn.level, // NOTE: seen this set while level synced, but zero otherwise?
        health_points: common_spawn.health_points,
        max_health_points: common_spawn.max_health_points,
        resource_points: common_spawn.resource_points,
        max_resource_points: common_spawn.max_resource_points,
        ..Default::default()
    }));

    let mut network = network.lock();
    network.send_in_range_inclusive_instance(
        from_actor_id,
        instance,
        FromServer::PacketSegment(ipc, from_actor_id),
        DestinationNetwork::ZoneClients,
    );
}

/// Sends an updated status effects list, as needed.
pub fn process_effects_list(
    network: Arc<Mutex<NetworkState>>,
    instance: &mut Instance,
    from_actor_id: ObjectId,
) {
    let is_dirty;
    {
        let Some(actor) = instance.find_actor_mut(from_actor_id) else {
            return;
        };

        let Some(status_effects) = actor.status_effects() else {
            return;
        };

        is_dirty = status_effects.is_dirty();
    }

    // Only update the client if absolutely necessary (e.g. an effect is added, removed or changed duration)
    if is_dirty {
        send_effects_list(network, instance, from_actor_id);

        let Some(actor) = instance.find_actor_mut(from_actor_id) else {
            return;
        };

        actor.status_effects_mut().unwrap().reset_dirty();
    }
}

/// Gives the actor a new effect. You can also optionally send an ACS, if needed.
pub fn gain_effect(
    network: Arc<Mutex<NetworkState>>,
    data: Arc<Mutex<WorldServer>>,
    from_id: ClientId,
    from_actor_id: ObjectId,
    effect_id: u16,
    effect_param: u16,
    effect_duration: f32,
    effect_source_actor_id: ObjectId,
    inform_players: bool,
) -> u8 {
    let mut data = data.lock();
    let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
        return 0;
    };

    gain_effect_instance(
        network,
        from_id,
        instance,
        from_actor_id,
        effect_id,
        effect_param,
        effect_duration,
        effect_source_actor_id,
        inform_players,
    )
}

/// Gives the actor a new effect. You can also optionally send an ACS, if needed.
pub fn gain_effect_instance(
    network: Arc<Mutex<NetworkState>>,
    from_id: ClientId,
    instance: &mut Instance,
    from_actor_id: ObjectId,
    effect_id: u16,
    effect_param: u16,
    effect_duration: f32,
    effect_source_actor_id: ObjectId,
    inform_players: bool,
) -> u8 {
    let Some(actor) = instance.find_actor_mut(from_actor_id) else {
        return 0;
    };

    let Some(status_effects) = actor.status_effects_mut() else {
        return 0;
    };

    let index = status_effects.len() as u8;
    status_effects.add(
        effect_id,
        effect_param,
        effect_duration,
        effect_source_actor_id,
    );

    if inform_players {
        {
            let mut network = network.lock();

            // Retail never fills in the source here (5,367 of 5,367 in Bob's Chronofoil captures.)
            let ipc = ActorControlCategory::GainEffect {
                effect_id: effect_id as u32,
                param: effect_param as u32,
                source_actor_id: ObjectId(0),
            };

            // Then, Send an actor control to inform the client if needed
            network.send_ac_in_range_inclusive_instance(instance, from_actor_id, ipc);
        }

        // We also need to send them an updated StatusEffectsList
        process_effects_list(network.clone(), instance, from_actor_id);
    }

    // If this effect was already active, its old expiry would remove the refreshed effect early. Cancel it.
    instance.retain_tasks(|task| {
        !(task.from_actor_id == from_actor_id
            && matches!(task.data, QueuedTaskData::LoseStatusEffect { effect_id: id, .. } if id == effect_id))
    });

    // Scheduling doesn't make sense when the effect never ends.
    if effect_duration == 0.0 {
        return index;
    }

    instance.insert_task(
        from_id,
        from_actor_id,
        Duration::from_secs_f32(effect_duration),
        QueuedTaskData::LoseStatusEffect {
            effect_id,
            effect_param,
            effect_source_actor_id,
        },
    );

    index
}

/// Removes an effect from the actor.
pub fn remove_effect(
    network: Arc<Mutex<NetworkState>>,
    data: Arc<Mutex<WorldServer>>,
    lua: Arc<Mutex<KawariLua>>,
    from_id: ClientId,
    from_actor_id: ObjectId,
    effect_id: u16,
    effect_param: u16,
    effect_source_actor_id: ObjectId,
) {
    // Remove it from our internal data model first
    {
        let mut data = data.lock();

        let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
            return;
        };

        let Some(actor) = instance.find_actor_mut(from_actor_id) else {
            return;
        };

        let Some(status_effects) = actor.status_effects_mut() else {
            return;
        };

        // If we don't have the status effect, just do nothing
        if status_effects.get(effect_id).is_none() {
            return;
        }

        status_effects.remove(effect_id);
    }

    // Then send the actor control to lose the effect
    {
        let mut network = network.lock();
        let data = data.lock();

        let ipc = ActorControlCategory::LoseEffect {
            effect_id: effect_id as u32,
            unk2: effect_param as u32,
            source_actor_id: effect_source_actor_id,
        };
        network.send_ac_in_range_inclusive(&data, from_actor_id, ipc);
    }

    // Finally, inform the client of their new status effects list
    {
        let mut data = data.lock();

        let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
            return;
        };
        process_effects_list(network.clone(), instance, from_actor_id);
    }

    // Also run the effect's Lua script in case it wants to do something!
    {
        let lua = lua.lock();
        let state = lua.0.app_data_ref::<KawariLuaState>().unwrap();

        let mut lua_player = LuaPlayer {
            player_data: PlayerData::default(),
            status_effects: StatusEffects::default(),
            queued_tasks: Vec::new(),
            zone_data: LuaZone::default(),
            base_parameters: BaseParameters::default(),
            current_level: 0,
            scene_error_code: 0,
            ..Default::default()
        };

        let key = effect_id as u32;
        if let Some(effect_script) = state.effect_scripts.get(&key) {
            lua.0
                .scope(|scope| {
                    let connection_data = scope.create_userdata_ref_mut(&mut lua_player).unwrap();

                    lua.0
                        .load(
                            std::fs::read(effect_script)
                                .expect("Failed to locate scripts directory!"),
                        )
                        .set_name("@".to_string() + effect_script)
                        .exec()
                        .unwrap();

                    let func: Function = lua.0.globals().get("onLose").unwrap();

                    func.call::<()>(connection_data).unwrap();

                    Ok(())
                })
                .unwrap();
        } else {
            // Most statuses need no script (their numbers are in `modifiers`), so this isn't worth a warning.
            tracing::debug!("Effect {effect_id} isn't scripted, nothing to run.");
        }

        // Inform the client of any new Lua tasks
        let mut network = network.lock();
        let msg = FromServer::NewTasks(lua_player.queued_tasks);
        network.send_to(from_id, msg, DestinationNetwork::ZoneClients);
    }
}
