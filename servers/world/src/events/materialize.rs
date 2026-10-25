//! Materialize events (handler type 57) Kawari does: materia extraction (0x390001) and self-repair with Dark Matter
//! (0x390003). Started by ClientTrigger 2800 (`ZoneConnection::begin_materialize`); scene 0's yield gives the result
//! (`ZoneConnection::finish_materialize`), and its return finishes the event as for retail's desyntheses (Bob's
//! Chronofoil capture of 2026-09-15; docs/src/fork/spiritbond.md.)

use async_trait::async_trait;
use kawari::ipc::zone::Condition;

use crate::{Event, EventHandler, ZoneConnection, lua::LuaPlayer};

#[derive(Debug, Default)]
pub struct MaterializeEventHandler;

#[async_trait]
impl EventHandler for MaterializeEventHandler {
    async fn on_yield(
        &self,
        event: &Event,
        connection: &mut ZoneConnection,
        _scene: u16,
        _yield_id: u8,
        _results: &[i32],
        _player: &mut LuaPlayer,
    ) {
        connection.finish_materialize(event).await;
    }

    /// Retail set condition 14 for the desynthesis event.
    fn condition(&self) -> Condition {
        Condition::Occupied39
    }

    /// Retail sent ActorControlSelf 138 with the handler after the EventStart.
    fn disable_event_pos_rollback(&self) -> bool {
        true
    }
}
