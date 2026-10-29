//! The chocobo companion: summoned with Gysahl Greens, following and fighting beside its owner, told what to do
//! (stances), wearing barding, ridden. See `docs/src/fork/chocobo.md`.
//!
//! What the client knows about it (name, rank, EXP, skill points, levels, barding) is kept with the character and
//! sent at login (PlayerSetup); the zone connection sends it here too (`ToServer::SetCompanion`), so the server can
//! summon it in whichever instance its owner is. Every packet and number here is from Bob's Chronofoil capture of
//! 2026-10-24 (e8158807, Middle La Noscea: summoning, riding, dismounting, barding, stances, skills, withdrawing),
//! unless it says otherwise. Its fighting, skill lines, ranks and EXP are from his capture of 2026-10-24 (8d85b18d,
//! Lakeland: a level 88 companion with Attacker at 6 fighting beside him in Attacker and Free Stance), the BuddySkill,
//! BuddyRank, Trait and Action sheets, and the Console Games Wiki's Chocobo Companion page for what no capture shows.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use kawari::{
    common::{LegacyEquipmentModelId, ObjectId, ObjectTypeId, ObjectTypeKind, RecordedAmount},
    config::get_config,
    ipc::zone::{
        ActionRequest, ActionType, ActorControlCategory, BattleNpcSubKind, CharacterDataFlag,
        CommonSpawn, DisplayFlag, Equip, ObjectKind, ServerZoneIpcData, ServerZoneIpcSegment,
        SpawnNpc, StatusEffect, StatusEffectList,
    },
};
use parking_lot::Mutex;

