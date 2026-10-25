//! Gear wear: condition (durability) lost and spiritbond gained by what the player wears, materia extraction from
//! fully spiritbound gear, and self-repair with Dark Matter. The rules only; the packets are
//! `zone_connection/spiritbond.rs` and `events/materialize.rs` (docs/src/fork/spiritbond.md.)
//!
//! Sources: Bob's Chronofoil captures (every UpdateInventorySlot batch that changed worn gear: deaths, syntheses,
//! gathers, duty and FATE completions; 2026-06 to 2026-10), and for what they don't show the wiki's Spiritbond,
//! Gear Durability, Materia and Materia Extraction pages (The Balance's research, patch 7.0.)

use kawari::common::ITEM_CONDITION_MAX;

/// 100% spiritbond.
pub const SPIRITBOND_MAX: u16 = 10_000;

/// The soul crystal's slot, which neither wears nor bonds.
pub const SOUL_CRYSTAL_SLOT: u16 = 13;

/// How much a piece wears relative to the others: on retail every activity took the same proportions, 3 for the body
/// to 4.5 for weapons and armour and 6 for accessories (syntheses 100/150/200, gathers 2/3/5, duties 37/55/75.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WearGroup {
    /// Weapons, the off hand, head, hands, legs and feet.
    Armour,
    Body,
    /// Earrings, necklace, bracelets and rings.
    Accessory,
}

/// The wear group of an equipped slot (container 1000's slot numbers.)
pub fn wear_group(slot: u16) -> WearGroup {
    match slot {
        3 => WearGroup::Body,
        8..=12 => WearGroup::Accessory,
        _ => WearGroup::Armour,
    }
}

/// A whole number spread ±5% around `base`, as retail's wear was (a synthesis' 150 came as 142–157.)
fn spread(base: u32, rng: &mut fastrand::Rng) -> u32 {
    if base < 20 {
        return base;
    }
    rng.u32(base * 95 / 100..=base * 105 / 100)
}

/// Condition one synthesis takes (done or failed): 150 from weapons and armour, 100 from the body, 200 from
/// accessories, each ±5% (Bob's capture of 2026-10-24, 5 syntheses, 60 items.)
pub fn synthesis_wear(slot: u16, rng: &mut fastrand::Rng) -> u16 {
    let base = match wear_group(slot) {
        WearGroup::Armour => 150,
        WearGroup::Body => 100,
        WearGroup::Accessory => 200,
    };
    spread(base, rng) as u16
}

/// Condition one gathering attempt takes: 3, 2 and 5 (a level 58 Miner at normal nodes, Bob's capture of 2026-10-24;
/// unspoiled Botanist nodes took 5–6 from armour, not modelled.)
pub fn gather_wear(slot: u16) -> u16 {
    match wear_group(slot) {
        WearGroup::Armour => 3,
        WearGroup::Body => 2,
        WearGroup::Accessory => 5,
    }
}

/// What kind of content was completed, for its wear and its spiritbond.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentKind {
    Dungeon,
    Trial,
    /// One turn of an eight-player raid.
    Raid,
    AllianceRaid,
    TreasureDungeon,
    Fate,
    /// Anything else (guildhests, quest battles...): treated as a trial, not seen.
    Other,
}

impl ContentKind {
    /// How many "units" of wear completing it takes (retail, from the batches sent at completion: trials, raid turns
    /// and FATEs 1, treasure dungeons 2, dungeons about 10, alliance raids about 14.)
    pub fn wear_units(self) -> u32 {
        match self {
            ContentKind::Dungeon => 10,
            ContentKind::AllianceRaid => 14,
            ContentKind::TreasureDungeon => 2,
            ContentKind::Trial | ContentKind::Raid | ContentKind::Fate | ContentKind::Other => 1,
        }
    }
}

/// Condition completing content takes from a slot: a unit per `wear_units`, ±5% on the total. The unit is 55/37/75
/// (armour/body/accessory) for a player above the content's level (synced down, or unsynced above it), 145/98/196 at
/// its own level (Bob's captures: level 80 trials and raids, Duty Support runs at level 15–19, Mt. Gulg.)
pub fn content_wear(
    slot: u16,
    kind: ContentKind,
    above_level: bool,
    rng: &mut fastrand::Rng,
) -> u16 {
    let unit = match (wear_group(slot), above_level) {
        (WearGroup::Armour, true) => 55,
        (WearGroup::Body, true) => 37,
        (WearGroup::Accessory, true) => 75,
        (WearGroup::Armour, false) => 145,
        (WearGroup::Body, false) => 98,
        (WearGroup::Accessory, false) => 196,
    };
    spread(unit * kind.wear_units(), rng).min(ITEM_CONDITION_MAX as u32) as u16
}

/// The crafting table's base, by how far the gear's item level is above the item made's (0 or below: 33.)
pub fn crafting_base(difference: u16) -> u32 {
    match difference {
        0..=13 => 33 - difference as u32,
        14 => 15,
        15..=20 => 10,
        21..=29 => 5,
        30..=39 => 3,
        40..=50 => 1,
        _ => 0,
    }
}

