//! The glamour dresser, glamour plates and armoire at an inn (`crate::glamour` keeps them.) Every packet and its order
//! is retail's, from Bob's Chronofoil capture of 2026-10-24 in the Hourglass (docs/src/fork/glamour.md.)

use kawari::{
    common::ContainerType,
    ipc::zone::{
        ActorControlCategory, Condition, ItemInfo, ServerZoneIpcData, ServerZoneIpcSegment,
    },
};

use crate::{
    ItemInfoQuery, ZoneConnection,
    glamour::{
        CHANGE_DYE0, CHANGE_DYE1, DRESSER_PAGE, DRESSER_SLOTS, DresserItem, HQ_OFFSET,
        NO_CONTAINER, PlateSlotChange,
    },
    inventory::Item,
};

/// A bag by its number in SaveGlamourPlate (the dyes come from the four bags: 3 was the fourth, Bob's bundles of
/// 2026-10-27.)
fn bag(container: u32) -> Option<ContainerType> {
    match container {
        0 => Some(ContainerType::Inventory0),
        1 => Some(ContainerType::Inventory1),
        2 => Some(ContainerType::Inventory2),
        3 => Some(ContainerType::Inventory3),
        _ => None,
    }
}

/// An item's HQ flag (`Item::item_flags`.)
const ITEM_FLAG_HQ: u8 = 1;

/// LogMessages retail sent (each with the item, or the plate's number.)
const LOG_STORED_IN_DRESSER: u32 = 4380;
const LOG_RESTORED_FROM_DRESSER: u32 = 4383;
const LOG_PLATE_SAVED: u32 = 4382;
const LOG_ITEM_OBTAINED: u32 = 789;
/// Applying a glamour plate: the plate's number (from 1), then one line per slot with the item worn and its glamour.
const LOG_PLATE_APPLIED: u32 = 4364;
const LOG_GLAMOUR_CAST: u32 = 4309;
/// ActorControlSelf retail sent after a plate was applied, `[1]` (meaning unknown; 1800 is a dresser slot changing.)
const PLATE_APPLIED: u32 = 1801;

/// The equipped slot each of a plate's 12 slots dresses: main hand, off hand, head, body, hands, legs, feet, earrings,
/// necklace, bracelets, right ring, left ring (no waist; retail's GlamourPlates against the gear it changed.)
const PLATE_TO_EQUIPPED: [u16; 12] = [0, 1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12];

/// ArmoireResult's results: done, and the item to store wasn't there (retail's first try of row 178.)
const ARMOIRE_DONE: u32 = 0;
const ARMOIRE_NOT_FOUND: u32 = 3;

/// The containers an item can be taken from into the armoire: the bags and the armoury chest.
const ARMOIRE_SOURCES: [ContainerType; 16] = [
    ContainerType::Inventory0,
    ContainerType::Inventory1,
    ContainerType::Inventory2,
    ContainerType::Inventory3,
    ContainerType::ArmoryWeapon,
    ContainerType::ArmoryOffWeapon,
    ContainerType::ArmoryHead,
    ContainerType::ArmoryBody,
    ContainerType::ArmoryHand,
    ContainerType::ArmoryLeg,
    ContainerType::ArmoryFoot,
    ContainerType::ArmoryEarring,
    ContainerType::ArmoryNeck,
    ContainerType::ArmoryWrist,
    ContainerType::ArmoryRing,
    ContainerType::ArmorySoulCrystal,
];

impl ZoneConnection {
    pub(crate) async fn log_message(&mut self, log_message: u32, id: u32) {
        self.actor_control_self(ActorControlCategory::LogMessage { log_message, id })
            .await;
    }