use crate::{
    ClientId, FromServer, GameData, ToServer,
    server::{
        WorldServer,
        actor::{NetworkedActor, NpcState, create_npc_common_spawn},
        instance::{Instance, QueuedTaskData},
        network::{DestinationNetwork, NetworkState},
        npc_behavior::npc_cast_bar_time,
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
/// Greens used while it's out add to its time, up to an hour (Bob, 2026-10-24: "it can stack up to an hour"; his
/// capture: 140,971 cs left, one more Greens, 318,680.)
const MAX_SUMMON_TIME: Duration = Duration::from_secs(60 * 60);
/// Withdrawn, it leaves; retail took it away (SetupPet with no pet, then DeleteActor) 2 s later.
const WITHDRAW_TIME: Duration = Duration::from_millis(2000);
/// Retail sent ActorControl 36 and 54 for a companion 0.7-0.85 s after its spawn.
const APPEAR_TIME: Duration = Duration::from_millis(800);

/// Its HP by its level's ParamGrow HpModifier, before its skill lines' HP traits: retail's companions had 36,330 at
/// level 90 (HpModifier 3,661) and 35,038 at level 88 (3,553), both with Attacker's +5% HP; it isn't a share of the
/// owner's (Bob's level 88 White Mage had 35,118, his level 90 character 69,756.) 9.42 fits both within 0.3%.
const HP_PER_HP_MODIFIER: f32 = 9.42;
/// What one potency of its attacks does, per HpModifier of its level, before Strength traits: retail's level 88
/// companion (HpModifier 3,553, Attacker at 6: +5% Strength) hit Lakeland's monsters for 1,743 with its auto-attack
/// (49 hits, taken as potency 100), 2,664 with Choco Slash (150, 6 hits) and 2,251 with Choco Beak (130, 12 hits):
/// 17.5 a potency. How it grows with level isn't recorded; HpModifier is a stand-in.
const DAMAGE_PER_POTENCY_PER_HP_MODIFIER: f32 = 17.5 / 1.05 / 3553.0;
/// Its auto-attack, as a potency for the scale above.
const AUTO_ATTACK_POTENCY: f32 = 100.0;
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

/// The first of its skills comes this long after it starts fighting, then one every `SKILL_EVERY` (retail: 5.1-8.1 s
/// into five fights, then 7.9-8.5 s apart, 9.5 once.)
const FIRST_SKILL_AFTER: (f32, f32) = (5.0, 8.0);
const SKILL_EVERY: (f32, f32) = (7.9, 8.5);
/// How often it looks after its owner's and its own health in Healer and Free Stance (the Wiki: Choco Cure below 75%
/// in Healer Stance, 50% in the others; Choco Regen when someone was hurt; Choco Surge whenever it can.)
const HEAL_EVERY: f32 = 3.0;
const HEALER_CURE_BELOW: f32 = 0.75;
const FREE_CURE_BELOW: f32 = 0.5;
/// Choco Regen and Choco Surge last this long (their Action descriptions.)
const REGEN_TIME: Duration = Duration::from_secs(18);
const SURGE_TIME: Duration = Duration::from_secs(30);
/// Its healing spells' cast (their Action rows' Cast100ms: 15.)
const HEAL_CAST: f32 = 1.5;

/// The skill lines (BuddySkill: Defender, Attacker, Healer) and the active skills in each, by the level they come at;
/// the levels between are traits (`TRAITS`.) Action rows 817-828.
pub const LINE_DEFENDER: usize = 0;
pub const LINE_ATTACKER: usize = 1;
pub const LINE_HEALER: usize = 2;
const SKILLS: [[(u8, u32); 4]; 3] = [
    // Choco Drop, Choco Kick, Choco Guard, Choco Strike.
    [(1, 820), (4, 819), (7, 818), (10, 817)],
    // Choco Slash, Choco Beak, Choco Rush, Choco Blast.
    [(1, 824), (4, 823), (7, 822), (10, 821)],
    // Choco Regen, Choco Cure, Choco Surge, Choco Medica.
    [(1, 828), (4, 827), (7, 826), (10, 825)],
];
const CHOCO_REGEN: u32 = 828;
const CHOCO_CURE: u32 = 827;
const CHOCO_SURGE: u32 = 826;
const CHOCO_MEDICA: u32 = 825;
/// Each skill's potency (the Action descriptions), for its damage or healing.
const POTENCIES: [(u32, f32); 11] = [
    (817, 150.0),
    (819, 130.0),
    (820, 150.0),
    (821, 170.0),
    (822, 130.0),
    (823, 130.0),
    (824, 150.0),
    (825, 150.0),
    (827, 300.0),
    (871, AUTO_ATTACK_POTENCY),
    (828, 25.0),
];

/// What a skill line's traits give, by line and the level they come at (BuddySkill's passive rows, Trait 71-88.)
#[derive(Debug, Clone, Copy, PartialEq)]
enum Trait {
    Strength(u8),
    Hp(u8),
    Mind(u8),
    Accuracy(u8),
    Critical(u8),
    AttackSpeed(u8),
}
const TRAITS: [[(u8, Trait); 6]; 3] = [
    [
        (2, Trait::Strength(5)),
        (3, Trait::Hp(10)),
        (5, Trait::Accuracy(5)),
        (6, Trait::Critical(15)),
        (8, Trait::Strength(5)),
        (9, Trait::Hp(10)),
    ],
    [
        (2, Trait::Strength(5)),
        (3, Trait::Hp(5)),
        (5, Trait::Accuracy(5)),
        (6, Trait::Critical(15)),
        (8, Trait::Strength(10)),
        (9, Trait::AttackSpeed(10)),
    ],
    [
        (2, Trait::Mind(10)),
        (3, Trait::Hp(5)),
        (5, Trait::Accuracy(5)),
        (6, Trait::Critical(15)),
        (8, Trait::Mind(15)),
        (9, Trait::Hp(5)),
    ],
];

/// What its traits add up to, in percent.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TraitBonus {
    pub strength: u32,
    pub hp: u32,
    pub mind: u32,
}

/// Its traits for these skill line levels (Defender, Attacker, Healer.)
pub fn trait_bonus(levels: [u8; 3]) -> TraitBonus {
    let mut bonus = TraitBonus::default();
    for (line, traits) in TRAITS.iter().enumerate() {
        for (level, gained) in traits {
            if levels[line] < *level {
                continue;
            }
            match gained {
                Trait::Strength(percent) => bonus.strength += *percent as u32,
                Trait::Hp(percent) => bonus.hp += *percent as u32,
                Trait::Mind(percent) => bonus.mind += *percent as u32,
                Trait::Accuracy(_) | Trait::Critical(_) | Trait::AttackSpeed(_) => {}
            }
        }
    }
    bonus
}

/// Its HP at a level whose ParamGrow HpModifier is `hp_modifier`, with these skill lines.
pub fn companion_hp(hp_modifier: u32, levels: [u8; 3]) -> u32 {
    let traits = trait_bonus(levels);
    (hp_modifier as f32 * HP_PER_HP_MODIFIER * (100 + traits.hp) as f32 / 100.0).round() as u32
}

/// What its actions do at a level whose HpModifier is `hp_modifier`: each attack's damage, each heal's amount (its
/// timeline's `recorded_amounts`, which the action pipeline scales its hits and heals to.)
fn recorded_amounts(
    hp_modifier: u32,
    levels: [u8; 3],
) -> std::collections::BTreeMap<u32, RecordedAmount> {
    let traits = trait_bonus(levels);
    let per_potency = hp_modifier as f32 * DAMAGE_PER_POTENCY_PER_HP_MODIFIER;
    POTENCIES
        .iter()
        .filter(|(action, _)| *action != CHOCO_REGEN)
        .map(|(action, potency)| {
            let healing = matches!(*action, CHOCO_CURE | CHOCO_MEDICA);
            // Healing isn't recorded: the same per potency, with Mind's traits instead of Strength's.
            let bonus = if healing {
                traits.mind
            } else {
                traits.strength
            };
            let amount = per_potency * potency * (100 + bonus) as f32 / 100.0;
            (
                *action,
                RecordedAmount {
                    players: healing.then_some(amount),
                    npcs: Some(amount),
                    ..Default::default()
                },
            )
        })
        .collect()
}

/// The active skills a line has learnt at `level`.
fn learnt(line: usize, level: u8) -> impl Iterator<Item = u32> {
    SKILLS[line]
        .iter()
        .filter(move |(at, _)| level >= *at)
        .map(|(_, action)| *action)
}

/// The attacks it picks from in a stance: Defender's or Attacker's learnt skills; in Free Stance each attacking line's
/// latest (retail's Free Stance companion, Attacker at 6, used only Choco Beak, four times.)
fn attacks_for(stance: u8, levels: [u8; 3]) -> Vec<u32> {
    let attacking = |line: usize| learnt(line, levels[line]).filter(|action| *action != 818);
    match stance {
        COMMAND_DEFENDER => attacking(LINE_DEFENDER).collect(),
        COMMAND_ATTACKER => attacking(LINE_ATTACKER).collect(),
        COMMAND_FREE => [LINE_DEFENDER, LINE_ATTACKER]
            .into_iter()
            .filter_map(|line| attacking(line).last())
            .collect(),
        _ => Vec::new(),
    }
}

/// Its auto-attack in a stance: none in Healer Stance (the Wiki), else Action 871.
fn auto_attack_for(stance: u8) -> u32 {
    if stance == COMMAND_HEALER {
        0
    } else {
        AUTO_ATTACK
    }
}

/// BuddyRank's ExpRequired: the EXP from each rank to the next (rank 0's 100 to rank 19's 3,620,000; 20 is the last.)
const RANK_EXP: [u32; 21] = [
    100, 4000, 38000, 82000, 158000, 294000, 490000, 700000, 940000, 1200000, 1458000, 1714000,
    1967000, 2217000, 2463000, 2705000, 2942000, 3174000, 3400000, 3620000, 0,
];
pub const MAX_RANK: u8 = 20;
/// From this rank on, it ranks up only by eating a Thavnairian Onion once its EXP bar is full (Bob; the Wiki.)
pub const ONION_RANK: u8 = 10;

/// The skill points a rank brings (the Wiki: the rank itself up to 10, 55 in all; 10 each to 16; then 11 to 14, 165
/// at 20, enough for every skill of the three lines. Retail's rank 6 companion had used 21 = 1 + ... + 6.)
pub fn skill_points_for_rank(rank: u8) -> u8 {
    match rank {
        0 => 0,
        1..=10 => rank,
        11..=16 => 10,
        _ => rank - 6,
    }
}

/// EXP gained: its rank and EXP after it, and the skill points the new ranks bring. Below rank 10 it ranks up as its
/// bar fills; from 10 on the bar stops when full, for a Thavnairian Onion.
pub fn gain_exp(rank: u8, exp: u32, gained: u32) -> (u8, u32, u8) {
    let (mut rank, mut exp, mut points) = (rank.min(MAX_RANK), exp.saturating_add(gained), 0u8);
    while rank < ONION_RANK && exp >= RANK_EXP[rank as usize] {
        exp -= RANK_EXP[rank as usize];
        rank += 1;
        points = points.saturating_add(skill_points_for_rank(rank));
    }
    if rank >= ONION_RANK {
        exp = exp.min(RANK_EXP[rank as usize]);
    }
    (rank, exp, points)
}

/// A Thavnairian Onion eaten: the next rank (from 10 to 20) and its points, if its EXP bar is full.
pub fn eat_onion(rank: u8, exp: u32) -> Option<(u8, u32, u8)> {
    if !(ONION_RANK..MAX_RANK).contains(&rank) || exp < RANK_EXP[rank as usize] {
        return None;
    }
    Some((rank + 1, 0, skill_points_for_rank(rank + 1)))
}

/// Unnamed ActorControls retail sent (their numbers):
/// 36 `[1, 18009]` and 54 `[1]` on the companion just after it was summoned (54 is Sapphire's ToggleNameHidden);
/// coming back with its owner at a zone-in, 267 `[0]` and 54 `[1]` instead (capture 8d85b18d);
const AC_COMPANION_APPEARED: u32 = 36;
const AC_COMPANION_APPEARED_ARG: u32 = 18009;
const AC_COMPANION_CAME_ALONG: u32 = 267;
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
    /// Its skill lines' levels: Defender, Attacker, Healer.
    pub levels: [u8; 3],
}

