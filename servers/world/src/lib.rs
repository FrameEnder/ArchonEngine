mod chat_connection;

/// Optional packet capture for debugging.
pub mod capture;
pub use chat_connection::{ChatConnection, ChatConnectionChannels, ChatPlayerData};

mod zone_connection;

/// Every opcode found again after a game update (docs/src/fork/opcodes.md).
pub mod auto_opcodes;
/// Help finding the fork's own opcodes again after a game update.
pub mod fork_opcodes;
use diesel::backend::Backend;
use diesel::deserialize::{self, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::serialize;
use diesel::sql_types::Text;
use diesel::sqlite::Sqlite;
use kawari::constants::{CLASSJOB_ARRAY_SIZE, SHARED_FATES_SIZE};
use serde::{Deserialize, Serialize};
pub use zone_connection::{
    GearUse, MaterializeRequest, ObsfucationData, PlayerData, RIDE_SHOOTING_OPENING_FLAGS,
    RIDE_SHOOTING_OPENING_VALUE, TeleportReason, ZoneConnection,
};

mod database;

/// The marketboard's items, prices, listings and history.
pub mod marketboard;

/// The Hunting Log's progress and how a kill counts towards it.
pub mod hunting_log;
pub use hunting_log::HUNTING_LOGS;

/// The Gold Saucer's lotteries and arcade machines: their rules and what the character keeps of them.
pub mod gold_saucer;

/// The GATEs: when they are and which one each is.
pub mod gate;

/// Retainers: who they are, what they hold, and how they're shown at a summoning bell.
pub mod retainer;
pub use database::{Content, Unlock, WorldDatabase};

pub mod lua;

mod event;
pub use event::{Event, EventHandler};

mod status_effects;
pub use status_effects::StatusEffects;

mod server;
pub use server::{Party, chosen_members, daily, duty_finder, server_main_loop};

mod custom_ipc_connection;
pub use custom_ipc_connection::CustomIpcConnection;

mod common;
pub use common::{
    BotPartyAction, CLIENT_QUEUE_SIZE, ClientHandle, ClientId, FromServer, MessageInfo,
    PartyUpdateTargets, ServerHandle, ToServer,
};

pub mod collision;
pub mod entrance_facing;
pub mod layer_filters;
mod navmesh;
pub use navmesh::{Navmesh, NavmeshParams, NavmeshTile};

pub mod auracite;

/// Inventory and storage management.
pub mod inventory;

mod bitmask;
pub use bitmask::{Bitmask, QuestBitmask};

/// Disciples of the Hand: the rules of a synthesis.
pub mod crafting;
pub mod glamour;
pub mod mender;
pub mod spiritbond;

/// Disciples of the Land: the rules at a node.
pub mod gathering;

mod gamedata;

/// Rules the client works out itself, ported from its code (`tools/clientlogic`).
pub mod client_rules;
pub use gamedata::{
    ActionInfo, GameData, GatheringPointItem, ItemInfoQuery, ItemRow, Recipe, RecipeInfo,
    RecipeIngredient, Roulette, SyncedItemStats, TerritoryNameKind,
};

mod chara_make;
pub use chara_make::CharaMake;

mod client_select_data;
pub use client_select_data::{ClientSelectData, RemakeMode};

mod events;
pub use events::*;

mod customize_data;
pub use customize_data::*;

use crate::zone_connection::PersistentQuest;

/// Define a new SQL-compatible array with an optional initial size.
macro_rules! define_sql_array {
    // With initial size.
    ($array_name:ident, $element_type:ident, $initial_size:expr) => {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AsExpression, FromSqlRow)]
        #[diesel(sql_type = Text)]
        pub struct $array_name(pub Vec<$element_type>);

        impl Default for $array_name {
            fn default() -> Self {
                Self(vec![0; $initial_size])
            }
        }

        impl serialize::ToSql<Text, Sqlite> for $array_name {
            fn to_sql<'b>(
                &'b self,
                out: &mut serialize::Output<'b, '_, Sqlite>,
            ) -> serialize::Result {
                out.set_value(serde_json::to_string(&self).unwrap());
                Ok(serialize::IsNull::No)
            }
        }

        impl deserialize::FromSql<Text, Sqlite> for $array_name {
            fn from_sql(mut bytes: <Sqlite as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
                Ok(serde_json::from_str(bytes.read_text())
                    .ok()
                    .unwrap_or_default())
            }
        }
    };
    // Without initial size.
    ($array_name:ident, $element_type:ident) => {
        #[derive(
            Debug, Clone, PartialEq, Default, Serialize, Deserialize, AsExpression, FromSqlRow,
        )]
        #[diesel(sql_type = Text)]
        pub struct $array_name(pub Vec<$element_type>);

        impl serialize::ToSql<Text, Sqlite> for $array_name {
            fn to_sql<'b>(
                &'b self,
                out: &mut serialize::Output<'b, '_, Sqlite>,
            ) -> serialize::Result {
                out.set_value(serde_json::to_string(&self).unwrap());
                Ok(serialize::IsNull::No)
            }
        }

        impl deserialize::FromSql<Text, Sqlite> for $array_name {
            fn from_sql(mut bytes: <Sqlite as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
                Ok(serde_json::from_str(bytes.read_text())
                    .ok()
                    .unwrap_or_default())
            }
        }
    };
}

