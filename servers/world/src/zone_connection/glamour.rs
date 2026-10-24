//! The glamour dresser, glamour plates and armoire at an inn (`crate::glamour` keeps them.) Every packet and its order
//! is retail's, from Bob's Chronofoil capture of 2026-10-24 in the Hourglass (docs/src/fork/glamour.md.)

use kawari::{
    common::ContainerType,
    ipc::zone::{ActorControlCategory, ServerZoneIpcData, ServerZoneIpcSegment},
};

use crate::{
    ItemInfoQuery, ZoneConnection,
    glamour::{DRESSER_PAGE, DRESSER_SLOTS, DresserItem, HQ_OFFSET},
    inventory::Item,
};

/// An item's HQ flag (`Item::item_flags`.)
const ITEM_FLAG_HQ: u8 = 1;

/// LogMessages retail sent (each with the item, or the plate's number.)
const LOG_STORED_IN_DRESSER: u32 = 4380;
const LOG_RESTORED_FROM_DRESSER: u32 = 4383;
const LOG_PLATE_SAVED: u32 = 4382;
const LOG_ITEM_OBTAINED: u32 = 789;

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
    async fn log_message(&mut self, log_message: u32, id: u32) {
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

    /// A glamour plate saved (SaveGlamourPlate): GlamourPlateUpdated and LogMessage 4382 with its number.
    pub async fn save_glamour_plate(
        &mut self,
        plate: u8,
        set: &[u8; 12],
        indices: &[u32; 12],
        containers: &[u32; 12],
    ) {
        let Some(saved) =
            self.player_data
                .inventory
                .glamour
                .save_plate(plate as usize, set, indices, containers)
        else {
            return;
        };
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
}
