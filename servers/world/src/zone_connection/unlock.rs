//! Utility functions for managing unlock data and bitmasks.

use crate::{ZoneConnection, bitmask::Bitmask};
use icarus::{Adventure::AdventureSheet, AetherCurrent::AetherCurrentSheet};
use kawari::{common::value_to_flag_byte_index_value, ipc::zone::ActorControlCategory};

impl ZoneConnection {
    /// The city aethernets' extra destinations (the gates, the airship landing): once every shard of a city's
    /// aethernet is attuned, its extras are learned (LearnTeleport each, after the last shard's), those that need a
    /// quest once it's done. Retail, Bob's Chronofoil capture cc62ad94 of 2026-10-28: the last of Limsa Lominsa's,
    /// Gridania's and Ul'dah's shards each brought their gates and airship landing at once (45, 46, 93; 31, 32, 94; 38,
    /// 39, 40, 95), before the city's big Aetheryte in Ul'dah, but not the White Wolf Gate (54), which needs quest 66338.
    pub async fn learn_aethernet_extras(&mut self) {
        let groups = self.game_data.lock().aethernet_extras();
        let mut learned = Vec::new();
        for (_, shards, extras) in groups {
            let has = |id: u32| self.player_data.aetheryte.unlocked.contains(id);
            if !shards.iter().all(|shard| has(*shard)) {
                continue;
            }
            for (extra, quest) in extras {
                if has(extra) || (quest != 0 && !self.quest_completed(quest)) {
                    continue;
                }
                learned.push(extra);
            }
        }
        for id in learned {
            self.player_data.aetheryte.unlocked.set(id);
            tracing::info!("Aethernet extra destination {id} learned");
            self.actor_control_self(ActorControlCategory::LearnTeleport { id, unlocked: true })
                .await;
        }
    }

    pub async fn toggle_orchestrion(&mut self, orchestrion_id: u32) {
        let should_unlock = self
            .player_data
            .unlock
            .orchestrion_rolls
            .toggle(orchestrion_id);

        let mut item_id = 0;

        if should_unlock {
            {
                let mut game_data = self.game_data.lock();
                item_id = game_data
                    .find_orchestrion_item_id(orchestrion_id)
                    .unwrap_or(0);
            }
        }

        self.actor_control_self(ActorControlCategory::ToggleOrchestrionUnlock {
            song_id: orchestrion_id,
            unlocked: should_unlock,
            item_id,
        })
        .await;
    }

    pub async fn toggle_glasses_style(&mut self, glasses_style_id: u32) {
        let should_unlock = self
            .player_data
            .unlock
            .glasses_styles
            .toggle(glasses_style_id);

        self.actor_control_self(ActorControlCategory::ToggleGlassesStyleUnlock {
            id: glasses_style_id,
            unlocked: should_unlock,
        })
        .await;
    }

    pub async fn toggle_ornament(&mut self, ornament_id: u32) {
        let should_unlock = self.player_data.unlock.ornaments.toggle(ornament_id);

        self.actor_control_self(ActorControlCategory::ToggleOrnamentUnlock {
            id: ornament_id,
            unlocked: should_unlock,
        })
        .await;
    }

    pub async fn unlock_buddy_equip(&mut self, buddy_equip_id: u32) {
        self.player_data
            .companion
            .unlocked_equip
            .set(buddy_equip_id);

        self.actor_control_self(ActorControlCategory::BuddyEquipUnlock { id: buddy_equip_id })
            .await;
    }

    pub async fn toggle_chocobo_taxi_stand(&mut self, chocobo_taxi_stand_id: u32) {
        let should_unlock = self
            .player_data
            .unlock
            .chocobo_taxi_stands
            .toggle(chocobo_taxi_stand_id);

        self.actor_control_self(ActorControlCategory::ToggleChocoboTaxiStandUnlock {
            id: chocobo_taxi_stand_id,
            unlocked: should_unlock,
        })
        .await;
    }

