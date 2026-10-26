use diesel::{
    backend::Backend,
    deserialize::{self, FromSqlRow},
    expression::AsExpression,
    serialize,
    sql_types::Text,
    sqlite::Sqlite,
};
use icarus::{ClassJob::ClassJobSheet, Race::RaceSheet};
use kawari::{
    common::{
        ContainerType, ItemOperationKind, LegacyEquipmentModelId, TerritoryIntendedUse,
        WeaponModelId,
    },
    config::get_config,
    ipc::zone::ItemInfo,
};
use serde::{Deserialize, Serialize};

use kawari::ipc::zone::ItemOperation;

mod buyback;

/// The Disciples of the Hand and of the Land: Carpenter (8) to Fisher (18) (the ClassJob sheet's order.)
pub const CLASSJOB_FIRST_HAND_OR_LAND: u8 = 8;
pub const CLASSJOB_LAST_HAND_OR_LAND: u8 = 18;

pub use buyback::BuyBackList;

mod equipped;
pub use equipped::{EQUIP_RESTRICTED, EquippedStorage};

mod generic;
pub use generic::GenericStorage;

mod item;
pub use item::Item;

mod storage;
pub use storage::{Storage, get_next_free_slot};

mod currency;
pub use currency::{CurrencyKind, CurrencyStorage};

mod iterators;
pub use iterators::{InventoryIterator, get_container_type};

mod crystals;
pub use crystals::{CrystalKind, CrystalsStorage};

use crate::{GameData, ItemInfoQuery};

/// A retainer's seven pages, where entrusted items go.
const RETAINER_PAGES: [ContainerType; 7] = [
    ContainerType::RetainerPage1,
    ContainerType::RetainerPage2,
    ContainerType::RetainerPage3,
    ContainerType::RetainerPage4,
    ContainerType::RetainerPage5,
    ContainerType::RetainerPage6,
    ContainerType::RetainerPage7,
];
/// The player's four bags, where withdrawn items go.
const PLAYER_BAGS: [ContainerType; 4] = [
    ContainerType::Inventory0,
    ContainerType::Inventory1,
    ContainerType::Inventory2,
    ContainerType::Inventory3,
];

/// The chocobo saddlebag's two pages, where its items go.
pub const SADDLEBAG_PAGES: [ContainerType; 2] =
    [ContainerType::SaddleBag1, ContainerType::SaddleBag2];
/// The premium saddlebag's (not offered: sent empty, as retail did for an account without it.)
pub const PREMIUM_SADDLEBAG_PAGES: [ContainerType; 2] = [
    ContainerType::PremiumSaddleBag1,
    ContainerType::PremiumSaddleBag2,
];

/// An item's HQ flag (`Item::item_flags`; ItemFlags.HighQuality in FFXIVClientStructs.)
pub const ITEM_FLAG_HQ: u8 = 1;

const MAX_NORMAL_STORAGE: usize = 35;
pub const MAX_LARGE_STORAGE: usize = 50;

#[derive(Debug, Clone, Deserialize, Serialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = Text)]
pub struct Inventory {
    pub equipped: EquippedStorage,
    pub pages: [GenericStorage<MAX_NORMAL_STORAGE>; 4],
    pub armoury_main_hand: GenericStorage<MAX_LARGE_STORAGE>,
    pub armoury_head: GenericStorage<MAX_NORMAL_STORAGE>,
    pub armoury_body: GenericStorage<MAX_NORMAL_STORAGE>,
    pub armoury_hands: GenericStorage<MAX_NORMAL_STORAGE>,
    pub armoury_legs: GenericStorage<MAX_NORMAL_STORAGE>,
    pub armoury_feet: GenericStorage<MAX_NORMAL_STORAGE>,
    pub armoury_off_hand: GenericStorage<MAX_NORMAL_STORAGE>,
    pub armoury_earring: GenericStorage<MAX_NORMAL_STORAGE>,
    pub armoury_necklace: GenericStorage<MAX_NORMAL_STORAGE>,
    pub armoury_bracelet: GenericStorage<MAX_NORMAL_STORAGE>,
    pub armoury_rings: GenericStorage<MAX_LARGE_STORAGE>,
    pub armoury_soul_crystal: GenericStorage<MAX_NORMAL_STORAGE>,
    pub currency: CurrencyStorage,
    pub crystals: CrystalsStorage,
    pub key_items: GenericStorage<MAX_NORMAL_STORAGE>,
    /// The glamour dresser, glamour plates and armoire (`crate::glamour`); missing in inventories saved before them.
    #[serde(default)]
    pub glamour: crate::glamour::GlamourStorage,
    /// The chocobo saddlebag's two pages of 35 (4000, 4001): sent when it's opened (client trigger 444), not at login.
    #[serde(default = "Inventory::empty_saddlebag")]
    pub saddlebag: [GenericStorage<MAX_NORMAL_STORAGE>; 2],
    /// The retainer called at a summoning bell, if any: their containers (10000-12002) work like the player's own while
    /// they're out. Kept in the retainer table, not with the player's inventory.
    #[serde(skip)]
    pub retainer: Option<Box<crate::retainer::RetainerInventory>>,
    /// Items the client put in an NPC's trade window (moved to HandIn, 2005) during the current event: their bag and
    /// slot, the item and how many. The client only points at them; they stay in their bags until the event takes them
    /// (`take_handed_in`), as retail took Color Your World's on its reward (Bob's Chronofoil capture of 2026-10-25.)
    #[serde(skip)]
    pub handed_in: Vec<(ContainerType, u16, u32, u32)>,
}

impl serialize::ToSql<Text, Sqlite> for Inventory {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(serde_json::to_string(&self).unwrap());
        Ok(serialize::IsNull::No)
    }
}

impl deserialize::FromSql<Text, Sqlite> for Inventory {
    fn from_sql(mut bytes: <Sqlite as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        Ok(serde_json::from_str(bytes.read_text())
            .ok()
            .unwrap_or_default())
    }
}

impl Default for Inventory {
    fn default() -> Self {
        // Right now we only use this for adding items to the main inventory.
        Self {
            equipped: EquippedStorage::default(),
            pages: [
                GenericStorage::new(ContainerType::Inventory0),
                GenericStorage::new(ContainerType::Inventory1),
                GenericStorage::new(ContainerType::Inventory2),
                GenericStorage::new(ContainerType::Inventory3),
            ],
            armoury_main_hand: GenericStorage::new(ContainerType::ArmoryWeapon),
            armoury_head: GenericStorage::new(ContainerType::ArmoryHead),
            armoury_body: GenericStorage::new(ContainerType::ArmoryBody),
            armoury_hands: GenericStorage::new(ContainerType::ArmoryHand),
            armoury_legs: GenericStorage::new(ContainerType::ArmoryLeg),
            armoury_feet: GenericStorage::new(ContainerType::ArmoryFoot),
            armoury_off_hand: GenericStorage::new(ContainerType::ArmoryOffWeapon),
            armoury_earring: GenericStorage::new(ContainerType::ArmoryEarring),
            armoury_necklace: GenericStorage::new(ContainerType::ArmoryNeck),
            armoury_bracelet: GenericStorage::new(ContainerType::ArmoryWrist),
            armoury_rings: GenericStorage::new(ContainerType::ArmoryRing),
            armoury_soul_crystal: GenericStorage::new(ContainerType::ArmorySoulCrystal),
            currency: CurrencyStorage::default(),
            crystals: CrystalsStorage::default(),
            key_items: GenericStorage::new(ContainerType::KeyItems),
            glamour: Default::default(),
            saddlebag: Self::empty_saddlebag(),
            retainer: None,
            handed_in: Vec::new(),
        }
    }
}

