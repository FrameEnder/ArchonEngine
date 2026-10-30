//! The GATEs as the client in the Gold Saucer hears of them (`gate.rs` has the schedule): the director at zone-in,
//! its beginning and end, and the answers to the client's questions about it.

use kawari::{
    common::{DirectorEvent, DirectorTrigger, HandlerId, ObjectTypeId, timestamp_secs},
    ipc::zone::{ActorControlCategory, ServerZoneIpcData, ServerZoneIpcSegment},
};

use crate::{
    ZoneConnection,
    gate::{GATE_TERRITORY, Gate, NPC_YELL_GATE_BEGINS, PHASE_OVER},
};

/// The client asking about the GATE (client trigger 808 to its director): `[1]`, then `[0]`, answered after the second.
const TRIGGER_ASK: u32 = 0;

/// DirectorEvent IDs of the GoldSaucer director (see `gate.rs`.)
const EVENT_JOINED: u32 = 5;
const EVENT_ENTRANCE: u32 = 6;
const EVENT_RESULT: u32 = 8;
const EVENT_GATE: u32 = 9;
const EVENT_PHASE: u32 = 11;

impl ZoneConnection {
    fn in_gold_saucer(&self) -> bool {
        self.player_data.volatile.zone_id as u16 == GATE_TERRITORY
    }

    async fn gate_event(&mut self, director: HandlerId, id: u32, args: [u32; 4]) {
        self.actor_control_self(ActorControlCategory::DirectorEvent {
            handler_id: director,
            event: DirectorEvent::Unknown {
                id,
                arg1: args[0],
                arg2: args[1],
                arg3: args[2],
                arg4: args[3],
            },
        })
        .await;
    }

    /// The GATE on now, if any.
    pub fn current_gate(&self) -> Option<Gate> {
        Gate::current(&mut self.game_data.lock(), timestamp_secs() as u64)
    }

    /// Whether the player has joined this GATE (they can play each once.)
    pub fn joined_gate(&self, gate: &Gate) -> bool {
        self.gate_joined == Some(gate.slot)
    }

    /// Coming into the Gold Saucer while a GATE is on: its director, as retail sent it right after ZoneInit.
    pub async fn send_gate_director(&mut self) {
        if !self.in_gold_saucer() {
            return;
        }
        if let Some(gate) = self.current_gate() {
            self.actor_control_self(ActorControlCategory::InitDirector {
                handler_id: gate.director(),
                content_id: gate.kind.content_id,
                flags: 0,
            })
            .await;
        }
    }

    /// When the next GATE begins or ends, for the connection's timer.
    pub fn next_gate_change(&self) -> u64 {
        crate::gate::next_change(self.gate_checked.max(timestamp_secs() as u64 - 1))
    }

    /// A GATE began or ended since the last look: everyone in the Gold Saucer is told.
    pub async fn gate_changed(&mut self) {
        let now = timestamp_secs() as u64;
        let since = self.gate_checked;
        self.gate_checked = now;
        if since == 0 || !self.in_gold_saucer() || !self.player_data.character.actor_id.is_valid() {
            return;
        }

        let slot = crate::gate::slot_at(now);
        for slot in slot.saturating_sub(1)..=slot {
            let gate = Gate::in_slot(&mut self.game_data.lock(), slot);
            let Some(gate) = gate else {
                continue;
            };
            if since < gate.start && gate.start <= now {
                self.announce_gate(&gate).await;
            }
            if since < gate.end && gate.end <= now {
                self.gate_event(gate.director(), EVENT_PHASE, [PHASE_OVER, 0, 0, 0])
                    .await;
                self.actor_control_self(ActorControlCategory::TerminateDirector {
                    handler_id: gate.director(),
                })
                .await;
            }
        }
    }

    /// A GATE begins: NpcYell 2450 (`[location, type]`), its director and the registration phase.
    async fn announce_gate(&mut self, gate: &Gate) {
        let name_id = self
            .game_data
            .lock()
            .get_npc_yell_name_id(NPC_YELL_GATE_BEGINS)
            .unwrap_or_default();
        self.send_ipc_self(ServerZoneIpcSegment::new(ServerZoneIpcData::NpcYell {
            object_id: ObjectTypeId::default(),
            name_id,
            npc_yell_id: NPC_YELL_GATE_BEGINS,
            param1: gate.row.location as u32,
            param2: gate.row.kind as u32,
            param3: 0,
            param4: 0,
        }))
        .await;
        self.actor_control_self(ActorControlCategory::InitDirector {
            handler_id: gate.director(),
            content_id: gate.kind.content_id,
            flags: 0,
        })
        .await;
        let phase = gate.phase(timestamp_secs() as u64);
        self.gate_event(gate.director(), EVENT_PHASE, [phase, 0, 0, 0])
            .await;
    }

    /// The client asks its GATE director what's on (client trigger 808): which GATE, where its entrance is and until
    /// when, whether the player has joined it, and the phase.
    pub async fn gate_trigger(&mut self, director: HandlerId, trigger: &DirectorTrigger) {
        let DirectorTrigger::Unknown { id, .. } = trigger else {
            return;
        };
        if *id != TRIGGER_ASK {
            return;
        }
        let Some(gate) = self.current_gate() else {
            return;
        };
        if gate.director() != director {
            return;
        }
        self.gate_event(director, EVENT_GATE, [gate.row.row, gate.place, 0, 0])
            .await;
        self.gate_event(
            director,
            EVENT_ENTRANCE,
            [gate.row.entrance, gate.registration_end as u32, 0, 0],
        )
        .await;
        if self.joined_gate(&gate) {
            self.gate_event(director, EVENT_RESULT, [1, 0, 0, 0]).await;
            self.gate_event(
                director,
                EVENT_JOINED,
                [1, gate.start as u32, gate.kind.registration as u32, 0],
            )
            .await;
        }
        let phase = gate.phase(timestamp_secs() as u64);
        self.gate_event(director, EVENT_PHASE, [phase, 0, 0, 0])
            .await;
    }
}
