//! Retainers: who they are, what they hold, and how they're shown at a summoning bell. See docs/src/fork/retainers.md.
//!
//! The packets and numbers are from Bob's Chronofoil captures of retail (2026-09-15, two retainers, eleven visits to
//! summoning bells.)

use std::io::Cursor;

use binrw::{BinRead, BinWrite};
use kawari::{
    common::{ContainerType, ObjectId, ObjectTypeId, Position},
    ipc::zone::{
        CommonSpawn, DisplayFlag, ObjectKind, PLAYER_RETAINER_ENTRIES, PlayerRetainerEntry,
        PlayerRetainerInfo, ServerZoneIpcData, SpawnNpc,
    },
};
use physis::savedata::chardat::CustomizeData;
use serde::{Deserialize, Serialize};

use crate::{
    GameData,
    inventory::{EquippedStorage, GenericStorage, Inventory, Item, Storage},
};

/// How many slots each of a retainer's seven inventory pages has (175 in all.)
pub const RETAINER_PAGE_SLOTS: usize = 25;
/// How many items a retainer can have on the market.
pub const RETAINER_MARKET_SLOTS: usize = 20;
/// How many slots the bell's list has; the client always gets all ten.
pub const RETAINER_LIST_SLOTS: usize = 10;
/// What RetainerInfoEnd says the character may hire. Retail sent 2 (the base two, without extra retainers bought), but
/// on Kawari anyone may have all ten.
pub const MAX_RETAINERS: u8 = 10;
/// The Item ID of gil, in a retainer's gil container.
const GIL_ITEM_ID: u32 = 1;
/// Crystals are items 2 (Fire Shard) to 19 (Water Cluster), one slot each in this order.
const FIRST_CRYSTAL_ITEM_ID: u32 = 2;
const CRYSTAL_SLOTS: usize = 18;

/// How far in front of the player a retainer appears. On retail they appear at a spot set for each bell, 1.6 to 2.8
/// yalms from where the player stood, roughly facing them (Bob's captures); Kawari doesn't read those spots yet.
const SUMMON_DISTANCE: f32 = 1.8;
/// The HP retail gave retainers (721 and 1,124 for two of them; only shown on their nameplate if at all.)
const RETAINER_HP: u32 = 1000;
/// Their MP on retail.
const RETAINER_MP: u16 = 400;

/// Everything a retainer keeps: seven pages of items, what they wear, their gil, crystals, and what they have on the
/// market with its asking prices.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RetainerInventory {
    pub pages: [GenericStorage<RETAINER_PAGE_SLOTS>; 7],
    pub equipped: EquippedStorage,
    pub gil: GenericStorage<1>,
    pub crystals: GenericStorage<CRYSTAL_SLOTS>,
    pub market: GenericStorage<RETAINER_MARKET_SLOTS>,
    /// The asking price (per item) of each market slot.
    pub market_prices: Vec<u32>,
}

impl Default for RetainerInventory {
    fn default() -> Self {
        let mut gil = GenericStorage::new(ContainerType::RetainerGil);
        gil.slots[0].item_id = GIL_ITEM_ID;
        let mut crystals = GenericStorage::new(ContainerType::RetainerCrystals);
        for (i, slot) in crystals.slots.iter_mut().enumerate() {
            slot.item_id = FIRST_CRYSTAL_ITEM_ID + i as u32;
        }
        Self {
            pages: [
                GenericStorage::new(ContainerType::RetainerPage1),
                GenericStorage::new(ContainerType::RetainerPage2),
                GenericStorage::new(ContainerType::RetainerPage3),
                GenericStorage::new(ContainerType::RetainerPage4),
                GenericStorage::new(ContainerType::RetainerPage5),
                GenericStorage::new(ContainerType::RetainerPage6),
                GenericStorage::new(ContainerType::RetainerPage7),
            ],
            equipped: EquippedStorage::default(),
            gil,
            crystals,
            market: GenericStorage::new(ContainerType::RetainerMarket),
            market_prices: vec![0; RETAINER_MARKET_SLOTS],
        }
    }
}

impl RetainerInventory {
    pub fn container(&self, container_type: ContainerType) -> Option<&dyn Storage> {
        Some(match container_type {
            ContainerType::RetainerPage1 => &self.pages[0],
            ContainerType::RetainerPage2 => &self.pages[1],
            ContainerType::RetainerPage3 => &self.pages[2],
            ContainerType::RetainerPage4 => &self.pages[3],
            ContainerType::RetainerPage5 => &self.pages[4],
            ContainerType::RetainerPage6 => &self.pages[5],
            ContainerType::RetainerPage7 => &self.pages[6],
            ContainerType::RetainerEquippedItems => &self.equipped,
            ContainerType::RetainerGil => &self.gil,
            ContainerType::RetainerCrystals => &self.crystals,
            ContainerType::RetainerMarket => &self.market,
            _ => return None,
        })
    }

