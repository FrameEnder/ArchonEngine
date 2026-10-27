//! Quests!

use icarus::Quest::QuestSheet;
use mlua::Function;

use crate::{
    Event, EventHandler, ItemInfoQuery, LuaEventHandler, ToServer, ZoneConnection,
    common::adjust_quest_id,
    event::quest_script_path,
    inventory::{Item, Storage},
    lua::{KawariLua, LuaPlayer},
    zone_connection::PersistentQuest,
};
use kawari::{
    common::{HandlerId, HandlerType, ObjectId},
    config::get_config,
    constants::COMPLETED_QUEST_BITMASK_SIZE,
    ipc::zone::{
        ActiveQuest, ActorControlCategory, EventLogMessage, QuestActiveList, QuestTracker,
        ServerZoneIpcData, ServerZoneIpcSegment, TrackedQuest,
    },
};

/// The unlock link for retainers, and the quest that gives it: The Scions of the Seventh Dawn (its Quest row's
/// UnlockLink, `docs/src/fork/quests.md`.)
pub const RETAINER_UNLOCK_LINK: u32 = 3;
pub const RETAINER_UNLOCK_QUEST: u32 = 509;

impl ZoneConnection {
    /// What the client is sent for one of our active quests.
    fn active_quest_entry(quest: &PersistentQuest) -> ActiveQuest {
        ActiveQuest {
            id: quest.id,
            sequence: quest.sequence,
            accept_class_job: quest.accept_class_job,
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

    /// The quest a won quest battle moves on: the QuestBattle sheet's (`sheet_quest`, a Quest row ID) when it's one
    /// being done, otherwise the active quest whose QUESTBATTLE parameters name the battle. Trial by Wind's row
    /// (54) doesn't name quest 91, so its win moved nothing and the battle was offered again and again (Bob's bundle
    /// of 2026-10-27.)
    pub fn quest_for_quest_battle(&self, quest_battle_id: u32, sheet_quest: u32) -> u32 {
        if self.quest_slot(adjust_quest_id(sheet_quest)).is_some() {
            return sheet_quest;
        }
        let active: Vec<u32> = self
            .player_data
            .quest
            .active
            .0
            .iter()
            .map(|quest| quest.id as u32 + QuestSheet::STARTING_ROW)
            .collect();
        let mut game_data = self.game_data.lock();
        active
            .into_iter()
            .find(|quest| {
                (0..4).any(|n| {
                    game_data.quest_param(*quest, &format!("QUESTBATTLE{n}")) == Some(quest_battle_id)
                })
            })
            .unwrap_or_else(|| {
                tracing::warn!(
                    "Quest battle {quest_battle_id}: its sheet's quest {sheet_quest} isn't being done, and no active quest names the battle"
                );
                sheet_quest
            })
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

    /// Tells the client which main scenario quest to show in the Scenario Guide, see `GameData::get_scenario_guide`.
    pub async fn send_scenario_guide(&mut self) {
        let active: Vec<u16> = self
            .player_data
            .quest
            .active
            .0
            .iter()
            .map(|quest| quest.id)
            .collect();
        let completed = &self.player_data.quest.completed;
        let guide = self.game_data.lock().get_scenario_guide(
            |id| completed.contains(id),
            &active,
            self.player_data.classjob.current_class as u8,
            self.player_data.classjob.first_class as u8,
            self.player_data.grand_company.active_company as u8,
        );
        let Some((last_completed, next_quest_id, layout_id)) = guide else {
            tracing::info!("Scenario Guide: no main scenario quest to show");
            return;
        };
        tracing::info!(
            "Scenario Guide: next quest {next_quest_id} (issuer layout {layout_id}), last completed {last_completed}"
        );

        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ScenarioGuide {
            quest_id_1: last_completed,
            next_quest_id,
            layout_id,
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
            accept_class_job: self.player_data.classjob.current_class as u8,
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
            let level = self.current_level(&game_data);
            rewards = game_data.get_quest_rewards(id, level);
        }

        // The log lines for the rewards come first, as retail (Bob's Chronofoil captures): the gil (LogMessage 1605),
        // then each item given (1606, or 1607 with a count.)
        if rewards.1 > 0 {
            self.send_reward_log(QUEST_REWARD_GIL_LOG, vec![rewards.1])
                .await;
        }

        // Add gil
        self.player_data.inventory.currency.get_slot_mut(0).quantity = self
            .player_data
            .inventory
            .currency
            .get_slot_mut(0)
            .quantity
            .saturating_add_unsigned(rewards.1);
        self.send_inventory().await;

        // Add the items it always gives
        let reward_items: Vec<Item>;
        {
            let mut game_data = self.game_data.lock();
            reward_items = game_data
                .get_quest_reward_items(id)
                .into_iter()
                .filter_map(|(item_id, count)| {
                    game_data
                        .get_item_info(ItemInfoQuery::ById(item_id))
                        .map(|info| Item::new(&info, count as i32))
                })
                .collect();
        }
        if !reward_items.is_empty() {
            for item in reward_items {
                let item_id = item.item_id;
                let count = item.quantity.max(0) as u32;
                if self.player_data.inventory.add_obtained_item(item) {
                    self.send_item_reward_log(item_id, count).await;
                } else {
                    tracing::error!(
                        "Couldn't add the reward {item_id} for quest {id}, the inventory is full!"
                    );
                }
            }
            self.send_inventory().await;
        }

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
                    let count = new_item.quantity.max(0) as u32;
                    if self
                        .player_data
                        .inventory
                        .add_in_next_free_slot(new_item)
                        .is_some()
                    {
                        self.send_item_reward_log(optional_item, count).await;
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

        // Then the EXP (retail: after the gil and items.)
        if rewards.0 > 0 {
            self.add_exp(rewards.0 as i32).await;
        }

        // The quests after it moved up a slot, so send the whole journal again
        self.send_active_quests().await;

        self.player_data.quest.completed.set(adjusted_id);
        self.grant_quest_unlocks(id, true).await;

        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::FinishQuest {
            quest_id: adjusted_id as u16,
            flag1: 1,
            flag2: 1,
        });
        self.send_ipc_self(ipc).await;

        self.send_quest_tracker().await;

        self.check_achievements(true).await;
    }

    /// Marks quest `id` completed with no rewards: its FinishQuest, as retail sends one for a quest another replaces
    /// right after that one's (Lurkers in the Grotto's, then 464's; Bob's Chronofoil capture of 2026-10-21.)
    pub async fn mark_quest_completed(&mut self, id: u32) {
        let adjusted_id = adjust_quest_id(id);
        if let Some(index) = self.quest_slot(adjusted_id) {
            self.player_data.quest.active.0.remove(index);
            self.send_active_quests().await;
        }
        self.player_data.quest.completed.set(adjusted_id);
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::FinishQuest {
            quest_id: adjusted_id as u16,
            flag1: 1,
            flag2: 1,
        });
        self.send_ipc_self(ipc).await;
    }

    /// Unlocks what finishing quest `id` (its full row ID) grants (`GameData::get_quest_unlock_links`). `notify`
    /// tells the client straight away; otherwise it's in PlayerSetup, at login.
    pub async fn grant_quest_unlocks(&mut self, id: u32, notify: bool) {
        let links = self.game_data.lock().get_quest_unlock_links(id);
        for link in links {
            if self.player_data.unlock.unlocks.contains(link) {
                continue;
            }
            tracing::info!("Quest {id} unlocks unlock link {link}");
            self.player_data.unlock.unlocks.set(link);
            if notify {
                self.actor_control_self(ActorControlCategory::ToggleUnlock {
                    id: link,
                    unlocked: true,
                })
                .await;
            }
        }
    }

    /// Characters who finished quests before quest unlocks were handled get them at login.
    pub async fn check_quest_unlocks(&mut self) {
        let completed: Vec<u32> = (0..COMPLETED_QUEST_BITMASK_SIZE as u32 * 8)
            .filter(|quest| self.player_data.quest.completed.contains(*quest))
            .collect();
        for quest in completed {
            self.grant_quest_unlocks(quest + QuestSheet::STARTING_ROW, false)
                .await;
        }
        // And the retainers' approval only with the quest that gives it: characters given it otherwise (an unlock-all,
        // an import) before finishing The Scions of the Seventh Dawn lose it until they do (Bob, 2026-10-26.) Their
        // retainers stay; the bell answers that they haven't approval yet.
        if self
            .player_data
            .unlock
            .unlocks
            .contains(RETAINER_UNLOCK_LINK)
            && !self
                .player_data
                .quest
                .completed
                .contains(RETAINER_UNLOCK_QUEST)
        {
            tracing::info!(
                "{} has the retainers' unlock without having finished quest {RETAINER_UNLOCK_QUEST}: taken back",
                self.player_data.character.name
            );
            self.player_data.unlock.unlocks.clear(RETAINER_UNLOCK_LINK);
        }
    }

    /// Whether the character may hire and call retainers (the unlock The Scions of the Seventh Dawn gives.)
    pub fn retainers_unlocked(&self) -> bool {
        self.player_data
            .unlock
            .unlocks
            .contains(RETAINER_UNLOCK_LINK)
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

        // Quests changed, so the main scenario quest to show may have too.
        self.send_scenario_guide().await;
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

    /// Changes an active quest's variables (index 0-5, value) without telling the client: the next journal update
    /// (e.g. `set_quest_sequence`) carries them.
    pub fn preset_quest_variables(&mut self, id: u32, variables: &[(u8, u8)]) {
        let adjusted_id = adjust_quest_id(id);
        let Some(slot) = self.quest_slot(adjusted_id) else {
            return;
        };
        for (index, value) in variables {
            if let Some(variable) = self.player_data.quest.active.0[slot]
                .variables
                .get_mut(*index as usize)
            {
                *variable = *value;
            }
        }
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

    /// A battle NPC we fought was defeated: gives its EXP, rolls its drops (`battle/Drops.lua`), and lets every
    /// active quest know, so it can count it towards its objectives.
    pub async fn on_battle_npc_killed(
        &mut self,
        lua_player: &mut LuaPlayer,
        events: &mut Vec<(Box<dyn EventHandler>, Event)>,
        actor_id: ObjectId,
        base_id: u32,
        name_id: u32,
        layout_id: u32,
        level: u8,
    ) {
        let (exp, player_level) = {
            let mut game_data = self.game_data.lock();
            let exp = game_data.get_bnpc_exp(level);
            let level = self
                .synced_level
                .map(|level| level as u16)
                .unwrap_or_else(|| self.current_level(&game_data));
            (exp, level)
        };
        // The Hunting Log first: retail counted the kill (and gave a finished entry's EXP) before the kill's own EXP.
        self.record_hunting_log_kill(name_id).await;
        // Enemies well below you give less (retail: full down to 2 levels below, then 10 % less a level.)
        let mut exp = exp * kill_exp_percent(level as i32 - player_level as i32) / 100;
        // In a duty, retail gave players synced down no kill EXP at all, and nobody a kill chain (Bob's captures.)
        let in_duty = self.content_finder_condition_id != 0;
        let synced_down = self
            .synced_level
            .is_some_and(|synced| (synced as u16) < self.current_level(&self.game_data.lock()));
        if in_duty && synced_down {
            exp = 0;
        }
        // Only enemies at or above your level count towards a chain.
        let chain = self.exp_chain_bonus(!in_duty && level as u16 >= player_level);
        if let Some((index, window, _)) = chain {
            self.actor_control_self(ActorControlCategory::ExpChain {
                index,
                window: window * 100,
            })
            .await;
        }
        if exp > 0 {
            self.add_kill_exp(exp, chain.map(|(_, _, bonus)| bonus).unwrap_or(0), actor_id)
                .await;
        }
        // Their chocobo companion, if it's out, gets as much (a stand-in; `server/companion.rs`.)
        self.handle
            .send(ToServer::CompanionKill(
                self.player_data.character.actor_id,
                exp.max(0) as u32,
            ))
            .await;

        let quest_ids: Vec<u16> = self
            .player_data
            .quest
            .active
            .0
            .iter()
            .map(|quest| quest.id)
            .collect();

        // Scripts should see the latest quest state.
        lua_player.player_data = self.player_data.clone();

        for quest_id in quest_ids {
            let handler_id = HandlerId::new(HandlerType::Quest, quest_id);
            let Some(path) = quest_script_path(&self.game_data, handler_id) else {
                continue;
            };
            let Some(handler) =
                LuaEventHandler::new(handler_id, None, &path, self.game_data.clone())
            else {
                continue;
            };

            handler.on_battle_npc_killed(lua_player, base_id, layout_id);
        }

        roll_drops(lua_player, name_id, level);

        self.process_lua_player(lua_player, events).await;

        // "To Crush Your Enemies" counts it.
        self.add_to_achievement_counter(super::achievement::COUNTER_ENEMIES_DEFEATED, 1)
            .await;
        lua_player.player_data = self.player_data.clone();
    }

    /// The player is leaving a duty they cleared: every active quest hears of it (`onDutyCleared`), as retail moved
    /// It's Probably Pirates (245) along as Bob left Sastasha (Chronofoil capture of 2026-10-23).
    pub async fn on_duty_cleared(
        &mut self,
        lua_player: &mut LuaPlayer,
        events: &mut Vec<(Box<dyn EventHandler>, Event)>,
        content_id: u16,
    ) {
        let quest_ids: Vec<u16> = self
            .player_data
            .quest
            .active
            .0
            .iter()
            .map(|quest| quest.id)
            .collect();
        lua_player.player_data = self.player_data.clone();

        for quest_id in quest_ids {
            let handler_id = HandlerId::new(HandlerType::Quest, quest_id);
            let Some(path) = quest_script_path(&self.game_data, handler_id) else {
                continue;
            };
            let Some(handler) =
                LuaEventHandler::new(handler_id, None, &path, self.game_data.clone())
            else {
                continue;
            };
            handler.on_duty_cleared(lua_player, content_id);
        }

        self.process_lua_player(lua_player, events).await;
        lua_player.player_data = self.player_data.clone();
    }

    /// Moves the EXP chain along for a kill that counts and returns its place in the chain (from 0), the window the
    /// next kill has (in seconds) and its bonus in percent; None for a kill that doesn't count.
    ///
    /// From Bob's Chronofoil captures (141 kills): the bonus is 0, 20, 25, 30, 40, then 50 from the sixth kill on, and
    /// the next kill has 100, 80, 60, 40, 20, then 10 s after the last one to keep it going (it broke after 62 s at the
    /// third, held after 44.5.) Only enemies at or above your level count.
    pub fn exp_chain_bonus(&mut self, counts: bool) -> Option<(u32, u32, u32)> {
        if !counts {
            return None;
        }
        let now = std::time::Instant::now();
        let (count, last) = self.exp_chain;
        let index = match last {
            Some(last)
                if count > 0
                    && now.duration_since(last).as_secs_f32()
                        <= exp_chain_window(count - 1) as f32 =>
            {
                count
            }
            _ => 0,
        };
        self.exp_chain = (index + 1, Some(now));
        Some((
            index,
            exp_chain_window(index),
            exp_chain_bonus_percent(index),
        ))
    }

    /// A FATE this player helped with was completed: its EXP and gil (gold rank; retail also gave Grand Company seals,
    /// about 5.7 a level, and sometimes items, not done.)
    pub async fn give_fate_rewards(&mut self, fate_id: u32) {
        let Some((exp, gil)) = self.game_data.lock().get_fate_rewards(fate_id) else {
            return;
        };
        self.player_data.inventory.currency.get_slot_mut(0).quantity = self
            .player_data
            .inventory
            .currency
            .get_slot_mut(0)
            .quantity
            .saturating_add_unsigned(gil);
        self.send_inventory().await;
        if exp > 0 {
            self.add_exp(exp as i32).await;
        }
    }

    /// An item reward's log line: 1606 for one, 1607 with a count.
    async fn send_item_reward_log(&mut self, item_id: u32, count: u32) {
        if count > 1 {
            self.send_reward_log(QUEST_REWARD_ITEMS_LOG, vec![0, item_id, count])
                .await;
        } else {
            self.send_reward_log(QUEST_REWARD_ITEM_LOG, vec![0, item_id])
                .await;
        }
    }

    /// A reward's log line (EventLogMessage from no event.)
    async fn send_reward_log(&mut self, message_type: u32, params: Vec<u32>) {
        let message = EventLogMessage {
            handler_id: HandlerId(0),
            message_type,
            params,
        };
        if let Some(ipc) = message.package() {
            self.send_ipc_self(ipc).await;
        }
    }

    pub fn record_fate_completion(&mut self) {
        let mut game_data = self.game_data.lock();
        if let Some(row) = game_data.get_fate_progress_ui_row(self.player_data.volatile.zone_id) {
            self.player_data.quest.shared_fates.0[row as usize] += 1;
        }
    }
}

/// LogMessage rows for a quest's rewards: the gil, an item, items with a count.
const QUEST_REWARD_GIL_LOG: u32 = 1605;
const QUEST_REWARD_ITEM_LOG: u32 = 1606;
const QUEST_REWARD_ITEMS_LOG: u32 = 1607;

/// An EXP chain's bonus at this place in it (from 0), in percent.
fn exp_chain_bonus_percent(index: u32) -> u32 {
    match index {
        0 => 0,
        1 => 20,
        2 => 25,
        3 => 30,
        4 => 40,
        _ => 50,
    }
}

/// How long the next kill has to keep a chain going after the one at this place, in seconds.
fn exp_chain_window(index: u32) -> u32 {
    match index {
        0 => 100,
        1 => 80,
        2 => 60,
        3 => 40,
        4 => 20,
        _ => 10,
    }
}

/// How much of an enemy's EXP you get for one `difference` levels from you (the enemy's minus yours), in percent:
/// all of it down to 2 below, then 10 % less a level (Bob's captures: 70 % at 4 below, 60 % at 5, about 40 % at 7.)
fn kill_exp_percent(difference: i32) -> i32 {
    if difference >= -2 {
        100
    } else {
        (110 + 10 * difference).max(0)
    }
}

/// Runs `battle/Drops.lua`'s `onBNpcKill`, which gives the player whatever the defeated battle NPC dropped.
fn roll_drops(lua_player: &mut LuaPlayer, name_id: u32, level: u8) {
    let file_name = get_config()
        .filesystem
        .locate_script_file("battle/Drops.lua");
    let Ok(file) = std::fs::read(&file_name) else {
        tracing::warn!("Failed to load {file_name}, enemies won't drop anything!");
        return;
    };

    let lua = KawariLua::new();
    let result = lua.0.scope(|scope| {
        lua.0
            .load(file)
            .set_name("@".to_string() + &file_name)
            .exec()?;
        let player = scope.create_userdata_ref_mut(lua_player)?;
        let func: Function = lua.0.globals().get("onBNpcKill")?;
        func.call::<()>((name_id, level, player))
    });
    if let Err(err) = result {
        tracing::warn!("Error while calling onBNpcKill in {file_name}: {err:?}");
    }
}

#[cfg(test)]
mod exp_tests {
    use super::*;

    #[test]
    fn chain_and_level_penalty() {
        assert_eq!(
            (0..7).map(exp_chain_bonus_percent).collect::<Vec<_>>(),
            vec![0, 20, 25, 30, 40, 50, 50]
        );
        assert_eq!(exp_chain_window(2), 60);
        assert_eq!(exp_chain_window(9), 10);
        assert_eq!(kill_exp_percent(3), 100);
        assert_eq!(kill_exp_percent(-2), 100);
        assert_eq!(kill_exp_percent(-4), 70);
        assert_eq!(kill_exp_percent(-5), 60);
        assert_eq!(kill_exp_percent(-7), 40);
        assert_eq!(kill_exp_percent(-20), 0);
        // Retail: a level 6 conjurer got 38 of 55 for a level 2 enemy.
        assert_eq!(55 * kill_exp_percent(2 - 6) / 100, 38);
    }
}