impl Inventory {
    /// Equip the starting items for a given classjob
    pub fn equip_classjob_items(&mut self, classjob_id: u16, game_data: &mut GameData) {
        let config = get_config();
        let sheet =
            ClassJobSheet::read_from(&mut game_data.resource, config.world.language()).unwrap();
        let row = sheet.row(classjob_id as u32).unwrap();

        let main_hand_id = row.ItemStartingWeaponMainHand as u32;
        self.equipped.main_hand = Item::new(
            &game_data
                .get_item_info(ItemInfoQuery::ById(main_hand_id))
                .unwrap(),
            1,
        );

        // TODO: don't hardcode
        self.equipped.ears = Item::new(
            &game_data
                .get_item_info(ItemInfoQuery::ById(0x3b1b))
                .unwrap(),
            1,
        );
        self.equipped.neck = Item::new(
            &game_data
                .get_item_info(ItemInfoQuery::ById(0x3b1a))
                .unwrap(),
            1,
        );
        self.equipped.wrists = Item::new(
            &game_data
                .get_item_info(ItemInfoQuery::ById(0x3b1c))
                .unwrap(),
            1,
        );
        // FIXME: I think this is actually based on a choice in the opening, and also defined in OpeningSystemDefine Excel sheet:
        self.equipped.right_ring = Item::new(
            &game_data
                .get_item_info(ItemInfoQuery::ById(0x114a))
                .unwrap(),
            1,
        );
        self.equipped.left_ring = Item::new(
            &game_data
                .get_item_info(ItemInfoQuery::ById(0x3b1d))
                .unwrap(),
            1,
        );
    }

    /// Equip the starting items for a given race
    pub fn equip_racial_items(&mut self, race_id: u8, gender: u8, game_data: &mut GameData) {
        let config = get_config();
        let sheet = RaceSheet::read_from(&mut game_data.resource, config.world.language()).unwrap();
        let row = sheet.row(race_id as u32).unwrap();

        let ids = if gender == 0 {
            [
                row.RSEMBody as u32,
                row.RSEMHands as u32,
                row.RSEMLegs as u32,
                row.RSEMFeet as u32,
            ]
        } else {
            [
                row.RSEFBody as u32,
                row.RSEFHands as u32,
                row.RSEFLegs as u32,
                row.RSEFFeet as u32,
            ]
        };

        self.equipped.body = Item::new(
            &game_data
                .get_item_info(ItemInfoQuery::ById(ids[0]))
                .unwrap(),
            1,
        );
        self.equipped.hands = Item::new(
            &game_data
                .get_item_info(ItemInfoQuery::ById(ids[1]))
                .unwrap(),
            1,
        );
        self.equipped.legs = Item::new(
            &game_data
                .get_item_info(ItemInfoQuery::ById(ids[2]))
                .unwrap(),
            1,
        );
        self.equipped.feet = Item::new(
            &game_data
                .get_item_info(ItemInfoQuery::ById(ids[3]))
                .unwrap(),
            1,
        );
    }

    /// Helper functions to reduce boilerplate
    pub fn get_item_mut(
        &mut self,
        storage_id: ContainerType,
        storage_index: u16,
    ) -> Option<&mut Item> {
        if let Some(container) = self.get_container_mut(&storage_id) {
            return Some(container.get_slot_mut(storage_index));
        }

        None
    }

    pub fn get_item(&self, storage_id: ContainerType, storage_index: u16) -> Option<Item> {
        if let Some(container) = self.get_container(storage_id) {
            return Some(*container.get_slot(storage_index));
        }

        None
    }

    pub fn process_action(&mut self, action: &ItemOperation) {
        // An NPC trade window: the item is only pointed at (retail moved it into HandIn and left its bag slot alone
        // until the reward took it; Bob's Chronofoil capture of 2026-10-25, Color Your World.)
        if matches!(
            action.operation_type,
            ItemOperationKind::Move | ItemOperationKind::Exchange
        ) {
            if action.dst_storage_id == ContainerType::HandIn {
                if let Some(item) = self.get_item(action.src_storage_id, action.src_container_index)
                    && item.quantity > 0
                {
                    let amount = if action.src_stack > 0 {
                        action.src_stack.min(item.quantity as u32)
                    } else {
                        item.quantity as u32
                    };
                    self.handed_in.retain(|(container, slot, ..)| {
                        (*container, *slot) != (action.src_storage_id, action.src_container_index)
                    });
                    self.handed_in.push((
                        action.src_storage_id,
                        action.src_container_index,
                        item.item_id,
                        amount,
                    ));
                }
                return;
            }
            if action.src_storage_id == ContainerType::HandIn {
                // Taken back out of the trade window.
                let index = action.src_container_index as usize;
                if index < self.handed_in.len() {
                    self.handed_in.remove(index);
                }
                return;
            }
        }
        match action.operation_type {
            ItemOperationKind::Discard => {
                if let Some(src_item) =
                    self.get_item_mut(action.src_storage_id, action.src_container_index)
                {
                    *src_item = Item::default();
                }
            }
            ItemOperationKind::CombineStack => {
                let src_item;
                {
                    if let Some(original_item) =
                        self.get_item_mut(action.src_storage_id, action.src_container_index)
                    {
                        src_item = *original_item;
                        *original_item = Item::default();
                    } else {
                        return;
                    }
                }

                if let Some(dst_item) =
                    self.get_item_mut(action.dst_storage_id, action.dst_container_index)
                {
                    dst_item.quantity = dst_item
                        .quantity
                        .saturating_add(src_item.quantity)
                        .min(dst_item.stack_size as i32);
                }
            }
            ItemOperationKind::SplitStack => {
                let mut src_item;
                {
                    let Some(original_item) =
                        self.get_item_mut(action.src_storage_id, action.src_container_index)
                    else {
                        tracing::warn!(
                            "Client sent a bogus storage id: {}! Rejecting item operation!",
                            action.src_storage_id
                        );
                        return;
                    };
                    if original_item.quantity >= action.dst_stack as i32 {
                        original_item.quantity -= action.dst_stack as i32;
                        src_item = *original_item;
                        src_item.quantity = action.dst_stack as i32
                    } else {
                        tracing::warn!(
                            "Client sent a bogus split amount: {}! Rejecting item operation!",
                            action.dst_stack
                        );
                        return;
                    }
                }

                if let Some(dst_item) =
                    self.get_item_mut(action.dst_storage_id, action.dst_container_index)
                {
                    dst_item.clone_from(&src_item);
                }
            }
            ItemOperationKind::Exchange | ItemOperationKind::Move => {
                let src_item;

                // Clear existing item so add in next free slot checks work.
                {
                    if let Some(src_slot) =
                        self.get_item_mut(action.src_storage_id, action.src_container_index)
                    {
                        src_item = *src_slot;
                        src_slot.quantity = 0;
                    } else {
                        return;
                    }
                }

                // move src item into dst slot
                if let Some(dst_slot) =
                    self.get_item_mut(action.dst_storage_id, action.dst_container_index)
                {
                    let dst_item = *dst_slot;
                    dst_slot.clone_from(&src_item);

                    // When equipping from the main inventory, the client puts whatever was equipped into the Armoury
                    // Chest instead of the slot the new item came from (if it has room.)
                    // NOTE: This has to match client behavior exactly! As this happens client-side.
                    let from_main_inventory = matches!(
                        action.src_storage_id,
                        ContainerType::Inventory0
                            | ContainerType::Inventory1
                            | ContainerType::Inventory2
                            | ContainerType::Inventory3
                    );
                    if from_main_inventory
                        && action.dst_storage_id == ContainerType::Equipped
                        && dst_item.quantity > 0
                        && let Some(destination) =
                            self.add_in_next_free_armory_slot(action.dst_container_index)
                    {
                        self.add_in_slot(dst_item, &destination.container, destination.slot);
                    } else if let Some(src_slot) =
                        self.get_item_mut(action.src_storage_id, action.src_container_index)
                    {
                        // move dst item into src slot
                        src_slot.clone_from(&dst_item);
                    }
                }
            }
            ItemOperationKind::RetainerMarketList
            | ItemOperationKind::RetainerMarketListCrystal => {
                self.list_on_retainer_market(action);
            }
            kind => {
                tracing::warn!(
                    "Unhandled item operation {kind:?}, the server's inventory didn't change!"
                );
            }
        }
    }

