//! What statuses, traits, critical hits and variance do to a player's (or an enemy's) damage and healing. See
//! docs/src/fork/combat.md, "Damage and healing modifiers".
//!
//! Retail numbers from Bob's Chronofoil captures (Conjurer 1–17, White Mage 80–88): player damage and heals vary about
//! ±5 %; damage is critical (flag 0x20 on the damage effect) about 4 % of the time at levels 1–20 and 12–13 % above,
//! for about 1.35–1.5×; heals have their own critical flag (0x20 in the heal's second byte); direct hits (0x40) are
//! almost never seen early on. The status percentages are the game's tooltips (Status sheet descriptions.)

use kawari::ipc::zone::{DamageKind, DamageType, TargetEffect, TargetEffectKind};

use crate::{StatusEffects, lua::EFFECT_ON_SOURCE};

/// ±5 % on every player hit and heal.
pub const PLAYER_VARIANCE: f32 = 0.05;
/// Direct hits multiply damage by 1.25.
const DIRECT_HIT_MULTIPLIER: f32 = 1.25;
/// The heal effect's critical flag, in its second byte.
const HEAL_CRITICAL: u8 = 0x20;

/// Statuses on whoever deals the damage: (status, physical multiplier, magic multiplier.)
const DAMAGE_DEALT: &[(u16, f32, f32)] = &[
    (76, 1.25, 1.25),   // Fight or Flight
    (125, 1.15, 1.15),  // Raging Strikes
    (1864, 1.10, 1.10), // Lance Charge
    (2720, 1.10, 1.10), // Power Surge
    (2677, 1.10, 1.10), // Surging Tempest
    // The other jobs' own and party buffs, as the job guide (7.x) has them (a party buff on whoever has it; the Astrologian's
    // Balance and Spear at their 6 %, Embolden at the party's 5 %):
    (1185, 1.05, 1.05), // Brotherhood
    (1181, 1.15, 1.15), // Riddle of Fire
    (2703, 1.05, 1.05), // Searing Light
    (1831, 1.20, 1.20), // No Mercy
    (1298, 1.13, 1.13), // Fugetsu
    (3668, 1.10, 1.10), // Hunter's Instinct
    (2599, 1.03, 1.03), // Arcane Circle
    (1821, 1.05, 1.05), // Standard Finish
    (1822, 1.05, 1.05), // Technical Finish
    (1297, 1.05, 1.05), // Embolden
    (1878, 1.06, 1.06), // Divination
    (3887, 1.06, 1.06), // The Balance
    (3889, 1.06, 1.06), // The Spear
    (3685, 1.05, 1.05), // Starry Muse
    // A quest battle's role statuses (`director::QUEST_BATTLE_ROLE_STATUSES`): tanks' and healers' damage +30 % (Bob's
    // Chronofoil captures: their hits in quest battles against the same characters' elsewhere, 1.26-1.31 times for
    // Gladiators and Paladins, 1.25-1.30 for Conjurers and White Mages; their heals unchanged.) The DPS one (1265)
    // left the Arcanist's damage as it was.
    (1264, 1.30, 1.30),
    (1041, 1.30, 1.30),
    // On an enemy, from players:
    (1195, 0.90, 0.95), // Feint
    (1203, 0.95, 0.90), // Addle
    (1193, 0.90, 0.90), // Reprisal
];

/// Statuses on whoever takes the damage: (status, multiplier.)
const DAMAGE_TAKEN: &[(u16, f32)] = &[
    (1191, 0.80), // Rampart
    (74, 0.70),   // Sentinel
    (1856, 0.85), // Sheltron (retail's status, Bob's Chronofoil captures)
    (89, 0.70),   // Vengeance
    (1873, 0.90), // Temperance (the party's status; 1872, the White Mage's, is healing +20 %)
    (3254, 1.10), // Trick Attack (on an enemy)
    (2586, 1.10), // Death's Design (on an enemy: the Reaper's 10 %, here everyone's)
    // The other jobs' mitigation (the job guide, 7.x; physical and magic halves averaged):
    (747, 0.70),  // Shadow Wall
    (746, 0.85),  // Dark Mind
    (2682, 0.90), // Oblation
    (1894, 0.92), // Dark Missionary
    (1832, 0.90), // Camouflage
    (1834, 0.70), // Nebula
    (3838, 0.60), // Great Nebula
    (1839, 0.92), // Heart of Light
    (1840, 0.85), // Heart of Stone
    (2683, 0.85), // Heart of Corundum
    (2684, 0.85), // Clarity of Corundum
    (1951, 0.85), // Tactician
    (1826, 0.85), // Shield Samba
    (849, 0.90),  // Collective Unconscious
    (3890, 0.90), // The Bole
    (3896, 0.90), // Sun Sign
    (2618, 0.90), // Kerachole
    (2619, 0.90), // Taurochole
    (3003, 0.90), // Holos
    (2711, 0.90), // Desperate Measures (Expedient)
    (1174, 0.90), // Intervention
    (1175, 0.85), // Passage of Arms
    (1362, 0.90), // Divine Veil (its barrier as a reduction)
];

