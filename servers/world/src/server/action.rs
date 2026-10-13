//! Executing actions and other related functions.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use mlua::Function;
use parking_lot::Mutex;

use crate::{
    ActionInfo, ClientId, FromServer, GameData, PlayerData, StatusEffects, ToServer,
    lua::{EffectsBuilder, KawariLua, KawariLuaState, LuaPlayer, LuaZone},
    server::{
        WorldServer,
        actor::{NetworkedActor, create_npc_common_spawn, update_actor_hp_mp},
        effect::{gain_effect, gain_effect_instance, process_effects_list},
        instance::{Instance, QueuedTaskData},
        network::{DestinationNetwork, NetworkState},
        party::get_party_id_from_actor_id,
        periodic::add_periodic_effect,
    },
    zone_connection::{BaseParameters, TeleportQuery},
};
use kawari::{
    common::{
        ANIMATION_LOCK_TIME, COMBO_TIMEOUT, CharacterMode, ObjectId, ObjectTypeId, ObjectTypeKind,
        STRIKING_DUMMY_NAME_ID,
    },
    config::get_config,
    ipc::zone::{
        ActionEffect, ActionRequest, ActionType, ActorControlCategory, BattleNpcSubKind,
        CommonSpawn, EffectEntry, EffectResult, ObjectKind, ServerZoneIpcData,
        ServerZoneIpcSegment, SpawnNpc, TargetEffect, TargetEffectKind,
    },
};

/// Status effect: Swiftcast. The next spell is cast immediately.
const EFFECT_SWIFTCAST: u16 = 167;

/// Status effect: Presence of Mind. Reduces cast times by 20%.
const EFFECT_PRESENCE_OF_MIND: u16 = 157;

/// Status effect: Thin Air. The next action costs no MP.
const EFFECT_THIN_AIR: u16 = 1217;

/// Extra leniency (in yalms) given to area-of-effect checks, to roughly account for target hitboxes.
const AOE_HITBOX_LENIENCY: f32 = 1.0;

/// How much of their damage NPCs deal to players, in percent.
///
/// NPC damage is worked out like a player's (strength or intelligence times potency), which comes out far too high
/// against players: in Chasing Shadows a level 5 Frenzied Bulb's auto-attack did ~65 damage on Kawari, where retail
/// footage shows ~8. Retail's enemy damage formula isn't known, so this scales it down to match. It only applies to
/// players, so NPCs fighting each other (such as quest battle allies) are unaffected.
const NPC_DAMAGE_TO_PLAYERS_PERCENT: u32 = 12;

/// Process action-related messages.
pub fn handle_action_messages(
    data: Arc<Mutex<WorldServer>>,
    game_data: Arc<Mutex<GameData>>,
    network: Arc<Mutex<NetworkState>>,
    msg: &ToServer,
) -> bool {
    if let ToServer::ActionRequest(from_id, from_actor_id, request) = msg {
        let cast_time;
        {
            let mut game_data = game_data.lock();
            cast_time = game_data.get_casttime(request.action_id).unwrap(); // TODO: take into account the haste stat like the client does
        }

        let mut delay_milliseconds = cast_time as u64 * 100;

        let mut data = data.lock();
        let Some(instance) = data.find_actor_instance_mut(*from_actor_id) else {
            return true;
        };

        // Swiftcast makes the next spell instant, and Presence of Mind reduces cast times by 20%.
        if delay_milliseconds > 0
            && let Some(actor) = instance.find_actor(*from_actor_id)
            && let Some(effects) = actor.status_effects()
        {
            if effects.get(EFFECT_SWIFTCAST).is_some() {
                delay_milliseconds = 0;
            } else if effects.get(EFFECT_PRESENCE_OF_MIND).is_some() {
                delay_milliseconds = delay_milliseconds * 80 / 100;
            }
        }

        if delay_milliseconds > 0 {
            let Some(actor) = instance.find_actor(*from_actor_id) else {
                return true;
            };

            let actor_cast = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorCast {
                spell_id: request.action_id as u16,
                action_id: request.action_id,
                action_type: request.action_type,
                omen_delay: 0,
                cast_time: delay_milliseconds as f32 / 1000.0,
                target: request.target.object_id,
                rotation: request.rotation1,
                interruptible: false,
                ballista_entity_id: ObjectId::default(),
                position: actor.position(),
            });
            let mut network = network.lock();
            network.send_in_range_inclusive_instance(
                *from_actor_id,
                instance,
                FromServer::PacketSegment(actor_cast, *from_actor_id),
                DestinationNetwork::ZoneClients,
            );
        }

        instance.insert_task(
            *from_id,
            *from_actor_id,
            Duration::from_millis(delay_milliseconds),
            QueuedTaskData::CastAction {
                request: request.clone(),
                interruptible: delay_milliseconds > 0,
            },
        );

        return true;
    }

    false
}

