//! The chocobo companion: summoned with Gysahl Greens, following and fighting beside its owner, told what to do
//! (stances), wearing barding, ridden. See `docs/src/fork/chocobo.md`.
//!
//! What the client knows about it (name, rank, EXP, skill points, levels, barding) is kept with the character and
//! sent at login (PlayerSetup); the zone connection sends it here too (`ToServer::SetCompanion`), so the server can
//! summon it in whichever instance its owner is. Every packet and number here is from Bob's Chronofoil capture of
//! 2026-10-24 (e8158807, Middle La Noscea: summoning, riding, dismounting, barding, stances, skills, withdrawing),
//! unless it says otherwise.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use kawari::{
    common::{LegacyEquipmentModelId, ObjectId},
    config::get_config,
    ipc::zone::{
        ActorControlCategory, BattleNpcSubKind, CharacterDataFlag, CommonSpawn, DisplayFlag, Equip,
        ObjectKind, ServerZoneIpcData, ServerZoneIpcSegment, SpawnNpc, StatusEffect,
        StatusEffectList,
    },
};
use parking_lot::Mutex;

use crate::{
    FromServer, GameData, ToServer,
    server::{
        WorldServer,
        actor::{NetworkedActor, NpcState, create_npc_common_spawn},
        instance::Instance,
        network::{DestinationNetwork, NetworkState},
    },
};

/// The companion's BNpcBase (retail's spawn.)
pub const COMPANION_BNPC_BASE: u32 = 952;
/// The Mount row of one's own chocobo ("Company Chocobo"): riding it takes the companion in.
pub const COMPANION_MOUNT: u16 = 1;

/// BuddyAction rows: the commands (client trigger 1700.)
pub const COMMAND_WITHDRAW: u8 = 2;
pub const COMMAND_FOLLOW: u8 = 3;
pub const COMMAND_FREE: u8 = 4;
pub const COMMAND_DEFENDER: u8 = 5;
pub const COMMAND_ATTACKER: u8 = 6;
pub const COMMAND_HEALER: u8 = 7;

/// One Gysahl Greens: 30 minutes (retail's CompanionUpdate said 180,000 centiseconds after one.)
const SUMMON_TIME: Duration = Duration::from_secs(30 * 60);
/// Greens used while it's out add to its time, up to an hour. A guess: the capture used one at a time.
const MAX_SUMMON_TIME: Duration = Duration::from_secs(60 * 60);
/// Withdrawn, it leaves; retail took it away (SetupPet with no pet, then DeleteActor) 2 s later.
const WITHDRAW_TIME: Duration = Duration::from_millis(2000);
/// Retail sent ActorControl 36 and 54 for a companion 0.7-0.85 s after its spawn.
const APPEAR_TIME: Duration = Duration::from_millis(800);

/// Its HP is this much of its owner's: retail's was 36,330 by Bob's 69,756 (both level 90.)
const HP_OF_OWNER: f32 = 36_330.0 / 69_756.0;
/// Its MP (retail's spawn, 7,000 of 7,000.)
const MP: u16 = 7000;
/// Its auto-attack (Action 871, the enemies' piercing one: another player's companion used it on a monster.)
const AUTO_ATTACK: u32 = 871;
/// How close it gets to what it attacks. A guess: no retail fight of Bob's companion was recorded.
const ATTACK_RANGE: f32 = 3.0;
/// A chocobo's body model (ModelChara 1's), in the legs slot of its models: id 1, variant 1, and the colour as its stain.
const BODY_MODEL: u16 = 1;
/// The colour (Stain) of a companion whose colour was never changed: Desert Yellow (retail's spawn.)
const DEFAULT_STAIN: u8 = 36;

/// Unnamed ActorControls retail sent (their numbers):
/// 36 `[1, 18009]` and 54 `[1]` on the companion just after it appeared (54 is Sapphire's ToggleNameHidden);
const AC_COMPANION_APPEARED: u32 = 36;
const AC_COMPANION_APPEARED_ARG: u32 = 18009;
const AC_NAME_SHOWN: u32 = 54;
/// 272 `[0]` on the companion as its owner mounted it;
const AC_COMPANION_MOUNTED: u32 = 272;
/// 928 `[12]` to the owner then (Kawari's `PillionPassengerRelatedUnk`.)
const ACS_MOUNT_SEATS: u32 = 12;

