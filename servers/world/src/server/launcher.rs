//! Unseen launchers that rain an attack on fixed spots while the party is near them
//! (`resources/data/launchers.json`).
//!
//! Castrum Meridianum's BNpcBase 6358 (44 HP, no HOSTILE flag, never seen to move) cast 29154, a 2.7 s area on a spot,
//! every 4.9 or 5.9 s on a few fixed spots of the road while the party was near them, never twice running on the same
//! spot, in all five retail runs: four of them, one stretch of road each, taking over from each other as the party
//! went on. Kawari had them stand idle (Bob's mark of 2026-11-21: "no random AOE fall in this area like in retail").
//! Each launcher listed goes on while a party member (a player, or one of its bots) is within `near` of one of its
//! spots, and picks one of them at random, not within `NOT_AGAIN_WITHIN` of the one before. As retail's did, it's put
//! on the spot (ActorSetPos: its position updates were the spot 0.2 s before each cast) and casts on itself there;
//! the area lands on whoever its battalion is at war with (`action.rs`), as any NPC's area on itself does (retail's
//! 29154 caught the party 4 at a time on spots 20 yalms from where the launcher began.)

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use glam::Vec3A;
use parking_lot::Mutex;

use crate::server::FromServer;
use crate::server::actor::NetworkedActor;
use crate::server::instance::Instance;
use crate::server::network::{DestinationNetwork, NetworkState};
use kawari::common::{ObjectId, Position};
use kawari::ipc::zone::{ActorSetPos, ServerZoneIpcData, ServerZoneIpcSegment};

use crate::GameData;

/// A launcher's next spot is never within this of the one before (yalms): 219 of retail's 220 casts.
pub const NOT_AGAIN_WITHIN: f32 = 6.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Launcher {
    pub layout: u32,
    pub action: u32,
    pub every: Vec<f32>,
    pub near: f32,
    pub spots: Vec<Vec3A>,
}

/// Where a launcher is in its turn: when it casts next, and where it cast last.
#[derive(Debug, Clone, Copy)]
pub struct LauncherState {
    pub next: Instant,
    pub last: Option<Vec3A>,
}

/// `territory`'s launchers (`launchers.json`.)
pub fn launchers(territory: u16) -> Vec<Launcher> {
    table().get(&territory).cloned().unwrap_or_default()
}

/// Every territory's launchers, read once.
fn table() -> &'static HashMap<u16, Vec<Launcher>> {
    static TABLE: OnceLock<HashMap<u16, Vec<Launcher>>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let value: serde_json::Value =
            serde_json::from_str(include_str!("../../../../resources/data/launchers.json"))
                .unwrap_or_default();
        let number = |value: &serde_json::Value| value.as_f64().map(|v| v as f32);
        let mut table = HashMap::new();
        for (territory, list) in value
            .get("territories")
            .and_then(|t| t.as_object())
            .into_iter()
            .flatten()
        {
            let Ok(territory) = territory.parse::<u16>() else {
                continue;
            };
            let launchers = list
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|entry| {
                    Some(Launcher {
                        layout: entry.get("layout")?.as_u64()? as u32,
                        action: entry.get("action")?.as_u64()? as u32,
                        every: entry
                            .get("every")?
                            .as_array()?
                            .iter()
                            .filter_map(number)
                            .collect(),
                        near: number(entry.get("near")?)?,
                        spots: entry
                            .get("spots")?
                            .as_array()?
                            .iter()
                            .filter_map(|spot| {
                                let spot = spot.as_array()?;
                                Some(Vec3A::new(
                                    number(spot.first()?)?,
                                    number(spot.get(1)?)?,
                                    number(spot.get(2)?)?,
                                ))
                            })
                            .collect(),
                    })
                })
                .collect();
            table.insert(territory, launchers);
        }
        table
    })
}

