//! Disciples of the Land at a node: what each action does and the numbers behind a gather.
//!
//! The action list is the 7.57 Action sheet's Miner and Botanist rows; the statuses and the boon and Bountiful Yield
//! formulas are GatherBuddy Reborn's (Apache-2.0; `AutoGather/Helpers/RotationSolver.cs`, `AutoGather.Actions.cs`).
//! Nothing records how retail works out the gathering chance or the EXP: those are stand-ins (docs/src/fork/gathering.md,
//! known-issues.md).

/// Where a gatherer's own statuses come from (GatherBuddy Reborn `AutoGather.ActionIDs.cs`, the Status sheet.)
pub const STATUS_SHARP_VISION: u16 = 218;
pub const STATUS_KINGS_YIELD: u16 = 219;
pub const STATUS_CLEAR_VISION: u16 = 754;
pub const STATUS_BOUNTIFUL_YIELD: u16 = 756;
pub const STATUS_GIFT_II: u16 = 759;
pub const STATUS_TWELVES_BOUNTY: u16 = 825;
pub const STATUS_BOUNTIFUL_YIELD_II: u16 = 1286;
pub const STATUS_GIVING_LAND: u16 = 1802;
pub const STATUS_GIFT_I: u16 = 2666;
pub const STATUS_TIDINGS: u16 = 2667;
pub const STATUS_EUREKA_MOMENT: u16 = 2765;

/// The statuses that last only while the player is at the node: taken off when its window closes.
pub const NODE_STATUSES: [u16; 10] = [
    STATUS_SHARP_VISION,
    STATUS_KINGS_YIELD,
    STATUS_CLEAR_VISION,
    STATUS_BOUNTIFUL_YIELD,
    STATUS_GIFT_II,
    STATUS_TWELVES_BOUNTY,
    STATUS_BOUNTIFUL_YIELD_II,
    STATUS_GIVING_LAND,
    STATUS_GIFT_I,
    STATUS_TIDINGS,
];

/// The phase table's size: the client keeps one byte per point slot (QuestManager +0x7AC; 160 bytes in the packet.)
pub const PHASE_SLOTS: usize = 160;

/// What a gatherer's action does at a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatheringAction {
    /// +n% gathering chance until the node closes (Sharp Vision I-III / Field Mastery I-III.)
    SharpVision(u8),
    /// +15% gathering chance on the next gather.
    ClearVision,
    /// +n% Gatherer's Boon chance (Gift I and II stack.)
    Gift { status: u16, percent: u8 },
    /// +3 crystals (+2 before level 71) from crystal items.
    TwelvesBounty,
    /// +1 item on the next gather.
    BountifulYield,
    /// +1 to +3 items on the next gather, by Gathering against the item's base.
    BountifulYieldII,
    /// +1 gathering attempt; from level 90, half the time Eureka Moment too.
    SolidReason,
    /// +n items on each gather until the node closes (King's Yield I and II, Blessed Harvest I and II.)
    KingsYield(u8),
    /// Shows the node's hidden items.
    Luck,
    /// More crystals from crystal items.
    GivingLand,
    /// +1 item when the boon comes up.
    Tidings,
    /// +1 gathering attempt, for Eureka Moment.
    WiseToTheWorld,
}

impl GatheringAction {
    /// The action an Action row (Miner's or Botanist's) is at a node.
    pub fn from_id(id: u32) -> Option<Self> {
        use GatheringAction::*;
        Some(match id {
            235 | 218 => SharpVision(5),
            237 | 220 => SharpVision(15),
            295 | 294 => SharpVision(50),
            4072 | 4086 => ClearVision,
            21177 | 21178 => Gift {
                status: STATUS_GIFT_I,
                percent: 10,
            },
            25589 | 25590 => Gift {
                status: STATUS_GIFT_II,
                percent: 30,
            },
            280 | 282 => TwelvesBounty,
            4073 | 4087 => BountifulYield,
            272 | 273 => BountifulYieldII,
            232 | 215 => SolidReason,
            239 | 222 => KingsYield(1),
            241 | 224 => KingsYield(2),
            4081 | 4095 => Luck,
            4589 | 4590 => GivingLand,
            21203 | 21204 => Tidings,
            26521 | 26522 => WiseToTheWorld,
            _ => return None,
        })
    }

    /// The status it gives and that status' param, if it gives one.
    pub fn status(self) -> Option<(u16, u16)> {
        use GatheringAction::*;
        match self {
            SharpVision(percent) => Some((STATUS_SHARP_VISION, percent as u16)),
            ClearVision => Some((STATUS_CLEAR_VISION, 15)),
            Gift { status, percent } => Some((status, percent as u16)),
            TwelvesBounty => Some((STATUS_TWELVES_BOUNTY, 0)),
            BountifulYield => Some((STATUS_BOUNTIFUL_YIELD, 1)),
            BountifulYieldII => Some((STATUS_BOUNTIFUL_YIELD_II, 0)),
            KingsYield(n) => Some((STATUS_KINGS_YIELD, n as u16)),
            GivingLand => Some((STATUS_GIVING_LAND, 0)),
            Tidings => Some((STATUS_TIDINGS, 0)),
            SolidReason | Luck | WiseToTheWorld => None,
        }
    }
}

/// The field actions away from a node and the status each toggles (Prospect/Triangulate, Truth of Mountains/Forests,
/// Sneak; all permanent in the Status sheet.)
pub fn field_action_status(id: u32) -> Option<u16> {
    match id {
        227 => Some(225), // Prospect
        210 => Some(217), // Triangulate
        238 => Some(222), // Truth of Mountains
        221 => Some(221), // Truth of Forests
        303 | 304 => Some(47),
        _ => None,
    }
}

