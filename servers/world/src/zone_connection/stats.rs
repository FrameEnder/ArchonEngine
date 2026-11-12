//! Managing statistics, including your classjob and other related information.

use crate::{
    GameData, SyncedItemStats, ToServer, ZoneConnection,
    gamedata::{ItemInfoQuery, ItemRow, Modifiers},
    inventory::{ITEM_FLAG_HQ, Storage},
    lua::LuaPlayer,
};
use icarus::{ParamGrow::ParamGrowRow, Tribe::TribeRow};
use kawari::{
    common::{ActionCategory, MAXIMUM_RESTED_EXP, ObjectId},
    ipc::zone::{
        ActorControlCategory, Condition, PlayerStats, ServerZoneIpcData, ServerZoneIpcSegment,
        UpdateClassInfo,
    },
};
use mlua::{UserData, UserDataMethods};

/// Every BaseParam row, some of them may be useless.
/// BaseParam index for Mind, the primary stat of healers.
const BASE_PARAM_MIND: u8 = 5;
/// BaseParam index for Dexterity, which physical attacks use instead of Strength for classes whose primary stat it is.
const BASE_PARAM_DEXTERITY: u8 = 2;

/// Each level's main stat modifier: what a player's main stats (Strength to Piety) start from before race and gear,
/// what their Determination is with no gear, and the baseline attack power is measured against. Levels 1-80 are
/// Sapphire's level table (1-10 are exactly what a fresh retail character shows as Determination); after that it rises
/// 5 a level, to 390 at 90 and 440 at 100.
const LEVEL_MAIN: [u16; 100] = [
    20, 21, 22, 24, 26, 27, 29, 31, 33, 35, 36, 38, 41, 44, 46, 49, 52, 54, 57, 60, 63, 67, 71, 74,
    78, 81, 85, 89, 92, 97, 101, 106, 110, 115, 119, 124, 128, 134, 139, 144, 150, 155, 161, 166,
    171, 177, 183, 189, 196, 202, 204, 205, 207, 209, 210, 212, 214, 215, 217, 218, 224, 228, 236,
    244, 252, 260, 268, 276, 284, 292, 296, 300, 305, 310, 315, 320, 325, 330, 335, 340, 345, 350,
    355, 360, 365, 370, 375, 380, 385, 390, 395, 400, 405, 410, 415, 420, 425, 430, 435, 440,
];

/// Retail's max HP (patch 7.x), fitted from Bob's Chronofoil captures: his own PlayerStats (exact Vitality and HP,
/// levels 1-22, 30-37, 50, 54, 60, 67, 70, 76, 80, 86-90 on several jobs) and level-synced party members, whose gear
/// is capped so their Vitality can be rebuilt (sync 50 and 80: 82 players matched to the HP, the rest had food or
/// were barely above the sync.) See docs/src/fork/combat.md.
///
/// HP = floor(base × job HP modifier / 100) + floor(per_vitality × (Vitality − the level's main stat)).
///
/// The base per level, at the levels it was measured; levels in between are interpolated (and past 90 it keeps rising
/// 100 a level, as from 80 to 90.)
const HP_BASE: [(u16, u32); 24] = [
    (1, 86),
    (10, 167),
    (18, 279),
    (19, 293),
    (20, 307),
    (21, 337),
    (22, 371),
    (30, 537),
    (31, 577),
    (36, 747),
    (37, 785),
    (50, 1400),
    (54, 1440),
    (60, 1500),
    (67, 1640),
    (70, 1700),
    (76, 1880),
    (80, 2000),
    (86, 2600),
    (87, 2700),
    (88, 2800),
    (90, 3000),
    (100, 4000),
    (u16::MAX, 4000),
];
/// HP per point of Vitality above the level's main stat, in tenths, for everyone but tanks.
const HP_PER_VITALITY: [(u16, u32); 13] = [
    (1, 56),
    (22, 56),
    (30, 80),
    (31, 81),
    (36, 90),
    (37, 90),
    (50, 112),
    (54, 119),
    (70, 140),
    (80, 188),
    (86, 221),
    (87, 226),
    (90, 243),
];
/// The same for tanks, measured from 50 to 80; outside that, the non-tank value times the tank/non-tank ratio at the
/// nearest end (15.5/11.2 at 50, 26.6/18.8 at 80.)
const TANK_HP_PER_VITALITY: [(u16, u32); 5] =
    [(50, 155), (60, 175), (67, 185), (76, 235), (80, 266)];

/// Linear interpolation in a (level, value) table, rounding half up; past the last level it continues the last slope.
fn interpolate_level_table(table: &[(u16, u32)], level: u16) -> u32 {
    let level = level.max(1) as i64;
    if level <= table[0].0 as i64 {
        return table[0].1;
    }
    for pair in table.windows(2) {
        let ((a, va), (b, vb)) = (pair[0], pair[1]);
        let (a, b, va, vb) = (a as i64, b as i64, va as i64, vb as i64);
        if level <= b {
            let span = b - a;
            return (va * span + (vb - va) * (level - a) + span / 2).div_euclid(span) as u32;
        }
    }
    let (&(a, va), &(b, vb)) = (&table[table.len() - 2], &table[table.len() - 1]);
    let (a, b, va, vb) = (a as i64, b as i64, va as i64, vb as i64);
    (vb + (vb - va) * (level - b) / (b - a)).max(0) as u32
}

/// HP per Vitality point (tenths) at this level, for tanks or not.
fn hp_per_vitality(level: u16, tank: bool) -> u32 {
    let non_tank = interpolate_level_table(&HP_PER_VITALITY, level);
    if !tank {
        return non_tank;
    }
    let (first, last) = (TANK_HP_PER_VITALITY[0], TANK_HP_PER_VITALITY[4]);
    if level < first.0 {
        (non_tank * first.1 * 2 + 112) / (112 * 2) // × 15.5/11.2, rounded
    } else if level > last.0 {
        (non_tank * last.1 * 2 + 188) / (188 * 2) // × 26.6/18.8, rounded
    } else {
        interpolate_level_table(&TANK_HP_PER_VITALITY, level)
    }
}

/// A player's max HP on retail for their level, job (HP modifier, tank or not) and Vitality, see `HP_BASE`.
pub fn retail_max_hp(level: u16, job_hp_modifier: u32, tank: bool, vitality: u32) -> u32 {
    let base = interpolate_level_table(&HP_BASE, level) as i64 * job_hp_modifier as i64 / 100;
    let above = vitality as i64 - level_main(level) as i64;
    let from_vitality = (hp_per_vitality(level, tank) as i64 * above).div_euclid(10);
    (base + from_vitality).max(1) as u32
}

/// The EXP message's source for EXP that isn't for an enemy (retail's 0xE0000000.)
const NO_EXP_SOURCE: ObjectId = ObjectId(0xE000_0000);

/// The size of one level's EXP bar in rested EXP's units (seconds.)
const RESTED_SECONDS_PER_BAR: i64 = 201_600;

/// What rested EXP adds to `base` EXP and the pool left, from Bob's Chronofoil captures (20+ kills, exact): with
/// enough in the pool, the total is floor(1.5 × base) − 1 and it uses up floor((base + part) × 201600 / the bar's EXP);
/// otherwise the part is floor(pool × the bar's EXP / 201600 / 3) and the pool empties.
pub fn rested_exp_part(base: i32, pool: i32, exp_to_next: i32) -> (i32, i32) {
    if pool <= 0 || base <= 0 || exp_to_next <= 0 {
        return (0, pool.max(0));
    }
    let full = (base * 3 / 2 - 1 - base).max(0);
    let cost = (base + full) as i64 * RESTED_SECONDS_PER_BAR / exp_to_next as i64;
    if pool as i64 >= cost {
        (full, pool - cost as i32)
    } else {
        let part = (pool as i64 * exp_to_next as i64 / RESTED_SECONDS_PER_BAR / 3) as i32;
        (part.min(full), 0)
    }
}

/// The main stat modifier for a level, see `LEVEL_MAIN`.
pub fn level_main(level: u16) -> u32 {
    LEVEL_MAIN[(level.clamp(1, LEVEL_MAIN.len() as u16) - 1) as usize] as u32
}

