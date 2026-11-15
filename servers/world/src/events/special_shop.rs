use async_trait::async_trait;
use kawari::{
    common::{
        ContainerType, ERR_INVENTORY_ADD_FAILED, HandlerId, INVENTORY_ACTION_ACK_SHOP,
        LogMessageType, ObjectTypeId,
    },
    ipc::zone::{EventLogMessage, ItemInfo, SceneFlags},
};

use crate::{
    Event, EventHandler, ShopEventHandler, ZoneConnection,
    inventory::{CurrencyKind, Item},
    lua::LuaPlayer,
};

/// For special shop events.
#[derive(Debug)]
pub struct SpecialShopEventHandler;

impl Default for SpecialShopEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl SpecialShopEventHandler {
    /// Item has been bought.
    pub const SCENE_ITEM_BOUGHT: u16 = 0;
    /// Displays shop interface
    pub const SCENE_SHOW_SHOP: u16 = 10;
    /// Keybound Square's MGC trader (SpecialShop 1549's event ID.)
    const MGC_TRADER: u32 = 1_771_021;
    /// The event's LogMessage for what was bought (retail's MGC trader: 750 with the item.)
    const LOG_MESSAGE_OBTAINED: u32 = 750;
    /// Seems to be an event termination scene? When used standalone without the proper event sequence, it softlocks.
    pub const SCENE_SHOP_END: u16 = 255;

    pub fn new() -> Self {
        Self {}
    }
}

impl SpecialShopEventHandler {
    async fn process_shop_event_return(
        &self,
        connection: &mut ZoneConnection,
        event: &Event,
        results: &[i32],
    ) -> bool {
        // If you're closing the window
        if results.len() == 1 {
            return true;
        }

        let item_index = results[1];

        let result;
        let cost;

        {
            let mut game_data = connection.game_data.lock();
            result = game_data.get_specialshop_item(event.id.0, item_index as u16);
            cost = game_data.get_specialshop_cost(event.id.0, item_index as u16);
        }

        // The MGC trader takes MGC for everything (retail's 54 purchases each took it); should the sheet name the
        // cost some other way than the MGC's item, it's still MGC, and the log says what the sheet had.
        let cost = cost.map(|(cost_item, cost_count)| {
            if event.id.0 == Self::MGC_TRADER && !crate::zone_connection::is_player_currency(cost_item) {
                tracing::warn!(
                    "The MGC trader's item {item_index} costs {cost_count} of {cost_item} by the sheet: taken as MGC"
                );
                (crate::server::KEYBOUND_MGC, cost_count)
            } else {
                (cost_item, cost_count)
            }
        });

        // Paid with a currency the client counts outside the inventory (Jonathas' Achievement Certificates, Keybound
        // Brawler's MGC: until 2026-11-15 the MGC trader took gil, the item's price, Bob's 99,999 for an emote.)
        if let (Some(item_info), Some((cost_item, cost_count))) = (&result, cost)
            && crate::zone_connection::is_player_currency(cost_item)
        {
            let held = connection.player_currency(cost_item);
            if held < cost_count {
                connection
                    .send_notice(
                        "Not enough to buy this item. Nice try bypassing the client-side check!",
                    )
                    .await;
                return false;
            }
            let Some(add_result) = connection
                .player_data
                .inventory
                .add_in_next_free_slot(Item::new(item_info, 1))
            else {
                tracing::error!(ERR_INVENTORY_ADD_FAILED);
                connection.send_notice(ERR_INVENTORY_ADD_FAILED).await;
                return false;
            };
            connection
                .set_player_currency(cost_item, held - cost_count)
                .await;
            ShopEventHandler::send_gilshop_item_update(connection, add_result).await;
            if cost_item == crate::server::KEYBOUND_MGC {
                // As retail's MGC trader (SpecialShop 1549, 54 purchases in Connor's Chronofoil captures of
                // 2026-11-03 to 11-09): the MGC's new count, the ack, the item's slot, then the event's LogMessage
                // 750 with the item and the shop's scene 0 again (flags 0x2001, `[100, 0, 0, 0]`.)
                let ipc = EventLogMessage {
                    handler_id: HandlerId(event.id.0),
                    message_type: Self::LOG_MESSAGE_OBTAINED,
                    params: vec![item_info.id],
                }
                .package()
                .unwrap();
                connection.send_ipc_self(ipc).await;
                connection
                    .event_scene(
                        event,
                        Self::SCENE_ITEM_BOUGHT,
                        SceneFlags::from_bits(8193).unwrap(),
                        vec![100, 0, 0, 0],
                    )
                    .await;
                return false;
            }
            Self::send_gilshop_ack(
                connection,
                event.id.0,
                item_info.id,
                1,
                cost_count,
                LogMessageType::ItemBought,
            )
            .await;
            connection
                .event_scene(
                    event,
                    10,
                    SceneFlags::from_bits(8193).unwrap(),
                    vec![1, 100],
                )
                .await;
            return false;
        }

        // TODO: Use the ItemCost field to determine which currency is used

        if let Some(item_info) = result {
            if connection.player_data.inventory.currency.gil.quantity >= item_info.price_mid as i32
            {
                if let Some(add_result) = connection
                    .player_data
                    .inventory
                    .add_in_next_free_slot(Item::new(&item_info, 1))
                {
                    connection.player_data.inventory.currency.gil.quantity = connection
                        .player_data
                        .inventory
                        .currency
                        .gil
                        .quantity
                        .saturating_sub_unsigned(item_info.price_mid);
                    ShopEventHandler::send_gilshop_item_update(
                        connection,
                        ItemInfo {
                            container: ContainerType::Currency,
                            slot: CurrencyKind::Gil as u16,
                            ..connection.player_data.inventory.currency.gil.into()
                        },
                    )
                    .await;

                    connection
                        .send_inventory_ack(u32::MAX, INVENTORY_ACTION_ACK_SHOP, 0)
                        .await;

                    ShopEventHandler::send_gilshop_item_update(connection, add_result).await;
                    Self::send_gilshop_ack(
                        connection,
                        event.id.0,
                        item_info.id,
                        1,
                        item_info.price_mid,
                        LogMessageType::ItemBought,
                    )
                    .await;

                    // See GenericShopkeeper.lua for information about this scene, the flags, and the params.
                    connection
                        .event_scene(
                            event,
                            10,
                            SceneFlags::from_bits(8193).unwrap(),
                            vec![1, 100],
                        )
                        .await;
                } else {
                    tracing::error!(ERR_INVENTORY_ADD_FAILED);
                    connection.send_notice(ERR_INVENTORY_ADD_FAILED).await;
                }
            } else {
                connection
                    .send_notice(
                        "Insufficient gil to buy item. Nice try bypassing the client-side check!",
                    )
                    .await;
            }
        } else {
            connection
                .send_notice("Unable to find shop item, this is a bug in Kawari!")
                .await;
        }

        false
    }

