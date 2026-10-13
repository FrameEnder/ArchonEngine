//! Quests!

use crate::{
    ItemInfoQuery, ZoneConnection,
    common::adjust_quest_id,
    inventory::{Item, Storage},
    zone_connection::PersistentQuest,
};
use kawari::{
    constants::COMPLETED_QUEST_BITMASK_SIZE,
    ipc::zone::{
        ActiveQuest, QuestActiveList, QuestTracker, ServerZoneIpcData, ServerZoneIpcSegment,
        TrackedQuest,
    },
};

impl ZoneConnection {
    /// What the client is sent for one of our active quests.
    fn active_quest_entry(quest: &PersistentQuest) -> ActiveQuest {
        ActiveQuest {
            id: quest.id,
            sequence: quest.sequence,
            bitflags: quest.variables,
            ..Default::default()
        }
    }

    /// The journal slot of an active quest.
    fn quest_slot(&self, adjusted_id: u32) -> Option<usize> {
        self.player_data
            .quest
            .active
            .0
            .iter()
            .position(|quest| quest.id as u32 == adjusted_id)
    }

    /// Sends the current state of the quest in `slot` to the client.
    async fn send_quest_update(&mut self, slot: usize) {
        let Some(quest) = self.player_data.quest.active.0.get(slot) else {
            return;
        };

        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::UpdateQuest {
            index: slot as u8,
            quest: Self::active_quest_entry(quest),
        });
        self.send_ipc_self(ipc).await;
    }

    pub async fn send_active_quests(&mut self) {
        let mut quests = Vec::new();
        for quest in &self.player_data.quest.active.0 {
            quests.push(Self::active_quest_entry(quest));
        }

        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::QuestActiveList(QuestActiveList {
            quests,
        }));
        self.send_ipc_self(ipc).await;
    }

    pub async fn send_scenario_guide(&mut self) {
        // TODO: temporary
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ScenarioGuide {
            quest_id_1: 39,
            next_quest_id: 85,
            layout_id: 1985113,
        });
        self.send_ipc_self(ipc).await;
    }

    pub async fn send_quest_information(&mut self) {
        self.send_active_quests().await;

        // quest complete list
        {
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::QuestCompleteList {
                completed_quests: self.player_data.quest.completed.data.clone(),
                unlocked_map_markers: self.player_data.quest.unlocked_map_markers.data.clone(),
            });
            self.send_ipc_self(ipc).await;
        }

        // legacy quest complete list
        {
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::LegacyQuestList {
                bitmask: self.player_data.quest.completed_legacy.data.clone(),
            });
            self.send_ipc_self(ipc).await;
        }

        // levequest complete list
        {
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::LevequestCompleteList {
                completed_levequests: self.player_data.quest.completed_levequests.data.clone(),
                unk2: Vec::default(),
            });
            self.send_ipc_self(ipc).await;
        }

        self.send_quest_tracker().await;
    }

    pub async fn accept_quest(&mut self, id: u32) {
        let adjusted_id = adjust_quest_id(id);

        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::AcceptQuest {
            quest_id: adjusted_id,
        });
        self.send_ipc_self(ipc).await;

        // Add it to our own internal data model, then update its slot in the journal
        self.player_data.quest.active.0.push(PersistentQuest {
            id: adjusted_id as u16,
            sequence: 0xFF,
            variables: [0; 6],
        });
        let slot = self.player_data.quest.active.0.len() - 1;
        self.send_quest_update(slot).await;

        self.send_quest_tracker().await;
    }

    /// `optional_item` is the reward the player picked (an item id, as the client returns it), or 0.
    pub async fn finish_quest(&mut self, id: u32, optional_item: u32) {
        let adjusted_id = adjust_quest_id(id);

        // Remove it from our internal data model
        if let Some(index) = self
            .player_data
            .quest
            .active
            .0
            .iter()
            .position(|x| x.id == adjusted_id as u16)
        {
            self.player_data.quest.active.0.remove(index);
        }

        // Grant rewards
        let rewards;
        {
            let mut game_data = self.game_data.lock();
            rewards = game_data.get_quest_rewards(id);
        }

        // Add gil
        // TODO: send log message
        self.player_data.inventory.currency.get_slot_mut(0).quantity = self
            .player_data
            .inventory
            .currency
            .get_slot_mut(0)
            .quantity
            .saturating_add_unsigned(rewards.1);
        self.send_inventory().await;

        // Add exp
        self.add_exp(rewards.0 as i32).await;

        // Add the optional reward they picked, as long as it's really one of this quest's
        if optional_item != 0 {
            let new_item;
            {
                let mut game_data = self.game_data.lock();
                new_item = game_data
                    .get_quest_optional_reward(id, optional_item)
                    .and_then(|quantity| {
                        game_data
                            .get_item_info(ItemInfoQuery::ById(optional_item))
                            .map(|x| Item::new(&x, quantity as i32))
                    });
            }
            match new_item {
                Some(new_item) => {
                    if self
                        .player_data
                        .inventory
                        .add_in_next_free_slot(new_item)
                        .is_some()
                    {
                        self.send_inventory().await;
                    } else {
                        tracing::error!(
                            "Couldn't add the reward {optional_item} for quest {id}, the inventory is full!"
                        );
                    }
                }
                None => tracing::warn!(
                    "Quest {id} doesn't have {optional_item} as an optional reward, so it wasn't given!"
                ),
            }
        }

        // The quests after it moved up a slot, so send the whole journal again
        self.send_active_quests().await;

        self.player_data.quest.completed.set(adjusted_id);

        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::FinishQuest {
            quest_id: adjusted_id as u16,
            flag1: 1,
            flag2: 1,
        });
        self.send_ipc_self(ipc).await;

        self.send_quest_tracker().await;
    }

    pub async fn finish_all_quests(&mut self) {
        self.player_data.quest.completed.data = vec![0xFF; COMPLETED_QUEST_BITMASK_SIZE];
        self.send_quest_information().await;
    }

    pub async fn send_quest_tracker(&mut self) {
        // Right now we don't support tracking, so just send the first five quests.
        let mut tracked_quests = [TrackedQuest::default(); 5];
        for (i, _) in self.player_data.quest.active.0.iter().take(5).enumerate() {
            tracked_quests[i] = TrackedQuest {
                active: true,
                quest_index: i as u8,
            };
        }

        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::QuestTracker(QuestTracker {
            tracked_quests,
        }));
        self.send_ipc_self(ipc).await;
    }

    pub async fn cancel_quest(&mut self, id: u32) {
        let adjusted_id = adjust_quest_id(id);

        // Remove it from our internal data model
        if let Some(index) = self
            .player_data
            .quest
            .active
            .0
            .iter()
            .position(|x| x.id == adjusted_id as u16)
        {
            self.player_data.quest.active.0.remove(index);
        }

        // TODO: inform the player, im not sure what this looks like in retail

        self.send_active_quests().await;
        self.send_quest_tracker().await;
    }

    pub async fn set_quest_sequence(&mut self, id: u32, sequence: u8) {
        let adjusted_id = adjust_quest_id(id);
        let Some(slot) = self.quest_slot(adjusted_id) else {
            tracing::warn!(
                "Tried to set the sequence of quest {adjusted_id}, but it isn't active!"
            );
            return;
        };

        self.player_data.quest.active.0[slot].sequence = sequence;
        self.send_quest_update(slot).await;
        self.send_quest_tracker().await;
    }

    /// Sets one of an active quest's variables (0 = UI8A through 5 = UI8F.)
    pub async fn set_quest_variable(&mut self, id: u32, index: u8, value: u8) {
        let adjusted_id = adjust_quest_id(id);
        let Some(slot) = self.quest_slot(adjusted_id) else {
            tracing::warn!("Tried to set a variable of quest {adjusted_id}, but it isn't active!");
            return;
        };
        let Some(variable) = self.player_data.quest.active.0[slot]
            .variables
            .get_mut(index as usize)
        else {
            return;
        };

        *variable = value;
        self.send_quest_update(slot).await;
    }

    pub async fn incomplete_quest(&mut self, id: u32) {
        let adjusted_id = adjust_quest_id(id);
        self.player_data.quest.completed.clear(adjusted_id);
        self.send_quest_information().await;
    }

    pub async fn incomplete_all_quests(&mut self) {
        self.player_data.quest.completed.data = vec![0x0; COMPLETED_QUEST_BITMASK_SIZE];
        self.send_quest_information().await;
    }

    pub fn record_fate_completion(&mut self) {
        let mut game_data = self.game_data.lock();
        if let Some(row) = game_data.get_fate_progress_ui_row(self.player_data.volatile.zone_id) {
            self.player_data.quest.shared_fates.0[row as usize] += 1;
        }
    }
}