/// How much attack power (or healing magic potency) above the level's main stat adds, per level:
/// f(AP) = floor(c × (AP − main) / main) + 100, in hundredths. Fitted to Bob's Chronofoil captures (2026-06 to 2026-11,
/// 124 captures, about 24,000 of the players' own hits and heals with their PlayerStats, see
/// docs/src/fork/combat.md): up to level 50 it's 75 over the level's main stat, for damage and healing alike, tanks
/// too (Conjurer and White Mage 6-44, Arcanist 8-45 and Gladiator/Paladin 1-44, within 1-2 % across 40 weapons; the
/// old 125 over ParamGrow's LevelModifier, Sapphire's, fitted worse, and at 80-90 gave a fifth of retail's damage.)
/// Past 50 the anchors are the community's (Allagan Studies: damage 165 at 80, 195 at 90, 237 at 100; tanks 115, 156,
/// 190; healing 569 × main / 1522 at 90), with the levels between interpolated; level 80's White Mage Glare and
/// level 87's Cure II against Glare III agree with them.
const ATTACK_POWER_ANCHORS: [(u16, f32); 4] = [(50, 75.0), (80, 165.0), (90, 195.0), (100, 237.0)];
const TANK_ATTACK_POWER_ANCHORS: [(u16, f32); 4] =
    [(50, 75.0), (80, 115.0), (90, 156.0), (100, 190.0)];
const HEALING_POTENCY_ANCHORS: [(u16, f32); 3] = [(50, 75.0), (90, 145.8), (100, 163.5)];
/// How much Determination above the level's main stat adds: f(DET) = floor(SCALE × (DET − main) / div) + 1000, in
/// thousandths (retail's 140; the captures can't tell it from 130.)
const DETERMINATION_SCALE: f32 = 140.0;
/// f(TNC)'s coefficient, for damage and healing dealt and damage taken (`BaseParameters::tenacity_factor`,
/// `modifiers::defence_multiplier`.)
pub const TENACITY_SCALE: f32 = 100.0;

/// The party bonus, in percent: one for each party bonus group (`GameData::party_bonus_group`) among the members the
/// player's party list shows (those in their zone), once there are two of them; none alone. Retail's PlayerStats in Bob's
/// Chronofoil captures (2026-06 to 2026-11, about 1,250 with a party): Strength, Dexterity, Vitality, Intelligence and
/// Mind ×(100 + n) / 100, rounded down, n the groups (2-5 seen; two tanks gave 1); Piety and the substats unchanged. A
/// party with a Ninja and the other four groups gave 5 %: the Rogue's line counts with the physical ranged, as the sheet
/// has it.
pub fn party_bonus_percent(groups: &[u8]) -> u8 {
    if groups.len() < 2 {
        return 0;
    }
    let mut distinct: Vec<u8> = groups.iter().copied().filter(|group| *group != 0).collect();
    distinct.sort_unstable();
    distinct.dedup();
    distinct.len() as u8
}

/// The coefficient for a level from its anchors: the first one's up to it, straight lines between, the last one's
/// slope past it.
fn anchored(anchors: &[(u16, f32)], level: u16) -> f32 {
    let first = anchors[0];
    if level <= first.0 {
        return first.1;
    }
    for pair in anchors.windows(2) {
        let ((low, a), (high, b)) = (pair[0], pair[1]);
        if level <= high {
            return a + (b - a) * (level - low) as f32 / (high - low) as f32;
        }
    }
    let ((low, a), (high, b)) = (anchors[anchors.len() - 2], anchors[anchors.len() - 1]);
    b + (b - a) * (level - high) as f32 / (high - low) as f32
}

#[derive(Default, Debug, Clone)]
pub struct BaseParameters {
    /// The BaseParam index of this class's primary stat (e.g. 5 for Mind.)
    pub primary_stat: u8,
    pub strength: u32,
    pub dexterity: u32,
    pub vitality: u32,
    pub intelligence: u32,
    pub mind: u32,
    pub piety: u32,
    pub hp: u32,
    pub mp: u32,
    pub tp: u32,
    pub gp: u32,
    pub cp: u32,
    pub physical_damage: u32,
    pub magic_damage: u32,
    pub delay: u32,
    pub additional_effect: u32,
    pub attack_speed: u32,
    pub block_rate: u32,
    pub block_strength: u32,
    pub tenacity: u32,
    pub attack_power: u32,
    pub defense: u32,
    pub direct_hit_rate: u32,
    pub evasion: u32,
    pub magic_defense: u32,
    pub critical_hit_power: u32,
    pub critical_hit_resilience: u32,
    pub critical_hit: u32,
    pub critical_hit_evasion: u32,
    pub slashing_resistance: u32,
    pub piercing_resistance: u32,
    pub blunt_resistance: u32,
    pub projectile_resistance: u32,
    pub attack_magic_potency: u32,
    pub healing_magic_potency: u32,
    pub enhancement_magic_potency: u32,
    pub elemental_bonus: u32,
    pub fire_resistance: u32,
    pub ice_resistance: u32,
    pub wind_resistance: u32,
    pub earth_resistance: u32,
    pub lightning_resistance: u32,
    pub water_resistance: u32,
    pub magic_resistance: u32,
    pub determination: u32,
    pub skill_speed: u32,
    pub spell_speed: u32,
    pub haste: u32,
    pub morale: u32,
    pub enmity: u32,
    pub enmity_reduction: u32,
    pub desynthesis_skill_gain: u32,
    pub exp_bonus: u32,
    pub regen: u32,
    pub special_attribute: u32,
    pub main_attribute: u32,
    pub secondary_attribute: u32,
    pub slow_resistance: u32,
    pub petrification_resistance: u32,
    pub paralysis_resistance: u32,
    pub silence_resistance: u32,
    pub blind_resistance: u32,
    pub posion_resistance: u32,
    pub stun_resistance: u32,
    pub sleep_resistance: u32,
    pub bind_resistance: u32,
    pub heavy_resistance: u32,
    pub doom_resistance: u32,
    pub reduced_durability_loss: u32,
    pub increased_spiritbond_gain: u32,
    pub craftmanship: u32,
    pub control: u32,
    pub gathering: u32,
    pub perception: u32,

    // Players only: their actions do what retail's damage formula gives, rather than potency times a stat. Set by
    // `use_retail_formula`, NPCs keep the old way (their damage is scaled to what they do on retail elsewhere.)
    pub retail_formula: bool,
    /// The level's main stat modifier (see `LEVEL_MAIN`.)
    pub level_main: u32,
    /// Whether the class is a tank (its attack power counts for less past level 50, `TANK_ATTACK_POWER_ANCHORS`.)
    pub tank: bool,
    /// The level's divisor (ParamGrow's LevelModifier.)
    pub level_div: u32,
    /// The level the stats are for.
    pub level: u16,
    /// The main hand weapon's physical and magic damage, and its auto-attack delay in milliseconds.
    pub weapon_physical_damage: u32,
    pub weapon_magic_damage: u32,
    pub weapon_delay_ms: u32,
    /// The class's modifier (percent) for the stat its physical attacks use, and for its primary stat.
    pub physical_stat_modifier: u32,
    pub primary_stat_modifier: u32,

    // Needed for cast time calculations
    pub base_speed: i32,
    pub level_modifier: i32,
}

