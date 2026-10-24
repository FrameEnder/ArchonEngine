use async_trait::async_trait;
use bitflags::bitflags;
use kawari::{
    common::{CharacterMode, ObjectTypeId},
    ipc::zone::{Condition, SceneFlags},
};

use crate::{Event, EventHandler, ZoneConnection, lua::LuaPlayer};

/// For gathering events: the node's window (`ZoneConnection::send_gathering_window`), each gather and what's left
/// (`zone_connection/gathering.rs`.)
#[derive(Debug)]
pub struct GatheringEventHandler {
    count: u8,
}

impl GatheringEventHandler {
    pub fn new(count: u8) -> Self {
        Self { count }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Default)]
pub struct GatheringItemFlag(u8);

bitflags! {
    impl GatheringItemFlag : u8 {
        const GATHERING_CHANCE_BONUS = 1;
        const BOON_CHANCE_BONUS = 2;
        const UNK1 = 4;
        const HIDDEN = 8;
        const RARE = 16;
        const BONUS = 32;
        const NOT_GATHERED_YET = 64;
        const UNK2 = 128;
    }
}

impl std::fmt::Debug for GatheringItemFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}

#[async_trait]
impl EventHandler for GatheringEventHandler {
    async fn on_talk(&self, _event: &Event, _target_id: ObjectTypeId, player: &mut LuaPlayer) {
        // The window needs the connection (what's left, the gatherer's stats), so it's sent from there.
        player.open_gathering_window();
    }

    async fn on_yield(
        &self,
        event: &Event,
        connection: &mut ZoneConnection,
        scene: u16,
        _yield_id: u8,
        results: &[i32],
        player: &mut LuaPlayer,
    ) {
        let point = event.id.event_id();
        let result = |i: usize| results.get(i).copied().unwrap_or_default();

        if scene == 0 && result(2) == 2 {
            // Gather the item in slot results[1], then the animation (scene 1), whose end brings us back below.
            connection.gather(event, result(1).max(0) as usize).await;
            player.play_scene(1, SceneFlags::NO_DEFAULT_CAMERA, vec![2, 266]);
            return;
        }

        if scene == 0 && result(2) == 0 {
            // Quit.
            connection.leave_gathering_point().await;
            player.finish_event();
            return;
        }

        // After a gather: the window again, or, if that was the last attempt, the node goes and the window closes.
        if connection.gathering.remaining(point, self.count) == 0 {
            connection.leave_gathering_point().await;
            connection.gathering_point_exhausted(point).await;
            player.finish_event();
            return;
        }
        connection.send_gathering_window(event, false).await;
    }

    fn condition(&self) -> Condition {
        Condition::ExecutingGatheringAction
    }

    fn character_mode(&self) -> CharacterMode {
        CharacterMode::Gathering
    }
}