/// Statuses that stop damage altogether.
const INVULNERABLE: &[u16] = &[
    82, // Hallowed Ground
    // The Gunbreaker's:
    1836, // Superbolide
];

/// Statuses that keep HP from dropping below 1.
const HP_FLOOR: &[u16] = &[
    409, // Holmgang
    810, // Living Dead (Walking Dead's heal back to full isn't done)
];

/// Statuses on whoever is healed: (status, multiplier.)
const HEALING_RECEIVED: &[(u16, f32)] = &[
    (102, 1.10), // Mantra
    (87, 1.20),  // Thrill of Battle
    // Rampart's +15 % healing received is a level 94 trait: retail's Rampart below that has none (58 effects in Bob's
    // captures), so it isn't here.

    // The other jobs' (the job guide, 7.x):
    (3888, 1.10), // The Arrow
    (2622, 1.20), // Krasis
    (2620, 1.10), // Physis II
    (2710, 1.10), // Protraction
];

/// Classes and jobs whose Tank Mastery takes 20 % off the damage they take from level 1 (GLA, MRD, PLD, WAR, DRK,
/// GNB.)
const TANKS: &[u8] = &[1, 3, 19, 21, 32, 37];
const TANK_MASTERY: f32 = 0.80;
/// How much of a hit a player's defense takes off: floor(15 × defense / level divisor) percent, physical hits by
/// Defense, magic and breath by Magic Defense, unique ones by neither. Fitted to Bob's Chronofoil captures (10,600 enemy
/// hits on his characters, 149 pairs of the same enemy's auto-attack on characters with different defense, 52 of its
/// other actions; levels 1-50): 15 with Tank Mastery's 0.8 is within the noise of the best fit (18.5, 0.86), and spells
/// fit Magic Defense better than Defense in 6 of 7 clear pairs.
const DEFENSE_COEFFICIENT: f32 = 15.0;
/// The mitigation (percent) the amounts Kawari scales enemy hits to were recorded with: Bob's casters and healers
/// (`Timeline::recorded_amounts`' `players`, `npc_damage_scale`), whose gear took a median 8 % of physical and 15 % of
/// magic damage at nearly every level from 1 to 50 in the same captures (3-4 % where they were under-geared). A
/// player's own mitigation is taken relative to it, so one geared like them takes what retail did; until 2026-11-12
/// gear made no difference, and tanks had a flat 0.83 for their armour (the Stone Vigil's 0.67 of the Arcanist's
/// damage on a Paladin, which this gives: about 400 defense against 160 at 42, 0.85 × Tank Mastery's 0.8.)
const RECORDED_PHYSICAL_MITIGATION: f32 = 8.0;
const RECORDED_MAGIC_MITIGATION: f32 = 15.0;

/// f(SPD) on damage and healing over time: × (1000 + floor(130 × (speed − base) / divisor)) / 1000, the community's
/// formula (Allagan Studies), as cast times already use the same 130 (`BaseParameters::get_adjusted_cast_time`). Bob's
/// captures don't separate it: below 50 his characters' speed stood at or within a few points of the base.
pub fn scale_periodic_by_speed(
    periodic: &mut [crate::lua::PeriodicSpec],
    speed: u32,
    base: u32,
    level_div: u32,
) {
    let factor = speed_factor(speed, base, level_div);
    if factor == 1000 {
        return;
    }
    for spec in periodic.iter_mut() {
        if matches!(
            spec.kind,
            crate::lua::PeriodicKind::Damage | crate::lua::PeriodicKind::Heal
        ) {
            spec.amount = (spec.amount as u64 * factor / 1000) as u32;
        }
    }
}

