//! The player's side of the chocobo companion: what the Companion window changes (stance, barding, skills), the
//! saddlebag, and the Gysahl Greens used up. The companion in the world is the server's (`server/companion.rs`.)
//! See `docs/src/fork/chocobo.md`; the packets are from Bob's Chronofoil capture of 2026-10-24.

use kawari::{
    common::{INVENTORY_ACTION_ACK_GENERAL, LegacyEquipmentModelId},
    ipc::zone::{ContainerInfo, ItemOperation, ServerZoneIpcData, ServerZoneIpcSegment},
};

use crate::{
    ToServer, ZoneConnection,
    inventory::{PREMIUM_SADDLEBAG_PAGES, SADDLEBAG_PAGES},
    server::{
        COMMAND_FOLLOW, COMMAND_HEALER, COMMAND_WITHDRAW, CompanionProfile, MAX_RANK, eat_onion,
        gain_exp,
    },
};

/// Each skill line (Defender, Attacker, Healer) has ten skills (BuddySkill.)
const MAX_SKILL_LEVEL: u8 = 10;

impl ZoneConnection {
    /// The companion as the server needs it: its name, rank, EXP and skill points, stance, barding's models, colour.
    pub fn companion_profile(&self) -> CompanionProfile {
        let buddy = &self.player_data.buddy;
        let mut barding = [0u8; 3];
        for (slot, row) in barding.iter_mut().enumerate() {
            *row = buddy.barding.0.get(slot).copied().unwrap_or_default();
        }
        let mut models = [LegacyEquipmentModelId::default(); 3];
        {
            let mut game_data = self.game_data.lock();
            for (slot, model) in models.iter_mut().enumerate() {
                if let Some(rows) = game_data.get_buddy_equip_models(barding[slot] as u32) {
                    *model = LegacyEquipmentModelId::from(rows[slot]);
                }
            }
        }
        CompanionProfile {
            name: buddy.name.clone(),
            exp: buddy.exp.max(0) as u32,
            rank: buddy.rank.clamp(0, u8::MAX as i32) as u8,
            skill_points: buddy.skill_points.clamp(0, u8::MAX as i32) as u8,
            command: buddy.active_command.clamp(0, u8::MAX as i32) as u8,
            models,
            barding,
            color: buddy.color.clamp(0, u8::MAX as i32) as u8,
            levels: [0, 1, 2].map(|line| buddy.levels.0.get(line).copied().unwrap_or_default()),
        }
    }

    /// The companion gained EXP from a kill (`FromServer::CompanionExp`): it ranks up below rank 10 as its bar fills,
    /// each rank bringing its skill points; from 10 its bar stops full until it eats a Thavnairian Onion.
    pub async fn companion_gained_exp(&mut self, exp: u32) {
        let buddy = &mut self.player_data.buddy;
        let (rank, now, points) = gain_exp(
            buddy.rank.clamp(0, MAX_RANK as i32) as u8,
            buddy.exp.max(0) as u32,
            exp,
        );
        if rank != buddy.rank as u8 {
            tracing::info!("The companion reached rank {rank} (+{points} skill points)");
        }
        buddy.rank = rank as i32;
        buddy.exp = now.min(i32::MAX as u32) as i32;
        buddy.skill_points += points as i32;
        self.send_companion().await;
    }

    /// A Thavnairian Onion fed to the companion (its item script): from rank 10, with its EXP bar full, it ranks up
    /// and the onion is used up. Otherwise nothing happens and the onion is kept.
    pub async fn feed_companion_onion(&mut self, item_id: u32) {
        let buddy = &mut self.player_data.buddy;
        let Some((rank, exp, points)) = eat_onion(
            buddy.rank.clamp(0, MAX_RANK as i32) as u8,
            buddy.exp.max(0) as u32,
        ) else {
            tracing::info!(
                "The companion can't eat a Thavnairian Onion now (rank {}, EXP {})",
                buddy.rank,
                buddy.exp
            );
            return;
        };
        buddy.rank = rank as i32;
        buddy.exp = exp as i32;
        buddy.skill_points += points as i32;
        tracing::info!(
            "The companion ate a Thavnairian Onion: rank {rank} (+{points} skill points)"
        );
        self.item_used_up(item_id).await;
        self.send_companion().await;
    }

