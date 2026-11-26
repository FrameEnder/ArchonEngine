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
//!
//! A sweep can go by several points in one run (`via`, `director:sweep_bnpc_path`): the Praetorium's Cid ran to the
//! terminal by seven of them at 6 yalms a second with an ActorMove every 0.31 s and no stop at any (five retail runs),
//! where one sweep a point stopped him at each till the next began (Bob's mark of 2026-11-24 on `f3c32e2`: "he stops
//! on each one"). Each tick's ActorMove is where it will be a tick on along the whole way, round a corner if one comes.

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
    /// The points it goes by on its way from `from` to `to`, in order (none: straight.)
    pub via: Vec<Vec3A>,
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
    /// It goes with its walk, whatever its speed (`movement_type`.)
    pub walking: bool,
}

impl Sweep {
    /// Its ActorMove's movement type: 2 (walking) for one that walks, as QB86's darting bombs (BNpcBase 1633) did at 9
    /// yalms a second (2,082 of their 2,139 ActorMoves in Bob's Chronofoil captures, the rest as they stopped), where
    /// the creeper (1634) ran at 1 (466 of 466); otherwise 0, a run.
    pub fn movement_type(&self) -> kawari::common::MoveAnimationType {
        if self.walking {
            kawari::common::MoveAnimationType::WALKING_OR_LANDING
        } else {
            kawari::common::MoveAnimationType::empty()
        }
    }

