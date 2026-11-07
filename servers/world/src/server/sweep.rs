//! NPCs a director sends along a line at a set speed, hurting whoever is in their way as they go
//! (`director:sweep_bnpc`): Koshchei's whirlwinds in the Stone Vigil.
//!
//! Retail (Bob's Chronofoil capture 1cefedf4 of 2026-11-07, five Typhoons): each whirlwind (BNpcBase 14536, no actions)
//! went straight down its lane at 3.0 yalms a second (ActorMove every 0.31 s, its speed byte 30), and a helper (14544)
//! rode with it: an ActorSetPos to the whirlwind's spot and its 28731 there (instant, a 3-yalm circle) every 0.58 s,
//! from when the first of them set off until its own whirlwind reached the lane's end. Kawari moves the walker the same
//! way (an ActorMove each tick to where it will be by the next, at the speed, and its position along the line on the
//! server), puts the helper where the walker is at each of its casts and has it use the action there; and the way
//! ahead of each moving walker is a danger the party gets out of (`dodge::dangers`.)

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use glam::{Vec2, Vec3A};
use parking_lot::Mutex;

use kawari::{
    common::{ObjectId, Position},
    ipc::zone::{ActorSetPos, ServerZoneIpcData, ServerZoneIpcSegment},
};

use crate::{
    FromServer, GameData,
    gamedata::AreaShape,
    server::{
        actor::NetworkedActor,
        dodge::Danger,
        instance::Instance,
        network::{DestinationNetwork, NetworkState},
    },
};

/// The director's tick (`director_tick`, every 500 ms).
const TICK: Duration = Duration::from_millis(500);

/// One NPC going along a line.
#[derive(Debug, Clone)]
pub struct Sweep {
    /// What moves.
    pub walker: ObjectId,
    pub from: Vec3A,
    pub to: Vec3A,
    /// Yalms a second.
    pub speed: f32,
    pub sets_off: Instant,
    /// What hurts as it goes: a helper (or the walker itself) using `action` where the walker is, every `every`, from
    /// `casts_from` until the walker arrives.
    pub helper: Option<ObjectId>,
    pub action: u32,
    pub every: Duration,
    pub casts_from: Instant,
    pub next_cast: Instant,
    /// The action's reach around the walker (its EffectRange.)
    pub radius: f32,
    /// Whether the walker's ActorMove has gone out.
    pub moving: bool,
}

impl Sweep {
    fn direction(&self) -> Vec3A {
        let flat = Vec3A::new(self.to.x - self.from.x, 0.0, self.to.z - self.from.z);
        flat.try_normalize().unwrap_or(Vec3A::Z)
    }

    fn length(&self) -> f32 {
        Vec2::new(self.to.x - self.from.x, self.to.z - self.from.z).length()
    }

    /// When it reaches the end.
    pub fn arrives(&self) -> Instant {
        self.sets_off + Duration::from_secs_f32(self.length() / self.speed.max(0.1))
    }

    /// Where the walker is at `now`.
    pub fn position(&self, now: Instant) -> Vec3A {
        if now <= self.sets_off {
            return self.from;
        }
        let gone = now.duration_since(self.sets_off).as_secs_f32() * self.speed;
        if gone >= self.length() {
            return self.to;
        }
        self.from + (self.to - self.from) * (gone / self.length())
    }

    /// The way it faces as it goes (the game's rotation: 0 is +z.)
    pub fn rotation(&self) -> f32 {
        let direction = self.direction();
        f32::atan2(direction.x, direction.z)
    }

    /// The way ahead of it while it moves and hurts: a line from just behind it (its action's reach) to its end,
    /// as wide as its reach each side, gone at its arrival. None before it sets off or casts, and after it arrives.
    pub fn danger(&self, now: Instant) -> Option<Danger> {
        if self.action == 0 || now < self.sets_off || now < self.casts_from || now >= self.arrives()
        {
            return None;
        }
        let at = self.position(now);
        let left = Vec2::new(self.to.x - at.x, self.to.z - at.z).length();
        Some(Danger {
            action_id: self.action,
            shape: AreaShape::Line {
                length: left + self.radius * 2.0,
                width: self.radius * 2.0,
            },
            caster_id: self.helper.unwrap_or(self.walker),
            caster: at - self.direction() * self.radius,
            rotation: self.rotation(),
            aimed_at: self.to,
            caster_radius: 0.0,
            until: self.arrives(),
        })
    }
}

/// The ActorMove speed byte for `speed` yalms a second (retail's whirlwinds: 30 for 3.0; its walk 20, its run 60.)
pub fn speed_byte(speed: f32) -> u8 {
    (speed * 10.0).round().clamp(1.0, 255.0) as u8
}

