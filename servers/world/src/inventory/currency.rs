use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, FromRepr};

use super::{Item, Storage};

// TODO: look at TomestonesItem Excel sheet for inventory slots

// TODO: Add society currencies, this is just a good baseline
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Display, EnumIter, FromRepr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum CurrencyKind {
    Gil = 1,
    StormSeal = 20,
    SerpentSeal,
    FlameSeal,
    WolfMark = 25,
    AlliedSeal = 27,
    TomestonePoetics,
    MGP = 29,
    TomestoneHelio = 47,
    TomestoneMaths,
    CenturioSeal = 10307,
    Venture = 21072,
    SackOfNuts = 26533,
    TrophyCrystal = 36656,
}

impl mlua::IntoLua for CurrencyKind {
    fn into_lua(self, _: &mlua::Lua) -> mlua::Result<mlua::Value> {
        Ok(mlua::Value::Integer(self as i64))
    }
}

impl mlua::FromLua for CurrencyKind {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::Integer(integer) => Ok(Self::from_repr(integer as u32).unwrap()),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Deserialize, Serialize, Debug)]
pub struct CurrencyStorage {
    pub gil: Item,
    /// Each Grand Company's seals have a slot of their own: 1 the Storm Seals, 2 the Serpent Seals, 3 the Flame Seals
    /// (retail's quest rewards: Bob's Chronofoil captures b8432613, the Serpent Seals in slot 2, and c99fe374, the
    /// Flame Seals in slot 3.) Saved characters from before 2026-10-28 have only the Flame Seals' slot.
    #[serde(default = "storm_seals")]
    pub storm_seal: Item,
    #[serde(default = "serpent_seals")]
    pub serpent_seal: Item,
    pub flame_seal: Item,
    pub wolf_mark: Item,
    pub tomestone_poetics: Item,
    pub tomestone_mathematics: Item, // TODO: rename season agnostic
    pub allied_seal: Item,
    pub mgp: Item,
    pub tomestone_heliometry: Item, // TODO: rename season agnostic
    pub dummy: Item,
}

fn storm_seals() -> Item {
    Item {
        item_id: CurrencyKind::StormSeal as u32,
        ..Default::default()
    }
}

fn serpent_seals() -> Item {
    Item {
        item_id: CurrencyKind::SerpentSeal as u32,
        ..Default::default()
    }
}

impl CurrencyStorage {
    pub fn get_slot_for_id(id: CurrencyKind) -> u16 {
        match id {
            CurrencyKind::Gil => 0,
            CurrencyKind::StormSeal => 1,
            CurrencyKind::SerpentSeal => 2,
            CurrencyKind::FlameSeal => 3,
            CurrencyKind::WolfMark => 4,
            CurrencyKind::MGP => 9,
            _ => unimplemented!(),
        }
    }

    pub fn get_item_for_id(&mut self, id: CurrencyKind) -> &mut Item {
        self.get_slot_mut(Self::get_slot_for_id(id))
    }
}

impl Default for CurrencyStorage {
    fn default() -> Self {
        Self {
            gil: Item {
                item_id: CurrencyKind::Gil as u32,
                ..Default::default()
            },
            storm_seal: storm_seals(),
            serpent_seal: serpent_seals(),
            flame_seal: Item {
                item_id: CurrencyKind::FlameSeal as u32,
                ..Default::default()
            },
            wolf_mark: Item {
                item_id: CurrencyKind::WolfMark as u32,
                ..Default::default()
            },
            tomestone_poetics: Item {
                item_id: CurrencyKind::TomestonePoetics as u32,
                ..Default::default()
            },
            tomestone_mathematics: Item {
                item_id: CurrencyKind::TomestoneMaths as u32,
                ..Default::default()
            },
            allied_seal: Item {
                item_id: CurrencyKind::AlliedSeal as u32,
                ..Default::default()
            },
            mgp: Item {
                item_id: CurrencyKind::MGP as u32,
                ..Default::default()
            },
            tomestone_heliometry: Item {
                item_id: CurrencyKind::TomestoneHelio as u32,
                ..Default::default()
            },
            dummy: Item::default(),
        }
    }
}

impl Storage for CurrencyStorage {
    fn max_slots(&self) -> u32 {
        11
    }

    fn get_slot_mut(&mut self, index: u16) -> &mut Item {
        match index {
            0 => &mut self.gil,
            1 => &mut self.storm_seal,
            2 => &mut self.serpent_seal,
            3 => &mut self.flame_seal,
            4 => &mut self.wolf_mark,
            6 => &mut self.tomestone_poetics,
            7 => &mut self.tomestone_mathematics,
            8 => &mut self.allied_seal,
            9 => &mut self.mgp,
            10 => &mut self.tomestone_heliometry,
            _ => &mut self.dummy,
        }
    }

    fn get_slot(&self, index: u16) -> &Item {
        match index {
            0 => &self.gil,
            1 => &self.storm_seal,
            2 => &self.serpent_seal,
            3 => &self.flame_seal,
            4 => &self.wolf_mark,
            6 => &self.tomestone_poetics,
            7 => &self.tomestone_mathematics,
            8 => &self.allied_seal,
            9 => &self.mgp,
            10 => &self.tomestone_heliometry,
            _ => &self.dummy,
        }
    }
}
