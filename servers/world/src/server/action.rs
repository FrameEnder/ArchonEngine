//! Executing actions and other related functions.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use glam::{Vec2, Vec3A};
use mlua::Function;
use parking_lot::Mutex;

use crate::{
    ActionInfo, ClientId, FromServer, GameData, PlayerData, StatusEffects, ToServer,
    gamedata::AreaShape,
    lua::{
        EFFECT_ON_SOURCE, EffectsBuilder, KawariLua, KawariLuaState, LuaPlayer, LuaZone,
        PeriodicKind, PeriodicSpec,
    },
    server::{
        WorldServer,
        actor::{
            NetworkedActor, bnpc_base_hp, create_npc_common_spawn, update_actor_hp_mp,
            update_actor_hp_mp_after_hit,
        },
        companion,
        effect::{gain_effect, gain_effect_instance, process_effects_list},
        first_attack,
        instance::{Instance, QueuedTaskData},
        modifiers,
        network::{DestinationNetwork, NetworkState},
        party::get_party_id_from_actor_id,
        periodic::add_periodic_effect,
    },
    zone_connection::{BaseParameters, TeleportQuery},
};
use kawari::{
    common::{
        ANIMATION_LOCK_TIME, ActionCategory, COMBO_TIMEOUT, CharacterMode, ObjectId, ObjectTypeId,
        ObjectTypeKind, Position, STRIKING_DUMMY_NAME_ID,
    },
    config::get_config,
    ipc::zone::{
        ActionEffect, ActionRequest, ActionType, ActorControlCategory, ActorControlSelf, AoeEffect,
        BattleNpcSubKind, CharacterDataFlag, CommonSpawn, DamageKind, DamageType, EffectEntry,
        EffectResult, ObjectKind, ServerZoneIpcData, ServerZoneIpcSegment, SpawnNpc, TargetEffect,
        TargetEffectKind,
    },
};

/// Status effect: Swiftcast. The next spell is cast immediately.
const EFFECT_SWIFTCAST: u16 = 167;

/// Status effect: Presence of Mind. Reduces cast times by 20%.
const EFFECT_PRESENCE_OF_MIND: u16 = 157;

/// Status effect: Thin Air. The next action costs no MP.
const EFFECT_THIN_AIR: u16 = 1217;

/// How many targets an area action's results can show (AoeEffect8); those beyond get flying text of their own.
const MAX_AOE_PACKET_TARGETS: usize = 8;

/// Extra leniency (in yalms) given to area-of-effect checks, to roughly account for target hitboxes.
const AOE_HITBOX_LENIENCY: f32 = 1.0;

/// How far past the edge of an NPC's area attack a player still counts as caught: the player's own hitbox, half a yalm.
/// It was `AOE_HITBOX_LENIENCY` (a whole yalm), and with the server seeing positions a fraction of a second late, players
/// who had left a 6-yalm Terror Eye were hit at 6.6 and 6.7 yalms (2026-10-17 bundle).
pub const PLAYER_HITBOX_RADIUS: f32 = 0.5;

/// How much of their damage NPCs deal to players, in percent, for NPCs whose retail damage isn't recorded in their
/// timeline (see `Timeline::damage_to_players`.)
///
/// NPC damage is worked out like a player's (strength or intelligence times potency), which comes out far too high
/// against players: in Chasing Shadows a level 5 Frenzied Bulb's auto-attack did ~65 damage on Kawari, where retail
/// footage shows ~8. Retail's enemy damage formula isn't known, so this scales it down to match. It only applies to
/// players, so NPCs fighting each other (such as quest battle allies) are unaffected.
const NPC_DAMAGE_TO_PLAYERS_PERCENT: u32 = 12;

/// How often an NPC's hit on a player is critical, in percent, and how much harder it hits. From Kawari Bestiary's
/// recording of the Central Shroud on retail: 3 of 60 recorded hits were critical, each about 1.5 times the usual.
const NPC_CRITICAL_PERCENT: u8 = 5;
const NPC_CRITICAL_MULTIPLIER: f32 = 1.5;

/// How much NPC damage to players varies from hit to hit (plus or minus), as a fraction. Retail auto-attacks from one
/// monster landed within about 5% of each other.
const NPC_DAMAGE_VARIANCE: f32 = 0.05;

/// How often an NPC's attack on a player is parried, in percent, and how much of a parried hit gets through: 5-6 % of
/// hits on Bob, a parried one 0.83 times a normal one (Bob's Chronofoil captures, 2,438 hits.)
const NPC_PARRY_PERCENT: u8 = 5;
const NPC_PARRY_MULTIPLIER: f32 = 0.83;

/// How often an NPC's attack on a player misses, in percent, by how many levels the player is above it (Bob's
/// Chronofoil captures: 0-3 % at the same level or below the enemy, 12-14 % 1-2 above, 21-30 % 3-10 above, 32-53 %
/// further.)
fn npc_miss_percent(player_level: u8, enemy_level: u8) -> u8 {
    match player_level.saturating_sub(enemy_level) {
        0 => 2,
        1..=2 => 12,
        3..=10 => 25,
        _ => 40,
    }
}

/// What to multiply an NPC's damage against players by: its recorded retail auto-attack damage over what its stats
/// give for one, or `NPC_DAMAGE_TO_PLAYERS_PERCENT` if it has no recording.
fn npc_damage_scale(actor: &NetworkedActor, level: u8) -> f32 {
    if let NetworkedActor::Npc {
        timeline,
        parameters,
        ..
    } = actor
        && parameters.physical_damage > 0
        && let Some(damage) = timeline.auto_attack_damage(level, bnpc_base_hp)
    {
        return damage / parameters.physical_damage as f32;
    }
    NPC_DAMAGE_TO_PLAYERS_PERCENT as f32 / 100.0
}

