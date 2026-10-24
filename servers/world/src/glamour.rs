//! The glamour dresser, the glamour plates and the armoire, as a character's own storage (kept with the inventory.)
//!
//! Their packets are retail's, from Bob's Chronofoil capture of 2026-10-24 in the Hourglass (docs/src/fork/glamour.md);
//! the sizes are 7.57's MirageManager and Cabinet (FFXIVClientStructs).

use kawari::ipc::zone::GlamourPlate;
use serde::{Deserialize, Serialize};

/// How many items the glamour dresser holds (MirageManager.PrismBoxItemIds.)
pub const DRESSER_SLOTS: usize = 800;
/// How many of them one GlamourDresser packet carries (retail sent pages 0 and 1.)
pub const DRESSER_PAGE: usize = 400;
/// How many glamour plates there are.
pub const PLATES: usize = 20;
/// The armoire's bitmask (ArmoireContents): one bit per Cabinet row.
pub const ARMOIRE_BYTES: usize = 136;
/// The container a glamour plate's slot names when its item comes from the dresser (SaveGlamourPlate; retail.)
pub const DRESSER_CONTAINER: u32 = 9999;
/// HQ items are their ID plus this in the dresser and on plates (retail: 1026447 for an HQ 26447.)
pub const HQ_OFFSET: u32 = 1_000_000;

/// One glamour dresser slot: the item (+ HQ_OFFSET if HQ) and its two stains.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct DresserItem {
    pub item_id: u32,
    pub stain0: u8,
    pub stain1: u8,
}

/// One glamour plate: an item and two stains for each of its 12 slots.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Plate {
    pub item_ids: [u32; 12],
    pub stain0: [u8; 12],
    pub stain1: [u8; 12],
}

impl From<Plate> for GlamourPlate {
    fn from(plate: Plate) -> Self {
        Self {
            item_ids: plate.item_ids,
            stain0: plate.stain0,
            stain1: plate.stain1,
        }
    }
}

/// A character's glamour dresser, glamour plates and armoire.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlamourStorage {
    /// The dresser's slots in order (an empty one has item 0; shorter than DRESSER_SLOTS means the rest are empty.)
    #[serde(default)]
    pub dresser: Vec<DresserItem>,
    /// The plates (fewer than PLATES means the rest are empty.)
    #[serde(default)]
    pub plates: Vec<Plate>,
    /// The armoire, one bit per Cabinet row stored.
    #[serde(default)]
    pub armoire: Vec<u8>,
}

impl GlamourStorage {
    /// The item in a dresser slot, if any.
    pub fn dresser_item(&self, index: usize) -> Option<DresserItem> {
        self.dresser
            .get(index)
            .copied()
            .filter(|item| item.item_id != 0)
    }

    /// Puts an item in the first empty dresser slot; returns the slot, or None when the dresser is full.
    pub fn store(&mut self, item: DresserItem) -> Option<usize> {
        let index = self
            .dresser
            .iter()
            .position(|slot| slot.item_id == 0)
            .unwrap_or(self.dresser.len());
        if index >= DRESSER_SLOTS {
            return None;
        }
        if index == self.dresser.len() {
            self.dresser.push(item);
        } else {
            self.dresser[index] = item;
        }
        Some(index)
    }

    /// Takes the item out of a dresser slot.
    pub fn take(&mut self, index: usize) -> Option<DresserItem> {
        let item = self.dresser_item(index)?;
        self.dresser[index] = DresserItem::default();
        while self.dresser.last().is_some_and(|slot| slot.item_id == 0) {
            self.dresser.pop();
        }
        Some(item)
    }

    /// One page of the dresser for the GlamourDresser packet: item IDs and both stains.
    pub fn page(
        &self,
        page: usize,
    ) -> ([u32; DRESSER_PAGE], [u8; DRESSER_PAGE], [u8; DRESSER_PAGE]) {
        let mut ids = [0; DRESSER_PAGE];
        let mut stain0 = [0; DRESSER_PAGE];
        let mut stain1 = [0; DRESSER_PAGE];
        for i in 0..DRESSER_PAGE {
            if let Some(item) = self.dresser.get(page * DRESSER_PAGE + i) {
                ids[i] = item.item_id;
                stain0[i] = item.stain0;
                stain1[i] = item.stain1;
            }
        }
        (ids, stain0, stain1)
    }

    /// The plates for the GlamourPlates packet.
    pub fn plates(&self) -> [GlamourPlate; PLATES] {
        let mut plates = [GlamourPlate::default(); PLATES];
        for (out, plate) in plates.iter_mut().zip(&self.plates) {
            *out = (*plate).into();
        }
        plates
    }

