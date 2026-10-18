//! Retainers at a summoning bell and the retainer vocate (docs/src/fork/retainers.md.) The order of the packets follows
//! Bob's Chronofoil captures of retail (2026-09-15, and 2026-10-18 on 2026.10.06 with hiring.)

use kawari::{
    common::{ContainerType, INVENTORY_ACTION_ACK_GENERAL, ItemOperationKind, ObjectId},
    ipc::zone::{
        ActorControlCategory, CurrencyInfo, ItemInfo, ItemMarketBoardInfo, ItemMarketBoardSummary,
        ItemOperation, RETAINER_STATE_DISMISSED, RETAINER_STATE_HIRED, RETAINER_STATE_SUMMONED,
        RetainerSaleHistory, RetainerState, SceneFlags, ServerZoneIpcData, ServerZoneIpcSegment,
    },
};

use crate::{
    Event, ToServer, ZoneConnection,
    event::EventHandler,
    retainer::{
        MAX_RETAINERS, RETAINER_LIST_SLOTS, Retainer, RetainerInventory, player_retainer_info,
        split_id, valid_name,
    },
};

/// The bell's scenes (CmnDefRetainerBell): 1 is the list again after sending a retainer away, 2 is a called
/// retainer's menu.
const SCENE_LIST_AGAIN: u16 = 1;
const SCENE_RETAINER_MENU: u16 = 2;
/// Scene 2's first parameter on retail (unknown.)
const MENU_PARAM_1: u32 = 4;
/// Scene 2's third parameter was 3 with two retainers hired (2026-09-15) and 4 with one (2026-10-18): taken as five
/// minus the number hired, a guess.
const MENU_PARAM_3_BASE: u32 = 5;
/// Scene 1's first parameter on retail, before the retainer's ID (unknown.)
const LIST_AGAIN_PARAM_1: u32 = 2;

/// The retainer called at a summoning bell.
#[derive(Debug, Clone)]
pub struct CalledRetainer {
    pub id: u64,
    pub name: String,
    /// The NPC they appear as.
    pub actor_id: ObjectId,
    /// Whether their market listings changed while called (retail's notice before RetainerState says so.)
    pub market_changed: bool,
}

impl ZoneConnection {
    fn content_id(&self) -> u64 {
        self.player_data.character.content_id as u64
    }

    /// The bell's list (client trigger 9003, command 2): all ten slots, empty ones with only their index, then
    /// RetainerInfoEnd. Everything shares one inventory sequence.
    pub async fn send_retainer_list(&mut self) {
        self.send_retainer_list_for(None).await;
    }

    /// The list, as the bell (`vocate_choice` None) or the retainer vocate sends it: before each of the vocate's menu
    /// choices, RetainerInfoEnd carries the choice (and 0 where the bell has 1.)
    pub async fn send_retainer_list_for(&mut self, vocate_choice: Option<u32>) {
        let retainers = self.database.lock().retainers(self.content_id());
        let sequence = self.player_data.item_sequence;
        for index in 0..RETAINER_LIST_SLOTS as u8 {
            let data = match retainers.iter().find(|r| r.display_order == index) {
                Some(retainer) => retainer.info(sequence),
                None => ServerZoneIpcData::RetainerInfo {
                    sequence,
                    unk2: 0,
                    retainer_id: 0,
                    index,
                    item_count: 0,
                    gil: 0,
                    market_item_count: 0,
                    town: 0,
                    classjob_id: 0,
                    level: 0,
                    market_expire: 0,
                    venture_id: 0,
                    venture_complete: 0,
                    available: 0,
                    unk11: 0,
                    name: String::new(),
                },
            };
            self.send_ipc_self(ServerZoneIpcSegment::new(data)).await;
        }
        self.send_retainer_info_end(RETAINER_LIST_SLOTS as u8, vocate_choice)
            .await;
    }

