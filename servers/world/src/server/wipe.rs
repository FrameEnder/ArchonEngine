//! A party wipe in a duty: every player down. Retail (the Bowl of Embers with Duty Support, Bob's Chronofoil capture
//! b8432613 of 2026-10-27, visit 0.16: the player fell, the Duty Support members still standing) then:
//!
//! * 1.0 s after the player fell, the director's PartyWipe (0x40000005);
//! * 4.1 s after, 0x80000029 and 0x40000013, the time left (0x4000000F), each player sent back to the start alive
//!   (PrepareZoning to the same territory with warp type 25, `Event`, the character hidden; ActorSetPos there, the
//!   same warp type; their mode back to normal), 0x40000011, the party's NPCs and the duty's own (Ifrit) spawned again
//!   where they began, the entrance circle shown again;
//! * 11.3 s after, DutyRecommence (0x40000006, the time left, 0, 2) and the entrance circle hidden: the fight can begin
//!   again.
//!
//! The director's script resets its own part (`onPartyWipe(director)`, as everyone is sent back: the Bowl of Embers'
//! Ifrit taken away and brought back) and can begin again (`onDutyRecommence(director)`).

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use kawari::{
    common::{
        ContentDirectorEvent, DirectorEvent, EventState, HandlerType, InstanceContentDirectorEvent,
        ObjectId, WarpType,
    },
    ipc::zone::{
        ActorControlCategory, ActorControlSelf, PrepareZoning, PrepareZoningFlag,
        ServerZoneIpcData, ServerZoneIpcSegment,
    },
};
use parking_lot::Mutex;

use crate::{
    ClientId, FromServer, GameData,
    server::{
        actor::{NetworkedActor, set_character_mode},
        instance::Instance,
        network::{DestinationNetwork, NetworkState},
        party_bot,
    },
};
use kawari::common::CharacterMode;

/// Retail's timing, from the player falling (see the module.)
const PARTY_WIPE_AFTER: Duration = Duration::from_millis(1000);
const RETURN_AFTER: Duration = Duration::from_millis(4100);
const RECOMMENCE_AFTER: Duration = Duration::from_millis(11300);
/// The party's NPCs get up beside the one they follow once that player has arrived at the start (retail spawned them
/// there 0.4 s after the player's ActorSetPos): once the player's position is theirs again (their client says where
/// they are) more than this far from where they fell, or at the recommencement at the latest. At a fixed 6.1 s they
/// got up where the player had fallen, in the arena, before the player's client had reported its arrival (Bob's
/// bundle of 2026-10-28.)
const ARRIVED_DISTANCE: f32 = 1.0;

/// The director events retail sent as everyone went back, with no name in Kawari yet.
const WIPE_EVENT_29: u32 = 0x80000029;
const WIPE_EVENT_13: u32 = 0x40000013;
const WIPE_EVENT_TIME_LEFT: u32 = 0x4000000F;
const WIPE_EVENT_11: u32 = 0x40000011;
/// DutyRecommence's third value (retail's 2.)
const RECOMMENCE_ARG3: u32 = 2;

/// A wipe under way: when the last player fell, and how far it has got.
#[derive(Debug, Clone, Copy)]
pub struct PartyWipe {
    pub fell: Instant,
    pub stage: u8,
    /// The first player's position as they were sent back (where they lay.)
    pub lay_at: Option<glam::Vec3A>,
}

fn players(instance: &Instance) -> Vec<(ObjectId, u32)> {
    instance
        .actors
        .iter()
        .filter_map(|(id, actor)| match actor {
            NetworkedActor::Player { spawn, .. } => Some((*id, spawn.common.health_points)),
            _ => None,
        })
        .collect()
}

fn first_player_position(instance: &Instance) -> Option<glam::Vec3A> {
    instance.actors.values().find_map(|actor| match actor {
        NetworkedActor::Player { spawn, .. } => Some(spawn.common.position.0),
        _ => None,
    })
}

fn send_director_event(
    network: &mut NetworkState,
    instance: &Instance,
    event: DirectorEvent,
) -> Option<()> {
    let handler_id = instance.content_handler_id()?;
    let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorControlSelf(ActorControlSelf {
        category: ActorControlCategory::DirectorEvent { handler_id, event },
    }));
    for (player, _) in players(instance) {
        network.send_to_by_actor_id(
            player,
            FromServer::PacketSegment(ipc.clone(), player),
            DestinationNetwork::ZoneClients,
        );
    }
    Some(())
}

fn unnamed(id: u32, arg1: u32, arg2: u32, arg3: u32) -> DirectorEvent {
    DirectorEvent::Unknown {
        id,
        arg1,
        arg2,
        arg3,
        arg4: 0,
    }
}

fn time_left(instance: &Instance) -> u32 {
    match (instance.duty_clock, instance.duration) {
        (Some((started, _)), Some(duration)) => duration
            .saturating_sub(Instant::now().duration_since(started))
            .as_secs() as u32,
        _ => 0,
    }
}