    pub async fn toggle_caught_fish(&mut self, caught_fish_id: u32) {
        let (value, index) = value_to_flag_byte_index_value(caught_fish_id);

        self.player_data.unlock.caught_fish.data[index as usize] ^= value;

        self.actor_control_self(ActorControlCategory::SetCaughtFishBitmask {
            index: index as u32,
            value: self.player_data.unlock.caught_fish.data[index as usize] as u32,
        })
        .await;
    }

    pub async fn toggle_caught_spearfish(&mut self, caught_spearfish_id: u32) {
        let (value, index) = value_to_flag_byte_index_value(caught_spearfish_id);

        self.player_data.unlock.caught_spearfish.data[index as usize] ^= value;

        self.actor_control_self(ActorControlCategory::SetCaughtSpearfishBitmask {
            index: index as u32,
            value: self.player_data.unlock.caught_spearfish.data[index as usize] as u32,
        })
        .await;
    }

    pub async fn toggle_triple_triad_card(&mut self, triple_triad_card_id: u32) {
        let should_unlock = self
            .player_data
            .unlock
            .triple_triad_cards
            .toggle(triple_triad_card_id);

        self.actor_control_self(ActorControlCategory::ToggleTripleTriadCardUnlock {
            id: triple_triad_card_id,
            unlocked: should_unlock,
        })
        .await;
    }

    // TODO: make logic that determines if all_vistas_recorded should be true or false automatically
    pub async fn toggle_adventure(&mut self, adventure_id: u32, all_vistas_recorded: bool) {
        let should_unlock = self.player_data.unlock.adventures.toggle(adventure_id);

        self.actor_control_self(ActorControlCategory::ToggleAdventureUnlock {
            id: adventure_id + AdventureSheet::STARTING_ROW,
            all_vistas_recorded,
            unlocked: should_unlock,
        })
        .await;
    }

    pub async fn toggle_cutscene_seen(&mut self, cutscene_id: u32, value: bool) {
        if value {
            self.player_data.unlock.cutscene_seen.set(cutscene_id);
        } else {
            self.player_data.unlock.cutscene_seen.clear(cutscene_id);
        }

        self.actor_control_self(ActorControlCategory::ToggleCutsceneSeen {
            id: cutscene_id,
            unlocked: value,
        })
        .await;
    }

    pub async fn toggle_minion(&mut self, minion_id: u32) {
        let should_unlock = self.player_data.unlock.minions.toggle(minion_id);

        self.actor_control_self(ActorControlCategory::ToggleMinionUnlock {
            minion_id,
            unlocked: should_unlock,
        })
        .await;
    }

    pub async fn toggle_aether_current(&mut self, aether_current_id: u32) {
        let aether_current_set;
        {
            let mut game_data = self.game_data.lock();
            aether_current_set = game_data.find_aether_current_set(aether_current_id as i32);
        }

        if let Some(aether_current_set_id) = aether_current_set {
            let should_unlock = self
                .player_data
                .aether_current
                .unlocked
                .toggle(aether_current_id - AetherCurrentSheet::STARTING_ROW);

            if should_unlock {
                let currents_needed_for_zone;
                let screen_image_id;

                {
                    let mut game_data = self.game_data.lock();

                    currents_needed_for_zone = game_data
                        .get_aether_currents_from_zone(aether_current_set_id)
                        .unwrap();

                    screen_image_id = game_data
                        .get_screenimage_from_aether_current_comp_flg_set(aether_current_set_id)
                        .unwrap();
                }

                let mut zone_complete = true;

                for current_needed in currents_needed_for_zone {
                    let current_unlocked = self.player_data.aether_current.unlocked.contains(
                        (current_needed - AetherCurrentSheet::STARTING_ROW as i32) as u32,
                    );

                    if !current_unlocked {
                        zone_complete = false;
                        break;
                    }
                }

                self.actor_control_self(ActorControlCategory::ToggleAetherCurrentUnlock {
                    id: aether_current_id,
                    attunement_complete: zone_complete,
                    padding: 0,
                    screen_image_id: screen_image_id as u16,
                    zone_id: aether_current_set_id as u8,
                    unk1: zone_complete,
                    show_flying_mounts_help: false,
                    remove_aether_current: false,
                })
                .await;
            } else {
                self.actor_control_self(ActorControlCategory::ToggleAetherCurrentUnlock {
                    id: aether_current_id,
                    attunement_complete: false,
                    padding: 0,
                    screen_image_id: 0,
                    zone_id: aether_current_set_id as u8,
                    unk1: false,
                    show_flying_mounts_help: false,
                    remove_aether_current: true,
                })
                .await;
            }
        }
    }