/// What the companion is (the character's, kept by the zone connection.)
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CompanionProfile {
    pub name: String,
    pub exp: u32,
    pub rank: u8,
    pub skill_points: u8,
    /// Its BuddyAction stance (`COMMAND_*`.)
    pub command: u8,
    /// Its barding's models: head, body, legs (BuddyEquip's ModelTop, ModelBody, ModelLegs.)
    pub models: [LegacyEquipmentModelId; 3],
    /// Its barding as BuddyEquip rows (the Mount packet and others' view of it ridden.)
    pub barding: [u8; 3],
    /// Its colour, a Stain row (0: never changed.)
    pub color: u8,
}

impl CompanionProfile {
    fn stain(&self) -> u8 {
        if self.color == 0 {
            DEFAULT_STAIN
        } else {
            self.color
        }
    }

    /// Its stance, Free when it was never told one.
    pub fn stance(&self) -> u8 {
        if (COMMAND_FOLLOW..=COMMAND_HEALER).contains(&self.command) {
            self.command
        } else {
            COMMAND_FREE
        }
    }

    /// Its models as a spawn or Equip carries them: barding in head, body and feet, its body (and colour) in legs.
    fn equipment_models(&self) -> [LegacyEquipmentModelId; 10] {
        let mut models = [LegacyEquipmentModelId::default(); 10];
        models[0] = self.models[0];
        models[1] = self.models[1];
        models[3] = LegacyEquipmentModelId {
            id: BODY_MODEL,
            variant: 1,
            stain: self.stain(),
        };
        models[4] = self.models[2];
        models
    }

    /// The Mount packet's bytes after the mount: 1, the colour, then the barding's models (u16 model, u16 variant
    /// each), as retail sent for the Company Chocobo.
    pub fn mount_bytes(&self) -> [u8; 14] {
        let mut bytes = [0u8; 14];
        bytes[0] = 1;
        bytes[1] = self.stain();
        for (index, model) in [self.models[0], self.models[1], self.models[2]]
            .iter()
            .enumerate()
        {
            let at = 2 + index * 4;
            bytes[at..at + 2].copy_from_slice(&model.id.to_le_bytes());
            bytes[at + 2..at + 4].copy_from_slice(&(model.variant as u16).to_le_bytes());
        }
        bytes
    }
}

/// A player's companion: what it is, and whether it's out.
#[derive(Debug, Clone, Default)]
pub struct Companion {
    pub profile: CompanionProfile,
    /// When its time runs out, while it's out.
    until: Option<Instant>,
    /// Its time left while its owner rides it (the clock stops: retail's went from 180,000 to 179,074 cs over a
    /// ride of 9.3 s that began 9.3 s after the summon... the 926 cs were the time before the ride.)
    paused: Option<Duration>,
    /// Its actor, while it's spawned.
    pub actor: Option<ObjectId>,
    /// Withdrawn: its actor, taken away at this time.
    leaving: Option<(ObjectId, Instant)>,
    /// Just spawned: ActorControl 36 and 54 go out at this time.
    appearing: Option<(ObjectId, Instant)>,
}

impl Companion {
    fn remaining(&self) -> Duration {
        match (self.until, self.paused) {
            (Some(until), _) => until.saturating_duration_since(Instant::now()),
            (None, Some(paused)) => paused,
            _ => Duration::ZERO,
        }
    }

    fn is_out(&self) -> bool {
        self.until.is_some() || self.paused.is_some()
    }
}

fn centiseconds(duration: Duration) -> u32 {
    (duration.as_millis() / 10) as u32
}

