//! Currencies the client counts outside the inventory: Achievement Certificates (and the total ever given, which says
//! how many are still owed). The client learns their counts from ActorControlSelf 378 (`PlayerCurrency`), sent for each
//! at login and whenever one changes; they're kept in `unlock.player_currencies`. Retail as recorded in Bob's Chronofoil
//! capture b23b91ef (2026-10-26); see docs/src/fork/character-windows.md.

use kawari::{common::INVENTORY_ACTION_ACK_SHOP, ipc::zone::ActorControlCategory};

use crate::{ItemInfoQuery, ZoneConnection};

/// Achievement Certificates, Jonathas' currency (the client script's CURRENCY_ACHIEVEMENT_AMOUNT.)
pub const ACHIEVEMENT_CERTIFICATE: u32 = 21172;
/// Every certificate ever given (CURRENCY_ACHIEVEMENT_TOTAL): spending them doesn't lower it.
pub const ACHIEVEMENT_CERTIFICATES_TOTAL: u32 = 21173;
/// What retail sends a 378 for at login, in its order.
pub const PLAYER_CURRENCIES: [u32; 2] = [ACHIEVEMENT_CERTIFICATE, ACHIEVEMENT_CERTIFICATES_TOTAL];
/// One certificate for every this many achievement points (the Console Games Wiki's Achievement Certificate page; not
/// checked against a retail character's points yet.)
pub const ACHIEVEMENT_POINTS_PER_CERTIFICATE: u32 = 50;
/// The log message retail sent after handing them out, with the certificate's item ID (LogMessage 750.)
const LOG_MESSAGE_CERTIFICATES_GIVEN: u32 = 750;
/// The count when the Item sheet has no stack size (retail sent 65000 for both, their StackSize.)
const DEFAULT_MAX_COUNT: u32 = 65000;

/// How many certificates `points` achievement points are worth beyond the `given` already handed out.
pub fn certificates_owed(points: u32, given: u32) -> u32 {
    (points / ACHIEVEMENT_POINTS_PER_CERTIFICATE).saturating_sub(given)
}

impl ZoneConnection {
    /// How many of `item_id` the player has.
    pub fn player_currency(&self, item_id: u32) -> u32 {
        self.player_data.unlock.player_currencies.count(item_id)
    }

    /// Tells the client how many of `item_id` the player has (ActorControlSelf 378.)
    async fn send_player_currency(&mut self, item_id: u32) {
        let max_count = self
            .game_data
            .lock()
            .get_item_info(ItemInfoQuery::ById(item_id))
            .map(|item| item.stack_size)
            .filter(|size| *size > 0)
            .unwrap_or(DEFAULT_MAX_COUNT);
        let count = self.player_currency(item_id);
        self.actor_control_self(ActorControlCategory::PlayerCurrency {
            unk1: 0,
            item_id,
            max_count,
            count,
            unk2: 1,
        })
        .await;
    }

    /// At login: every currency's count, as retail does.
    pub async fn send_player_currencies(&mut self) {
        for item_id in PLAYER_CURRENCIES {
            self.send_player_currency(item_id).await;
        }
    }

    /// Sets how many of `item_id` the player has and tells the client as retail does: an InventoryActionAck (sequence
    /// 0xFFFFFFFF, type 6), then the 378.
    pub async fn set_player_currency(&mut self, item_id: u32, count: u32) {
        self.player_data
            .unlock
            .player_currencies
            .set(item_id, count);
        self.send_inventory_ack(u32::MAX, INVENTORY_ACTION_ACK_SHOP, 0)
            .await;
        self.send_player_currency(item_id).await;
    }

    /// The player's achievement points: the Points of every achievement completed.
    pub fn achievement_points(&self) -> u32 {
        self.game_data
            .lock()
            .achievement_points(&self.player_data.unlock.achievements)
    }

    /// Hands out the certificates the player's achievement points are owed (Jonathas' scene 2): both counts go up, then
    /// log message 750. Returns how many.
    pub async fn give_achievement_certificates(&mut self) -> u32 {
        let given = self.player_currency(ACHIEVEMENT_CERTIFICATES_TOTAL);
        let owed = certificates_owed(self.achievement_points(), given);
        if owed == 0 {
            return 0;
        }
        let held = self.player_currency(ACHIEVEMENT_CERTIFICATE);
        self.set_player_currency(ACHIEVEMENT_CERTIFICATE, held.saturating_add(owed))
            .await;
        self.set_player_currency(ACHIEVEMENT_CERTIFICATES_TOTAL, given + owed)
            .await;
        self.send_event_log_message(
            LOG_MESSAGE_CERTIFICATES_GIVEN,
            vec![ACHIEVEMENT_CERTIFICATE],
        )
        .await;
        owed
    }

    /// A log message with no event (handler 0), as retail's EventLogMessage for the certificates.
    async fn send_event_log_message(&mut self, message_type: u32, params: Vec<u32>) {
        let Some(ipc) = kawari::ipc::zone::EventLogMessage {
            handler_id: kawari::common::HandlerId(0),
            message_type,
            params,
        }
        .package() else {
            return;
        };
        self.send_ipc_self(ipc).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owed() {
        assert_eq!(certificates_owed(49, 0), 0);
        assert_eq!(certificates_owed(50, 0), 1);
        assert_eq!(certificates_owed(175, 1), 2);
        assert_eq!(certificates_owed(100, 3), 0);
    }
}