/// f(SPD) in thousandths.
fn speed_factor(speed: u32, base: u32, level_div: u32) -> u64 {
    let over = speed.saturating_sub(base);
    (SPEED_COEFFICIENT * over as f32 / level_div.max(1) as f32).floor() as u64 + 1000
}
const SPEED_COEFFICIENT: f32 = 130.0;

/// A defender's Defense and Magic Defense and their level's divisor (ParamGrow's LevelModifier.)
#[derive(Debug, Clone, Copy)]
pub struct Defences {
    pub defense: u32,
    pub magic_defense: u32,
    pub level_div: u32,
    /// Tenacity above the level's base: takes floor(100 × it / divisor) thousandths off every hit (the community's
    /// formula; the recordings' casters and healers had none.)
    pub tenacity_over: u32,
}

/// What a defender's defences leave of a hit of this type, against what the recordings' characters' left.
pub fn defence_multiplier(defences: Option<Defences>, damage_type: DamageType) -> f32 {
    let Some(defences) = defences else {
        return 1.0;
    };
    let (value, recorded) = match damage_type {
        DamageType::Unique | DamageType::LimitBreak => return 1.0,
        DamageType::Magic | DamageType::Breath => {
            (defences.magic_defense, RECORDED_MAGIC_MITIGATION)
        }
        _ => (defences.defense, RECORDED_PHYSICAL_MITIGATION),
    };
    let mitigation = (DEFENSE_COEFFICIENT * value as f32 / defences.level_div.max(1) as f32)
        .floor()
        .min(95.0);
    let tenacity = (crate::zone_connection::TENACITY_SCALE * defences.tenacity_over as f32
        / defences.level_div.max(1) as f32)
        .floor()
        / 1000.0;
    (100.0 - mitigation) / (100.0 - recorded) * (1.0 - tenacity)
}

fn is_physical(damage_type: DamageType) -> bool {
    !matches!(
        damage_type,
        DamageType::Magic | DamageType::Breath | DamageType::Unique | DamageType::LimitBreak
    )
}

/// How much more (or less) damage someone with these statuses deals.
pub fn damage_dealt_multiplier(statuses: &[u16], damage_type: DamageType) -> f32 {
    let physical = is_physical(damage_type);
    DAMAGE_DEALT
        .iter()
        .filter(|(id, _, _)| statuses.contains(id))
        .map(|(_, physical_multiplier, magic_multiplier)| {
            if physical {
                *physical_multiplier
            } else {
                *magic_multiplier
            }
        })
        .product()
}

/// How much of the damage someone with these statuses (and this class or job) takes.
pub fn damage_taken_multiplier(statuses: &[u16], classjob: u8, is_player: bool) -> f32 {
    let from_statuses: f32 = DAMAGE_TAKEN
        .iter()
        .filter(|(id, _)| statuses.contains(id))
        .map(|(_, multiplier)| *multiplier)
        .product();
    let tank = if is_player && TANKS.contains(&classjob) {
        TANK_MASTERY
    } else {
        1.0
    };
    from_statuses * tank
}

pub fn healing_received_multiplier(statuses: &[u16]) -> f32 {
    HEALING_RECEIVED
        .iter()
        .filter(|(id, _)| statuses.contains(id))
        .map(|(_, multiplier)| *multiplier)
        .product()
}

/// Stun, Sleep, Petrification and the Navel's Granite Gaol (292, on whomever Titan's Rock Throw hit until the gaol
/// round them breaks: retail's gaoled Duty Support members did nothing for its 11-22 s, Bob's Chronofoil capture
/// 650dbfd8): an NPC with one does nothing. Petrification (1, 9 s) is quest battle 61's Stone Gaze's: retail's
/// petrified allies did nothing for its 9 s seven times in ten (the other three acted within a second, as if it had
/// been lifted; Bob's Chronofoil captures 130088ec and 3c56e09f.)
const INCAPACITATED: &[u16] = &[PETRIFICATION, 2, 3, GRANITE_GAOL];
/// Petrification.
pub const PETRIFICATION: u16 = 1;
/// The Navel's Granite Gaol.
pub const GRANITE_GAOL: u16 = 292;
/// Sleep, which ends when they're hit.
pub const SLEEP: u16 = 3;
/// Bind: an enemy with it doesn't move (but still attacks.)
const BIND: u16 = 13;

pub fn is_incapacitated(statuses: &StatusEffects) -> bool {
    statuses.ids().any(|id| INCAPACITATED.contains(&id))
}

