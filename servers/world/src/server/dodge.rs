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

use physis::layer::TriggerBoxShape;

use crate::{
    GameData,
    gamedata::AreaShape,
    server::{actor::NetworkedActor, instance::Instance, zone::MapRange},
};

/// How far past an area attack's hit edge a dodger stops (retail's stood at 11.0 yalms from a circle they otherwise
/// left as soon as they could; the hit edge's own grazing allowance is `PLAYER_HITBOX_RADIUS`, in `in_area`.)
pub const DODGE_MARGIN: f32 = 0.5;
/// When a dodger sets off after it notices an area attack starting: a moment, a little quicker than retail's (1.0-1.4 s
/// into the cast in the capture, sampled every 0.31 s; Bob found the bots slow to react, 2026-10-26), or at once when
/// waiting would leave too little time to get out.
pub const REACTION_SECONDS: (f32, f32) = (0.5, 0.9);
/// Area attacks cast quicker than this go off before anyone can get out: retail's Duty Support members never dodged
/// Chudo-Yudo's Lion's Breath (902, a 0.7 s cone at its target), which hit the tank 24 times of 25 (Bob's Chronofoil
/// capture 1cefedf4 of 2026-11-07); Kawari's bots all got out of it, and it hit nobody in two runs of three (Bob's
/// bundles 20261107-221820 and 20261108-145830). Retail's members set off 1.0-1.4 s into a cast (the module.)
pub const TOO_QUICK_TO_DODGE: Duration = Duration::from_millis(1000);
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
    /// The action (Action row; a sweep's, `sweep.rs`): what a director's authored spots go by (`AuthoredSpots`.)
    pub action_id: u32,
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