    /// One retainer's entry (client trigger 9003, command 3, after one was sent away.)
    pub async fn send_one_retainer_info(&mut self, id: u64) {
        let Some(retainer) = self.database.lock().retainer(self.content_id(), id) else {
            return;
        };
        let ipc = ServerZoneIpcSegment::new(retainer.info(self.player_data.item_sequence));
        self.send_ipc_self(ipc).await;
        self.send_retainer_info_end(1, None).await;
    }

    async fn send_retainer_info_end(&mut self, entry_count: u8, vocate_choice: Option<u32>) {
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::RetainerInfoEnd {
            sequence: self.player_data.item_sequence,
            entry_count,
            max_retainers: MAX_RETAINERS,
            unk1: u8::from(vocate_choice.is_none()),
            unk2: vocate_choice.unwrap_or(0),
            unk3: u32::MAX,
            unk4: u32::MAX,
            unk5: u16::MAX as u32,
        });
        self.send_ipc_self(ipc).await;
        self.player_data.item_sequence += 1;
    }

    /// Whether the character has any retainers.
    pub fn has_retainers(&mut self) -> bool {
        !self.database.lock().retainers(self.content_id()).is_empty()
    }

    /// Calls a retainer (the bell's list returned their ID): RetainerState, their containers and market prices, their
    /// NPC, then their menu (scene 2.)
    pub async fn summon_retainer(
        &mut self,
        id: u64,
        events: &mut [(Box<dyn EventHandler>, Event)],
    ) -> bool {
        // Only one at a time.
        self.release_called_retainer().await;

        let Some(retainer) = self.database.lock().retainer(self.content_id(), id) else {
            tracing::warn!("The client asked for retainer {id:#x}, who isn't this character's!");
            return false;
        };

        let hired = self.database.lock().retainers(self.content_id()).len() as u32;
        self.send_retainer_state_notice(false).await;
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::RetainerState(RetainerState {
            retainer_id: retainer.id,
            flags: market_flags(&retainer.inventory, false),
            state: RETAINER_STATE_SUMMONED,
            name: retainer.name.clone(),
            ..Default::default()
        }));
        self.send_ipc_self(ipc).await;

        self.send_retainer_containers(&retainer.inventory).await;
        self.send_retainer_state_notice(false).await;
        self.player_data.inventory.retainer = Some(Box::new(retainer.inventory.clone()));

        let actor_id = ObjectId(fastrand::u32(0x4000_0000..0x5000_0000));
        let spawn = {
            let mut game_data = self.game_data.lock();
            retainer.spawn(
                &mut game_data,
                self.player_data.volatile.position,
                self.player_data.volatile.rotation as f32,
            )
        };
        self.handle
            .send(ToServer::SpawnRetainer(
                self.player_data.character.actor_id,
                actor_id,
                Box::new(spawn),
            ))
            .await;
        self.called_retainer = Some(CalledRetainer {
            id: retainer.id,
            name: retainer.name.clone(),
            actor_id,
            market_changed: false,
        });

        if let Some(event) = events.last() {
            self.event_scene(
                &event.1,
                SCENE_RETAINER_MENU,
                SceneFlags::HIDE_HOTBAR,
                vec![
                    MENU_PARAM_1,
                    actor_id.0,
                    MENU_PARAM_3_BASE.saturating_sub(hired),
                    0,
                ],
            )
            .await;
        }
        true
    }

    /// A retainer's containers in retail's order, then the asking prices of what they have on the market and how many.
    async fn send_retainer_containers(&mut self, inventory: &RetainerInventory) {
        for container_type in RetainerInventory::SUMMON_ORDER {
            if let Some(container) = inventory.container(container_type) {
                self.send_container(container, container_type).await;
            }
        }

        let listings = inventory.market_listings();
        let sequence = self.player_data.item_sequence;
        for (slot, _, price) in &listings {
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ItemMarketBoardInfo(
                ItemMarketBoardInfo {
                    sequence,
                    container: ContainerType::RetainerMarket,
                    slot: *slot,
                    unk: 0,
                    price_per_unit: *price,
                },
            ));
            self.send_ipc_self(ipc).await;
        }
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ItemMarketBoardSummary(
            ItemMarketBoardSummary {
                sequence,
                count: listings.len() as u32,
            },
        ));
        self.send_ipc_self(ipc).await;
        self.player_data.item_sequence += 1;
    }

    /// Sends the called retainer away (their menu closed): RetainerState, then the list again (scene 1) with their ID.
    pub async fn dismiss_retainer(&mut self, events: &mut [(Box<dyn EventHandler>, Event)]) {
        let Some(called) = self.called_retainer.clone() else {
            return;
        };
        self.send_retainer_state_notice(called.market_changed).await;
        let flags = self
            .player_data
            .inventory
            .retainer
            .as_ref()
            .map(|inventory| market_flags(inventory, called.market_changed))
            .unwrap_or_default();
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::RetainerState(RetainerState {
            retainer_id: called.id,
            flags,
            state: RETAINER_STATE_DISMISSED,
            name: called.name.clone(),
            ..Default::default()
        }));
        self.send_ipc_self(ipc).await;
        self.release_called_retainer().await;

        if let Some(event) = events.last() {
            let [high, low] = split_id(called.id);
            self.event_scene(
                &event.1,
                SCENE_LIST_AGAIN,
                SceneFlags::HIDE_HOTBAR,
                vec![LIST_AGAIN_PARAM_1, high, low],
            )
            .await;
        }
    }

    /// Saves what the called retainer holds (after the player moved something to or from them.)
    pub fn save_called_retainer(&mut self) {
        let (Some(called), Some(inventory)) = (
            self.called_retainer.as_ref(),
            self.player_data.inventory.retainer.as_ref(),
        ) else {
            return;
        };
        self.database
            .lock()
            .save_retainer_inventory(called.id, inventory);
    }

    /// Saves the called retainer and takes their NPC away, without any scene (also as the player logs out.)
    pub async fn release_called_retainer(&mut self) {
        self.save_called_retainer();
        self.player_data.inventory.retainer = None;
        if let Some(called) = self.called_retainer.take() {
            self.handle
                .send(ToServer::DespawnRetainer(
                    self.player_data.character.actor_id,
                    called.actor_id,
                ))
                .await;
        }
    }

    /// Entrusting, withdrawing or gil from the retainer's menu (ItemOperation types BASE+22/23/24.) Retail's replies
    /// (Bob's 2026-10-18 capture): UpdateInventorySlot for the source and the destination, then the ack; for gil both
    /// gil totals (CurrencyCrystalInfo), then the ack.
    pub async fn retainer_item_operation(&mut self, action: &ItemOperation) {
        if action.operation_type == ItemOperationKind::RetainerGil {
            if self
                .player_data
                .inventory
                .transfer_gil_with_retainer(action)
            {
                let player_gil = self.player_data.inventory.currency.gil;
                let retainer_gil = self
                    .player_data
                    .inventory
                    .retainer
                    .as_ref()
                    .map(|retainer| retainer.gil.slots[0])
                    .unwrap_or_default();
                for (container, item) in [
                    (ContainerType::Currency, player_gil),
                    (ContainerType::RetainerGil, retainer_gil),
                ] {
                    let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::CurrencyCrystalInfo(
                        CurrencyInfo {
                            sequence: self.player_data.item_sequence,
                            container,
                            slot: 0,
                            quantity: item.quantity,
                            item_id: player_gil.item_id,
                        },
                    ));
                    self.send_ipc_self(ipc).await;
                }
            }
        } else if let Some(slots) = self.player_data.inventory.transfer_with_retainer(action) {
            if action.src_storage_id == ContainerType::RetainerMarket {
                self.mark_market_changed();
            }
            for (container, slot) in slots {
                self.send_inventory_slot(container, slot).await;
            }
        }
        self.send_inventory_ack(action.context_id, INVENTORY_ACTION_ACK_GENERAL, 0)
            .await;
        self.player_data.item_sequence += 1;
        self.save_called_retainer();
    }

    /// The called retainer's market listings changed (listed, re-priced or taken back.)
    pub fn mark_market_changed(&mut self) {
        if let Some(called) = self.called_retainer.as_mut() {
            called.market_changed = true;
        }
    }

    /// The 8-byte packet retail sends before every RetainerState and after a retainer's containers.
    async fn send_retainer_state_notice(&mut self, market_changed: bool) {
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::RetainerStateNotice {
            unk: 0,
            market_changed: u32::from(market_changed),
        });
        self.send_ipc_self(ipc).await;
    }

    /// How many retainers the character has (ActorControlSelf 152), as a retainer vocate visit ends.
    pub async fn send_retainer_count(&mut self) {
        let count = self.database.lock().retainers(self.content_id()).len() as u32;
        self.actor_control_self(ActorControlCategory::RetainerCount { count })
            .await;
    }

    /// Hires the retainer chosen at the vocate: the appearance from the editor (SetRetainerAppearance) and the name the
    /// client typed (an event text answer), in the character's city. Then, as retail: the notice, RetainerState
    /// (state 1), and the yield is answered with 0 (accepted.) Answers 1 if the name isn't usable or something's
    /// missing.
    pub async fn hire_retainer_at_vocate(&mut self, scene: u16, yield_id: u8) {
        let name = self.pending_event_text.take().unwrap_or_default();
        let appearance = self.pending_retainer_appearance.take();
        let hired = match appearance {
            Some((voice, ref customize)) if valid_name(&name) => {
                let town = self.player_data.city_state;
                self.database.lock().hire_retainer(
                    self.content_id(),
                    &name,
                    0,
                    1,
                    town,
                    &customize,
                    voice,
                )
            }
            _ => None,
        };
        let Some(retainer) = hired else {
            tracing::warn!(
                "Couldn't hire a retainer named {name:?} (appearance given: {})",
                appearance.is_some()
            );
            self.resume_event_here(scene, yield_id, vec![1]).await;
            return;
        };
        self.send_retainer_state_notice(false).await;
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::RetainerState(RetainerState {
            retainer_id: retainer.id,
            flags: market_flags(&retainer.inventory, false),
            state: RETAINER_STATE_HIRED,
            name: retainer.name.clone(),
            ..Default::default()
        }));
        self.send_ipc_self(ipc).await;
        self.resume_event_here(scene, yield_id, vec![0]).await;
    }

    /// Lets a retainer go at the vocate (yield 9 with their ID): refused with [0, 1] while they have items on the
    /// market, else released with [1, 0] (Bob's 2026-10-18 capture.)
    pub async fn release_retainer_at_vocate(&mut self, id: u64, scene: u16, yield_id: u8) {
        let retainer = self.database.lock().retainer(self.content_id(), id);
        let result = match retainer {
            Some(retainer) if retainer.inventory.market_listings().is_empty() => {
                self.database.lock().release_retainer(self.content_id(), id);
                vec![1, 0]
            }
            _ => vec![0, 1],
        };
        self.resume_event_here(scene, yield_id, result).await;
    }

    async fn resume_event_here(&mut self, scene: u16, yield_id: u8, params: Vec<u32>) {
        let Some(handler) = self.event_handler_id else {
            return;
        };
        self.resume_event(handler.0, scene, yield_id, params).await;
    }

    /// Sends one inventory slot as it is now (UpdateInventorySlot), e.g. when the client asks for it again (client
    /// trigger 419.)
    pub async fn send_inventory_slot(&mut self, container: ContainerType, slot: u16) {
        let Some(item) = self.player_data.inventory.get_item(container, slot) else {
            return;
        };
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::UpdateInventorySlot(ItemInfo {
            sequence: self.player_data.item_sequence,
            container,
            slot,
            ..item.into()
        }));
        self.send_ipc_self(ipc).await;
    }

    /// A new asking price for one of the called retainer's market listings (client trigger 400.)
    pub fn set_retainer_market_price(&mut self, slot: u32, price: u32) {
        let Some(retainer) = self.player_data.inventory.retainer.as_mut() else {
            return;
        };
        if let Some(listed) = retainer.market_prices.get_mut(slot as usize) {
            *listed = price;
            self.mark_market_changed();
            self.save_called_retainer();
        }
    }

    /// The character's retainers for the marketboard, as a retainer's sale list opens.
    pub async fn send_player_retainer_info(&mut self) {
        let retainers = self.database.lock().retainers(self.content_id());
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::PlayerRetainerInfo(
            player_retainer_info(&retainers),
        ));
        self.send_ipc_self(ipc).await;
    }

    /// A retainer's sale history: empty, Kawari doesn't keep sales yet.
    pub async fn send_retainer_sale_history(&mut self, retainer_id: u64) {
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::RetainerSaleHistory(
            RetainerSaleHistory { retainer_id },
        ));
        self.send_ipc_self(ipc).await;
    }

    /// Hires a retainer for the GM command `!retainer hire`: they look and sound like the character (until Kawari
    /// Web can change that) and belong to the character's city.
    pub async fn hire_retainer(&mut self, name: &str, classjob: u8, level: u8) {
        if !valid_name(name) {
            self.send_notice(
                "A retainer's name is one word of 3 to 20 letters, starting with a capital.",
            )
            .await;
            return;
        }
        let hired = {
            let customize = self.player_data.customize.chara_make.customize.clone();
            let voice = self.player_data.customize.chara_make.voice_id as u8;
            let town = self.player_data.city_state;
            self.database.lock().hire_retainer(
                self.content_id(),
                name,
                classjob,
                level.max(1),
                town,
                &customize,
                voice,
            )
        };
        match hired {
            Some(retainer) => {
                self.send_notice(&format!(
                    "Hired {} (slot {}). Call them at a summoning bell.",
                    retainer.name,
                    retainer.display_order + 1
                ))
                .await
            }
            None => self.send_notice("All ten retainer slots are taken.").await,
        }
    }

    /// Lists the character's retainers for `!retainer list`.
    pub async fn list_retainers(&mut self) {
        let retainers: Vec<Retainer> = self.database.lock().retainers(self.content_id());
        if retainers.is_empty() {
            self.send_notice("No retainers yet: !retainer hire <name> [class/job] [level]")
                .await;
        }
        for retainer in retainers {
            self.send_notice(&format!(
                "{}. {} (class/job {}, level {}): {} items, {} gil, {} on the market",
                retainer.display_order + 1,
                retainer.name,
                retainer.classjob,
                retainer.level,
                retainer.inventory.item_count(),
                retainer.inventory.gil(),
                retainer.inventory.market_listings().len()
            ))
            .await;
        }
    }

    /// Lets a retainer go for `!retainer release <name>`, with everything they hold.
    pub async fn release_retainer_by_name(&mut self, name: &str) {
        let found = self
            .database
            .lock()
            .retainers(self.content_id())
            .into_iter()
            .find(|r| r.name.eq_ignore_ascii_case(name));
        let Some(retainer) = found else {
            self.send_notice(&format!("No retainer called {name}."))
                .await;
            return;
        };
        if self
            .called_retainer
            .as_ref()
            .is_some_and(|called| called.id == retainer.id)
        {
            self.send_notice("Send them away at the bell first.").await;
            return;
        }
        self.database
            .lock()
            .release_retainer(self.content_id(), retainer.id);
        self.send_notice(&format!("{} has left your service.", retainer.name))
            .await;
    }
}

/// RetainerState's flags: the top byte 0x01 with items on the market, 0x81 after the market changed (retail.)
fn market_flags(inventory: &RetainerInventory, market_changed: bool) -> u32 {
    let byte = if market_changed {
        0x81
    } else if inventory.market_listings().is_empty() {
        0
    } else {
        0x01
    };
    byte << 24
}