/// The Gatherer's Boon chance for this much Perception against the item's base (GatherBuddy Reborn's
/// `CalculateBoonChance`, integer maths), before Gifts and node bonuses.
pub fn boon_chance(perception: u32, base_perception: u32) -> u32 {
    if base_perception == 0 {
        return 0;
    }
    let score = (100 * perception / base_perception).min(150);
    if score >= 100 {
        (score - 100) * (60 - 35) / (150 - 100) + 35
    } else if score >= 80 {
        (score - 80) * (35 - 15) / (100 - 80) + 15
    } else if score >= 70 {
        (score - 70) * (15 - 10) / (80 - 70) + 10
    } else if score >= 60 {
        (score - 60) * 10 / (70 - 60)
    } else {
        0
    }
}

/// Bountiful Yield II's extra items (GatherBuddy Reborn's `CalculateBountifulBonus`): 3 with 110% of the item's base
/// Gathering, 2 with 90%, else 1.
pub fn bountiful_bonus(gathering: u32, base_gathering: u32) -> u32 {
    if gathering >= base_gathering * 11 / 10 {
        3
    } else if gathering >= base_gathering * 9 / 10 {
        2
    } else {
        1
    }
}

/// The gathering chance. A stand-in (no source has retail's formula): full with the item's base Gathering, falling
/// in proportion below it; then Sharp Vision and Clear Vision on top. An item that needs more Perception than the
/// gatherer has can't be gathered.
pub fn gathering_chance(
    gathering: u32,
    base_gathering: u32,
    perception: u32,
    perception_required: u32,
    bonus: u32,
) -> u32 {
    if perception < perception_required {
        return 0;
    }
    let base = (100 * gathering)
        .checked_div(base_gathering)
        .map_or(100, |chance| chance.min(100));
    (base + bonus).min(100)
}

/// GP back for each gather at a node: 5, 6 from level 80 (the Enhanced GP Regeneration II trait; GatherBuddy Reborn's
/// `GPRegenPerHit`.)
pub fn gp_per_gather(level: u8) -> u16 {
    if level >= 80 { 6 } else { 5 }
}

/// Whether an item is a shard, crystal or cluster (Item rows 2–19), which Twelve's Bounty and the Giving Land add to.
pub fn is_crystal(item_id: u32) -> bool {
    (2..=19).contains(&item_id)
}

/// The Eorzea time of day, in minutes, at a Unix time in seconds (one Eorzea hour is 175 s.)
pub fn eorzea_minute_of_day(unix_seconds: u64) -> u32 {
    ((unix_seconds * 60 / 175) % 1440) as u32
}

/// Whether a window of Eorzea minutes (start, end; the end may wrap past midnight) holds `minute`.
pub fn window_holds(window: (u32, u32), minute: u32) -> bool {
    let (start, end) = window;
    if start <= end {
        (start..end).contains(&minute)
    } else {
        minute >= start || minute < end
    }
}

/// The Eorzea minute of day the window that holds `minute` began at, to tell windows apart.
pub fn window_start(windows: &[(u32, u32)], minute: u32) -> Option<u32> {
    windows
        .iter()
        .find(|window| window_holds(**window, minute))
        .map(|window| window.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boon() {
        assert_eq!(boon_chance(0, 100), 0);
        assert_eq!(boon_chance(60, 100), 0);
        assert_eq!(boon_chance(70, 100), 10);
        assert_eq!(boon_chance(80, 100), 15);
        assert_eq!(boon_chance(100, 100), 35);
        assert_eq!(boon_chance(150, 100), 60);
        assert_eq!(boon_chance(400, 100), 60);
    }

    #[test]
    fn bountiful() {
        assert_eq!(bountiful_bonus(1100, 1000), 3);
        assert_eq!(bountiful_bonus(900, 1000), 2);
        assert_eq!(bountiful_bonus(899, 1000), 1);
    }

    #[test]
    fn chance() {
        assert_eq!(gathering_chance(500, 500, 0, 0, 0), 100);
        assert_eq!(gathering_chance(250, 500, 0, 0, 0), 50);
        assert_eq!(gathering_chance(250, 500, 0, 0, 15), 65);
        assert_eq!(gathering_chance(5000, 500, 10, 20, 50), 0);
    }

    #[test]
    fn eorzea_time() {
        // 175 s is one Eorzea hour; a day is 4200 s.
        assert_eq!(eorzea_minute_of_day(0), 0);
        assert_eq!(eorzea_minute_of_day(175), 60);
        assert_eq!(eorzea_minute_of_day(4200), 0);
        assert!(window_holds((22 * 60, 60), 23 * 60));
        assert!(window_holds((22 * 60, 60), 30));
        assert!(!window_holds((22 * 60, 60), 60));
        assert_eq!(window_start(&[(120, 240), (840, 960)], 900), Some(840));
    }

    #[test]
    fn actions() {
        assert_eq!(
            GatheringAction::from_id(241),
            Some(GatheringAction::KingsYield(2))
        );
        assert_eq!(
            GatheringAction::from_id(222).and_then(|a| a.status()),
            Some((STATUS_KINGS_YIELD, 1))
        );
        assert_eq!(GatheringAction::from_id(240), None);
        assert_eq!(field_action_status(210), Some(217));
    }
}
