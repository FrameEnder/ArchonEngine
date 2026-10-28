//! Party bots and party NPCs getting out of enemies' area attacks (`party_bot::update_party` moves them.)
//!
//! Retail's Duty Support members, in Bob's Chronofoil capture of the Tam-Tara Deepcroft (2026-10-24, 3ec702a8, visit
//! 0.3, `questkit chronofoil --battle`): both times the last boss began its 4.7 s attack around itself (987), all
//! three set off 1.0-1.4 s into the cast (the first samples showing them moving), ran straight out at run speed (about
//! 1.9 yalms each 0.31 s sample), stopped just outside it (11.0 yalms from its middle, all three), waited there until
//! it had gone off, and walked back in 1.3-1.5 s after it had (6.1-6.2 s into the cast). Single-target casts and its
//! self-buff (430) moved nobody.

use std::time::{Duration, Instant};

use glam::{Vec2, Vec3A};

use kawari::{common::ObjectId, ipc::zone::CharacterDataFlag};

use crate::{
    GameData,
    gamedata::AreaShape,
    server::{actor::NetworkedActor, instance::Instance},
};

/// How far past an area attack's hit edge a dodger stops (retail's stood at 11.0 yalms from a circle they otherwise
/// left as soon as they could; the hit edge's own grazing allowance is `PLAYER_HITBOX_RADIUS`, in `in_area`.)
pub const DODGE_MARGIN: f32 = 0.5;
/// When a dodger sets off after it notices an area attack starting: a moment, a little quicker than retail's (1.0-1.4 s
/// into the cast in the capture, sampled every 0.31 s; Bob found the bots slow to react, 2026-10-26), or at once when
/// waiting would leave too little time to get out.
pub const REACTION_SECONDS: (f32, f32) = (0.5, 0.9);
/// How long after the attack has gone off a dodger stays out before it goes back to fighting (retail: 1.3-1.5 s.)
pub const LINGER: Duration = Duration::from_millis(1300);
/// How far a dodger looks for a safe spot.
const SEARCH_RADIUS: f32 = 40.0;
const SEARCH_STEP: f32 = 0.5;
const SEARCH_DIRECTIONS: usize = 24;
/// Among the nearest safe spots (within this much of the very nearest), the one closest to where it would rather be.
const NEAR_ENOUGH: f32 = 1.0;

/// An enemy's area attack being cast: where it will land, and when.
#[derive(Debug, Clone, Copy)]
pub struct Danger {
    pub shape: AreaShape,
    /// Who's casting it (what Reprisal, Feint and Addle go on when there's no getting out.)
    pub caster_id: ObjectId,
    pub caster: Vec3A,
    /// Which way the caster faced as it started.
    pub rotation: f32,
    /// Where it was aimed (its target's position then.)
    pub aimed_at: Vec3A,
    pub caster_radius: f32,
    /// When it goes off.
    pub until: Instant,
}

impl Danger {
    /// Whether it would catch someone at `point`, counting `margin` yalms past its hit edge.
    pub fn catches(&self, point: Vec3A, margin: f32) -> bool {
        super::action::in_area_with_reach(
            self.shape,
            self.caster,
            self.rotation,
            self.aimed_at,
            point,
            self.caster_radius,
            super::action::PLAYER_HITBOX_RADIUS + margin,
        )
    }