/// Just raised (status 418, Transcendent: `action::STATUS_RAISED`): no damage reaches them until it ends, after 5 s or
/// at their first action, not by moving. A raised party bot or party NPC keeps it so: it walks, follows and dodges as
/// ever but uses no action, auto-attacks included, until it ends (Bob, 2026-11-10: "allowing the bot to move is fine,
/// just don't have them use an action until transcendence is done".)
const RAISED: u16 = crate::server::action::STATUS_RAISED;

/// An NPC that uses no action now: just raised (`RAISED`.)
pub fn holds_actions(statuses: &StatusEffects) -> bool {
    statuses.ids().any(|id| id == RAISED)
}

pub fn is_rooted(statuses: &StatusEffects) -> bool {
    statuses
        .ids()
        .any(|id| id == BIND || INCAPACITATED.contains(&id))
}

pub fn is_invulnerable(statuses: &[u16]) -> bool {
    statuses.iter().any(|id| INVULNERABLE.contains(id))
}

pub fn has_hp_floor(statuses: &[u16]) -> bool {
    statuses.iter().any(|id| HP_FLOOR.contains(id))
}

/// A player's chances: critical hit rate and multiplier, direct hit rate, from their Critical Hit and Direct Hit
/// Rate (the usual formulas: rate = ⌊200 × (CRIT − sub) / div + 50⌋ / 1000, multiplier = ⌊200 × (CRIT − sub) / div +
/// 1400⌋ / 1000, direct hit rate = ⌊550 × (DH − sub) / div⌋ / 1000; sub is the level's base substat, div its
/// LevelModifier.)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HitChances {
    pub critical_rate: f32,
    pub critical_multiplier: f32,
    pub direct_hit_rate: f32,
}

impl HitChances {
    pub fn from_stats(critical_hit: u32, direct_hit: u32, level_sub: u32, level_div: u32) -> Self {
        let div = level_div.max(1) as f32;
        let crit = critical_hit.saturating_sub(level_sub) as f32;
        let dh = direct_hit.saturating_sub(level_sub) as f32;
        Self {
            critical_rate: (200.0 * crit / div + 50.0).floor() / 1000.0,
            critical_multiplier: (200.0 * crit / div + 1400.0).floor() / 1000.0,
            direct_hit_rate: (550.0 * dh / div).floor() / 1000.0,
        }
    }
}

/// Everything about one player action that changes its numbers.
pub struct PlayerHit<'a> {
    pub chances: HitChances,
    /// The caster's statuses.
    pub caster_statuses: &'a [u16],
    /// The target's statuses, class or job, and whether it's a player.
    pub target_statuses: &'a [u16],
    pub target_classjob: u8,
    pub target_is_player: bool,
    /// How many levels the target (an enemy) stands above the caster; 0 or less for none (`level_gap_multiplier`.)
    pub level_gap: i32,
    /// Random numbers in [0, 1): variance, critical roll, direct hit roll (passed in so tests can fix them.)
    pub rolls: [f32; 3],
}

/// What a player's hit does to an enemy above their level: retail's players did about 0.895 as much one to three levels
/// up, 0.86 at four or five and 0.81 at six or seven (Bob's Chronofoil captures, open world, a few hundred hits against
/// the formula; `combat.md`). Further up wasn't recorded: the last step is kept.
pub fn level_gap_multiplier(gap: i32) -> f32 {
    match gap {
        i32::MIN..=0 => 1.0,
        1..=3 => 0.895,
        4..=5 => 0.86,
        _ => 0.81,
    }
}