    /// The glamour dresser's two pages (ClientTrigger 2350.)
    pub async fn send_glamour_dresser(&mut self) {
        for page in 0..DRESSER_SLOTS / DRESSER_PAGE {
            let (item_ids, stain0, stain1) = self.player_data.inventory.glamour.page(page);
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::GlamourDresser {
                page: page as u32,
                item_ids,
                stain0,
                stain1,
            });
            self.send_ipc_self(ipc).await;
        }
    }

    /// The glamour plates (ClientTrigger 2356 as the window opens.)
    pub async fn send_glamour_plates(&mut self) {
        let plates = self.player_data.inventory.glamour.plates();
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::GlamourPlates { plates });
        self.send_ipc_self(ipc).await;
    }

    /// An item goes into the glamour dresser (ClientTrigger 2351): out of its slot, into the first empty dresser
    /// slot, then GlamourDresserSlot and LogMessage 4380.
    pub async fn store_in_glamour_dresser(&mut self, container: ContainerType, slot: u16) {
        let Some(item) = self.player_data.inventory.get_item(container, slot) else {
            return;
        };
        if item.quantity <= 0 || item.item_id == 0 {
            return;
        }
        let item_id = if item.item_flags & ITEM_FLAG_HQ != 0 {
            item.item_id + HQ_OFFSET
        } else {
            item.item_id
        };
        let stored = DresserItem {
            item_id,
            stain0: item.stains[0],
            stain1: item.stains[1],
        };
        let Some(index) = self.player_data.inventory.glamour.store(stored) else {
            tracing::info!("The glamour dresser is full");
            return;
        };
        if let Some(slot_item) = self.player_data.inventory.get_item_mut(container, slot) {
            *slot_item = Item::default();
        }
        self.send_inventory_slot(container, slot).await;
        self.actor_control_self(ActorControlCategory::GlamourDresserSlot {
            index: index as u32,
            item_id,
        })
        .await;
        self.log_message(LOG_STORED_IN_DRESSER, item_id).await;
    }

    /// An item comes out of the glamour dresser (ClientTrigger 2352) into the first free bag slot: GlamourDresserSlot
    /// with 0, InventoryActionAck, the slot, LogMessages 4383 and 789, as retail.
    pub async fn restore_from_glamour_dresser(&mut self, index: u32) {
        let Some(stored) = self
            .player_data
            .inventory
            .glamour
            .dresser_item(index as usize)
        else {
            return;
        };
        let hq = stored.item_id >= HQ_OFFSET;
        let base_id = if hq {
            stored.item_id - HQ_OFFSET
        } else {
            stored.item_id
        };
        let Some(info) = self
            .game_data
            .lock()
            .get_item_info(ItemInfoQuery::ById(base_id))
        else {
            return;
        };
        let mut item = Item::new(&info, 1);
        if hq {
            item.item_flags |= ITEM_FLAG_HQ;
        }
        item.stains = [stored.stain0, stored.stain1];
        let Some(placed) = self.player_data.inventory.add_in_next_free_slot(item) else {
            tracing::info!("No room for the glamour dresser's item");
            return;
        };
        self.player_data.inventory.glamour.take(index as usize);

        self.actor_control_self(ActorControlCategory::GlamourDresserSlot { index, item_id: 0 })
            .await;
        self.log_message(LOG_RESTORED_FROM_DRESSER, stored.item_id)
            .await;
        self.send_inventory_action_done().await;
        self.send_inventory_slot(placed.container, placed.slot)
            .await;
        self.log_message(LOG_ITEM_OBTAINED, stored.item_id).await;
    }

    /// Retail's InventoryActionAck after the dresser or armoire gave an item back: sequence 0xFFFFFFFF, type 6.
    async fn send_inventory_action_done(&mut self) {
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::InventoryActionAck {
            sequence: u32::MAX,
            action_type: 6,
            unk1: 0,
            equipped_items_inventory_type: ContainerType::Inventory0,
        });
        self.send_ipc_self(ipc).await;
    }

    /// A glamour plate saved (SaveGlamourPlate): the slots it changed (`GlamourStorage::save_plate`), one of each dye
    /// it used taken from its bag slot (the slot sent again), then GlamourPlateUpdated and LogMessage 4382 with its
    /// number. `dyes` is, for each slot, the bag and slot of its first and second dye (`NO_CONTAINER` for none.)
    pub async fn save_glamour_plate(
        &mut self,
        plate: u8,
        slots: &[PlateSlotChange; 12],
        dyes: &[[(u32, u16); 2]; 12],
    ) {
        let Some(saved) = self
            .player_data
            .inventory
            .glamour
            .save_plate(plate as usize, slots)
        else {
            return;
        };
        for (slot, change) in slots.iter().enumerate() {
            for (channel, bit) in [CHANGE_DYE0, CHANGE_DYE1].into_iter().enumerate() {
                let (container, dye_slot) = dyes[slot][channel];
                if !change.changed || change.what & bit == 0 || container == NO_CONTAINER {
                    continue;
                }
                let Some(container) = bag(container) else {
                    tracing::warn!(
                        "Glamour plate {plate}: slot {slot}'s dye is in container {container}, not a bag; not taken"
                    );
                    continue;
                };
                let taken = self
                    .player_data
                    .inventory
                    .get_item_mut(container, dye_slot)
                    .filter(|item| item.quantity > 0)
                    .map(|item| {
                        let dye = item.item_id;
                        item.quantity -= 1;
                        if item.quantity == 0 {
                            *item = Item::default();
                        }
                        dye
                    });
                match taken {
                    Some(dye) => {
                        tracing::info!(
                            "Glamour plate {plate}: slot {slot} dyed (stain {}) with item {dye} from {container} slot {dye_slot}, one used",
                            if channel == 0 {
                                change.stain0
                            } else {
                                change.stain1
                            }
                        );
                        self.send_inventory_slot(container, dye_slot).await;
                    }
                    None => tracing::warn!(
                        "Glamour plate {plate}: slot {slot}'s dye at {container} slot {dye_slot} isn't there"
                    ),
                }
            }
        }
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::GlamourPlateUpdated {
            index: plate as u32,
            plate: saved.into(),
        });
        self.send_ipc_self(ipc).await;
        self.log_message(LOG_PLATE_SAVED, plate as u32 + 1).await;
    }

    /// The armoire's contents (ClientTrigger 423.)
    pub async fn send_armoire(&mut self) {
        let stored = self.player_data.inventory.glamour.armoire_bytes();
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ArmoireContents { stored });
        self.send_ipc_self(ipc).await;
    }

    /// The packet retail sent with the armoire's contents (ClientTrigger 427): 1 and zeros.
    pub async fn send_armoire_info(&mut self) {
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ArmoireInfo {
            unk1: 1,
            unk2: vec![0; 500],
        });
        self.send_ipc_self(ipc).await;
    }

    /// An item goes into the armoire (ClientTrigger 424): the first of that item in the bags or armoury chest.
    pub async fn store_in_armoire(&mut self, cabinet_id: u32) {
        let item_id = self.game_data.lock().get_cabinet_item(cabinet_id);
        let found = item_id.and_then(|item_id| {
            ARMOIRE_SOURCES.iter().find_map(|container| {
                let storage = self.player_data.inventory.get_container(*container)?;
                (0..storage.max_slots() as u16).find_map(|slot| {
                    let item = storage.get_slot(slot);
                    (item.quantity > 0 && item.item_id == item_id).then_some((*container, slot))
                })
            })
        });
        let Some((container, slot)) = found else {
            self.actor_control_self(ActorControlCategory::ArmoireResult {
                result: ARMOIRE_NOT_FOUND,
                cabinet_id,
                stored: 0,
            })
            .await;
            return;
        };
        if !self
            .player_data
            .inventory
            .glamour
            .set_in_armoire(cabinet_id, true)
        {
            return;
        }
        if let Some(item) = self.player_data.inventory.get_item_mut(container, slot) {
            item.quantity -= 1;
            if item.quantity <= 0 {
                *item = Item::default();
            }
        }
        self.send_inventory_slot(container, slot).await;
        self.actor_control_self(ActorControlCategory::ArmoireResult {
            result: ARMOIRE_DONE,
            cabinet_id,
            stored: 1,
        })
        .await;
    }

    /// An item comes out of the armoire (ClientTrigger 425) into the first free bag slot.
    pub async fn withdraw_from_armoire(&mut self, cabinet_id: u32) {
        if !self.player_data.inventory.glamour.in_armoire(cabinet_id) {
            return;
        }
        let Some(item_id) = self.game_data.lock().get_cabinet_item(cabinet_id) else {
            return;
        };
        let Some(info) = self
            .game_data
            .lock()
            .get_item_info(ItemInfoQuery::ById(item_id))
        else {
            return;
        };
        let Some(placed) = self
            .player_data
            .inventory
            .add_in_next_free_slot(Item::new(&info, 1))
        else {
            tracing::info!("No room for the armoire's item");
            return;
        };
        self.player_data
            .inventory
            .glamour
            .set_in_armoire(cabinet_id, false);
        self.actor_control_self(ActorControlCategory::ArmoireResult {
            result: ARMOIRE_DONE,
            cabinet_id,
            stored: 0,
        })
        .await;
        self.send_inventory_action_done().await;
        self.send_inventory_slot(placed.container, placed.slot)
            .await;
        self.log_message(LOG_ITEM_OBTAINED, item_id).await;
    }

    /// The character window's glamour plate list opened or closed (ClientTrigger 2357): retail set condition 14 while
    /// it was open.
    pub async fn toggle_glamour_plate_selection(&mut self, open: bool) {
        self.conditions
            .toggle_condition(Condition::Occupied39, open);
        self.send_conditions().await;
    }

    /// Applies a glamour plate (0–19) to the gear worn (ClientTrigger 2358, or a gearset with the plate linked). Each of
    /// the plate's slots that has an item glamours the piece worn there with it and its stains; an empty plate slot
    /// leaves the piece as it is (retail kept the main hand's glamour under plates without one). Retail sent the slots
    /// that changed, the stats, the model, LogMessage 4364 with the plate's number, a 4309 line for each slot with an
    /// item (the piece worn, its glamour), then ActorControlSelf 1801 `[1]`.
    pub async fn apply_glamour_plate(&mut self, plate: usize) {
        let Some(glamours) = self
            .player_data
            .inventory
            .glamour
            .plates
            .get(plate)
            .copied()
        else {
            return;
        };
        let mut lines = Vec::new();
        for (index, equip_slot) in PLATE_TO_EQUIPPED.iter().enumerate() {
            let glamour_id = glamours.item_ids[index] % HQ_OFFSET;
            if glamour_id == 0 {
                continue;
            }
            let stains = [glamours.stain0[index], glamours.stain1[index]];
            let Some(item) = self
                .player_data
                .inventory
                .get_item_mut(ContainerType::Equipped, *equip_slot)
            else {
                continue;
            };
            if item.is_empty_slot() {
                continue;
            }
            lines.push((item.item_id, glamour_id));
            if item.glamour_id == glamour_id && item.stains == stains {
                continue;
            }
            item.glamour_id = glamour_id;
            item.stains = stains;
            let item = *item;
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::UpdateInventorySlot(ItemInfo {
                sequence: self.player_data.shop_sequence,
                container: ContainerType::Equipped,
                slot: *equip_slot,
                ..item.into()
            }));
            self.send_ipc_self(ipc).await;
            self.player_data.shop_sequence += 1;
        }
        self.send_stats().await;
        self.inform_equip().await;
        self.log_message(LOG_PLATE_APPLIED, plate as u32 + 1).await;
        for (item_id, glamour_id) in lines {
            self.actor_control_self(ActorControlCategory::Unknown {
                category: 517,
                param1: LOG_GLAMOUR_CAST,
                param2: item_id,
                param3: glamour_id,
                param4: 0,
                param5: 0,
            })
            .await;
        }
        self.actor_control_self(ActorControlCategory::Unknown {
            category: PLATE_APPLIED,
            param1: 1,
            param2: 0,
            param3: 0,
            param4: 0,
            param5: 0,
        })
        .await;
    }
}
