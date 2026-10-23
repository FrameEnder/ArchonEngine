// ! The friend system.
use crate::{
    ToServer, ZoneConnection,
    server::{BOT_LEVEL, bot_friends},
};
use kawari::{
    common::ObjectId,
    config::get_config,
    ipc::zone::{
        FriendGroupIconInfo, OnlineStatus, OnlineStatusMask, PlayerEntry, ServerZoneIpcData,
        ServerZoneIpcSegment, SocialListUIFlags,
    },
};
impl ZoneConnection {
    pub async fn refresh_friend_list(&mut self) {
        // Only refresh if we ran out of results from a prior run.
        if self.friend_results.is_empty() {
            let mut db = self.database.lock();
            let mut game_data = self.game_data.lock();
            self.friend_results =
                db.find_friend_list(&mut game_data, self.player_data.character.content_id);

            // The party bots kept as friends (`server/bot_friends.rs`): always online, here (so Invite to Party,
            // which calls one, can be chosen), at their level.
            let bots = bot_friends::list(self.player_data.character.content_id as u64);
            if !bots.is_empty() {
                self.friend_results
                    .retain(|entry| entry.content_id != 0 && !entry.name.is_empty());
                let config = get_config();
                for bot in bots {
                    self.friend_results.push(PlayerEntry {
                        content_id: bot.content_id,
                        timestamp: bot.added,
                        current_world_id: config.world.world_id,
                        ui_flags: SocialListUIFlags::ENABLE_CONTEXT_MENU,
                        zone_id: self.player_data.volatile.zone_id as u16,
                        online_status_mask: OnlineStatusMask::from_online_status(
                            OnlineStatus::Online,
                        ),
                        classjob_id: bot.classjob,
                        classjob_level: BOT_LEVEL,
                        home_world_id: config.world.world_id,
                        name: bot.name,
                        ..Default::default()
                    });
                }
            }

            self.friend_index = 0;
        }
    }

    pub fn add_to_friend_list(&mut self, friend_content_id: u64, pending: i32) {
        let mut db = self.database.lock();
        db.add_to_friend_list(
            friend_content_id as i64,
            self.player_data.character.content_id,
            pending,
        );
    }

    pub async fn remove_from_friend_list(&mut self, their_content_id: u64, their_name: String) {
        // A party bot kept as a friend: forgotten (`server/bot_friends.rs`.)
        if bot_friends::is_bot_content_id(their_content_id) {
            if bot_friends::remove(
                self.player_data.character.content_id as u64,
                their_content_id,
            ) {
                self.friend_results.clear();
                self.friend_removed(their_content_id, their_name).await;
            }
            return;
        }
        let their_actor_id;
        {
            let mut db = self.database.lock();
            their_actor_id = db.find_actor_id(their_content_id);

            // If we can't find them for some reason, don't proceed.
            if their_actor_id == ObjectId::default() {
                tracing::warn!(
                    "Unable to find {}'s actor id (it was {:#?})! What happened?)",
                    their_content_id,
                    ObjectId::default()
                );
                return;
            }

            // NOTE: This removes each other on both sides, so the receiver doesn't need to do this
            db.remove_from_friend_list(
                their_content_id as i64,
                self.player_data.character.content_id,
            );
        }

        self.handle
            .send(ToServer::FriendRemoved(
                self.player_data.character.actor_id,
                self.player_data.character.content_id as u64,
                self.player_data.character.name.clone(),
                their_actor_id,
                their_content_id,
                their_name,
            ))
            .await;
    }

    pub async fn friend_removed(&mut self, their_content_id: u64, their_name: String) {
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::FriendRemoved {
            content_id: their_content_id,
            name: their_name.clone(),
            unk1: 1,
        });

        self.send_ipc_self(ipc).await;
    }

    pub async fn set_friend_group_icon(&mut self, icon_info: &FriendGroupIconInfo) {
        {
            let mut db = self.database.lock();
            db.set_friend_group_icon(
                self.player_data.character.content_id,
                icon_info.content_id as i64,
                icon_info.icon,
            );
        }

        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::FriendGroupIcon(icon_info.clone()));
        self.send_ipc_self(ipc).await;
    }
}
