mod chat_connection;

/// Optional packet capture for debugging.
pub mod capture;
pub use chat_connection::{ChatConnection, ChatConnectionChannels, ChatPlayerData};

mod zone_connection;

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
pub use zone_connection::{ObsfucationData, PlayerData, TeleportReason, ZoneConnection};

mod database;

/// The marketboard's items, prices, listings and history.
pub mod marketboard;

/// Retainers: who they are, what they hold, and how they're shown at a summoning bell.
pub mod retainer;
pub use database::{Content, Unlock, WorldDatabase};

pub mod lua;

mod event;
pub use event::{Event, EventHandler};

mod status_effects;
pub use status_effects::StatusEffects;

mod server;
pub use server::{Party, server_main_loop};

mod custom_ipc_connection;
pub use custom_ipc_connection::CustomIpcConnection;

mod common;
pub use common::{
    CLIENT_QUEUE_SIZE, ClientHandle, ClientId, FromServer, MessageInfo, ServerHandle, ToServer,
};

mod navmesh;
pub use navmesh::{Navmesh, NavmeshParams, NavmeshTile};

pub mod auracite;

/// Inventory and storage management.
pub mod inventory;

mod bitmask;
pub use bitmask::{Bitmask, QuestBitmask};

mod gamedata;
pub use gamedata::{
    ActionInfo, GameData, GatheringPointItem, ItemInfoQuery, ItemRow, Recipe, Roulette,
    SyncedItemStats, TerritoryNameKind,
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
define_sql_array!(GlassesIds, u16, 2);
define_sql_array!(SharedFates, u8, SHARED_FATES_SIZE);
define_sql_array!(AchievementCounters, AchievementCounter);

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