/// Shows (or hides) the duty's entrance circle, as retail did around a wipe.
fn entrance_circle(network: &mut NetworkState, instance: &mut Instance, shown: bool) {
    let Some(circle) = instance.find_entrance_circle() else {
        return;
    };
    let state = if shown {
        EventState::empty()
    } else {
        EventState::OFF | EventState::UNK2 | EventState::UNK3
    };
    if let Some(NetworkedActor::Object { object, .. }) = instance.find_actor_mut(circle) {
        object.event_state = state;
    }
    network.send_ac_in_range_instance(
        instance,
        circle,
        ActorControlCategory::SetEventState { state },
    );
}

/// Watches a running InstanceContent for every player down and plays the wipe out (see the module.) Called every
/// tick.
pub fn wipe_tick(
    network: Arc<Mutex<NetworkState>>,
    instance: &mut Instance,
    game_data: Arc<Mutex<GameData>>,
) {
    let is_instance_content = instance
        .content_handler_id()
        .is_some_and(|id| id.handler_type() == HandlerType::InstanceContent);
    // Only while the duty runs: from its commencement until it's complete (the clock stops.)
    if !is_instance_content || instance.duty_clock.is_none() {
        instance.party_wipe = None;
        return;
    }

    let now = Instant::now();
    let Some(mut wipe) = instance.party_wipe else {
        let players = players(instance);
        if !players.is_empty() && players.iter().all(|(_, hp)| *hp == 0) {
            tracing::info!("Party wipe: every player is down");
            instance.party_wipe = Some(PartyWipe {
                fell: now,
                stage: 0,
                lay_at: None,
            });
        }
        return;
    };
    let since = now.duration_since(wipe.fell);

    if wipe.stage == 0 && since >= PARTY_WIPE_AFTER {
        wipe.stage = 1;
        send_director_event(
            &mut network.lock(),
            instance,
            DirectorEvent::ContentDirector(ContentDirectorEvent::InstanceContent(
                InstanceContentDirectorEvent::PartyWipe {
                    arg1: 0,
                    arg2: 0,
                    arg3: 0,
                    arg4: 0,
                },
            )),
        );
    }

    if wipe.stage == 1 && since >= RETURN_AFTER {
        wipe.stage = 2;
        wipe.lay_at = first_player_position(instance);
        let left = time_left(instance);
        {
            let mut network = network.lock();
            for event in [
                unnamed(WIPE_EVENT_29, 0, 0, 0),
                unnamed(WIPE_EVENT_13, 0, 0, 0),
                unnamed(WIPE_EVENT_TIME_LEFT, left, 0, 0),
            ] {
                send_director_event(&mut network, instance, event);
            }
        }

        // Everyone back to the start, alive.
        let territory = instance.zone.id;
        let clients: Vec<(ClientId, ObjectId)> = {
            let network = network.lock();
            players(instance)
                .into_iter()
                .filter_map(|(actor_id, _)| {
                    network
                        .clients
                        .iter()
                        .find(|(_, (handle, _))| handle.actor_id == actor_id)
                        .map(|(client, _)| (*client, actor_id))
                })
                .collect()
        };
        for (client, actor_id) in &clients {
            crate::server::return_to_start(
                &network,
                &game_data,
                instance,
                *client,
                *actor_id,
                PrepareZoning {
                    territory_type_id: territory,
                    warp_type: WarpType::Event,
                    hide_character: 1,
                    fade_out_delay: 0,
                    flags: PrepareZoningFlag::UNK2,
                    ..Default::default()
                },
            );
            set_character_mode(
                instance,
                &mut network.lock(),
                *actor_id,
                CharacterMode::Normal,
                0,
                true,
            );
        }

        {
            let mut network = network.lock();
            send_director_event(&mut network, instance, unnamed(WIPE_EVENT_11, 0, 0, 0));
            entrance_circle(&mut network, instance, true);
        }

        // The script's own reset (its boss taken away and brought back); what it had queued for later was the
        // attempt's.
        instance.later_director_tasks.clear();
        if let Some(director) = instance.directors.first_mut() {
            director.call_hook("onPartyWipe");
        }
    }

    let arrived = match (wipe.lay_at, first_player_position(instance)) {
        (Some(lay), Some(now_at)) => lay.distance(now_at) > ARRIVED_DISTANCE,
        _ => true,
    };
    if wipe.stage == 2 && (arrived || since >= RECOMMENCE_AFTER) {
        wipe.stage = 3;
        // The party's NPCs, standing or not, back beside the one they follow.
        let bots: Vec<(ObjectId, ObjectId)> = instance
            .party_bots
            .iter()
            .map(|(id, bot)| (*id, bot.leader))
            .collect();
        for (bot, leader) in bots {
            party_bot::get_up_beside(&network, instance, bot, leader);
        }
    }

    if wipe.stage == 3 && since >= RECOMMENCE_AFTER {
        let left = time_left(instance);
        {
            let mut network = network.lock();
            send_director_event(
                &mut network,
                instance,
                DirectorEvent::ContentDirector(ContentDirectorEvent::InstanceContent(
                    InstanceContentDirectorEvent::DutyRecommence {
                        arg1: left,
                        arg2: 0,
                        arg3: RECOMMENCE_ARG3,
                        arg4: 0,
                    },
                )),
            );
            entrance_circle(&mut network, instance, false);
        }
        if let Some(director) = instance.directors.first_mut() {
            director.call_hook("onDutyRecommence");
        }
        instance.party_wipe = None;
        return;
    }

    instance.party_wipe = Some(wipe);
}