    pub async fn toggle_aether_current_comp_flg_set(
        &mut self,
        aether_current_comp_flg_set_id: u32,
    ) {
        // Because AetherCurrentCompFlgSet starts at Index 1, we need to adjust the mask so this gives the proper values
        let should_unlock = self
            .player_data
            .aether_current
            .comp_flg_set
            .toggle(aether_current_comp_flg_set_id - 1);

        let screen_image_id;
        {
            let mut game_data = self.game_data.lock();
            screen_image_id = game_data
                .get_screenimage_from_aether_current_comp_flg_set(aether_current_comp_flg_set_id)
                .unwrap();
        }

        self.actor_control_self(ActorControlCategory::ToggleAetherCurrentUnlock {
            id: 0xFFFFFFFF, // The client does a check, if (as of 7.31h) id is greater than 56, then no individual Aether Current logic is done. This, hopefully, lasts for long.
            attunement_complete: should_unlock,
            padding: 0,
            screen_image_id: screen_image_id as u16,
            zone_id: aether_current_comp_flg_set_id as u8,
            unk1: should_unlock,
            show_flying_mounts_help: false,
            remove_aether_current: !should_unlock,
        })
        .await;

        // A zone's aether currents all attuned completes its "Freebird" achievement.
        self.check_achievements(true).await;
    }

    // Difference between this an toggle_orchestrion, is that this one doesn't send an ActorControlSelf
    // The GM command itself manages the unlock for the client, so it isn't needed here
    pub fn gm_set_orchestrion(&mut self, value: bool, orchestrion_id: u32) {
        if value {
            self.player_data
                .unlock
                .orchestrion_rolls
                .set(orchestrion_id);
        } else {
            self.player_data
                .unlock
                .orchestrion_rolls
                .clear(orchestrion_id);
        }
    }

    /// Learns what an item teaches and uses one up, as using it does on retail. `kind` comes from `items/Items.lua`,
    /// which names the ItemAction types the client's own check (UIState::IsItemActionUnlocked, `0x868890`) knows:
    /// "minion" (853, Data[0]), "buddy_equip" (1013), "mount" (1322), "unlock" (2633 and 43141: an unlock link, which
    /// is what emotes, hairstyles and many other items teach), "triple_triad_card" (3357, the item's AdditionalData),
    /// "ornament" (20086), "orchestrion" (25183) and "glasses" (37312, a Glasses row). Already known: nothing happens
    /// and the item is kept (the client normally won't let it be used then.)
    pub async fn learn_from_item(&mut self, kind: &str, id: u32, item_id: u32) {
        fn has<const N: usize>(mask: &Bitmask<N>, id: u32) -> bool {
            (id as usize / 8) < mask.data.len() && mask.contains(id)
        }

        let unlock = &self.player_data.unlock;
        let known = match kind {
            "minion" => has(&unlock.minions, id),
            "mount" => {
                let order = self.game_data.lock().find_mount_order(id).unwrap_or(-1);
                order < 0 || has(&unlock.mounts, order as u32)
            }
            "unlock" => has(&unlock.unlocks, id),
            "buddy_equip" => has(&self.player_data.companion.unlocked_equip, id),
            "triple_triad_card" => has(&unlock.triple_triad_cards, id),
            "ornament" => has(&unlock.ornaments, id),
            "orchestrion" => has(&unlock.orchestrion_rolls, id),
            "glasses" => match self.game_data.lock().get_glasses_style(id) {
                Some(style) => has(&unlock.glasses_styles, style),
                None => true,
            },
            _ => {
                tracing::warn!("Item {item_id} teaches something unknown ({kind} {id})");
                return;
            }
        };
        if known {
            tracing::info!("Item {item_id}: already knows {kind} {id}");
            return;
        }

        tracing::info!("Item {item_id} teaches {kind} {id}");
        match kind {
            "minion" => self.toggle_minion(id).await,
            "mount" => {
                let order = self.game_data.lock().find_mount_order(id).unwrap_or(0);
                self.player_data.unlock.mounts.set(order as u32);
                self.actor_control_self(ActorControlCategory::ToggleMountUnlock {
                    order: order as u32,
                    id,
                    unlocked: true,
                })
                .await;
            }
            "unlock" => {
                self.player_data.unlock.unlocks.set(id);
                self.actor_control_self(ActorControlCategory::ToggleUnlock { id, unlocked: true })
                    .await;
            }
            "buddy_equip" => self.unlock_buddy_equip(id).await,
            "triple_triad_card" => self.toggle_triple_triad_card(id).await,
            "ornament" => self.toggle_ornament(id).await,
            "orchestrion" => self.toggle_orchestrion(id).await,
            "glasses" => {
                let style = self.game_data.lock().get_glasses_style(id).unwrap_or(0);
                self.toggle_glasses_style(style).await;
            }
            _ => {}
        }

        if self.player_data.inventory.use_up_one(item_id) {
            self.send_inventory().await;
        }
    }
}