/// The owner's CompanionUpdate (ActorControlSelf 253): out or not, its time when `with_time`, EXP, rank, points.
fn send_update(
    network: &mut NetworkState,
    owner: ObjectId,
    companion: &Companion,
    with_time: bool,
) {
    network.send_to_by_actor_id(
        owner,
        FromServer::ActorControlSelf(ActorControlCategory::CompanionUpdate {
            summoned: companion.actor.is_some() && companion.leaving.is_none(),
            set_time_remaining: with_time,
            time_remaining: centiseconds(companion.remaining()),
            unk4: companion.profile.exp,
            unk5: companion.profile.rank as u32,
            unk6: companion.profile.skill_points as u32,
        }),
        DestinationNetwork::ZoneClients,
    );
}

fn send_self(network: &mut NetworkState, owner: ObjectId, category: ActorControlCategory) {
    network.send_to_by_actor_id(
        owner,
        FromServer::ActorControlSelf(category),
        DestinationNetwork::ZoneClients,
    );
}

fn unknown(category: u32, param1: u32, param2: u32) -> ActorControlCategory {
    ActorControlCategory::Unknown {
        category,
        param1,
        param2,
        param3: 0,
        param4: 0,
        param5: 0,
    }
}

/// The companion's StatusEffectList, to its owner (retail sent one before the spawn, for the party list.)
fn send_status_list(
    network: &mut NetworkState,
    owner: ObjectId,
    actor_id: ObjectId,
    spawn: &SpawnNpc,
) {
    let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::StatusEffectList(StatusEffectList {
        statuses: [StatusEffect::default(); 30],
        classjob_id: spawn.common.class_job,
        level: spawn.common.level,
        flags: spawn.common.level,
        health_points: spawn.common.health_points,
        max_health_points: spawn.common.max_health_points,
        resource_points: spawn.common.resource_points,
        max_resource_points: spawn.common.max_resource_points,
        ..Default::default()
    }));
    network.send_to_by_actor_id(
        owner,
        FromServer::PacketSegment(ipc, actor_id),
        DestinationNetwork::ZoneClients,
    );
}

/// The companion as retail spawned it: BNpcBase 952 with its name, owned by the player, of sub-kind Chocobo, at its
/// owner's level with about half their HP, its barding and colour.
fn companion_spawn(
    game_data: &mut GameData,
    owner_id: ObjectId,
    owner: &CommonSpawn,
    profile: &CompanionProfile,
) -> SpawnNpc {
    let hp = ((owner.max_health_points as f32 * HP_OF_OWNER) as u32).max(1);
    let base = create_npc_common_spawn(
        game_data,
        COMPANION_BNPC_BASE,
        0,
        Some(hp),
        owner.level as u32,
    );
    SpawnNpc {
        common: CommonSpawn {
            name: profile.name.clone(),
            owner_id,
            object_kind: ObjectKind::BattleNpc(BattleNpcSubKind::Chocobo),
            level: owner.level,
            class_job: 0,
            position: owner.position,
            rotation: owner.rotation,
            max_health_points: hp,
            health_points: hp,
            max_resource_points: MP,
            resource_points: MP,
            // Retail's were 0x40028; 0x20 (Kawari's INVISIBLE) is left out, as nothing seen afterwards would show it.
            display_flags: DisplayFlag::UNK1 | DisplayFlag::UNK2,
            // Retail's: ModelChara 1, ModelScale 27.
            model_chara: 1,
            model_scale_id: 27,
            battalion: 0,
            models: profile.equipment_models(),
            ..base.common
        },
        character_data_flags: CharacterDataFlag::from_bits_retain(1),
        normal_scaling: true,
        max_links: 4,
        link_range: 20,
        ..base
    }
}