    /// The nearest points out of it from `from` (a little past `margin`), as the shape gives them: straight out of a
    /// circle, out of a cone's far end or off its nearer side, off a line's nearer side or past its end.
    fn exits(&self, from: Vec3A, margin: f32) -> Vec<Vec3A> {
        let flat = |v: Vec3A| Vec2::new(v.x, v.z);
        let lift = |v: Vec2| Vec3A::new(v.x, from.y, v.y);
        let reach = super::action::PLAYER_HITBOX_RADIUS + margin + 0.05;
        let here = flat(from);
        let caster = flat(self.caster);
        let forward = Vec2::new(self.rotation.sin(), self.rotation.cos());
        let away = |centre: Vec2, distance: f32| {
            let out = (here - centre).try_normalize().unwrap_or(-forward);
            centre + out * distance
        };
        match self.shape {
            AreaShape::Circle { radius, on_target } => {
                if on_target {
                    vec![lift(away(flat(self.aimed_at), radius + reach))]
                } else {
                    vec![lift(away(caster, radius + self.caster_radius + reach))]
                }
            }
            AreaShape::Cone { radius, angle } => {
                let mut exits = vec![lift(away(caster, radius + self.caster_radius + reach))];
                // Off either side: rotate the offset to just past the cone's edge, keeping its distance.
                let offset = here - caster;
                let distance = offset.length().max(self.caster_radius + reach);
                let half = angle.to_radians() / 2.0;
                for side in [-1.0f32, 1.0] {
                    let edge = half + (reach / distance).atan() + 0.05;
                    let direction = Vec2::from_angle(side * edge).rotate(forward);
                    exits.push(lift(caster + direction * distance));
                }
                exits
            }
            AreaShape::Line { length, width } => {
                let offset = here - caster;
                let along = offset.dot(forward);
                let across = offset.perp_dot(forward);
                let side = Vec2::new(forward.y, -forward.x); // perp_dot's positive side
                let mut exits = Vec::new();
                for sign in [-1.0f32, 1.0] {
                    exits.push(lift(
                        caster + forward * along + side * sign * (width / 2.0 + reach),
                    ));
                }
                // Past its far end, or behind the caster.
                exits.push(lift(
                    caster + forward * (length + self.caster_radius + reach) + side * across,
                ));
                exits.push(lift(caster - forward * reach + side * across));
                exits
            }
            AreaShape::Donut {
                radius,
                hole,
                on_target,
            } => {
                // Into the hole (where it's safe), or out past the ring.
                let (centre, outer) = if on_target {
                    (flat(self.aimed_at), radius)
                } else {
                    (caster, radius + self.caster_radius)
                };
                let mut exits = vec![lift(away(centre, outer + reach))];
                let inside = outer * hole - reach;
                if inside > 0.0 {
                    exits.push(lift(away(centre, inside * 0.5)));
                    exits.push(lift(centre));
                }
                exits
            }
            AreaShape::Cross { length, width } => {
                // Off the nearer arm's side, into a corner between the arms.
                let offset = here - caster;
                let along = offset.dot(forward);
                let across = offset.perp_dot(forward);
                let side = Vec2::new(forward.y, -forward.x);
                let out = width / 2.0 + reach;
                let arm = length + self.caster_radius + reach;
                let corner = |a: f32, b: f32| caster + forward * a + side * b;
                let sign = |v: f32| if v < 0.0 { -1.0 } else { 1.0 };
                vec![
                    lift(corner(
                        along.abs().max(out) * sign(along),
                        out * sign(across),
                    )),
                    lift(corner(
                        out * sign(along),
                        across.abs().max(out) * sign(across),
                    )),
                    lift(corner(arm * sign(along), across)),
                    lift(corner(along, arm * sign(across))),
                ]
            }
        }
    }
}

/// Every area attack a hostile NPC is casting now.
pub fn dangers(instance: &Instance, game_data: &mut GameData, now: Instant) -> Vec<Danger> {
    instance
        .actors
        .iter()
        .filter_map(|(id, actor)| match actor {
            NetworkedActor::Npc {
                spawn,
                cast: Some(cast),
                timeline,
                ..
            } if spawn.common.health_points > 0
                // An enemy, or a director's helper (held, not hostile: Ifrit's, which cast his Eruption and Radiant
                // Plumes; the bots stood in all of them, Bob's bundles of 2026-10-28.)
                && (spawn
                    .character_data_flags
                    .contains(CharacterDataFlag::HOSTILE)
                    || instance.held_npcs.contains(id))
                && cast.until > now
                && cast.action_id != 0 =>
            {
                let shape = game_data.get_action_info(cast.action_id)?.area_shape()?;
                Some(Danger {
                    shape,
                    caster_id: *id,
                    caster: spawn.common.position.0,
                    rotation: cast.rotation,
                    aimed_at: cast.target_position,
                    caster_radius: timeline.hitbox_radius,
                    until: cast.until,
                })
            }
            _ => None,
        })
        .collect()
}