/// The gathering table's base, by how far the gear's equip level is above the node's: unspoiled and legendary nodes
/// give ten times a normal one's.
pub fn gathering_base(difference: u16, rare: bool) -> u32 {
    let (rare_base, normal) = match difference {
        0 => (50, 5),
        1..=4 => (49, 4),
        5..=9 => (48, 3),
        10..=14 => (47, 2),
        15..=19 => (46, 1),
        _ => (0, 0),
    };
    if rare { rare_base } else { normal }
}

/// Spiritbond Gain counts up to 2 at a normal node and 5 at a legendary one for gatherers.
pub fn gathering_sb_gain_cap(rare: bool) -> u32 {
    if rare { 5 } else { 2 }
}

/// The wiki's formula (The Balance, patch 7.0), exact for every synthesis and gather in Bob's captures: the base plus
/// half of Spiritbond Gain, times 1 + ¼ per materia melded and ¼ more for HQ, then up to 10% more at random.
pub fn points(base: u32, sb_gain: u32, melds: u32, hq: bool, rng: &mut fastrand::Rng) -> u32 {
    if base == 0 {
        return 0;
    }
    // In quarters, to stay in integers: (base + gain / 2) × (4 + melds + hq) / 4.
    let quarters = 4 + melds + hq as u32;
    let min = (base * 2 + sb_gain) * quarters / 8;
    let max = min * 11 / 10;
    rng.u32(min..=max)
}

/// The base for completing content, a stand-in fitted to Bob's captures (the real formula isn't known): low-level
/// dungeons gave 2,650 an item at about their level (67 or 112 kills alike), Mt. Gulg 1,782, a ring 31 levels above a
/// level 19 duty 1,410; trials 396 and 101, raid turns about 30, a treasure dungeon 56. Alliance raids and FATEs
/// weren't seen with gear below 100%: dungeons' and treasure dungeons' numbers stand in.
pub fn content_base(kind: ContentKind, content_level: u8, equip_level: u8) -> u32 {
    match kind {
        ContentKind::Dungeon | ContentKind::AllianceRaid => {
            if equip_level as u32 > content_level as u32 + 20 {
                1410
            } else if content_level <= 50 {
                2650
            } else {
                1782
            }
        }
        ContentKind::Trial | ContentKind::Other => 250,
        ContentKind::Raid => 30,
        ContentKind::TreasureDungeon | ContentKind::Fate => 56,
    }
}

/// What a piece of gear is for, which decides the materia it gives (the wiki's Materia Extraction page.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GearRole {
    War,
    Magic,
    Hand,
    Land,
}

/// BaseParam rows of the materia each role can give.
pub fn materia_params(role: GearRole) -> &'static [u8] {
    match role {
        // Tenacity, Skill Speed, Direct Hit Rate, Critical Hit, Determination.
        GearRole::War => &[19, 45, 22, 27, 44],
        // Piety, Spell Speed, Direct Hit Rate, Critical Hit, Determination.
        GearRole::Magic => &[6, 46, 22, 27, 44],
        // Craftsmanship, Control, CP.
        GearRole::Hand => &[70, 71, 11],
        // Gathering, Perception, GP.
        GearRole::Land => &[72, 73, 10],
    }
}

/// The role of a class or job.
pub fn class_role(classjob: u8) -> GearRole {
    match classjob {
        8..=15 => GearRole::Hand,
        16..=18 => GearRole::Land,
        6 | 7 | 24..=28 | 33 | 35 | 36 | 40 | 42 => GearRole::Magic,
        _ => GearRole::War,
    }
}

/// The role an item's Item.MaterializeType gives, None if it can't be extracted from (0: relics, soul crystals...).
/// From the Item sheet: 3 is tank gear, 4–6 maiming, striking and aiming/scouting, 8 casting, 9 healing, 11 gatherers'
/// and 12 crafters' gear, 2 Disciples of War gear; 1, 7 and 10 are gear for every class (Weathered, Hempen...), which
/// gives the materia of the class the player is on (a guess: the wiki doesn't say.)
pub fn gear_role(materialize_type: u8, player_classjob: u8) -> Option<GearRole> {
    match materialize_type {
        0 => None,
        2..=6 => Some(GearRole::War),
        8 | 9 => Some(GearRole::Magic),
        11 => Some(GearRole::Land),
        12 => Some(GearRole::Hand),
        _ => Some(class_role(player_classjob)),
    }
}