/// Spawns the companion beside its owner in their instance and tells the owner, in retail's order: CompanionUpdate,
/// its command (264), SetupPet, its StatusEffectList, CompanionUpdate again; then the spawn, and 36 and 54 on it.
/// `after_ride` adds what retail sent coming off it (263 1, then 0, and SetPetParameters.)
fn spawn(
    data: &mut WorldServer,
    network: &Arc<Mutex<NetworkState>>,
    game_data: &Arc<Mutex<GameData>>,
    owner: ObjectId,
    after_ride: bool,
) {
    let Some(companion) = data.companions.get(&owner).cloned() else {
        return;
    };
    let Some(instance) = data.find_actor_instance_mut(owner) else {
        return;
    };
    let Some(owner_spawn) = instance
        .find_actor(owner)
        .map(|actor| actor.get_common_spawn().clone())
    else {
        return;
    };

    let actor_id = Instance::generate_actor_id();
    let spawn = {
        let mut game_data = game_data.lock();
        companion_spawn(&mut game_data, owner, &owner_spawn, &companion.profile)
    };

    {
        let mut network = network.lock();
        if !after_ride {
            send_update(
                &mut network,
                owner,
                &Companion {
                    actor: Some(actor_id),
                    ..companion.clone()
                },
                true,
            );
        }
        send_self(
            &mut network,
            owner,
            ActorControlCategory::UnkCompanionRelated {
                unk1: actor_id,
                unk2: companion.profile.stance() as u32,
            },
        );
        if after_ride {
            send_self(
                &mut network,
                owner,
                ActorControlCategory::SetPetEntityId { unk1: 1 },
            );
        }
        send_self(
            &mut network,
            owner,
            ActorControlCategory::SetupPet {
                owner_id: owner,
                pet_id: 0,
                pet_actor_id: actor_id,
                unk2: 1,
                unk3: 0,
            },
        );
        send_status_list(&mut network, owner, actor_id, &spawn);
    }

    let config = get_config();
    {
        let mut game_data = game_data.lock();
        instance.insert_npc(actor_id, spawn, &config, &mut game_data);
    }
    if let Some(NetworkedActor::Npc {
        timeline, state, ..
    }) = instance.find_actor_mut(actor_id)
    {
        timeline.autoattack_action_id = AUTO_ATTACK;
        timeline.attack_range = ATTACK_RANGE;
        timeline.rotation = None;
        *state = NpcState::Follow;
    }

    let Some(companion) = data.companions.get_mut(&owner) else {
        return;
    };
    companion.actor = Some(actor_id);
    companion.appearing = Some((actor_id, Instant::now() + APPEAR_TIME));
    if let Some(paused) = companion.paused.take() {
        companion.until = Some(Instant::now() + paused);
    }
    let mut network = network.lock();
    send_update(&mut network, owner, companion, true);
    if after_ride {
        send_self(
            &mut network,
            owner,
            ActorControlCategory::SetPetEntityId { unk1: 0 },
        );
        send_update(&mut network, owner, companion, true);
        send_self(
            &mut network,
            owner,
            ActorControlCategory::SetPetParameters {
                pet_id: 0,
                unk2: 0,
                unk3: 0,
                unk4: 7,
            },
        );
    }
    tracing::info!("{owner}'s chocobo companion came out as {actor_id}");
}

/// Gysahl Greens were used: the companion comes out for 30 minutes, or its time goes up if it's already out.
pub(super) fn summon(
    data: &Arc<Mutex<WorldServer>>,
    network: &Arc<Mutex<NetworkState>>,
    game_data: &Arc<Mutex<GameData>>,
    owner: ObjectId,
) {
    let mut data = data.lock();
    let Some(companion) = data.companions.get_mut(&owner) else {
        tracing::warn!("{owner} used Gysahl Greens, but their companion isn't known to the server");
        return;
    };
    let remaining = if companion.leaving.is_some() {
        Duration::ZERO
    } else {
        companion.remaining()
    };
    let time = (remaining + SUMMON_TIME).min(MAX_SUMMON_TIME);
    if companion.paused.is_some() {
        companion.paused = Some(time);
        return;
    }
    let already_out = companion.actor.is_some() && companion.leaving.is_none();
    companion.until = Some(Instant::now() + time);
    if already_out {
        send_update(&mut network.lock(), owner, companion, true);
        return;
    }
    spawn(&mut data, network, game_data, owner, false);
}