/// A party bot getting out of area attacks: where to (None when nowhere safe was found: it stays), when it sets off,
/// and when the last attack it's avoiding goes off.
#[derive(Debug, Clone, Copy)]
pub struct Dodge {
    pub to: Option<Vec3A>,
    pub set_off: Instant,
    pub until: Instant,
    /// Out of it and waiting: it stays put, but still acts (a caster or healer in reach keeps going.)
    pub holding: bool,
}

/// Where someone at `from` should go to be out of every one of `dangers`: the nearest such spot, walkable, and among
/// spots about as near, the one closest to `rather` (their place in the fight.) None if they're already out of all of
/// them, or nowhere within `SEARCH_RADIUS` is.
#[cfg(test)]
pub fn safe_spot(
    from: Vec3A,
    rather: Vec3A,
    dangers: &[Danger],
    walkable: impl Fn(Vec3A) -> bool,
) -> Option<Vec3A> {
    safe_spots(from, rather, dangers, walkable)
        .into_iter()
        .next()
}

/// The spots out of every one of `dangers` that `walkable` accepts, best first: the nearest ones (within `NEAR_ENOUGH`
/// of the very nearest) by how close they are to `rather`, then the rest by distance. Empty if `from` is already safe
/// or there's nowhere within `SEARCH_RADIUS`. The caller checks each can be reached (`party_bot`: a navmesh path to it.)
pub fn safe_spots(
    from: Vec3A,
    rather: Vec3A,
    dangers: &[Danger],
    walkable: impl Fn(Vec3A) -> bool,
) -> Vec<Vec3A> {
    let safe = |point: Vec3A| {
        dangers
            .iter()
            .all(|danger| !danger.catches(point, DODGE_MARGIN))
    };
    if safe(from) {
        return Vec::new();
    }
    let flat_distance = |a: Vec3A, b: Vec3A| Vec2::new(a.x - b.x, a.z - b.z).length();

    // The shapes' own exits, then rings around it, nearest first.
    let mut candidates: Vec<Vec3A> = dangers
        .iter()
        .filter(|danger| danger.catches(from, DODGE_MARGIN))
        .flat_map(|danger| danger.exits(from, DODGE_MARGIN))
        .collect();
    let mut ring = SEARCH_STEP;
    while ring <= SEARCH_RADIUS {
        for i in 0..SEARCH_DIRECTIONS {
            let angle = i as f32 / SEARCH_DIRECTIONS as f32 * std::f32::consts::TAU;
            candidates.push(from + Vec3A::new(angle.sin() * ring, 0.0, angle.cos() * ring));
        }
        ring += SEARCH_STEP;
    }

    let mut good: Vec<Vec3A> = candidates
        .into_iter()
        .filter(|point| safe(*point) && walkable(*point))
        .collect();
    good.sort_by(|a, b| flat_distance(from, *a).total_cmp(&flat_distance(from, *b)));
    let Some(nearest) = good.first().map(|point| flat_distance(from, *point)) else {
        return good;
    };
    let split = good
        .iter()
        .position(|point| flat_distance(from, *point) > nearest + NEAR_ENOUGH)
        .unwrap_or(good.len());
    good[..split].sort_by(|a, b| flat_distance(*a, rather).total_cmp(&flat_distance(*b, rather)));
    good
}