/// Applies variance, critical and direct hits, and the status modifiers to a player's damage and heals (the effects
/// their action's script returned.) Heals that go back to the caster (drains) only get the variance.
pub fn apply_player_hit(effects: &mut [TargetEffect], hit: &PlayerHit) {
    let variance = 1.0 + (hit.rolls[0] * 2.0 - 1.0) * PLAYER_VARIANCE;
    let critical = hit.rolls[1] < hit.chances.critical_rate;
    let direct = hit.rolls[2] < hit.chances.direct_hit_rate;
    for effect in effects.iter_mut() {
        match &mut effect.0 {
            TargetEffectKind::Damage {
                amount,
                damage_kind,
                damage_type,
                ..
            } => {
                if *amount == 0 {
                    continue;
                }
                let mut damage = *amount as f32 * variance * level_gap_multiplier(hit.level_gap);
                if critical {
                    damage *= hit.chances.critical_multiplier;
                    *damage_kind |= DamageKind::CRITICAL;
                }
                if direct {
                    damage *= DIRECT_HIT_MULTIPLIER;
                    *damage_kind |= DamageKind::DIRECT_HIT;
                }
                damage *= damage_dealt_multiplier(hit.caster_statuses, *damage_type);
                damage *= damage_taken_multiplier(
                    hit.target_statuses,
                    hit.target_classjob,
                    hit.target_is_player,
                );
                *amount = damage.round().clamp(1.0, u16::MAX as f32) as u16;
            }
            TargetEffectKind::Heal { unk1, amount } => {
                if *amount == 0 {
                    continue;
                }
                let mut heal = *amount as f32 * variance;
                if unk1[4] & EFFECT_ON_SOURCE == 0 {
                    if critical {
                        heal *= hit.chances.critical_multiplier;
                        unk1[1] |= HEAL_CRITICAL;
                    }
                    heal *= healing_received_multiplier(hit.target_statuses);
                }
                *amount = heal.round().clamp(1.0, u16::MAX as f32) as u16;
            }
            _ => {}
        }
    }
}

/// What an enemy's hit on a player becomes after the enemy's debuffs (Feint, Addle, Reprisal) and the player's
/// defences (Rampart, Sentinel... Tank Mastery; Hallowed Ground stops it.) Its variance and criticals are already
/// rolled (`roll_npc_hit`.)
pub fn apply_hit_on_player(
    effects: &mut [TargetEffect],
    enemy_statuses: &[u16],
    player_statuses: &[u16],
    player_classjob: u8,
    defences: Option<Defences>,
) {
    let invulnerable = is_invulnerable(player_statuses);
    for effect in effects.iter_mut() {
        if let TargetEffectKind::Damage {
            amount,
            damage_type,
            ..
        } = &mut effect.0
        {
            if invulnerable {
                effect.0 = TargetEffectKind::Invincible;
                continue;
            }
            let damage = *amount as f32
                * damage_dealt_multiplier(enemy_statuses, *damage_type)
                * damage_taken_multiplier(player_statuses, player_classjob, true)
                * defence_multiplier(defences, *damage_type);
            *amount = damage.round().clamp(1.0, u16::MAX as f32) as u16;
        }
    }
}

#[cfg(test)]
mod tests {
    use kawari::ipc::zone::DamageElement;

    use super::*;

    fn damage(amount: u16, damage_type: DamageType) -> TargetEffect {
        TargetEffect(TargetEffectKind::Damage {
            damage_kind: DamageKind::default(),
            damage_type,
            damage_element: DamageElement::Unaspected,
            bonus_percent: 0,
            unk3: 0,
            unk4: 0,
            amount,
        })
    }

    fn amount(effect: &TargetEffect) -> u16 {
        match effect.0 {
            TargetEffectKind::Damage { amount, .. } | TargetEffectKind::Heal { amount, .. } => {
                amount
            }
            _ => 0,
        }
    }

    const LOW_LEVEL: HitChances = HitChances {
        critical_rate: 0.05,
        critical_multiplier: 1.4,
        direct_hit_rate: 0.0,
    };

    #[test]
    fn statuses_change_damage() {
        // Fight or Flight on the caster, Trick Attack on the enemy.
        let mut effects = [damage(1000, DamageType::Slashing)];
        apply_player_hit(
            &mut effects,
            &PlayerHit {
                chances: LOW_LEVEL,
                caster_statuses: &[76],
                target_statuses: &[3254],
                target_classjob: 0,
                target_is_player: false,
                level_gap: 0,
                rolls: [0.5, 0.99, 0.99],
            },
        );
        assert_eq!(amount(&effects[0]), 1375);

        // Feint on an enemy only softens its physical hits much.
        assert_eq!(damage_dealt_multiplier(&[1195], DamageType::Blunt), 0.9);
        assert_eq!(damage_dealt_multiplier(&[1195], DamageType::Magic), 0.95);

        // A quest battle's tank and healer statuses: +30 %; the DPS one changes nothing.
        assert_eq!(damage_dealt_multiplier(&[1041], DamageType::Magic), 1.3);
        assert_eq!(damage_dealt_multiplier(&[1264], DamageType::Slashing), 1.3);
        assert_eq!(damage_dealt_multiplier(&[1265], DamageType::Magic), 1.0);
    }