impl BaseParameters {
    pub fn get_mut(&mut self, index: u8) -> &mut u32 {
        match index {
            0 => &mut self.strength, // HACK: Classjob #0 (used by enemies) index this value I think, so don't remove that without thinking first.
            1 => &mut self.strength,
            2 => &mut self.dexterity,
            3 => &mut self.vitality,
            4 => &mut self.intelligence,
            5 => &mut self.mind,
            6 => &mut self.piety,
            7 => &mut self.hp,
            8 => &mut self.mp,
            9 => &mut self.tp,
            10 => &mut self.gp,
            11 => &mut self.cp,
            12 => &mut self.physical_damage,
            13 => &mut self.magic_damage,
            14 => &mut self.delay,
            15 => &mut self.additional_effect,
            16 => &mut self.attack_speed,
            17 => &mut self.block_rate,
            18 => &mut self.block_strength,
            19 => &mut self.tenacity,
            20 => &mut self.attack_power,
            21 => &mut self.defense,
            22 => &mut self.direct_hit_rate,
            23 => &mut self.evasion,
            24 => &mut self.magic_defense,
            25 => &mut self.critical_hit_power,
            26 => &mut self.critical_hit_resilience,
            27 => &mut self.critical_hit,
            28 => &mut self.critical_hit_evasion,
            29 => &mut self.slashing_resistance,
            30 => &mut self.piercing_resistance,
            31 => &mut self.blunt_resistance,
            32 => &mut self.projectile_resistance,
            33 => &mut self.attack_magic_potency,
            34 => &mut self.healing_magic_potency,
            35 => &mut self.enhancement_magic_potency,
            36 => &mut self.elemental_bonus,
            37 => &mut self.fire_resistance,
            38 => &mut self.ice_resistance,
            39 => &mut self.wind_resistance,
            40 => &mut self.earth_resistance,
            41 => &mut self.lightning_resistance,
            42 => &mut self.water_resistance,
            43 => &mut self.magic_resistance,
            44 => &mut self.determination,
            45 => &mut self.skill_speed,
            46 => &mut self.spell_speed,
            47 => &mut self.haste,
            48 => &mut self.morale,
            49 => &mut self.enmity,
            50 => &mut self.enmity_reduction,
            51 => &mut self.desynthesis_skill_gain,
            52 => &mut self.exp_bonus,
            53 => &mut self.regen,
            54 => &mut self.special_attribute,
            55 => &mut self.main_attribute,
            56 => &mut self.secondary_attribute,
            57 => &mut self.slow_resistance,
            58 => &mut self.petrification_resistance,
            59 => &mut self.paralysis_resistance,
            60 => &mut self.silence_resistance,
            61 => &mut self.blind_resistance,
            62 => &mut self.posion_resistance,
            63 => &mut self.stun_resistance,
            64 => &mut self.sleep_resistance,
            65 => &mut self.bind_resistance,
            66 => &mut self.heavy_resistance,
            67 => &mut self.doom_resistance,
            68 => &mut self.reduced_durability_loss,
            69 => &mut self.increased_spiritbond_gain,
            70 => &mut self.craftmanship,
            71 => &mut self.control,
            72 => &mut self.gathering,
            73 => &mut self.perception,
            _ => unreachable!(),
        }
    }

    /// Calculates a set of attributes based on the level and class modifiers.
    pub fn perform_calculations(
        &mut self,
        primary_stat: u8,
        tribe_row: &TribeRow,
        param_grow: &ParamGrowRow,
        modifiers: &Modifiers,
    ) {
        // Akh Morning data can't be extrapolated from game sheets, and it's missing a significant amount of entries from 50-70, so let's try making a decent approximation with the BaseSpeed, HpModifier and LevelModifier columns.
        // Players' main stats start from the level's main stat modifier, as on retail.
        let base = self.stat_base(param_grow);
        self.strength = modifiers
            .apply_to(1, base)
            .saturating_add_signed(tribe_row.STR as i32);
        self.dexterity = modifiers
            .apply_to(2, base)
            .saturating_add_signed(tribe_row.DEX as i32);
        self.vitality = modifiers
            .apply_to(3, base)
            .saturating_add_signed(tribe_row.VIT as i32);
        self.intelligence = modifiers
            .apply_to(4, base)
            .saturating_add_signed(tribe_row.INT as i32);
        self.mind = modifiers
            .apply_to(5, base)
            .saturating_add_signed(tribe_row.MND as i32);
        self.piety = modifiers
            .apply_to(6, base)
            .saturating_add_signed(tribe_row.PIE as i32);

        self.spell_speed = param_grow.BaseSpeed as u32;
        self.tenacity = param_grow.BaseSpeed as u32;
        self.primary_stat = primary_stat;
        self.attack_power = *self.get_mut(primary_stat);
        self.skill_speed = self.tenacity;
        self.haste = 100; // Controls cast times

        // This is fixed and isn't modified by any items in retail, so it's safe to be set here.
        self.mp = param_grow.MpModifier as u32;

        // Needed for cast time calculations
        self.base_speed = param_grow.BaseSpeed;
        self.level_modifier = param_grow.LevelModifier;

        if self.retail_formula {
            // On retail, a player's substats start at the level's sub modifier, and Determination at its main one.
            self.critical_hit = param_grow.BaseSpeed as u32;
            self.direct_hit_rate = param_grow.BaseSpeed as u32;
            self.determination = self.level_main;
            self.physical_stat_modifier = modifiers.strength as u32;
            if primary_stat == BASE_PARAM_DEXTERITY {
                self.physical_stat_modifier = modifiers.dexterity as u32;
            }
            self.primary_stat_modifier = match primary_stat {
                1 => modifiers.strength,
                2 => modifiers.dexterity,
                3 => modifiers.vitality,
                4 => modifiers.intelligence,
                5 => modifiers.mind,
                6 => modifiers.piety,
                _ => 100,
            } as u32;
        }
    }

    /// Players use retail's stats and damage formula from now on: call before `perform_calculations`.
    pub fn use_retail_formula(
        &mut self,
        level: u16,
        param_grow: &ParamGrowRow,
        weapon: Option<&ItemRow>,
    ) {
        self.retail_formula = true;
        self.level = level;
        self.level_main = level_main(level);
        self.level_div = (param_grow.LevelModifier as u32).max(1);
        if let Some(weapon) = weapon {
            self.weapon_physical_damage = weapon.physical_damage as u32;
            self.weapon_magic_damage = weapon.magic_damage as u32;
            self.weapon_delay_ms = weapon.delay_ms as u32;
        }
    }

    /// What main stats start from at this level.
    fn stat_base(&self, param_grow: &ParamGrowRow) -> u32 {
        if self.retail_formula {
            self.level_main
        } else {
            param_grow.BaseSpeed as u32
        }
    }

    // This should be called after item stat calculations.
    pub fn calculate_potencies(
        &mut self,
        param_grow: &ParamGrowRow,
        modifiers: Option<&Modifiers>,
    ) {
        self.physical_damage = self.strength;
        // Healers' spells deal damage based on Mind, everyone else's on Intelligence.
        self.attack_magic_potency = if self.primary_stat == BASE_PARAM_MIND {
            self.mind
        } else {
            self.intelligence
        };
        self.healing_magic_potency = self.mind;
        if self.retail_formula {
            // Physical attack power is Strength, or Dexterity for classes whose primary stat it is (a conjurer's is
            // its Strength on retail.)
            self.physical_damage = if self.primary_stat == BASE_PARAM_DEXTERITY {
                self.dexterity
            } else {
                self.strength
            };
            self.attack_power = self.physical_damage;
        }

        if self.retail_formula
            && let Some(modifiers) = modifiers
        {
            self.tank = modifiers.tank;
            self.hp = retail_max_hp(
                self.level,
                modifiers.hp as u32,
                modifiers.tank,
                self.vitality,
            );
            return;
        }

        // To calculate HP, we use a formula loosely inspired by Akh Morning and take some liberties to keep it fairly simple, at least for now.
        // TODO: This formula isn't the greatest for 1-50, as near the end of that range it's fairly low compared to retail. For level 80+ though it's pretty close.
        // In clearer terms without all the casts: (100 + hp_modifier + (total_vit - base_vit) * (level_mod / 100)) * job_hp_modifier
        let classjob_hp_mod;
        let classjob_vit_mod;

        if let Some(modifiers) = modifiers {
            classjob_hp_mod = modifiers.hp as f32 / 100.0;
            classjob_vit_mod = modifiers.vitality as f32 / 100.0;
        } else {
            classjob_hp_mod = 1.0;
            classjob_vit_mod = 1.0;
        };

        let hp_mod = param_grow.HpModifier as f32;
        let base_vit = (self.stat_base(param_grow) as f32) * classjob_vit_mod;
        let lv_mod = param_grow.LevelModifier as f32;

        self.hp = (100.0
            + hp_mod
            + ((self.vitality as f32 - base_vit) * (lv_mod / 100.0)) * classjob_hp_mod)
            .round() as u32;
    }

    /// f(WD): the weapon's damage plus a little of the level's main stat (by the class's modifier for the stat used.)
    fn weapon_factor(&self, stat_modifier: u32, weapon_damage: u32) -> f32 {
        (self.level_main as f32 * stat_modifier as f32 / 1000.0).floor() + weapon_damage as f32
    }