    /// Tells the server what the companion is now.
    pub async fn send_companion(&mut self) {
        let profile = self.companion_profile();
        self.handle
            .send(ToServer::SetCompanion(
                self.player_data.character.actor_id,
                profile,
            ))
            .await;
    }

    /// A command from the companion's menu (client trigger 1700): withdraw it, or a stance it keeps from then on.
    pub async fn companion_command(&mut self, command: u32) {
        let command = command.min(u8::MAX as u32) as u8;
        if command == COMMAND_WITHDRAW {
            self.handle
                .send(ToServer::WithdrawCompanion(
                    self.player_data.character.actor_id,
                ))
                .await;
            return;
        }
        if !(COMMAND_FOLLOW..=COMMAND_HEALER).contains(&command) {
            tracing::warn!("Unknown companion command {command}");
            return;
        }
        self.player_data.buddy.active_command = command as i32;
        self.send_companion().await;
    }

    /// Barding put on in the Companion window (client trigger 1701): `slot` 0 head, 1 body, 2 legs, a BuddyEquip row
    /// the character has (0 takes it off.)
    pub async fn set_companion_barding(&mut self, slot: u32, equip: u32) {
        if slot > 2 || equip > u8::MAX as u32 {
            return;
        }
        if equip != 0 && !self.player_data.companion.unlocked_equip.contains(equip) {
            tracing::warn!("Barding {equip} isn't one the character has");
            return;
        }
        let mut barding = self.player_data.buddy.barding.0.clone();
        barding.resize(3, 0);
        barding[slot as usize] = equip as u8;
        self.player_data.buddy.barding.0 = barding;
        self.send_companion().await;
    }

    /// The next skill of a line learnt (client trigger 1702: 0 Defender, 1 Attacker, 2 Healer.) It costs as many
    /// points as the skill's place in its line: retail's companion, 11 points with Attacker at 4, had 6 after the
    /// fifth and none after the sixth.
    pub async fn learn_companion_skill(&mut self, line: u32) {
        let Some(level) = self.player_data.buddy.levels.0.get(line as usize).copied() else {
            return;
        };
        let cost = level as i32 + 1;
        if level >= MAX_SKILL_LEVEL || self.player_data.buddy.skill_points < cost {
            tracing::warn!(
                "Can't learn companion skill line {line}: level {level}, points {}",
                self.player_data.buddy.skill_points
            );
            return;
        }
        self.player_data.buddy.skill_points -= cost;
        self.player_data.buddy.levels.0[line as usize] = level + 1;
        self.send_companion().await;
    }

    /// The saddlebag opened (client trigger 444): its two pages with their items, then the premium saddlebag's two,
    /// empty, as retail sent for an account without it.
    pub async fn open_saddlebag(&mut self) {
        let inventory = self.player_data.inventory.clone();
        for container_type in SADDLEBAG_PAGES {
            if let Some(container) = inventory.get_container(container_type) {
                self.send_container(container, container_type).await;
            }
        }
        for container_type in PREMIUM_SADDLEBAG_PAGES {
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ContainerInfo(ContainerInfo {
                sequence: self.player_data.item_sequence,
                num_items: 0,
                container: container_type,
                ..Default::default()
            }));
            self.send_ipc_self(ipc).await;
            self.player_data.item_sequence += 1;
        }
    }

    /// Store into or take from the saddlebag from its window (`SaddlebagStore` / `SaddlebagTake`): where it went is
    /// the server's choice, so both slots are sent, then the acknowledgement, as retail did.
    pub async fn saddlebag_item_operation(&mut self, action: &ItemOperation) {
        if let Some(slots) = self.player_data.inventory.transfer_with_saddlebag(action) {
            for (container, slot) in slots {
                self.send_inventory_slot(container, slot).await;
            }
        }
        self.send_inventory_ack(action.context_id, INVENTORY_ACTION_ACK_GENERAL, 0)
            .await;
        self.player_data.item_sequence += 1;
    }

    /// An item used up (Gysahl Greens): one fewer in the bags, and the slot sent again.
    pub async fn item_used_up(&mut self, item_id: u32) {
        if let Some((container, slot)) = self.player_data.inventory.use_up_one_at(item_id) {
            self.send_inventory_slot(container, slot).await;
        }
    }
}