/// Executes an action, and returns a list of Tasks that must be executed by the client.
pub fn execute_action(
    network: Arc<Mutex<NetworkState>>,
    data: Arc<Mutex<WorldServer>>,
    game_data: Arc<Mutex<GameData>>,
    lua: Arc<Mutex<KawariLua>>,
    from_id: ClientId,
    from_actor_id: ObjectId,
    request: ActionRequest,
) {
    let mut lua_player = LuaPlayer {
        player_data: PlayerData::default(),
        status_effects: StatusEffects::default(),
        queued_tasks: Vec::new(),
        zone_data: LuaZone::default(),
        base_parameters: BaseParameters::default(),
        current_level: 0,
    };
    // TODO: maybe move these misc effects to their own dedicated functions or something? I had some trouble remembering where this was
    // We need to set the player's mount id in their common spawn so both pillion works and also letting players see this existing actor's mount when they spawn.
    if request.action_type == ActionType::Mount {
        let mut data = data.lock();
        let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
            return;
        };

        let Some(actor) = instance.find_actor_mut(from_actor_id) else {
            return;
        };

        let common = actor.get_common_spawn_mut();

        common.current_mount = request.action_id as u16;
        common.mode = CharacterMode::Mounted;

        {
            let mut network = network.lock();
            let msg = FromServer::SetCurrentMount(common.current_mount);
            network.send_to_by_actor_id(from_actor_id, msg, DestinationNetwork::ZoneClients);
        }
    }

    // We need to set the ornament so new players see it
    if request.action_type == ActionType::Ornament {
        let mut data = data.lock();
        let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
            return;
        };

        let Some(actor) = instance.find_actor_mut(from_actor_id) else {
            return;
        };

        let common = actor.get_common_spawn_mut();

        common.ornament_id = request.action_id as u16;
    }

    // Targeting/cost info, only relevant to normal actions.
    let action_info;
    let has_cast_time;
    {
        let mut game_data = game_data.lock();
        if request.action_type == ActionType::Action {
            action_info = game_data.get_action_info(request.action_id);
            has_cast_time = game_data.get_casttime(request.action_id).unwrap_or(0) > 0;
        } else {
            action_info = None;
            has_cast_time = false;
        }
    }

    // Party members are needed to figure out who area-of-effect heals and buffs hit.
    let party_members: Vec<ObjectId> = if action_info.is_some_and(|info| info.is_area_of_effect()) {
        let network = network.lock();
        get_party_id_from_actor_id(&network, from_actor_id)
            .and_then(|party_id| network.parties.get(&party_id))
            .map(|party| {
                party
                    .members
                    .iter()
                    .filter(|member| member.is_valid())
                    .map(|member| member.actor_id)
                    .collect()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let in_combo;
    let combo_action_id;
    {
        let mut game_data = game_data.lock();
        combo_action_id = game_data.get_combo_action(request.action_id);

        let data = data.lock();
        let Some(instance) = data.find_actor_instance(from_actor_id) else {
            return;
        };

        let Some(actor) = instance.find_actor(from_actor_id) else {
            return;
        };

        if let NetworkedActor::Player {
            last_combo_action, ..
        } = actor
        {
            in_combo = combo_action_id == *last_combo_action;
        } else {
            in_combo = false;
        }
    }

    let mut effects_builder;
    let common_spawn;
    {
        let data = data.lock();

        let Some(instance) = data.find_actor_instance(from_actor_id) else {
            return;
        };

        let Some(actor) = instance.find_actor(from_actor_id) else {
            return;
        };

        lua_player.player_data.teleport_query = match actor {
            NetworkedActor::Player { teleport_query, .. } => teleport_query.clone(),
            _ => TeleportQuery::default(),
        };
        lua_player.base_parameters = match actor {
            NetworkedActor::Player { parameters, .. } => parameters.clone(),
            NetworkedActor::Npc { parameters, .. } => parameters.clone(),
            _ => BaseParameters::default(),
        };
        // Let scripts check the caster's buffs (e.g. Freecure, Sacred Sight.)
        if let Some(status_effects) = actor.status_effects() {
            lua_player.status_effects = status_effects.clone();
        }

        common_spawn = actor.get_common_spawn().clone();
        lua_player.current_level = common_spawn.level;

        effects_builder = match &request.action_type {
            ActionType::None => None,
            ActionType::Action => {
                execute_normal_action(lua.clone(), &request, &mut lua_player, in_combo)
            }
            ActionType::Item => {
                execute_item_action(game_data.clone(), lua.clone(), &request, &mut lua_player)
            }
            ActionType::Mount => {
                execute_mount_action(network.clone(), from_actor_id, &request, actor, instance)
            }
            ActionType::Ornament => {
                execute_ornament_action(network.clone(), from_actor_id, &request, actor, instance)
            }
            _ => unimplemented!(),
        };

        // Tone down NPC damage against players, see NPC_DAMAGE_TO_PLAYERS_PERCENT.
        let npc_hitting_player = matches!(actor, NetworkedActor::Npc { .. })
            && matches!(
                instance.find_actor(request.target.object_id),
                Some(NetworkedActor::Player { .. })
            );
        if npc_hitting_player && let Some(effects_builder) = &mut effects_builder {
            for effect in &mut effects_builder.effects {
                if let TargetEffectKind::Damage { amount, .. } = &mut effect.0 {
                    *amount =
                        ((*amount as u32 * NPC_DAMAGE_TO_PLAYERS_PERCENT / 100).max(1)) as u16;
                }
            }
        }
    }

    // tell them the action results
    if let Some(mut effects_builder) = effects_builder {
        // Only set for area-of-effect actions, which are applied differently.
        let mut aoe_result: Option<AreaOfEffectResult> = None;

        // Update our internal data model to their new HP
        {
            let mut data = data.lock();

            let Some(instance) = data.find_actor_instance_mut(request.target.object_id) else {
                return;
            };

            // aggro any NPCs, more so the more damage this does
            {
                let Some(actor) = instance.find_actor_mut(request.target.object_id) else {
                    return;
                };

                let damage: u32 = effects_builder
                    .effects
                    .iter()
                    .filter_map(|effect| match effect.0 {
                        TargetEffectKind::Damage { amount, .. } => Some(amount as u32),
                        _ => None,
                    })
                    .sum();
                actor.add_enmity(from_actor_id, damage);
            }

            // Handle invulnerability
            {
                let Some(actor) = instance.find_actor_mut(request.target.object_id) else {
                    return;
                };

                if let NetworkedActor::Npc {
                    currently_invulnerable,
                    ..
                } = actor
                    && *currently_invulnerable
                {
                    effects_builder.effects = effects_builder
                        .effects
                        .iter()
                        .map(|effect| match effect.0 {
                            TargetEffectKind::Damage { .. } => {
                                TargetEffect(TargetEffectKind::Invincible {})
                            }
                            _ => *effect,
                        })
                        .collect();
                }
            }

            // Handle combos
            {
                // TODO: don't send this for auto-attacks. it should be harmless in the mean time

                let Some(actor) = instance.find_actor_mut(from_actor_id) else {
                    return;
                };

                let sequence;
                if let NetworkedActor::Player {
                    last_combo_action,
                    combo_sequence,
                    ..
                } = actor
                {
                    *last_combo_action = request.action_id as u16;
                    sequence = *combo_sequence;

                    if combo_action_id == *last_combo_action {
                        *combo_sequence += 1;
                    }
                } else {
                    sequence = 0;
                }

                // Ensure we cancel any pending combo resets
                // We *intentionally* cancel all combos here because they're mutually exclusive with each other.
                // NOTE: retain() keeps what matches, so this must be negated. Otherwise every other queued task in
                // the instance (status effect expiries, death fade-outs, etc.) gets deleted whenever anyone uses an action.
                instance.retain_tasks(|task| {
                    !(task.from_actor_id == from_actor_id
                        && matches!(task.data, QueuedTaskData::ResetCombo))
                });

                // Add a new combo reset
                instance.insert_task(
                    from_id,
                    from_actor_id,
                    COMBO_TIMEOUT,
                    QueuedTaskData::ResetCombo,
                );

                effects_builder
                    .effects
                    .push(TargetEffect(TargetEffectKind::ExecuteCombo {
                        sequence,
                        unk2: 0,
                        unk3: 0,
                        unk4: 0,
                        unk5: 128,
                        action_id: request.action_id as u16,
                    }));
            }

            if let Some(info) = action_info.filter(|info| info.is_area_of_effect()) {
                // Fill in the damage element shown to the client.
                {
                    let mut game_data = game_data.lock();
                    for effect in &mut effects_builder.effects {
                        if let TargetEffectKind::Damage { damage_element, .. } = &mut effect.0 {
                            *damage_element =
                                game_data.get_action_damage_element(request.action_id);
                        }
                    }
                }

                aoe_result = Some(apply_area_of_effect(
                    network.clone(),
                    instance,
                    from_actor_id,
                    &request,
                    &info,
                    &party_members,
                    &effects_builder,
                ));
            } else {
                let heal_to_full = effects_builder.heal_to_full;
                for effect in &mut effects_builder.effects {
                    match &mut effect.0 {
                        TargetEffectKind::Heal { amount, .. } => {
                            let Some(actor) = instance.find_actor_mut(request.target.object_id)
                            else {
                                return;
                            };
                            let common_spawn = actor.get_common_spawn_mut();
                            // You can't heal the dead, that's what Raise is for.
                            if common_spawn.health_points > 0 {
                                if heal_to_full {
                                    // Show how much was actually restored.
                                    *amount = (common_spawn.max_health_points
                                        - common_spawn.health_points)
                                        .min(u16::MAX as u32)
                                        as u16;
                                    common_spawn.health_points = common_spawn.max_health_points;
                                } else {
                                    common_spawn.health_points = (common_spawn.health_points
                                        + *amount as u32)
                                        .min(common_spawn.max_health_points);
                                }
                            }
                        }
                        TargetEffectKind::Damage {
                            amount,
                            damage_element,
                            ..
                        } => {
                            let Some(actor) = instance.find_actor_mut(request.target.object_id)
                            else {
                                return;
                            };
                            let common_spawn = actor.get_common_spawn_mut();
                            if common_spawn.name_id != STRIKING_DUMMY_NAME_ID {
                                common_spawn.health_points =
                                    common_spawn.health_points.saturating_sub(*amount as u32);
                            }

                            // Update from game data
                            let mut game_data = game_data.lock();
                            *damage_element =
                                game_data.get_action_damage_element(request.action_id);
                        }
                        TargetEffectKind::InterruptAction => {
                            // TODO: this could cancel more than just casting, so we need to be more specific eventually
                            // TODO: also cancel the cast visually
                            instance.cancel_actor_tasks(request.target.object_id);
                        }
                        TargetEffectKind::SummonPet { .. } => {
                            let Some(actor) = instance.find_actor(from_actor_id) else {
                                return;
                            };

                            let pet_id = 23; // TODO: hardcoded

                            let mut network = network.lock();
                            network.send_to_by_actor_id(
                                from_actor_id,
                                FromServer::ActorControlSelf(
                                    ActorControlCategory::SetPetParameters {
                                        pet_id,
                                        unk2: 2,
                                        unk3: 5,
                                        unk4: 7,
                                    },
                                ),
                                DestinationNetwork::ZoneClients,
                            );

                            let pet_actor_id = ObjectId(fastrand::u32(..));

                            let config = get_config();
                            let mut game_data = game_data.lock();
                            let base_npc = create_npc_common_spawn(
                                &mut game_data,
                                13498,
                                10261,
                                None,
                                actor.get_common_spawn().level as u32,
                            ); // TODO: hardcoded pet base id/name
                            instance.insert_npc(
                                pet_actor_id,
                                SpawnNpc {
                                    common: CommonSpawn {
                                        pet_id,
                                        owner_id: from_actor_id,
                                        object_kind: ObjectKind::BattleNpc(BattleNpcSubKind::Pet),
                                        level: actor.get_common_spawn().level,
                                        position: actor.position(),
                                        rotation: actor.rotation(),
                                        ..base_npc.common
                                    },
                                    ..base_npc
                                },
                                &config,
                                &mut game_data,
                            );

                            network.send_to_by_actor_id(
                                from_actor_id,
                                FromServer::ActorControlSelf(ActorControlCategory::SetupPet {
                                    owner_id: from_actor_id,
                                    pet_id,
                                    pet_actor_id,
                                    unk2: 1,
                                    unk3: 1,
                                }),
                                DestinationNetwork::ZoneClients,
                            );
                        }
                        TargetEffectKind::SummonCompanion { .. } => {
                            let Some(actor) = instance.find_actor(from_actor_id) else {
                                return;
                            };

                            let mut network = network.lock();
                            network.send_to_by_actor_id(
                                from_actor_id,
                                FromServer::ActorControlSelf(
                                    ActorControlCategory::CompanionUpdate {
                                        summoned: true,
                                        set_time_remaining: true,
                                        time_remaining: 180000,
                                        unk4: 0,
                                        unk5: 18,
                                        unk6: 29,
                                    },
                                ),
                                DestinationNetwork::ZoneClients,
                            );

                            let companion_actor_id = ObjectId(fastrand::u32(..));

                            let config = get_config();
                            let mut game_data = game_data.lock();
                            let base_npc = create_npc_common_spawn(
                                &mut game_data,
                                952,
                                0,
                                None,
                                actor.get_common_spawn().level as u32,
                            ); // TODO: hardcoded pet base id/name
                            instance.insert_npc(
                                companion_actor_id,
                                SpawnNpc {
                                    common: CommonSpawn {
                                        name: "Test Companion".to_string(),
                                        owner_id: from_actor_id,
                                        object_kind: ObjectKind::BattleNpc(
                                            BattleNpcSubKind::Chocobo,
                                        ),
                                        level: actor.get_common_spawn().level,
                                        position: actor.position(),
                                        rotation: actor.rotation(),
                                        ..base_npc.common
                                    },
                                    ..base_npc
                                },
                                &config,
                                &mut game_data,
                            );

                            // TODO: add to the party list

                            network.send_to_by_actor_id(
                                from_actor_id,
                                FromServer::ActorControlSelf(
                                    ActorControlCategory::UnkCompanionRelated {
                                        unk1: companion_actor_id,
                                        unk2: 4,
                                    },
                                ),
                                DestinationNetwork::ZoneClients,
                            );
                        }
                        _ => {}
                    }
                }

                update_actor_hp_mp(network.clone(), instance, request.target.object_id);
            }
        }

        // TODO: send Cooldown ActorControlSelf

        {
            let mut network = network.lock();

            // ActionResult
            {
                // Area-of-effect actions show their results on a single representative target (if any.)
                // TODO: use the multi-target ActionEffect packets so every target gets flying text.
                let (packet_targets, packet_effects) = match &aoe_result {
                    Some(aoe) => (
                        aoe.display_target
                            .map(|object_id| ObjectTypeId {
                                object_id,
                                object_type: ObjectTypeKind::None,
                            })
                            .into_iter()
                            .collect(),
                        aoe.display_effects.clone(),
                    ),
                    None => (vec![request.target], effects_builder.effects.clone()),
                };

                // The packet only has room for 8 effects per target.
                let mut effects = [TargetEffect::default(); 8];
                let effect_count = packet_effects.len().min(effects.len());
                effects[..effect_count].copy_from_slice(&packet_effects[..effect_count]);

                let action_animation_id;
                {
                    let mut game_data = game_data.lock();

                    // TODO: Not sure if this is correct in every item situation
                    // If the action is an item being used, the animation id doesn't necessarily match the action id.
                    action_animation_id = if request.action_type == ActionType::Item
                        && let Some((action_type, _, _)) =
                            game_data.lookup_item_action_data(request.action_id)
                    {
                        action_type
                    } else {
                        // Otherwise, just assume the animation id is the action key for now.
                        request.action_id as u16
                    };
                }

                let ipc = ActionEffect {
                    animation_target_id: request.target,
                    targets: packet_targets,
                    action_id: request.action_id,
                    animation_lock: ANIMATION_LOCK_TIME,
                    rotation: common_spawn.rotation,
                    spell_id: action_animation_id,
                    source_sequence: request.sequence,
                    effects,
                    action_type: request.action_type,
                    global_sequence: network.global_action_sequence,
                    ..Default::default()
                }
                .package()
                .unwrap();
                network.global_action_sequence += 1;

                let mut data = data.lock();

                let Some(instance) = data.find_actor_instance_mut(request.target.object_id) else {
                    return;
                };
                network.send_in_range_inclusive_instance(
                    from_actor_id,
                    instance,
                    FromServer::PacketSegment(ipc, from_actor_id),
                    DestinationNetwork::ZoneClients,
                );
            }
        }

        // EffectResult
        // TODO: is this always sent? needs investigation
        {
            let mut num_self_entries = 0u8;
            let mut self_entries = [EffectEntry::default(); 4];

            let mut num_target_entries = 0u8;
            let mut target_entries = [EffectEntry::default(); 4];

            for effect in &effects_builder.effects {
                // Area-of-effect actions already handed out their status effects.
                if let TargetEffectKind::GainEffect {
                    effect_id,
                    duration,
                    param,
                    ..
                } = effect.0
                    && aoe_result.is_none()
                    && (num_target_entries as usize) < target_entries.len()
                {
                    let index = gain_effect(
                        network.clone(),
                        data.clone(),
                        ClientId::default(),
                        request.target.object_id,
                        effect_id,
                        param,
                        duration,
                        from_actor_id, // It's always given by this player.
                        false,         // EffectsResult will show it for us
                    );

                    target_entries[num_target_entries as usize] = EffectEntry {
                        index,
                        id: effect_id,
                        param,
                        duration,
                        source_actor_id: from_actor_id,
                        ..Default::default()
                    };
                    num_target_entries += 1;
                }

                if let TargetEffectKind::GainEffectSelf {
                    effect_id,
                    duration,
                    param,
                    ..
                } = effect.0
                    && (num_self_entries as usize) < self_entries.len()
                {
                    let index = gain_effect(
                        network.clone(),
                        data.clone(),
                        from_id,
                        from_actor_id,
                        effect_id,
                        param,
                        duration,
                        from_actor_id,
                        false, // EffectsResult will show it for us
                    );

                    self_entries[num_self_entries as usize] = EffectEntry {
                        index,
                        id: effect_id,
                        param,
                        duration,
                        source_actor_id: from_actor_id,
                        ..Default::default()
                    };
                    num_self_entries += 1;
                }

                // To lose effects, we just omit them from the list but increase the entry count!
                if let TargetEffectKind::LoseEffect { .. } = effect.0
                    && (num_self_entries as usize) < self_entries.len()
                {
                    self_entries[num_self_entries as usize] = EffectEntry::default();
                    num_self_entries += 1;

                    // TODO: need to re-review and restore this...
                    // self.status_effects.remove(effect_id);
                }
            }

            if num_self_entries > 0 {
                let mut network = network.lock();
                let ipc =
                    ServerZoneIpcSegment::new(ServerZoneIpcData::EffectResult(EffectResult {
                        count: 1,
                        global_sequence: network.global_action_sequence,
                        target_id: request.target.object_id,
                        health_points: common_spawn.health_points,
                        max_health_points: common_spawn.max_health_points,
                        resource_points: common_spawn.resource_points,
                        classjob_id: common_spawn.class_job,
                        entry_count: num_self_entries,
                        statuses: self_entries,
                        ..Default::default()
                    }));
                let mut data = data.lock();
                let Some(instance) = data.find_actor_instance_mut(request.target.object_id) else {
                    return;
                };
                network.send_in_range_inclusive_instance(
                    from_actor_id,
                    instance,
                    FromServer::PacketSegment(ipc, from_actor_id),
                    DestinationNetwork::ZoneClients,
                );
            }

            if num_target_entries > 0 {
                let mut data = data.lock();
                let Some(instance) = data.find_actor_instance_mut(request.target.object_id) else {
                    return;
                };

                let Some(actor) = instance.find_actor(request.target.object_id) else {
                    return;
                };

                let common_spawn = actor.get_common_spawn();

                let mut network = network.lock();
                let ipc =
                    ServerZoneIpcSegment::new(ServerZoneIpcData::EffectResult(EffectResult {
                        count: 1,
                        global_sequence: network.global_action_sequence,
                        target_id: from_actor_id,
                        health_points: common_spawn.health_points,
                        max_health_points: common_spawn.max_health_points,
                        resource_points: common_spawn.resource_points,
                        classjob_id: common_spawn.class_job,
                        entry_count: num_target_entries,
                        statuses: target_entries,
                        ..Default::default()
                    }));
                let Some(instance) = data.find_actor_instance_mut(request.target.object_id) else {
                    return;
                };
                network.send_in_range_inclusive_instance(
                    request.target.object_id,
                    instance,
                    FromServer::PacketSegment(ipc, request.target.object_id),
                    DestinationNetwork::ZoneClients,
                );
            }
        }

        // Over-time effects, MP, consumed buffs and job gauges.
        apply_caster_results(
            network.clone(),
            data.clone(),
            from_id,
            from_actor_id,
            &request,
            action_info,
            has_cast_time,
            aoe_result.map(|aoe| aoe.status_targets),
            &effects_builder,
        );
    }

    let mut network = network.lock();
    let msg = FromServer::NewTasks(lua_player.queued_tasks);
    network.send_to(from_id, msg, DestinationNetwork::ZoneClients);

    // Update cooldowns, currently only for the GM command
    {
        let data = data.lock();

        let Some(instance) = data.find_actor_instance(from_actor_id) else {
            return;
        };

        let Some(actor) = instance.find_actor(from_actor_id) else {
            return;
        };

        let NetworkedActor::Player {
            remove_cooldowns, ..
        } = actor
        else {
            return;
        };

        if *remove_cooldowns {
            let mut game_data = game_data.lock();
            let cooldown_group = game_data.get_action_cooldown_group(request.action_id) as u32 - 1;
            network.send_to_by_actor_id(
                from_actor_id,
                FromServer::ActorControlSelf(ActorControlCategory::SetCooldownTimer {
                    cooldown_group,
                    unk1: 0,
                    unk2: 0,
                }),
                DestinationNetwork::ZoneClients,
            );
        }
    }
}

/// The outcome of an area-of-effect action.
struct AreaOfEffectResult {
    /// The target shown in the ActionEffect packet, if anyone was hit.
    display_target: Option<ObjectId>,
    /// The effects shown on `display_target`.
    display_effects: Vec<TargetEffect>,
    /// Everyone who received the action's status effects (used for over-time effects.)
    status_targets: Vec<ObjectId>,
}

/// Returns true if this actor is an enemy that is still alive.
fn is_living_enemy(actor: &NetworkedActor) -> bool {
    matches!(actor, NetworkedActor::Npc { .. })
        && actor.get_common_spawn().object_kind == ObjectKind::BattleNpc(BattleNpcSubKind::Enemy)
        && actor.get_common_spawn().health_points > 0
}

/// Applies an area-of-effect action to everyone it hits.
///
/// Damage (and any status effects, if the action deals damage) goes to enemies, while healing (and status effects otherwise) go to the caster and their party.
fn apply_area_of_effect(
    network: Arc<Mutex<NetworkState>>,
    instance: &mut Instance,
    from_actor_id: ObjectId,
    request: &ActionRequest,
    info: &ActionInfo,
    party_members: &[ObjectId],
    effects_builder: &EffectsBuilder,
) -> AreaOfEffectResult {
    let Some(caster) = instance.find_actor(from_actor_id) else {
        return AreaOfEffectResult {
            display_target: None,
            display_effects: Vec::new(),
            status_targets: Vec::new(),
        };
    };
    let caster_position = caster.position();

    // Targeted area-of-effects (e.g. Cure III) are centered on the target, otherwise it's the caster (e.g. Medica.)
    let center = if info.range > 0 {
        instance
            .find_actor(request.target.object_id)
            .map(|actor| actor.position())
            .unwrap_or(caster_position)
    } else {
        caster_position
    };
    let radius = info.effect_range as f32 + AOE_HITBOX_LENIENCY;
    let distance_to = |actor: &NetworkedActor| actor.position().0.distance(center.0);

    let has_damage = effects_builder
        .effects
        .iter()
        .any(|effect| matches!(effect.0, TargetEffectKind::Damage { .. }));
    let has_heal = effects_builder
        .effects
        .iter()
        .any(|effect| matches!(effect.0, TargetEffectKind::Heal { .. }));
    let has_interrupt = effects_builder
        .effects
        .iter()
        .any(|effect| matches!(effect.0, TargetEffectKind::InterruptAction));

    // Enemies in range, closest first, but the primary target always comes first.
    let mut hostiles: Vec<(ObjectId, f32)> = instance
        .actors
        .iter()
        .filter(|(_, actor)| is_living_enemy(actor) && distance_to(actor) <= radius)
        .map(|(id, actor)| (*id, distance_to(actor)))
        .collect();
    hostiles.sort_by(|a, b| a.1.total_cmp(&b.1));
    let mut hostiles: Vec<ObjectId> = hostiles.into_iter().map(|(id, _)| id).collect();
    if let Some(index) = hostiles
        .iter()
        .position(|id| *id == request.target.object_id)
    {
        let id = hostiles.remove(index);
        hostiles.insert(0, id);
    }

    // The caster and any party members in range that are still alive. The primary target comes first.
    let mut friendlies: Vec<ObjectId> = Vec::new();
    for id in std::iter::once(from_actor_id).chain(party_members.iter().copied()) {
        if friendlies.contains(&id) {
            continue;
        }
        if let Some(actor) = instance.find_actor(id)
            && matches!(actor, NetworkedActor::Player { .. })
            && actor.get_common_spawn().health_points > 0
            && distance_to(actor) <= radius
        {
            friendlies.push(id);
        }
    }
    if let Some(index) = friendlies
        .iter()
        .position(|id| *id == request.target.object_id)
    {
        let id = friendlies.remove(index);
        friendlies.insert(0, id);
    }

    let mut touched: Vec<ObjectId> = Vec::new();

    // Damage enemies
    if has_damage {
        let falloff = effects_builder.aoe_falloff_percent as u32;
        for (index, target_id) in hostiles.iter().enumerate() {
            let Some(actor) = instance.find_actor_mut(*target_id) else {
                continue;
            };

            let mut invulnerable = false;
            if let NetworkedActor::Npc {
                currently_invulnerable,
                ..
            } = actor
            {
                invulnerable = *currently_invulnerable;
            }

            // The first target takes full damage, everyone else may take less.
            let total: u32 = effects_builder
                .effects
                .iter()
                .filter_map(|effect| match effect.0 {
                    TargetEffectKind::Damage { amount, .. } => Some(amount as u32),
                    _ => None,
                })
                .map(|amount| {
                    if index == 0 {
                        amount
                    } else {
                        amount * (100 - falloff) / 100
                    }
                })
                .sum();
            actor.add_enmity(from_actor_id, total);

            if !invulnerable {
                let common_spawn = actor.get_common_spawn_mut();
                if common_spawn.name_id != STRIKING_DUMMY_NAME_ID {
                    common_spawn.health_points = common_spawn.health_points.saturating_sub(total);
                }

                // The ActionEffect packet only shows the first target, so show numbers on the others.
                if index > 0 {
                    let mut network = network.lock();
                    network.send_ac_in_range_inclusive_instance(
                        instance,
                        *target_id,
                        ActorControlCategory::DamageFlyingText {
                            unk1: 0,
                            amount: total,
                            source_actor_id: from_actor_id,
                            unk4: 0,
                        },
                    );
                }
            }

            if has_interrupt {
                instance.cancel_actor_tasks(*target_id);
            }

            touched.push(*target_id);
        }
    }

    // Heal the caster and their party
    if has_heal {
        let total: u32 = effects_builder
            .effects
            .iter()
            .filter_map(|effect| match effect.0 {
                TargetEffectKind::Heal { amount, .. } => Some(amount as u32),
                _ => None,
            })
            .sum();

        for target_id in &friendlies {
            let Some(actor) = instance.find_actor_mut(*target_id) else {
                continue;
            };
            let common_spawn = actor.get_common_spawn_mut();
            common_spawn.health_points = if effects_builder.heal_to_full {
                common_spawn.max_health_points
            } else {
                (common_spawn.health_points + total).min(common_spawn.max_health_points)
            };

            if !touched.contains(target_id) {
                touched.push(*target_id);
            }
        }
    }

    // Status effects go to whoever the action is aimed at.
    let status_receivers = if has_damage { &hostiles } else { &friendlies };
    let mut status_targets = Vec::new();
    for target_id in status_receivers {
        for effect in &effects_builder.effects {
            if let TargetEffectKind::GainEffect {
                effect_id,
                duration,
                param,
                ..
            } = effect.0
            {
                gain_effect_instance(
                    network.clone(),
                    ClientId::default(),
                    instance,
                    *target_id,
                    effect_id,
                    param,
                    duration,
                    from_actor_id,
                    true,
                );
                if !status_targets.contains(target_id) {
                    status_targets.push(*target_id);
                }
            }
        }
    }

    for target_id in touched {
        update_actor_hp_mp(network.clone(), instance, target_id);
    }

    // Pick who the ActionEffect packet shows: an enemy for damaging actions, otherwise someone healed.
    let (display_target, display_is_hostile) = if has_damage && !hostiles.is_empty() {
        (hostiles.first().copied(), true)
    } else if (has_heal || !has_damage) && !friendlies.is_empty() {
        (friendlies.first().copied(), false)
    } else {
        (None, false)
    };

    let display_target_invulnerable = display_target
        .and_then(|id| instance.find_actor(id))
        .is_some_and(|actor| {
            matches!(
                actor,
                NetworkedActor::Npc {
                    currently_invulnerable: true,
                    ..
                }
            )
        });

    let display_effects = effects_builder
        .effects
        .iter()
        .filter(|effect| match effect.0 {
            TargetEffectKind::Damage { .. } | TargetEffectKind::InterruptAction => {
                display_is_hostile
            }
            TargetEffectKind::Heal { .. } => display_target.is_some() && !display_is_hostile,
            TargetEffectKind::GainEffect { .. } => {
                display_target.is_some() && display_is_hostile == has_damage
            }
            _ => true,
        })
        .map(|effect| match effect.0 {
            TargetEffectKind::Damage { .. } if display_target_invulnerable => {
                TargetEffect(TargetEffectKind::Invincible {})
            }
            _ => *effect,
        })
        .collect();

    AreaOfEffectResult {
        display_target,
        display_effects,
        status_targets,
    }
}

/// Handles everything that happens to the caster after an action resolves: over-time effects, MP, consumed buffs and job gauges.
#[allow(clippy::too_many_arguments)]
fn apply_caster_results(
    network: Arc<Mutex<NetworkState>>,
    data: Arc<Mutex<WorldServer>>,
    from_id: ClientId,
    from_actor_id: ObjectId,
    request: &ActionRequest,
    action_info: Option<ActionInfo>,
    has_cast_time: bool,
    aoe_status_targets: Option<Vec<ObjectId>>,
    effects_builder: &EffectsBuilder,
) {
    let mut data = data.lock();
    let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
        return;
    };

    // Start over-time effects on whoever received the status effects.
    let status_targets = aoe_status_targets.unwrap_or_else(|| vec![request.target.object_id]);
    for spec in &effects_builder.periodic {
        if spec.on_self {
            add_periodic_effect(instance, from_actor_id, from_actor_id, spec);
        } else {
            for target_id in &status_targets {
                add_periodic_effect(instance, *target_id, from_actor_id, spec);
            }
        }
    }

    // Figure out the MP cost, and which buffs were used up.
    let mut consumed_effects = effects_builder.consumed_effects.clone();
    let mut mp_cost = effects_builder
        .mp_cost_override
        .unwrap_or_else(|| action_info.map(|info| info.mp_cost()).unwrap_or(0));
    if let Some(actor) = instance.find_actor(from_actor_id)
        && let Some(status_effects) = actor.status_effects()
    {
        if mp_cost > 0 && status_effects.get(EFFECT_THIN_AIR).is_some() {
            mp_cost = 0;
            consumed_effects.push(EFFECT_THIN_AIR);
        }
        if has_cast_time && status_effects.get(EFFECT_SWIFTCAST).is_some() {
            consumed_effects.push(EFFECT_SWIFTCAST);
        }
    }

    if mp_cost > 0 || effects_builder.restore_mp > 0 {
        if let Some(actor) = instance.find_actor_mut(from_actor_id) {
            let common_spawn = actor.get_common_spawn_mut();
            let mp = (common_spawn.resource_points.saturating_sub(mp_cost) as u32)
                .saturating_add(effects_builder.restore_mp)
                .min(common_spawn.max_resource_points as u32);
            common_spawn.resource_points = mp as u16;
        }
        update_actor_hp_mp(network.clone(), instance, from_actor_id);
    }

    for effect_id in consumed_effects {
        instance.insert_task(
            from_id,
            from_actor_id,
            Duration::ZERO,
            QueuedTaskData::LoseStatusEffect {
                effect_id,
                effect_param: 0,
                effect_source_actor_id: from_actor_id,
            },
        );
    }

    // Remove a single stack from stacking buffs (e.g. Sacred Sight), and the whole buff on the last stack.
    for effect_id in &effects_builder.consumed_stacks {
        let effect_id = *effect_id;
        let remaining_duration = instance
            .queued_task
            .iter()
            .find(|task| {
                task.from_actor_id == from_actor_id
                    && matches!(task.data, QueuedTaskData::LoseStatusEffect { effect_id: id, .. } if id == effect_id)
            })
            .map(|task| {
                task.point
                    .saturating_duration_since(Instant::now())
                    .as_secs_f32()
            });

        let Some(actor) = instance.find_actor_mut(from_actor_id) else {
            continue;
        };
        let Some(status_effects) = actor.status_effects_mut() else {
            continue;
        };
        let Some(effect) = status_effects.get(effect_id) else {
            continue;
        };

        if effect.param > 1 {
            status_effects.set_param(effect_id, effect.param - 1);
            if let Some(remaining_duration) = remaining_duration {
                status_effects.set_duration(effect_id, remaining_duration);
            }
            process_effects_list(network.clone(), instance, from_actor_id);
        } else {
            instance.insert_task(
                from_id,
                from_actor_id,
                Duration::ZERO,
                QueuedTaskData::LoseStatusEffect {
                    effect_id,
                    effect_param: effect.param,
                    effect_source_actor_id: from_actor_id,
                },
            );
        }
    }

    // Update the job gauge
    if !effects_builder.gauge_ops.is_empty()
        && let Some(NetworkedActor::Player {
            spawn, job_gauge, ..
        }) = instance.find_actor_mut(from_actor_id)
    {
        let level = spawn.common.level;
        for op in &effects_builder.gauge_ops {
            job_gauge.apply(*op, level);
        }

        let ipc = job_gauge.to_ipc(spawn.common.class_job);
        let mut network = network.lock();
        network.send_to_by_actor_id(
            from_actor_id,
            FromServer::PacketSegment(ipc, from_actor_id),
            DestinationNetwork::ZoneClients,
        );
    }
}

pub fn cancel_action(network: Arc<Mutex<NetworkState>>, from_id: ClientId, action_id: u32) {
    let msg = FromServer::ActorControlSelf(ActorControlCategory::CancelCast {
        unk1: 538,
        unk2: 1,
        action_id,
    });

    let mut network = network.lock();
    network.send_to(from_id, msg, DestinationNetwork::ZoneClients);
}

/// Handles normal actions, powered by Lua.
pub fn execute_normal_action(
    lua: Arc<Mutex<KawariLua>>,
    request: &ActionRequest,
    lua_player: &mut LuaPlayer,
    in_combo: bool,
) -> Option<EffectsBuilder> {
    let mut effects_builder = None;
    let lua = lua.lock();
    let state = lua.0.app_data_ref::<KawariLuaState>().unwrap();

    let key = request.action_id;
    if let Some(action_script) = state.action_scripts.get(&key) {
        lua.0
            .scope(|scope| {
                let connection_data = scope.create_userdata_ref_mut(lua_player).unwrap();

                lua.0
                    .load(
                        std::fs::read(action_script).expect("Failed to locate scripts directory!"),
                    )
                    .set_name("@".to_string() + action_script)
                    .exec()
                    .unwrap();

                let func: Function = lua.0.globals().get("doAction").unwrap();

                effects_builder = Some(
                    func.call::<EffectsBuilder>((connection_data, in_combo))
                        .unwrap(),
                );

                Ok(())
            })
            .unwrap();
    } else {
        tracing::warn!("Action {key} isn't scripted yet!");
    }

    effects_builder
}

/// Handles item actions, powered by Lua.
pub fn execute_item_action(
    game_data: Arc<Mutex<GameData>>,
    lua: Arc<Mutex<KawariLua>>,
    request: &ActionRequest,
    lua_player: &mut LuaPlayer,
) -> Option<EffectsBuilder> {
    let lua = lua.lock();

    let key = request.action_id;
    let (action_type, action_data, additional_data);
    let is_misc;
    {
        let mut game_data = game_data.lock();
        (action_type, action_data, additional_data) =
            game_data.lookup_item_action_data(key).unwrap_or_default();
        is_misc = game_data.item_is_misc(key);
    }

    let mut effects_builder = None;
    lua.0
        .scope(|scope| {
            let connection_data = scope.create_userdata_ref_mut(lua_player).unwrap();

            let func: Function = lua.0.globals().get("dispatchItem").unwrap();

            match func.call::<(String, u32)>((
                &connection_data,
                key,
                action_type,
                action_data,
                additional_data,
                is_misc,
            )) {
                Ok((action_script, arg)) => {
                    lua.0
                        .load(
                            std::fs::read(
                                get_config().filesystem.locate_script_file(&action_script),
                            )
                            .expect("Failed to locate scripts directory!"),
                        )
                        .set_name("@".to_string() + &action_script)
                        .exec()
                        .unwrap();

                    let func: Function = lua.0.globals().get("doAction").unwrap();

                    effects_builder =
                        Some(func.call::<EffectsBuilder>((connection_data, arg)).unwrap());
                }
                Err(err) => {
                    tracing::error!("Error while calling dispatchItem: {:?}", err);
                }
            }

            Ok(())
        })
        .unwrap();

    effects_builder
}

/// Handles mount-related actions.
pub fn execute_mount_action(
    network: Arc<Mutex<NetworkState>>,
    from_actor_id: ObjectId,
    request: &ActionRequest,
    actor: &NetworkedActor,
    instance: &Instance,
) -> Option<EffectsBuilder> {
    let mut network = network.lock();

    let common_spawn = actor.get_common_spawn();

    let mut effects = [TargetEffect::default(); 8];
    effects[0] = TargetEffect(TargetEffectKind::Mount {
        unk1: 1,
        unk2: 0,
        id: request.action_id as u16,
    });

    let ipc = ActionEffect {
        animation_target_id: request.target,
        targets: vec![request.target],
        action_id: request.action_id,
        animation_lock: ANIMATION_LOCK_TIME,
        rotation: common_spawn.rotation,
        spell_id: request.action_id as u16,
        source_sequence: request.sequence,
        effects,
        action_type: request.action_type,
        global_sequence: network.global_action_sequence,
        ..Default::default()
    }
    .package()
    .unwrap();
    network.global_action_sequence += 1;

    network.send_in_range_inclusive_instance(
        from_actor_id,
        instance,
        FromServer::PacketSegment(ipc, from_actor_id),
        DestinationNetwork::ZoneClients,
    );

    let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::Mount {
        id: request.action_id as u16,
        unk1: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    });
    network.send_in_range_inclusive_instance(
        from_actor_id,
        instance,
        FromServer::PacketSegment(ipc, from_actor_id),
        DestinationNetwork::ZoneClients,
    );

    None
}

