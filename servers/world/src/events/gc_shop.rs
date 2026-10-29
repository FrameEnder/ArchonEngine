//! A Grand Company quartermaster's seal exchange (GCShop events, 0x16xxxx: 1441793 the Maelstrom's, 1441794 the Twin
//! Adders', 1441795 the Immortal Flames'), as retail played it (Bob's Chronofoil captures 9c4e397d, bab8c255 and
//! 0845b9a2 of 2026-10-29: seven purchases at the three quartermasters):
//!
//! 1. Scene 0 (flags 0x2001, no parameters), which the client ends with no results; then scene 10, the list (0x2001).
//! 2. Each purchase is scene 10 returning 7 results: 0, the item's index in its tab, the item, the seals it costs in
//!    all, the Grand Company, how many, and the tab (GCScripShopItem's row: a GCScripShopCategory, e.g. 1 the
//!    Maelstrom's Materiel, 10 the Twin Adders'.) Retail answered: EventLogMessage 1783 from the shop (the item, how
//!    many, the seals), the seals' slot (the company's currency slot), InventoryActionAck (0xFFFFFFFF, 6), the item's
//!    slot, then scene 10 again with one parameter, 0.
//! 3. Closing the list is scene 10 returning 1 first: the event ends.

use async_trait::async_trait;
use kawari::{
    common::{
        ContainerType, ERR_INVENTORY_ADD_FAILED, HandlerId, INVENTORY_ACTION_ACK_SHOP, ObjectTypeId,
    },
    ipc::zone::{EventLogMessage, SceneFlags},
};

use crate::{
    Event, EventHandler, ItemInfoQuery, ZoneConnection,
    inventory::{Item, Storage},
    lua::LuaPlayer,
};

/// For a quartermaster's seal exchange.
#[derive(Debug, Default)]
pub struct GrandCompanyShopEventHandler;

impl GrandCompanyShopEventHandler {
    /// The greeting, which the client ends with no results.
    pub const SCENE_GREETING: u16 = 0;
    /// The list of what the seals buy.
    pub const SCENE_LIST: u16 = 10;
    /// Retail's flags for both scenes.
    const FLAGS: u32 = 0x2001;
    /// The log line for a purchase (the item, how many, the seals.)
    const LOG_BOUGHT: u32 = 1783;

    pub fn new() -> Self {
        Self
    }

    fn flags() -> SceneFlags {
        SceneFlags::from_bits_retain(Self::FLAGS)
    }

    /// Buys what the client asked for; false if it doesn't check out (the list's own checks were bypassed, or the
    /// item doesn't fit.)
    async fn buy(connection: &mut ZoneConnection, event: &Event, results: &[i32]) -> bool {
        let [_, index, item_id, _, company, count, tab, ..] = results else {
            return false;
        };
        let count = (*count).max(1) as u32;
        let company_here = connection.game_data.lock().get_gc_shop_company(event.id.0);
        let active = connection.player_data.grand_company.active_company as u8;
        if company_here != Some(active) || *company as u8 != active {
            tracing::warn!(
                "Quartermaster {}: the player's company {active} isn't this shop's ({company_here:?})",
                event.id.0
            );
            return false;
        }
        let rank = connection
            .grand_company_rank(connection.player_data.grand_company.active_company)
            .unwrap_or_default();
        let listed = connection
            .game_data
            .lock()
            .get_gc_shop_item(*tab as u32, *index as u16);
        let Some((item, price, required_rank)) = listed else {
            tracing::warn!("Quartermaster {}: no item {tab}.{index}", event.id.0);
            return false;
        };
        let total = price * count;
        let seals = connection
            .player_data
            .inventory
            .currency
            .get_slot_mut(active as u16)
            .quantity
            .max(0) as u32;
        if item != *item_id as u32 || rank < required_rank || seals < total {
            tracing::warn!(
                "Quartermaster {}: can't buy {count} of {item} for {total} seals (asked for {item_id}; rank {rank}, needs {required_rank}; {seals} seals)",
                event.id.0
            );
            return false;
        }
        let new_item = connection
            .game_data
            .lock()
            .get_item_info(ItemInfoQuery::ById(item))
            .map(|info| Item::new(&info, count as i32));
        let Some(new_item) = new_item else {
            return false;
        };
        let Some((container, slot)) = connection
            .player_data
            .inventory
            .add_obtained_item_at(new_item)
        else {
            tracing::error!(ERR_INVENTORY_ADD_FAILED);
            connection.send_notice(ERR_INVENTORY_ADD_FAILED).await;
            return false;
        };

        if let Some(ipc) = (EventLogMessage {
            handler_id: HandlerId(event.id.0),
            message_type: Self::LOG_BOUGHT,
            params: vec![item, count, total],
        })
        .package()
        {
            connection.send_ipc_self(ipc).await;
        }
        connection
            .player_data
            .inventory
            .currency
            .get_slot_mut(active as u16)
            .quantity -= total as i32;
        connection
            .send_inventory_slot(ContainerType::Currency, active as u16)
            .await;
        connection
            .send_inventory_ack(u32::MAX, INVENTORY_ACTION_ACK_SHOP, 0)
            .await;
        connection.send_inventory_slot(container, slot).await;
        tracing::info!(
            "Quartermaster {}: bought {count} of {item} for {total} seals",
            event.id.0
        );
        true
    }
}

#[async_trait]
impl EventHandler for GrandCompanyShopEventHandler {
    async fn on_talk(&self, _event: &Event, _target_id: ObjectTypeId, player: &mut LuaPlayer) {
        player.play_scene(Self::SCENE_GREETING, Self::flags(), Vec::new());
    }

    async fn on_return(
        &self,
        event: &Event,
        connection: &mut ZoneConnection,
        scene: u16,
        results: &[i32],
        player: &mut LuaPlayer,
    ) {
        match scene {
            Self::SCENE_GREETING => player.play_scene(Self::SCENE_LIST, Self::flags(), Vec::new()),
            Self::SCENE_LIST if results.first() == Some(&0) && results.len() >= 7 => {
                Self::buy(connection, event, results).await;
                // Retail showed the list again after each purchase, with one parameter, 0.
                connection
                    .event_scene(event, Self::SCENE_LIST, Self::flags(), vec![0])
                    .await;
            }
            _ => player.finish_event(),
        }
    }
}