    pub fn container_mut(&mut self, container_type: ContainerType) -> Option<&mut dyn Storage> {
        Some(match container_type {
            ContainerType::RetainerPage1 => &mut self.pages[0],
            ContainerType::RetainerPage2 => &mut self.pages[1],
            ContainerType::RetainerPage3 => &mut self.pages[2],
            ContainerType::RetainerPage4 => &mut self.pages[3],
            ContainerType::RetainerPage5 => &mut self.pages[4],
            ContainerType::RetainerPage6 => &mut self.pages[5],
            ContainerType::RetainerPage7 => &mut self.pages[6],
            ContainerType::RetainerEquippedItems => &mut self.equipped,
            ContainerType::RetainerGil => &mut self.gil,
            ContainerType::RetainerCrystals => &mut self.crystals,
            ContainerType::RetainerMarket => &mut self.market,
            _ => return None,
        })
    }

    /// The containers in the order retail sends them as a retainer is called: the pages, gil, crystals, market, then
    /// what they wear.
    pub const SUMMON_ORDER: [ContainerType; 11] = [
        ContainerType::RetainerPage1,
        ContainerType::RetainerPage2,
        ContainerType::RetainerPage3,
        ContainerType::RetainerPage4,
        ContainerType::RetainerPage5,
        ContainerType::RetainerPage6,
        ContainerType::RetainerPage7,
        ContainerType::RetainerGil,
        ContainerType::RetainerCrystals,
        ContainerType::RetainerMarket,
        ContainerType::RetainerEquippedItems,
    ];

    /// How many of their inventory slots are filled (RetainerInfo's item count.)
    pub fn item_count(&self) -> u8 {
        self.pages
            .iter()
            .flat_map(|page| page.slots.iter())
            .filter(|item| !item.is_empty_slot())
            .count() as u8
    }

    /// The market slots that hold something, with their asking prices.
    pub fn market_listings(&self) -> Vec<(u16, Item, u32)> {
        self.market
            .slots
            .iter()
            .enumerate()
            .filter(|(_, item)| !item.is_empty_slot())
            .map(|(slot, item)| {
                (
                    slot as u16,
                    *item,
                    self.market_prices.get(slot).copied().unwrap_or(0),
                )
            })
            .collect()
    }

    pub fn gil(&self) -> u32 {
        self.gil.slots[0].quantity.max(0) as u32
    }
}

/// A retainer's ID as the two numbers the bell's scenes use (high half first.)
pub fn split_id(id: u64) -> [u32; 2] {
    [(id >> 32) as u32, id as u32]
}

pub fn join_id(high: u32, low: u32) -> u64 {
    ((high as u64) << 32) | low as u64
}

/// Whether a container belongs to a retainer.
pub fn is_retainer_container(container_type: ContainerType) -> bool {
    matches!(
        container_type,
        ContainerType::RetainerPage1
            | ContainerType::RetainerPage2
            | ContainerType::RetainerPage3
            | ContainerType::RetainerPage4
            | ContainerType::RetainerPage5
            | ContainerType::RetainerPage6
            | ContainerType::RetainerPage7
            | ContainerType::RetainerEquippedItems
            | ContainerType::RetainerGil
            | ContainerType::RetainerCrystals
            | ContainerType::RetainerMarket
    )
}

/// One of a character's retainers.
#[derive(Debug, Clone)]
pub struct Retainer {
    pub id: u64,
    pub content_id: u64,
    /// Their slot in the bell's list (0-9.)
    pub display_order: u8,
    pub name: String,
    pub classjob: u8,
    pub level: u8,
    /// FFXIVClientStructs' RetainerTown: 1 Limsa Lominsa, 2 Gridania, 3 Ul'dah.
    pub town: u8,
    pub customize: CustomizeData,
    pub voice: u8,
    pub inventory: RetainerInventory,
    /// Unix time.
    pub hired_at: i64,
}

impl Retainer {
    /// A new retainer ID. Retail's look like 0x0078_0017_xxxx_xxxx; ours are random, below 2^63 so they fit the
    /// database's signed column.
    pub fn new_id() -> u64 {
        fastrand::u64(1 << 48..1 << 62)
    }

    /// Their entry in the bell's list.
    pub fn info(&self, sequence: u32) -> ServerZoneIpcData {
        let listings = self.inventory.market_listings();
        ServerZoneIpcData::RetainerInfo {
            sequence,
            unk2: 0,
            retainer_id: self.id,
            index: self.display_order,
            item_count: self.inventory.item_count(),
            gil: self.inventory.gil(),
            market_item_count: listings.len() as u8,
            town: self.town,
            classjob_id: self.classjob,
            level: self.level,
            // TODO: listings ending (retail: seven days after the retainer was last called.)
            market_expire: 0,
            venture_id: 0,
            venture_complete: 0,
            available: 1,
            unk11: 0,
            name: self.name.clone(),
        }
    }