/// Process action-related messages.
pub fn handle_action_messages(
    data: Arc<Mutex<WorldServer>>,
    game_data: Arc<Mutex<GameData>>,
    network: Arc<Mutex<NetworkState>>,
    msg: &ToServer,
) -> bool {
    if let ToServer::ActionRequest(from_id, from_actor_id, request) = msg {
        if request.action_type == ActionType::EventItem {
            start_event_item(data, game_data, network, *from_id, *from_actor_id, request);
            return true;
        }

        let (category, cast_time);
        let recast;
        let can_target_dead;
        let cast_action;
        {
            let mut game_data = game_data.lock();
            can_target_dead = game_data.action_can_target_dead(request.action_id);
            // An item is used through its ItemAction's own action (its type: an Action row), whose cast it has and
            // whose ID the cast bar shows: retail's Vesper Bay Aetheryte Ticket (item 30362) cast action 9994 for 5 s
            // (Bob's Chronofoil capture b8432613); the item's ID read as an Action row gave some unrelated action's.
            cast_action = if request.action_type == ActionType::Item {
                game_data
                    .lookup_item_action_data(request.action_id)
                    .map(|(action, _, _)| action as u32)
                    .filter(|action| *action != 0)
                    .unwrap_or(request.action_id)
            } else {
                request.action_id
            };
            // Not an action the Action sheet has (an ActionRequest the server doesn't handle yet): ignored, rather
            // than taking the server down.
            let Some(cast) = game_data.get_casttime(cast_action) else {
                tracing::warn!(
                    "No Action row for {:?} {}, ignoring the request",
                    request.action_type,
                    request.action_id
                );
                return true;
            };
            (category, cast_time) = cast;
            recast = game_data.get_action_recast(request.action_id);
        }

        let mut data = data.lock();
        let Some(instance) = data.find_actor_instance_mut(*from_actor_id) else {
            return true;
        };

        let Some(actor) = instance.find_actor(*from_actor_id) else {
            return true;
        };

        if request.action_type == ActionType::Action
            && let NetworkedActor::Player { .. } = actor
            && let Some(log_message) =
                rejection(instance, *from_actor_id, actor, request, can_target_dead)
        {
            let ipc =
                ServerZoneIpcSegment::new(ServerZoneIpcData::ActorControlSelf(ActorControlSelf {
                    category: ActorControlCategory::ActionRejected {
                        log_message,
                        action_type: 1,
                        action_id: request.action_id,
                        elapsed: 0,
                    },
                }));
            network.lock().send_to_by_actor_id(
                *from_actor_id,
                FromServer::PacketSegment(ipc, *from_actor_id),
                DestinationNetwork::ZoneClients,
            );
            return true;
        }

        let parameters = match actor {
            NetworkedActor::Player { parameters, .. } => parameters,
            NetworkedActor::Npc { parameters, .. } => parameters,
            _ => unreachable!(),
        };
        let mut adjusted_cast_time = parameters.get_adjusted_cast_time(category, cast_time);

        // Swiftcast makes the next spell instant, and Presence of Mind reduces cast times by 20%.
        if adjusted_cast_time > 0
            && let Some(effects) = actor.status_effects()
        {
            if effects.get(EFFECT_SWIFTCAST).is_some() {
                adjusted_cast_time = 0;
            } else if effects.get(EFFECT_PRESENCE_OF_MIND).is_some() {
                adjusted_cast_time = adjusted_cast_time * 80 / 100;
            }
        }

        if adjusted_cast_time > 0 {
            let actor_cast = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorCast {
                spell_id: cast_action as u16,
                action_id: request.action_id,
                action_type: request.action_type,
                omen_delay: 0,
                cast_time: adjusted_cast_time as f32 / 1000.0,
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

        // Retail tells the player each action's recast as they use it (ActorControlSelf 17, with the cast bar or the
        // action's effect; Bob's Chronofoil captures, exact for 5,394 of 6,043 GCDs): the GCD's 2.5 s (or whatever)
        // sped up by Spell or Skill Speed and haste, like a cast time; others their Recast100ms, times their charges.
        if request.action_type == ActionType::Action
            && let NetworkedActor::Player { .. } = actor
            && let Some((cooldown_group, recast_100ms, max_charges)) = recast
            && cooldown_group > 0
            && recast_100ms > 0
        {
            // As the client works it out (`client_rules`, ActionManager.GetAdjustedRecastTime and GetMaxCharges): the
            // row's recast plus its learnt traits' (Enhanced Swiftcast's -20 s...), sped up by speed and haste for a
            // weaponskill or spell with one charge at the player's level (never under 1.5 s); otherwise times its
            // charges at the highest level (RecastDetail.Total is the whole gauge, which the client's
            // GetRecastTimeForGroup cuts down while the extra charges aren't learnt): retail sent 60 s for Divine
            // Benison and 120 s for Tetragrammaton to a white mage synced to 80, below their level 88 and 98 charge
            // traits (Chronofoil capture 0.1). Every trait in the client's tables has no quest.
            let common = actor.get_common_spawn();
            let (class_job, level) = (common.class_job, common.level);
            let (charges, gauge_charges, trait_ms) = {
                let mut game_data = game_data.lock();
                let no_quest = |_: u32| true;
                (
                    crate::client_rules::max_charges(
                        &mut game_data,
                        request.action_id,
                        max_charges,
                        class_job,
                        level,
                        &no_quest,
                    ),
                    crate::client_rules::max_charges(
                        &mut game_data,
                        request.action_id,
                        max_charges,
                        class_job,
                        u8::MAX,
                        &no_quest,
                    ),
                    crate::client_rules::recast_trait_ms(
                        &mut game_data,
                        request.action_id,
                        class_job,
                        level,
                        &no_quest,
                    ),
                )
            };
            let recast = (recast_100ms as i32 * 100 + trait_ms).max(0) as u32;
            let recast_ms = if crate::client_rules::recast_uses_speed(category, charges) {
                parameters.get_adjusted_recast_time(category, recast).max(0) as u32
            } else {
                recast * gauge_charges.max(1) as u32
            };
            let ipc =
                ServerZoneIpcSegment::new(ServerZoneIpcData::ActorControlSelf(ActorControlSelf {
                    category: ActorControlCategory::SetCooldownTimerMax {
                        cooldown_group: cooldown_group as u32 - 1,
                        action_id: request.action_id,
                        milliseconds: recast_ms / 10,
                    },
                }));
            let mut network = network.lock();
            network.send_to_by_actor_id(
                *from_actor_id,
                FromServer::PacketSegment(ipc, *from_actor_id),
                DestinationNetwork::ZoneClients,
            );
        }

        // A cast lands as its bar ends. Retail sends a player's ActionEffect half a second before that (Bob's Chronofoil
        // captures: 0.50-0.54 s early for 4,165 of 4,805 casts of 1.1 to 7.7 s), but the target's side (its health,
        // aggro, death) only comes with the result, 1.07 s after that effect; Kawari applies an action's effects when
        // it sends them, so landing early took the target's health off before the bar had ended (Bob's 2026-10-21
        // bundle). Moving in the last half second doesn't interrupt it (slidecasting, `server/mod.rs`.)
        let lands_after = adjusted_cast_time;

        instance.insert_task(
            *from_id,
            *from_actor_id,
            Duration::from_millis(lands_after as u64),
            QueuedTaskData::CastAction {
                request: request.clone(),
                interruptible: adjusted_cast_time > 0,
            },
        );

        return true;
    }

    false
}

/// A key item used on something (ActionRequest of type EventItem), as retail does it (Bob's Chronofoil capture of
/// 2026-10-18, Out of House and Home): its cast bar (ActorCast with the EventItem's Action as the spell, the item as
/// the action and its CastTime), then, when the bar ends (`use_event_item`), the use itself.
fn start_event_item(
    data: Arc<Mutex<WorldServer>>,
    game_data: Arc<Mutex<GameData>>,
    network: Arc<Mutex<NetworkState>>,
    from_id: ClientId,
    from_actor_id: ObjectId,
    request: &ActionRequest,
) {
    let Some(info) = game_data.lock().get_event_item(request.action_id) else {
        tracing::warn!(
            "{} isn't an event item, ignoring its use",
            request.action_id
        );
        return;
    };

    let mut data = data.lock();
    let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
        return;
    };
    let Some(actor) = instance.find_actor(from_actor_id) else {
        return;
    };
    if info.cast_time_ms > 0 {
        let actor_cast = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorCast {
            spell_id: info.action,
            action_id: request.action_id,
            action_type: request.action_type,
            omen_delay: 0,
            cast_time: info.cast_time_ms as f32 / 1000.0,
            target: request.target.object_id,
            rotation: request.rotation1,
            interruptible: false,
            ballista_entity_id: ObjectId::default(),
            position: actor.position(),
        });
        let mut network = network.lock();
        network.send_in_range_inclusive_instance(
            from_actor_id,
            instance,
            FromServer::PacketSegment(actor_cast, from_actor_id),
            DestinationNetwork::ZoneClients,
        );
    }
    // Moving cancels it like any cast (`CastAction`, which `execute_queued_task` sends to `use_event_item`.)
    instance.insert_task(
        from_id,
        from_actor_id,
        Duration::from_millis(info.cast_time_ms as u64),
        QueuedTaskData::CastAction {
            request: request.clone(),
            interruptible: info.cast_time_ms > 0,
        },
    );
}