    /// Saves a plate from SaveGlamourPlate: each slot that's set takes the dresser item at its index (with that item's
    /// stains); a slot that isn't set, or whose item doesn't come from the dresser, is left empty (retail: the off hand,
    /// not set, came back 0.) Returns the plate, or None for a plate number out of range.
    pub fn save_plate(
        &mut self,
        index: usize,
        set: &[u8; 12],
        indices: &[u32; 12],
        containers: &[u32; 12],
    ) -> Option<Plate> {
        if index >= PLATES {
            return None;
        }
        let mut plate = Plate::default();
        for slot in 0..12 {
            if set[slot] == 0 || containers[slot] != DRESSER_CONTAINER {
                continue;
            }
            if let Some(item) = self.dresser_item(indices[slot] as usize) {
                plate.item_ids[slot] = item.item_id;
                plate.stain0[slot] = item.stain0;
                plate.stain1[slot] = item.stain1;
            }
        }
        if self.plates.len() <= index {
            self.plates.resize(index + 1, Plate::default());
        }
        self.plates[index] = plate;
        Some(plate)
    }

    /// Whether a Cabinet row is in the armoire.
    pub fn in_armoire(&self, cabinet_id: u32) -> bool {
        let (byte, bit) = (cabinet_id as usize / 8, cabinet_id % 8);
        self.armoire.get(byte).is_some_and(|b| b & (1 << bit) != 0)
    }

    /// Stores (or takes out) a Cabinet row; false for a row the bitmask can't hold.
    pub fn set_in_armoire(&mut self, cabinet_id: u32, stored: bool) -> bool {
        let (byte, bit) = (cabinet_id as usize / 8, cabinet_id % 8);
        if byte >= ARMOIRE_BYTES {
            return false;
        }
        if self.armoire.len() <= byte {
            self.armoire.resize(byte + 1, 0);
        }
        if stored {
            self.armoire[byte] |= 1 << bit;
        } else {
            self.armoire[byte] &= !(1 << bit);
        }
        true
    }

    /// The armoire's bitmask for the ArmoireContents packet.
    pub fn armoire_bytes(&self) -> [u8; ARMOIRE_BYTES] {
        let mut bytes = [0; ARMOIRE_BYTES];
        for (out, byte) in bytes.iter_mut().zip(&self.armoire) {
            *out = *byte;
        }
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: u32) -> DresserItem {
        DresserItem {
            item_id: id,
            stain0: 0,
            stain1: 0,
        }
    }

    #[test]
    fn dresser() {
        let mut storage = GlamourStorage::default();
        // Retail: 444 items in slots 0–443, the next one stored went to 444.
        for i in 0..444 {
            assert_eq!(storage.store(item(1000 + i)), Some(i as usize));
        }
        assert_eq!(storage.store(item(31428)), Some(444));
        assert_eq!(storage.take(444), Some(item(31428)));
        assert_eq!(storage.dresser.len(), 444);
        assert_eq!(storage.take(10), Some(item(1010)));
        assert_eq!(storage.store(item(5)), Some(10));
        let (ids, _, _) = storage.page(1);
        assert_eq!(ids[43], 1443);
        assert_eq!(ids[44], 0);
    }

    #[test]
    fn plate() {
        let mut storage = GlamourStorage::default();
        storage.dresser = vec![item(0); 421];
        storage.dresser[331] = item(17825);
        storage.dresser[420] = DresserItem {
            item_id: 48994,
            stain0: 6,
            stain1: 0,
        };
        // Retail's save of plate 10 (index 9): main hand from 331, the off hand not set, the body from 420.
        let mut set = [0; 12];
        set[0] = 1;
        set[3] = 1;
        let mut indices = [0; 12];
        indices[0] = 331;
        indices[3] = 420;
        let containers = [DRESSER_CONTAINER; 12];
        let plate = storage.save_plate(9, &set, &indices, &containers).unwrap();
        assert_eq!(plate.item_ids[0], 17825);
        assert_eq!(plate.item_ids[1], 0);
        assert_eq!(plate.item_ids[3], 48994);
        assert_eq!(plate.stain0[3], 6);
        assert_eq!(storage.plates().len(), PLATES);
        assert!(
            storage
                .save_plate(20, &set, &indices, &containers)
                .is_none()
        );
    }

    #[test]
    fn armoire() {
        let mut storage = GlamourStorage::default();
        assert!(storage.set_in_armoire(204, true));
        assert!(storage.set_in_armoire(208, true));
        // Retail's bitmask: byte 25 = 0x10 (204), byte 26 = 0x01 (208.)
        let bytes = storage.armoire_bytes();
        assert_eq!(bytes[25], 0x10);
        assert_eq!(bytes[26], 0x01);
        assert!(storage.in_armoire(204));
        assert!(storage.set_in_armoire(204, false));
        assert!(!storage.in_armoire(204));
        assert!(!storage.set_in_armoire(ARMOIRE_BYTES as u32 * 8, true));
    }

    #[test]
    fn old_inventory_json_still_reads() {
        // Inventories saved before this have no glamour storage: it must default, not fail (a failed read empties the
        // whole inventory.)
        let storage: GlamourStorage = serde_json::from_str("{}").unwrap();
        assert!(storage.dresser.is_empty());
    }
}