/// Moves the director's sweeping NPCs on and has them hurt where they are, as their times come; each sweep ends as its
/// walker arrives (or is gone.) Called every director tick.
pub fn update(
    network: &Arc<Mutex<NetworkState>>,
    game_data: &Arc<Mutex<GameData>>,
    instance: &mut Instance,
    now: Instant,
) {
    if instance.sweeps.is_empty() {
        return;
    }
    let mut sweeps = std::mem::take(&mut instance.sweeps);
    sweeps.retain_mut(|sweep| {
        if instance.find_actor(sweep.walker).is_none() {
            return false;
        }
        let arrives = sweep.arrives();
        let at = sweep.position(now);

        // On its way: an ActorMove each tick to where it will be by the next, at its speed, as retail's every 0.31 s
        // and as NPCs walk (one ActorMove to the end of a long line snapped it there on the client: Bob's bundle
        // 20261107-212704, the Stone Vigil's first boss's run to its Swinge spot.)
        if now >= sweep.sets_off {
            sweep.moving = true;
            let ahead = sweep.position(now + TICK);
            network.lock().send_in_range_instance(
                sweep.walker,
                instance,
                FromServer::ActorMove(
                    sweep.walker,
                    Position(ahead),
                    sweep.rotation(),
                    kawari::common::MoveAnimationType::empty(),
                    kawari::common::MoveAnimationState::None,
                    kawari::common::JumpState::NoneOrFalling,
                    Some(speed_byte(sweep.speed)),
                ),
                DestinationNetwork::ZoneClients,
            );
        }
        if let Some(NetworkedActor::Npc {
            spawn,
            navmesh_path,
            navmesh_path_lerp,
            last_position,
            ..
        }) = instance.find_actor_mut(sweep.walker)
        {
            spawn.common.position = Position(at);
            if sweep.moving {
                spawn.common.rotation = sweep.rotation();
            }
            navmesh_path.clear();
            *navmesh_path_lerp = 0.0;
            *last_position = Some(at);
        }

        // Its hurt, where it is (the helper put there first, as retail's ActorSetPos.)
        if sweep.action != 0 && now >= sweep.casts_from && now < arrives {
            while sweep.next_cast <= now {
                sweep.next_cast += sweep.every.max(Duration::from_millis(100));
            }
            let caster = sweep.helper.unwrap_or(sweep.walker);
            if let Some(helper) = sweep.helper {
                if let Some(NetworkedActor::Npc { spawn, .. }) = instance.find_actor_mut(helper) {
                    spawn.common.position = Position(at);
                }
                let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorSetPos(ActorSetPos {
                    position: Position(at),
                    rotation: sweep.rotation(),
                    ..Default::default()
                }));
                network.lock().send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::PacketSegment(ipc, helper),
                    DestinationNetwork::ZoneClients,
                );
            }
            if instance.find_actor(caster).is_some() {
                crate::server::npc_behavior::npc_use_action(
                    network,
                    game_data,
                    instance,
                    caster,
                    sweep.action,
                    caster,
                );
            }
        }
        // Kept a tick past its arrival, for the last ActorMove to its end.
        now < arrives + TICK
    });
    // Any the director added meanwhile (an action's script) stay too.
    sweeps.append(&mut instance.sweeps);
    instance.sweeps = sweeps;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn whirlwind(now: Instant, lane: f32, after: f32) -> Sweep {
        Sweep {
            walker: ObjectId(1),
            from: Vec3A::new(57.0, 4.5, lane),
            to: Vec3A::new(30.4, 4.0, lane),
            speed: 3.0,
            sets_off: now + Duration::from_secs_f32(after),
            helper: Some(ObjectId(2)),
            action: 28731,
            every: Duration::from_millis(580),
            casts_from: now + Duration::from_secs_f32(3.9),
            next_cast: now + Duration::from_secs_f32(3.9),
            radius: 3.0,
            moving: false,
        }
    }

    /// Retail's first Typhoon (Bob's capture 1cefedf4, 1146.5 s): the whirlwinds in lanes -72 and -82.5 set off 3.9 s
    /// after they came and took 8.9 s to the lane's end; the one in -88 set off 7 s after them.
    #[test]
    fn along_the_lane() {
        let now = Instant::now();
        let first = whirlwind(now, -82.5, 3.9);
        assert_eq!(first.position(now + Duration::from_secs(2)).x, 57.0);
        let halfway = first.position(now + Duration::from_secs_f32(3.9 + 4.43));
        assert!((halfway.x - 43.7).abs() < 0.1, "{halfway:?}");
        assert!(((first.arrives() - now).as_secs_f32() - 12.77).abs() < 0.05);
        assert_eq!(first.position(now + Duration::from_secs(20)).x, 30.4);
        assert_eq!(speed_byte(3.0), 30);
        // The way ahead is a danger while it goes; someone just behind it or in another lane isn't in it.
        let t = now + Duration::from_secs_f32(3.9 + 4.43);
        let danger = first.danger(t).unwrap();
        assert!(danger.catches(Vec3A::new(40.0, 4.0, -82.5), 0.0));
        assert!(danger.catches(Vec3A::new(40.0, 4.0, -80.0), 0.0));
        assert!(!danger.catches(Vec3A::new(48.0, 4.0, -82.5), 0.0));
        assert!(!danger.catches(Vec3A::new(40.0, 4.0, -77.0), 0.0));
        // The one yet to set off is no danger until it does (retail's party stood in its lane meanwhile.)
        let last = whirlwind(now, -88.0, 10.9);
        assert!(last.danger(now + Duration::from_secs(8)).is_none());
        assert!(last.danger(now + Duration::from_secs(12)).is_some());
        assert!(last.danger(now + Duration::from_secs(25)).is_none());
    }
}
