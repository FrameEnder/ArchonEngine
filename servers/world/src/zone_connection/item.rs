//! Managing the inventory, equipped model IDs and shops.

use crate::{
    ItemInfoQuery, ToServer, ZoneConnection,
    inventory::{
        DesiredHousingInventoryPages, EQUIP_RESTRICTED, Item, Storage, get_next_free_slot,
    },
};
use kawari::{
    common::{
        ContainerType, ERR_INVENTORY_ADD_FAILED, INVENTORY_ACTION_ACK_GENERAL, ItemOperationKind,
        LegacyEquipmentModelId, ObjectId, WeaponModelId,
    },
    ipc::zone::{
        ActorControlCategory, ContainerInfo, CurrencyInfo, Equip, ItemInfo, ItemOperation,
        ServerZoneIpcData, ServerZoneIpcSegment,
    },
};

use physis::equipment::EquipSlot;
use strum::IntoEnumIterator;

impl ZoneConnection {
    /// Inform other clients (including yourself) that you changed your equipped model ids.
    pub async fn inform_equip(&mut self) {
        let main_weapon_id;
        let sub_weapon_id;
        let model_ids;
        let second_model_stain_ids;
        {
            let mut game_data = self.game_data.lock();
            let inventory = &self.player_data.inventory;

            main_weapon_id = inventory.get_main_weapon_id(&mut game_data);
            sub_weapon_id = inventory.get_shown_sub_weapon_id(
                &mut game_data,
                self.player_data.classjob.current_class as u8,
            );
            model_ids = inventory.legacy_model_ids(&mut game_data);
            second_model_stain_ids = inventory.second_model_stain_ids();
        }

        self.handle
            .send(ToServer::Equip(
                self.player_data.character.actor_id,
                main_weapon_id,
                sub_weapon_id,
                model_ids,
                second_model_stain_ids,
            ))
            .await;
    }

