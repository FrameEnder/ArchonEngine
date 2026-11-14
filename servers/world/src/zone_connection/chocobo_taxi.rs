//! Chocobo porters (ChocoboTaxiStand events, 0x12xxxx; `events/generic/Chocobokeep.lua`): the porter takes the player
//! along a set route to another stand. As retail did it in nine rides (Bob's Chronofoil captures cc62ad94 of 2026-10-28
//! and 1fac2416 and 2528a32d of 2026-10-30; ChocoboTaxi 25 in the South Shroud, 56 and 57 in Eastern Thanalan):
//! * The menu's answer is the ChocoboTaxi row. The fare goes (the gil's slot), the event finishes, BetweenAreas, and
//!   PrepareZoning to the same territory with warp type 10 (ChocoboTaxi), the character hidden (1) and flags 0x02.
//! * 1.5 s later the player is put at the route's start (ActorSetPos, warp type 10) and mounts the porter's chocobo
//!   (Mount 3, all zeroes after it; SetMode mounted to everyone), the online status UsingAChocoboPorter, and
//!   `ChocoboTaxiStart` with the row and its Location. As the client finishes zoning, UsingChocoboTaxi instead of
//!   BetweenAreas.
//! * The client rides the route itself (its position updates as it goes), sending `ChocoboTaxiCheckpoint` at each point
//!   and `ChocoboTaxiArrived` at the end (43 s for ChocoboTaxi 25, 59-60 s for 56 and 57.) Then the online status, the
//!   mode (normal) and the conditions go back.
//! Not sent here: ActorControlSelf 263 `[1]`/`[0]`, 236, 928 `[9]`, 504 `[32]`, 930, 253 and 260 that came with it
//! (what they do isn't known), and the EventFinish's 0x12 (retail's result byte for the ride; Kawari's 1.)

use kawari::{
    common::{CharacterMode, ContainerType, Position, WarpType},
    ipc::zone::{Condition, ItemInfo, OnlineStatus, ServerZoneIpcData, ServerZoneIpcSegment},
};

use crate::{
    ToServer, ZoneConnection,
    inventory::{CurrencyKind, CurrencyStorage},
};

/// The porter's chocobo (Mount 3, retail's Mount packet in all nine rides.)
pub const CHOCOBO_TAXI_MOUNT: u16 = 3;

impl ZoneConnection {
    /// Starts the ride along ChocoboTaxi `taxi` from `start` (the event's already finished): the fare, then the warp
    /// to the route's start. False if the route isn't in the game data or the player can't pay.
    pub async fn begin_chocobo_taxi(&mut self, taxi: u32, start: Position, rotation: f32) -> bool {
        let route = self.game_data.lock().chocobo_taxi(taxi);
        let Some((location, fare, minutes)) = route else {
            tracing::warn!("No ChocoboTaxi {taxi}: no ride");
            return false;
        };
        let slot = self
            .player_data
            .inventory
            .currency
            .get_item_for_id(CurrencyKind::Gil);
        if slot.quantity < fare as i32 {
            let gil = slot.quantity;
            tracing::info!("ChocoboTaxi {taxi} costs {fare} gil, the player has {gil}: no ride");
            return false;
        }
        tracing::info!(
            "ChocoboTaxi {taxi}: Location {location}, {fare} gil, {minutes} minutes, from {start:?}"
        );
        slot.quantity -= fare as i32;
        let slot = *slot;
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::UpdateInventorySlot(ItemInfo {
            sequence: self.player_data.item_sequence,
            container: ContainerType::Currency,
            slot: CurrencyStorage::get_slot_for_id(CurrencyKind::Gil),
            ..slot.into()
        }));
        self.send_ipc_self(ipc).await;

