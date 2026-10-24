//! Disciples of the Hand: a synthesis, step by step.
//!
//! Nothing here comes from Sapphire (its crafting is from before Endwalker, with TP and other actions). The rules are
//! the ones today's crafting tools agree on: raphael-rs (723b17e, 2026-10-18), Teamcraft's simulator (74e167a) and
//! Craftimizer (1d4a2c6), checked against the 7.57 client where it shows them (CP costs, the Synthesis window's
//! values). Where they disagree it follows Craftimizer and says so; docs/src/fork/crafting.md has the details and
//! known-issues.md what nobody knows yet (condition chances, EXP).

/// Inner Quiet, its stacks as the param (Craftimizer `SimulatorUtils.cs`, as are the other status IDs.)
pub const STATUS_INNER_QUIET: u16 = 251;
pub const STATUS_WASTE_NOT: u16 = 252;
pub const STATUS_GREAT_STRIDES: u16 = 254;
pub const STATUS_WASTE_NOT_II: u16 = 257;
pub const STATUS_MANIPULATION: u16 = 1164;
pub const STATUS_INNOVATION: u16 = 2189;
pub const STATUS_FINAL_APPRAISAL: u16 = 2190;
pub const STATUS_MUSCLE_MEMORY: u16 = 2191;
pub const STATUS_VENERATION: u16 = 2226;
pub const STATUS_HEART_AND_SOUL: u16 = 2665;
pub const STATUS_EXPEDIENCE: u16 = 3812;
pub const STATUS_TRAINED_PERFECTION: u16 = 3813;

/// Every status a synthesis can give, so they can all be taken off when it ends.
pub const CRAFTING_STATUSES: [u16; 12] = [
    STATUS_INNER_QUIET,
    STATUS_WASTE_NOT,
    STATUS_GREAT_STRIDES,
    STATUS_WASTE_NOT_II,
    STATUS_MANIPULATION,
    STATUS_INNOVATION,
    STATUS_FINAL_APPRAISAL,
    STATUS_MUSCLE_MEMORY,
    STATUS_VENERATION,
    STATUS_HEART_AND_SOUL,
    STATUS_EXPEDIENCE,
    STATUS_TRAINED_PERFECTION,
];

/// The chance of a high-quality result for each whole percent of the maximum quality (the same 101 numbers in
/// raphael's `HQ_LOOKUP`, Teamcraft's `HQ_TABLE` and Craftimizer; the client only shows what the server sends.)
const HQ_TABLE: [u8; 101] = [
    1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8,
    9, 9, 9, 10, 10, 10, 11, 11, 11, 12, 12, 12, 13, 13, 13, 14, 14, 14, 15, 15, 15, 16, 16, 17,
    17, 17, 18, 18, 18, 19, 19, 20, 20, 21, 22, 23, 24, 26, 28, 31, 34, 38, 42, 47, 52, 58, 64, 68,
    71, 74, 76, 78, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 94, 96, 98, 100,
];

/// The synthesis' condition, numbered as the client numbers it (`CraftCondition`, the scene's params 14 and 15).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CraftCondition {
    Normal = 1,
    Good = 2,
    Excellent = 3,
    Poor = 4,
    Centered = 5,
    Sturdy = 6,
    Pliant = 7,
    Malleable = 8,
    Primed = 9,
    GoodOmen = 10,
    Robust = 11,
}

impl CraftCondition {
    const ALL: [CraftCondition; 11] = [
        CraftCondition::Normal,
        CraftCondition::Good,
        CraftCondition::Excellent,
        CraftCondition::Poor,
        CraftCondition::Centered,
        CraftCondition::Sturdy,
        CraftCondition::Pliant,
        CraftCondition::Malleable,
        CraftCondition::Primed,
        CraftCondition::GoodOmen,
        CraftCondition::Robust,
    ];

    fn is_good(self) -> bool {
        matches!(self, CraftCondition::Good | CraftCondition::Excellent)
    }
}

/// What an action does, whichever class's row of it was used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CraftAction {
    BasicSynthesis,
    BasicTouch,
    MastersMend,
    StandardTouch,
    Observe,
    HastyTouch,
    RapidSynthesis,
    TricksOfTheTrade,
    PreciseTouch,
    MuscleMemory,
    CarefulSynthesis,
    PrudentTouch,
    AdvancedTouch,
    Reflect,
    PreparatoryTouch,
    Groundwork,
    DelicateSynthesis,
    IntensiveSynthesis,
    TrainedEye,
    ByregotsBlessing,
    HeartAndSoul,
    CarefulObservation,
    PrudentSynthesis,
    TrainedFinesse,
    RefinedTouch,
    DaringTouch,
    QuickInnovation,
    ImmaculateMend,
    TrainedPerfection,
    WasteNot,
    WasteNotII,
    Veneration,
    GreatStrides,
    Innovation,
    FinalAppraisal,
    Manipulation,
}