    /// Takes the items handed in through an NPC's trade window out of their bags (and forgets them); returns how many
    /// stacks were taken. Each must still be the item that was handed in.
    pub fn take_handed_in(&mut self) -> usize {
        let mut taken = 0;
        for (container, slot, item_id, amount) in std::mem::take(&mut self.handed_in) {
            if let Some(item) = self.get_item_mut(container, slot)
                && item.item_id == item_id
                && item.quantity >= amount as i32
            {
                item.quantity -= amount as i32;
                if item.quantity == 0 {
                    *item = Item::default();
                }
                taken += 1;
            }
        }
        taken
    }

    /// Entrusting to or withdrawing from the called retainer from their menu (`ItemOperationKind::RetainerEntrust` /
    /// `RetainerWithdraw`): `src_stack` of the source item (all of it when 0) goes where the server picks, as retail
    /// does (Bob's 2026-10-18 capture): onto the same item's stack if it has room, otherwise the first empty slot of the
    /// retainer's pages (entrusting) or the player's bags (withdrawing.) Returns the source and destination slots, or
    /// None if nothing moved (empty source, no room, no retainer called.)
    pub fn transfer_with_retainer(
        &mut self,
        action: &ItemOperation,
    ) -> Option<[(ContainerType, u16); 2]> {
        self.retainer.as_ref()?;
        let destinations: &[ContainerType] =
            if action.operation_type == ItemOperationKind::RetainerEntrust {
                &RETAINER_PAGES
            } else {
                &PLAYER_BAGS
            };
        self.transfer_to(action, destinations)
    }

    /// `src_stack` of an operation's source item (all of it when 0) onto the same item's stack with room in
    /// `destinations`, else their first empty slot.
    fn transfer_to(
        &mut self,
        action: &ItemOperation,
        destinations: &[ContainerType],
    ) -> Option<[(ContainerType, u16); 2]> {
        let source = self.get_item(action.src_storage_id, action.src_container_index)?;
        if source.is_empty_slot() {
            return None;
        }
        let quantity = match action.src_stack as i32 {
            0 => source.quantity,
            wanted => wanted.min(source.quantity),
        };
        // The same item's stack with room first, then the first empty slot.
        let mut chosen = None;
        if source.stack_size > 1 {
            'stacks: for container_type in destinations {
                let Some(container) = self.get_container(*container_type) else {
                    continue;
                };
                for slot in 0..container.max_slots() as u16 {
                    let item = container.get_slot(slot);
                    if !item.is_empty_slot()
                        && item.item_id == source.item_id
                        && item.item_flags == source.item_flags
                        && item.quantity + quantity <= source.stack_size as i32
                    {
                        chosen = Some((*container_type, slot));
                        break 'stacks;
                    }
                }
            }
        }
        if chosen.is_none() {
            chosen = destinations.iter().find_map(|container_type| {
                let container = self.get_container(*container_type)?;
                get_next_free_slot(container).map(|slot| (*container_type, slot))
            });
        }
        let (dst_container, dst_slot) = chosen?;