/// The key item's cast bar ended: retail's ActionEffect (the EventItem's Action as the animation, no effects, a 0.1 s
/// lock), then the quest's event, started by the zone connection (`FromServer::EventItemUsed`.)
pub fn use_event_item(
    network: Arc<Mutex<NetworkState>>,
    data: Arc<Mutex<WorldServer>>,
    game_data: Arc<Mutex<GameData>>,
    from_id: ClientId,
    from_actor_id: ObjectId,
    request: &ActionRequest,
) {
    let spell_id = game_data
        .lock()
        .get_event_item(request.action_id)
        .map(|info| info.action)
        .unwrap_or_default();
    let data = data.lock();
    let Some(instance) = data.find_actor_instance(from_actor_id) else {
        return;
    };
    let Some(actor) = instance.find_actor(from_actor_id) else {
        return;
    };
    let mut network = network.lock();
    let ipc = ActionEffect {
        animation_target_id: request.target,
        targets: vec![request.target],
        action_id: request.action_id,
        animation_lock: EVENT_ITEM_ANIMATION_LOCK,
        rotation: actor.rotation(),
        spell_id,
        source_sequence: request.sequence,
        action_type: ActionType::EventItem,
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
    network.send_to(
        from_id,
        FromServer::EventItemUsed(request.action_id, request.target),
        DestinationNetwork::ZoneClients,
    );
}

/// What retail's ActionEffect for a key item's use locks (0.1 s, Bob's 2026-10-18 capture.)
const EVENT_ITEM_ANIMATION_LOCK: f32 = 0.1;

/// Whether an action is aimed at something the server doesn't have as an actor: an event NPC or object the client
/// places itself (the wounded Wood Wailer Trial by Water has you heal.)
pub fn targets_event_object(
    instance: &Instance,
    from_actor_id: ObjectId,
    request: &ActionRequest,
) -> bool {
    let target = request.target.object_id;
    request.action_type == ActionType::Action
        && target.is_valid()
        && target != from_actor_id
        && instance.find_actor(target).is_none()
}

/// An action's cast on an event NPC or object ended. As retail (Bob's Chronofoil capture b23b91ef, Cure on Trial by
/// Water's wounded Wood Wailer): the ActionEffect on it (0.1 s lock; retail's carried a heal Kawari doesn't show),
/// then, about a second later, the quest's event of type ActionResult with the action as its argument, which the zone
/// connection starts if a quest listens for it (`FromServer::ActionOnEventObject`.)
pub fn use_action_on_event_object(
    network: Arc<Mutex<NetworkState>>,
    data: Arc<Mutex<WorldServer>>,
    from_id: ClientId,
    from_actor_id: ObjectId,
    request: &ActionRequest,
) {
    let data = data.lock();
    let Some(instance) = data.find_actor_instance(from_actor_id) else {
        return;
    };
    let Some(actor) = instance.find_actor(from_actor_id) else {
        return;
    };
    let mut network = network.lock();
    let ipc = ActionEffect {
        animation_target_id: request.target,
        targets: vec![request.target],
        action_id: request.action_id,
        animation_lock: EVENT_ITEM_ANIMATION_LOCK,
        rotation: actor.rotation(),
        spell_id: request.action_id as u16,
        source_sequence: request.sequence,
        action_type: ActionType::Action,
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
    network.send_to(
        from_id,
        FromServer::ActionOnEventObject(request.action_id, request.target),
        DestinationNetwork::ZoneClients,
    );
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
        scene_error_code: 0,
        ..Default::default()
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

        // Riding one's own chocobo: others see its barding, and the companion, if it's out, goes in.
        if request.action_id == companion::COMPANION_MOUNT as u32 {
            if let Some((barding, color)) = companion::barding(&data, from_actor_id)
                && let Some(instance) = data.find_actor_instance_mut(from_actor_id)
                && let Some(actor) = instance.find_actor_mut(from_actor_id)
            {
                let common = actor.get_common_spawn_mut();
                common.mount_head = barding[0];
                common.mount_body = barding[1];
                common.mount_feet = barding[2];
                common.mount_color = color;
            }
            companion::mounted(&mut data, &mut network.lock(), from_actor_id);
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
    let action_category;
    {
        let mut game_data = game_data.lock();
        if request.action_type == ActionType::Action {
            action_info = game_data.get_action_info(request.action_id);
            let cast = game_data.get_casttime(request.action_id);
            has_cast_time = cast.is_some_and(|(_, cast_time)| cast_time > 0);
            action_category = cast.map(|(category, _)| category);
        } else {
            action_info = None;
            has_cast_time = false;
            action_category = None;
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
    let takes_part_in_combos;
    {
        let mut game_data = game_data.lock();
        combo_action_id = game_data.get_combo_action(request.action_id);
        takes_part_in_combos = request.action_type == ActionType::Action
            && game_data.takes_part_in_combos(request.action_id);

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
    // An NPC's own animation lock and swing (see `Timeline::animation_locks`.)
    let npc_animation_lock: Option<f32>;
    let npc_auto_attack: bool;
    // Which way an NPC faced as it began casting this: its hit goes out that way (it turned back to its target in the
    // tick the cast ended, and Kottos' Catapult swung the tank's way, Bob's bundle of 2026-10-26.)
    let npc_cast_rotation: Option<f32>;
    // Other players caught in an enemy's area attack, with what it does to each of them.
    let mut area_bystanders: Vec<(ObjectId, Vec<TargetEffect>)> = Vec::new();
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
        npc_cast_rotation = match actor {
            NetworkedActor::Npc {
                cast: Some(cast), ..
            } if cast.action_id == request.action_id => Some(cast.rotation),
            _ => None,
        };
        // Retail's NPC auto-attacks lock for 0.1 s and swing one of three ways, like a player's (Bob's Copperbell Mines
        // captures of 2026-10-25: 870 and 872, 0.1 every time, 0/1/2 a third each over 547); their other actions carry
        // a lock of their own, the same every use (the timeline's `animation_locks`). Kawari sent 0.6 and swing 0.
        (npc_animation_lock, npc_auto_attack) = match actor {
            NetworkedActor::Npc { timeline, .. } => {
                let auto = timeline.autoattack_action_id == request.action_id
                    && request.action_type == ActionType::Action;
                let lock = if auto {
                    Some(ANIMATION_LOCK_AFTER_CAST)
                } else {
                    timeline.animation_locks.get(&request.action_id).copied()
                };
                (lock, auto)
            }
            _ => (None, false),
        };
        lua_player.current_level = common_spawn.level;
        lua_player.is_player = matches!(actor, NetworkedActor::Player { .. });
        lua_player.hp = common_spawn.health_points;
        lua_player.max_hp = common_spawn.max_health_points;
        lua_player.mp = common_spawn.resource_points;
        lua_player.max_mp = common_spawn.max_resource_points;

        effects_builder = match &request.action_type {
            ActionType::None => None,
            ActionType::Action => {
                execute_normal_action(lua.clone(), &request, &mut lua_player, in_combo)
            }
            ActionType::Item => {
                execute_item_action(game_data.clone(), lua.clone(), &request, &mut lua_player)
            }
            ActionType::Mount => {
                // One's own chocobo shows its barding and colour (the companion's.)
                let barding = (request.action_id == companion::COMPANION_MOUNT as u32)
                    .then(|| companion::mount_bytes(&data, from_actor_id))
                    .flatten();
                execute_mount_action(
                    network.clone(),
                    from_actor_id,
                    &request,
                    actor,
                    instance,
                    barding,
                )
            }
            ActionType::Ornament => {
                execute_ornament_action(network.clone(), from_actor_id, &request, actor, instance)
            }
            _ => unimplemented!(),
        };

        // Bring NPC damage against players to what retail does, see npc_damage_scale. Hits vary a little, and are
        // sometimes critical.
        // Party bots and party NPCs are hit as players are (`hit_like_a_player`): an enemy's hit on them was worked out
        // unscaled, like one NPC on another, about eight times what it does to a player, so Sastasha's Duty Support
        // members died to two enemies (Bob, 2026-10-23).
        let enemy_attacking = matches!(actor, NetworkedActor::Npc { .. })
            && !instance.party_bots.contains_key(&from_actor_id);
        // Chocobo companions too: they have about half their owner's HP (server/companion.rs.)
        let hit_like_a_player = |id: ObjectId| {
            matches!(instance.find_actor(id), Some(NetworkedActor::Player { .. }))
                || (enemy_attacking
                    && (instance.party_bots.contains_key(&id)
                        || companion::is_companion(instance, id)))
        };
        let npc_hitting_player = matches!(actor, NetworkedActor::Npc { .. })
            && hit_like_a_player(request.target.object_id);
        // What an enemy's action did to a party NPC on retail, when its timeline has it: its `npcs` amount, recorded on
        // the Duty Support members (Sastasha's bosses), taken as it is. Otherwise a party NPC takes its share of a
        // player's (`PartyBot::damage_taken`.)
        let is_party_npc = |id: ObjectId| {
            enemy_attacking
                && instance
                    .party_bots
                    .get(&id)
                    .is_some_and(|bot| bot.party_npc.is_some())
        };
        let member_recorded = |id: ObjectId| match actor {
            NetworkedActor::Npc { timeline, .. } if is_party_npc(id) => {
                timeline.recorded_amount(request.action_id, false)
            }
            _ => None,
        };
        let recorded_for = |id: ObjectId| match actor {
            NetworkedActor::Npc { timeline, .. } => member_recorded(id).or_else(|| {
                timeline.recorded_amount_at(
                    request.action_id,
                    hit_like_a_player(id),
                    common_spawn.level,
                )
            }),
            _ => None,
        };
        // Area attacks land on everyone of the target's side inside them when the cast finishes (aimed where the
        // target was, and facing the way the enemy was, when it started casting), not just on its target: the
        // players, and in content the NPCs fighting beside them (retail's Terror Eye in Duty, Honor, Country hit the
        // player and the allies by them, Bob's Recorder capture of 2026-10-18).
        if let Some(effects_builder) = &mut effects_builder
            && let Some(shape) = action_info.and_then(|info| info.area_shape())
            && let NetworkedActor::Npc { cast, timeline, .. } = actor
            && request.target.object_id != from_actor_id
            && let Some(target) = instance.find_actor(request.target.object_id)
            && effects_builder
                .effects
                .iter()
                .any(|effect| matches!(effect.0, TargetEffectKind::Damage { .. }))
        {
            let (rotation, aimed_at) = match cast {
                Some(cast) => (cast.rotation, cast.target_position),
                None => (actor.rotation(), target.position().0),
            };
            let caught = |position: Vec3A| {
                in_area(
                    shape,
                    actor.position().0,
                    rotation,
                    aimed_at,
                    position,
                    timeline.hitbox_radius,
                )
            };

            // Everyone else gets their own roll of the same attack (but drains only heal from the target.)
            let mut unscaled = effects_builder.effects.clone();
            unscaled.retain(|effect| {
                !matches!(effect.0, TargetEffectKind::Heal { unk1, .. } if unk1[4] & EFFECT_ON_SOURCE != 0)
            });
            let side = area_side(target);
            // NPCs only where the recording says what the attack does to them: without it an NPC's hit on another
            // isn't scaled, which for an area attack nobody aimed at them would be far too much.
            let hits_npcs = timeline.recorded_amount(request.action_id, false).is_some();
            for (id, other) in &instance.actors {
                if *id != request.target.object_id
                    && *id != from_actor_id
                    && area_side(other).is_some()
                    && area_side(other) == side
                    && (hits_npcs || hit_like_a_player(*id))
                    && other.get_common_spawn().health_points > 0
                    && caught(other.position().0)
                {
                    area_bystanders.push((*id, unscaled.clone()));
                }
            }

            if !caught(target.position().0) {
                // The target dodged it.
                effects_builder.effects.clear();
            }
        }

        // An area attack an NPC casts on itself (retail's invisible helpers marking the ground, as the Copperbell Mines'
        // Kottos' three did with 28544, Bob's captures of 2026-10-25): it lands on everyone inside it its battalion is
        // at war with, and not on itself. (Aimed at someone, the area above takes that one's side instead.)
        if let Some(effects_builder) = &mut effects_builder
            && let Some(shape) = action_info.and_then(|info| info.area_shape())
            && let NetworkedActor::Npc { cast, timeline, .. } = actor
            && request.target.object_id == from_actor_id
            && effects_builder
                .effects
                .iter()
                .any(|effect| matches!(effect.0, TargetEffectKind::Damage { .. }))
        {
            let (rotation, aimed_at) = match cast {
                Some(cast) => (cast.rotation, cast.target_position),
                None => (actor.rotation(), actor.position().0),
            };
            let enemies = game_data
                .lock()
                .get_battalion_enemies(common_spawn.battalion as u32);
            let hits_npcs = timeline.recorded_amount(request.action_id, false).is_some();
            for (id, other) in &instance.actors {
                // Only characters: EObjs and coffers have no CommonSpawn (asking one for it panicked the server as
                // Kottos' first marker went off, Bob's bundle of 2026-10-26.)
                if area_side(other).is_none() {
                    continue;
                }
                let other_spawn = other.get_common_spawn();
                if *id != from_actor_id
                    && enemies
                        .get(other_spawn.battalion as usize)
                        .copied()
                        .unwrap_or(false)
                    && (hits_npcs || hit_like_a_player(*id))
                    && other_spawn.health_points > 0
                    && in_area(
                        shape,
                        actor.position().0,
                        rotation,
                        aimed_at,
                        other.position().0,
                        timeline.hitbox_radius,
                    )
                {
                    area_bystanders.push((*id, effects_builder.effects.clone()));
                }
            }
            effects_builder.effects.clear();
        }

        // What retail's recording says this NPC's action does to this kind of target (Timeline::recorded_amounts):
        // allies and enemies in quest battles hurt and heal each other, and the player, as much as they did there.
        let miss_percent = npc_miss_percent(
            instance
                .find_actor(request.target.object_id)
                .map(|target| target.get_common_spawn().level)
                .unwrap_or(common_spawn.level),
            common_spawn.level,
        );
        let recorded = recorded_for(request.target.object_id);

        if let Some(recorded) = recorded
            && let Some(effects_builder) = &mut effects_builder
        {
            let damage: u32 = effects_builder
                .effects
                .iter()
                .filter_map(|effect| match effect.0 {
                    TargetEffectKind::Damage { amount, .. } => Some(amount as u32),
                    _ => None,
                })
                .sum();
            if damage > 0 {
                let scale = recorded / damage as f32;
                roll_npc_hit(&mut effects_builder.effects, scale, miss_percent);
                scale_npc_periodic_damage(&mut effects_builder.periodic, scale);
            } else {
                scale_npc_heal(&mut effects_builder.effects, recorded);
            }
        } else if npc_hitting_player && let Some(effects_builder) = &mut effects_builder {
            let scale = npc_damage_scale(actor, common_spawn.level);
            roll_npc_hit(&mut effects_builder.effects, scale, miss_percent);
            scale_npc_periodic_damage(&mut effects_builder.periodic, scale);
        }
        // Everyone else caught in an area attack is hit as that kind of target would be: what the recording says the
        // action does to a player or to an NPC, or for a player without one the NPC's usual scale.
        for (id, effects) in &mut area_bystanders {
            let Some(bystander) = instance.find_actor(*id) else {
                continue;
            };
            let is_player = hit_like_a_player(*id);
            let damage: u32 = effects
                .iter()
                .filter_map(|effect| match effect.0 {
                    TargetEffectKind::Damage { amount, .. } => Some(amount as u32),
                    _ => None,
                })
                .sum();
            let recorded = recorded_for(*id);
            let scale = match recorded {
                Some(recorded) if damage > 0 => Some(recorded / damage as f32),
                _ if is_player => Some(npc_damage_scale(actor, common_spawn.level)),
                _ => None,
            };
            if let Some(scale) = scale {
                let miss_percent =
                    npc_miss_percent(bystander.get_common_spawn().level, common_spawn.level);
                roll_npc_hit(effects, scale, miss_percent);
            }
        }

        // A player's auto-attack shows their weapon's damage type.
        if let Some(effects_builder) = &mut effects_builder
            && matches!(actor, NetworkedActor::Player { .. })
            && request.action_type == ActionType::Action
            && AUTO_ATTACK_ACTIONS.contains(&request.action_id)
            && let Some(kind) = auto_attack_damage_type(common_spawn.class_job)
        {
            for effect in &mut effects_builder.effects {
                if let TargetEffectKind::Damage { damage_type, .. } = &mut effect.0 {
                    *damage_type = kind;
                }
            }
        }

        // What statuses, traits, critical hits and variance do (see `modifiers`.)
        if let Some(effects_builder) = &mut effects_builder {
            let target = instance.find_actor(request.target.object_id);
            let caster_statuses = status_ids(actor);
            match actor {
                NetworkedActor::Player { parameters, .. } => {
                    let target_statuses = target.map(status_ids).unwrap_or_default();
                    modifiers::apply_player_hit(
                        &mut effects_builder.effects,
                        &modifiers::PlayerHit {
                            chances: modifiers::HitChances::from_stats(
                                parameters.critical_hit,
                                parameters.direct_hit_rate,
                                parameters.base_speed.max(0) as u32,
                                parameters.level_modifier.max(0) as u32,
                            ),
                            caster_statuses: &caster_statuses,
                            target_statuses: &target_statuses,
                            target_classjob: target
                                .map(|target| target.get_common_spawn().class_job)
                                .unwrap_or_default(),
                            target_is_player: matches!(target, Some(NetworkedActor::Player { .. })),
                            rolls: [fastrand::f32(), fastrand::f32(), fastrand::f32()],
                        },
                    );
                }
                NetworkedActor::Npc { .. } => {
                    // A player's defences; a party bot's as a player's, a party NPC's as retail's member took hits
                    // (`PartyBot::damage_taken`.)
                    let defend = |effects: &mut Vec<TargetEffect>, id: ObjectId| {
                        let Some(defender) = instance.find_actor(id) else {
                            return;
                        };
                        let bot = enemy_attacking
                            .then(|| instance.party_bots.get(&id))
                            .flatten();
                        match (defender, bot) {
                            (_, Some(bot)) if bot.party_npc.is_some() => {
                                if member_recorded(id).is_none() {
                                    scale_damage(effects, bot.damage_taken())
                                }
                            }
                            (NetworkedActor::Player { .. }, _) | (_, Some(_)) => {
                                modifiers::apply_hit_on_player(
                                    effects,
                                    &caster_statuses,
                                    &status_ids(defender),
                                    defender.get_common_spawn().class_job,
                                )
                            }
                            _ => {}
                        }
                    };
                    defend(&mut effects_builder.effects, request.target.object_id);
                    for (id, effects) in &mut area_bystanders {
                        defend(effects, *id);
                    }
                }
                _ => {}
            }
        }
    }

    // tell them the action results
    // Gysahl Greens: the companion comes out after the ActionEffect, as retail's did.
    let mut summon_companion = false;
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
                let multiplier = enmity_multiplier(instance, from_actor_id);
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
                // A heal (or anything on oneself) doesn't make the target hate whoever used it: an NPC healed by an
                // ally got the healer on its enmity list, the healing enmity piled up there, and once its own target
                // fell it turned on its healer (Duty, Honor, Country's fistfighters on their spellweaver, Bob's
                // 2026-10-20 bundle).
                let heals_target = effects_builder.effects.iter().any(|effect| {
                    matches!(effect.0, TargetEffectKind::Heal { unk1, .. } if unk1[4] & EFFECT_ON_SOURCE == 0)
                });
                if request.target.object_id != from_actor_id && (damage > 0 || !heals_target) {
                    actor.add_enmity(from_actor_id, damage.saturating_mul(multiplier));
                }
                if effects_builder.provoke {
                    actor.provoke(from_actor_id);
                }
            }

            // Esuna and the like: the target's first status that can be removed goes.
            if effects_builder.cleanse {
                let removed = instance
                    .find_actor(request.target.object_id)
                    .and_then(|actor| actor.status_effects())
                    .and_then(|statuses| {
                        let mut game_data = game_data.lock();
                        statuses
                            .data()
                            .into_iter()
                            .find(|status| game_data.is_dispellable(status.effect_id))
                    });
                if let Some(status) = removed {
                    instance.insert_task(
                        from_id,
                        request.target.object_id,
                        Duration::ZERO,
                        QueuedTaskData::LoseStatusEffect {
                            effect_id: status.effect_id,
                            effect_param: status.param,
                            effect_source_actor_id: status.source_actor_id,
                        },
                    );
                }
            }

            // The first to attack it claims it (FirstAttack), sent before the action's results as retail does.
            if effects_builder.effects.iter().any(|effect| {
                matches!(
                    effect.0,
                    TargetEffectKind::Damage { .. } | TargetEffectKind::Miss
                )
            }) {
                first_attack::tag(
                    &mut network.lock(),
                    instance,
                    request.target.object_id,
                    from_actor_id,
                );
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

            // Handle combos as retail (Bob's Chronofoil captures): only actions in a combo (starting or continuing one)
            // change it, so other spells and weaponskills in between don't break it, and neither do abilities. Spells and
            // weaponskills show ExecuteCombo (value: the action, 128), with 1 when it continued a combo; a combo step
            // used out of its combo shows none, and abilities never do (retail: Fast Blade 205 of 208, Riot Blade 137 of
            // 141; Fight or Flight 0 of 20, Low Blow 0 of 12, Sprint 0 of 43). Sent on Fight or Flight, it made the
            // client drop the Fast Blade -> Riot Blade combo (2026-10-20 bundle.)
            if is_combo_action(action_category) && takes_part_in_combos {
                let Some(actor) = instance.find_actor_mut(from_actor_id) else {
                    return;
                };

                if let NetworkedActor::Player {
                    last_combo_action,
                    combo_sequence,
                    ..
                } = actor
                {
                    *combo_sequence = next_combo_sequence(in_combo, *combo_sequence);
                    *last_combo_action = request.action_id as u16;
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
            }
            let out_of_combo_step = combo_action_id != 0 && !in_combo;
            let caster_is_player = matches!(
                instance.find_actor(from_actor_id),
                Some(NetworkedActor::Player { .. })
            );
            if request.action_type == ActionType::Action
                && caster_is_player
                && is_combo_action(action_category)
                && !AUTO_ATTACK_ACTIONS.contains(&request.action_id)
                && !out_of_combo_step
            {
                effects_builder
                    .effects
                    .push(TargetEffect(TargetEffectKind::ExecuteCombo {
                        sequence: u8::from(in_combo && combo_action_id != 0),
                        unk2: 0,
                        unk3: 0,
                        unk4: 0,
                        unk5: 128,
                        action_id: request.action_id as u16,
                    }));
            }

            // Players' area actions hit every enemy (or ally) inside; enemies' are handled above, on their target.
            let caster_is_player = matches!(
                instance.find_actor(from_actor_id),
                Some(NetworkedActor::Player { .. })
            );
            if let Some(info) =
                action_info.filter(|info| info.is_area_of_effect() && caster_is_player)
            {
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
                        TargetEffectKind::Heal { unk1, amount } => {
                            // Drains heal whoever used them, not the target.
                            let healed = if unk1[4] & EFFECT_ON_SOURCE != 0 {
                                from_actor_id
                            } else {
                                request.target.object_id
                            };
                            let Some(actor) = instance.find_actor_mut(healed) else {
                                continue;
                            };
                            let common_spawn = actor.get_common_spawn_mut();
                            let before = common_spawn.health_points;
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
                            let effective = common_spawn.health_points.saturating_sub(before);
                            add_healing_enmity(instance, from_actor_id, effective);
                        }
                        TargetEffectKind::Damage {
                            amount,
                            damage_element,
                            ..
                        } => {
                            let Some(actor) = instance.find_actor_mut(request.target.object_id)
                            else {
                                continue;
                            };
                            let hit_player = matches!(actor, NetworkedActor::Player { .. });
                            let floor = u32::from(modifiers::has_hp_floor(&status_ids(actor)));
                            let common_spawn = actor.get_common_spawn_mut();
                            if common_spawn.name_id != STRIKING_DUMMY_NAME_ID {
                                common_spawn.health_points = common_spawn
                                    .health_points
                                    .saturating_sub(*amount as u32)
                                    .max(floor.min(common_spawn.health_points));
                            }

                            // Being hit interrupts searching something for a quest.
                            if hit_player && *amount > 0 {
                                instance.interrupt_event_action(
                                    &mut network.lock(),
                                    request.target.object_id,
                                );
                            }

                            // Being hit wakes you up.
                            if *amount > 0
                                && let Some(sleep) = instance
                                    .find_actor(request.target.object_id)
                                    .and_then(|actor| actor.status_effects())
                                    .and_then(|statuses| statuses.get(modifiers::SLEEP))
                            {
                                instance.insert_task(
                                    from_id,
                                    request.target.object_id,
                                    Duration::ZERO,
                                    QueuedTaskData::LoseStatusEffect {
                                        effect_id: sleep.effect_id,
                                        effect_param: sleep.param,
                                        effect_source_actor_id: sleep.source_actor_id,
                                    },
                                );
                            }

                            // Update from game data
                            let mut game_data = game_data.lock();
                            *damage_element =
                                game_data.get_action_damage_element(request.action_id);
                        }
                        TargetEffectKind::InterruptAction => {
                            interrupt_cast(network.clone(), instance, request.target.object_id);
                        }
                        TargetEffectKind::SummonPet { .. } => {
                            let Some(actor) = instance.find_actor(from_actor_id) else {
                                continue;
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
                            // Summoned once the action's results are out (server/companion.rs.)
                            summon_companion = true;
                        }
                        _ => {}
                    }
                }

                update_actor_hp_mp_after_hit(network.clone(), instance, request.target.object_id);
                if from_actor_id != request.target.object_id {
                    // The caster's HP may have changed too, e.g. from a drain.
                    update_actor_hp_mp(network.clone(), instance, from_actor_id);
                }

                // The packet shows the target (if the attack caught it) and the first others it caught; those
                // beyond get flying text of their own.
                let main_shown = usize::from(
                    request.target.object_id != from_actor_id
                        && !effects_builder.effects.is_empty(),
                );
                for (index, (target_id, effects)) in area_bystanders.iter().enumerate() {
                    hit_area_bystander(
                        network.clone(),
                        instance,
                        from_actor_id,
                        *target_id,
                        effects,
                        main_shown + index >= MAX_AOE_PACKET_TARGETS,
                    );
                }
            }
        }

        // TODO: send Cooldown ActorControlSelf

        // The ActionEffect's global sequence, which its results name, and who it showed effects on.
        let effect_sequence;
        let shown_on: Vec<ObjectId>;
        {
            let mut network = network.lock();

            // ActionResult
            {
                let as_object = |object_id: ObjectId| ObjectTypeId {
                    object_id,
                    object_type: ObjectTypeKind::None,
                };
                // Everyone the action landed on, with what it did to each: an area action shows every one of them, as
                // retail does (players' area actions and NPCs' area attacks alike), not one representative target with
                // flying text on the rest.
                let area_action = action_info
                    .is_some_and(|info| info.area_shape().is_some() || info.is_area_of_effect());
                let aimed_at_someone = request.target.object_id.is_valid()
                    && request.target.object_id != ObjectId(0xE000_0000);
                let shown: Vec<(ObjectTypeId, Vec<TargetEffect>)> = match &aoe_result {
                    Some(aoe) => aoe
                        .shown
                        .iter()
                        .map(|(id, effects)| (as_object(*id), effects.clone()))
                        .collect(),
                    None => {
                        let mut shown = Vec::new();
                        // An area attack that missed its target (or one cast on oneself, whose caster takes nothing)
                        // shows only those it caught.
                        if !area_action
                            || (request.target.object_id != from_actor_id
                                && !effects_builder.effects.is_empty())
                        {
                            shown.push((request.target, effects_builder.effects.clone()));
                        }
                        shown.extend(
                            area_bystanders
                                .iter()
                                .map(|(id, effects)| (as_object(*id), effects.clone())),
                        );
                        shown
                    }
                };
                // AoeEffect8 for an area action unless it was aimed at someone and landed on exactly one (retail, Bob's
                // Copperbell Mines captures of 2026-10-25: area attacks that caught no one, several, or were aimed at
                // the ground went as AoeEffect8, 172 of them; one aimed at someone that caught one, as ActionEffect1.)
                let use_aoe_packet = area_action
                    && !(shown.len() == 1 && aimed_at_someone)
                    && shown.len() <= MAX_AOE_PACKET_TARGETS;
                let effects_of = |effects: &[TargetEffect]| {
                    // The packet only has room for 8 effects per target.
                    let mut packed = [TargetEffect::default(); 8];
                    let count = effects.len().min(packed.len());
                    packed[..count].copy_from_slice(&effects[..count]);
                    packed
                };

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

                let animation_lock = npc_animation_lock.unwrap_or_else(|| {
                    animation_lock(request.action_type, request.action_id, has_cast_time)
                });
                // Auto-attacks swing one of three ways, picked at random (retail, Bob's Chronofoil captures: 0, 1 and 2
                // about a third each over 663; Kawari always sent 0.)
                let animation_variation = if request.action_type == ActionType::Action
                    && (AUTO_ATTACK_ACTIONS.contains(&request.action_id) || npc_auto_attack)
                {
                    fastrand::u8(0..3)
                } else {
                    0
                };
                let ipc = if use_aoe_packet {
                    // Aimed at no one: the animation plays on no one, at the spot (retail: 0xE0000000 and the point.)
                    let (animation_target_id, position) = if aimed_at_someone {
                        (request.target, Position::default())
                    } else {
                        let aimed = data
                            .lock()
                            .find_actor_instance(from_actor_id)
                            .and_then(|instance| match instance.find_actor(from_actor_id) {
                                Some(NetworkedActor::Npc {
                                    cast: Some(cast), ..
                                }) => Some(Position(cast.target_position)),
                                _ => None,
                            })
                            .unwrap_or(common_spawn.position);
                        (as_object(ObjectId(0xE000_0000)), aimed)
                    };
                    AoeEffect {
                        animation_target_id,
                        action_id: request.action_id,
                        global_sequence: network.global_action_sequence,
                        animation_lock,
                        source_sequence: request.sequence,
                        rotation: npc_cast_rotation.unwrap_or(common_spawn.rotation),
                        spell_id: action_animation_id,
                        animation_variation,
                        action_type: request.action_type,
                        effects: shown
                            .iter()
                            .map(|(_, effects)| effects_of(effects))
                            .collect(),
                        targets: shown.iter().map(|(target, _)| *target).collect(),
                        position,
                        ..Default::default()
                    }
                    .package()
                    .unwrap()
                } else {
                    let (targets, effects) = match shown.first() {
                        Some((target, effects)) => (vec![*target], effects.clone()),
                        None => (vec![request.target], Vec::new()),
                    };
                    ActionEffect {
                        animation_target_id: request.target,
                        targets,
                        action_id: request.action_id,
                        animation_lock,
                        rotation: npc_cast_rotation.unwrap_or(common_spawn.rotation),
                        spell_id: action_animation_id,
                        animation_variation,
                        source_sequence: request.sequence,
                        effects: effects_of(&effects),
                        action_type: request.action_type,
                        global_sequence: network.global_action_sequence,
                        ..Default::default()
                    }
                    .package()
                    .unwrap()
                };
                effect_sequence = network.global_action_sequence;
                shown_on = shown
                    .iter()
                    .take(if use_aoe_packet {
                        MAX_AOE_PACKET_TARGETS
                    } else {
                        1
                    })
                    .filter(|(_, effects)| !effects.is_empty())
                    .map(|(target, _)| target.object_id)
                    .collect();
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

        if summon_companion {
            if request.action_type == ActionType::Item {
                network.lock().send_to_by_actor_id(
                    from_actor_id,
                    FromServer::ItemUsedUp(request.action_id),
                    DestinationNetwork::ZoneClients,
                );
            }
            companion::summon(&data, &network, &game_data, from_actor_id);
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
                        // The ActionEffect's own (retail: 6,603 of 6,838 results in Bob's 2026-10-21 capture;
                        // Kawari sent the next one, which no effect had.)
                        global_sequence: effect_sequence,
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

            // Each target the effect showed on gets its result as the hit lands: its health, under the effect's
            // global sequence (EffectResultBasic; EffectResult when it gained statuses, above and below.) The client
            // holds an action's damage until then, so without it each hit only showed when the next action's effect
            // came (Bob's 2026-10-21 bundle). Retail sends it 0.5 s after the ActionEffect (3,677 of 6,180).
            {
                let with_statuses = (num_target_entries > 0).then_some(request.target.object_id);
                let mut data = data.lock();
                if let Some(instance) = data.find_actor_instance_mut(from_actor_id) {
                    let lands_in = EFFECT_RESULT_AFTER;
                    for target_id in shown_on.iter().copied() {
                        if Some(target_id) == with_statuses {
                            continue;
                        }
                        let Some(target) = instance.find_actor(target_id) else {
                            continue;
                        };
                        let current_hp = target.get_common_spawn().health_points;
                        let segment =
                            ServerZoneIpcSegment::new(ServerZoneIpcData::EffectResultBasic {
                                count: 1,
                                global_sequence: effect_sequence,
                                target_id,
                                current_hp,
                                unk3: 0,
                                unk4: 0,
                            });
                        instance.insert_task(
                            ClientId::default(),
                            target_id,
                            lands_in,
                            QueuedTaskData::EffectResultLands { segment },
                        );
                    }
                }
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
                        // The ActionEffect's own (retail: 6,603 of 6,838 results in Bob's 2026-10-21 capture;
                        // Kawari sent the next one, which no effect had.)
                        global_sequence: effect_sequence,
                        // The target's own EffectResult names the target (retail, Bob's Chronofoil captures.)
                        target_id: request.target.object_id,
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

        // An NPC that gained statuses (its target, or itself) has its StatusEffectList go out too, as retail sends for
        // party NPCs and enemies on every change, gain and loss (Bob's Copperbell Mines capture with Duty Support,
        // 2026-10-25: 618 lists from NPCs to 15 here). Without it their status icons (a damage over time on an enemy,
        // a party NPC's buff) didn't show on its nameplate or the party list.
        {
            let mut data = data.lock();
            if let Some(instance) = data.find_actor_instance_mut(from_actor_id) {
                let mut changed = vec![request.target.object_id];
                if from_actor_id != request.target.object_id {
                    changed.push(from_actor_id);
                }
                for id in changed {
                    if matches!(instance.find_actor(id), Some(NetworkedActor::Npc { .. })) {
                        process_effects_list(network.clone(), instance, id);
                    }
                }
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
    /// Everyone who received the action's status effects (used for over-time effects.)
    status_targets: Vec<ObjectId>,
    /// Everyone it landed on with what it did to each, in the order the packet shows them.
    shown: Vec<(ObjectId, Vec<TargetEffect>)>,
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
            status_targets: Vec::new(),
            shown: Vec::new(),
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
    let multiplier = enmity_multiplier(instance, from_actor_id);

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
    // What each target took, for the packet.
    let mut damage_taken: Vec<(ObjectId, bool)> = Vec::new();

    // Damage enemies
    if has_damage {
        let falloff = effects_builder.aoe_falloff_percent as u32;
        for (index, target_id) in hostiles.iter().enumerate() {
            first_attack::tag(&mut network.lock(), instance, *target_id, from_actor_id);
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
            actor.add_enmity(from_actor_id, total.saturating_mul(multiplier));

            if !invulnerable {
                let common_spawn = actor.get_common_spawn_mut();
                if common_spawn.name_id != STRIKING_DUMMY_NAME_ID {
                    common_spawn.health_points = common_spawn.health_points.saturating_sub(total);
                }

                // The packet shows the first eight; the rest get numbers of their own.
                if index >= MAX_AOE_PACKET_TARGETS {
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
                interrupt_cast(network.clone(), instance, *target_id);
            }

            damage_taken.push((*target_id, invulnerable));
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
            let before = common_spawn.health_points;
            common_spawn.health_points = if effects_builder.heal_to_full {
                common_spawn.max_health_points
            } else {
                (common_spawn.health_points + total).min(common_spawn.max_health_points)
            };
            let effective = common_spawn.health_points.saturating_sub(before);
            add_healing_enmity(instance, from_actor_id, effective);

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
        update_actor_hp_mp_after_hit(network.clone(), instance, target_id);
    }

    // Each target with what it took: an enemy its own damage (less for all but the first), or Invincible; an ally
    // the heal; and the statuses on those they went to.
    let mut shown: Vec<(ObjectId, Vec<TargetEffect>)> = Vec::new();
    if has_damage {
        let falloff = effects_builder.aoe_falloff_percent as u32;
        for (index, (target_id, invulnerable)) in damage_taken.iter().enumerate() {
            let effects = effects_builder
                .effects
                .iter()
                .filter(|effect| !matches!(effect.0, TargetEffectKind::Heal { .. }))
                .map(|effect| {
                    let mut effect = *effect;
                    match &mut effect.0 {
                        TargetEffectKind::Damage { .. } if *invulnerable => {
                            effect = TargetEffect(TargetEffectKind::Invincible {});
                        }
                        TargetEffectKind::Damage { amount, .. } if index > 0 => {
                            *amount = (*amount as u32 * (100 - falloff) / 100) as u16;
                        }
                        _ => {}
                    }
                    effect
                })
                .collect();
            shown.push((*target_id, effects));
        }
    }
    if has_heal || !has_damage {
        for target_id in &friendlies {
            if shown.iter().any(|(id, _)| id == target_id) {
                continue;
            }
            let effects = effects_builder
                .effects
                .iter()
                .filter(|effect| {
                    !matches!(
                        effect.0,
                        TargetEffectKind::Damage { .. } | TargetEffectKind::InterruptAction
                    )
                })
                .filter(|effect| {
                    !matches!(effect.0, TargetEffectKind::GainEffect { .. }) || !has_damage
                })
                .copied()
                .collect();
            shown.push((*target_id, effects));
        }
    }

    AreaOfEffectResult {
        status_targets,
        shown,
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

pub fn cancel_action(
    network: Arc<Mutex<NetworkState>>,
    instance: &mut Instance,
    from_id: ClientId,
    actor_id: ObjectId,
    action_id: u32,
) {
    let msg = FromServer::ActorControlSelf(ActorControlCategory::CancelCast {
        unk1: 538,
        unk2: 1,
        action_id,
        unk3: 0,
    });

    let mut network = network.lock();
    network.send_to(from_id, msg, DestinationNetwork::ZoneClients);

    // An interrupted NPC can move and act again straight away.
    if let Some(actor) = instance.find_actor_mut(actor_id)
        && let NetworkedActor::Npc { cast, .. } = actor
    {
        *cast = None;
    }
}

/// Interrupts what an actor is casting (Head Graze, Interject): its cast tasks only (its statuses still run out), and
/// everyone nearby sees the cast bar cancelled.
fn interrupt_cast(network: Arc<Mutex<NetworkState>>, instance: &mut Instance, actor_id: ObjectId) {
    let cancelled = instance.cancel_actor_casts(actor_id);
    if let Some(NetworkedActor::Npc { cast, .. }) = instance.find_actor_mut(actor_id) {
        *cast = None;
    }
    let mut network = network.lock();
    for action_id in cancelled {
        network.send_ac_in_range_inclusive_instance(
            instance,
            actor_id,
            ActorControlCategory::CancelCast {
                unk1: 538,
                unk2: 1,
                action_id,
                unk3: 0,
            },
        );
    }
}

/// The IDs of an actor's statuses.
fn status_ids(actor: &NetworkedActor) -> Vec<u16> {
    actor
        .status_effects()
        .map(|statuses| statuses.ids().collect())
        .unwrap_or_default()
}

/// Whether an action takes part in combos: spells and weaponskills (ActionCategory 2 and 3.)
fn is_combo_action(category: Option<ActionCategory>) -> bool {
    matches!(
        category,
        Some(ActionCategory::Spell | ActionCategory::Weaponskill)
    )
}

/// The combo step after a spell or weaponskill: one more when it continued the combo, else back to the start.
fn next_combo_sequence(in_combo: bool, sequence: u8) -> u8 {
    if in_combo {
        sequence.saturating_add(1)
    } else {
        0
    }
}

/// Tank stances (Iron Will, Defiance, Grit, Royal Guard): their wearer's damage makes ten times the enmity, as retail
/// (Bob's Chronofoil captures: fitted ×10.05-10.8 in 9 eight-player duties; no tank bonus without a stance.)
const TANK_STANCE_STATUSES: [u16; 4] = [79, 91, 743, 1833];
const TANK_STANCE_ENMITY_MULTIPLIER: u32 = 10;

/// What an actor's damage is multiplied by for enmity (`TANK_STANCE_STATUSES`.)
fn enmity_multiplier(instance: &Instance, actor_id: ObjectId) -> u32 {
    let in_stance = instance
        .find_actor(actor_id)
        .and_then(|actor| actor.status_effects())
        .is_some_and(|effects| {
            TANK_STANCE_STATUSES
                .iter()
                .any(|status| effects.get(*status).is_some())
        });
    if in_stance {
        TANK_STANCE_ENMITY_MULTIPLIER
    } else {
        1
    }
}

/// Healing makes enmity on every enemy that has the healer on its list: half of what was actually restored (overheal
/// doesn't count), shared between them (retail: 0.43-0.57 in the one clean case in Bob's captures.)
pub fn add_healing_enmity(instance: &mut Instance, healer: ObjectId, effective: u32) {
    if effective == 0 {
        return;
    }
    let enemies: Vec<ObjectId> = instance
        .actors
        .iter()
        .filter(|(_, actor)| {
            matches!(actor, NetworkedActor::Npc { enmity, .. } if enmity.contains_key(&healer))
        })
        .map(|(id, _)| *id)
        .collect();
    if enemies.is_empty() {
        return;
    }
    let share = effective / 2 / enemies.len() as u32;
    for enemy in enemies {
        if let Some(actor) = instance.find_actor_mut(enemy) {
            actor.add_enmity(healer, share);
        }
    }
}

/// When an action's result (EffectResultBasic) follows its ActionEffect on retail: 0.5 s (Bob's Chronofoil capture of
/// 2026-10-21: 3,677 of 6,180 results; a player's cast's comes about 0.57 s after its bar ends.)
const EFFECT_RESULT_AFTER: Duration = Duration::from_millis(500);

/// LogMessage 579, "Cannot execute at this time."
const LOG_CANNOT_EXECUTE: u32 = 579;
/// LogMessage 563, "Invalid target."
const LOG_INVALID_TARGET: u32 = 563;

/// Why the server refuses a player's action, as the LogMessage retail answers with (ActorControlSelf 700,
/// `ActionRejected`), if it does. Only what retail was seen refusing (Bob's Chronofoil captures, 2026-06 to 2026-10,
/// 42 sessions): a caster who has died (579, the one such case), and a target that has died (563: in 8 of 9 the
/// target was killed 0-0.2 s before the request arrived, the 9th had gone) for an action that can't be used on the
/// dead. Retail never
/// refused for range, facing or line of sight in those captures (the client checks them, ActionManager
/// .GetActionInRangeOrLoS, see the fork's combat page), so neither does Kawari.
fn rejection(
    instance: &Instance,
    caster_id: ObjectId,
    caster: &NetworkedActor,
    request: &ActionRequest,
    can_target_dead: bool,
) -> Option<u32> {
    let target_id = request.target.object_id;
    // Only a character the server knows: layout objects and NPCs the client places itself aren't in the instance,
    // and objects have no health.
    let target_hp = if target_id.is_valid() && target_id != caster_id {
        match instance.find_actor(target_id) {
            Some(target @ (NetworkedActor::Player { .. } | NetworkedActor::Npc { .. })) => {
                Some(target.get_common_spawn().health_points)
            }
            _ => None,
        }
    } else {
        None
    };
    rejection_log_message(
        caster.get_common_spawn().health_points,
        target_hp,
        can_target_dead,
    )
}

/// `rejection` without the instance: the caster's health, the target's (`None`: no target, the caster itself, or not a
/// character), and whether the action may be used on the dead.
fn rejection_log_message(
    caster_hp: u32,
    target_hp: Option<u32>,
    can_target_dead: bool,
) -> Option<u32> {
    if caster_hp == 0 {
        Some(LOG_CANNOT_EXECUTE)
    } else if target_hp == Some(0) && !can_target_dead {
        Some(LOG_INVALID_TARGET)
    } else {
        None
    }
}

/// Teleport and Return (General Action rows 5 and 6.)
const TELEPORT_ACTION: u32 = 5;
const RETURN_ACTION: u32 = 6;
/// Auto-attack (Action rows 7 and 8.)
const AUTO_ATTACK_ACTIONS: [u32; 2] = [7, 8];
/// How far a player's auto-attack reaches past their own hitbox and the target's (retail, Bob's Chronofoil captures of
/// 2026-10-18: a Gladiator's 597 auto-attacks all came within 3.6 yalms of the target's hitbox, on the ground; in 17 s
/// spent 3.6 to 4.7 yalms off, none came, and they started again 0.1 s after he stepped back in.)
const AUTO_ATTACK_RANGE: f32 = 3.0;
/// Archers, Bards, Machinists and Dancers shoot from 25 yalms (Sapphire's; not seen in Bob's captures.)
const RANGED_AUTO_ATTACK_RANGE: f32 = 25.0;
const RANGED_AUTO_ATTACK_CLASSES: [u8; 4] = [5, 23, 31, 38];
/// The hitbox assumed for an NPC whose timeline doesn't record one: with the player's 3.5 yalms that's 5, NPCs' own
/// default reach (`attack_range`.)
const UNKNOWN_HITBOX_RADIUS: f32 = 1.5;

/// Who a player's next auto-attack hits, if anyone. As on retail (Bob's Chronofoil captures: 663 auto-attacks, every
/// one on the player's current target, also after switching targets without turning auto-attack off and on), it's
/// their current target, not the one they turned it on with, and only an enemy (or their duel opponent) that's alive
/// and in reach; and none while they're casting (2 of 663 during a cast). Auto-attack stays on meanwhile.
pub fn auto_attack_target(instance: &Instance, player_id: ObjectId) -> Option<ObjectId> {
    let Some(NetworkedActor::Player {
        spawn,
        dueling_opponent_id,
        ..
    }) = instance.find_actor(player_id)
    else {
        return None;
    };
    let target_id = spawn.common.target_id.object_id;
    if target_id == player_id || !target_id.is_valid() || spawn.common.health_points == 0 {
        return None;
    }
    let target = instance.find_actor(target_id)?;
    let target_hitbox = match target {
        NetworkedActor::Npc {
            spawn: npc,
            timeline,
            ..
        } if npc
            .character_data_flags
            .contains(CharacterDataFlag::HOSTILE) =>
        {
            if timeline.hitbox_radius > 0.0 {
                timeline.hitbox_radius
            } else {
                UNKNOWN_HITBOX_RADIUS
            }
        }
        NetworkedActor::Player { .. } if target_id == *dueling_opponent_id => PLAYER_HITBOX_RADIUS,
        _ => return None,
    };
    if target.get_common_spawn().health_points == 0 {
        return None;
    }

    let casting = instance.queued_task.iter().any(|task| {
        task.from_actor_id == player_id
            && matches!(
                task.data,
                QueuedTaskData::CastAction {
                    interruptible: true,
                    ..
                }
            )
    });
    if casting {
        return None;
    }

    in_auto_attack_reach(
        spawn.common.class_job,
        spawn.common.position.0,
        target.position().0,
        target_hitbox,
    )
    .then_some(target_id)
}

/// Whether a player of this class at `from` reaches a target at `to` (with a hitbox of `target_hitbox`) with an
/// auto-attack. Distances are on the ground, as measured.
fn in_auto_attack_reach(class_job: u8, from: Vec3A, to: Vec3A, target_hitbox: f32) -> bool {
    let range = if RANGED_AUTO_ATTACK_CLASSES.contains(&class_job) {
        RANGED_AUTO_ATTACK_RANGE
    } else {
        AUTO_ATTACK_RANGE
    };
    let distance = Vec2::new(from.x - to.x, from.z - to.z).length();
    distance <= range + PLAYER_HITBOX_RADIUS + target_hitbox
}

/// The damage type a player's auto-attack shows: their weapon's (retail: Gladiators slashing, Conjurers and White Mages
/// blunt, Bob's Chronofoil captures; the other classes by their weapons.) Kawari's script sent "Physical" (7).
fn auto_attack_damage_type(class_job: u8) -> Option<DamageType> {
    match class_job {
        1 | 3 | 19 | 21 | 29 | 30 | 32 | 34 | 37 | 39 | 41 => Some(DamageType::Slashing),
        4 | 5 | 22 | 23 | 31 | 35 => Some(DamageType::Piercing),
        2 | 6 | 7 | 20 | 24 | 25 | 26 | 27 | 28 | 33 | 36 | 40 | 42 => Some(DamageType::Blunt),
        _ => None,
    }
}
/// The animation lock retail sends after a spell that had a cast, and after an auto-attack (Bob's Chronofoil captures:
/// Stone, Cure, Holy, Raise... all 0.1).
const ANIMATION_LOCK_AFTER_CAST: f32 = 0.1;
/// After Teleport and Return (2.1 on retail.)
const ANIMATION_LOCK_TELEPORT: f32 = 2.1;

/// The animation lock (ActionEffect's) for an action: 0.6 for instant ones, as retail sends.
fn animation_lock(action_type: ActionType, action_id: u32, has_cast_time: bool) -> f32 {
    // Mounting locks for 0.1 s on retail (Bob's Chronofoil captures.)
    if action_type == ActionType::Mount {
        return ANIMATION_LOCK_AFTER_CAST;
    }
    if action_type != ActionType::Action {
        return ANIMATION_LOCK_TIME;
    }
    // Actions retail gives their own lock (every use the same; Bob's Chronofoil captures.)
    if let Some((_, lock)) = SPECIAL_ANIMATION_LOCKS
        .iter()
        .find(|(id, _)| *id == action_id)
    {
        return *lock;
    }
    if action_id == TELEPORT_ACTION || action_id == RETURN_ACTION {
        ANIMATION_LOCK_TELEPORT
    } else if has_cast_time || AUTO_ATTACK_ACTIONS.contains(&action_id) {
        ANIMATION_LOCK_AFTER_CAST
    } else {
        ANIMATION_LOCK_TIME
    }
}

/// Actions whose animation lock isn't the usual 0.6 s on retail, and theirs.
const SPECIAL_ANIMATION_LOCKS: [(u32, f32); 20] = [
    (2259, 0.35),
    (2261, 0.35),
    (18805, 0.35),
    (18806, 0.35),
    (18807, 0.35),
    (92, 0.8),
    (94, 0.8),
    (96, 0.8),
    (2262, 0.8),
    (7515, 0.8),
    (16478, 0.8),
    (24401, 0.8),
    (24402, 0.8),
    (34684, 0.8),
    (16147, 0.5),
    (16146, 0.7),
    (37008, 0.7),
    (16150, 0.77),
    (16480, 1.5),
    (18780, 1.5),
];

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
                    // The action's ID too, for scripts registered for several (resources/scripts/gathering/.)
                    func.call::<EffectsBuilder>((connection_data, in_combo, request.action_id))
                        .unwrap(),
                );

                Ok(())
            })
            .unwrap();
    } else {
        tracing::warn!("Action {key} isn't scripted yet!");
        // Still answer, with no effects: without an ActionEffect the client waits for one (its hotbar and animation
        // lock hang until they time out.)
        effects_builder = Some(EffectsBuilder::default());
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
    barding: Option<[u8; 14]>,
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

    // Retail's for the Company Chocobo: 1, its colour, its barding's three models (u16 model, u16 variant.)
    let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::Mount {
        id: request.action_id as u16,
        unk1: barding.unwrap_or([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
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

/// Which side of a fight an actor is on, for whom an NPC's area attack catches besides its target: players and the
/// NPCs that aren't hostile to them (`false`), or hostile NPCs (`true`). `None` for anything that isn't a fighter.
fn area_side(actor: &NetworkedActor) -> Option<bool> {
    match actor {
        NetworkedActor::Player { .. } => Some(false),
        NetworkedActor::Npc { spawn, .. }
            if matches!(spawn.common.object_kind, ObjectKind::BattleNpc(_)) =>
        {
            Some(
                spawn
                    .character_data_flags
                    .contains(CharacterDataFlag::HOSTILE),
            )
        }
        _ => None,
    }
}

/// Whether `point` is inside an area attack of this `shape`, used by an NPC at `caster` (with a hitbox of
/// `caster_radius`) facing `rotation` and aimed at `aimed_at`. Distances are on the ground, and the player's own hitbox
/// (`PLAYER_HITBOX_RADIUS`) counts, so grazing the edge is a hit.
pub fn in_area(
    shape: AreaShape,
    caster: Vec3A,
    rotation: f32,
    aimed_at: Vec3A,
    point: Vec3A,
    caster_radius: f32,
) -> bool {
    in_area_with_reach(
        shape,
        caster,
        rotation,
        aimed_at,
        point,
        caster_radius,
        PLAYER_HITBOX_RADIUS,
    )
}

/// `in_area` counting `reach` yalms past the shape's edge instead of the player's hitbox (party bots dodging keep a
/// margin beyond it: `dodge.rs`.)
pub fn in_area_with_reach(
    shape: AreaShape,
    caster: Vec3A,
    rotation: f32,
    aimed_at: Vec3A,
    point: Vec3A,
    caster_radius: f32,
    reach: f32,
) -> bool {
    let flat = |v: Vec3A| Vec2::new(v.x, v.z);
    let offset = flat(point) - flat(caster);
    let distance = offset.length();
    // Facing `rotation` means looking along (sin, cos) on the ground, the same way NPCs turn to their targets.
    let forward = Vec2::new(rotation.sin(), rotation.cos());

    match shape {
        AreaShape::Circle { radius, on_target } => {
            if on_target {
                flat(point).distance(flat(aimed_at)) <= radius + reach
            } else {
                distance <= radius + caster_radius + reach
            }
        }
        AreaShape::Cone { radius, angle } => {
            if distance > radius + caster_radius + reach {
                return false;
            }
            if distance <= caster_radius + reach {
                return true;
            }
            let between = (forward.dot(offset) / distance).clamp(-1.0, 1.0).acos();
            let graze = (reach / distance).atan();
            between <= angle.to_radians() / 2.0 + graze
        }
        AreaShape::Line { length, width } => {
            let along = offset.dot(forward);
            let across = offset.perp_dot(forward).abs();
            along >= -reach
                && along <= length + caster_radius + reach
                && across <= width / 2.0 + reach
        }
        AreaShape::Donut {
            radius,
            hole,
            on_target,
        } => {
            // The ring as the client draws it: its outer edge as a circle's, the hole that share of it.
            let (from_middle, outer) = if on_target {
                (flat(point).distance(flat(aimed_at)), radius)
            } else {
                (distance, radius + caster_radius)
            };
            from_middle <= outer + reach && from_middle + reach >= outer * hole
        }
        AreaShape::Cross { length, width } => {
            let along = offset.dot(forward).abs();
            let across = offset.perp_dot(forward).abs();
            let arm = length + caster_radius + reach;
            let half = width / 2.0 + reach;
            (along <= arm && across <= half) || (across <= arm && along <= half)
        }
    }
}

#[cfg(test)]
mod combo_tests {
    use super::*;

    /// What retail refused (ActorControlSelf 700): a dead caster, a dead target, but not Raise on a dead target.
    #[test]
    fn rejections() {
        assert_eq!(rejection_log_message(0, None, false), Some(579));
        assert_eq!(rejection_log_message(1000, Some(0), false), Some(563));
        assert_eq!(rejection_log_message(1000, Some(0), true), None);
        assert_eq!(rejection_log_message(1000, Some(5), false), None);
        assert_eq!(rejection_log_message(1000, None, false), None);
    }

    /// Fast Blade → Fight or Flight → Riot Blade: the ability in between doesn't touch the combo, the second
    /// weaponskill continues it, and an unrelated one starts over.
    #[test]
    fn abilities_dont_break_combos() {
        assert!(is_combo_action(Some(ActionCategory::Weaponskill)));
        assert!(is_combo_action(Some(ActionCategory::Spell)));
        assert!(!is_combo_action(Some(ActionCategory::Ability)));
        assert!(!is_combo_action(Some(ActionCategory::AutoAttack)));
        assert!(!is_combo_action(None));

        assert_eq!(next_combo_sequence(false, 0), 0);
        assert_eq!(next_combo_sequence(true, 0), 1);
        assert_eq!(next_combo_sequence(true, 1), 2);
        assert_eq!(next_combo_sequence(false, 2), 0);
    }

    /// Retail's animation locks (Bob's Chronofoil captures.)
    #[test]
    fn animation_locks() {
        assert_eq!(animation_lock(ActionType::Action, 119, true), 0.1); // Stone
        assert_eq!(animation_lock(ActionType::Action, 121, false), 0.6); // Aero
        assert_eq!(animation_lock(ActionType::Action, 7, false), 0.1); // auto-attack
        assert_eq!(animation_lock(ActionType::Action, 5, true), 2.1); // Teleport
        assert_eq!(animation_lock(ActionType::Action, 6, true), 2.1); // Return
        assert_eq!(animation_lock(ActionType::Item, 4551, false), 0.6);
        assert_eq!(animation_lock(ActionType::Mount, 1, false), 0.1);
        assert_eq!(animation_lock(ActionType::Action, 92, false), 0.8); // Jump
        assert_eq!(animation_lock(ActionType::Action, 2259, false), 0.35);
    }

    /// Retail's reach (Bob's Chronofoil captures): 3.5 yalms past the target's hitbox, on the ground.
    #[test]
    fn auto_attack_reach() {
        let reaches = |class_job, x, y, z, hitbox| {
            in_auto_attack_reach(class_job, Vec3A::ZERO, Vec3A::new(x, y, z), hitbox)
        };
        // A Gladiator next to an antling soldier (hitbox 1.95), and 3.4 / 3.7 yalms past its edge.
        assert!(reaches(1, 2.0, 0.0, 0.0, 1.95));
        assert!(reaches(1, 5.35, 0.0, 0.0, 1.95));
        assert!(!reaches(1, 5.65, 0.0, 0.0, 1.95));
        // Height doesn't count.
        assert!(reaches(1, 3.0, 2.5, 0.0, 0.4));
        // Bob's 2026-10-20 bundle: 9 yalms away is out of reach of a sun bat (0.4); an Archer reaches it.
        assert!(!reaches(1, 0.0, 0.0, 9.0, 0.4));
        assert!(reaches(5, 0.0, 0.0, 9.0, 0.4));

        assert_eq!(auto_attack_damage_type(1), Some(DamageType::Slashing));
        assert_eq!(auto_attack_damage_type(24), Some(DamageType::Blunt));
    }
}

#[cfg(test)]
mod area_tests {
    use super::*;

    #[test]
    fn frost_breath_cone() {
        // An anole (hitbox 1.12) facing +z breathes a 90 degree, 6 yalm cone.
        let cone = AreaShape::Cone {
            radius: 6.0,
            angle: 90.0,
        };
        let at = |x: f32, z: f32| {
            in_area(
                cone,
                Vec3A::ZERO,
                0.0,
                Vec3A::ZERO,
                Vec3A::new(x, 0.0, z),
                1.12,
            )
        };
        assert!(at(0.0, 4.0)); // right in front
        assert!(at(2.5, 4.0)); // inside the cone's edge
        assert!(!at(0.0, -4.0)); // behind it
        assert!(!at(6.0, 1.0)); // well off to the side
        assert!(!at(0.0, 9.0)); // out of reach
    }

    #[test]
    fn ground_circle() {
        // Acorn Armageddon: a 6 yalm circle where the target stood.
        let circle = AreaShape::Circle {
            radius: 6.0,
            on_target: true,
        };
        let aimed = Vec3A::new(10.0, 0.0, 0.0);
        assert!(in_area(
            circle,
            Vec3A::ZERO,
            0.0,
            aimed,
            Vec3A::new(14.0, 0.0, 0.0),
            3.0
        ));
        assert!(!in_area(
            circle,
            Vec3A::ZERO,
            0.0,
            aimed,
            Vec3A::new(18.0, 0.0, 0.0),
            3.0
        ));
    }

    #[test]
    fn leaving_a_circle_is_a_miss() {
        // Terror Eye: 6 yalms around where the player stood. The 2026-10-17 bundle's player was hit at 6.6 and 6.7
        // yalms; only half a yalm past the edge (their own hitbox) still counts.
        let circle = AreaShape::Circle {
            radius: 6.0,
            on_target: true,
        };
        let at = |x: f32| {
            in_area(
                circle,
                Vec3A::ZERO,
                0.0,
                Vec3A::ZERO,
                Vec3A::new(x, 0.0, 0.0),
                1.44,
            )
        };
        assert!(at(4.5));
        assert!(at(6.4));
        assert!(!at(6.6));
        assert!(!at(6.7));
        assert!(!at(7.8));
    }
}

/// Brings an NPC's heal (not one on itself from a drain) to `recorded`, the average retail's recording shows for it,
/// varied a little like its hits.
/// An NPC's damage over time is scaled as its hit was: the action script works it out from the NPC's own stats as
/// if it were a player's, which made the corsair conjurers' Aero tick for 26-28 on the player where retail's ticks
/// were 2-7 (its hit, scaled to the recorded 7, was right; Bob's bundle of 2026-10-22 and his Chronofoil capture of
/// 2026-10-21, both runs of Victory in Peril.) Aero's tick is 30 potency to the hit's 50, so the same scale gives 4.
fn scale_npc_periodic_damage(periodic: &mut [PeriodicSpec], scale: f32) {
    for spec in periodic.iter_mut() {
        if matches!(spec.kind, PeriodicKind::Damage) && !spec.on_self {
            spec.amount = ((spec.amount as f32 * scale).round() as u32).max(1);
        }
    }
}

fn scale_npc_heal(effects: &mut [TargetEffect], recorded: f32) {
    for effect in effects.iter_mut() {
        if let TargetEffectKind::Heal { unk1, amount } = &mut effect.0
            && unk1[4] & EFFECT_ON_SOURCE == 0
        {
            let variance = 1.0 + (fastrand::f32() * 2.0 - 1.0) * NPC_DAMAGE_VARIANCE;
            *amount = (recorded * variance).round().clamp(1.0, u16::MAX as f32) as u16;
        }
    }
}

/// Rolls one enemy hit on a player: scaled to what retail does (see `npc_damage_scale`), with a little variance, and
/// a chance to miss, be parried or be critical. Drains give back what they took.
/// Multiplies an NPC hit's damage by `factor` (at least 1 a hit.)
fn scale_damage(effects: &mut [TargetEffect], factor: f32) {
    for effect in effects.iter_mut() {
        if let TargetEffectKind::Damage { amount, .. } = &mut effect.0 {
            *amount = (*amount as f32 * factor)
                .round()
                .clamp(1.0, u16::MAX as f32) as u16;
        }
    }
}

fn roll_npc_hit(effects: &mut Vec<TargetEffect>, scale: f32, miss_percent: u8) {
    let roll = fastrand::u8(..100);
    let missed = roll < miss_percent;
    let parried = !missed && roll < miss_percent + NPC_PARRY_PERCENT;
    if missed {
        // Nothing lands: no damage, and no drain either.
        effects.retain(|effect| {
            !matches!(effect.0, TargetEffectKind::Heal { unk1, .. } if unk1[4] & EFFECT_ON_SOURCE != 0)
        });
        for effect in effects.iter_mut() {
            if matches!(effect.0, TargetEffectKind::Damage { .. }) {
                effect.0 = TargetEffectKind::Miss;
            }
        }
    }
    let mut dealt = 0u32;
    for effect in effects.iter_mut() {
        if let TargetEffectKind::Damage {
            amount,
            damage_kind,
            ..
        } = &mut effect.0
        {
            let variance = 1.0 + (fastrand::f32() * 2.0 - 1.0) * NPC_DAMAGE_VARIANCE;
            let mut damage = *amount as f32 * scale * variance;
            if parried {
                damage *= NPC_PARRY_MULTIPLIER;
            } else if fastrand::u8(..100) < NPC_CRITICAL_PERCENT {
                damage *= NPC_CRITICAL_MULTIPLIER;
                *damage_kind |= DamageKind::CRITICAL;
            }
            *amount = damage.round().clamp(1.0, u16::MAX as f32) as u16;
            dealt += *amount as u32;
        }
    }

    for effect in effects.iter_mut() {
        if let TargetEffectKind::Heal { unk1, amount } = &mut effect.0
            && unk1[4] & EFFECT_ON_SOURCE != 0
        {
            *amount = dealt.min(u16::MAX as u32) as u16;
        }
    }
}

/// Lands an enemy's area attack on another player (or NPC of the target's side) inside it: their damage (shown as
/// flying text, since the ActionEffect packet only carries the target), any statuses, their new HP, and for an NPC
/// enmity on the attacker as for its target.
fn hit_area_bystander(
    network: Arc<Mutex<NetworkState>>,
    instance: &mut Instance,
    from_actor_id: ObjectId,
    target_id: ObjectId,
    effects: &[TargetEffect],
    flying_text: bool,
) {
    let damage: u32 = effects
        .iter()
        .filter_map(|effect| match effect.0 {
            TargetEffectKind::Damage { amount, .. } => Some(amount as u32),
            _ => None,
        })
        .sum();
    let missed = effects
        .iter()
        .any(|effect| matches!(effect.0, TargetEffectKind::Miss));

    let Some(actor) = instance.find_actor_mut(target_id) else {
        return;
    };
    let floor = u32::from(modifiers::has_hp_floor(&status_ids(actor)));
    if actor.get_common_spawn().health_points == 0 {
        return;
    }
    actor.add_enmity(from_actor_id, damage);
    let common_spawn = actor.get_common_spawn_mut();
    common_spawn.health_points = common_spawn
        .health_points
        .saturating_sub(damage)
        .max(floor.min(common_spawn.health_points));

    if damage > 0 {
        let mut network = network.lock();
        // Being hit interrupts searching something for a quest.
        instance.interrupt_event_action(&mut network, target_id);
        // Shown in the area attack's packet otherwise (AoeEffect8.)
        if flying_text {
            network.send_ac_in_range_inclusive_instance(
                instance,
                target_id,
                ActorControlCategory::DamageFlyingText {
                    unk1: 0,
                    amount: damage,
                    source_actor_id: from_actor_id,
                    unk4: 0,
                },
            );
        }
    }

    if !missed {
        for effect in effects {
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
                    target_id,
                    effect_id,
                    param,
                    duration,
                    from_actor_id,
                    true,
                );
            }
        }
    }

    update_actor_hp_mp_after_hit(network, instance, target_id);
}