    pub async fn send_inventory(&mut self) {
        // After the first time, only what changed, slot by slot (`send_changed_slots`.)
        if self.inventory_shown {
            self.send_changed_slots().await;
            return;
        }
        self.inventory_shown = true;
        for (container_type, container) in (&self.player_data.inventory.clone()).into_iter() {
            self.send_container(container, container_type).await;
        }

        // Inform the client of dummy container states that are not implemented
        for container_type in [
            ContainerType::Mail,
            ContainerType::Unk2,
            ContainerType::ArmoryWaist,
        ] {
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ContainerInfo(ContainerInfo {
                sequence: self.player_data.item_sequence,
                num_items: 0,
                container: container_type,
                ..Default::default()
            }));
            self.send_ipc_self(ipc).await;
            self.player_data.item_sequence += 1;
        }
    }

    /// The inventory again, as retail updates it once the client has it: an UpdateInventorySlot for each slot whose
    /// item changed, an empty one for each slot emptied (Bob's Chronofoil capture of a synthesis: the materials',
    /// crystals' and the made item's slots, one each). The whole inventory resent (UpdateItem and ContainerInfo, as at
    /// login) didn't take on the client for the slots it already had: a stack used up stayed in its bags and the item
    /// made in its place never showed, and the next synthesis and gear set named what wasn't there (Bob's Lira Ormont
    /// bundles of 2026-10-26). Gathering, shops, quests' rewards and the other callers of `send_inventory` all go
    /// through here.
    pub async fn send_changed_slots(&mut self) {
        let inventory = self.player_data.inventory.clone();
        let mut current: std::collections::HashMap<(ContainerType, u16), ItemInfo> =
            std::collections::HashMap::new();
        for (container_type, container) in (&inventory).into_iter() {
            for slot in 0..container.max_slots() as u16 {
                let item = container.get_slot(slot);
                if item.is_empty_slot() {
                    continue;
                }
                current.insert(
                    (container_type, slot),
                    ItemInfo {
                        sequence: 0,
                        container: container_type,
                        slot,
                        ..(*item).into()
                    },
                );
            }
        }
        let mut changed: Vec<(ContainerType, u16)> = current
            .iter()
            .filter(|(key, info)| self.shown_items.get(key) != Some(*info))
            .map(|(key, _)| *key)
            .collect();
        // A retainer's containers only while one is called: the client has none otherwise, and an UpdateInventorySlot
        // for one crashed it (Bob's teleport of 2026-10-27, the retainer dismissed 2 minutes before.)
        let retainer_out = inventory.retainer.is_some();
        changed.extend(
            self.shown_items
                .keys()
                .filter(|key| !current.contains_key(key))
                .filter(|(container, _)| {
                    retainer_out || !crate::retainer::is_retainer_container(*container)
                })
                .copied(),
        );
        changed.sort_by_key(|(container, slot)| (*container as u16, *slot));
        for (container, slot) in changed {
            let info = current
                .get(&(container, slot))
                .cloned()
                .unwrap_or(ItemInfo {
                    container,
                    slot,
                    ..Default::default()
                });
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::UpdateInventorySlot(ItemInfo {
                sequence: self.player_data.item_sequence,
                ..info
            }));
            self.send_ipc_self(ipc).await;
            self.player_data.item_sequence += 1;
        }
    }

    /// Sends the updateitem and containerinfo packets for the equipped container.
    pub async fn send_equipped_inventory(&mut self) {
        let equipped = self.player_data.inventory.equipped;

        self.send_container(&equipped, ContainerType::Equipped)
            .await;
    }

    pub async fn send_housing_inventory(&mut self, which: DesiredHousingInventoryPages) {
        let cloned_inv = match which {
            DesiredHousingInventoryPages::Interior => {
                self.player_data.house_inventory.interior.clone()
            }
            DesiredHousingInventoryPages::InteriorStoreroom => {
                self.player_data.house_inventory.interior_storeroom.clone()
            }
            DesiredHousingInventoryPages::Exterior => {
                self.player_data.house_inventory.exterior.clone()
            }
            DesiredHousingInventoryPages::ExteriorStoreroom => {
                self.player_data.house_inventory.exterior_storeroom.clone()
            }
            DesiredHousingInventoryPages::None => {
                return;
            }
        };

        for container in cloned_inv.into_iter() {
            self.send_container(&container, container.kind).await;
        }
    }

    pub async fn send_container(&mut self, container: &dyn Storage, container_type: ContainerType) {
        let mut num_items = 0;
        for slot_index in 0..container.max_slots() {
            let item = container.get_slot(slot_index as u16);
            // Don't tell the client about things they don't have
            if item.is_empty_slot() {
                continue;
            }

            let ipc = match container_type {
                // A retainer's gil came this way too on retail (Bob's Chronofoil captures.)
                ContainerType::Currency
                | ContainerType::Crystals
                | ContainerType::RetainerGil
                | ContainerType::RetainerCrystals => ServerZoneIpcSegment::new(
                    ServerZoneIpcData::CurrencyCrystalInfo(CurrencyInfo {
                        sequence: self.player_data.item_sequence,
                        container: container_type,
                        quantity: item.quantity,
                        item_id: item.item_id,
                        slot: slot_index as u16,
                    }),
                ),
                _ => ServerZoneIpcSegment::new(ServerZoneIpcData::UpdateItem(ItemInfo {
                    sequence: self.player_data.item_sequence,
                    container: container_type,
                    slot: slot_index as u16,
                    ..(*item).into()
                })),
            };

            self.send_ipc_self(ipc).await;

            num_items += 1;
        }

        // Inform the client of container state
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ContainerInfo(ContainerInfo {
            container: container_type,
            num_items,
            sequence: self.player_data.item_sequence,
            ..Default::default()
        }));
        self.send_ipc_self(ipc).await;

        self.player_data.item_sequence += 1;
    }

    /// Sends two containers to the client, particularly helpful when doing various housing operations, such as moving an item from the main inventory directly to the storeroom.
    pub async fn send_affected_containers(
        &mut self,
        src_container_type: ContainerType,
        dst_container_type: ContainerType,
    ) {
        // This cloning is sort of ugly, but we run into numerous borrowing issues if we don't.
        let main_inventory = self.player_data.inventory.clone();
        let house_inventory = self.player_data.house_inventory.clone();

        let get_container = |kind: &ContainerType| -> Option<&dyn Storage> {
            if let Some(temp_container) = main_inventory.get_container(*kind) {
                return Some(temp_container);
            } else if let Some(temp_container) = house_inventory.get_container(*kind) {
                return Some(temp_container);
            }

            None
        };

        let Some(src_container) = get_container(&src_container_type) else {
            return;
        };

        let Some(dst_container) = get_container(&dst_container_type) else {
            return;
        };

        self.send_container(src_container, src_container_type).await;
        self.send_container(dst_container, dst_container_type).await;
    }

    pub async fn update_equip(
        &mut self,
        actor_id: ObjectId,
        main_weapon_id: WeaponModelId,
        sub_weapon_id: WeaponModelId,
        models: [LegacyEquipmentModelId; 10],
        second_model_stain_ids: [u8; 10],
    ) {
        self.send_stats().await;

        // Retail's own Equip has the class's level after the class (Bob's Chronofoil capture d3379099: 1 for a level 1
        // Gladiator, 35 for a level 35 Conjurer); Kawari's was 0.
        let level = if self.player_data.character.actor_id == actor_id {
            let game_data = self.game_data.lock();
            self.current_level(&game_data) as u8
        } else {
            0
        };
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::Equip(Equip {
            main_weapon_id,
            sub_weapon_id,
            classjob_id: self.player_data.classjob.current_class as u8,
            level,
            models,
            second_model_stain_ids,
            ..Default::default()
        }));

        self.send_ipc_from(actor_id, ipc).await;

        // TODO: get a capture of another player equipping stuff to see if we get this as well, but it seems unlikely.
        if self.player_data.character.actor_id == actor_id {
            let level;
            {
                let mut game_data = self.game_data.lock();

                level = self
                    .player_data
                    .inventory
                    .equipped
                    .calculate_item_level(&mut game_data) as u32;
            }

            self.actor_control_self(ActorControlCategory::SetItemLevel { level })
                .await;

            // This seems to be pattern/crest related, it's seen when (un)equipping stuff.
            self.actor_control_self(ActorControlCategory::Unknown {
                category: 57,
                param1: 0,
                param2: 0,
                param3: 0,
                param4: 0,
                param5: 0,
            })
            .await;

            // After a gearset retail sent ActorControlSelf 805 here, with nothing in it (every EquipGearset in Bob's
            // Chronofoil captures 44df02fc and d3379099), meaning unknown.
            if self.gearset_equipped {
                self.gearset_equipped = false;
                self.actor_control_self(ActorControlCategory::Unknown {
                    category: GEARSET_EQUIPPED_END,
                    param1: 0,
                    param2: 0,
                    param3: 0,
                    param4: 0,
                    param5: 0,
                })
                .await;
            }
        }

        self.send_stats().await;
        self.update_class_info().await;
    }

    /// Dying takes 600 (2 %) off the condition of everything the player wears, as retail, right after the death
    /// (Bob's Chronofoil captures, 374 items), each slot sent again.
    pub async fn wear_gear_on_death(&mut self) {
        const DEATH_WEAR: u16 = 600;
        // The soul crystal doesn't wear.
        const SOUL_CRYSTAL_SLOT: u16 = physis::equipment::EquipSlot::SoulCrystal as u16;
        let equipped = &mut self.player_data.inventory.equipped;
        let mut worn = Vec::new();
        for slot in (0..equipped.max_slots() as u16).filter(|slot| *slot != SOUL_CRYSTAL_SLOT) {
            let item = equipped.get_slot_mut(slot);
            if !item.is_empty_slot() && item.condition > 0 {
                item.condition = item.condition.saturating_sub(DEATH_WEAR);
                worn.push(slot);
            }
        }
        for slot in worn {
            self.send_inventory_slot(ContainerType::Equipped, slot)
                .await;
        }
    }

    /// An item an enemy dropped, obtained directly: retail's LootMessage (kind 3: who, the item, how many) gives the
    /// log line, then an ack (0xFFFFFFFF, 6) and the slot (Bob's Chronofoil captures; here the whole inventory.)
    pub async fn obtain_drop(&mut self, id: u32, quantity: i32) {
        let new_item = {
            let mut game_data = self.game_data.lock();
            game_data
                .get_item_info(ItemInfoQuery::ById(id))
                .map(|info| Item::new(&info, quantity))
        };
        let Some(new_item) = new_item else {
            return;
        };
        if !self.player_data.inventory.add_obtained_item(new_item) {
            self.send_notice(ERR_INVENTORY_ADD_FAILED).await;
            return;
        }
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::LootMessage {
            kind: LOOT_MESSAGE_OBTAINED,
            params: [
                self.player_data.character.actor_id.0,
                id,
                quantity.max(0) as u32,
                0,
                0,
                0,
                0,
            ],
        });
        self.send_ipc_self(ipc).await;
        self.send_inventory_ack(u32::MAX, INVENTORY_ACTION_ACK_GENERAL, 0)
            .await;
        self.send_inventory().await;
    }

    /// Gil an enemy dropped (`actor::kill_gil`), as retail gave it after the kill's EXP: the gil's slot
    /// (UpdateInventorySlot), LootMessage 9 with the amount, then ActorControlSelf 42 (Bob's Chronofoil captures c99fe374
    /// and cc62ad94: 136 drops.)
    pub async fn obtain_dropped_gil(&mut self, gil: u32) {
        add_gil(self, gil);
        self.send_inventory_slot(ContainerType::Currency, 0).await;
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::LootMessage {
            kind: LOOT_MESSAGE_GIL,
            params: [gil, 0, 0, 0, 0, 0, 0],
        });
        self.send_ipc_self(ipc).await;
        self.actor_control_self(ActorControlCategory::GilObtained)
            .await;
    }

    /// An item an event gives (a quest's `player:obtain_item`): retail's InventoryActionAck (0xFFFFFFFF, 6), the slot
    /// (here the whole inventory), ActorControlSelf LogMessage 789 with the item (the "obtained" pop-up), then
    /// EventLogMessage 750 with the item (the log line). Bob's Chronofoil capture b8432613: Dressed to Deceive's two
    /// pieces of attire as it was accepted, one after the other.
    pub async fn obtain_event_item(&mut self, id: u32, quantity: i32) {
        const LOG_MESSAGE_ITEM_POP_UP: u32 = 789;
        const LOG_MESSAGE_ITEM_OBTAINED: u32 = 750;
        let new_item = {
            let mut game_data = self.game_data.lock();
            game_data
                .get_item_info(ItemInfoQuery::ById(id))
                .map(|info| Item::new(&info, quantity))
        };
        let Some(new_item) = new_item else {
            tracing::warn!("An event gave item {id}, which isn't in the game's data");
            return;
        };
        if !self.player_data.inventory.add_obtained_item(new_item) {
            self.send_notice(ERR_INVENTORY_ADD_FAILED).await;
            return;
        }
        self.send_inventory_ack(u32::MAX, INVENTORY_ACTION_ACK_GENERAL, 0)
            .await;
        self.send_inventory().await;
        self.actor_control_self(ActorControlCategory::LogMessage {
            log_message: LOG_MESSAGE_ITEM_POP_UP,
            id,
        })
        .await;
        self.send_event_log_message(LOG_MESSAGE_ITEM_OBTAINED, vec![id])
            .await;
    }

    /// A gear coffer opened (`items/Coffer.lua`): what it holds, a set of pieces per class (a gladiator's sword and
    /// shield, everyone else's one weapon); the first set whose first piece the current class or job can equip (that
    /// item's ClassJobCategory) is given, as an event gives an item, and the coffer used up. Nothing for the class, not
    /// in the bags, no room for the set, or in instanced content (the item's description: it can't be opened there):
    /// nothing happens and the coffer is kept. Retail's own packets for this aren't recorded yet.
    pub async fn open_coffer(&mut self, coffer_id: u32, contents: &[Vec<u32>]) {
        if self.content_finder_condition_id != 0 || self.content_handler_id.is_some() {
            tracing::info!("Coffer {coffer_id} not opened: in instanced content");
            return;
        }
        if self.player_data.inventory.count_in_bags(coffer_id) == 0 {
            tracing::warn!("Coffer {coffer_id} opened, but there's none in the bags");
            return;
        }
        let class_job = self.player_data.classjob.current_class as u8;
        let given: Vec<u32> = {
            let mut game_data = self.game_data.lock();
            contents
                .iter()
                .find(|set| {
                    set.first().is_some_and(|main| {
                        game_data
                            .get_item_info(ItemInfoQuery::ById(*main))
                            .is_some()
                            && {
                                let category = game_data.get_item_classjobcategory(*main);
                                game_data
                                    .get_applicable_classjobs(category as u16)
                                    .contains(&class_job)
                            }
                    })
                })
                .cloned()
                .unwrap_or_default()
        };
        if given.is_empty() {
            tracing::info!("Coffer {coffer_id} holds nothing for class {class_job}");
            return;
        }
        // Room for every piece, tried on a copy first (the coffer's own slot isn't freed until it's opened.)
        let mut trial = self.player_data.inventory.clone();
        let fits = {
            let mut game_data = self.game_data.lock();
            given.iter().all(|id| {
                game_data
                    .get_item_info(ItemInfoQuery::ById(*id))
                    .is_some_and(|info| trial.add_obtained_item(Item::new(&info, 1)))
            })
        };
        if !fits {
            self.send_notice(ERR_INVENTORY_ADD_FAILED).await;
            return;
        }
        tracing::info!("Coffer {coffer_id} opened for class {class_job}: {given:?}");
        // As retail opened a Level 18 Weapon Coffer for a gladiator (Bob's Chronofoil capture c99fe374 of 2026-10-28,
        // Dags: the Carnage Sword and the Iron Lantern Shield): ActorControlSelf 527 (1232, 0, the item) for each
        // piece, the coffer's slot emptied, then for each piece the ack (0xFFFFFFFF, 6), its slot and LogMessage 789
        // (the "obtained" pop-up); no log line of its own.
        const COFFER_OBTAINED: u32 = 527;
        const COFFER_OBTAINED_ARG: u32 = 1232;
        const LOG_MESSAGE_ITEM_POP_UP: u32 = 789;
        for id in &given {
            self.actor_control_self(ActorControlCategory::Unknown {
                category: COFFER_OBTAINED,
                param1: COFFER_OBTAINED_ARG,
                param2: 0,
                param3: *id,
                param4: 0,
                param5: 0,
            })
            .await;
        }
        self.item_used_up(coffer_id).await;
        for id in given {
            let new_item = {
                let mut game_data = self.game_data.lock();
                game_data
                    .get_item_info(ItemInfoQuery::ById(id))
                    .map(|info| Item::new(&info, 1))
            };
            let Some(placed) =
                new_item.and_then(|item| self.player_data.inventory.add_obtained_item_at(item))
            else {
                self.send_notice(ERR_INVENTORY_ADD_FAILED).await;
                continue;
            };
            self.send_inventory_ack(u32::MAX, INVENTORY_ACTION_ACK_GENERAL, 0)
                .await;
            self.send_inventory_slot(placed.0, placed.1).await;
            self.actor_control_self(ActorControlCategory::LogMessage {
                log_message: LOG_MESSAGE_ITEM_POP_UP,
                id,
            })
            .await;
        }
    }

    /// An item won from a coffer (`server/loot.rs`): the LootMessages went to the party already; retail then sent the
    /// winner the ack (0xFFFFFFFF, 6), the item's slot and ActorControlSelf LogMessage 789 with the item (the "obtained"
    /// pop-up), the same for a roll won in a Duty Finder party and a Duty Support coffer's item (Bob's Chronofoil capture
    /// c99fe374 of 2026-10-28: Sastasha, Tam-Tara and Copperbell.)
    pub async fn obtain_loot(&mut self, id: u32) {
        const LOG_MESSAGE_ITEM_POP_UP: u32 = 789;
        let new_item = {
            let mut game_data = self.game_data.lock();
            game_data
                .get_item_info(ItemInfoQuery::ById(id))
                .map(|info| Item::new(&info, 1))
        };
        let Some(placed) =
            new_item.and_then(|item| self.player_data.inventory.add_obtained_item_at(item))
        else {
            self.send_notice(ERR_INVENTORY_ADD_FAILED).await;
            return;
        };
        self.send_inventory_ack(u32::MAX, INVENTORY_ACTION_ACK_GENERAL, 0)
            .await;
        self.send_inventory_slot(placed.0, placed.1).await;
        self.actor_control_self(ActorControlCategory::LogMessage {
            log_message: LOG_MESSAGE_ITEM_POP_UP,
            id,
        })
        .await;
    }

    /// An item of a coffer that's the opener's alone (Duty Support, `server/loot.rs`): as retail, LootMessage 3 (who,
    /// the item, 1) and the item (`obtain_loot`); a unique item they already have is lost instead, LootMessage 8 (who,
    /// the item, 1, 8) then 2 (the item, 1) (Bob's Chronofoil capture c99fe374: a second Acolyte's Skirt in Tam-Tara,
    /// the Plundered Gauntlets and a second Foestriker's Tabard in Copperbell.)
    pub async fn obtain_personal_loot(&mut self, id: u32) {
        let me = self.player_data.character.actor_id.0;
        let unique = self.game_data.lock().item_is_unique(id);
        if unique && self.player_data.inventory.count_owned(id) > 0 {
            for (kind, params) in [
                (
                    LOOT_MESSAGE_ALREADY_HAVE,
                    [me, id, 1, LOOT_ALREADY_HAVE_REASON, 0, 0, 0],
                ),
                (LOOT_MESSAGE_LOST, [id, 1, 0, 0, 0, 0, 0]),
            ] {
                let ipc =
                    ServerZoneIpcSegment::new(ServerZoneIpcData::LootMessage { kind, params });
                self.send_ipc_self(ipc).await;
            }
            return;
        }
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::LootMessage {
            kind: LOOT_MESSAGE_OBTAINED,
            params: [me, id, 1, 0, 0, 0, 0],
        });
        self.send_ipc_self(ipc).await;
        self.obtain_loot(id).await;
    }

    pub async fn send_inventory_ack(&mut self, sequence: u32, action_type: u8, unk1: u8) {
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::InventoryActionAck {
            sequence,
            action_type,
            equipped_items_inventory_type: ContainerType::Inventory0, // TODO: wrong!!
            unk1,
        });
        self.send_ipc_self(ipc).await;
        self.player_data.item_sequence += 1;
    }

    pub async fn send_inventory_transaction_finish(
        &mut self,
        operation_type: ItemOperationKind,
        unk2: u32,
    ) {
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::InventoryTransactionFinish {
            sequence: self.player_data.item_sequence,
            sequence_repeat: self.player_data.item_sequence,
            operation_type,
            unk2,
        });
        self.send_ipc_self(ipc).await;
        self.player_data.item_sequence += 1;
    }

    /// Swaps two items from two (possibly different) containers and informs the client of this change.
    pub async fn swap_items(
        &mut self,
        src_container: ContainerType,
        src_index: u16,
        dst_container: ContainerType,
        dst_index: u16,
    ) {
        let Some(src_item) = self
            .player_data
            .inventory
            .get_item(src_container, src_index)
        else {
            tracing::warn!(
                "Unable to swap items: src_container was an invalid container for this operation: {src_container}!"
            );
            return;
        };

        // move src item into dst slot
        let Some(dst_slot) = self
            .player_data
            .inventory
            .get_item_mut(dst_container, dst_index)
        else {
            tracing::warn!(
                "Unable to swap items: dst_container was an invalid container for this operation: {dst_container}!"
            );
            return;
        };

        let dst_item = *dst_slot;
        let was_empty = dst_item.quantity == 0;
        dst_slot.clone_from(&src_item);

        // move dst item into src slot
        let src_slot = self
            .player_data
            .inventory
            .get_item_mut(src_container, src_index)
            .unwrap(); // This unwrap should be fine since we've reached this point.
        src_slot.clone_from(&dst_item);

        // Then inform the client of the updated slots, we have to do this since this is caused server-side.
        {
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::UpdateInventorySlot(ItemInfo {
                sequence: self.player_data.item_sequence,
                container: src_container,
                slot: src_index,
                ..dst_item.into()
            }));
            self.send_ipc_self(ipc).await;
            self.player_data.item_sequence += 1;
        }

        {
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::UpdateInventorySlot(ItemInfo {
                sequence: self.player_data.item_sequence,
                container: dst_container,
                slot: dst_index,
                ..src_item.into()
            }));
            self.send_ipc_self(ipc).await;
            self.player_data.item_sequence += 1;
        }

        // And also update the current transaction
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::InventoryTransaction {
            sequence: self.player_data.item_sequence,
            operation_type: if was_empty {
                ItemOperationKind::Move
            } else {
                ItemOperationKind::Exchange
            },

            src_actor_id: self.player_data.character.actor_id,
            src_storage_id: src_container,
            src_container_index: src_index,
            src_stack: src_item.quantity as u32,
            src_item_id: src_item.item_id,

            dst_actor_id: self.player_data.character.actor_id,
            dst_storage_id: dst_container,
            dst_container_index: dst_index,
            dst_stack: dst_item.quantity as u32,
            dst_item_id: dst_item.item_id,

            dummy_container: ContainerType::Equipped,
        });
        self.send_ipc_self(ipc).await;
    }

    /// Changes the class based on the weapon equipped.
    pub async fn change_class_based_on_weapon(&mut self) {
        // Check the weapon's compatible class jobs:
        let classjobs;
        {
            let mut game_data = self.game_data.lock();

            let weapon = self.player_data.inventory.equipped.main_hand.item_id;
            let item_info = game_data
                .get_item_info(ItemInfoQuery::ById(weapon))
                .unwrap();
            classjobs = game_data.get_applicable_classjobs(item_info.classjob_category as u16);
        }

        let previous_class = self.player_data.classjob.current_class;
        self.player_data.classjob.current_class = classjobs.first().copied().unwrap() as i32;
        assert!(self.player_data.classjob.current_class != 0); // If this is 0, then something went seriously wrong.

        // UpdateClassInfo comes with the equipment (`update_equip`), after the class change, as on retail.
        if self.player_data.classjob.current_class != previous_class {
            self.finish_changing_class().await;
        }
    }

    /// Changes the class based on the soul crystal equipped.
    pub async fn change_class_based_on_soul_crystal(&mut self) {
        // Then check the soul crystal:
        let soul_crystal = self.player_data.inventory.equipped.soul_crystal;
        if soul_crystal.quantity > 0 {
            let classjob_id;
            {
                let mut game_data = self.game_data.lock();
                classjob_id = game_data.get_applicable_classjob(soul_crystal.item_id);
            }

            if let Some(classjob_id) = classjob_id {
                let previous_class = self.player_data.classjob.current_class;
                self.player_data.classjob.current_class = classjob_id as i32;
                assert!(self.player_data.classjob.current_class != 0); // If this is 0, then something went seriously wrong.

                // As for the weapon: UpdateClassInfo comes with the equipment.
                if self.player_data.classjob.current_class != previous_class {
                    self.finish_changing_class().await;
                }
            }
        }
    }

    /// Removes armor that's incompatible with your current class.
    pub async fn remove_incompatible_armor(&mut self, action: &ItemOperation) {
        // NOTE: This has to match client behavior exactly! As this happens client-side.

        let mut game_data = self.game_data.lock();

        // First remove incompatible classjob gear.
        for slot in EquipSlot::iter() {
            let item = self.player_data.inventory.equipped.get_slot(slot as u16);
            if item.quantity > 0 {
                let classjob_category = game_data.get_item_classjobcategory(item.item_id);
                let classjobs = game_data.get_applicable_classjobs(classjob_category as u16);
                if !classjobs.contains(&(self.player_data.classjob.current_class as u8)) {
                    tracing::info!(
                        "Unequipping item in slot {slot:#?} because it's incompatible with the current class."
                    );
                    self.player_data.inventory.unequip_equipment(slot as u16);
                }
            }
        }

        // Then unequip slots that are restricted by any body armor.
        let body_item = self
            .player_data
            .inventory
            .equipped
            .get_slot(EquipSlot::Body as u16);
        if body_item.quantity > 0
            && let Some(body_item_info) =
                game_data.get_item_info(ItemInfoQuery::ById(body_item.item_id))
        {
            let body_restrictions = [
                (EquipSlot::Head, body_item_info.equip_restrictions.head),
                (EquipSlot::Hands, body_item_info.equip_restrictions.hands),
                (EquipSlot::Legs, body_item_info.equip_restrictions.legs),
                (EquipSlot::Feet, body_item_info.equip_restrictions.feet),
            ];
            for (slot, restriction) in body_restrictions {
                if action.dst_storage_id == ContainerType::Equipped
                    && restriction == EQUIP_RESTRICTED
                {
                    // If body was equipped, remove this restricted gear.
                    if action.dst_container_index == EquipSlot::Body as u16 {
                        tracing::info!(
                            "Unequipping item in slot {slot:#?} because it's incompatible with the current body armor."
                        );
                        self.player_data.inventory.unequip_equipment(slot as u16);
                    }
                    // Otherwise, we're equipping into a restricted slot, so remove the body instead and exit the loop.
                    else if action.dst_container_index == slot as u16 {
                        tracing::info!(
                            "Unequipping item in slot Body because it's incompatible with the current {slot:#?} armor."
                        );
                        self.player_data
                            .inventory
                            .unequip_equipment(EquipSlot::Body as u16);
                        break;
                    }
                }
            }
        }
    }

    /// Equips a gearset from the client's EquipGearset (or EquipGearsetGlamour): each slot's item from where the client
    /// says it is, then GearSetEquipped, the finish, the equipped container, the model and the stats.
    pub async fn equip_gearset(
        &mut self,
        gearset_index: u32,
        containers: &[ContainerType; 14],
        indices: &[i16; 14],
    ) {
        // TODO: handle missing items, full inventory and such
        for slot in 0..14 {
            let from_slot = indices[slot];
            let from_container = containers[slot];

            if from_container == ContainerType::Equipped {
                continue;
            }

            let mut from_item = Item::default();

            if from_slot != -1
                && let Some(the_item) = self
                    .player_data
                    .inventory
                    .get_item(from_container, from_slot as u16)
            {
                from_item = the_item;
            }

            let equipped_item = self.player_data.inventory.equipped.get_slot(slot as u16);

            if from_slot != -1 && from_item.is_empty_slot() {
                // The client names an item where the server has none: its view of that bag is out of date. Leave this
                // piece as it is (rather than take off what's worn) and tell the client what's really in the slot.
                tracing::warn!(
                    "Gear set {gearset_index} wants {from_container:?} {from_slot} for slot {slot}, which is empty here."
                );
                self.send_inventory_slot(from_container, from_slot as u16)
                    .await;
                continue;
            }

            if !from_item.is_empty_slot() && !equipped_item.is_empty_slot() {
                // If there is something equipped and a replacement for it, we must swap.
                self.swap_items(
                    from_container,
                    from_slot as u16,
                    ContainerType::Equipped,
                    slot as u16,
                )
                .await;
            } else if !from_item.is_empty_slot() && equipped_item.is_empty_slot() {
                // If there is nothing equipped but a new item in that slot, we just have to move it.
                // TODO: be a little smarter about this maybe?
                self.swap_items(
                    from_container,
                    from_slot as u16,
                    ContainerType::Equipped,
                    slot as u16,
                )
                .await;
            } else if from_item.is_empty_slot() && !equipped_item.is_empty_slot() {
                // If there is something equipped but the slot is empty in the gearset, we have to move it somewhere.
                let target_container_type = ContainerType::from_equip_slot(slot as u8);

                if let Some(target_container) = self
                    .player_data
                    .inventory
                    .get_container(target_container_type)
                    && let Some(free_slot) = get_next_free_slot(target_container)
                {
                    self.swap_items(
                        ContainerType::Equipped,
                        slot as u16,
                        target_container_type,
                        free_slot,
                    )
                    .await;
                }
            }
        }

        self.gearset_equipped = true;

        // Inform the client that the gearset was successfully equipped.
        self.actor_control_self(ActorControlCategory::GearSetEquipped {
            gearset_index: gearset_index,
        })
        .await;

        // And that we're done modifying the inventory.
        self.send_inventory_transaction_finish(ItemOperationKind::UnkGearSetEquipped, 3584)
            .await;

        // Retail also re-sends the equipped container
        self.send_equipped_inventory().await;
        self.inform_equip().await;

        // Change class as needed.
        self.change_class_based_on_weapon().await;

        // Then finally, resend stats.
        self.send_stats().await;
    }
}

/// Adds gil to the player's gil slot (the currency container's slot 0.)
pub fn add_gil(connection: &mut ZoneConnection, gil: u32) {
    let slot = connection.player_data.inventory.currency.get_slot_mut(0);
    slot.quantity = slot.quantity.saturating_add_unsigned(gil);
}

/// ActorControlSelf 805, retail's after every gearset (`update_equip`.)
const GEARSET_EQUIPPED_END: u32 = 805;

/// LootMessage's kind for gil an enemy dropped (the amount.)
const LOOT_MESSAGE_GIL: u32 = 9;
/// LootMessage's kind for an item obtained directly (an enemy's drop, a Duty Support coffer's item.)
const LOOT_MESSAGE_OBTAINED: u32 = 3;
/// LootMessage's kinds for a unique item one already has: 8 (with the reason, 8), then 2, the item lost.
const LOOT_MESSAGE_ALREADY_HAVE: u32 = 8;
const LOOT_ALREADY_HAVE_REASON: u32 = 8;
const LOOT_MESSAGE_LOST: u32 = 2;