    /// Retail's damage and healing formula, with its roundings: floor(floor(potency × f(AP) × f(DET) / 100 / 1000)
    /// × f(WD) / 100), where f(AP) (hundredths) grows with attack power or healing magic potency above the level's main
    /// stat (`ATTACK_POWER_ANCHORS`) and f(DET) (thousandths) with Determination above it. Variance, criticals and
    /// traits are left to the caller (traits come in with the potency.)
    fn retail_damage(
        &self,
        potency: u32,
        attack_power: u32,
        weapon_factor: f32,
        coefficient: f32,
    ) -> u16 {
        let main = self.level_main.max(1) as f32;
        let div = self.level_div.max(1) as f32;
        let f_ap =
            ((coefficient * (attack_power as f32 - main) / main).floor() + 100.0).max(1.0) as u64;
        let f_det = ((DETERMINATION_SCALE * (self.determination as f32 - main) / div).floor()
            + 1000.0)
            .max(1.0) as u64;
        let base = potency as u64 * f_ap * f_det / 100_000;
        let base = base * self.tenacity_factor() / 1000;
        (base * weapon_factor.max(0.0) as u64 / 100).clamp(1, u16::MAX as u64) as u16
    }

    /// f(TNC), in thousandths: Tenacity above the level's base (ParamGrow's BaseSpeed) adds
    /// floor(100 × (TNC − base) / divisor) to damage and healing, as the community's formula has it (Allagan Studies, from
    /// Shadowbringers). Only tanks' gear has Tenacity; at the levels Bob's captures cover a tank's was at most 20-30
    /// over the base (a level 42 Paladin: 287 against 263), under 1 % here, within the formula fit's 1-2 %, so the
    /// captures can't check the coefficient.
    fn tenacity_factor(&self) -> u64 {
        let over = self.tenacity.saturating_sub(self.base_speed.max(0) as u32);
        (TENACITY_SCALE * over as f32 / self.level_div.max(1) as f32).floor() as u64 + 1000
    }

    /// f(AP)'s coefficient for damage at this level (`ATTACK_POWER_ANCHORS`, the tanks' own past 50.)
    fn attack_power_coefficient(&self) -> f32 {
        anchored(
            if self.tank {
                &TANK_ATTACK_POWER_ANCHORS
            } else {
                &ATTACK_POWER_ANCHORS
            },
            self.level.max(1),
        )
    }

    /// Calculates amount of physical damage to apply based on potency.
    fn calc_physical_damage(&self, potency: u32) -> u16 {
        if self.retail_formula {
            let f_wd = self.weapon_factor(self.physical_stat_modifier, self.weapon_physical_damage);
            return self.retail_damage(
                potency,
                self.physical_damage,
                f_wd,
                self.attack_power_coefficient(),
            );
        }
        let normalized_potency = potency as f32 / 100.0;
        (normalized_potency * self.physical_damage as f32).floor() as u16
    }

    /// Calculates amount of auto-attack damage based on potency: physical damage, with f(WD) scaled by the weapon's
    /// delay over 3 seconds (a slow weapon hits harder, less often.)
    fn calc_auto_attack_damage(&self, potency: u32) -> u16 {
        if !self.retail_formula || self.weapon_delay_ms == 0 {
            return self.calc_physical_damage(potency);
        }
        let f_wd = self.weapon_factor(self.physical_stat_modifier, self.weapon_physical_damage);
        let f_auto = (f_wd * self.weapon_delay_ms as f32 / 3000.0).floor();
        self.retail_damage(
            potency,
            self.physical_damage,
            f_auto,
            self.attack_power_coefficient(),
        )
    }

    /// Calculates amount of magic damage to apply based on potency.
    fn calc_magical_damage(&self, potency: u32) -> u16 {
        if self.retail_formula {
            let f_wd = self.weapon_factor(self.primary_stat_modifier, self.weapon_magic_damage);
            return self.retail_damage(
                potency,
                self.attack_magic_potency,
                f_wd,
                self.attack_power_coefficient(),
            );
        }
        let normalized_potency = potency as f32 / 100.0;
        (normalized_potency * self.attack_magic_potency as f32).floor() as u16
    }

    /// Calculates amount of healing to apply based on potency.
    fn calc_heal_amount(&self, potency: u32) -> u16 {
        if self.retail_formula {
            let f_wd = self.weapon_factor(self.primary_stat_modifier, self.weapon_magic_damage);
            let coefficient = anchored(&HEALING_POTENCY_ANCHORS, self.level.max(1));
            return self.retail_damage(potency, self.healing_magic_potency, f_wd, coefficient);
        }
        let normalized_potency = potency as f32 / 100.0;
        (normalized_potency * self.healing_magic_potency as f32).floor() as u16
    }

    /// Food's bonuses (`GameData::food_bonuses`): a percentage of the stat as it stands, rounded down, up to the food's
    /// maximum, or a flat amount. The game data's numbers, as the wiki describes them; no capture has the stats before
    /// and after a meal (the only one, cc26e0ea, came with a change of class.)
    pub fn add_food(&mut self, bonuses: &[crate::gamedata::FoodBonus]) {
        for bonus in bonuses {
            if bonus.base_param == 0 || bonus.base_param as usize > 73 {
                continue;
            }
            let stat = self.get_mut(bonus.base_param);
            let amount = if bonus.relative {
                ((*stat as i64 * bonus.value as i64 / 100) as i32).min(bonus.max)
            } else {
                bonus.value
            };
            *stat = stat.saturating_add_signed(amount.max(0));
        }
    }

    /// The party bonus (`party_bonus_percent`) on the five main attributes, rounded down.
    pub fn add_party_bonus(&mut self, percent: u8) {
        if percent == 0 {
            return;
        }
        for stat in [
            &mut self.strength,
            &mut self.dexterity,
            &mut self.vitality,
            &mut self.intelligence,
            &mut self.mind,
        ] {
            *stat = (*stat as u64 * (100 + percent as u64) / 100) as u32;
        }
    }

    /// Adds what the equipped items give (see `GameData::equipped_item_stats`): their attributes, defense and magic
    /// defense.
    pub fn add_equipped_items(&mut self, items: &[SyncedItemStats]) {
        for item in items {
            self.defense += item.defense;
            self.magic_defense += item.magic_defense;
            for (param_id, value) in item.params.into_iter().chain(item.materia) {
                if param_id != 0 {
                    *self.get_mut(param_id) = self.get_mut(param_id).saturating_add_signed(value);
                }
            }
        }
    }

    /// Takes into account the haste and skill/speed stat based on `Client::Game::ActionManager::GetAdjustedCastTime`.
    /// `cast_time` should be in milliseconds!
    /// Returns a cast time in milliseconds.
    pub fn get_adjusted_cast_time(&self, action_category: ActionCategory, cast_time: u32) -> i32 {
        // It only applies to these kinds of actions...
        if action_category != ActionCategory::Invalid
            && action_category != ActionCategory::AutoAttack
            && action_category != ActionCategory::Spell
            && action_category != ActionCategory::Weaponskill
        {
            return cast_time as i32;
        }

        let stat_speed_value = match action_category {
            ActionCategory::Weaponskill => self.skill_speed as i32,
            ActionCategory::Spell => self.spell_speed as i32,
            _ => self.base_speed,
        };

        let ivar12 = ((self.base_speed - stat_speed_value) * 0x82) / (self.level_modifier) + 1000;
        let mut new_val = 0;
        if -1 < ivar12 {
            new_val = ivar12;
        }

        (self.haste as i32 * ((new_val * cast_time as i32) / 1000)) / 100
    }
}

impl BaseParameters {
    /// A recast after speed and haste, in milliseconds: like a cast time (Skill Speed for weaponskills, Spell Speed for
    /// spells), but never under 1.5 s (the client's recast adjustment, called by
    /// `Client::Game::ActionManager::GetAdjustedRecastTime` for actions with at most one charge; read on the
    /// 2026.10.16 client and Bob's build of 2026-10-20.) Other kinds of action keep their recast.
    pub fn get_adjusted_recast_time(&self, action_category: ActionCategory, recast: u32) -> i32 {
        if !matches!(
            action_category,
            ActionCategory::Spell | ActionCategory::Weaponskill
        ) {
            return recast as i32;
        }
        self.get_adjusted_cast_time(action_category, recast)
            .max(crate::client_rules::MIN_ADJUSTED_RECAST_MS)
    }
}