    #[test]
    fn variance_and_criticals() {
        let hit = |rolls| {
            let mut effects = [damage(100, DamageType::Magic)];
            apply_player_hit(
                &mut effects,
                &PlayerHit {
                    chances: LOW_LEVEL,
                    caster_statuses: &[],
                    target_statuses: &[],
                    target_classjob: 0,
                    target_is_player: false,
                    level_gap: 0,
                    rolls,
                },
            );
            effects[0].0
        };
        // ±5 %.
        assert!(matches!(
            hit([0.0, 0.9, 0.9]),
            TargetEffectKind::Damage { amount: 95, .. }
        ));
        assert!(matches!(
            hit([0.999, 0.9, 0.9]),
            TargetEffectKind::Damage { amount: 105, .. }
        ));
        // A critical: ×1.4 and the flag.
        let TargetEffectKind::Damage {
            amount,
            damage_kind,
            ..
        } = hit([0.5, 0.01, 0.9])
        else {
            panic!()
        };
        assert_eq!(amount, 140);
        assert!(damage_kind.contains(DamageKind::CRITICAL));
    }

    #[test]
    fn tanks_and_defences() {
        // A level 42 Paladin (about 400 defense, divisor 272: 22 %) under Rampart: 0.78 / 0.92 x Tank Mastery x
        // Rampart.
        let paladin = Some(Defences {
            defense: 400,
            magic_defense: 400,
            level_div: 272,
            tenacity_over: 0,
        });
        let mut effects = [damage(1000, DamageType::Slashing)];
        apply_hit_on_player(&mut effects, &[], &[1191], 19, paladin);
        assert_eq!(amount(&effects[0]), 543);

        let mut effects = [damage(1000, DamageType::Slashing)];
        apply_hit_on_player(&mut effects, &[], &[82], 19, paladin);
        assert!(matches!(effects[0].0, TargetEffectKind::Invincible));

        // A White Mage geared as the recordings' characters were (8 % and 15 %) takes it all, by either.
        let geared = Some(Defences {
            defense: 146,
            magic_defense: 273,
            level_div: 272,
            tenacity_over: 0,
        });
        let mut effects = [damage(1000, DamageType::Slashing)];
        apply_hit_on_player(&mut effects, &[], &[], 24, geared);
        assert_eq!(amount(&effects[0]), 1000);
        let mut effects = [damage(1000, DamageType::Magic)];
        apply_hit_on_player(&mut effects, &[], &[], 24, geared);
        assert_eq!(amount(&effects[0]), 1000);

        // Naked, more; a unique hit ignores defences.
        let naked = Some(Defences {
            defense: 0,
            magic_defense: 0,
            level_div: 272,
            tenacity_over: 0,
        });
        let mut effects = [damage(1000, DamageType::Slashing)];
        apply_hit_on_player(&mut effects, &[], &[], 24, naked);
        assert_eq!(amount(&effects[0]), 1087);
        let mut effects = [damage(1000, DamageType::Unique)];
        apply_hit_on_player(&mut effects, &[], &[], 24, naked);
        assert_eq!(amount(&effects[0]), 1000);
    }

    /// Speed above the base raises damage and healing over time; at the base nothing changes. Tenacity takes a little
    /// off hits.
    #[test]
    fn speed_and_tenacity() {
        assert_eq!(speed_factor(155, 155, 168), 1000);
        // 400 over the base at a divisor of 1000: 52 thousandths.
        assert_eq!(speed_factor(800, 400, 1000), 1052);
        let tank = Some(Defences {
            defense: 0,
            magic_defense: 0,
            level_div: 272,
            tenacity_over: 27,
        });
        // floor(100 × 27 / 272) = 9 thousandths off, on top of defense's 1 / 0.92.
        let multiplier = defence_multiplier(tank, DamageType::Slashing);
        assert!((multiplier - 0.991 / 0.92).abs() < 1e-4);
    }

    #[test]
    fn enemies_above_take_less() {
        assert_eq!(level_gap_multiplier(-3), 1.0);
        assert_eq!(level_gap_multiplier(2), 0.895);
        assert_eq!(level_gap_multiplier(5), 0.86);
        assert_eq!(level_gap_multiplier(7), 0.81);
    }

    #[test]
    fn chances_from_stats() {
        // At level 1 (sub 56, div 56) with base Critical Hit: 5 % for ×1.4.
        let chances = HitChances::from_stats(56, 56, 56, 56);
        assert_eq!(chances.critical_rate, 0.05);
        assert_eq!(chances.critical_multiplier, 1.4);
        assert_eq!(chances.direct_hit_rate, 0.0);
    }
}
