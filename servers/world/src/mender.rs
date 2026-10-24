//! What a mender (Merchant & Mender) charges and which items each of its "repair all" buttons covers.
//!
//! From Bob's Chronofoil capture of 2026-10-24 at a Merchant & Mender: repairing the ten pieces he wore cost 2,059 gil,
//! exactly the sum of each item's ItemRepairPrice (at its equip level) times the share of its condition missing,
//! rounded up per item (docs/src/fork/mender.md.)

use kawari::common::{ContainerType, ITEM_CONDITION_MAX};

/// The armoury chest groups the mender's window repairs together (ClientTrigger 1601's group.)
pub fn armoury_group(group: u32) -> &'static [ContainerType] {
    use ContainerType::*;
    match group {
        0 => &[ArmoryWeapon, ArmoryOffWeapon],
        1 => &[ArmoryHead, ArmoryBody, ArmoryHand],
        2 => &[ArmoryWaist, ArmoryLeg, ArmoryFoot],
        3 => &[ArmoryEarring, ArmoryNeck],
        4 => &[ArmoryWrist, ArmoryRing],
        _ => &[],
    }
}

/// The gil to bring an item at `condition` (of 30,000) back to full, for its ItemRepairPrice `price`: the price times
/// the share missing, rounded up.
pub fn repair_cost(price: u32, condition: u16) -> u32 {
    let missing = ITEM_CONDITION_MAX.saturating_sub(condition) as u64;
    (price as u64 * missing).div_ceil(ITEM_CONDITION_MAX as u64) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retail_costs() {
        // Bob's ten pieces (equip level 82 and 84, ItemRepairPrice 700) and their conditions: 2,059 gil on retail.
        let conditions = [
            22021, 23571, 21846, 21884, 21981, 20051, 20275, 20256, 20057, 20120,
        ];
        let total: u32 = conditions.iter().map(|c| repair_cost(700, *c)).sum();
        assert_eq!(total, 2059);
        assert_eq!(repair_cost(700, ITEM_CONDITION_MAX), 0);
        assert_eq!(repair_cost(700, 0), 700);
    }

    #[test]
    fn groups() {
        assert_eq!(armoury_group(0).len(), 2);
        assert_eq!(
            armoury_group(4),
            &[ContainerType::ArmoryWrist, ContainerType::ArmoryRing]
        );
        assert!(armoury_group(9).is_empty());
    }
}