impl UserData for BaseParameters {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("calc_physical_damage", |_, this, potency: u32| {
            Ok(this.calc_physical_damage(potency))
        });
        methods.add_method("calc_auto_attack_damage", |_, this, potency: u32| {
            Ok(this.calc_auto_attack_damage(potency))
        });
        methods.add_method("calc_magical_damage", |_, this, potency: u32| {
            Ok(this.calc_magical_damage(potency))
        });
        methods.add_method("calc_heal_amount", |_, this, potency: u32| {
            Ok(this.calc_heal_amount(potency))
        });
    }
}

impl ZoneConnection {
    pub async fn update_class_info(&mut self) {
        self.send_class_info().await;

        // Update rested EXP so the bar doesn't reset.
        self.actor_control_self(ActorControlCategory::UpdateRestedExp {
            exp: self.player_data.classjob.rested_exp as u32,
        })
        .await;

        // Send this too, otherwise actions dependent on the gauge won't function
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorGauge {
            classjob_id: self.player_data.classjob.current_class as u8,
            data: [0; 15],
        });
        self.send_ipc_self(ipc).await;
    }

    /// The player's own share of a paralysis proc, as retail sent it (Bob's Chronofoil capture 44df02fc, every proc
    /// on the player in The Lominsan Way): when it cut a cast short, the conditions (Casting off) and CancelCast for
    /// each, then the proc (FloatingNumberVFX `17, 30, 0, player`), all as ActorControl from the player; the server
    /// then sends StatusEffectList and UpdateClassInfo (`paralysis::seize_up`.)
    pub async fn paralysis_proc(&mut self, cancelled: Vec<(u32, u32)>) {
        let actor_id = self.player_data.character.actor_id;
        if !cancelled.is_empty() {
            self.conditions.toggle_condition(Condition::Casting, false);
            self.send_conditions().await;
        }
        for (unk1, action_id) in cancelled {
            self.actor_control(
                actor_id,
                ActorControlCategory::CancelCast {
                    unk1,
                    unk2: 1,
                    action_id,
                    unk3: 1,
                },
            )
            .await;
        }
        self.actor_control(
            actor_id,
            ActorControlCategory::FloatingNumberVFX {
                unk1: crate::server::STATUS_PARALYSIS as u32,
                kind: 30,
                amount: 0,
                object_id: actor_id,
            },
        )
        .await;
    }

    /// UpdateClassInfo alone.
    pub async fn send_class_info(&mut self) {
        let ipc;
        {
            let game_data = self.game_data.lock();

            ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::UpdateClassInfo(UpdateClassInfo {
                classjob_id: self.player_data.classjob.current_class as u8,
                class_level: self.current_level(&game_data),
                // While synced, retail sends the synced level here and the real one in class_level: the client
                // locks and downgrades actions by it.
                current_level: self
                    .synced_level
                    .map(|level| level as u16)
                    .unwrap_or(self.current_level(&game_data)),
                synced_level: self.synced_level.unwrap_or_default() as u16,
                current_exp: self.current_exp(&game_data),
                // Retail fills it here too (604800 at most, a full bar), not only in UpdateRestedExp.
                rested_exp: self.player_data.classjob.rested_exp as u32,
            }));
        }
        self.send_ipc_self(ipc).await;
    }

    /// Syncs the player to a FATE's level (`on`), or ends it (the FATE ended, or they asked.) As retail (Bob's
    /// Chronofoil captures, docs/src/fork/level-sync.md): only when they're above the FATE's maximum level; stats, class
    /// info, then FateLevelSynced with the FATE (0 when unsyncing.)
    pub async fn set_fate_level_sync(&mut self, fate_id: u32, on: bool) {
        if on {
            let (max_level, current_level) = {
                let mut game_data = self.game_data.lock();
                (
                    game_data.get_fate_max_level(fate_id),
                    self.current_level(&game_data),
                )
            };
            let Some(max_level) = max_level.filter(|max| current_level > *max as u16) else {
                tracing::info!("Not syncing to FATE {fate_id}: already at or below its level.");
                return;
            };
            self.synced_level = Some(max_level);
            self.synced_item_level = None;
            self.synced_fate = Some(fate_id);
        } else {
            if self.synced_fate.is_none() {
                return;
            }
            self.synced_level = None;
            self.synced_fate = None;
        }

        self.send_stats().await;
        self.update_class_info().await;
        self.actor_control_self(ActorControlCategory::FateLevelSynced {
            fate_id: if on { fate_id } else { 0 },
        })
        .await;
    }

    /// Tells the client its class changed (`current_class` already set), as retail does it (Bob's Chronofoil capture
    /// d3379099 of 2026-11-04: a Conjurer to Gladiator and back, by the weapon and by a gearset): the new class's
    /// gauge, the stats, UnkClassRelated, the ClassJobChange ActorControl (to the player and everyone around), the pet
    /// hotbar cleared (SetupPet and SetPetParameters with no pet, twice), then the equipment (Equip, the item level)
    /// and UpdateClassInfo last, which `update_equip` sends (Kawari's came first, before UnkClassRelated.) Only when the
    /// class really changes: retail sent none of this for a weapon or gearset of the same class.
    pub async fn finish_changing_class(&mut self) {
        let classjob_id = self.player_data.classjob.current_class as u8;
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorGauge {
            classjob_id,
            data: [0; 15],
        });
        self.send_ipc_self(ipc).await;
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::UnkJobGauge { unk: [0; 8] });
        self.send_ipc_self(ipc).await;
        self.send_stats().await;

        let ipc;
        {
            let game_data = self.game_data.lock();

            ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::UnkClassRelated {
                classjob_id,
                class_level: self.current_level(&game_data),
                current_level: self.current_level(&game_data),
            });
        }
        self.send_ipc_self(ipc).await;

        // An ActorControl (not ActorControlSelf) to the player, as retail, and to everyone around: the VFX.
        let actor_id = self.player_data.character.actor_id;
        let category = ActorControlCategory::ClassJobChangeVFX {
            classjob_id: classjob_id as u32,
        };
        self.actor_control(actor_id, category.clone()).await;
        self.handle
            .send(ToServer::BroadcastActorControl(actor_id, category))
            .await;

        for _ in 0..2 {
            self.clear_pet_hotbar().await;
        }

        // Commit back our classjob data after changing. Done here so it gets picked up by any paths that change classjob.
        {
            let mut db = self.database.lock();
            db.commit_classjob_and_inventory(&self.player_data);
        }
    }

    /// SetupPet and SetPetParameters with no pet (the pet hotbar hidden), as retail sent them on every class change
    /// and when a class was unlocked (Bob's Chronofoil capture d3379099): `[owner, 0, no actor, 1, 1]` and
    /// `[0, 0, 7, 0]`.
    pub async fn clear_pet_hotbar(&mut self) {
        self.actor_control_self(ActorControlCategory::SetupPet {
            owner_id: self.player_data.character.actor_id,
            pet_id: 0,
            pet_actor_id: ObjectId::default(),
            unk2: 1,
            unk3: 1,
        })
        .await;
        self.actor_control_self(ActorControlCategory::SetPetParameters {
            pet_id: 0,
            unk2: 0,
            unk3: 7,
            unk4: 0,
        })
        .await;
    }

    /// What each equipped item gives, capped under item level sync `sync_item_level`.
    fn equipped_item_stats(
        &self,
        game_data: &mut GameData,
        sync_item_level: Option<u16>,
    ) -> Vec<SyncedItemStats> {
        let equipped = &self.player_data.inventory.equipped;
        (0..equipped.max_slots())
            .map(|i| equipped.get_slot(i as u16))
            .filter(|item| item.quantity > 0)
            // Gear at 0% condition gives no stats (the wiki's Gear Durability page.)
            .filter(|item| item.condition > 0)
            .filter_map(|item| {
                let materia: Vec<(u16, u8)> = item
                    .materia
                    .iter()
                    .copied()
                    .zip(item.materia_grades.iter().copied())
                    .collect();
                game_data.equipped_item_stats(
                    item.item_id,
                    item.item_flags & ITEM_FLAG_HQ != 0,
                    sync_item_level,
                    &materia,
                )
            })
            .collect()
    }

    /// Parameters at `level` (the synced level while synced), with the equipped items capped at `sync_item_level`, or
    /// without any items when `with_items` is false.
    fn parameters_at(
        &self,
        level: u16,
        sync_item_level: Option<u16>,
        with_items: bool,
    ) -> BaseParameters {
        let mut game_data = self.game_data.lock();

        let modifiers = game_data
            .get_class_job_modifiers(self.player_data.classjob.current_class as u32)
            .expect("Failed to read param grow");

        let tribe_row = game_data
            .get_tribe(self.player_data.customize.chara_make.customize.tribe as u8)
            .expect("Failed to read tribe");

        let param_grow = game_data
            .get_param_grow(level as u32)
            .expect("Failed to read param grow");

        let primary_stat = game_data
            .get_job_primary_stat(self.player_data.classjob.current_class as u16)
            .unwrap_or(1);

        let main_hand = &self.player_data.inventory.equipped.main_hand;
        let weapon = game_data.get_item_info(ItemInfoQuery::ById(main_hand.item_id));
        // Its HQ bonus counts, and under sync it's capped at the sync item level's (`GameData::weapon_damage`.)
        let weapon_damage = game_data.weapon_damage(
            main_hand.item_id,
            main_hand.item_flags & ITEM_FLAG_HQ != 0,
            sync_item_level.filter(|_| with_items),
        );

        let mut base_parameters = BaseParameters::default();
        base_parameters.use_retail_formula(level, &param_grow, weapon.as_ref());
        if let Some((physical, magic)) = weapon_damage {
            base_parameters.weapon_physical_damage = physical;
            base_parameters.weapon_magic_damage = magic;
        }
        base_parameters.perform_calculations(primary_stat, &tribe_row, &param_grow, &modifiers);
        if with_items {
            let items = self.equipped_item_stats(&mut game_data, sync_item_level);
            base_parameters.add_equipped_items(&items);
            // Food on the stats with gear, then the party bonus on the attributes, before HP and attack power come
            // from them. (Which of the two comes first on retail isn't known: they only meet on Vitality.)
            if let Some(well_fed) = self.well_fed {
                base_parameters.add_food(&game_data.food_bonuses(well_fed));
            }
            base_parameters.add_party_bonus(self.party_bonus);
        }
        // Crafters have 180 CP before their gear (Craftimizer's base stats, which it adds gear to.)
        if crate::common::is_crafter(self.player_data.classjob.current_class as u8) {
            base_parameters.cp += 180;
        }
        base_parameters.calculate_potencies(&param_grow, Some(&modifiers));
        // Retail's PlayerStats has the weapon's delay here (2400 for a Maple Wand.)
        if let Some(weapon) = &weapon {
            base_parameters.delay = weapon.delay_ms as u32;
        }

        base_parameters
    }

    /// Scaled parameters based on level and item level sync.
    pub fn base_parameters(&self) -> BaseParameters {
        let level = match self.synced_level {
            Some(level) => level as u16,
            None => self.current_level(&self.game_data.lock()),
        };

        // Under sync, items are capped at the content's own item level if it has one, or the synced level's.
        let sync_item_level = self.synced_level.map(|_| {
            self.synced_item_level.unwrap_or_else(|| {
                self.game_data
                    .lock()
                    .get_param_grow(level as u32)
                    .map(|param_grow| param_grow.ItemLevelSync)
                    .unwrap_or_default()
            })
        });
        // Minimum IL: no higher than the duty's minimum, whether the level is synced or not.
        let sync_item_level = match (sync_item_level, self.minimum_item_level) {
            (Some(synced), Some(minimum)) => Some(synced.min(minimum)),
            (synced, minimum) => synced.or(minimum),
        };

        let mut parameters = self.parameters_at(level, sync_item_level, true);
        // The Echo raises maximum HP (`server/duty_finder.rs`.)
        if self.echo_multiplier > 1.0 {
            parameters.hp = (parameters.hp as f32 * self.echo_multiplier).round() as u32;
        }
        parameters
    }

    /// The character's own parameters without any gear, at their real level even while synced: what retail sends
    /// as PlayerStats' base values (the character window shows the difference as the gear's bonus.)
    pub fn naked_base_parameters(&self) -> BaseParameters {
        let level = self.current_level(&self.game_data.lock());
        self.parameters_at(level, None, false)
    }

    pub async fn send_stats(&mut self) {
        let base_parameters = self.base_parameters();
        let unscaled_base_parameters = self.naked_base_parameters();

        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::PlayerStats(PlayerStats {
            strength: base_parameters.strength,
            dexterity: base_parameters.dexterity,
            vitality: base_parameters.vitality,
            intelligence: base_parameters.intelligence,
            mind: base_parameters.mind,
            piety: base_parameters.piety,
            hp: base_parameters.hp,
            mp: base_parameters.mp,
            tp: base_parameters.tp,
            gp: base_parameters.gp,
            cp: base_parameters.cp,
            delay: base_parameters.delay,
            tenacity: base_parameters.tenacity,
            attack_power: base_parameters.attack_power,
            defense: base_parameters.defense,
            direct_hit_rate: base_parameters.direct_hit_rate,
            evasion: base_parameters.evasion,
            magic_defense: base_parameters.magic_defense,
            critical_hit: base_parameters.critical_hit,
            attack_magic_potency: base_parameters.attack_magic_potency,
            healing_magic_potency: base_parameters.healing_magic_potency,
            elemental_bonus: base_parameters.elemental_bonus,
            determination: base_parameters.determination,
            skill_speed: base_parameters.skill_speed,
            spell_speed: base_parameters.spell_speed,
            haste: base_parameters.haste,
            craftmanship: base_parameters.craftmanship,
            control: base_parameters.control,
            gathering: base_parameters.gathering,
            perception: base_parameters.perception,
            base_strength: unscaled_base_parameters.strength,
            base_dexterity: unscaled_base_parameters.dexterity,
            base_vitality: unscaled_base_parameters.vitality,
            base_intelligence: unscaled_base_parameters.intelligence,
            base_mind: unscaled_base_parameters.mind,
            base_piety: unscaled_base_parameters.piety,
        }));
        self.send_ipc_self(ipc).await;

        self.update_server_stats().await;
    }

    /// Inform the server of new updated level/HP/MP stats.
    async fn update_server_stats(&mut self) {
        let current_level;
        {
            let game_data = self.game_data.lock();
            if let Some(synced_level) = self.synced_level {
                current_level = synced_level;
            } else {
                current_level = self.current_level(&game_data) as u8;
            }
        }

        let base_parameters = self.base_parameters();

        self.handle
            .send(ToServer::SetNewStatValues(
                self.player_data.character.actor_id,
                current_level,
                self.player_data.classjob.current_class as u8,
                base_parameters,
            ))
            .await;
    }

    /// Brings the scripts' view of the player up to date: their data and the level they're at (the synced level
    /// while synced, as their spawn has it). Event scripts saw level 0 until 2026-10-30, so every warp asking for a
    /// class level (WarpCondition's ClassLevel) turned the player away: Romarique's ferry to Sweetbloom Pier (Warp
    /// 131077, condition 4: level 10) said to come back at level 10 to a level 24 (Bob's bundle 20261030-101331.)
    pub fn sync_lua_player(&self, lua_player: &mut LuaPlayer) {
        lua_player.player_data = self.player_data.clone();
        lua_player.current_level = match self.synced_level {
            Some(level) => level,
            None => self.current_level(&self.game_data.lock()) as u8,
        };
    }

    pub fn current_level(&self, game_data: &GameData) -> u16 {
        let index = game_data
            .get_exp_array_index(self.player_data.classjob.current_class as u16)
            .expect("Failed to find EXP array index?!");
        self.player_data.classjob.levels.0[index as usize]
    }

    pub fn set_current_level(&mut self, level: u16) {
        self.set_level_for(self.player_data.classjob.current_class as u8, level);
    }

    pub fn set_level_for(&mut self, classjob_id: u8, level: u16) {
        let game_data = self.game_data.lock();

        let index = game_data
            .get_exp_array_index(classjob_id as u16)
            .expect("Failed to find EXP array index?!");
        self.player_data.classjob.levels.0[index as usize] = level;
    }

    pub fn current_exp(&self, game_data: &GameData) -> i32 {
        let index = game_data
            .get_exp_array_index(self.player_data.classjob.current_class as u16)
            .expect("Failed to find EXP array index?!");
        self.player_data.classjob.exp.0[index as usize]
    }

    pub fn set_current_exp(&mut self, exp: i32) {
        let game_data = self.game_data.lock();

        let index = game_data
            .get_exp_array_index(self.player_data.classjob.current_class as u16)
            .expect("Failed to find EXP array index?!");
        self.player_data.classjob.exp.0[index as usize] = exp;
    }

    pub async fn update_hp_mp(&mut self, actor_id: ObjectId, hp: u32, mp: u16) {
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::UpdateHpMpTp {
            hp,
            mp,
            // Players' max MP is ParamGrow's MpModifier, 10000 at every level.
            max_mp: 10000,
        });

        self.send_ipc_from(actor_id, ipc).await;
    }

    /// Adds EXP to the current classjob, handles level-up and so on.
    /// Gives EXP that the rested bonus doesn't apply to, e.g. a quest's reward.
    pub async fn add_exp(&mut self, exp: i32) {
        self.gain_exp(exp, 0, false, NO_EXP_SOURCE).await;
    }

    /// Gives EXP from gathering, which the rested bonus applies to while it lasts, on top of `bonus_percent`.
    pub async fn add_exp_with_bonus(&mut self, exp: i32, bonus_percent: u32) {
        self.gain_exp(exp, bonus_percent as i32, true, NO_EXP_SOURCE)
            .await;
    }

    /// Gives EXP for defeating `source` (a kill chain's bonus in percent on top, and the rested bonus.)
    pub async fn add_kill_exp(&mut self, exp: i32, chain_percent: u32, source: ObjectId) {
        self.gain_exp(exp, chain_percent as i32, true, source).await;
    }

    async fn gain_exp(
        &mut self,
        base: i32,
        bonus_percent: i32,
        rested_applies: bool,
        source: ObjectId,
    ) {
        // As retail (Bob's Chronofoil captures): the chain's bonus on the base, then the rested part on top, the
        // message showing both bonuses as one percentage.
        let rested = if rested_applies {
            self.use_exp_bonus(base)
        } else {
            0
        };
        let exp = base + base * bonus_percent / 100 + rested;
        let shown_percent = bonus_percent + rested * 100 / base.max(1);

        self.actor_control_self(ActorControlCategory::EXPFloatingMessage {
            classjob_id: self.player_data.classjob.current_class as u32,
            amount: exp as u32,
            bonus_percent: shown_percent as u32,
            source_actor_id: source,
        })
        .await;

        let index;
        let mut level_up = 0;
        {
            let mut game_data = self.game_data.lock();

            index = game_data
                .get_exp_array_index(self.player_data.classjob.current_class as u16)
                .expect("Failed to find EXP array index?!");

            self.player_data.classjob.exp.0[index as usize] += exp;

            // Keep going until we have leftover EXP
            loop {
                let curr_exp = self.player_data.classjob.exp.0[index as usize];
                let max_exp = game_data
                    .get_max_exp(self.player_data.classjob.levels.0[index as usize] as u32);
                let difference = curr_exp - max_exp;
                if difference >= 0 {
                    level_up += 1;
                    self.player_data.classjob.exp.0[index as usize] = difference;
                } else {
                    break;
                }
            }
        }

        if level_up > 0 {
            let curr_level = self.player_data.classjob.levels.0[index as usize];
            let new_level = curr_level + level_up;
            self.set_current_level(new_level);

            // The client announces each level between this and the new one ("You attain level 2!" up to the new
            // level, with every action learned on the way) and sending 0 made it start from 1 (Bob's screenshot.) It's
            // most likely the previous level, which is what Sapphire sends third for its level-up effect.
            // The fourth value is the level the player is at now, the synced one when synced (retail.)
            let effective_level = self
                .synced_level
                .map(|level| level as u16)
                .unwrap_or(new_level);
            self.actor_control_self(ActorControlCategory::LevelUpMessage {
                classjob_id: self.player_data.classjob.current_class as u32,
                level: new_level as u32,
                unk2: curr_level as u32,
                unk3: effective_level as u32,
            })
            .await;
            self.send_stats().await;
            self.update_class_info().await;
            self.check_achievements(true).await;
        }

        // Retail only sends the new EXP (and the rested pool) after a gain; stats and the class info on a level-up.
        self.actor_control_self(ActorControlCategory::SetEXP {
            classjob_id: self.player_data.classjob.current_class as u32,
            amount: self.player_data.classjob.exp.0[index as usize] as u32,
            rested_exp: self.player_data.classjob.rested_exp as u32,
        })
        .await;
    }

    /// The number of seconds to add to the rested EXP bonus.
    pub async fn add_rested_exp_seconds(&mut self, seconds: i32) {
        self.player_data.classjob.rested_exp += seconds;
        self.player_data.classjob.rested_exp = self
            .player_data
            .classjob
            .rested_exp
            .clamp(0, MAXIMUM_RESTED_EXP);

        self.send_rested_exp().await;
    }

    /// Sends the rested EXP bonus to the client.
    pub async fn send_rested_exp(&mut self) {
        self.actor_control_self(ActorControlCategory::UpdateRestedExp {
            exp: self.player_data.classjob.rested_exp as u32,
        })
        .await;
    }

    /// Uses up rested EXP for `base` EXP earned, returning the extra EXP it gives (see `rested_exp_part`.)
    pub fn use_exp_bonus(&mut self, base: i32) -> i32 {
        if self.player_data.classjob.rested_exp <= 0 {
            return 0;
        }
        let exp_to_next = {
            let mut game_data = self.game_data.lock();
            let current_level = self.current_level(&game_data);
            game_data.get_max_exp(current_level as u32)
        };
        let (part, pool) = rested_exp_part(base, self.player_data.classjob.rested_exp, exp_to_next);
        self.player_data.classjob.rested_exp = pool;
        part
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The party bonus as retail's PlayerStats had it (Bob's Chronofoil captures): a White Mage's Mind 1891 and
    /// Vitality 1762 became 1928 and 1797 with two groups, 1947 and 1814 with three; two tanks gave 1 %, alone none.
    /// Food: a percentage of the stat up to its maximum, or a flat amount.
    #[test]
    fn food() {
        use crate::gamedata::FoodBonus;
        let mut parameters = BaseParameters {
            critical_hit: 1000,
            determination: 500,
            vitality: 400,
            ..Default::default()
        };
        parameters.add_food(&[
            // 10 % up to 50: capped.
            FoodBonus {
                base_param: 27,
                relative: true,
                value: 10,
                max: 50,
            },
            // 10 % up to 80: 50.
            FoodBonus {
                base_param: 44,
                relative: true,
                value: 10,
                max: 80,
            },
            // Flat 7.
            FoodBonus {
                base_param: 3,
                relative: false,
                value: 7,
                max: 0,
            },
        ]);
        assert_eq!(
            (
                parameters.critical_hit,
                parameters.determination,
                parameters.vitality
            ),
            (1050, 550, 407)
        );
    }

    #[test]
    fn the_party_bonus() {
        assert_eq!(party_bonus_percent(&[2]), 0);
        assert_eq!(party_bonus_percent(&[1, 1]), 1);
        assert_eq!(party_bonus_percent(&[2, 1]), 2);
        assert_eq!(party_bonus_percent(&[2, 1, 4, 4]), 3);
        assert_eq!(party_bonus_percent(&[1, 2, 3, 4, 5, 3, 4, 1]), 5);
        let mut parameters = BaseParameters {
            mind: 1891,
            vitality: 1762,
            piety: 1078,
            ..Default::default()
        };
        parameters.add_party_bonus(2);
        assert_eq!(
            (parameters.mind, parameters.vitality, parameters.piety),
            (1928, 1797, 1078)
        );
        let mut parameters = BaseParameters {
            mind: 1891,
            vitality: 1762,
            ..Default::default()
        };
        parameters.add_party_bonus(3);
        assert_eq!((parameters.mind, parameters.vitality), (1947, 1814));
    }

    /// Rested EXP as retail gave it (Bob's Chronofoil captures): a full pool's part, the seconds it uses, a short
    /// pool's part, an empty pool nothing.
    #[test]
    fn rested_exp() {
        // Full pool: floor(1.5 × base) − 1 in all.
        for (base, total) in [(95, 141), (105, 156), (115, 171), (120, 179), (125, 186)] {
            let (part, _) = rested_exp_part(base, 604_800, 6_000);
            assert_eq!(base + part, total);
        }
        // Level 10 (ExpToNext 5900): 141 EXP used 4817 seconds.
        assert_eq!(rested_exp_part(95, 604_800, 5_900), (46, 604_800 - 4_817));
        // A short pool: 1317 seconds at level 12 (ExpToNext 8620) gave 18.
        assert_eq!(rested_exp_part(120, 1_317, 8_620), (18, 0));
        assert_eq!(rested_exp_part(120, 0, 8_800), (0, 0));
    }

    /// Retail max HP (Bob's Chronofoil captures): his own (level, job HP modifier, Vitality, HP), and synced party
    /// members whose Vitality was rebuilt from capped gear.
    #[test]
    fn retail_max_hp_matches_captures() {
        // Conjurer (100) and White Mage (105).
        for (level, job, vitality, hp) in [
            (1, 100, 23, 102),
            (1, 105, 25, 118),
            (9, 105, 47, 243),
            (12, 100, 42, 217),
            (17, 105, 79, 429),
            (18, 105, 75, 409),
            (22, 105, 106, 607),
            (31, 105, 146, 969),
            (36, 105, 188, 1360),
            (50, 105, 332, 2926),
            (50, 105, 575, 5647),
            (80, 105, 1408, 22178),
            (80, 105, 1811, 29754),
            (87, 105, 1832, 35763),
            (90, 105, 3224, 72016),
        ] {
            assert_eq!(
                retail_max_hp(level, job, false, vitality),
                hp,
                "level {level}"
            );
        }
        // Samurai 54 (109), Reaper 70 (115); tanks (140): Gunbreaker 60, Paladin 67, Dark Knight 76.
        assert_eq!(retail_max_hp(54, 109, false, 550), 5626);
        assert_eq!(retail_max_hp(70, 115, false, 1249), 15353);
        assert_eq!(retail_max_hp(60, 140, true, 925), 14472);
        assert_eq!(retail_max_hp(67, 140, true, 931), 14561);
        assert_eq!(retail_max_hp(76, 140, true, 1509), 30573);
        // Synced party members at the gear cap: Dark Knight 80, Warrior (145) and Paladin 50.
        assert_eq!(retail_max_hp(80, 140, true, 1973), 46237);
        assert_eq!(retail_max_hp(50, 145, true, 640), 8819);
        assert_eq!(retail_max_hp(50, 140, true, 640), 8749);
    }

    /// A level 9 conjurer as a retail recording shows one: Mind 41, Strength 22, Determination 33, with a Maple Wand
    /// (11 physical and magic damage, 2.4s delay).
    fn retail_conjurer() -> BaseParameters {
        BaseParameters {
            retail_formula: true,
            level_main: level_main(9),
            level_div: 76,
            weapon_physical_damage: 11,
            weapon_magic_damage: 11,
            weapon_delay_ms: 2400,
            physical_stat_modifier: 50,
            primary_stat_modifier: 105,
            physical_damage: 22,
            attack_magic_potency: 41,
            healing_magic_potency: 41,
            determination: 33,
            ..Default::default()
        }
    }

    #[test]
    fn level_main_matches_retail() {
        // A fresh retail character's Determination at levels 1-10.
        let retail = [20, 21, 22, 24, 26, 27, 29, 31, 33, 35];
        for (level, main) in retail.iter().enumerate() {
            assert_eq!(level_main(level as u16 + 1), *main);
        }
        assert_eq!(level_main(90), 390);
        assert_eq!(level_main(100), 440);
    }

    #[test]
    fn damage_matches_retail() {
        let conjurer = retail_conjurer();
        // Retail: Stone (140) about 21.7 against level 9 enemies, Aero (50) about 8.4, Cure (450) about 72 (the means
        // after ±5 % variance and its rounding down, about half a point under the formula's.) At such small numbers
        // one point is 4 %: across levels 6-44 the formula is within 1-2 % (`docs/src/fork/combat.md`.)
        assert_eq!(conjurer.calc_magical_damage(140), 23);
        assert_eq!(conjurer.calc_magical_damage(50), 8);
        assert_eq!(conjurer.calc_heal_amount(450), 74);

        // Retail: a level 6 conjurer's auto-attack (Strength 19) hits for about 6.
        let level_6 = BaseParameters {
            level_main: level_main(6),
            level_div: 68,
            physical_damage: 19,
            determination: 27,
            ..conjurer
        };
        assert_eq!(level_6.calc_auto_attack_damage(90), 6);
    }

    /// Past level 50 attack power counts for far more than Sapphire's f(AP) gave: a White Mage synced to 80 with the
    /// same weapon did Glare for 3533.5 on average with 1557 Mind and 4507.1 with 1933 (196 and 23 hits, Bob's
    /// captures fefaf4cb and 10108225), 1.2755 times as much; Sapphire's f(AP) gave 1.166.
    #[test]
    fn attack_power_scales_as_retail_at_80() {
        let white_mage = |mind| BaseParameters {
            retail_formula: true,
            level: 80,
            level_main: level_main(80),
            level_div: 1300,
            weapon_magic_damage: 90,
            primary_stat_modifier: 115,
            attack_magic_potency: mind,
            determination: 714,
            ..Default::default()
        };
        let low = white_mage(1557).calc_magical_damage(290) as f32;
        let high = white_mage(1933).calc_magical_damage(290) as f32;
        assert!((high / low - 1.2755).abs() < 0.02, "{}", high / low);
    }

    /// Healing magic potency counts for less than attack power past 50: at 87 retail's Cure II (800) healed 2.0225
    /// times what Glare III (310) did, both with 1928 Mind and Maim and Mend II (Bob's capture 281f92fb: 10 and 17
    /// casts.) With one f(AP) for both it was the potencies' 2.58.
    #[test]
    fn healing_scales_as_retail_at_87() {
        let white_mage = BaseParameters {
            retail_formula: true,
            level: 87,
            level_main: level_main(87),
            level_div: 1700,
            weapon_magic_damage: 120,
            primary_stat_modifier: 115,
            attack_magic_potency: 1928,
            healing_magic_potency: 1928,
            determination: 749,
            ..Default::default()
        };
        let cure_ii = white_mage.calc_heal_amount(800 * 13 / 10) as f32;
        let glare_iii = white_mage.calc_magical_damage(310 * 13 / 10) as f32;
        assert!(
            (cure_ii / glare_iii - 2.0225).abs() < 0.03,
            "{}",
            cure_ii / glare_iii
        );
    }

    #[test]
    fn coefficients_between_anchors() {
        assert_eq!(anchored(&ATTACK_POWER_ANCHORS, 1), 75.0);
        assert_eq!(anchored(&ATTACK_POWER_ANCHORS, 50), 75.0);
        assert_eq!(anchored(&ATTACK_POWER_ANCHORS, 65), 120.0);
        assert_eq!(anchored(&ATTACK_POWER_ANCHORS, 87), 186.0);
        assert_eq!(anchored(&TANK_ATTACK_POWER_ANCHORS, 90), 156.0);
    }

    // RDM lv 52 as of 7.56h1
    #[test]
    fn test_jolt_cast_time() {
        let mut stat = BaseParameters::default();
        stat.level_modifier = 392;
        stat.base_speed = 344;
        stat.spell_speed = 385;
        stat.haste = 100;

        // 1.97 seconds
        assert_eq!(
            stat.get_adjusted_cast_time(ActionCategory::Spell, 2000),
            1974
        );
    }

    // The client's 1.5 s floor on sped-up recasts, and abilities untouched.
    #[test]
    fn test_recast_floor() {
        let mut stat = BaseParameters::default();
        stat.level_modifier = 2780;
        stat.base_speed = 420;
        stat.spell_speed = 420;
        stat.haste = 50;
        assert_eq!(
            stat.get_adjusted_recast_time(ActionCategory::Spell, 2500),
            1500
        );
        stat.haste = 100;
        assert_eq!(
            stat.get_adjusted_recast_time(ActionCategory::Spell, 2500),
            2500
        );
        assert_eq!(
            stat.get_adjusted_recast_time(ActionCategory::Ability, 60000),
            60000
        );
    }

    // THM lv 31 as of 7.56h1
    #[test]
    fn test_blizzard_cast_time() {
        let mut stat = BaseParameters::default();
        stat.level_modifier = 181;
        stat.base_speed = 181;
        stat.spell_speed = 181;
        stat.haste = 100;

        // 1.97 seconds
        assert_eq!(
            stat.get_adjusted_cast_time(ActionCategory::Spell, 2000),
            2000
        );
    }
}