/// Which grade of materia extracting gives, as an index into the Materia sheet's Item column (0 is I), from the wiki's
/// Materia page: I up to equip level 14; then 90% the lower grade and 10% the next for 15–29 (I/II), 30–44 (II/III) and
/// 45–50 (III/IV, IV 75% from item level 60 to 130 and always above); V for 51–60, VI 61–70, VII 71–80, IX 81–90 and
/// XI 91–100 (the even grades above VI come from elsewhere.)
pub fn materia_grade(equip_level: u8, item_level: u16, rng: &mut fastrand::Rng) -> usize {
    let roll = rng.u32(0..100);
    match equip_level {
        0..=14 => 0,
        15..=29 => (roll < 10) as usize,
        30..=44 => 1 + (roll < 10) as usize,
        45..=50 => {
            let high_chance = match item_level {
                0..=59 => 10,
                60..=130 => 75,
                _ => 100,
            };
            2 + (roll < high_chance) as usize
        }
        51..=60 => 4,
        61..=70 => 5,
        71..=80 => 6,
        81..=90 => 8,
        _ => 10,
    }
}

/// The highest gear level each grade of Dark Matter repairs (ItemRepairResource rows 1–8, the wiki's Gear Durability
/// page): grade 1 to 10, 2 to 20 ... 6 to 60, 7 to 80, 8 to 100.
pub fn dark_matter_max_level(grade: usize) -> u8 {
    match grade {
        1 => 10,
        2 => 20,
        3 => 30,
        4 => 40,
        5 => 50,
        6 => 60,
        7 => 80,
        _ => 100,
    }
}

/// A crafter repairs gear up to 10 levels above their own (the Repair window's "Repair Level" is the equip level less
/// 10.)
pub fn can_self_repair(crafter_level: u16, equip_level: u8) -> bool {
    crafter_level + 10 >= equip_level as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crafting_table() {
        assert_eq!(crafting_base(0), 33);
        assert_eq!(crafting_base(13), 20);
        assert_eq!(crafting_base(14), 15);
        assert_eq!(crafting_base(20), 10);
        assert_eq!(crafting_base(29), 5);
        assert_eq!(crafting_base(39), 3);
        assert_eq!(crafting_base(50), 1);
        assert_eq!(crafting_base(51), 0);
    }

    #[test]
    fn gathering_table() {
        assert_eq!(gathering_base(0, true), 50);
        assert_eq!(gathering_base(0, false), 5);
        assert_eq!(gathering_base(19, false), 1);
        assert_eq!(gathering_base(20, true), 0);
    }

    #[test]
    fn retail_points() {
        let mut rng = fastrand::Rng::with_seed(1);
        for _ in 0..200 {
            // An HQ piece without materia at a synthesis of its item level: 41–45 on retail.
            let p = points(33, 0, 0, true, &mut rng);
            assert!((41..=45).contains(&p), "{p}");
            // An NQ weapon one level above: 32–35.
            let p = points(32, 0, 0, false, &mut rng);
            assert!((32..=35).contains(&p), "{p}");
            // A level 58 Miner's gear at an unspoiled node of its level: 50–55 (retail +50..54.)
            let p = points(50, 0, 0, false, &mut rng);
            assert!((50..=55).contains(&p), "{p}");
        }
        assert_eq!(points(0, 10, 5, true, &mut rng), 0);
    }

    #[test]
    fn wear_amounts() {
        let mut rng = fastrand::Rng::with_seed(2);
        for _ in 0..200 {
            assert!((142..=157).contains(&synthesis_wear(0, &mut rng)));
            assert!((95..=105).contains(&synthesis_wear(3, &mut rng)));
            assert!((190..=210).contains(&synthesis_wear(11, &mut rng)));
            // A level 80 dungeon synced: about 550 from armour; a trial 55.
            let w = content_wear(0, ContentKind::Dungeon, true, &mut rng);
            assert!((522..=577).contains(&w), "{w}");
            let w = content_wear(9, ContentKind::Trial, true, &mut rng);
            assert!((71..=78).contains(&w), "{w}");
        }
        assert_eq!(gather_wear(3), 2);
        assert_eq!(gather_wear(12), 5);
    }

    #[test]
    fn extraction() {
        let mut rng = fastrand::Rng::with_seed(3);
        assert_eq!(materia_grade(10, 10, &mut rng), 0);
        assert_eq!(materia_grade(55, 300, &mut rng), 4);
        assert_eq!(materia_grade(85, 600, &mut rng), 8);
        assert_eq!(materia_grade(100, 700, &mut rng), 10);
        let grades: Vec<usize> = (0..1000).map(|_| materia_grade(20, 20, &mut rng)).collect();
        let high = grades.iter().filter(|g| **g == 1).count();
        assert!((50..=150).contains(&high), "{high}");
        assert_eq!(gear_role(0, 1), None);
        assert_eq!(gear_role(12, 1), Some(GearRole::Hand));
        assert_eq!(gear_role(9, 1), Some(GearRole::Magic));
        assert_eq!(gear_role(1, 16), Some(GearRole::Land));
        assert_eq!(materia_params(GearRole::Land), &[72, 73, 10]);
    }

    #[test]
    fn self_repair() {
        assert!(can_self_repair(40, 50));
        assert!(!can_self_repair(39, 50));
        assert_eq!(dark_matter_max_level(5), 50);
        assert_eq!(dark_matter_max_level(8), 100);
    }
}