/// Takes the companion away (withdrawn, its time up, or fallen): on it ActorControl 258, the owner told it's gone
/// (CompanionUpdate, its command 264 with Withdraw), then 2 s later SetupPet with no pet and it's deleted.
fn withdraw(data: &mut WorldServer, network: &mut NetworkState, owner: ObjectId) {
    let Some(companion) = data.companions.get_mut(&owner) else {
        return;
    };
    companion.until = None;
    companion.paused = None;
    let Some(actor_id) = companion.actor else {
        return;
    };
    if companion.leaving.is_some() {
        return;
    }
    companion.leaving = Some((actor_id, Instant::now() + WITHDRAW_TIME));
    let companion = companion.clone();
    if let Some(instance) = data.find_actor_instance(actor_id) {
        network.send_ac_in_range_instance(
            instance,
            actor_id,
            ActorControlCategory::DismissCompanion { unk1: 1 },
        );
    }
    send_update(network, owner, &companion, true);
    send_self(
        network,
        owner,
        ActorControlCategory::UnkCompanionRelated {
            unk1: actor_id,
            unk2: COMMAND_WITHDRAW as u32,
        },
    );
}

/// Takes the companion's actor out of the world (no message to its owner.)
fn remove_actor(data: &mut WorldServer, network: &mut NetworkState, actor_id: ObjectId) {
    if let Some(instance) = data.find_actor_instance_mut(actor_id) {
        network.remove_actor(instance, actor_id);
    }
}

/// The owner rides their own chocobo (Mount 1): the companion goes in (ActorControl 272 on it, then it's taken away),
/// its clock stops, and the owner is told (928 with 12, 263 with 1) around the Mount packet.
pub(super) fn mounted(data: &mut WorldServer, network: &mut NetworkState, owner: ObjectId) {
    let Some(companion) = data.companions.get_mut(&owner) else {
        return;
    };
    send_self(network, owner, unknown(928, ACS_MOUNT_SEATS, 0));
    let Some(actor_id) = companion.actor.take() else {
        return;
    };
    if companion.leaving.is_some() {
        companion.actor = Some(actor_id);
        return;
    }
    companion.paused = Some(companion.remaining());
    companion.until = None;
    companion.appearing = None;
    if let Some(instance) = data.find_actor_instance(actor_id) {
        network.send_ac_in_range_instance(instance, actor_id, unknown(AC_COMPANION_MOUNTED, 0, 0));
    }
    remove_actor(data, network, actor_id);
    send_self(
        network,
        owner,
        ActorControlCategory::SetPetEntityId { unk1: 1 },
    );
}

/// The owner's companion barding, for the Mount packet of their own chocobo.
pub(super) fn mount_bytes(data: &WorldServer, owner: ObjectId) -> Option<[u8; 14]> {
    data.companions
        .get(&owner)
        .map(|companion| companion.profile.mount_bytes())
}

/// Its barding as BuddyEquip rows, for the owner's spawn while riding their chocobo.
pub(super) fn barding(data: &WorldServer, owner: ObjectId) -> Option<([u8; 3], u8)> {
    data.companions
        .get(&owner)
        .map(|companion| (companion.profile.barding, companion.profile.color))
}

/// Whether this actor is someone's chocobo companion (enemies hit it as they hit players.)
pub fn is_companion(instance: &Instance, actor_id: ObjectId) -> bool {
    matches!(instance.find_actor(actor_id), Some(NetworkedActor::Npc { spawn, .. })
        if spawn.common.owner_id.is_valid()
            && spawn.common.object_kind == ObjectKind::BattleNpc(BattleNpcSubKind::Chocobo))
}

/// The player left their instance (a zone change, logging out): their companion's actor goes with them, its time kept.
pub(super) fn owner_left(data: &mut WorldServer, network: &mut NetworkState, owner: ObjectId) {
    let Some(companion) = data.companions.get_mut(&owner) else {
        return;
    };
    let leaving = companion.leaving.take().map(|(actor_id, _)| actor_id);
    let actor = companion.actor.take();
    companion.appearing = None;
    if leaving.is_some() {
        companion.until = None;
        companion.paused = None;
    }
    for actor_id in [leaving, actor].into_iter().flatten() {
        remove_actor(data, network, actor_id);
    }
}

