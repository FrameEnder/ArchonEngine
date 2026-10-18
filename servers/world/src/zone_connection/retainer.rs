//! Retainers at a summoning bell (docs/src/fork/retainers.md.) The order of the packets follows Bob's Chronofoil
//! captures of retail (2026-09-15).

use kawari::{
    common::{ContainerType, ObjectId},
    ipc::zone::{
        ItemMarketBoardInfo, ItemMarketBoardSummary, RETAINER_STATE_DISMISSED,
        RETAINER_STATE_SUMMONED, RetainerSaleHistory, RetainerState, SceneFlags, ServerZoneIpcData,
        ServerZoneIpcSegment,
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
/// Scene 2's first and third parameters on retail (unknown.)
const MENU_PARAM_1: u32 = 4;
const MENU_PARAM_3: u32 = 3;
/// Scene 1's first parameter on retail, before the retainer's ID (unknown.)
const LIST_AGAIN_PARAM_1: u32 = 2;

/// The retainer called at a summoning bell.
#[derive(Debug, Clone)]
pub struct CalledRetainer {
    pub id: u64,
    pub name: String,
    /// The NPC they appear as.
    pub actor_id: ObjectId,
}

impl ZoneConnection {
    fn content_id(&self) -> u64 {
        self.player_data.character.content_id as u64
    }

    /// The bell's list (client trigger 9003, command 2): all ten slots, empty ones with only their index, then
    /// RetainerInfoEnd. Everything shares one inventory sequence.
    pub async fn send_retainer_list(&mut self) {
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
        self.send_retainer_info_end(RETAINER_LIST_SLOTS as u8).await;
    }

    /// One retainer's entry (client trigger 9003, command 3, after one was sent away.)
    pub async fn send_one_retainer_info(&mut self, id: u64) {
        let Some(retainer) = self.database.lock().retainer(self.content_id(), id) else {
            return;
        };
        let ipc = ServerZoneIpcSegment::new(retainer.info(self.player_data.item_sequence));
        self.send_ipc_self(ipc).await;
        self.send_retainer_info_end(1).await;
    }

    async fn send_retainer_info_end(&mut self, entry_count: u8) {
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::RetainerInfoEnd {
            sequence: self.player_data.item_sequence,
            entry_count,
            max_retainers: MAX_RETAINERS,
            unk1: 1,
            unk2: 0,
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

        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::RetainerState(RetainerState {
            retainer_id: retainer.id,
            flags: 0,
            state: RETAINER_STATE_SUMMONED,
            name: retainer.name.clone(),
            ..Default::default()
        }));
        self.send_ipc_self(ipc).await;

        self.send_retainer_containers(&retainer.inventory).await;
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
        });

        if let Some(event) = events.last() {
            self.event_scene(
                &event.1,
                SCENE_RETAINER_MENU,
                SceneFlags::HIDE_HOTBAR,
                vec![MENU_PARAM_1, actor_id.0, MENU_PARAM_3, 0],
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
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::RetainerState(RetainerState {
            retainer_id: called.id,
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