        let destination = self.get_item_mut(dst_container, dst_slot)?;
        if destination.is_empty_slot() {
            *destination = Item { quantity, ..source };
        } else {
            destination.quantity += quantity;
        }
        let source_slot = self.get_item_mut(action.src_storage_id, action.src_container_index)?;
        source_slot.quantity -= quantity;
        if source_slot.quantity <= 0 {
            *source_slot = Item::default();
        }
        Some([
            (action.src_storage_id, action.src_container_index),
            (dst_container, dst_slot),
        ])
    }

    /// Gil to or from the called retainer (`ItemOperationKind::RetainerGil`): retail's entrust carried the amount in
    /// `src_stack` (and `dst_stack`), its "withdraw everything" all zeros (Bob's 2026-10-18 capture; a partial
    /// withdrawal wasn't captured.) Returns false if there's no retainer or not enough gil.
    pub fn transfer_gil_with_retainer(&mut self, action: &ItemOperation) -> bool {
        let Some(retainer) = self.retainer.as_mut() else {
            return false;
        };
        let retainer_gil = &mut retainer.gil.slots[0];
        let player_gil = &mut self.currency.gil;
        let amount = action.src_stack as i32;
        if amount > 0 {
            if player_gil.quantity < amount {
                return false;
            }
            player_gil.quantity -= amount;
            retainer_gil.quantity += amount;
        } else {
            player_gil.quantity += retainer_gil.quantity;
            retainer_gil.quantity = 0;
        }
        retainer_gil.item_id = player_gil.item_id;
        true
    }

    /// Puts `src_stack` of the item at the source up for sale in the called retainer's market slot at the destination,
    /// at `dst_item_id` gil each (see `ItemOperationKind::RetainerMarketList`.)
    fn list_on_retainer_market(&mut self, action: &ItemOperation) {
        let Some(source) = self.get_item(action.src_storage_id, action.src_container_index) else {
            return;
        };
        let quantity = (action.src_stack as i32).clamp(1, source.quantity.max(1));
        let Some(retainer) = self.retainer.as_mut() else {
            tracing::warn!("Listing an item for sale without a retainer called, ignoring it!");
            return;
        };
        let slot = action.dst_container_index as usize;
        if slot >= retainer.market.slots.len() || source.is_empty_slot() {
            return;
        }
        retainer.market.slots[slot] = Item { quantity, ..source };
        if let Some(price) = retainer.market_prices.get_mut(slot) {
            *price = action.dst_item_id;
        }
        if let Some(src_slot) = self.get_item_mut(action.src_storage_id, action.src_container_index)
        {
            src_slot.quantity -= quantity;
            // Crystals keep their slot (and item ID) when there are none left.
            if src_slot.quantity <= 0 && action.src_storage_id != ContainerType::Crystals {
                *src_slot = Item::default();
            }
        }
    }

    fn add_in_empty_slot(&mut self, item: Item) -> Option<ItemInfo> {
        for page in &mut self.pages {
            for (slot_index, slot) in page.slots.iter_mut().enumerate() {
                if slot.quantity == 0 {
                    slot.clone_from(&item);
                    return Some(ItemInfo {
                        slot: slot_index as u16,
                        container: page.kind,
                        ..(*slot).into()
                    });
                }
            }
        }
        None
    }

    /// Adds an item where it belongs: crystals to the crystal bag, anything else to the next free slot. Returns false
    /// if there wasn't room.
    pub fn add_obtained_item(&mut self, item: Item) -> bool {
        self.add_obtained_item_at(item).is_some()
    }

    /// `add_obtained_item`, saying which slot it went into (for sending only that slot, as retail does.)
    pub fn add_obtained_item_at(&mut self, item: Item) -> Option<(ContainerType, u16)> {
        if let Some(kind) = CrystalKind::from_repr(item.item_id) {
            let slot = self.crystals.get_item_for_id(kind);
            if slot.quantity + item.quantity > item.stack_size as i32 {
                return None;
            }
            slot.quantity += item.quantity;
            return Some((
                ContainerType::Crystals,
                CrystalsStorage::get_slot_for_id(kind),
            ));
        }
        self.add_in_next_free_slot(item)
            .map(|info| (info.container, info.slot))
    }

    /// Uses up one of an item from the bags (the first stack found), e.g. one that's been used. Returns false if they
    /// have none.
    pub fn use_up_one(&mut self, item_id: u32) -> bool {
        self.use_up_one_at(item_id).is_some()
    }

    /// How many of an item are in the bags.
    pub fn count_in_bags(&self, item_id: u32) -> u32 {
        self.pages
            .iter()
            .flat_map(|page| (0..page.max_slots() as u16).map(move |index| page.get_slot(index)))
            .filter(|slot| slot.item_id == item_id)
            .map(|slot| slot.quantity.max(0) as u32)
            .sum()
    }

    /// `use_up_one`, saying which slot it came from.
    pub fn use_up_one_at(&mut self, item_id: u32) -> Option<(ContainerType, u16)> {
        for page in &mut self.pages {
            for index in 0..page.max_slots() as u16 {
                let slot = page.get_slot_mut(index);
                if slot.item_id == item_id && slot.quantity > 0 {
                    slot.quantity -= 1;
                    if slot.quantity == 0 {
                        *slot = Item::default();
                    }
                    return Some((page.kind, index));
                }
            }
        }
        None
    }

    fn empty_saddlebag() -> [GenericStorage<MAX_NORMAL_STORAGE>; 2] {
        SADDLEBAG_PAGES.map(GenericStorage::new)
    }

    /// Into or out of the chocobo saddlebag from its window (`ItemOperationKind::SaddlebagStore` / `SaddlebagTake`),
    /// the server picking the slot as for a retainer: onto the same item's stack if it has room, else the first empty
    /// slot (Bob's Chronofoil capture of 2026-10-24: a Store went onto a stack of 37, then into a free slot.) Returns
    /// the source and destination slots, or None if nothing moved.
    pub fn transfer_with_saddlebag(
        &mut self,
        action: &ItemOperation,
    ) -> Option<[(ContainerType, u16); 2]> {
        let destinations: &[ContainerType] =
            if action.operation_type == ItemOperationKind::SaddlebagStore {
                &SADDLEBAG_PAGES
            } else {
                &PLAYER_BAGS
            };
        self.transfer_to(action, destinations)
    }

    pub fn add_in_next_free_slot(&mut self, item: Item) -> Option<ItemInfo> {
        if item.stack_size > 1 {
            for page in &mut self.pages {
                for (slot_index, slot) in page.slots.iter_mut().enumerate() {
                    // HQ and NQ (item_flags) don't share a stack.
                    if slot.item_id == item.item_id
                        && slot.item_flags == item.item_flags
                        && slot.quantity + item.quantity <= item.stack_size as i32
                    {
                        slot.quantity += item.quantity;
                        return Some(ItemInfo {
                            slot: slot_index as u16,
                            container: page.kind,
                            ..(*slot).into()
                        });
                    }
                }
            }
        }

        // If we didn't find any stacks, or the item isn't stackable, try again to find an empty inventory slot.
        self.add_in_empty_slot(item)
    }

    pub fn add_in_next_free_armory_slot(&self, equip_index: u16) -> Option<ItemInfo> {
        let container_type = ContainerType::from_equip_slot(equip_index as u8);

        if let Some(container) = self.get_container(container_type) {
            for i in 0..container.max_slots() {
                if container.get_slot(i as u16).quantity == 0 {
                    return Some(ItemInfo {
                        slot: i as u16,
                        container: container_type,
                        ..(*container.get_slot(i as u16)).into()
                    });
                }
            }
        }

        None
    }

    pub fn add_in_slot(&mut self, item: Item, container_type: &ContainerType, index: u16) {
        let Some(container) = self.get_container_mut(container_type) else {
            return;
        };

        let slot = container.get_slot_mut(index);
        slot.clone_from(&item);
    }

    fn get_container_mut(&mut self, container_type: &ContainerType) -> Option<&mut dyn Storage> {
        match container_type {
            ContainerType::Inventory0 => Some(&mut self.pages[0]),
            ContainerType::Inventory1 => Some(&mut self.pages[1]),
            ContainerType::Inventory2 => Some(&mut self.pages[2]),
            ContainerType::Inventory3 => Some(&mut self.pages[3]),
            ContainerType::Equipped => Some(&mut self.equipped),
            ContainerType::Currency => Some(&mut self.currency),
            ContainerType::Crystals => Some(&mut self.crystals),
            ContainerType::ArmoryOffWeapon => Some(&mut self.armoury_off_hand),
            ContainerType::ArmoryHead => Some(&mut self.armoury_head),
            ContainerType::ArmoryBody => Some(&mut self.armoury_body),
            ContainerType::ArmoryHand => Some(&mut self.armoury_hands),
            ContainerType::ArmoryLeg => Some(&mut self.armoury_legs),
            ContainerType::ArmoryFoot => Some(&mut self.armoury_feet),
            ContainerType::ArmoryEarring => Some(&mut self.armoury_earring),
            ContainerType::ArmoryNeck => Some(&mut self.armoury_necklace),
            ContainerType::ArmoryWrist => Some(&mut self.armoury_bracelet),
            ContainerType::ArmoryRing => Some(&mut self.armoury_rings),
            ContainerType::ArmorySoulCrystal => Some(&mut self.armoury_soul_crystal),
            ContainerType::ArmoryWeapon => Some(&mut self.armoury_main_hand),
            ContainerType::KeyItems => Some(&mut self.key_items),
            ContainerType::SaddleBag1 => Some(&mut self.saddlebag[0]),
            ContainerType::SaddleBag2 => Some(&mut self.saddlebag[1]),
            _ => self
                .retainer
                .as_mut()
                .and_then(|retainer| retainer.container_mut(*container_type)),
        }
    }

    pub fn get_container(&self, container_type: ContainerType) -> Option<&dyn Storage> {
        match container_type {
            ContainerType::Inventory0 => Some(&self.pages[0]),
            ContainerType::Inventory1 => Some(&self.pages[1]),
            ContainerType::Inventory2 => Some(&self.pages[2]),
            ContainerType::Inventory3 => Some(&self.pages[3]),
            ContainerType::Equipped => Some(&self.equipped),
            ContainerType::Currency => Some(&self.currency),
            ContainerType::Crystals => Some(&self.crystals),
            ContainerType::ArmoryOffWeapon => Some(&self.armoury_off_hand),
            ContainerType::ArmoryHead => Some(&self.armoury_head),
            ContainerType::ArmoryBody => Some(&self.armoury_body),
            ContainerType::ArmoryHand => Some(&self.armoury_hands),
            ContainerType::ArmoryLeg => Some(&self.armoury_legs),
            ContainerType::ArmoryFoot => Some(&self.armoury_feet),
            ContainerType::ArmoryEarring => Some(&self.armoury_earring),
            ContainerType::ArmoryNeck => Some(&self.armoury_necklace),
            ContainerType::ArmoryWrist => Some(&self.armoury_bracelet),
            ContainerType::ArmoryRing => Some(&self.armoury_rings),
            ContainerType::ArmorySoulCrystal => Some(&self.armoury_soul_crystal),
            ContainerType::ArmoryWeapon => Some(&self.armoury_main_hand),
            ContainerType::KeyItems => Some(&self.key_items),
            ContainerType::SaddleBag1 => Some(&self.saddlebag[0]),
            ContainerType::SaddleBag2 => Some(&self.saddlebag[1]),
            _ => self
                .retainer
                .as_ref()
                .and_then(|retainer| retainer.container(container_type)),
        }
    }

    /// Returns the model ID for the equipped main weapon.
    pub fn get_main_weapon_id(&self, game_data: &mut GameData) -> WeaponModelId {
        let mut model = WeaponModelId::from(
            game_data
                .get_primary_model_id(self.equipped.main_hand.apparent_id())
                .unwrap_or(0),
        );
        model.stains = self.equipped.main_hand.stains;

        model
    }

    /// The off-hand model as others see it for a player of `classjob`: none at all for a crafter or gatherer (classes
    /// 8-18), as retail's Equip sent in all 106 for those classes in Bob's Chronofoil captures, whatever was in the
    /// off-hand; the client shows a crafter's tools and station itself while it synthesizes. Sent as their tools'
    /// models, a Carpenter's saw's sub model is its workbench, which stood beside them all the time, and a second one
    /// came with each synthesis (Bob's bundle of 2026-10-26.)
    pub fn get_shown_sub_weapon_id(&self, game_data: &mut GameData, classjob: u8) -> WeaponModelId {
        if (CLASSJOB_FIRST_HAND_OR_LAND..=CLASSJOB_LAST_HAND_OR_LAND).contains(&classjob) {
            return WeaponModelId::default();
        }
        self.get_sub_weapon_id(game_data)
    }

    /// Returns the model ID for the equipped sub weapon.
    pub fn get_sub_weapon_id(&self, game_data: &mut GameData) -> WeaponModelId {
        // Use sub model from main hand (if available), e.g. quivers for bows. Otherwise fall back to off-hand, e.g. shields.
        if let Some(model) = game_data.get_sub_model_id(self.equipped.main_hand.apparent_id()) {
            let mut model = WeaponModelId::from(model);
            model.stains = self.equipped.main_hand.stains;

            model
        } else {
            let mut model = WeaponModelId::from(
                game_data
                    .get_primary_model_id(self.equipped.off_hand.apparent_id())
                    .unwrap_or(0),
            );
            model.stains = self.equipped.off_hand.stains;

            model
        }
    }

    /// Creates a LegacyEquipmentModelId from an Item, taking into account the first dye slot.
    fn legacy_model_id(game_data: &mut GameData, item: &Item) -> LegacyEquipmentModelId {
        let mut model = LegacyEquipmentModelId::from(
            game_data
                .get_primary_model_id(item.apparent_id())
                .unwrap_or(0) as u32,
        );
        model.stain = item.stains[0];

        model
    }

    pub fn legacy_model_ids(&self, game_data: &mut GameData) -> [LegacyEquipmentModelId; 10] {
        [
            Self::legacy_model_id(game_data, &self.equipped.head),
            Self::legacy_model_id(game_data, &self.equipped.body),
            Self::legacy_model_id(game_data, &self.equipped.hands),
            Self::legacy_model_id(game_data, &self.equipped.legs),
            Self::legacy_model_id(game_data, &self.equipped.feet),
            Self::legacy_model_id(game_data, &self.equipped.ears),
            Self::legacy_model_id(game_data, &self.equipped.neck),
            Self::legacy_model_id(game_data, &self.equipped.wrists),
            Self::legacy_model_id(game_data, &self.equipped.left_ring),
            Self::legacy_model_id(game_data, &self.equipped.right_ring),
        ]
    }

    pub fn second_model_stain_ids(&self) -> [u8; 10] {
        [
            self.equipped.head.stains[1],
            self.equipped.body.stains[1],
            self.equipped.hands.stains[1],
            self.equipped.legs.stains[1],
            self.equipped.feet.stains[1],
            self.equipped.ears.stains[1],
            self.equipped.neck.stains[1],
            self.equipped.wrists.stains[1],
            self.equipped.left_ring.stains[1],
            self.equipped.right_ring.stains[1],
        ]
    }

    fn prepare_items_in_container(container: &mut impl Storage, data: &mut GameData) {
        for index in 0..container.max_slots() {
            let item = container.get_slot_mut(index as u16);

            if item.is_empty_slot() {
                continue;
            }

            if let Some(info) = data.get_item_info(ItemInfoQuery::ById(item.item_id)) {
                *item = Item {
                    quantity: item.quantity,
                    item_id: item.item_id,
                    crafter_content_id: item.crafter_content_id,
                    item_flags: item.item_flags,
                    condition: item.condition,
                    spiritbond_or_collectability: item.spiritbond_or_collectability,
                    glamour_id: item.glamour_id,
                    materia: item.materia,
                    materia_grades: item.materia_grades,
                    stains: item.stains,
                    ..info.into()
                };
            }
        }
    }

    pub fn prepare_player_inventory(inventory: &mut Inventory, data: &mut GameData) {
        // TODO: implement iter_mut for Inventory so all of this can be reduced down
        for index in 0..inventory.pages.len() {
            Self::prepare_items_in_container(&mut inventory.pages[index], data);
        }

        Self::prepare_items_in_container(&mut inventory.equipped, data);
        Self::prepare_items_in_container(&mut inventory.armoury_main_hand, data);
        Self::prepare_items_in_container(&mut inventory.armoury_body, data);
        Self::prepare_items_in_container(&mut inventory.armoury_hands, data);
        Self::prepare_items_in_container(&mut inventory.armoury_legs, data);
        Self::prepare_items_in_container(&mut inventory.armoury_feet, data);
        Self::prepare_items_in_container(&mut inventory.armoury_off_hand, data);
        Self::prepare_items_in_container(&mut inventory.armoury_earring, data);
        Self::prepare_items_in_container(&mut inventory.armoury_necklace, data);
        Self::prepare_items_in_container(&mut inventory.armoury_bracelet, data);
        Self::prepare_items_in_container(&mut inventory.armoury_rings, data);
        Self::prepare_items_in_container(&mut inventory.key_items, data);
        // Skip soul crystals
    }

    /// Equips the given soul crystal and places the old one (if any) back into the Armoury Chest.
    pub fn equip_soul_crystal(&mut self, id: u32) {
        // NOTE: This has to match client behavior exactly! See ItemOperation code for more details.

        // If we already have it equipped, do nothing.
        if self.equipped.soul_crystal.item_id == id && self.equipped.soul_crystal.quantity > 0 {
            return;
        }

        for i in 0..self.armoury_soul_crystal.max_slots() as u16 {
            // Find the soul crystal in the Armoury Chest.
            let armoury_slot = self.armoury_soul_crystal.get_slot(i);
            if armoury_slot.item_id == id && armoury_slot.quantity > 0 {
                // Perform the swap.
                let operation = ItemOperation {
                    operation_type: ItemOperationKind::Exchange,
                    src_storage_id: ContainerType::ArmorySoulCrystal,
                    src_container_index: i,
                    dst_storage_id: ContainerType::Equipped,
                    dst_container_index: 13,
                    ..Default::default()
                };
                self.process_action(&operation);

                return;
            }
        }
    }

    /// Checks if the soul crystal exists in your inventory.
    pub fn has_soul_crystal(&mut self, id: u32) -> bool {
        // TODO: can you move the soul crystal somewhere else?

        for i in 0..self.armoury_soul_crystal.max_slots() as u16 {
            // Find the soul crystal in the Armoury Chest.
            let armoury_slot = self.armoury_soul_crystal.get_slot(i);
            if armoury_slot.item_id == id && armoury_slot.quantity > 0 {
                return true;
            }
        }

        false
    }

    /// Puts the equipment in `slot` back into the Armoury Chest.
    pub fn unequip_equipment(&mut self, slot: u16) {
        // NOTE: This has to match client behavior exactly! See ItemOperation code for more details.

        // If we already have nothing, do nothing.
        if self.equipped.get_slot(slot).quantity == 0 {
            return;
        }

        let destination_info = self.add_in_next_free_armory_slot(slot).unwrap();
        self.add_in_slot(
            *self.equipped.get_slot(slot),
            &destination_info.container,
            destination_info.slot,
        );

        *self.equipped.get_slot_mut(slot) = Item::default();
    }
}