/// Process companion messages; returns true if the message was only for this.
pub(super) fn handle_companion_messages(
    data: Arc<Mutex<WorldServer>>,
    network: Arc<Mutex<NetworkState>>,
    game_data: Arc<Mutex<GameData>>,
    msg: &ToServer,
) -> bool {
    match msg {
        ToServer::SetCompanion(owner, profile) => {
            let mut data = data.lock();
            let companion = data.companions.entry(*owner).or_default();
            let barding_changed = companion.profile.models != profile.models
                || companion.profile.color != profile.color;
            let stance_changed = companion.profile.stance() != profile.stance();
            let points_changed = companion.profile.skill_points != profile.skill_points;
            companion.profile = profile.clone();
            let companion = companion.clone();
            let Some(actor_id) = companion.actor.filter(|_| companion.leaving.is_none()) else {
                return true;
            };
            let mut network = network.lock();
            if stance_changed {
                send_self(
                    &mut network,
                    *owner,
                    ActorControlCategory::UnkCompanionRelated {
                        unk1: actor_id,
                        unk2: profile.stance() as u32,
                    },
                );
                // Told to follow or to heal, it leaves the fight.
                if matches!(profile.stance(), COMMAND_FOLLOW | COMMAND_HEALER)
                    && let Some(instance) = data.find_actor_instance_mut(actor_id)
                    && let Some(NetworkedActor::Npc { enmity, .. }) =
                        instance.find_actor_mut(actor_id)
                {
                    enmity.clear();
                }
            }
            if points_changed {
                send_update(&mut network, *owner, &companion, false);
            }
            if (barding_changed || points_changed)
                && let Some(instance) = data.find_actor_instance_mut(actor_id)
            {
                let models = profile.equipment_models();
                let mut level = 0;
                if let Some(NetworkedActor::Npc { spawn, .. }) = instance.find_actor_mut(actor_id) {
                    spawn.common.models = models;
                    level = spawn.common.level;
                }
                if barding_changed {
                    // Retail's Equip for it: no weapons, its level, the models.
                    let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::Equip(Equip {
                        level,
                        models,
                        ..Default::default()
                    }));
                    network.send_in_range_inclusive_instance(
                        actor_id,
                        instance,
                        FromServer::PacketSegment(ipc, actor_id),
                        DestinationNetwork::ZoneClients,
                    );
                }
                if points_changed
                    && let Some(spawn) = instance.find_actor(actor_id).map(|actor| SpawnNpc {
                        common: actor.get_common_spawn().clone(),
                        ..Default::default()
                    })
                {
                    // A skill learnt: retail sent the companion's StatusEffectList.
                    send_status_list(&mut network, *owner, actor_id, &spawn);
                }
            }
            true
        }
        ToServer::WithdrawCompanion(owner) => {
            let mut data = data.lock();
            withdraw(&mut data, &mut network.lock(), *owner);
            true
        }
        ToServer::ZoneIn(_, owner, _) => {
            let mut data = data.lock();
            let Some(companion) = data.companions.get(owner) else {
                return false;
            };
            if companion.is_out() && companion.actor.is_none() && companion.paused.is_none() {
                spawn(&mut data, &network, &game_data, *owner, false);
            } else if !companion.is_out() {
                // Retail at zone-in with no companion out: 263 with 0, CompanionUpdate, SetPetParameters.
                let companion = companion.clone();
                let mut network = network.lock();
                send_self(
                    &mut network,
                    *owner,
                    ActorControlCategory::SetPetEntityId { unk1: 0 },
                );
                send_update(&mut network, *owner, &companion, true);
                send_self(
                    &mut network,
                    *owner,
                    ActorControlCategory::SetPetParameters {
                        pet_id: 0,
                        unk2: 0,
                        unk3: 0,
                        unk4: 7,
                    },
                );
            }
            false
        }
        ToServer::Dismounted(owner, _) => {
            let mut data = data.lock();
            if data
                .companions
                .get(owner)
                .is_some_and(|companion| companion.paused.is_some_and(|left| !left.is_zero()))
            {
                spawn(&mut data, &network, &game_data, *owner, true);
            }
            false
        }
        ToServer::Disconnected(_, owner) => {
            let mut data = data.lock();
            owner_left(&mut data, &mut network.lock(), *owner);
            data.companions.remove(owner);
            false
        }
        _ => false,
    }
}