impl CompanionProfile {
    /// Its colour as the game shows it: Desert Yellow (36) when it was never changed.
    pub fn stain(&self) -> u8 {
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
    /// Just spawned: ActorControl 36 (or 267 coming along) and 54 go out at this time.
    appearing: Option<(ObjectId, Instant, bool)>,
    /// Fighting: its next skill's time.
    next_skill: Option<Instant>,
    /// When it next looks after health (Healer and Free Stance.)
    next_heal_check: Option<Instant>,
    /// Its Choco Regen on its owner and on itself, and its Choco Surge, until then.
    regen_owner_until: Option<Instant>,
    regen_self_until: Option<Instant>,
    surge_until: Option<Instant>,
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

/// The ParamGrow HpModifier of a level (its HP and damage scale with it.)
fn hp_modifier(game_data: &mut GameData, level: u8) -> u32 {
    game_data
        .get_param_grow(level.max(1) as u32)
        .map(|row| row.HpModifier as u32)
        .unwrap_or(1)
}

/// The companion as retail spawned it: BNpcBase 952 with its name, owned by the player, of sub-kind Chocobo, at its
/// owner's level with the HP of that level and its traits, its barding and colour.
fn companion_spawn(
    game_data: &mut GameData,
    owner_id: ObjectId,
    owner: &CommonSpawn,
    profile: &CompanionProfile,
) -> SpawnNpc {
    let hp = companion_hp(hp_modifier(game_data, owner.level), profile.levels).max(1);
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
    came_along: bool,
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
    let (spawn, amounts) = {
        let mut game_data = game_data.lock();
        let modifier = hp_modifier(&mut game_data, owner_spawn.level);
        (
            companion_spawn(&mut game_data, owner, &owner_spawn, &companion.profile),
            recorded_amounts(modifier, companion.profile.levels),
        )
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
        timeline.autoattack_action_id = auto_attack_for(companion.profile.stance());
        timeline.attack_range = ATTACK_RANGE;
        timeline.rotation = None;
        timeline.recorded_amounts = amounts;
        *state = NpcState::Follow;
    }

    let Some(companion) = data.companions.get_mut(&owner) else {
        return;
    };
    companion.actor = Some(actor_id);
    companion.appearing = Some((actor_id, Instant::now() + APPEAR_TIME, came_along));
    companion.next_skill = None;
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
    spawn(&mut data, network, game_data, owner, false, false);
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
            let levels_changed = companion.profile.levels != profile.levels;
            let exp_changed =
                companion.profile.exp != profile.exp || companion.profile.rank != profile.rank;
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
            if points_changed || exp_changed {
                send_update(&mut network, *owner, &companion, false);
            }
            // Its auto-attack (none in Healer Stance), and what its new skills and traits make of its HP and hits.
            if (stance_changed || levels_changed)
                && let Some(instance) = data.find_actor_instance_mut(actor_id)
                && let Some(NetworkedActor::Npc {
                    timeline, spawn, ..
                }) = instance.find_actor_mut(actor_id)
            {
                timeline.autoattack_action_id = auto_attack_for(profile.stance());
                if levels_changed {
                    let modifier = hp_modifier(&mut game_data.lock(), spawn.common.level);
                    timeline.recorded_amounts = recorded_amounts(modifier, profile.levels);
                    let hp = companion_hp(modifier, profile.levels).max(1);
                    spawn.common.health_points = (spawn.common.health_points as u64 * hp as u64
                        / spawn.common.max_health_points.max(1) as u64)
                        as u32;
                    spawn.common.max_health_points = hp;
                }
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
        ToServer::CompanionKill(owner, exp) => {
            // Something its owner fought was defeated: with the companion out, it gets the EXP (a stand-in: as much as
            // its owner's base EXP for it, from monsters around their level), and the owner sees its EXP again, as
            // retail sent CompanionUpdate (no time) ~2 s after each kill, even with nothing gained.
            let data = data.lock();
            let Some(companion) = data.companions.get(owner) else {
                return true;
            };
            if companion.actor.is_none() || companion.leaving.is_some() {
                return true;
            }
            let mut network = network.lock();
            if *exp > 0 {
                network.send_to_by_actor_id(
                    *owner,
                    FromServer::CompanionExp(*exp),
                    DestinationNetwork::ZoneClients,
                );
            } else {
                send_update(&mut network, *owner, companion, false);
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
                spawn(&mut data, &network, &game_data, *owner, false, true);
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
                spawn(&mut data, &network, &game_data, *owner, true, false);
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

        if let Some((actor_id, at, came_along)) = companion.appearing
            && now >= at
        {
            if let Some(companion) = data.companions.get_mut(&owner) {
                companion.appearing = None;
            }
            if let Some(instance) = data.find_actor_instance(actor_id) {
                let mut network = network.lock();
                let appeared = if came_along {
                    unknown(AC_COMPANION_CAME_ALONG, 0, 0)
                } else {
                    unknown(AC_COMPANION_APPEARED, 1, AC_COMPANION_APPEARED_ARG)
                };
                network.send_ac_in_range_instance(instance, actor_id, appeared);
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
        fight(data, network, owner, actor_id, &companion, now);
        look_after(data, network, owner, actor_id, &companion, now);
    }
}

fn seconds_between(range: (f32, f32)) -> Duration {
    Duration::from_secs_f32(range.0 + fastrand::f32() * (range.1 - range.0))
}

/// The companion uses one of its actions on `target`, through the action pipeline as other NPCs' do; with a cast, its
/// ActorCast first.
fn cast(
    instance: &mut Instance,
    network: &mut NetworkState,
    actor_id: ObjectId,
    action: u32,
    target: ObjectId,
    cast_time: f32,
) {
    let Some((rotation, position)) = instance
        .find_actor(actor_id)
        .map(|actor| (actor.rotation(), actor.position()))
    else {
        return;
    };
    let request = ActionRequest {
        action_id: action,
        action_type: ActionType::Action,
        rotation1: rotation,
        target: ObjectTypeId {
            object_id: target,
            object_type: ObjectTypeKind::None,
        },
        ..Default::default()
    };
    if cast_time > 0.0 {
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorCast {
            spell_id: action as u16,
            action_type: ActionType::Action,
            omen_delay: 0,
            action_id: action,
            cast_time: npc_cast_bar_time(cast_time),
            target,
            rotation,
            interruptible: false,
            ballista_entity_id: ObjectId::default(),
            position,
        });
        network.send_in_range_instance(
            actor_id,
            instance,
            FromServer::PacketSegment(ipc, actor_id),
            DestinationNetwork::ZoneClients,
        );
    }
    instance.insert_task(
        ClientId::default(),
        actor_id,
        Duration::from_secs_f32(cast_time),
        QueuedTaskData::CastAction {
            request,
            interruptible: false,
        },
    );
}

/// In a fight, its skills: the first 5-8 s in, then one every ~8 s, picked at random from what its stance uses (retail:
/// Choco Slash and Choco Beak in Attacker Stance, in no fixed order.)
fn fight(
    data: &mut WorldServer,
    network: &Arc<Mutex<NetworkState>>,
    owner: ObjectId,
    actor_id: ObjectId,
    companion: &Companion,
    now: Instant,
) {
    let attacks = attacks_for(companion.profile.stance(), companion.profile.levels);
    let Some(instance) = data.find_actor_instance_mut(actor_id) else {
        return;
    };
    let target = match instance.find_actor(actor_id) {
        Some(NetworkedActor::Npc { spawn, enmity, .. })
            if !enmity.is_empty() && spawn.common.target_id.object_id.is_valid() =>
        {
            Some(spawn.common.target_id.object_id)
        }
        _ => None,
    }
    .filter(|target| {
        instance
            .find_actor(*target)
            .is_some_and(|actor| actor.get_common_spawn().health_points > 0)
    });
    let next_skill = match (target, companion.next_skill) {
        (None, _) => None,
        _ if attacks.is_empty() => None,
        (Some(_), None) => Some(now + seconds_between(FIRST_SKILL_AFTER)),
        (Some(target), Some(at)) if now >= at => {
            let action = attacks[fastrand::usize(..attacks.len())];
            cast(instance, &mut network.lock(), actor_id, action, target, 0.0);
            Some(now + seconds_between(SKILL_EVERY))
        }
        (Some(_), at) => at,
    };
    if let Some(companion) = data.companions.get_mut(&owner) {
        companion.next_skill = next_skill;
    }
}

/// In Healer and Free Stance with Healer skills learnt, it looks after its owner and itself every few seconds: Choco
/// Medica when both are low, Choco Cure on whoever is (below 75% in Healer Stance, 50% in Free), Choco Regen on
/// someone hurt without it, Choco Surge in Healer Stance whenever it's worn off and someone is hurt (the Wiki's
/// account; no retail Healer Stance is recorded.)
fn look_after(
    data: &mut WorldServer,
    network: &Arc<Mutex<NetworkState>>,
    owner: ObjectId,
    actor_id: ObjectId,
    companion: &Companion,
    now: Instant,
) {
    let stance = companion.profile.stance();
    let level = companion.profile.levels[LINE_HEALER];
    if level == 0 || !matches!(stance, COMMAND_HEALER | COMMAND_FREE) {
        return;
    }
    if companion.next_heal_check.is_some_and(|at| now < at) {
        return;
    }
    let Some(instance) = data.find_actor_instance_mut(actor_id) else {
        return;
    };
    let health = |id: ObjectId| {
        instance.find_actor(id).and_then(|actor| {
            let spawn = actor.get_common_spawn();
            (spawn.health_points > 0)
                .then(|| spawn.health_points as f32 / spawn.max_health_points.max(1) as f32)
        })
    };
    let (Some(owner_health), Some(own_health)) = (health(owner), health(actor_id)) else {
        return;
    };
    let has = |action: u32| learnt(LINE_HEALER, level).any(|known| known == action);
    let below = if stance == COMMAND_HEALER {
        HEALER_CURE_BELOW
    } else {
        FREE_CURE_BELOW
    };
    let expired = |until: Option<Instant>| until.is_none_or(|until| now >= until);

    let mut regen_owner_until = companion.regen_owner_until;
    let mut regen_self_until = companion.regen_self_until;
    let mut surge_until = companion.surge_until;
    let choice = if has(CHOCO_MEDICA) && owner_health < below && own_health < below {
        Some((CHOCO_MEDICA, actor_id))
    } else if has(CHOCO_CURE) && owner_health.min(own_health) < below {
        Some((
            CHOCO_CURE,
            if owner_health <= own_health {
                owner
            } else {
                actor_id
            },
        ))
    } else if stance == COMMAND_HEALER
        && has(CHOCO_SURGE)
        && expired(surge_until)
        && owner_health.min(own_health) < 1.0
    {
        surge_until = Some(now + SURGE_TIME);
        Some((CHOCO_SURGE, actor_id))
    } else if has(CHOCO_REGEN) && owner_health < 1.0 && expired(regen_owner_until) {
        regen_owner_until = Some(now + REGEN_TIME);
        Some((CHOCO_REGEN, owner))
    } else if has(CHOCO_REGEN) && own_health < 1.0 && expired(regen_self_until) {
        regen_self_until = Some(now + REGEN_TIME);
        Some((CHOCO_REGEN, actor_id))
    } else {
        None
    };
    if let Some((action, target)) = choice {
        cast(
            instance,
            &mut network.lock(),
            actor_id,
            action,
            target,
            HEAL_CAST,
        );
    }
    if let Some(companion) = data.companions.get_mut(&owner) {
        companion.next_heal_check = Some(now + Duration::from_secs_f32(HEAL_EVERY.max(HEAL_CAST)));
        companion.regen_owner_until = regen_owner_until;
        companion.regen_self_until = regen_self_until;
        companion.surge_until = surge_until;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hp_matches_retail() {
        // Bob's companions with Attacker at 4 and 6 (+5% HP): 36,330 at level 90, 35,038 at 88.
        let hp_90 = companion_hp(3661, [0, 6, 0]) as f32;
        let hp_88 = companion_hp(3553, [0, 6, 0]) as f32;
        assert!((hp_90 / 36_330.0 - 1.0).abs() < 0.005, "{hp_90}");
        assert!((hp_88 / 35_038.0 - 1.0).abs() < 0.005, "{hp_88}");
    }

    #[test]
    fn damage_matches_retail() {
        // Level 88 (HpModifier 3,553), Attacker at 6: 1,743 auto-attack, 2,664 Choco Slash, 2,251 Choco Beak.
        let amounts = recorded_amounts(3553, [0, 6, 0]);
        let npcs = |action| amounts[&action].npcs.unwrap();
        assert!((npcs(871) - 1750.0).abs() < 20.0);
        assert!((npcs(824) - 2625.0).abs() < 50.0);
        assert!((npcs(823) - 2275.0).abs() < 50.0);
    }

    #[test]
    fn traits() {
        assert_eq!(
            trait_bonus([0, 6, 0]),
            TraitBonus {
                strength: 5,
                hp: 5,
                mind: 0
            }
        );
        assert_eq!(trait_bonus([10, 10, 10]).hp, 20 + 5 + 10);
    }

    #[test]
    fn stances() {
        assert_eq!(attacks_for(COMMAND_ATTACKER, [0, 6, 0]), vec![824, 823]);
        // Free Stance: each attacking line's latest (retail used only Choco Beak.)
        assert_eq!(attacks_for(COMMAND_FREE, [0, 6, 0]), vec![823]);
        // Choco Guard isn't an attack.
        assert_eq!(attacks_for(COMMAND_DEFENDER, [7, 0, 0]), vec![820, 819]);
        assert!(attacks_for(COMMAND_HEALER, [10, 10, 10]).is_empty());
    }

    #[test]
    fn ranks() {
        // Retail's rank 6 companion had used 21 points (Attacker 1-6.)
        assert_eq!((1..=6).map(skill_points_for_rank).sum::<u8>(), 21);
        assert_eq!(
            (1..=10)
                .map(skill_points_for_rank)
                .map(u32::from)
                .sum::<u32>(),
            55
        );
        assert_eq!(
            (1..=20)
                .map(skill_points_for_rank)
                .map(u32::from)
                .sum::<u32>(),
            165
        );
        // Bob's: rank 6 with 487,338 of 490,000.
        assert_eq!(gain_exp(6, 487_338, 3000), (7, 338, 7));
        assert_eq!(gain_exp(0, 0, 4100), (2, 0, 3));
        // From rank 10 the bar stops full, and only an onion moves it on.
        assert_eq!(gain_exp(10, 1_457_000, 5000), (10, 1_458_000, 0));
        assert_eq!(eat_onion(10, 1_457_999), None);
        assert_eq!(eat_onion(10, 1_458_000), Some((11, 0, 10)));
        assert_eq!(eat_onion(9, 1_200_000), None);
        assert_eq!(eat_onion(20, 0), None);
    }
}