/// Handles ornament-related actions.
pub fn execute_ornament_action(
    network: Arc<Mutex<NetworkState>>,
    from_actor_id: ObjectId,
    request: &ActionRequest,
    actor: &NetworkedActor,
    instance: &Instance,
) -> Option<EffectsBuilder> {
    let mut network = network.lock();

    let common_spawn = actor.get_common_spawn();

    let effects = [TargetEffect::default(); 8]; // TODO: is there an ornament effect?

    let ipc = ActionEffect {
        animation_target_id: request.target,
        targets: vec![request.target],
        action_id: request.action_id,
        animation_lock: ANIMATION_LOCK_TIME,
        rotation: common_spawn.rotation,
        spell_id: request.action_id as u16,
        source_sequence: request.sequence,
        effects,
        action_type: request.action_type,
        global_sequence: network.global_action_sequence,
        ..Default::default()
    }
    .package()
    .unwrap();
    network.global_action_sequence += 1;

    network.send_in_range_inclusive_instance(
        from_actor_id,
        instance,
        FromServer::PacketSegment(ipc, from_actor_id),
        DestinationNetwork::ZoneClients,
    );

    // TODO: set in commonspawn for new spawns?

    network.send_ac_in_range_inclusive_instance(
        instance,
        from_actor_id,
        ActorControlCategory::SetOrnament {
            id: request.action_id,
        },
    );

    None
}