    /// Its way's legs, `from` by each of `via` to `to`.
    fn legs(&self) -> impl Iterator<Item = (Vec3A, Vec3A)> + '_ {
        let points: Vec<Vec3A> = std::iter::once(self.from)
            .chain(self.via.iter().copied())
            .chain(std::iter::once(self.to))
            .collect();
        (0..points.len() - 1).map(move |i| (points[i], points[i + 1]))
    }

    fn across(a: Vec3A, b: Vec3A) -> f32 {
        Vec2::new(b.x - a.x, b.z - a.z).length()
    }

    /// The leg it's on after going `gone` yalms, and how far along it.
    fn leg_at(&self, gone: f32) -> (Vec3A, Vec3A, f32) {
        let mut left = gone;
        let mut last = (self.from, self.to);
        for (a, b) in self.legs() {
            let length = Self::across(a, b);
            if left < length {
                return (a, b, left);
            }
            left -= length;
            last = (a, b);
        }
        (last.0, last.1, Self::across(last.0, last.1))
    }

    fn gone(&self, now: Instant) -> f32 {
        if now <= self.sets_off {
            0.0
        } else {
            now.duration_since(self.sets_off).as_secs_f32() * self.speed
        }
    }

    fn direction_of(a: Vec3A, b: Vec3A) -> Vec3A {
        Vec3A::new(b.x - a.x, 0.0, b.z - a.z)
            .try_normalize()
            .unwrap_or(Vec3A::Z)
    }

    /// The way it's going at `now` (its leg's.)
    fn direction_at(&self, now: Instant) -> Vec3A {
        let (a, b, _) = self.leg_at(self.gone(now));
        Self::direction_of(a, b)
    }

    fn direction(&self) -> Vec3A {
        // A straight sweep's one way; a path's, its last leg's.
        let (a, b) = self.legs().last().unwrap_or((self.from, self.to));
        Self::direction_of(a, b)
    }

    fn length(&self) -> f32 {
        self.legs().map(|(a, b)| Self::across(a, b)).sum()
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
        let gone = self.gone(now);
        if gone >= self.length() {
            return self.to;
        }
        let (a, b, along) = self.leg_at(gone);
        let length = Self::across(a, b);
        if length <= 0.0 {
            return b;
        }
        a + (b - a) * (along / length)
    }

    /// The way it faces as it goes (the game's rotation: 0 is +z): a straight sweep's one way.
    pub fn rotation(&self) -> f32 {
        let direction = self.direction();
        f32::atan2(direction.x, direction.z)
    }

    /// The way it faces at `now`, on whichever leg it's on.
    pub fn rotation_at(&self, now: Instant) -> f32 {
        let direction = self.direction_at(now);
        f32::atan2(direction.x, direction.z)
    }

    /// The way from `at` to where it will be at `then`: a path's ActorMove faces where it's going next, round a corner.
    fn facing_towards(&self, at: Vec3A, then: Instant, now: Instant) -> f32 {
        let ahead = self.position(then);
        let way = Vec3A::new(ahead.x - at.x, 0.0, ahead.z - at.z);
        if way.length() < 0.01 {
            return self.rotation_at(now);
        }
        f32::atan2(way.x, way.z)
    }

    /// The way ahead of it while it moves and hurts: a line from just behind it (its action's reach) to its end,
    /// as wide as its reach each side, gone at its arrival. None before it sets off or casts, and after it arrives.
    pub fn danger(&self, now: Instant) -> Option<Danger> {
        if self.action == 0 || now < self.sets_off || now < self.casts_from || now >= self.arrives()
        {
            return None;
        }
        // The rest of the leg it's on.
        let at = self.position(now);
        let (_, end, _) = self.leg_at(self.gone(now));
        let left = Vec2::new(end.x - at.x, end.z - at.z).length();
        Some(Danger {
            action_id: self.action,
            shape: AreaShape::Line {
                length: left + self.radius * 2.0,
                width: self.radius * 2.0,
            },
            caster_id: self.helper.unwrap_or(self.walker),
            caster: at - self.direction_at(now) * self.radius,
            rotation: self.rotation_at(now),
            aimed_at: end,
            follows: None,
            caster_radius: 0.0,
            until: self.arrives(),
            stack: false,
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
        // Gone, or fallen on the way (a dead walker isn't carried on: one of quest battle 61's adds killed
        // on its way down from the ledge, `Battle:walk`.)
        if instance.find_actor(sweep.walker).is_none_or(|walker| {
            walker.get_common_spawn().mode == kawari::common::CharacterMode::Dead
        }) {
            return false;
        }
        let arrives = sweep.arrives();
        let at = sweep.position(now);

        // On its way: an ActorMove each tick to where it will be by the next, at its speed, as retail's every 0.31 s
        // and as NPCs walk (one ActorMove to the end of a long line snapped it there on the client: Bob's bundle
        // 20261107-212704, the Stone Vigil's first boss's run to its Swinge spot.) The tick that brings it to its end
        // has sent it there already; one more ActorMove, facing the way it ran, turned Chudo-Yudo back to the wall
        // after it had turned into the room for its Swinge, so the cone the client and its plugins drew pointed away
        // from the one that hit (Bob's bundle 20261110-134659: ActorCast at 1.41, then an ActorMove at -1.79.)
        let arrived = now >= arrives;
        if now >= sweep.sets_off && !arrived {
            sweep.moving = true;
            let ahead = sweep.position(now + TICK);
            let rotation = if sweep.via.is_empty() {
                sweep.rotation()
            } else {
                sweep.facing_towards(at, now + TICK, now)
            };
            network.lock().send_in_range_instance(
                sweep.walker,
                instance,
                FromServer::ActorMove(
                    sweep.walker,
                    Position(ahead),
                    rotation,
                    sweep.movement_type(),
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
            if sweep.moving && !arrived {
                spawn.common.rotation = sweep.rotation_at(now);
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
            let rotation = sweep.rotation_at(now);
            if let Some(helper) = sweep.helper {
                if let Some(NetworkedActor::Npc { spawn, .. }) = instance.find_actor_mut(helper) {
                    spawn.common.position = Position(at);
                }
                let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorSetPos(ActorSetPos {
                    position: Position(at),
                    rotation,
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
        // Kept a tick past its arrival, to put it at its end on the server.
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
            via: Vec::new(),
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
            walking: false,
        }
    }

    /// Known issues, quest battle 86: the darting bombs went with movement type 2 at 9 yalms a second on retail, where
    /// Kawari's sweeps always ran (0).
    #[test]
    fn a_walking_sweep_walks() {
        let now = Instant::now();
        let mut bomb = whirlwind(now, 0.0, 0.0);
        assert_eq!(
            bomb.movement_type(),
            kawari::common::MoveAnimationType::empty()
        );
        bomb.walking = true;
        assert_eq!(
            bomb.movement_type(),
            kawari::common::MoveAnimationType::WALKING_OR_LANDING
        );
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

    /// Bob's mark of 2026-11-24 in the Praetorium: "the points cid walks along make his movement a tad stiff as he
    /// stops on each one". Retail's Cid ran his seven points to the terminal at 6 yalms a second without a stop; a path
    /// sweep goes on round each, its ActorMove each tick where it will be a tick on, and reaches the end when the
    /// whole way's length at its speed says.
    #[test]
    fn a_path_goes_on_round_its_points_without_stopping() {
        let now = Instant::now();
        let mut cid = whirlwind(now, 0.0, 0.0);
        cid.action = 0;
        cid.helper = None;
        cid.speed = 6.0;
        cid.from = Vec3A::new(0.0, 0.0, 0.0);
        cid.via = vec![Vec3A::new(6.0, 0.0, 0.0)];
        cid.to = Vec3A::new(6.0, 0.0, 6.0);
        // Twelve yalms at six a second.
        assert!(((cid.arrives() - now).as_secs_f32() - 2.0).abs() < 0.01);
        let at = |s: f32| cid.position(now + Duration::from_secs_f32(s));
        assert!((at(0.5) - Vec3A::new(3.0, 0.0, 0.0)).length() < 0.01);
        assert!((at(1.5) - Vec3A::new(6.0, 0.0, 3.0)).length() < 0.01);
        // Facing +x on the first leg, +z on the second.
        let first = cid.rotation_at(now + Duration::from_secs_f32(0.5));
        assert!((first - std::f32::consts::FRAC_PI_2).abs() < 0.01);
        assert!(cid.rotation_at(now + Duration::from_secs_f32(1.5)).abs() < 0.01);
        // 0.75 s in, a tick before the corner, its ActorMove goes on past it, so it never stops there.
        let t = now + Duration::from_secs_f32(0.75);
        let ahead = cid.position(t + TICK);
        assert!(ahead.z > 0.5 && (ahead.x - 6.0).abs() < 0.01, "{ahead:?}");
        let facing = cid.facing_towards(at(0.75), t + TICK, t);
        assert!(facing > 0.0 && facing < std::f32::consts::FRAC_PI_2);
        assert_eq!(at(3.0), cid.to);
    }
}