/// Represents a single housing plot's collective inventory, both inside and out.
// TODO: This will need to adjustments in 7.5x
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HousingInventory {
    //pub plot_size: kawari::ipc::zone::PlotSize,
    pub interior: Vec<GenericStorage<MAX_LARGE_STORAGE>>,
    pub interior_storeroom: Vec<GenericStorage<MAX_LARGE_STORAGE>>,
    // TODO: Unclear if we need to emulate this
    pub interior_appearance: Vec<GenericStorage<MAX_LARGE_STORAGE>>,
    pub exterior: Vec<GenericStorage<MAX_LARGE_STORAGE>>,
    pub exterior_storeroom: Vec<GenericStorage<MAX_LARGE_STORAGE>>,
    // TODO: Unclear if we need to emulate this
    pub exterior_appearance: Vec<GenericStorage<MAX_LARGE_STORAGE>>,
}

impl HousingInventory {
    pub const INT_STORAGE_APT_FC: usize = 2;
    pub const INT_STORAGE_SMALL: usize = 3;
    pub const INT_STORAGE_MEDIUM: usize = 4;
    pub const INT_STORAGE_MANSION: usize = 8;
}

// By default, be an apartment/fc chamber.
impl Default for HousingInventory {
    fn default() -> Self {
        Self {
            // plot_size: kawari::ipc::zone::PlotSize::Small,
            interior: vec![
                GenericStorage::<MAX_LARGE_STORAGE>::new(
                    ContainerType::HousingInteriorPlacedItems1,
                ),
                GenericStorage::<MAX_LARGE_STORAGE>::new(
                    ContainerType::HousingInteriorPlacedItems2,
                ),
                GenericStorage::<MAX_LARGE_STORAGE>::new(
                    ContainerType::HousingInteriorPlacedItems3,
                ),
                GenericStorage::<MAX_LARGE_STORAGE>::new(
                    ContainerType::HousingInteriorPlacedItems4,
                ),
                GenericStorage::<MAX_LARGE_STORAGE>::new(
                    ContainerType::HousingInteriorPlacedItems5,
                ),
                GenericStorage::<MAX_LARGE_STORAGE>::new(
                    ContainerType::HousingInteriorPlacedItems6,
                ),
                GenericStorage::<MAX_LARGE_STORAGE>::new(
                    ContainerType::HousingInteriorPlacedItems7,
                ),
                GenericStorage::<MAX_LARGE_STORAGE>::new(
                    ContainerType::HousingInteriorPlacedItems8,
                ),
            ],
            interior_storeroom: vec![
                GenericStorage::<MAX_LARGE_STORAGE>::new(ContainerType::HousingInteriorStoreroom1),
                GenericStorage::<MAX_LARGE_STORAGE>::new(ContainerType::HousingInteriorStoreroom2),
                GenericStorage::<MAX_LARGE_STORAGE>::new(ContainerType::HousingInteriorStoreroom3),
                GenericStorage::<MAX_LARGE_STORAGE>::new(ContainerType::HousingInteriorStoreroom4),
                GenericStorage::<MAX_LARGE_STORAGE>::new(ContainerType::HousingInteriorStoreroom5),
                GenericStorage::<MAX_LARGE_STORAGE>::new(ContainerType::HousingInteriorStoreroom6),
                GenericStorage::<MAX_LARGE_STORAGE>::new(ContainerType::HousingInteriorStoreroom7),
                GenericStorage::<MAX_LARGE_STORAGE>::new(ContainerType::HousingInteriorStoreroom8),
            ],
            interior_appearance: vec![GenericStorage::<MAX_LARGE_STORAGE>::new(
                ContainerType::HousingExteriorAppearance,
            )],

            exterior: vec![GenericStorage::<MAX_LARGE_STORAGE>::new(
                ContainerType::HousingExteriorPlacedItems,
            )],
            exterior_storeroom: vec![GenericStorage::<MAX_LARGE_STORAGE>::new(
                ContainerType::HousingExteriorPlacedItems,
            )],
            exterior_appearance: vec![GenericStorage::<MAX_LARGE_STORAGE>::new(
                ContainerType::HousingExteriorAppearance,
            )],
        }
    }
}