/// Whether `actor` is one of its zone's launchers. Nobody fights one: no bot takes it on and no player's area hits it
/// (retail's ten in the Praetorium took no hit in five runs; in Bob's run of 2026-11-24 his tank bot attacked one and
/// everyone's areas caught it till it died: on the enemies' battalion, it was a foe to Kawari.)
pub fn is_launcher(instance: &Instance, actor: &NetworkedActor) -> bool {
    let NetworkedActor::Npc { spawn, .. } = actor else {
        return false;
    };
    spawn.common.layout_id != 0
        && table().get(&instance.zone.id).is_some_and(|launchers| {
            launchers
                .iter()
                .any(|launcher| launcher.layout == spawn.common.layout_id)
        })
}

fn across_the_ground(a: Vec3A, b: Vec3A) -> f32 {
    Vec3A::new(a.x - b.x, 0.0, a.z - b.z).length()
}

impl Launcher {
    /// Whether one of `party` is within `near` of one of its spots.
    pub fn goes_on(&self, party: &[Vec3A]) -> bool {
        party.iter().any(|member| {
            self.spots
                .iter()
                .any(|spot| across_the_ground(*member, *spot) <= self.near)
        })
    }

    /// Whether it casts: its party is near and the director hasn't put it off (`director:launcher`): the
    /// Praetorium's cast only while their stretch was fought, though the party was near before and after.
    pub fn casts(&self, off: &std::collections::HashSet<u32>, party: &[Vec3A]) -> bool {
        !off.contains(&self.layout) && self.goes_on(party)
    }

    /// Its next spot, `roll` from 0 to 1: one of its spots not within `NOT_AGAIN_WITHIN` of `last`.
    pub fn next_spot(&self, last: Option<Vec3A>, roll: f32) -> Option<Vec3A> {
        let choices: Vec<Vec3A> = self
            .spots
            .iter()
            .copied()
            .filter(|spot| {
                last.is_none_or(|last| across_the_ground(*spot, last) > NOT_AGAIN_WITHIN)
            })
            .collect();
        let choices = if choices.is_empty() {
            self.spots.clone()
        } else {
            choices
        };
        let index = ((roll * choices.len() as f32) as usize).min(choices.len().checked_sub(1)?);
        choices.get(index).copied()
    }

    /// The time to its next cast, `roll` from 0 to 1: one of its `every` (5.4 s if none.)
    pub fn wait(&self, roll: f32) -> Duration {
        let seconds = if self.every.is_empty() {
            5.4
        } else {
            self.every[((roll * self.every.len() as f32) as usize).min(self.every.len() - 1)]
        };
        Duration::from_secs_f32(seconds)
    }
}