impl ZoneConnection {
    /// Whether this character has unlocked an InstanceContent (the bitmasks `unlock_content` sets, with its offsets:
    /// dungeons from 1, guildhests from 10001, trials from 20001, raids from 30001.) Other kinds count as unlocked.
    pub fn has_unlocked_content(&self, game_data: &mut crate::GameData, content: u16) -> bool {
        let content_data = &self.player_data.content;
        match game_data.find_type_for_content(content) {
            Some(kawari::common::InstanceContentType::Dungeon) => content
                .checked_sub(1)
                .is_some_and(|i| content_data.unlocked_dungeons.contains(i as u32)),
            Some(kawari::common::InstanceContentType::Guildhests) => content
                .checked_sub(10001)
                .is_some_and(|i| content_data.unlocked_guildhests.contains(i as u32)),
            Some(kawari::common::InstanceContentType::Trial) => content
                .checked_sub(20001)
                .is_some_and(|i| content_data.unlocked_trials.contains(i as u32)),
            Some(kawari::common::InstanceContentType::Raid) => content
                .checked_sub(30001)
                .is_some_and(|i| content_data.unlocked_raids.contains(i as u32)),
            _ => true,
        }
    }

    /// A duty for a roulette among those this character (who queues, so the party's leader) has unlocked and is high
    /// enough for on their current class; None if there are none.
    pub fn pick_roulette_duty(
        &self,
        game_data: &mut crate::GameData,
        roulette: crate::Roulette,
        limited_leveling: bool,
    ) -> Option<u16> {
        let level = self.current_level(game_data) as u8;
        // Limited Leveling Roulette: the party's levels, the player's own first (`duty_finder::limited_leveling_allows`.)
        let mut party_levels = vec![level];
        party_levels.extend(self.bot_party_levels.iter().copied());
        let limited = limited_leveling && matches!(roulette, crate::Roulette::Leveling);
        // Retail only rolls duties the party has unlocked and is high enough for (Bob, 2026-10-24: Kawari rolled any.)
        let allowed: Vec<u16> = game_data
            .roulette_duties(roulette)
            .into_iter()
            .filter(|(_, content, required)| {
                *required <= level
                    && content.is_none_or(|c| self.has_unlocked_content(game_data, c))
                    && (!limited
                        || crate::server::duty_finder::limited_leveling_allows(
                            *required,
                            &party_levels,
                        ))
            })
            .map(|(id, _, _)| id as u16)
            .collect();
        fastrand::choice(allowed)
    }
}