/// What a party member does about an area attack it can't get out of (Bob, 2026-10-24: give up running and shield as
/// much damage on as many allies as it can.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Brace {
    /// Lowers what the caster does to everyone (Reprisal, Feint, Addle), on the caster.
    OnCaster,
    /// Shields or heals everyone around it (Succor, Temperance, Medica).
    AroundSelf,
    /// Shields one ally: the most hurt one it catches (Adloquium, Divine Benison, Aquaveil).
    OnAlly,
    /// Its own (Sheltron, Vengeance, Sentinel, Rampart).
    OnSelf,
}

/// The actions a party member braces with, by Action row, in the order it tries them within a kind (the stronger
/// first.) Each has a script; whether a member has one is its class and level (`GameData::brace_actions`.)
pub const BRACE_ACTIONS: [(u32, Brace); 13] = [
    (7535, Brace::OnCaster),    // Reprisal (tanks)
    (7549, Brace::OnCaster),    // Feint (melee)
    (7560, Brace::OnCaster),    // Addle (casters)
    (16536, Brace::AroundSelf), // Temperance
    (186, Brace::AroundSelf),   // Succor
    (124, Brace::AroundSelf),   // Medica
    (25861, Brace::OnAlly),     // Aquaveil
    (7432, Brace::OnAlly),      // Divine Benison
    (185, Brace::OnAlly),       // Adloquium
    (3542, Brace::OnSelf),      // Sheltron
    (44, Brace::OnSelf),        // Vengeance
    (17, Brace::OnSelf),        // Sentinel
    (7531, Brace::OnSelf),      // Rampart
];

/// The brace that covers the most: with two or more allies caught, something for all of them (on the caster, then
/// around itself); then a shield on one ally; then its own, if it's caught itself. `ready` are the actions it has and
/// can use now, in `BRACE_ACTIONS`' order.
pub fn choose_brace(
    ready: &[(u32, Brace)],
    allies_caught: usize,
    self_caught: bool,
) -> Option<(u32, Brace)> {
    let first = |kind: Brace| ready.iter().copied().find(|(_, brace)| *brace == kind);
    if allies_caught >= 2 {
        if let Some(found) = first(Brace::OnCaster).or_else(|| first(Brace::AroundSelf)) {
            return Some(found);
        }
    }
    if allies_caught >= 1
        && let Some(found) = first(Brace::OnCaster).or_else(|| first(Brace::OnAlly))
    {
        return Some(found);
    }
    if self_caught {
        return first(Brace::OnSelf).or_else(|| first(Brace::OnCaster));
    }
    None
}