    /// The retainer as an NPC standing in front of the player at `position`, facing `rotation` (the player's.)
    pub fn spawn(&self, game_data: &mut GameData, position: Position, rotation: f32) -> SpawnNpc {
        let mut position = position;
        position.0.x += SUMMON_DISTANCE * rotation.sin();
        position.0.z += SUMMON_DISTANCE * rotation.cos();
        let facing = if rotation > 0.0 {
            rotation - std::f32::consts::PI
        } else {
            rotation + std::f32::consts::PI
        };

        // The model helpers work on an inventory's equipped items.
        let wearing = Inventory {
            equipped: self.inventory.equipped,
            ..Default::default()
        };

        SpawnNpc {
            common: CommonSpawn {
                main_weapon_model: wearing.get_main_weapon_id(game_data),
                sec_weapon_model: wearing.get_sub_weapon_id(game_data),
                max_health_points: RETAINER_HP,
                health_points: RETAINER_HP,
                max_resource_points: RETAINER_MP,
                resource_points: RETAINER_MP,
                display_flags: DisplayFlag::UNK1,
                rotation: facing,
                object_kind: ObjectKind::Retainer,
                voice: self.voice,
                level: self.level,
                class_job: self.classjob,
                position,
                models: wearing.legacy_model_ids(game_data),
                second_model_stain_ids: wearing.second_model_stain_ids(),
                name: self.name.clone(),
                look: self.customize.clone(),
                target_id: ObjectTypeId {
                    object_id: ObjectId::default(),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

/// The character's retainers for the marketboard (PlayerRetainerInfo.)
pub fn player_retainer_info(retainers: &[Retainer]) -> PlayerRetainerInfo {
    let mut info = PlayerRetainerInfo::default();
    for (entry, retainer) in info
        .entries
        .iter_mut()
        .zip(retainers.iter().take(PLAYER_RETAINER_ENTRIES))
    {
        *entry = PlayerRetainerEntry {
            retainer_id: retainer.id,
            // The numbers retail sent; unknown.
            unk1: 0,
            unk2: 1,
            unk3: 0,
            unk4: 5,
            name: retainer.name.clone(),
        };
    }
    info
}

/// A CustomizeData as the hex the database keeps.
pub fn customize_to_hex(customize: &CustomizeData) -> String {
    let mut buffer = Cursor::new(Vec::new());
    customize.write_le(&mut buffer).unwrap();
    buffer
        .into_inner()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn customize_from_hex(hex: &str) -> Option<CustomizeData> {
    let bytes: Vec<u8> = (0..hex.len())
        .step_by(2)
        .filter_map(|i| hex.get(i..i + 2))
        .filter_map(|pair| u8::from_str_radix(pair, 16).ok())
        .collect();
    CustomizeData::read_le(&mut Cursor::new(bytes)).ok()
}

/// Whether `name` is one a retainer can have: one word of 3 to 20 letters (and apostrophes or hyphens), starting with a
/// capital, as the client's own check asks.
pub fn valid_name(name: &str) -> bool {
    let length = name.chars().count();
    (3..=20).contains(&length)
        && name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        && name
            .chars()
            .all(|c| c.is_ascii_alphabetic() || c == '\'' || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retainer_ids_in_scenes() {
        // A retail retainer ID and the two numbers the bell's return had (Bob's 2026-09-15 capture.)
        assert_eq!(split_id(0x0078_0017_4aa7_e097), [0x0078_0017, 0x4aa7_e097]);
        assert_eq!(join_id(0x0078_0017, 0x4aa7_e097), 0x0078_0017_4aa7_e097);
        // As the client returns it: the low half negative (the 2026-10-18 bundle.)
        assert_eq!(
            join_id(235_309_321, -1_966_305_711i64 as u32),
            0x0E06_8909_8ACC_8E51
        );
    }

    #[test]
    fn names() {
        assert!(valid_name("Abc"));
        assert!(valid_name("Mi'ra"));
        assert!(!valid_name("ab"));
        assert!(!valid_name("abc"));
        assert!(!valid_name("Two Words"));
        assert!(!valid_name("Abcdefghijklmnopqrstu"));
    }

    #[test]
    fn new_retainer_inventory() {
        let inventory = RetainerInventory::default();
        assert_eq!(inventory.item_count(), 0);
        assert_eq!(inventory.gil(), 0);
        assert_eq!(inventory.gil.slots[0].item_id, 1);
        assert_eq!(inventory.crystals.slots[17].item_id, 19);
        assert!(inventory.market_listings().is_empty());
        let json = serde_json::to_string(&inventory).unwrap();
        let back: RetainerInventory = serde_json::from_str(&json).unwrap();
        assert_eq!(back.pages[6].kind, ContainerType::RetainerPage7);
    }

    #[test]
    fn retainer_containers() {
        for container in RetainerInventory::SUMMON_ORDER {
            assert!(is_retainer_container(container));
            assert!(RetainerInventory::default().container(container).is_some());
        }
        assert!(!is_retainer_container(ContainerType::Inventory0));
    }
}