/// Every area attack a hostile NPC is casting now: one at war with the players' side (by battalion, `action::at_war`;
/// the HOSTILE character flag is sent for allies too.)
pub fn dangers(instance: &Instance, game_data: &mut GameData, now: Instant) -> Vec<Danger> {
    let players_enemies = game_data.get_battalion_enemies(crate::server::action::PLAYER_BATTALION);
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
                && ((spawn
                    .character_data_flags
                    .contains(CharacterDataFlag::HOSTILE)
                    && crate::server::action::at_war(&players_enemies, actor))
                    || instance.held_npcs.contains(id))
                && cast.until > now
                && cast.action_id != 0
                && cast.length >= TOO_QUICK_TO_DODGE =>
            {
                let shape = game_data.get_action_info(cast.action_id)?.area_shape()?;
                Some(Danger {
                    action_id: cast.action_id,
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
        // And what a director sweeps along a line (`director:sweep_bnpc`): the way ahead of each moving one.
        .chain(instance.sweeps.iter().filter_map(|sweep| sweep.danger(now)))
        .collect()
}

/// How far inside a boss room's floor a dodger keeps (its hitbox: retail's party went no further than 19.4 yalms from
/// the middle of Brayflox's Longstop's last boss's 20-yalm terrace.)
pub const ARENA_MARGIN: f32 = 0.5;
/// Area attacks going off within this much of each other are one wave: a dodger gets out of all of them or none.
const SAME_WAVE: Duration = Duration::from_millis(250);

/// The floor of a boss room being fought (`DirectorBoss::arena`): a player can't leave it, so a dodger doesn't either.
#[derive(Debug, Clone, Copy)]
pub struct Arena {
    centre: Vec3A,
    /// Its half-sizes across the ground (x, z): a cylinder's or sphere's radius twice, a box's half-sizes.
    half: Vec2,
    round: bool,
}

impl Arena {
    /// From the layout's range: a cylinder or a sphere (round), or a box (turned boxes aren't, as Kawari's map ranges.)
    pub fn of(range: &MapRange) -> Option<Self> {
        let half = Vec2::new(range.scale.x, range.scale.z);
        match range.trigger_box_shape {
            TriggerBoxShape::Cylinder | TriggerBoxShape::Sphere => Some(Self {
                centre: range.position,
                half: Vec2::splat(range.scale.x),
                round: true,
            }),
            TriggerBoxShape::Box => Some(Self {
                centre: range.position,
                half,
                round: false,
            }),
            _ => None,
        }
    }

    /// Whether `point` is on it, `ARENA_MARGIN` in from its edge.
    pub fn holds(&self, point: Vec3A) -> bool {
        let offset = Vec2::new(point.x - self.centre.x, point.z - self.centre.z);
        let half = self.half - Vec2::splat(ARENA_MARGIN);
        if self.round {
            offset.length() <= half.x
        } else {
            offset.x.abs() <= half.x && offset.y.abs() <= half.y
        }
    }
}

/// The floor of the boss room the instance's director is fighting in, if it named one (and the zone has that range: a
/// layout ID that isn't one leaves the dodgers free, as before.)
pub fn arena(instance: &Instance) -> Option<Arena> {
    let id = instance
        .directors
        .iter()
        .find_map(|director| director.arena())?;
    instance
        .zone
        .map_ranges
        .iter()
        .find(|range| range.instance_id == id)
        .and_then(Arena::of)
}

/// The dangers a dodger tries to be out of, in turn: all of them, then all but the last wave to go off, and so on down
/// to the first wave alone (Bob, 2026-11-04: with Brayflox's Longstop's last boss's puddles over its whole terrace,
/// old and new, get out of the older ones and wait for them to go off, then out of the newer ones, as a player does,
/// never off the floor.) Each is the soonest waves, soonest first.
pub fn in_turn(dangers: &[Danger]) -> Vec<Vec<Danger>> {
    let mut soonest = dangers.to_vec();
    soonest.sort_by_key(|danger| danger.until);
    // Where each wave ends: the first danger going off more than `SAME_WAVE` after the one before it.
    let mut ends: Vec<usize> = (1..soonest.len())
        .filter(|&i| soonest[i].until > soonest[i - 1].until + SAME_WAVE)
        .collect();
    ends.push(soonest.len());
    ends.iter()
        .rev()
        .filter(|&&end| end > 0)
        .map(|&end| soonest[..end].to_vec())
        .collect()
}

/// What a dodger caught in an area attack does.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Choice {
    /// Runs there.
    Go(Vec3A),
    /// Stays: it's out of the soonest waves already, and waits for them to go off before it gets out of the rest.
    Wait,
    /// Nowhere out of even the soonest wave (it braces.)
    Nowhere,
}

/// Out of as many of the waves as it can (`in_turn`, all of them first): for each, staying if it's out of them where
/// it stands, else the spot it was already going to (`previous`) if that's still out of them and allowed, else the
/// first of the spots out of them (`safe_spots`) that `pick` takes (`party_bot`: the first a navmesh path reaches.)
pub fn choose(
    from: Vec3A,
    rather: Vec3A,
    in_turn: &[Vec<Danger>],
    previous: Option<Vec3A>,
    walkable: impl Fn(Vec3A) -> bool,
    pick: impl Fn(Vec<Vec3A>) -> Option<Vec3A>,
) -> Choice {
    let out_of = |point: Vec3A, wave: &[Danger]| {
        wave.iter()
            .all(|danger| !danger.catches(point, DODGE_MARGIN))
    };
    for wave in in_turn {
        if out_of(from, wave) {
            return Choice::Wait;
        }
        if let Some(to) = previous.filter(|to| walkable(*to) && out_of(*to, wave)) {
            return Choice::Go(to);
        }
        if let Some(to) = pick(safe_spots(from, rather, wave, &walkable)) {
            return Choice::Go(to);
        }
    }
    Choice::Nowhere
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
    safe_spots(from, rather, dangers, &walkable)
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
    walkable: &impl Fn(Vec3A) -> bool,
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

/// When a party member sets off for a spot its duty's director authored (`AuthoredSpots`, `PartyCue`) unless the
/// director says otherwise: at once, or a moment after (retail's Duty Support members in the Stone Vigil were at
/// theirs 0.0-0.5 s into Chudo-Yudo's Swinge and Isgebind's Cauterize, Bob's Chronofoil capture 1cefedf4 of
/// 2026-11-07: they set off as the cast began, or as the boss leapt before it.)
pub const AUTHORED_REACTION_SECONDS: (f32, f32) = (0.0, 0.3);
/// How near its recorded spot a caster must be for an authored entry to go with its cast (the bosses' leap and flight
/// spots are fixed: within 0.1 yalms over retail's runs.)
pub const CASTER_NEAR: f32 = 3.0;

/// Where each role of a party goes, on the ground (x, z): "tank", "healer", "melee", "ranged", and "dps" for either
/// (a second DPS takes "dps2" if there is one.) Retail's Duty Support members went to the same spots, by role, every
/// time a boss did the same thing from the same place (the Stone Vigil, Bob's capture 1cefedf4.)
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RoleSpots(pub Vec<(String, Vec2)>);

impl RoleSpots {
    /// Where the `nth` (0-based) party member of `role` ("tank", "healer", "melee", "ranged") goes.
    pub fn for_role(&self, role: &str, nth: usize) -> Option<Vec2> {
        let find = |name: &str| {
            self.0
                .iter()
                .find(|(spot_role, _)| spot_role == name)
                .map(|(_, at)| *at)
        };
        let dps = matches!(role, "melee" | "ranged");
        let numbered = (nth > 0).then(|| format!("{role}{}", nth + 1));
        let numbered_dps = (dps && nth > 0).then(|| format!("dps{}", nth + 1));
        numbered
            .as_deref()
            .and_then(find)
            .or_else(|| numbered_dps.as_deref().and_then(find))
            .or_else(|| find(role))
            .or_else(|| if dps { find("dps") } else { None })
    }
}

/// Where a duty's party goes as an enemy casts an action (`director:party_spots`): retail's Duty Support members
/// didn't run from such a cast by its shape, but to spots their duty set for that cast from that place, whether it
/// would have caught them or not.
#[derive(Debug, Clone, PartialEq)]
pub struct AuthoredSpots {
    pub action: u32,
    /// Where the caster casts it from (x, z), or anywhere.
    pub caster: Option<Vec2>,
    pub spots: RoleSpots,
    /// When they set off after the cast began (`REACTION_SECONDS`' kind: a roll between the two.)
    pub react: (f32, f32),
    /// The spots stand inside this attack by design: one that covers the whole arena and hurts less the farther out
    /// you are (Titan's Geocrush; retail's Duty Support members went to the arena's edge, inside it: Bob's capture
    /// 650dbfd8), so its own area doesn't send them to the shapes. Other attacks being cast still do.
    pub inside: bool,
}

impl AuthoredSpots {
    /// Whether these spots go with `danger`.
    pub fn matches(&self, danger: &Danger) -> bool {
        danger.action_id == self.action
            && self.caster.is_none_or(|at| {
                Vec2::new(danger.caster.x - at.x, danger.caster.z - at.y).length() <= CASTER_NEAR
            })
    }
}

/// Spots a director sends the party to for a while, whatever is being cast (`director:party_positions`): Isgebind's
/// party spreading out as it took off for its Sheets of Ice (Bob's capture 1cefedf4.)
#[derive(Debug, Clone)]
pub struct PartyCue {
    pub spots: RoleSpots,
    pub from: Instant,
    pub until: Instant,
    pub react: (f32, f32),
}

/// Where the `nth` party member of `role` should be now by its director's authority, and until when: the spots for
/// the soonest danger they go with (`AuthoredSpots`), else the director's cue (`PartyCue`.) None when there are
/// none, when the role has no spot, or when its spot is in one of the dangers, to its hit edge (then it dodges by the
/// shapes, as a second Swinge aimed at the tank where it stood moved only the tank, to the cone's edge: Bob's capture
/// 1cefedf4.)
pub fn authored_spot(
    authored: &[AuthoredSpots],
    cue: Option<&PartyCue>,
    dangers: &[Danger],
    role: &str,
    nth: usize,
    now: Instant,
) -> Option<(Vec2, Instant, (f32, f32))> {
    let mut soonest: Vec<&Danger> = dangers.iter().collect();
    soonest.sort_by_key(|danger| danger.until);
    // The danger the spots go with, when they stand inside it by design (`AuthoredSpots::inside`.)
    let from_cast = soonest.iter().find_map(|danger| {
        authored
            .iter()
            .find(|entry| entry.matches(danger))
            .and_then(|entry| {
                entry.spots.for_role(role, nth).map(|at| {
                    let inside = entry.inside.then_some(*danger);
                    (at, danger.until, entry.react, inside)
                })
            })
    });
    let from_cue = || {
        cue.filter(|cue| now >= cue.from && now < cue.until)
            .and_then(|cue| {
                cue.spots
                    .for_role(role, nth)
                    .map(|at| (at, cue.until, cue.react, None))
            })
    };
    let (at, until, react, inside) = from_cast.or_else(from_cue)?;
    let point = Vec3A::new(at.x, 0.0, at.y);
    // Height doesn't matter to the shapes on the ground (`in_area_with_reach` goes by x and z.) Retail's spots are
    // trusted to their hit edge, without the dodgers' margin: some stood just outside (the Conjurer 4 yalms from a
    // whirlwind's lane, its reach 3 and a body 0.5.)
    let caught = dangers
        .iter()
        .filter(|danger| !inside.is_some_and(|skip| std::ptr::eq(skip, *danger)))
        .any(|danger| danger.catches(point, 0.0));
    (!caught).then_some((at, until, react))
}

/// When to set off for an authored spot: `react`'s span after noticing (`roll` between 0 and 1 picks within it.)
pub fn authored_set_off(now: Instant, react: (f32, f32), roll: f32) -> Instant {
    let (low, high) = react;
    now + Duration::from_secs_f32((low + (high - low) * roll.clamp(0.0, 1.0)).max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn danger(shape: AreaShape) -> Danger {
        Danger {
            action_id: 0,
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
        assert!(safe_spots(Vec3A::new(0.0, 0.0, 2.0), Vec3A::ZERO, &[danger], &inside).is_empty());
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

    /// Brayflox's Longstop's last boss's puddles (Bob's bundle 20261104-204423, the spots from its helpers): 10-yalm
    /// circles on the middle of its 20-yalm terrace and on a hexagon 16 yalms out, the middle and set A going off
    /// first, the middle and set B 5.1 s later. Out of both there's nowhere on the floor, only off it (where Kawari's
    /// party went); on it, out of the first wave (by a B spot), then, once that's gone off, out of the second.
    #[test]
    fn puddles_in_turn_on_the_floor() {
        let now = Instant::now();
        let range = MapRange {
            trigger_box_shape: TriggerBoxShape::Cylinder,
            position: Vec3A::ZERO,
            scale: Vec3A::new(20.0, 10.0, 20.0),
            ..Default::default()
        };
        let arena = Arena::of(&range).unwrap();
        let puddle = |angle: Option<f32>, after: f32| {
            let caster = angle.map_or(Vec3A::ZERO, |degrees: f32| {
                let a = degrees.to_radians();
                Vec3A::new(a.sin() * 16.0, 0.0, a.cos() * 16.0)
            });
            Danger {
                caster,
                aimed_at: caster,
                caster_radius: 0.5,
                until: now + Duration::from_secs_f32(after),
                ..danger(AreaShape::Circle {
                    radius: 10.0,
                    on_target: false,
                })
            }
        };
        let first = [None, Some(0.0), Some(120.0), Some(240.0)].map(|at| puddle(at, 9.0));
        let second = [None, Some(60.0), Some(180.0), Some(300.0)].map(|at| puddle(at, 14.1));
        let all: Vec<Danger> = first.iter().chain(&second).copied().collect();
        let on_floor = |point: Vec3A| arena.holds(point);
        let first_spot = |spots: Vec<Vec3A>| spots.into_iter().next();

        let waves = in_turn(&all);
        assert_eq!(waves.len(), 2);
        assert_eq!(waves[0].len(), 8);
        assert_eq!(waves[1].len(), 4);

        // On an A spot: out of everything is only off the floor.
        let here = first[1].caster;
        let anywhere_spot = safe_spot(here, here, &all, anywhere).unwrap();
        assert!(!arena.holds(anywhere_spot), "{anywhere_spot:?}");
        // On it: out of the first wave, by the nearer B spot, and inside the edge.
        let Choice::Go(to) = choose(here, here, &waves, None, on_floor, first_spot) else {
            panic!("it should go");
        };
        assert!(arena.holds(to));
        assert!(first.iter().all(|danger| !danger.catches(to, DODGE_MARGIN)));
        assert!(second.iter().any(|danger| danger.catches(to, DODGE_MARGIN)));
        // There, it waits for the first wave.
        assert_eq!(
            choose(to, to, &waves, Some(to), on_floor, first_spot),
            Choice::Wait
        );
        // Once it's gone off, out of the second, still on the floor.
        let next = in_turn(&second);
        let Choice::Go(then) = choose(to, to, &next, Some(to), on_floor, first_spot) else {
            panic!("it should go again");
        };
        assert!(arena.holds(then));
        assert!(
            second
                .iter()
                .all(|danger| !danger.catches(then, DODGE_MARGIN))
        );
    }

    #[test]
    fn arena_shapes() {
        let round = Arena::of(&MapRange {
            trigger_box_shape: TriggerBoxShape::Cylinder,
            position: Vec3A::new(10.0, 5.0, 10.0),
            scale: Vec3A::new(20.0, 10.0, 20.0),
            ..Default::default()
        })
        .unwrap();
        assert!(round.holds(Vec3A::new(29.4, 0.0, 10.0)));
        assert!(!round.holds(Vec3A::new(29.6, 0.0, 10.0)));
        assert!(!round.holds(Vec3A::new(25.0, 0.0, 25.0)));
        let square = Arena::of(&MapRange {
            trigger_box_shape: TriggerBoxShape::Box,
            position: Vec3A::ZERO,
            scale: Vec3A::new(10.0, 5.0, 4.0),
            ..Default::default()
        })
        .unwrap();
        assert!(square.holds(Vec3A::new(9.0, 0.0, 3.0)));
        assert!(!square.holds(Vec3A::new(9.0, 0.0, 3.6)));
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

    fn spots(list: &[(&str, f32, f32)]) -> RoleSpots {
        RoleSpots(
            list.iter()
                .map(|(role, x, z)| (role.to_string(), Vec2::new(*x, *z)))
                .collect(),
        )
    }

    #[test]
    fn role_spots_by_role() {
        let spots = spots(&[
            ("tank", 0.0, 1.0),
            ("healer", 2.0, 3.0),
            ("dps", 4.0, 5.0),
            ("dps2", 6.0, 7.0),
        ]);
        assert_eq!(spots.for_role("tank", 0), Some(Vec2::new(0.0, 1.0)));
        assert_eq!(spots.for_role("melee", 0), Some(Vec2::new(4.0, 5.0)));
        assert_eq!(spots.for_role("ranged", 1), Some(Vec2::new(6.0, 7.0)));
        // A second healer has no spot of its own: the healers' spot.
        assert_eq!(spots.for_role("healer", 1), Some(Vec2::new(2.0, 3.0)));
        assert_eq!(RoleSpots::default().for_role("tank", 0), None);
    }

    /// Chudo-Yudo's Swinge from its leap spot at (-15, 116) (Bob's capture 1cefedf4, 458.5 and 464.9 s): the first
    /// (facing 1.693) sent all three to their spots, none of them in it; the second, aimed at the tank where it stood
    /// (facing -2.761), caught the tank's spot, so only the tank dodged by the cone, and the others stayed.
    #[test]
    fn swinge_spots() {
        let now = Instant::now();
        let entry = AuthoredSpots {
            action: 903,
            caster: Some(Vec2::new(-15.0, 116.0)),
            spots: spots(&[
                ("tank", -17.0, 111.0),
                ("dps", -19.0, 120.0),
                ("healer", -18.0, 123.0),
            ]),
            react: AUTHORED_REACTION_SECONDS,
            inside: false,
        };
        let swinge = |rotation: f32| Danger {
            action_id: 903,
            caster: Vec3A::new(-15.03, 0.44, 115.98),
            rotation,
            caster_radius: 3.0,
            ..danger(AreaShape::Cone {
                radius: 40.0,
                angle: 60.0,
            })
        };
        let first = [swinge(1.693)];
        let tank = authored_spot(std::slice::from_ref(&entry), None, &first, "tank", 0, now);
        assert_eq!(tank.map(|(at, ..)| at), Some(Vec2::new(-17.0, 111.0)));
        let second = [swinge(-2.761)];
        assert!(
            authored_spot(std::slice::from_ref(&entry), None, &second, "tank", 0, now).is_none()
        );
        let dps = authored_spot(
            std::slice::from_ref(&entry),
            None,
            &second,
            "ranged",
            0,
            now,
        );
        assert_eq!(dps.map(|(at, ..)| at), Some(Vec2::new(-19.0, 120.0)));
        // From elsewhere (its other leap spots), or another action: not these spots.
        let elsewhere = [Danger {
            caster: Vec3A::new(15.0, 0.0, 116.0),
            ..swinge(1.693)
        }];
        assert!(
            authored_spot(
                std::slice::from_ref(&entry),
                None,
                &elsewhere,
                "tank",
                0,
                now
            )
            .is_none()
        );
    }

    /// Titan's Geocrush from the middle of the Navel covers the arena: retail's Duty Support members went to its edge
    /// all the same (Bob's capture 650dbfd8: the tank to (0, -19), the healer (0, 19-21), the dps (19-21, 0)), so its
    /// spots are trusted inside it (`inside`); another attack being cast on a spot still sends that one to the shapes.
    #[test]
    fn spots_inside_their_attack() {
        let now = Instant::now();
        let mut entry = AuthoredSpots {
            action: 651,
            caster: Some(Vec2::ZERO),
            spots: spots(&[
                ("tank", 0.0, -19.0),
                ("healer", 0.0, 19.0),
                ("dps", 19.0, 0.0),
            ]),
            react: AUTHORED_REACTION_SECONDS,
            inside: false,
        };
        let geocrush = Danger {
            action_id: 651,
            caster: Vec3A::ZERO,
            ..danger(AreaShape::Circle {
                radius: 30.0,
                on_target: false,
            })
        };
        let only = [geocrush];
        let tank = |entry: &AuthoredSpots, dangers: &[Danger]| {
            authored_spot(std::slice::from_ref(entry), None, dangers, "tank", 0, now)
                .map(|(at, ..)| at)
        };
        assert_eq!(tank(&entry, &only), None);
        entry.inside = true;
        assert_eq!(tank(&entry, &only), Some(Vec2::new(0.0, -19.0)));
        let other = Danger {
            action_id: 652,
            caster: Vec3A::new(0.0, 0.0, -19.0),
            ..danger(AreaShape::Circle {
                radius: 5.0,
                on_target: false,
            })
        };
        assert_eq!(tank(&entry, &[geocrush, other]), None);
    }

    /// A director's cue holds only for its time, and a cast's own spots come first.
    #[test]
    fn cue_spots() {
        let now = Instant::now();
        let cue = PartyCue {
            spots: spots(&[
                ("tank", 0.0, -267.0),
                ("dps", 17.8, -253.4),
                ("healer", 0.0, -229.0),
            ]),
            from: now,
            until: now + Duration::from_secs(8),
            react: AUTHORED_REACTION_SECONDS,
        };
        let healer = authored_spot(&[], Some(&cue), &[], "healer", 0, now);
        assert_eq!(healer.map(|(at, ..)| at), Some(Vec2::new(0.0, -229.0)));
        assert!(
            authored_spot(
                &[],
                Some(&cue),
                &[],
                "healer",
                0,
                now + Duration::from_secs(9)
            )
            .is_none()
        );
        // A Sheet of Ice on the healer's spot: it dodges by the circle instead.
        let sheet = Danger {
            aimed_at: Vec3A::new(0.0, 0.0, -229.0),
            ..danger(AreaShape::Circle {
                radius: 5.0,
                on_target: true,
            })
        };
        assert!(authored_spot(&[], Some(&cue), &[sheet], "healer", 0, now).is_none());
    }
}
