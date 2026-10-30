//! What the Gold Saucer shows of a GATE (`gate.rs` has the schedule; `zone_connection/gate.rs` tells each client of
//! its director): as one begins its entrance's NPC appears, with the GATE's director as its event, and the screens and
//! the square's set piece play their GATE timelines; as it's over they go back and the NPC goes. As retail did (Bob's
//! Chronofoil captures deaf7e28 and 2c98f9e8, the archive's dd09ac61, ea704111 and 61a9a4f3):
//! * the entrance: SpawnNpc of the event NPC whose layout ID is the GFATE row's first LGBPopRange (Air Force One's
//!   7773571, Sabotender Guia), its handler the director, at the GATE's start; DeleteActor at its end. Kawari's
//!   drop-in (`dropins/w1eb_planmap.json`) has them, each `nonpop` so they're out only for their GATE.
//! * the screens and set pieces: ActorControl 413 from each (`GATE_SCREENS`, `GateKind::stage`); a later spawn has the
//!   first number as its `args1`.

use std::sync::Arc;

use kawari::{common::timestamp_secs, config::get_config, ipc::zone::ActorControlCategory};
use parking_lot::Mutex;

use crate::{
    GameData,
    gate::{GATE_SCREENS, GATE_TERRITORY, Gate, GateShown, SCREEN_OFF, SCREEN_ON},
    server::{actor::NetworkedActor, instance::Instance, network::NetworkState},
};

/// An EObj's timeline states (ActorControl 413), remembered for those it's spawned for later.
const SHARED_GROUP_TIMELINE_STATES: u32 = 413;

fn play_states(
    network: &mut NetworkState,
    instance: &mut Instance,
    layout_id: u32,
    states: [u32; 2],
) {
    let Some(actor_id) = instance.find_object_by_eobj_or_layout_id(layout_id) else {
        return;
    };
    if let Some(NetworkedActor::Object { object, .. }) = instance.find_actor_mut(actor_id) {
        object.args1 = states[0];
    }
    network.send_ac_in_range_instance(
        instance,
        actor_id,
        ActorControlCategory::Unknown {
            category: SHARED_GROUP_TIMELINE_STATES,
            param1: states[0],
            param2: states[1],
            param3: 0,
            param4: 0,
            param5: 0,
        },
    );
}

/// Brings a GATE out in the Gold Saucer, or puts it away, as the time comes.
pub fn gate_tick(
    network: Arc<Mutex<NetworkState>>,
    instance: &mut Instance,
    game_data: Arc<Mutex<GameData>>,
) {
    if instance.zone.id != GATE_TERRITORY || instance.content_finder_condition_id != 0 {
        return;
    }
    let now = timestamp_secs() as u64;
    let current = Gate::current(&mut game_data.lock(), now);

    // The one shown is over (or another has begun.)
    if let Some(shown) = instance.gate_shown
        && current.is_none_or(|gate| gate.slot != shown.slot)
    {
        instance.gate_shown = None;
        let ended = Gate::in_slot(&mut game_data.lock(), shown.slot);
        let mut network = network.lock();
        if let Some(actor_id) = shown.entrance {
            network.remove_actor(instance, actor_id);
        }
        for screen in GATE_SCREENS {
            play_states(&mut network, instance, screen, SCREEN_OFF);
        }
        if let Some((stage, _, over)) = ended.and_then(|gate| gate.kind.stage) {
            play_states(&mut network, instance, stage, over);
        }
    }

    // One begins.
    if let Some(gate) = current
        && instance.gate_shown.is_none()
    {
        let entrance = instance
            .zone
            .get_battle_npc(gate.row.entrance)
            .map(|mut npc| {
                npc.common.handler_id = gate.director();
                let actor_id = Instance::generate_actor_id();
                let config = get_config();
                instance.insert_npc(actor_id, npc, &config, &mut game_data.lock());
                actor_id
            });
        if entrance.is_none() {
            tracing::warn!(
                "GATE row {}: no NPC {} for its entrance (dropins/w1eb_planmap.json)",
                gate.row.row,
                gate.row.entrance
            );
        }
        instance.gate_shown = Some(GateShown {
            slot: gate.slot,
            entrance,
        });
        let mut network = network.lock();
        for screen in GATE_SCREENS {
            play_states(&mut network, instance, screen, SCREEN_ON);
        }
        if let Some((stage, begins, _)) = gate.kind.stage {
            play_states(&mut network, instance, stage, begins);
        }
    }
}