/// Each class's row of each action: CraftAction rows (from 100000) and the buffs' Action rows, one per class
/// (Teamcraft's simulator, `getIds()`; their order isn't always the classes', which doesn't matter here.)
const ACTION_IDS: &[(CraftAction, &[u32])] = &[
    (
        CraftAction::BasicSynthesis,
        &[
            100001, 100015, 100030, 100045, 100060, 100075, 100090, 100105,
        ],
    ),
    (
        CraftAction::BasicTouch,
        &[
            100002, 100016, 100031, 100046, 100061, 100076, 100091, 100106,
        ],
    ),
    (
        CraftAction::MastersMend,
        &[
            100003, 100017, 100032, 100047, 100062, 100077, 100092, 100107,
        ],
    ),
    (
        CraftAction::StandardTouch,
        &[
            100004, 100018, 100034, 100048, 100064, 100078, 100093, 100109,
        ],
    ),
    (
        CraftAction::Observe,
        &[
            100010, 100023, 100040, 100053, 100070, 100082, 100099, 100113,
        ],
    ),
];

/// Actions whose eight rows are consecutive, from the first (the Carpenter's) on.
const ACTION_RANGES: &[(CraftAction, u32)] = &[
    (CraftAction::PreciseTouch, 100128),
    (CraftAction::CarefulSynthesis, 100203),
    (CraftAction::PrudentTouch, 100227),
    (CraftAction::TrainedEye, 100283),
    (CraftAction::PreparatoryTouch, 100299),
    (CraftAction::IntensiveSynthesis, 100315),
    (CraftAction::DelicateSynthesis, 100323),
    (CraftAction::ByregotsBlessing, 100339),
    (CraftAction::HastyTouch, 100355),
    (CraftAction::RapidSynthesis, 100363),
    (CraftAction::TricksOfTheTrade, 100371),
    (CraftAction::MuscleMemory, 100379),
    (CraftAction::Reflect, 100387),
    (CraftAction::CarefulObservation, 100395),
    (CraftAction::Groundwork, 100403),
    (CraftAction::AdvancedTouch, 100411),
    (CraftAction::HeartAndSoul, 100419),
    (CraftAction::PrudentSynthesis, 100427),
    (CraftAction::TrainedFinesse, 100435),
    (CraftAction::RefinedTouch, 100443),
    (CraftAction::DaringTouch, 100451),
    (CraftAction::QuickInnovation, 100459),
    (CraftAction::ImmaculateMend, 100467),
    (CraftAction::TrainedPerfection, 100475),
    (CraftAction::GreatStrides, 260),
    (CraftAction::Manipulation, 4574),
    (CraftAction::WasteNot, 4631),
    (CraftAction::Innovation, 19004),
    (CraftAction::FinalAppraisal, 19012),
    (CraftAction::Veneration, 19297),
];

impl CraftAction {
    /// The action a CraftAction or Action row is, or None if it isn't a synthesis action.
    pub fn from_id(id: u32) -> Option<Self> {
        if let Some((action, _)) = ACTION_IDS.iter().find(|(_, ids)| ids.contains(&id)) {
            return Some(*action);
        }
        // Waste Not II's last two rows came later than the others.
        if (4639..=4644).contains(&id) || id == 19002 || id == 19003 {
            return Some(CraftAction::WasteNotII);
        }
        ACTION_RANGES
            .iter()
            .find(|(_, first)| (*first..*first + 8).contains(&id))
            .map(|(action, _)| *action)
    }

    /// The CP it costs before conditions and combos (the CraftAction sheet's Cost, the Action sheet's
    /// PrimaryCostValue: checked against both.)
    fn base_cp(self) -> i32 {
        use CraftAction::*;
        match self {
            BasicSynthesis | HastyTouch | RapidSynthesis | TricksOfTheTrade | HeartAndSoul
            | CarefulObservation | DaringTouch | QuickInnovation | TrainedPerfection => 0,
            BasicTouch | PreciseTouch | Groundwork | PrudentSynthesis | Veneration | Innovation => {
                18
            }
            MastersMend => 88,
            StandardTouch | DelicateSynthesis | TrainedFinesse | GreatStrides => 32,
            Observe | CarefulSynthesis => 7,
            MuscleMemory | Reflect | IntensiveSynthesis => 6,
            PrudentTouch => 25,
            AdvancedTouch => 46,
            PreparatoryTouch => 40,
            TrainedEye => 250,
            ByregotsBlessing | RefinedTouch => 24,
            ImmaculateMend => 112,
            WasteNot => 56,
            WasteNotII => 98,
            FinalAppraisal => 1,
            Manipulation => 96,
        }
    }

    /// The durability it costs before Waste Not and conditions.
    fn base_durability(self) -> i32 {
        use CraftAction::*;
        match self {
            PrudentTouch | PrudentSynthesis => 5,
            PreparatoryTouch | Groundwork => 20,
            MastersMend | Observe | TricksOfTheTrade | TrainedEye | HeartAndSoul
            | CarefulObservation | TrainedFinesse | QuickInnovation | ImmaculateMend
            | TrainedPerfection | WasteNot | WasteNotII | Veneration | GreatStrides
            | Innovation | FinalAppraisal | Manipulation => 0,
            _ => 10,
        }
    }

    fn success_rate(self) -> u32 {
        match self {
            CraftAction::HastyTouch | CraftAction::DaringTouch => 60,
            CraftAction::RapidSynthesis => 50,
            _ => 100,
        }
    }

    /// Whether it takes a step (and so ticks the buffs and changes the condition.)
    pub fn increases_step(self) -> bool {
        !matches!(
            self,
            CraftAction::FinalAppraisal
                | CraftAction::HeartAndSoul
                | CraftAction::QuickInnovation
                | CraftAction::CarefulObservation
        )
    }