// Arrays we store in SQL.
define_sql_array!(ClassLevels, u16, CLASSJOB_ARRAY_SIZE);
define_sql_array!(ClassExperience, i32, CLASSJOB_ARRAY_SIZE);
define_sql_array!(ActiveQuests, PersistentQuest);
define_sql_array!(FavoriteAetherytes, u16);
define_sql_array!(PartyMembers, i64, 8);
define_sql_array!(GrandCompanyRanks, u8, 3);
define_sql_array!(BuddyLevels, u8, 3);
define_sql_array!(BuddyBarding, u8, 3);
define_sql_array!(GlassesIds, u16, 2);
define_sql_array!(SharedFates, u8, SHARED_FATES_SIZE);
define_sql_array!(AchievementCounters, AchievementCounter);
define_sql_array!(PlayerCurrencies, PlayerCurrencyCount);
define_sql_array!(HuntingLogs, HuntingLogProgress);
use hunting_log::HuntingLogProgress;

/// How many of a currency the client counts outside the inventory (ActorControlSelf 378) one has.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct PlayerCurrencyCount {
    pub item_id: u32,
    pub count: u32,
}

impl PlayerCurrencies {
    /// How many of `item_id` one has (0 if none.)
    pub fn count(&self, item_id: u32) -> u32 {
        self.0
            .iter()
            .find(|entry| entry.item_id == item_id)
            .map(|entry| entry.count)
            .unwrap_or_default()
    }

    /// Sets how many of `item_id` one has.
    pub fn set(&mut self, item_id: u32, count: u32) {
        match self.0.iter_mut().find(|entry| entry.item_id == item_id) {
            Some(entry) => entry.count = count,
            None => self.0.push(PlayerCurrencyCount { item_id, count }),
        }
    }
}

/// One of the counters "do this N times" achievements count (Achievement Type 1: its Key), and how far it's got.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct AchievementCounter {
    pub key: u32,
    pub value: u32,
}

/// Helper for tests to initialize GameData, instead of each test creating its own.
/// Please call `tracing_subscriber::fmt::try_init()` so this is printed in the CI log!
#[cfg(test)]
pub fn test_game_data() -> Option<&'static mut GameData> {
    // Be able to skip these on the CI as we don't have game data there (yet)
    if std::env::var("CI") == Ok("1".to_string()) {
        tracing::warn!("Skipping test because it requires gamedata, and we're on the CI!");
        return None;
    }

    static mut DEEP_THOUGHT: std::sync::LazyLock<GameData> =
        std::sync::LazyLock::new(|| GameData::new());
    unsafe { Some(&mut *DEEP_THOUGHT) }
}