        self.chocobo_taxi = Some((taxi, location));
        self.conditions.set_condition(Condition::BetweenAreas);
        self.send_conditions().await;
        let zone_id = self.player_data.volatile.zone_id as u16;
        self.change_zone(
            zone_id,
            Some(start),
            Some(rotation),
            WarpType::ChocoboTaxi,
            0,
        )
        .await;
        true
    }

    /// Puts the player on `mount` as a quest's event does, without the summoning cast: Every Little Thing She Does Is
    /// Magitek's reaper (MOUNT0, 14) as Wedge's scene 9 ends, and again from Wedge after a dismount. Retail's (Bob's
    /// Chronofoil capture da424662): ActorControlSelf 928 `[9]` (12 when the player summons one), the Mount packet, then
    /// 263 `[1]` (`[1]` after a summoned one too); the mount goes in the spawn others see, the mode is Mounted, and the
    /// player gets off as from any mount. See `docs/src/fork/chocobo.md` (Riding).
    pub async fn mount_by_event(&mut self, mount: u16) {
        use kawari::ipc::zone::ActorControlCategory;
        self.actor_control_self(ActorControlCategory::PillionPassengerRelatedUnk { unk: 9 })
            .await;
        self.set_character_mode(CharacterMode::Mounted, 0, true)
            .await;
        self.player_data.volatile.current_mount = mount as i32;
        self.handle
            .send(ToServer::ChocoboTaxiMount(
                self.player_data.character.actor_id,
                mount,
            ))
            .await;
        self.actor_control_self(ActorControlCategory::SetPetEntityId { unk1: 1 })
            .await;
    }

    /// Takes the player off their mount for an event (`answer_dismount_yield`: the client's yield 232, as Every Little
    /// Thing She Does Is Magitek's scene 27 makes on the reaper). Retail's (Bob's Chronofoil capture da424662):
    /// ActorControl 2 (SetMode) `[1]`, then 930 (PlayDismountAnimation) with where they land, both from the player; the
    /// mount leaves the spawn others see.
    pub async fn dismount_by_event(&mut self) {
        use kawari::ipc::zone::ActorControlCategory;
        if self.player_data.volatile.current_mount == 0 {
            return;
        }
        self.set_character_mode(CharacterMode::Normal, 0, true)
            .await;
        self.player_data.volatile.current_mount = 0;
        self.handle
            .send(ToServer::ChocoboTaxiMount(
                self.player_data.character.actor_id,
                0,
            ))
            .await;
        let (unk1, unk2, unk3) = kawari::common::dismount_landing(
            &self.player_data.volatile.position,
            self.player_data.volatile.rotation as f32,
        );
        self.actor_control(
            self.player_data.character.actor_id,
            ActorControlCategory::PlayDismountAnimation { unk1, unk2, unk3 },
        )
        .await;
    }

    /// The client has finished the warp to the route's start: on the porter's chocobo, and the ride begins.
    pub async fn start_chocobo_taxi(&mut self) {
        let Some((taxi, location)) = self.chocobo_taxi else {
            return;
        };
        self.riding_chocobo_taxi = Some(taxi);
        self.chocobo_taxi = None;

        // Retail's order: SetMode mounted, then the Mount.
        self.set_character_mode(CharacterMode::Mounted, 0, true)
            .await;
        self.player_data.volatile.current_mount = CHOCOBO_TAXI_MOUNT as i32;
        self.handle
            .send(ToServer::ChocoboTaxiMount(
                self.player_data.character.actor_id,
                CHOCOBO_TAXI_MOUNT,
            ))
            .await;
        self.set_chocobo_taxi_status(true).await;

        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ChocoboTaxiStart { taxi, location });
        self.send_ipc_self(ipc).await;

        self.conditions.remove_condition(Condition::BetweenAreas);
        self.conditions.set_condition(Condition::UsingChocoboTaxi);
        self.send_conditions().await;
    }

    /// The client has ridden the route to its end (`ChocoboTaxiArrived`.)
    pub async fn finish_chocobo_taxi(&mut self, checkpoints: u32) {
        let Some(taxi) = self.riding_chocobo_taxi.take() else {
            tracing::warn!(
                "A chocobo porter's ride ended ({checkpoints} points), but none had begun"
            );
            return;
        };
        tracing::info!("ChocoboTaxi {taxi} arrived after {checkpoints} points");

        self.set_chocobo_taxi_status(false).await;
        self.handle
            .send(ToServer::ChocoboTaxiMount(
                self.player_data.character.actor_id,
                0,
            ))
            .await;
        self.player_data.volatile.current_mount = 0;
        self.set_character_mode(CharacterMode::Normal, 0, true)
            .await;

        self.conditions
            .remove_condition(Condition::UsingChocoboTaxi);
        self.send_conditions().await;
    }

    /// UsingAChocoboPorter on (with the player's usual statuses) or off again, for the player and everyone around.
    async fn set_chocobo_taxi_status(&mut self, riding: bool) {
        if !riding {
            self.update_online_status().await;
            return;
        }
        let mut mask = self.get_online_status_mask();
        mask.set_status(OnlineStatus::UsingAChocoboPorter);
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::SetOnlineStatus(mask));
        self.send_ipc_self(ipc).await;
        self.handle
            .send(ToServer::SetOnlineStatus(
                self.player_data.character.actor_id,
                OnlineStatus::UsingAChocoboPorter,
            ))
            .await;
    }
}