    /// The level it's learned at, on the CRP row (the client hides those above the crafter's.)
    pub fn level(self) -> u8 {
        use CraftAction::*;
        match self {
            BasicSynthesis => 1,
            BasicTouch => 5,
            MastersMend => 7,
            HastyTouch | RapidSynthesis => 9,
            Observe | TricksOfTheTrade => 13,
            WasteNot | Veneration => 15,
            StandardTouch => 18,
            GreatStrides => 21,
            Innovation => 26,
            FinalAppraisal => 42,
            WasteNotII => 47,
            ByregotsBlessing => 50,
            PreciseTouch => 53,
            MuscleMemory => 54,
            CarefulObservation => 55,
            CarefulSynthesis => 62,
            Manipulation => 65,
            PrudentTouch => 66,
            AdvancedTouch => 68,
            Reflect => 69,
            PreparatoryTouch => 71,
            Groundwork => 72,
            DelicateSynthesis => 76,
            IntensiveSynthesis => 78,
            TrainedEye => 80,
            HeartAndSoul => 86,
            PrudentSynthesis => 88,
            TrainedFinesse => 90,
            RefinedTouch => 92,
            DaringTouch | QuickInnovation => 96,
            ImmaculateMend => 98,
            TrainedPerfection => 100,
        }
    }
}

/// Base progress or quality for a step of potency 100 (`stat*10/divider + add`, times the recipe level's modifier %
/// for crafters at or below its level), in exact integers: the simulators work it out in floats, and only Teamcraft's
/// matched the exact value everywhere.
pub fn base_gain(stat: u32, divider: u8, add: u32, modifier: Option<u8>) -> i32 {
    let stat = stat as i64;
    let divider = divider.max(1) as i64;
    let add = add as i64;
    match modifier {
        Some(modifier) => {
            let modifier = modifier as i64;
            ((stat * 10 * modifier + add * divider * modifier) / (divider * 100)) as i32
        }
        None => (stat * 10 / divider + add) as i32,
    }
}

/// What a synthesis is made of: the crafter's numbers and the recipe's.
#[derive(Debug, Clone, Copy)]
pub struct CraftSettings {
    /// The crafter's level.
    pub level: u8,
    /// The recipe level's ClassJobLevel (Trained Eye needs the crafter 10 above it.)
    pub recipe_level: u8,
    pub max_cp: i32,
    pub base_progress: i32,
    pub base_quality: i32,
    pub max_progress: i32,
    pub max_quality: i32,
    pub max_durability: i32,
    /// The recipe level's ConditionsFlag: bit n means condition n+1 can come up.
    pub conditions_flag: u16,
    pub is_expert: bool,
    pub can_hq: bool,
}

impl CraftSettings {
    /// The settings for a crafter with these stats at this level making `recipe`.
    pub fn new(
        recipe: &crate::gamedata::RecipeInfo,
        level: u8,
        craftsmanship: u32,
        control: u32,
        cp: u32,
    ) -> Self {
        let modified = level <= recipe.class_job_level;
        Self {
            level,
            recipe_level: recipe.class_job_level,
            max_cp: cp as i32,
            base_progress: base_gain(
                craftsmanship,
                recipe.progress_divider,
                2,
                modified.then_some(recipe.progress_modifier),
            ),
            base_quality: base_gain(
                control,
                recipe.quality_divider,
                35,
                modified.then_some(recipe.quality_modifier),
            ),
            max_progress: recipe.max_progress as i32,
            max_quality: recipe.max_quality as i32,
            max_durability: recipe.max_durability as i32,
            conditions_flag: recipe.conditions_flag,
            is_expert: recipe.is_expert,
            can_hq: recipe.can_hq,
        }
    }
}

/// The timed buffs, in steps left.
#[derive(Debug, Clone, Copy, Default)]
struct Buffs {
    waste_not: u8,
    waste_not_ii: u8,
    veneration: u8,
    great_strides: u8,
    innovation: u8,
    final_appraisal: u8,
    muscle_memory: u8,
    manipulation: u8,
    expedience: u8,
}

impl Buffs {
    fn tick(&mut self) {
        for steps in [
            &mut self.waste_not,
            &mut self.waste_not_ii,
            &mut self.veneration,
            &mut self.great_strides,
            &mut self.innovation,
            &mut self.final_appraisal,
            &mut self.muscle_memory,
            &mut self.manipulation,
            &mut self.expedience,
        ] {
            *steps = steps.saturating_sub(1);
        }
    }
}

/// What the last action leaves for the next one's combo (the scene's StateFlags.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Combo {
    #[default]
    None,
    BasicTouch,
    AdvancedTouch,
}

/// What one action did, for the step's scene.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StepOutcome {
    pub success: bool,
    pub cp_recovered: i32,
    pub progress_delta: i32,
    pub quality_delta: i32,
    pub durability_delta: i32,
    /// The condition it was used under.
    pub used_condition: u8,
    /// The statuses that ran out or were used up, with the param each had.
    pub worn_off: Vec<(u16, u16)>,
}

/// A synthesis in progress.
#[derive(Debug, Clone)]
pub struct Craft {
    pub settings: CraftSettings,
    /// Steps taken so far (the window shows this plus one.)
    pub steps: u32,
    pub progress: i32,
    pub quality: i32,
    pub durability: i32,
    pub cp: i32,
    pub condition: CraftCondition,
    inner_quiet: u8,
    buffs: Buffs,
    combo: Combo,
    careful_observations: u8,
    heart_and_soul: bool,
    heart_and_soul_used: bool,
    quick_innovation_used: bool,
    trained_perfection: bool,
    trained_perfection_used: bool,
}

