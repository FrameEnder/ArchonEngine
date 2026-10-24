//! Repairs at a mender (Merchant & Mender, CustomTalk 19): one item, everything worn, or a group of the armoury chest.
//! The packets and their order are retail's, from Bob's Chronofoil capture of 2026-10-24 (docs/src/fork/mender.md);
//! `crate::mender` has the cost.

use kawari::{
    common::{ContainerType, ITEM_CONDITION_MAX, ItemOperationKind},
    ipc::zone::{ActorControlCategory, ItemInfo, ServerZoneIpcData, ServerZoneIpcSegment},
};

use crate::{ZoneConnection, mender};

/// LogMessages retail sent: one item repaired (with the item), and how many a "repair all" repaired.
const LOG_REPAIRED_ONE: u32 = 1385;
const LOG_REPAIRED_SOME: u32 = 1388;
/// ActorControlSelf categories retail sent after a repair, meaning unknown: 90 with the container after a single item,
/// 91 after every repair, 93 `[1]` after a "repair all".
const REPAIR_AFTER_ONE: u32 = 90;
const REPAIR_DONE: u32 = 91;
const REPAIR_AFTER_ALL: u32 = 93;

impl ZoneConnection {
    async fn repair_actor_control(&mut self, category: u32, param1: u32) {
        self.actor_control_self(ActorControlCategory::Unknown {
            category,
            param1,
            param2: 0,
            param3: 0,
            param4: 0,
            param5: 0,
        })
        .await;
    }

    async fn send_slot(&mut self, container: ContainerType, slot: u16) {
        let Some(item) = self.player_data.inventory.get_item(container, slot) else {
            return;
        };
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::UpdateInventorySlot(ItemInfo {
            sequence: self.player_data.shop_sequence,
            container,
            slot,
            ..item.into()
        }));
        self.send_ipc_self(ipc).await;
        self.player_data.shop_sequence += 1;
    }

    /// What repairing this slot costs, if it needs and can have one.
    fn repair_cost_of(&mut self, container: ContainerType, slot: u16) -> Option<u32> {
        let item = self.player_data.inventory.get_item(container, slot)?;
        if item.is_empty_slot() || item.condition >= ITEM_CONDITION_MAX {
            return None;
        }
        let price = self.game_data.lock().repair_price(item.item_id)?;
        Some(mender::repair_cost(price, item.condition))
    }

    /// Repairs every item that needs it in these containers, if the gil covers them all: retail sent each item's slot,
    /// LogMessage 1388 with how many, the gil, an Update InventoryTransaction for each item and the gil, the Repair
    /// finish, the stats when worn gear changed, then ActorControlSelf 91 and 93 `[1]`.
    async fn repair_all_in(&mut self, containers: &[ContainerType]) {
        let mut repairs = Vec::new();
        let mut total = 0;
        for &container in containers {
            let size = self
                .player_data
                .inventory
                .get_container(container)
                .map(|c| c.max_slots())
                .unwrap_or(0);
            for slot in 0..size as u16 {
                if let Some(cost) = self.repair_cost_of(container, slot) {
                    repairs.push((container, slot));
                    total += cost;
                }
            }
        }
        if repairs.is_empty() {
            return;
        }
        if (self.player_data.inventory.currency.gil.quantity.max(0) as u32) < total {
            tracing::info!(
                "A repair of {} items costs {total} gil, more than the player has",
                repairs.len()
            );
            return;
        }

        for &(container, slot) in &repairs {
            if let Some(item) = self.player_data.inventory.get_item_mut(container, slot) {
                item.condition = ITEM_CONDITION_MAX;
            }
            self.send_slot(container, slot).await;
        }
        self.actor_control_self(ActorControlCategory::LogMessage {
            log_message: LOG_REPAIRED_SOME,
            id: repairs.len() as u32,
        })
        .await;
        self.player_data.inventory.currency.gil.quantity -= total as i32;
        self.send_slot(ContainerType::Currency, 0).await;

        let actor_id = self.player_data.character.actor_id;
        let mut updated: Vec<(ContainerType, u16)> = repairs.clone();
        updated.push((ContainerType::Currency, 0));
        for (container, slot) in updated {
            let Some(item) = self.player_data.inventory.get_item(container, slot) else {
                continue;
            };
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::InventoryTransaction {
                sequence: self.player_data.item_sequence,
                operation_type: ItemOperationKind::Update,
                src_actor_id: actor_id,
                src_storage_id: container,
                src_container_index: slot,
                src_stack: item.quantity as u32,
                src_item_id: item.item_id,
                dst_actor_id: Default::default(),
                dummy_container: ContainerType::DiscardingItemSentinel,
                dst_storage_id: ContainerType::DiscardingItemSentinel,
                dst_container_index: u16::MAX,
                dst_stack: 0,
                dst_item_id: 0,
            });
            self.send_ipc_self(ipc).await;
        }
        // Retail's second word: one more than the transactions, in its second byte (11 transactions: 0x0C00.)
        self.send_inventory_transaction_finish(
            ItemOperationKind::Repair,
            ((repairs.len() as u32 + 2) & 0xFF) << 8,
        )
        .await;

        if containers.contains(&ContainerType::Equipped) {
            self.send_stats().await;
        }
        self.repair_actor_control(REPAIR_DONE, 0).await;
        self.repair_actor_control(REPAIR_AFTER_ALL, 1).await;
    }

    /// Repairs one item (ClientTrigger 1600): retail sent its slot, the gil, LogMessage 1385 with the item, the stats,
    /// then ActorControlSelf 90 with the container and 91.
    pub async fn repair_item(&mut self, container: ContainerType, slot: u16, item_id: u32) {
        let Some(item) = self.player_data.inventory.get_item(container, slot) else {
            return;
        };
        if item.item_id != item_id {
            tracing::warn!(
                "The client wants item {item_id} repaired in {container:?} {slot}, which holds {}",
                item.item_id
            );
            return;
        }
        let Some(cost) = self.repair_cost_of(container, slot) else {
            return;
        };
        if (self.player_data.inventory.currency.gil.quantity.max(0) as u32) < cost {
            tracing::info!("Repairing {item_id} costs {cost} gil, more than the player has");
            return;
        }
        if let Some(item) = self.player_data.inventory.get_item_mut(container, slot) {
            item.condition = ITEM_CONDITION_MAX;
        }
        self.player_data.inventory.currency.gil.quantity -= cost as i32;
        self.send_slot(container, slot).await;
        self.send_slot(ContainerType::Currency, 0).await;
        self.actor_control_self(ActorControlCategory::LogMessage {
            log_message: LOG_REPAIRED_ONE,
            id: item_id,
        })
        .await;
        self.send_stats().await;
        self.repair_actor_control(REPAIR_AFTER_ONE, container as u32)
            .await;
        self.repair_actor_control(REPAIR_DONE, 0).await;
    }

    /// The mender's "repair all" for one armoury chest group (ClientTrigger 1601.)
    pub async fn repair_armoury_group(&mut self, group: u32) {
        self.repair_all_in(mender::armoury_group(group)).await;
    }

    /// The mender's "repair all" for one container, e.g. the gear worn (ClientTrigger 1602.)
    pub async fn repair_container(&mut self, container: ContainerType) {
        self.repair_all_in(&[container]).await;
    }
}