impl HousingInventory {
    pub fn add_in_empty_slot(
        &mut self,
        item: Item,
        desired_pages: DesiredHousingInventoryPages,
    ) -> Option<ItemInfo> {
        let desired_pages = match desired_pages {
            DesiredHousingInventoryPages::Interior => &mut self.interior,
            DesiredHousingInventoryPages::InteriorStoreroom => &mut self.interior_storeroom,
            DesiredHousingInventoryPages::Exterior => {
                if !self.exterior.is_empty() {
                    &mut self.exterior
                } else {
                    return None;
                }
            }
            DesiredHousingInventoryPages::ExteriorStoreroom => {
                if !self.exterior_storeroom.is_empty() {
                    &mut self.exterior_storeroom
                } else {
                    return None;
                }
            }
            DesiredHousingInventoryPages::None => {
                return None;
            }
        };

        for page in desired_pages {
            for (slot_index, slot) in page.slots.iter_mut().enumerate() {
                if slot.quantity == 0 {
                    slot.clone_from(&item);
                    return Some(ItemInfo {
                        slot: slot_index as u16,
                        container: page.kind,
                        ..(*slot).into()
                    });
                }
            }
        }
        None
    }

    pub fn get_desired_pages_from_intendeduse(
        &self,
        intended_use: TerritoryIntendedUse,
        storeroom: bool,
    ) -> DesiredHousingInventoryPages {
        match intended_use {
            TerritoryIntendedUse::HousingOutdoor => {
                if storeroom {
                    DesiredHousingInventoryPages::ExteriorStoreroom
                } else {
                    DesiredHousingInventoryPages::Exterior
                }
            }
            TerritoryIntendedUse::HousingIndoor => {
                if storeroom {
                    DesiredHousingInventoryPages::InteriorStoreroom
                } else {
                    DesiredHousingInventoryPages::Interior
                }
            }
            _ => DesiredHousingInventoryPages::None,
        }
    }