impl Craft {
    pub fn new(settings: CraftSettings, initial_quality: i32) -> Self {
        Self {
            settings,
            steps: 0,
            progress: 0,
            quality: initial_quality.min(settings.max_quality),
            durability: settings.max_durability,
            cp: settings.max_cp,
            condition: CraftCondition::Normal,
            inner_quiet: 0,
            buffs: Buffs::default(),
            combo: Combo::None,
            careful_observations: 0,
            heart_and_soul: false,
            heart_and_soul_used: false,
            quick_innovation_used: false,
            trained_perfection: false,
            trained_perfection_used: false,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.progress >= self.settings.max_progress
    }

    pub fn is_failed(&self) -> bool {
        !self.is_complete() && self.durability <= 0
    }

    pub fn is_over(&self) -> bool {
        self.is_complete() || self.durability <= 0
    }

    /// The chance of a high-quality result, as the window shows it.
    pub fn hq_chance(&self) -> u32 {
        if !self.settings.can_hq {
            return 0;
        }
        hq_chance(self.quality, self.settings.max_quality)
    }

    /// What it costs in CP now: the client's own rule (`0x140EA4440` in 7.57): Standard and Advanced Touch cost 18
    /// in their combos, Pliant halves (rounding up).
    pub fn cp_cost(&self, action: CraftAction) -> i32 {
        let mut cost = action.base_cp();
        if (action == CraftAction::StandardTouch && self.combo == Combo::BasicTouch)
            || (action == CraftAction::AdvancedTouch && self.combo == Combo::AdvancedTouch)
        {
            cost = 18;
        }
        if self.condition == CraftCondition::Pliant {
            cost = (cost + 1) / 2;
        }
        cost
    }

    fn durability_cost(&self, action: CraftAction) -> i32 {
        let base = action.base_durability();
        if base == 0 {
            return 0;
        }
        if self.trained_perfection {
            return 0;
        }
        let mut divider = 1;
        if self.buffs.waste_not > 0 || self.buffs.waste_not_ii > 0 {
            divider *= 2;
        }
        if matches!(
            self.condition,
            CraftCondition::Sturdy | CraftCondition::Robust
        ) {
            divider *= 2;
        }
        (base + divider - 1) / divider
    }

    /// Whether the action can be used now.
    pub fn can_use(&self, action: CraftAction) -> bool {
        use CraftAction::*;
        if self.is_over() || self.settings.level < action.level() || self.cp < self.cp_cost(action)
        {
            return false;
        }
        let good = self.condition.is_good() || self.heart_and_soul;
        match action {
            MuscleMemory | Reflect => self.steps == 0,
            TrainedEye => {
                self.steps == 0
                    && !self.settings.is_expert
                    && self.settings.level >= self.settings.recipe_level.saturating_add(10)
            }
            TricksOfTheTrade | PreciseTouch | IntensiveSynthesis => good,
            ByregotsBlessing => self.inner_quiet > 0,
            TrainedFinesse => self.inner_quiet == 10,
            PrudentTouch | PrudentSynthesis => {
                self.buffs.waste_not == 0 && self.buffs.waste_not_ii == 0
            }
            DaringTouch => self.buffs.expedience > 0,
            HeartAndSoul => !self.heart_and_soul_used,
            QuickInnovation => !self.quick_innovation_used && self.buffs.innovation == 0,
            TrainedPerfection => !self.trained_perfection_used,
            CarefulObservation => self.careful_observations < 3,
            _ => true,
        }
    }

    fn add_progress(&mut self, potency: i32) {
        let mut buff = 100;
        if self.buffs.muscle_memory > 0 {
            buff += 100;
            self.buffs.muscle_memory = 0;
        }
        if self.buffs.veneration > 0 {
            buff += 50;
        }
        let condition = if self.condition == CraftCondition::Malleable {
            150
        } else {
            100
        };
        let gain =
            self.settings.base_progress as i64 * potency as i64 * condition * buff / 1_000_000;
        self.progress += gain as i32;
        if self.buffs.final_appraisal > 0 && self.progress >= self.settings.max_progress {
            self.progress = self.settings.max_progress - 1;
            self.buffs.final_appraisal = 0;
        }
    }

    fn add_quality(&mut self, potency: i32) {
        let mut buff = 100;
        if self.buffs.great_strides > 0 {
            buff += 100;
            self.buffs.great_strides = 0;
        }
        if self.buffs.innovation > 0 {
            buff += 50;
        }
        let condition = match self.condition {
            CraftCondition::Poor => 50,
            CraftCondition::Good => 150,
            CraftCondition::Excellent => 400,
            _ => 100,
        };
        let inner_quiet = 100 + 10 * self.inner_quiet as i64;
        let gain =
            self.settings.base_quality as i64 * potency as i64 * condition * inner_quiet * buff
                / 100_000_000;
        self.add_quality_raw(gain as i32);
    }

    fn add_quality_raw(&mut self, gain: i32) {
        self.quality += gain;
        if self.settings.level >= 11 {
            self.add_inner_quiet();
        }
    }

    fn add_inner_quiet(&mut self) {
        self.inner_quiet = (self.inner_quiet + 1).min(10);
    }

    /// Adds a buff: its steps plus the one about to be taken, two more under Primed.
    fn buff_steps(&self, action: CraftAction, steps: u8) -> u8 {
        let mut steps = steps + action.increases_step() as u8;
        if self.condition == CraftCondition::Primed {
            steps += 2;
        }
        steps
    }

    /// The good condition was what an action needed, or Heart and Soul stood in for it (and is used up.)
    fn use_up_heart_and_soul(&mut self) {
        if !self.condition.is_good() {
            self.heart_and_soul = false;
        }
    }

    fn next_condition(&mut self, rng: &mut fastrand::Rng) {
        self.condition = match self.condition {
            CraftCondition::Poor | CraftCondition::Good => CraftCondition::Normal,
            CraftCondition::Excellent => CraftCondition::Poor,
            CraftCondition::GoodOmen => CraftCondition::Good,
            CraftCondition::Robust => CraftCondition::Sturdy,
            _ => self.random_condition(rng),
        };
    }

    /// One of the conditions the recipe allows, with Craftimizer's chances (Teamcraft's differ; retail's aren't known.)
    fn random_condition(&self, rng: &mut fastrand::Rng) -> CraftCondition {
        let mut roll = rng.f32();
        for condition in CraftCondition::ALL {
            if self.settings.conditions_flag & (1 << (condition as u8 - 1)) == 0 {
                continue;
            }
            let chance = match condition {
                CraftCondition::Good if self.settings.is_expert => 0.12,
                CraftCondition::Good if self.settings.level >= 63 => 0.15,
                CraftCondition::Good => 0.18,
                CraftCondition::Excellent => 0.04,
                CraftCondition::Centered | CraftCondition::Sturdy | CraftCondition::Primed => 0.15,
                CraftCondition::Pliant | CraftCondition::Robust => 0.10,
                CraftCondition::Malleable => 0.13,
                CraftCondition::GoodOmen => 0.12,
                _ => 0.0,
            };
            roll -= chance;
            if roll < 0.0 {
                return condition;
            }
        }
        CraftCondition::Normal
    }

    /// Uses an action. None if it can't be used now (and nothing changes.)
    pub fn use_action(
        &mut self,
        action: CraftAction,
        rng: &mut fastrand::Rng,
    ) -> Option<StepOutcome> {
        use CraftAction::*;
        if !self.can_use(action) {
            return None;
        }

        let before = self.statuses();
        let (progress, quality, durability) = (self.progress, self.quality, self.durability);
        let used_condition = self.condition;
        let cp_cost = self.cp_cost(action);
        let durability_cost = self.durability_cost(action);

        let mut rate = action.success_rate();
        if self.condition == CraftCondition::Centered {
            rate += 25;
        }
        let success = rate >= 100 || rng.u32(0..100) < rate;

        let mut cp_recovered = 0;
        if success {
            match action {
                BasicSynthesis => {
                    self.add_progress(if self.settings.level >= 31 { 120 } else { 100 })
                }
                RapidSynthesis => {
                    self.add_progress(if self.settings.level >= 63 { 500 } else { 250 })
                }
                CarefulSynthesis => {
                    self.add_progress(if self.settings.level >= 82 { 180 } else { 150 })
                }
                Groundwork => {
                    let mut potency = if self.settings.level >= 86 { 360 } else { 300 };
                    if self.durability < durability_cost {
                        potency /= 2;
                    }
                    self.add_progress(potency);
                }
                IntensiveSynthesis => {
                    self.add_progress(400);
                    self.use_up_heart_and_soul();
                }
                PrudentSynthesis => self.add_progress(180),
                MuscleMemory => {
                    self.add_progress(300);
                    self.buffs.muscle_memory = self.buff_steps(action, 5);
                }
                DelicateSynthesis => {
                    self.add_progress(if self.settings.level >= 94 { 150 } else { 100 });
                    self.add_quality(100);
                }
                BasicTouch | PrudentTouch => self.add_quality(100),
                StandardTouch => self.add_quality(125),
                AdvancedTouch | DaringTouch => self.add_quality(150),
                HastyTouch => {
                    self.add_quality(100);
                    if self.settings.level >= 96 {
                        self.buffs.expedience = self.buff_steps(action, 1);
                    }
                }
                PreciseTouch => {
                    self.add_quality(150);
                    self.add_inner_quiet();
                    self.use_up_heart_and_soul();
                }
                Reflect => {
                    self.add_quality(300);
                    self.add_inner_quiet();
                }
                PreparatoryTouch => {
                    self.add_quality(200);
                    self.add_inner_quiet();
                }
                RefinedTouch => {
                    self.add_quality(100);
                    if self.combo == Combo::BasicTouch {
                        self.add_inner_quiet();
                    }
                }
                TrainedFinesse => self.add_quality(100),
                ByregotsBlessing => {
                    self.add_quality(100 + 20 * self.inner_quiet as i32);
                    self.inner_quiet = 0;
                }
                TrainedEye => {
                    let missing = self.settings.max_quality - self.quality;
                    self.add_quality_raw(missing);
                }
                TricksOfTheTrade => {
                    cp_recovered = 20.min(self.settings.max_cp - self.cp);
                    self.cp += cp_recovered;
                    self.use_up_heart_and_soul();
                }
                MastersMend => {
                    self.durability = (self.durability + 30).min(self.settings.max_durability)
                }
                ImmaculateMend => self.durability = self.settings.max_durability,
                Observe => {}
                HeartAndSoul => self.heart_and_soul = true,
                CarefulObservation => {}
                QuickInnovation => self.buffs.innovation = self.buff_steps(action, 1),
                TrainedPerfection => self.trained_perfection = true,
                WasteNot => {
                    self.buffs.waste_not = self.buff_steps(action, 4);
                    self.buffs.waste_not_ii = 0;
                }
                WasteNotII => {
                    self.buffs.waste_not_ii = self.buff_steps(action, 8);
                    self.buffs.waste_not = 0;
                }
                Veneration => self.buffs.veneration = self.buff_steps(action, 4),
                GreatStrides => self.buffs.great_strides = self.buff_steps(action, 3),
                Innovation => self.buffs.innovation = self.buff_steps(action, 4),
                FinalAppraisal => self.buffs.final_appraisal = self.buff_steps(action, 5),
                Manipulation => self.buffs.manipulation = self.buff_steps(action, 8),
            }
        }

        self.cp -= cp_cost;
        if action.base_durability() > 0 {
            self.durability -= durability_cost;
            // Trained Perfection is used up by the first action that would have cost durability.
            self.trained_perfection = false;
        }

        if action.increases_step() {
            // Manipulation mends at the end of every step but the one it's used on, if the craft is still going.
            if self.durability > 0
                && !self.is_complete()
                && self.buffs.manipulation > 0
                && action != Manipulation
            {
                self.durability = (self.durability + 5).min(self.settings.max_durability);
            }
            self.steps += 1;
            self.next_condition(rng);
            self.buffs.tick();
        } else if action == CarefulObservation {
            self.next_condition(rng);
        }

        self.combo = match action {
            BasicTouch => Combo::BasicTouch,
            StandardTouch if self.combo == Combo::BasicTouch => Combo::AdvancedTouch,
            Observe => Combo::AdvancedTouch,
            _ => Combo::None,
        };
        match action {
            CarefulObservation => self.careful_observations += 1,
            HeartAndSoul => self.heart_and_soul_used = true,
            QuickInnovation => self.quick_innovation_used = true,
            TrainedPerfection => self.trained_perfection_used = true,
            _ => {}
        }

        let after = self.statuses();
        let worn_off = before
            .into_iter()
            .filter(|(id, _)| !after.iter().any(|(other, _)| other == id))
            .collect();

        Some(StepOutcome {
            success,
            cp_recovered,
            progress_delta: self.progress - progress,
            quality_delta: self.quality - quality,
            durability_delta: self.durability - durability,
            used_condition: used_condition as u8,
            worn_off,
        })
    }

    /// The statuses the crafter has now, each with its param: steps left, or Inner Quiet's stacks.
    pub fn statuses(&self) -> Vec<(u16, u16)> {
        let mut statuses = Vec::new();
        let mut push = |id: u16, param: u8, active: bool| {
            if active {
                statuses.push((id, param as u16));
            }
        };
        push(STATUS_INNER_QUIET, self.inner_quiet, self.inner_quiet > 0);
        let buffs = self.buffs;
        push(STATUS_WASTE_NOT, buffs.waste_not, buffs.waste_not > 0);
        push(
            STATUS_WASTE_NOT_II,
            buffs.waste_not_ii,
            buffs.waste_not_ii > 0,
        );
        push(STATUS_VENERATION, buffs.veneration, buffs.veneration > 0);
        push(
            STATUS_GREAT_STRIDES,
            buffs.great_strides,
            buffs.great_strides > 0,
        );
        push(STATUS_INNOVATION, buffs.innovation, buffs.innovation > 0);
        push(
            STATUS_FINAL_APPRAISAL,
            buffs.final_appraisal,
            buffs.final_appraisal > 0,
        );
        push(
            STATUS_MUSCLE_MEMORY,
            buffs.muscle_memory,
            buffs.muscle_memory > 0,
        );
        push(
            STATUS_MANIPULATION,
            buffs.manipulation,
            buffs.manipulation > 0,
        );
        push(STATUS_EXPEDIENCE, buffs.expedience, buffs.expedience > 0);
        push(STATUS_HEART_AND_SOUL, 0, self.heart_and_soul);
        push(STATUS_TRAINED_PERFECTION, 0, self.trained_perfection);
        statuses
    }

    /// The scene's StateFlags (param 17), read by the client for the window and the CP costs.
    pub fn state_flags(&self, success: bool) -> u32 {
        let mut flags = 0;
        if self.steps > 0 {
            flags |= 0x2;
        }
        if self.is_complete() {
            flags |= 0x4;
        } else if self.is_failed() {
            flags |= 0x8;
        }
        if success {
            flags |= 0x10;
        }
        match self.combo {
            Combo::BasicTouch => flags |= 0x0800_0000,
            Combo::AdvancedTouch => flags |= 0x1000_0000,
            Combo::None => {}
        }
        if self.careful_observations >= 3 {
            flags |= 0x4000_0000;
        }
        if self.heart_and_soul_used {
            flags |= 0x8000_0000;
        }
        flags
    }
}

/// The chance of a high-quality result for this much quality out of the maximum.
pub fn hq_chance(quality: i32, max_quality: i32) -> u32 {
    if quality <= 0 || max_quality <= 0 {
        return 1;
    }
    if quality >= max_quality {
        return 100;
    }
    HQ_TABLE[(quality as i64 * 100 / max_quality as i64) as usize] as u32
}

/// The quality high-quality materials start a synthesis with (raphael's `get_initial_quality`): the recipe's
/// MaterialQualityFactor % of the maximum, shared by item level among the ingredients that can be HQ.
/// `ingredients` is (amount, item level, can be HQ, how many HQ the crafter put in).
pub fn initial_quality(
    max_quality: u32,
    material_quality_factor: u8,
    ingredients: &[(u32, u32, bool, u32)],
) -> u32 {
    let mut max_ilvl = 0u64;
    let mut provided_ilvl = 0u64;
    for (amount, item_level, can_be_hq, hq) in ingredients {
        if !can_be_hq {
            continue;
        }
        max_ilvl += *amount as u64 * *item_level as u64;
        provided_ilvl += (*hq).min(*amount) as u64 * *item_level as u64;
    }
    if max_ilvl == 0 {
        return 0;
    }
    (max_quality as u64 * material_quality_factor as u64 * provided_ilvl / max_ilvl / 100) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A level 100 crafter on recipe level 690 (raphael's rlvl table: 6600 progress, 12000 quality, 80
    /// durability, dividers 170/150, modifiers 90/75), 5000 craftsmanship, 4800 control, 600 CP.
    fn settings() -> CraftSettings {
        CraftSettings {
            level: 100,
            recipe_level: 100,
            max_cp: 600,
            base_progress: base_gain(5000, 170, 2, Some(90)),
            base_quality: base_gain(4800, 150, 35, Some(75)),
            max_progress: 6600,
            max_quality: 12000,
            max_durability: 80,
            conditions_flag: 0b1111,
            is_expert: false,
            can_hq: true,
        }
    }

    #[test]
    fn base_gains() {
        // 5000*10/170 + 2 = 296.1..., *0.9 = 266.5...; 4800*10/150 + 35 = 355, *0.75 = 266.25.
        assert_eq!(base_gain(5000, 170, 2, Some(90)), 266);
        assert_eq!(base_gain(4800, 150, 35, Some(75)), 266);
        assert_eq!(base_gain(5000, 170, 2, None), 296);
        // Teamcraft's exact value where raphael's f32 is one lower (div 150, mod 75; research/scripts/rounding.py.)
        assert_eq!(
            base_gain(51, 170, 2, Some(80)),
            (51 * 10 * 80 + 2 * 170 * 80) / (170 * 100)
        );
    }

    #[test]
    fn action_ids() {
        assert_eq!(
            CraftAction::from_id(100001),
            Some(CraftAction::BasicSynthesis)
        );
        assert_eq!(
            CraftAction::from_id(100105),
            Some(CraftAction::BasicSynthesis)
        );
        assert_eq!(
            CraftAction::from_id(100482),
            Some(CraftAction::TrainedPerfection)
        );
        assert_eq!(CraftAction::from_id(19003), Some(CraftAction::WasteNotII));
        assert_eq!(CraftAction::from_id(267), Some(CraftAction::GreatStrides));
        assert_eq!(CraftAction::from_id(268), None);
        assert_eq!(CraftAction::from_id(7), None);
    }

    #[test]
    fn a_rotation() {
        let mut rng = fastrand::Rng::with_seed(1);
        let mut craft = Craft::new(
            CraftSettings {
                conditions_flag: 1,
                max_durability: 200,
                ..settings()
            },
            0,
        );

        // Muscle Memory: 266*300/100 = 798 progress, then +100% on the next progress action for 5 steps.
        let outcome = craft
            .use_action(CraftAction::MuscleMemory, &mut rng)
            .unwrap();
        assert_eq!(outcome.progress_delta, 798);
        assert_eq!(outcome.durability_delta, -10);
        assert_eq!(craft.cp, 594);
        assert!(craft.statuses().contains(&(STATUS_MUSCLE_MEMORY, 5)));

        // Veneration (+50%) doesn't take Muscle Memory away and costs no durability.
        craft.use_action(CraftAction::Veneration, &mut rng).unwrap();
        assert!(craft.statuses().contains(&(STATUS_VENERATION, 4)));
        assert!(craft.statuses().contains(&(STATUS_MUSCLE_MEMORY, 4)));

        // Groundwork: 266*360*(100+100+50)/10000 = 2394; Muscle Memory used up and listed as worn off.
        let outcome = craft.use_action(CraftAction::Groundwork, &mut rng).unwrap();
        assert_eq!(outcome.progress_delta, 2394);
        assert_eq!(outcome.durability_delta, -20);
        assert_eq!(outcome.worn_off, vec![(STATUS_MUSCLE_MEMORY, 4)]);

        // Basic Touch then Standard Touch at 18 CP, then Advanced Touch at 18.
        craft.use_action(CraftAction::BasicTouch, &mut rng).unwrap();
        assert_eq!(craft.state_flags(true) & 0x0800_0000, 0x0800_0000);
        let cp = craft.cp;
        let outcome = craft
            .use_action(CraftAction::StandardTouch, &mut rng)
            .unwrap();
        assert_eq!(cp - craft.cp, 18);
        // Inner Quiet 1 from Basic Touch: 266*125*110/10000 = 365.
        assert_eq!(outcome.quality_delta, 365);
        let cp = craft.cp;
        craft
            .use_action(CraftAction::AdvancedTouch, &mut rng)
            .unwrap();
        assert_eq!(cp - craft.cp, 18);
        assert!(craft.statuses().contains(&(STATUS_INNER_QUIET, 3)));

        // Byregot's needs Inner Quiet, uses it up.
        let outcome = craft
            .use_action(CraftAction::ByregotsBlessing, &mut rng)
            .unwrap();
        assert_eq!(outcome.quality_delta, 266 * 160 * 130 / 10000);
        assert!(!craft.can_use(CraftAction::ByregotsBlessing));
        assert_eq!(outcome.worn_off, vec![(STATUS_INNER_QUIET, 3)]);

        // Finish it.
        while !craft.is_complete() {
            craft
                .use_action(CraftAction::CarefulSynthesis, &mut rng)
                .unwrap();
        }
        assert!(craft.state_flags(true) & 0x4 != 0);
        assert!(
            craft
                .use_action(CraftAction::BasicSynthesis, &mut rng)
                .is_none()
        );
    }

    #[test]
    fn durability_and_failure() {
        let mut rng = fastrand::Rng::with_seed(2);
        let mut craft = Craft::new(
            CraftSettings {
                conditions_flag: 1,
                max_durability: 20,
                ..settings()
            },
            0,
        );
        craft.use_action(CraftAction::WasteNot, &mut rng).unwrap();
        // Prudent Touch can't be used under Waste Not.
        assert!(!craft.can_use(CraftAction::PrudentTouch));
        // Preparatory Touch: 20 halved.
        let outcome = craft
            .use_action(CraftAction::PreparatoryTouch, &mut rng)
            .unwrap();
        assert_eq!(outcome.durability_delta, -10);
        // Trained Perfection: the next one is free.
        craft
            .use_action(CraftAction::TrainedPerfection, &mut rng)
            .unwrap();
        let outcome = craft.use_action(CraftAction::Groundwork, &mut rng).unwrap();
        assert_eq!(outcome.durability_delta, 0);
        assert!(!craft.can_use(CraftAction::TrainedPerfection));
        // Two more Basic Touches (5 each under Waste Not) break it.
        craft.use_action(CraftAction::BasicTouch, &mut rng).unwrap();
        craft.use_action(CraftAction::BasicTouch, &mut rng).unwrap();
        assert!(craft.is_failed());
        assert!(craft.state_flags(true) & 0x8 != 0);
    }

    #[test]
    fn manipulation_and_final_appraisal() {
        let mut rng = fastrand::Rng::with_seed(3);
        let mut craft = Craft::new(
            CraftSettings {
                conditions_flag: 1,
                max_progress: 2000,
                ..settings()
            },
            0,
        );
        craft
            .use_action(CraftAction::Manipulation, &mut rng)
            .unwrap();
        assert_eq!(craft.durability, 80);
        craft.use_action(CraftAction::BasicTouch, &mut rng).unwrap();
        assert_eq!(craft.durability, 75);

        craft
            .use_action(CraftAction::FinalAppraisal, &mut rng)
            .unwrap();
        // No step: Manipulation still has 7 left.
        assert!(craft.statuses().contains(&(STATUS_MANIPULATION, 7)));
        assert!(craft.statuses().contains(&(STATUS_FINAL_APPRAISAL, 5)));
        while craft.progress < 1999 {
            craft
                .use_action(CraftAction::CarefulSynthesis, &mut rng)
                .unwrap();
        }
        assert_eq!(craft.progress, 1999);
        assert!(!craft.is_complete());
    }

    #[test]
    fn hq_and_initial_quality() {
        assert_eq!(hq_chance(0, 100), 1);
        assert_eq!(hq_chance(50, 100), 15);
        assert_eq!(hq_chance(99, 100), 98);
        assert_eq!(hq_chance(100, 100), 100);
        // Two of three ingredient lots HQ by item level: 12000*50% * 2/3.
        assert_eq!(
            initial_quality(
                12000,
                50,
                &[(2, 10, true, 2), (1, 20, true, 0), (3, 1, false, 0)]
            ),
            12000 * 50 * 20 / 40 / 100
        );
    }

    #[test]
    fn pliant_and_conditions() {
        let mut craft = Craft::new(settings(), 0);
        craft.condition = CraftCondition::Pliant;
        assert_eq!(craft.cp_cost(CraftAction::Manipulation), 48);
        assert_eq!(craft.cp_cost(CraftAction::ByregotsBlessing), 12);
        assert_eq!(craft.cp_cost(CraftAction::FinalAppraisal), 1);

        // A normal recipe (flag 15) only ever rolls Normal, Good and Excellent, then Poor after Excellent.
        let mut rng = fastrand::Rng::with_seed(4);
        let mut seen = [false; 12];
        let mut craft = Craft::new(
            CraftSettings {
                max_progress: i32::MAX,
                max_durability: 100_000,
                ..settings()
            },
            0,
        );
        for _ in 0..2000 {
            let before = craft.condition;
            craft
                .use_action(CraftAction::BasicSynthesis, &mut rng)
                .unwrap();
            if before == CraftCondition::Excellent {
                assert_eq!(craft.condition, CraftCondition::Poor);
            }
            seen[craft.condition as usize] = true;
        }
        assert!(seen[1] && seen[2] && seen[3] && seen[4]);
        assert!(!seen[5..].iter().any(|seen| *seen));
    }
}