/// When to set off: `REACTION_SECONDS` after noticing (`roll` between 0 and 1 picks within it), or now if the attack
/// goes off before it could get `distance` yalms at `speed` after waiting.
pub fn set_off_at(now: Instant, until: Instant, distance: f32, speed: f32, roll: f32) -> Instant {
    let (low, high) = REACTION_SECONDS;
    let wait = Duration::from_secs_f32(low + (high - low) * roll.clamp(0.0, 1.0));
    let run = Duration::from_secs_f32(distance / speed.max(0.1));
    if now + wait + run > until {
        now
    } else {
        now + wait
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn danger(shape: AreaShape) -> Danger {
        Danger {
            shape,
            caster_id: ObjectId::default(),
            caster: Vec3A::ZERO,
            rotation: 0.0, // facing +z
            aimed_at: Vec3A::new(0.0, 0.0, 10.0),
            caster_radius: 1.0,
            until: Instant::now() + Duration::from_secs(4),
        }
    }

    fn anywhere(_: Vec3A) -> bool {
        true
    }

    fn flat(a: Vec3A) -> f32 {
        Vec2::new(a.x, a.z).length()
    }

    /// Retail's dodge of the Tam-Tara Deepcroft's last boss: straight out of a circle around it, just past its edge.
    #[test]
    fn circle_around_the_caster() {
        let circle = danger(AreaShape::Circle {
            radius: 9.0,
            on_target: false,
        });
        let from = Vec3A::new(3.0, 0.0, 0.0);
        let spot = safe_spot(from, from, &[circle], anywhere).unwrap();
        assert!(!circle.catches(spot, DODGE_MARGIN));
        // Out along the way it stood: 9 + 1 (hitbox) + 0.5 + 0.5 and a hair.
        assert!(spot.z.abs() < 0.3, "{spot:?}");
        assert!((flat(spot) - 11.0).abs() < 0.2, "{spot:?}");
        // Already out: nowhere to go.
        assert!(safe_spot(Vec3A::new(12.0, 0.0, 0.0), from, &[circle], anywhere).is_none());
    }

    /// A donut 20 across with a hole a quarter of it, around a caster with a 1-yalm hitbox: someone in the ring goes
    /// into the hole (nearer than out past it), and someone in the hole stays.
    #[test]
    fn donut_into_the_hole() {
        let donut = danger(AreaShape::Donut {
            radius: 20.0,
            hole: 0.25,
            on_target: false,
        });
        // The ring runs from 5.25 (a quarter of 21) to 21 out; the hole's edge less a body and the margin is 4.25.
        assert!(donut.catches(Vec3A::new(8.0, 0.0, 0.0), DODGE_MARGIN));
        assert!(!donut.catches(Vec3A::new(3.0, 0.0, 0.0), DODGE_MARGIN));
        assert!(!donut.catches(Vec3A::new(23.0, 0.0, 0.0), DODGE_MARGIN));
        let spot = safe_spot(Vec3A::new(8.0, 0.0, 0.0), Vec3A::ZERO, &[donut], anywhere).unwrap();
        assert!(flat(spot) < 4.3, "{spot:?}");
        assert!(safe_spot(Vec3A::new(2.0, 0.0, 0.0), Vec3A::ZERO, &[donut], anywhere).is_none());
    }

    /// A cross 10 yalms each way, 4 wide: off the arm into a corner between them.
    #[test]
    fn cross_into_a_corner() {
        let cross = danger(AreaShape::Cross {
            length: 10.0,
            width: 4.0,
        });
        // On the arm ahead (facing +z), and on the one across.
        assert!(cross.catches(Vec3A::new(0.5, 0.0, 6.0), DODGE_MARGIN));
        assert!(cross.catches(Vec3A::new(-6.0, 0.0, 0.5), DODGE_MARGIN));
        assert!(!cross.catches(Vec3A::new(5.0, 0.0, 5.0), DODGE_MARGIN));
        let spot = safe_spot(Vec3A::new(0.5, 0.0, 6.0), Vec3A::ZERO, &[cross], anywhere).unwrap();
        assert!(!cross.catches(spot, DODGE_MARGIN));
        assert!(
            (spot - Vec3A::new(0.5, 0.0, 6.0)).length() < 3.5,
            "{spot:?}"
        );
    }

    #[test]
    fn circle_on_the_target() {
        let circle = danger(AreaShape::Circle {
            radius: 5.0,
            on_target: true,
        });
        let from = Vec3A::new(0.0, 0.0, 8.0); // 2 short of the middle, towards the caster
        let spot = safe_spot(from, from, &[circle], anywhere).unwrap();
        assert!(!circle.catches(spot, DODGE_MARGIN));
        assert!(spot.z < 5.0, "out the near side, {spot:?}");
    }

    #[test]
    fn cone_sidesteps() {
        let cone = danger(AreaShape::Cone {
            radius: 8.0,
            angle: 90.0,
        });
        // In front, a little to the right: off the right side is nearer than out of its far end.
        let from = Vec3A::new(1.5, 0.0, 4.0);
        let spot = safe_spot(from, from, &[cone], anywhere).unwrap();
        assert!(!cone.catches(spot, DODGE_MARGIN));
        assert!(Vec3A::distance(from, spot) < 4.0, "{spot:?}");
        // Behind the caster is safe already.
        assert!(safe_spot(Vec3A::new(0.0, 0.0, -3.0), from, &[cone], anywhere).is_none());
    }

    #[test]
    fn line_steps_off_the_side() {
        let line = danger(AreaShape::Line {
            length: 20.0,
            width: 4.0,
        });
        let from = Vec3A::new(0.5, 0.0, 10.0);
        let spot = safe_spot(from, from, &[line], anywhere).unwrap();
        assert!(!line.catches(spot, DODGE_MARGIN));
        // Off the nearer side (+x): 2 + 0.5 + 0.5 from the middle, along the same spot.
        assert!(spot.x > 2.9 && spot.x < 3.5, "{spot:?}");
        assert!((spot.z - 10.0).abs() < 0.6, "{spot:?}");
    }

    /// Two at once: the spot is out of both, and of the nearest ones, the one by where it would rather be.
    #[test]
    fn out_of_everything_near_its_place() {
        let circle = danger(AreaShape::Circle {
            radius: 4.0,
            on_target: false,
        });
        let line = Danger {
            caster: Vec3A::new(-20.0, 0.0, 0.0),
            rotation: std::f32::consts::FRAC_PI_2, // facing +x, through the middle
            ..danger(AreaShape::Line {
                length: 40.0,
                width: 4.0,
            })
        };
        let from = Vec3A::new(1.0, 0.0, 0.0);
        let rather = Vec3A::new(0.0, 0.0, -20.0);
        let spot = safe_spot(from, rather, &[circle, line], anywhere).unwrap();
        assert!(!circle.catches(spot, DODGE_MARGIN) && !line.catches(spot, DODGE_MARGIN));
        assert!(spot.z < 0.0, "towards where it would rather be, {spot:?}");
    }

    /// A wall (not walkable) on one side: it goes the other way.
    #[test]
    fn only_where_it_can_walk() {
        let circle = danger(AreaShape::Circle {
            radius: 4.0,
            on_target: false,
        });
        let from = Vec3A::new(1.0, 0.0, 0.0);
        let spot = safe_spot(from, from, &[circle], |point: Vec3A| point.x < 0.0).unwrap();
        assert!(
            spot.x < 0.0 && !circle.catches(spot, DODGE_MARGIN),
            "{spot:?}"
        );
    }

    #[test]
    fn nowhere_to_go_gives_up() {
        let danger = danger(AreaShape::Circle {
            radius: 10.0,
            on_target: false,
        });
        // Only the inside of the circle is on the map: no spot at all.
        let inside = |point: Vec3A| flat(point) < 8.0;
        assert!(safe_spots(Vec3A::new(0.0, 0.0, 2.0), Vec3A::ZERO, &[danger], inside).is_empty());
    }

    #[test]
    fn braces_cover_the_most() {
        let all = [
            (7535, Brace::OnCaster),
            (186, Brace::AroundSelf),
            (185, Brace::OnAlly),
            (7531, Brace::OnSelf),
        ];
        assert_eq!(choose_brace(&all, 3, true), Some((7535, Brace::OnCaster)));
        assert_eq!(
            choose_brace(&all[1..], 3, true),
            Some((186, Brace::AroundSelf))
        );
        assert_eq!(
            choose_brace(&all[1..], 1, false),
            Some((185, Brace::OnAlly))
        );
        assert_eq!(
            choose_brace(&all[3..], 1, true),
            Some((7531, Brace::OnSelf))
        );
        assert_eq!(choose_brace(&all[3..], 0, false), None);
    }

    #[test]
    fn when_to_set_off() {
        let now = Instant::now();
        let until = now + Duration::from_secs_f32(4.7);
        // Plenty of time: after the reaction.
        let at = set_off_at(now, until, 8.0, 6.0, 0.5);
        assert_eq!(at, now + Duration::from_secs_f32(0.7));
        // A quick cast: at once.
        assert_eq!(
            set_off_at(now, now + Duration::from_secs(1), 8.0, 6.0, 0.5),
            now
        );
    }
}