    fn get_container_mut(&mut self, container_type: &ContainerType) -> Option<&mut dyn Storage> {
        match container_type {
            ContainerType::HousingInteriorPlacedItems1 => Some(&mut self.interior[0]),
            ContainerType::HousingInteriorPlacedItems2 => Some(&mut self.interior[1]),
            ContainerType::HousingInteriorPlacedItems3 => Some(&mut self.interior[2]),
            ContainerType::HousingInteriorPlacedItems4 => Some(&mut self.interior[3]),
            ContainerType::HousingInteriorPlacedItems5 => Some(&mut self.interior[4]),
            ContainerType::HousingInteriorPlacedItems6 => Some(&mut self.interior[5]),
            ContainerType::HousingInteriorPlacedItems7 => Some(&mut self.interior[6]),
            ContainerType::HousingInteriorPlacedItems8 => Some(&mut self.interior[7]),

            ContainerType::HousingInteriorStoreroom1 => Some(&mut self.interior_storeroom[0]),
            ContainerType::HousingInteriorStoreroom2 => Some(&mut self.interior_storeroom[1]),
            ContainerType::HousingInteriorStoreroom3 => Some(&mut self.interior_storeroom[2]),
            ContainerType::HousingInteriorStoreroom4 => Some(&mut self.interior_storeroom[3]),
            ContainerType::HousingInteriorStoreroom5 => Some(&mut self.interior_storeroom[4]),
            ContainerType::HousingInteriorStoreroom6 => Some(&mut self.interior_storeroom[5]),
            ContainerType::HousingInteriorStoreroom7 => Some(&mut self.interior_storeroom[6]),
            ContainerType::HousingInteriorStoreroom8 => Some(&mut self.interior_storeroom[7]),

            ContainerType::HousingInteriorAppearance => Some(&mut self.interior_appearance[0]),
            ContainerType::HousingExteriorAppearance => Some(&mut self.exterior_appearance[0]),

            ContainerType::HousingExteriorPlacedItems => Some(&mut self.exterior[0]),
            ContainerType::HousingExteriorStoreroom => Some(&mut self.exterior_storeroom[0]),
            _ => None,
        }
    }

    pub fn get_container(&self, container_type: ContainerType) -> Option<&dyn Storage> {
        match container_type {
            ContainerType::HousingInteriorPlacedItems1 => Some(&self.interior[0]),
            ContainerType::HousingInteriorPlacedItems2 => Some(&self.interior[1]),
            ContainerType::HousingInteriorPlacedItems3 => Some(&self.interior[2]),
            ContainerType::HousingInteriorPlacedItems4 => Some(&self.interior[3]),
            ContainerType::HousingInteriorPlacedItems5 => Some(&self.interior[4]),
            ContainerType::HousingInteriorPlacedItems6 => Some(&self.interior[5]),
            ContainerType::HousingInteriorPlacedItems7 => Some(&self.interior[6]),
            ContainerType::HousingInteriorPlacedItems8 => Some(&self.interior[7]),

            ContainerType::HousingInteriorStoreroom1 => Some(&self.interior_storeroom[0]),
            ContainerType::HousingInteriorStoreroom2 => Some(&self.interior_storeroom[1]),
            ContainerType::HousingInteriorStoreroom3 => Some(&self.interior_storeroom[2]),
            ContainerType::HousingInteriorStoreroom4 => Some(&self.interior_storeroom[3]),
            ContainerType::HousingInteriorStoreroom5 => Some(&self.interior_storeroom[4]),
            ContainerType::HousingInteriorStoreroom6 => Some(&self.interior_storeroom[5]),
            ContainerType::HousingInteriorStoreroom7 => Some(&self.interior_storeroom[6]),
            ContainerType::HousingInteriorStoreroom8 => Some(&self.interior_storeroom[7]),

            ContainerType::HousingInteriorAppearance => Some(&self.interior_appearance[0]),
            ContainerType::HousingExteriorAppearance => Some(&self.exterior_appearance[0]),

            ContainerType::HousingExteriorPlacedItems => Some(&self.exterior[0]),
            ContainerType::HousingExteriorStoreroom => Some(&self.exterior_storeroom[0]),
            _ => None,
        }
    }

    /// Helper functions to reduce boilerplate
    pub fn get_item_mut(
        &mut self,
        storage_id: ContainerType,
        storage_index: u16,
    ) -> Option<&mut Item> {
        if let Some(container) = self.get_container_mut(&storage_id) {
            return Some(container.get_slot_mut(storage_index));
        }

        None
    }

    pub fn get_item(&self, storage_id: ContainerType, storage_index: u16) -> Option<Item> {
        if let Some(container) = self.get_container(storage_id) {
            return Some(*container.get_slot(storage_index));
        }

        None
    }
}

/// Used to decide which set of housing inventory pages to send.
#[derive(Debug, Copy, Clone)]
pub enum DesiredHousingInventoryPages {
    None,
    Exterior,
    ExteriorStoreroom,
    Interior,
    InteriorStoreroom,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(item_id: u32) -> Item {
        Item {
            quantity: 1,
            item_id,
            ..Default::default()
        }
    }

    /// Equipping a body piece from the main inventory, like Bob did on 2026-10-13 (see kawari-diagnostics.)
    #[test]
    fn equip_from_inventory_moves_old_item_to_armoury() {
        let mut inventory = Inventory::default();
        inventory.add_in_slot(item(3010), &ContainerType::Inventory0, 2);
        inventory.add_in_slot(item(2990), &ContainerType::Equipped, 3);

        inventory.process_action(&ItemOperation {
            operation_type: ItemOperationKind::Exchange,
            src_storage_id: ContainerType::Inventory0,
            src_container_index: 2,
            dst_storage_id: ContainerType::Equipped,
            dst_container_index: 3,
            ..Default::default()
        });

        assert_eq!(inventory.equipped.get_slot(3).item_id, 3010);
        assert_eq!(inventory.pages[0].get_slot(2).quantity, 0);
        assert_eq!(inventory.armoury_body.get_slot(0).item_id, 2990);
        assert_eq!(inventory.armoury_body.get_slot(0).quantity, 1);
    }