    // TODO: When we add support for ItemObtainedLogMessage, rename this and update this
    async fn send_gilshop_ack(
        connection: &mut ZoneConnection,
        event_id: u32,
        item_id: u32,
        item_quantity: u32,
        price_per_item: u32,
        message_type: LogMessageType,
    ) {
        let ipc = EventLogMessage {
            handler_id: HandlerId(event_id),
            message_type: message_type as u32,
            params: vec![item_id, item_quantity, item_quantity * price_per_item],
        }
        .package()
        .unwrap();
        connection.send_ipc_self(ipc).await;
    }
}

#[async_trait]
impl EventHandler for SpecialShopEventHandler {
    async fn on_talk(&self, _event: &Event, _target_id: ObjectTypeId, player: &mut LuaPlayer) {
        player.play_scene(Self::SCENE_ITEM_BOUGHT, SceneFlags::HIDE_HOTBAR, Vec::new());
    }

    async fn on_return(
        &self,
        event: &Event,
        connection: &mut ZoneConnection,
        scene: u16,
        results: &[i32],
        player: &mut LuaPlayer,
    ) {
        if scene == Self::SCENE_ITEM_BOUGHT {
            if Self::process_shop_event_return(self, connection, event, results).await {
                player.finish_event();
            }
        } else if scene == Self::SCENE_SHOW_SHOP {
            player.play_scene(
                Self::SCENE_SHOP_END,
                SceneFlags::NO_DEFAULT_CAMERA | SceneFlags::HIDE_HOTBAR,
                Vec::new(),
            );
        } else {
            player.finish_event();
        }
    }
}
