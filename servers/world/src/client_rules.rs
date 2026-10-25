//! Rules the game client works out by itself, ported from its own code so the server agrees with what the player sees.
//!
//! The tables below are read from the client's executable by `tools/clientlogic/clientlogic.py` (re-run it after a
//! game patch and paste its `--rust` output here): identical on the 2026.10.16 client and Bob's later build of
//! 2026-10-20. Only action, class and trait numbers; the traits' values, levels and classes come from the Trait
//! sheet at run time.

use std::collections::HashMap;

use kawari::common::ActionCategory;
use parking_lot::Mutex;

use crate::GameData;

/// GetMaxCharges' chains: an action (whose Action row has charges) and the traits it checks in turn; the first one
/// learnt gives its Value as the charges, else the row's MaxCharges.
pub const MAX_CHARGE_TRAITS: &[(u32, &[u32])] = &[
    (71, &[161]),
    (83, &[438]),
    (95, &[436]),
    (110, &[445]),
    (117, &[445]),
    (2262, &[279]),
    (2874, &[292]),
    (2876, &[450]),
    (2890, &[292]),
    (3570, &[625]),
    (3614, &[634, 316]),
    (3640, &[272]),
    (7386, &[421]),
    (7432, &[490]),
    (7499, &[443]),
    (7518, &[485]),
    (16010, &[256, 254]),
    (16151, &[425]),
    (16483, &[442]),
    (16485, &[442]),
    (16486, &[442]),
    (16498, &[605]),
    (16499, &[605]),
    (16556, &[500]),
    (24380, &[383]),
    (24381, &[383]),
    (25762, &[431]),
    (25799, &[480]),
    (34646, &[529]),
    (34670, &[545]),
    (34671, &[545]),
    (34672, &[545]),
    (34673, &[545]),
    (34674, &[540]),
    (35347, &[545]),
    (35348, &[540]),
    (36951, &[580]),
    (44893, &[751]),
];
/// GetAdjustedRecastTime's trait adjustments: an action, a trait whose Value × 100 ms is added to its recast once
/// learnt, and the classes that apply on (empty: every class.)
pub const RECAST_TRAITS: &[(u32, u32, &[u8])] = &[
    (158, 463, &[25]),
    (184, 213, &[27]),
    (3585, 494, &[28]),
    (3606, 189, &[33]),
    (7405, 447, &[23]),
    (7496, 591, &[34]),
    (7519, 306, &[35]),
    (7561, 644, &[]),
    (16012, 456, &[38]),
    (16481, 591, &[34]),
    (16542, 629, &[28]),
    (16801, 213, &[27]),
    (16802, 213, &[27]),
    (16803, 213, &[27]),
    (16889, 452, &[31]),
    (24294, 636, &[40]),
    (24300, 378, &[40]),
    (44904, 757, &[43]),
    (44905, 756, &[43]),
];

/// The shortest recast speed and haste can make for a weaponskill or spell (ActionManager's recast adjustment, the
/// function GetAdjustedRecastTime calls for actions without charges: `cmovl` with 1500 ms.)
pub const MIN_ADJUSTED_RECAST_MS: i32 = 1500;

/// Whether `trait_id` is learnt by `class_job` at `level`, and its Value: the client's trait check (the function
/// GetMaxCharges and GetAdjustedRecastTime call with a trait ID) wants the trait's ClassJobCategory to include the
/// class, its Level to be set and reached, and its Quest (if any) completed (`quest_done`.)
pub fn learnt_trait(
    game_data: &mut GameData,
    trait_id: u32,
    class_job: u8,
    level: u8,
    quest_done: &dyn Fn(u32) -> bool,
) -> Option<i16> {
    let row = game_data.trait_sheet.row(trait_id)?;
    let (category, trait_level, quest, value) =
        (row.ClassJobCategory, row.Level, row.Quest, row.Value);
    if category == 0 || trait_level == 0 || level < trait_level {
        return None;
    }
    if quest != 0 && !quest_done(quest) {
        return None;
    }
    category_has(game_data, category, class_job).then_some(value)
}

/// Whether a ClassJobCategory includes a class (cached: the sheet is read whole each time otherwise.)
fn category_has(game_data: &mut GameData, category: u8, class_job: u8) -> bool {
    static CATEGORIES: Mutex<Option<HashMap<u8, Vec<u8>>>> = Mutex::new(None);
    let mut cache = CATEGORIES.lock();
    let map = cache.get_or_insert_with(HashMap::new);
    let classes = map
        .entry(category)
        .or_insert_with(|| game_data.get_applicable_classjobs(category as u16));
    classes.contains(&class_job)
}

/// An action's charges at a level (ActionManager.GetMaxCharges): 1 if its Action row has none, else the Value of the
/// first trait of its chain that's learnt, else the row's MaxCharges.
pub fn max_charges(
    game_data: &mut GameData,
    action_id: u32,
    row_max_charges: u8,
    class_job: u8,
    level: u8,
    quest_done: &dyn Fn(u32) -> bool,
) -> u8 {
    if row_max_charges == 0 {
        return 1;
    }
    if let Some((_, chain)) = MAX_CHARGE_TRAITS.iter().find(|(id, _)| *id == action_id) {
        for trait_id in *chain {
            if let Some(value) = learnt_trait(game_data, *trait_id, class_job, level, quest_done) {
                return value.clamp(0, u8::MAX as i16) as u8;
            }
        }
    }
    row_max_charges
}

/// What learnt traits add to an action's recast, in milliseconds (ActionManager.GetAdjustedRecastTime: each one's
/// Value × 100; negative shortens it.)
pub fn recast_trait_ms(
    game_data: &mut GameData,
    action_id: u32,
    class_job: u8,
    level: u8,
    quest_done: &dyn Fn(u32) -> bool,
) -> i32 {
    RECAST_TRAITS
        .iter()
        .filter(|(id, _, classes)| {
            *id == action_id && (classes.is_empty() || classes.contains(&class_job))
        })
        .filter_map(|(_, trait_id, _)| {
            learnt_trait(game_data, *trait_id, class_job, level, quest_done)
        })
        .map(|value| value as i32 * 100)
        .sum()
}

/// Whether a recast is sped up by Skill or Spell Speed and haste: weaponskills and spells, when the action has no more
/// than one charge (GetAdjustedRecastTime only adjusts those.)
pub fn recast_uses_speed(category: ActionCategory, charges: u8) -> bool {
    charges <= 1
        && matches!(
            category,
            ActionCategory::Spell | ActionCategory::Weaponskill
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_are_sorted_and_unique() {
        assert!(MAX_CHARGE_TRAITS.windows(2).all(|w| w[0].0 < w[1].0));
        // Essential Dignity: Enhanced Essential Dignity II (3) first, then Enhanced Essential Dignity (2.)
        assert_eq!(
            MAX_CHARGE_TRAITS
                .iter()
                .find(|(a, _)| *a == 3614)
                .unwrap()
                .1,
            &[634, 316]
        );
        // Swiftcast: Enhanced Swiftcast on every class.
        assert!(RECAST_TRAITS.contains(&(7561, 644, &[])));
    }

    #[test]
    fn speed_only_for_one_charge_weaponskills_and_spells() {
        assert!(recast_uses_speed(ActionCategory::Spell, 1));
        assert!(recast_uses_speed(ActionCategory::Weaponskill, 0));
        assert!(!recast_uses_speed(ActionCategory::Ability, 1));
        assert!(!recast_uses_speed(ActionCategory::Spell, 2));
    }
}