    /// An NPC trade (Color Your World on retail, 2026-10-25): moving the item into HandIn leaves it in its bag; the
    /// event takes it at the reward.
    #[test]
    fn hand_in_is_taken_at_the_reward() {
        let mut inventory = Inventory::default();
        let mut dye = item(4745);
        dye.quantity = 3;
        inventory.add_in_slot(dye, &ContainerType::Inventory1, 5);

        inventory.process_action(&ItemOperation {
            operation_type: ItemOperationKind::Move,
            src_storage_id: ContainerType::Inventory1,
            src_container_index: 5,
            src_stack: 1,
            dst_storage_id: ContainerType::HandIn,
            dst_container_index: 0,
            ..Default::default()
        });
        assert_eq!(inventory.pages[1].get_slot(5).quantity, 3);

        assert_eq!(inventory.take_handed_in(), 1);
        assert_eq!(inventory.pages[1].get_slot(5).quantity, 2);
        // Taken once only.
        assert_eq!(inventory.take_handed_in(), 0);
        assert_eq!(inventory.pages[1].get_slot(5).quantity, 2);
    }

    /// Entrusting an item to a called retainer and taking it back: the retail moves (type 0x203 on 2026.09.15) between
    /// the player's bags and the retainer's pages.
    #[test]
    fn entrust_and_withdraw() {
        let mut inventory = Inventory::default();
        // Without a retainer called, their containers don't exist.
        assert!(
            inventory
                .get_container(ContainerType::RetainerPage1)
                .is_none()
        );

        inventory.retainer = Some(Box::default());
        inventory.add_in_slot(item(5057), &ContainerType::Inventory1, 4);
        inventory.process_action(&ItemOperation {
            operation_type: ItemOperationKind::Move,
            src_storage_id: ContainerType::Inventory1,
            src_container_index: 4,
            dst_storage_id: ContainerType::RetainerPage2,
            dst_container_index: 20,
            ..Default::default()
        });
        assert!(inventory.pages[1].get_slot(4).is_empty_slot());
        let retainer = inventory.retainer.as_ref().unwrap();
        assert_eq!(retainer.pages[1].get_slot(20).item_id, 5057);
        assert_eq!(retainer.item_count(), 1);

        inventory.process_action(&ItemOperation {
            operation_type: ItemOperationKind::Move,
            src_storage_id: ContainerType::RetainerPage2,
            src_container_index: 20,
            dst_storage_id: ContainerType::Inventory0,
            dst_container_index: 0,
            ..Default::default()
        });
        assert_eq!(inventory.pages[0].get_slot(0).item_id, 5057);
        assert_eq!(inventory.retainer.as_ref().unwrap().item_count(), 0);
    }

    /// Putting part of a stack up for sale (type 0x215 on 2026.09.15: one item from bag 2 slot 34 into market slot 0
    /// at 3,400 gil.)
    #[test]
    fn list_on_retainer_market() {
        let mut inventory = Inventory::default();
        inventory.retainer = Some(Box::default());
        inventory.add_in_slot(
            Item {
                quantity: 5,
                ..item(4850)
            },
            &ContainerType::Inventory2,
            34,
        );
        inventory.process_action(&ItemOperation {
            operation_type: ItemOperationKind::RetainerMarketList,
            src_storage_id: ContainerType::Inventory2,
            src_container_index: 34,
            src_stack: 1,
            dst_storage_id: ContainerType::RetainerMarket,
            dst_container_index: 0,
            dst_item_id: 3400,
            ..Default::default()
        });
        assert_eq!(inventory.pages[2].get_slot(34).quantity, 4);
        let listings = inventory.retainer.as_ref().unwrap().market_listings();
        assert_eq!(listings.len(), 1);
        assert_eq!(
            (
                listings[0].0,
                listings[0].1.item_id,
                listings[0].1.quantity,
                listings[0].2
            ),
            (0, 4850, 1, 3400)
        );
    }

    /// Entrusting from the retainer's menu (0x132 on 2026.10.06): the server picks the slot, merging into the same
    /// item's stack; withdrawing part of it back; gil both ways.
    #[test]
    fn entrust_from_the_menu() {
        let mut inventory = Inventory::default();
        inventory.retainer = Some(Box::default());
        let ore = |quantity| Item {
            quantity,
            stack_size: 999,
            ..item(5106)
        };
        inventory.add_in_slot(ore(10), &ContainerType::Inventory0, 30);
        let entrust = |slot, stack| ItemOperation {
            operation_type: ItemOperationKind::RetainerEntrust,
            src_storage_id: ContainerType::Inventory0,
            src_container_index: slot,
            src_stack: stack,
            ..Default::default()
        };
        assert_eq!(
            inventory.transfer_with_retainer(&entrust(30, 7)),
            Some([
                (ContainerType::Inventory0, 30),
                (ContainerType::RetainerPage1, 0)
            ])
        );
        assert_eq!(inventory.pages[0].get_slot(30).quantity, 3);
        // The rest goes onto the same stack.
        assert_eq!(
            inventory.transfer_with_retainer(&entrust(30, 3)),
            Some([
                (ContainerType::Inventory0, 30),
                (ContainerType::RetainerPage1, 0)
            ])
        );
        assert!(inventory.pages[0].get_slot(30).is_empty_slot());
        assert_eq!(
            inventory.retainer.as_ref().unwrap().pages[0].slots[0].quantity,
            10
        );

        // Withdraw one.
        let withdraw = ItemOperation {
            operation_type: ItemOperationKind::RetainerWithdraw,
            src_storage_id: ContainerType::RetainerPage1,
            src_container_index: 0,
            src_stack: 1,
            ..Default::default()
        };
        assert_eq!(
            inventory.transfer_with_retainer(&withdraw),
            Some([
                (ContainerType::RetainerPage1, 0),
                (ContainerType::Inventory0, 0)
            ])
        );
        assert_eq!(
            inventory.retainer.as_ref().unwrap().pages[0].slots[0].quantity,
            9
        );

        // Gil: entrust 20, then take it all back.
        inventory.currency.gil.quantity = 100;
        let gil = |amount| ItemOperation {
            operation_type: ItemOperationKind::RetainerGil,
            src_stack: amount,
            dst_stack: amount,
            ..Default::default()
        };
        assert!(inventory.transfer_gil_with_retainer(&gil(20)));
        assert_eq!(inventory.currency.gil.quantity, 80);
        assert_eq!(inventory.retainer.as_ref().unwrap().gil(), 20);
        assert!(!inventory.transfer_gil_with_retainer(&gil(500)));
        assert!(inventory.transfer_gil_with_retainer(&gil(0)));
        assert_eq!(inventory.currency.gil.quantity, 100);
        assert_eq!(inventory.retainer.as_ref().unwrap().gil(), 0);
    }

    /// Swapping with the Armoury Chest is still a plain swap.
    #[test]
    fn equip_from_armoury_swaps() {
        let mut inventory = Inventory::default();
        inventory.add_in_slot(item(3010), &ContainerType::ArmoryBody, 0);
        inventory.add_in_slot(item(2990), &ContainerType::Equipped, 3);

        inventory.process_action(&ItemOperation {
            operation_type: ItemOperationKind::Exchange,
            src_storage_id: ContainerType::ArmoryBody,
            src_container_index: 0,
            dst_storage_id: ContainerType::Equipped,
            dst_container_index: 3,
            ..Default::default()
        });

        assert_eq!(inventory.equipped.get_slot(3).item_id, 3010);
        assert_eq!(inventory.armoury_body.get_slot(0).item_id, 2990);
    }
}