/// Each of the instance's launchers casts when its time has come and its party is near.
pub fn tick(
    network: &Arc<Mutex<NetworkState>>,
    game_data: &Arc<Mutex<GameData>>,
    instance: &mut Instance,
) {
    if instance.awaiting_commence {
        return;
    }
    let launchers = launchers(instance.zone.id);
    if launchers.is_empty() {
        return;
    }
    let now = Instant::now();
    let party: Vec<Vec3A> = instance
        .actors
        .iter()
        .filter(|(id, actor)| {
            (matches!(actor, NetworkedActor::Player { .. }) || instance.party_bots.contains_key(id))
                && actor.get_common_spawn().health_points > 0
        })
        .map(|(_, actor)| actor.position().0)
        .collect();
    for launcher in launchers {
        let Some(actor_id) = instance.actors.iter().find_map(|(id, actor)| match actor {
            NetworkedActor::Npc { spawn, .. }
                if spawn.common.layout_id == launcher.layout && spawn.common.health_points > 0 =>
            {
                Some(*id)
            }
            _ => None,
        }) else {
            continue;
        };
        if !launcher.casts(&instance.launchers_off, &party) {
            // Away from its road it rests; it begins one of its waits after someone comes (retail's first came within
            // 3 s of the player's coming.)
            instance.launchers.remove(&launcher.layout);
            continue;
        }
        let state = *instance
            .launchers
            .entry(launcher.layout)
            .or_insert_with(|| LauncherState {
                next: now + launcher.wait(fastrand::f32()) / 2,
                last: None,
            });
        if now < state.next {
            continue;
        }
        let Some(spot) = launcher.next_spot(state.last, fastrand::f32()) else {
            continue;
        };
        instance.launchers.insert(
            launcher.layout,
            LauncherState {
                next: now + launcher.wait(fastrand::f32()),
                last: Some(spot),
            },
        );
        // Retail's launcher was at the spot as it cast there (its position updates: the spot, 0.2 s before each
        // ActorCast) and cast on itself: put there, then cast.
        let rotation = if let Some(NetworkedActor::Npc {
            spawn,
            navmesh_path,
            navmesh_path_lerp,
            last_position,
            ..
        }) = instance.find_actor_mut(actor_id)
        {
            spawn.common.position = Position(spot);
            navmesh_path.clear();
            *navmesh_path_lerp = 0.0;
            *last_position = Some(spot);
            spawn.common.rotation
        } else {
            continue;
        };
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorSetPos(ActorSetPos {
            position: Position(spot),
            rotation,
            ..Default::default()
        }));
        network.lock().send_to_instance(
            ObjectId::default(),
            instance,
            FromServer::PacketSegment(ipc, actor_id),
            DestinationNetwork::ZoneClients,
        );
        crate::server::npc_behavior::npc_use_action(
            network,
            game_data,
            instance,
            actor_id,
            launcher.action,
            actor_id,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn castrum_meridianums_launchers_rain_on_their_spots_while_the_party_is_near() {
        let launchers = launchers(1043);
        assert_eq!(
            launchers.iter().map(|l| l.layout).collect::<Vec<_>>(),
            [9089359, 9089363, 9089367, 9089374]
        );
        let first = &launchers[0];
        assert_eq!(first.action, 29154);
        // Bob's mark, on its road: near its spots (-40.7, 66.85, 215.87) and the like.
        let mark = Vec3A::new(-30.59, 64.90, 235.51);
        assert!(!first.goes_on(&[Vec3A::new(-11.0, 56.4, 205.9)]));
        assert!(first.goes_on(&[Vec3A::new(-45.7, 67.4, 221.5)]));
        assert!(
            first.goes_on(&[mark])
                == first
                    .spots
                    .iter()
                    .any(|s| across_the_ground(mark, *s) <= 12.0)
        );
        // Never twice running on the same spot.
        let last = Vec3A::new(-40.7, 66.85, 215.87);
        for i in 0..50 {
            let spot = first.next_spot(Some(last), i as f32 / 50.0).unwrap();
            assert!(across_the_ground(spot, last) > NOT_AGAIN_WITHIN, "{spot:?}");
            assert!(first.spots.contains(&spot));
        }
        // 4.9 or 5.9 s between casts.
        assert_eq!(first.wait(0.1), Duration::from_secs_f32(4.9));
        assert_eq!(first.wait(0.9), Duration::from_secs_f32(5.9));
    }

    /// Bob's run of 2026-11-24 in the Praetorium and its five retail runs: the ten launchers (BNpcBase 6358) cast 29049
    /// on their own spot every 9.5 to 13.2 s, only while their stretch was fought; the director puts them on and off.
    #[test]
    fn the_praetoriums_launchers_cast_only_while_the_director_has_them_on() {
        let launchers = launchers(1044);
        assert_eq!(launchers.len(), 10);
        assert!(
            launchers
                .iter()
                .all(|l| l.action == 29049 && l.spots.len() == 1)
        );
        let first = &launchers[0];
        let near = [first.spots[0] + Vec3A::new(10.0, 0.0, 0.0)];
        let mut off = std::collections::HashSet::new();
        assert!(first.casts(&off, &near));
        off.insert(first.layout);
        assert!(!first.casts(&off, &near));
        off.remove(&first.layout);
        assert!(!first.casts(&off, &[first.spots[0] + Vec3A::new(80.0, 0.0, 0.0)]));
        assert!(first.wait(0.0) >= Duration::from_secs_f32(9.5));
        assert!(first.wait(0.99) <= Duration::from_secs_f32(13.2));
    }
}