/// Every server tick: companions whose time is up leave, the withdrawn are taken away, the newly out show themselves,
/// and in a fighting stance a companion goes after what its owner is fighting.
pub(super) fn update(data: &mut WorldServer, network: &Arc<Mutex<NetworkState>>) {
    let now = Instant::now();
    let owners: Vec<ObjectId> = data.companions.keys().copied().collect();
    for owner in owners {
        let Some(companion) = data.companions.get(&owner).cloned() else {
            continue;
        };

        if let Some((actor_id, at)) = companion.leaving
            && now >= at
        {
            let mut network = network.lock();
            if let Some(companion) = data.companions.get_mut(&owner) {
                companion.leaving = None;
                if companion.actor == Some(actor_id) {
                    companion.actor = None;
                }
            }
            send_self(
                &mut network,
                owner,
                ActorControlCategory::SetupPet {
                    owner_id: owner,
                    pet_id: 0,
                    pet_actor_id: ObjectId(0xE000_0000),
                    unk2: 1,
                    unk3: 0,
                },
            );
            if let Some(companion) = data.companions.get(&owner).cloned() {
                send_update(&mut network, owner, &companion, true);
            }
            remove_actor(data, &mut network, actor_id);
            continue;
        }

        if let Some((actor_id, at)) = companion.appearing
            && now >= at
        {
            if let Some(companion) = data.companions.get_mut(&owner) {
                companion.appearing = None;
            }
            if let Some(instance) = data.find_actor_instance(actor_id) {
                let mut network = network.lock();
                network.send_ac_in_range_instance(
                    instance,
                    actor_id,
                    unknown(AC_COMPANION_APPEARED, 1, AC_COMPANION_APPEARED_ARG),
                );
                network.send_ac_in_range_instance(instance, actor_id, unknown(AC_NAME_SHOWN, 1, 0));
            }
        }

        let Some(actor_id) = companion.actor.filter(|_| companion.leaving.is_none()) else {
            continue;
        };
        let time_up = companion.until.is_some_and(|until| now >= until);
        let fallen = data
            .find_actor_instance(actor_id)
            .and_then(|instance| instance.find_actor(actor_id))
            .is_none_or(|actor| actor.get_common_spawn().health_points == 0);
        if time_up || fallen {
            withdraw(data, &mut network.lock(), owner);
            continue;
        }

        if matches!(
            companion.profile.stance(),
            COMMAND_FREE | COMMAND_DEFENDER | COMMAND_ATTACKER
        ) && let Some(instance) = data.find_actor_instance_mut(actor_id)
        {
            assist(instance, owner, actor_id);
        }
    }
}

/// A companion in a fighting stance goes after what its owner fights: their target, if it's an enemy that has them
/// on its list, else any enemy that has them on its list.
fn assist(instance: &mut Instance, owner: ObjectId, actor_id: ObjectId) {
    let owner_target = instance
        .find_actor(owner)
        .map(|actor| actor.get_common_spawn().target_id.object_id);
    let fighting_owner = |id: &ObjectId, actor: &NetworkedActor| {
        *id != actor_id
            && matches!(actor, NetworkedActor::Npc { enmity, spawn, .. }
                if enmity.contains_key(&owner) && spawn.common.health_points > 0)
    };
    let foe = owner_target
        .filter(|target| {
            instance
                .find_actor(*target)
                .is_some_and(|actor| fighting_owner(target, actor))
        })
        .or_else(|| {
            instance
                .actors
                .iter()
                .find(|(id, actor)| fighting_owner(id, actor))
                .map(|(id, _)| *id)
        });
    let Some(foe) = foe else {
        return;
    };
    if let Some(NetworkedActor::Npc { enmity, .. }) = instance.find_actor_mut(actor_id)
        && !enmity.contains_key(&foe)
        && enmity.is_empty()
    {
        enmity.insert(foe, 1);
    }
}
