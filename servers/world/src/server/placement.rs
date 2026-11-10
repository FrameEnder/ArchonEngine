//! Where an NPC should stand in a fight, beyond reaching its target (the director's `keep_near` and `stand_behind`.)
//!
//! A support stays near the tank (and the player), but not directly behind the tank: with a smaller health pool it has
//! to keep out of cones and cleaves, so its place is usually behind the enemy the tank is fighting. A ranged sub-DPS
//! like Papalymo stays near both the one tanking and the player. See `docs/src/fork/combat.md` ("Placement").

use std::collections::HashMap;

use glam::Vec3A;
use kawari::common::ObjectId;

/// How close (in yalms) to its spot an NPC has to be to count as there.
pub const SPOT_TOLERANCE: f32 = 2.5;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Placement {
    /// Stay within `within` yalms of every one of these (it heads back to the middle of them when it isn't.)
    pub anchors: Vec<ObjectId>,
    pub within: f32,
    /// Stand behind the enemy this actor (usually the tank) is fighting, this many yalms beyond it.
    pub behind: Option<(ObjectId, f32)>,
    /// A tank: stand on the far side of its own target from these (the party), this many yalms off it, so the
    /// enemy faces away from them and its cones and cleaves miss them.
    pub opposite: Option<(Vec<ObjectId>, f32)>,
    /// A spot of its own in a boss fight (`boss_fight_spot`): it goes there and stays, whoever else moves, and its
    /// chases stop there too. Its party bot keeps it while its target is in reach from it (`keeps_spot`) and picks
    /// another when not.
    pub hold: Option<Vec3A>,
}

/// A tank moves round to the far side only when the party is less than this far round from it (degrees, seen from
/// the enemy: 180 is straight across.) Player tanks in Bob's Chronofoil capture of five Sastasha runs stood more than
/// 120 round about half the time (313 hits: 113 the median.)
pub const OPPOSITE_ENOUGH_DEGREES: f32 = 120.0;

/// How far inside `within` an NPC that strayed comes back to, and how far inside it a chase stops: the room between
/// the two keeps it from going back and forth at the edge. Sent back to the middle of its anchors and let chase to its
/// target again, a ranged party NPC ran up, attacked, walked back two steps and ran up again (Bob's Copperbell Mines
/// bundles of 2026-10-25.)
pub const RETURN_MARGIN: f32 = 3.0;

impl Placement {
    pub fn is_empty(&self) -> bool {
        self.anchors.is_empty()
            && self.behind.is_none()
            && self.opposite.is_none()
            && self.hold.is_none()
    }

    /// A tank's spot across its target (at `target`) from the party, if it isn't far enough round already.
    fn opposite_spot(
        &self,
        at: Vec3A,
        positions: &HashMap<ObjectId, Vec3A>,
        target: Vec3A,
    ) -> Option<Vec3A> {
        let (party, beyond) = self.opposite.as_ref()?;
        let members: Vec<Vec3A> = party
            .iter()
            .filter_map(|id| positions.get(id).copied())
            .collect();
        if members.is_empty() {
            return None;
        }
        let flat = |v: Vec3A| Vec3A::new(v.x, 0.0, v.z);
        let centre = members.iter().copied().sum::<Vec3A>() / members.len() as f32;
        let to_party = flat(centre - target).normalize_or_zero();
        let to_tank = flat(at - target).normalize_or_zero();
        if to_party == Vec3A::ZERO {
            return None;
        }
        if to_tank != Vec3A::ZERO
            && to_party.dot(to_tank).clamp(-1.0, 1.0).acos().to_degrees() >= OPPOSITE_ENOUGH_DEGREES
        {
            return None;
        }
        let spot = target - to_party * *beyond;
        (Vec3A::distance(at, spot) > SPOT_TOLERANCE).then_some(spot)
    }

    /// The nearest point from `at` towards `middle` that's `RETURN_MARGIN` inside `within` of every anchor (the middle
    /// itself when none is, anchors far apart.)
    fn back_within(&self, at: Vec3A, middle: Vec3A, anchors: &[Vec3A]) -> Vec3A {
        let limit = (self.within - RETURN_MARGIN).max(0.0);
        let inside = |point: Vec3A| {
            anchors
                .iter()
                .all(|anchor| point.distance(*anchor) <= limit)
        };
        if !inside(middle) {
            return middle;
        }
        // The share of the way from `at` to the middle, the least that's inside.
        let (mut out, mut ok) = (0.0_f32, 1.0_f32);
        for _ in 0..20 {
            let mid = (out + ok) / 2.0;
            if inside(at.lerp(middle, mid)) {
                ok = mid;
            } else {
                out = mid;
            }
        }
        at.lerp(middle, ok)
    }

    /// Where a chase after a target at `target` should stop, so it doesn't take the NPC further than `RETURN_MARGIN`
    /// inside `within` of an anchor (it would only be sent back, and come again): the target itself if it's near
    /// enough, else the nearest point towards it that is.
    pub fn chase_goal(&self, target: Vec3A, positions: &HashMap<ObjectId, Vec3A>) -> Vec3A {
        let anchors: Vec<Vec3A> = self
            .anchors
            .iter()
            .filter_map(|anchor| positions.get(anchor).copied())
            .collect();
        if let Some(spot) = self.hold {
            return spot;
        }
        if anchors.is_empty() || self.within <= 0.0 {
            return target;
        }
        let limit = (self.within - RETURN_MARGIN).max(0.0);
        if anchors
            .iter()
            .all(|anchor| target.distance(*anchor) <= limit)
        {
            return target;
        }
        let middle = anchors.iter().copied().sum::<Vec3A>() / anchors.len() as f32;
        self.back_within(target, middle, &anchors)
    }

    /// Where the NPC at `at` has to go now, if anywhere: back to the middle of its anchors when it's strayed too far
    /// from any of them, else to its spot behind the tank's target, as long as that spot keeps it near its anchors and
    /// its own target (at `own_target`, `reach` yalms) is in reach from there.
    pub fn goal(
        &self,
        at: Vec3A,
        positions: &HashMap<ObjectId, Vec3A>,
        targets: &HashMap<ObjectId, ObjectId>,
        own_target: Option<Vec3A>,
        reach: f32,
    ) -> Option<Vec3A> {
        if let Some(spot) = self.hold {
            return (flat(at, spot) > SPOT_TOLERANCE).then_some(spot);
        }
        let anchors: Vec<Vec3A> = self
            .anchors
            .iter()
            .filter_map(|anchor| positions.get(anchor).copied())
            .collect();
        if !anchors.is_empty()
            && anchors
                .iter()
                .any(|anchor| Vec3A::distance(at, *anchor) > self.within)
        {
            let middle = anchors.iter().copied().sum::<Vec3A>() / anchors.len() as f32;
            return Some(self.back_within(at, middle, &anchors));
        }

        if self.opposite.is_some() {
            // Only a spot that keeps it near its anchors: a tank sent round to one beyond its limit from the player,
            // then back inside it, then round again, circled through its target every second and the enemy turned
            // with it, its cones and areas aimed wherever it had turned (Kottos, Bob's bundle of 2026-10-26.)
            let limit = (self.within - RETURN_MARGIN).max(0.0);
            return self
                .opposite_spot(at, positions, own_target?)
                .filter(|spot| anchors.iter().all(|anchor| spot.distance(*anchor) <= limit));
        }

        let (tank, beyond) = self.behind?;
        let tank_pos = *positions.get(&tank)?;
        let enemy_pos = *positions.get(targets.get(&tank)?)?;
        let direction = (enemy_pos - tank_pos).normalize_or_zero();
        if direction == Vec3A::ZERO {
            return None;
        }
        let spot = enemy_pos + direction * beyond;
        let keeps_near = anchors
            .iter()
            .all(|anchor| Vec3A::distance(spot, *anchor) <= self.within);
        let in_reach = own_target.is_none_or(|target| Vec3A::distance(spot, target) <= reach);
        (keeps_near && in_reach && Vec3A::distance(at, spot) > SPOT_TOLERANCE).then_some(spot)
    }
}

/// Across the ground only.
fn flat(a: Vec3A, b: Vec3A) -> f32 {
    Vec3A::new(a.x - b.x, 0.0, a.z - b.z).length()
}

/// Where a party member stands to fight a boss in a boss fight: `beyond` yalms off the edge of what it fights (at
/// `target`, its hitbox `hitbox`, facing `facing`), `angle` degrees round from where that faces (0 in front, 180
/// behind), or the nearest that's `walkable` (round to either side by 20 degrees at a time, then nearer.) Retail's
/// party in Bob's Chronofoil captures (129 boss fights in ten dungeons and trials, a sample a second): ranged and
/// healers stood 9.5-11 yalms from the boss's middle (the median; 6-14 the middle half), off to its sides (the median
/// 90-115 degrees from its facing), melee 4-6 yalms, round its back flanks (players 125 degrees).
pub fn boss_fight_spot(
    target: Vec3A,
    facing: f32,
    hitbox: f32,
    angle: f32,
    beyond: f32,
    walkable: impl Fn(Vec3A) -> bool,
) -> Option<Vec3A> {
    for nearer in [1.0, 0.6] {
        for turn in [0.0, 20.0, -20.0, 40.0, -40.0, 60.0, -60.0, 80.0, -80.0] {
            let round = facing + (angle + turn).to_radians();
            let distance = hitbox + beyond * nearer;
            let spot = target + Vec3A::new(round.sin(), 0.0, round.cos()) * distance;
            if walkable(spot) {
                return Some(spot);
            }
        }
    }
    None
}

/// Whether a boss-fight spot (`boss_fight_spot`) still does: what it fights (at `target`) is no nearer than its own
/// edge and no further than `keep` yalms off, and in its reach (`reach`, from its middle) from there. Retail's ranged
/// and healers stood still three-quarters of a boss fight, and still half of the seconds the boss moved (Bob's
/// captures: they stepped out of attacks, but didn't walk after the boss as it was turned.)
pub fn keeps_spot(spot: Vec3A, target: Vec3A, hitbox: f32, keep: f32, reach: f32) -> bool {
    let distance = flat(spot, target);
    distance >= hitbox * 0.5 && distance <= (hitbox + keep).min(reach)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TANK: ObjectId = ObjectId(1);
    const ENEMY: ObjectId = ObjectId(2);
    const PLAYER: ObjectId = ObjectId(3);

    fn world() -> (HashMap<ObjectId, Vec3A>, HashMap<ObjectId, ObjectId>) {
        let positions = HashMap::from([
            (TANK, Vec3A::new(0.0, 0.0, 0.0)),
            (ENEMY, Vec3A::new(5.0, 0.0, 0.0)),
            (PLAYER, Vec3A::new(-3.0, 0.0, 0.0)),
        ]);
        (positions, HashMap::from([(TANK, ENEMY)]))
    }

    /// The tank goes round its target until the party is across from it, and stays put once it's far enough round.
    #[test]
    fn tank_across_from_the_party() {
        let positions = HashMap::from([
            (PLAYER, Vec3A::new(-3.0, 0.0, 0.0)),
            (ObjectId(4), Vec3A::new(-3.0, 0.0, 2.0)),
        ]);
        let placement = Placement {
            opposite: Some((vec![PLAYER, ObjectId(4)], 2.5)),
            ..Default::default()
        };
        let enemy = Vec3A::new(5.0, 0.0, 0.0);
        // On the party's side of the enemy: round to the far side, 2.5 yalms beyond it.
        let spot = placement
            .goal(
                Vec3A::new(3.0, 0.0, 0.0),
                &positions,
                &HashMap::new(),
                Some(enemy),
                3.0,
            )
            .unwrap();
        assert!(spot.x > enemy.x + 2.0, "{spot:?}");
        // Already straight across from the party: no move.
        let centre = Vec3A::new(-3.0, 0.0, 1.0);
        let across = enemy + (enemy - centre).normalize() * 2.5;
        assert!(
            placement
                .goal(across, &positions, &HashMap::new(), Some(enemy), 3.0)
                .is_none()
        );
    }

    #[test]
    fn behind_the_tanks_target() {
        let (positions, targets) = world();
        let placement = Placement {
            behind: Some((TANK, 6.0)),
            ..Default::default()
        };
        // The tank faces the enemy along +x, so the spot is 6 yalms past the enemy, out of its frontal cone.
        let goal = placement.goal(Vec3A::new(-5.0, 0.0, 0.0), &positions, &targets, None, 15.0);
        assert_eq!(goal, Some(Vec3A::new(11.0, 0.0, 0.0)));
        // Already there: nowhere to go.
        assert_eq!(
            placement.goal(Vec3A::new(10.0, 0.0, 1.0), &positions, &targets, None, 15.0),
            None
        );
        // Its own target out of reach from there: it doesn't go.
        let far = Some(Vec3A::new(-40.0, 0.0, 0.0));
        assert_eq!(
            placement.goal(Vec3A::new(-5.0, 0.0, 0.0), &positions, &targets, far, 15.0),
            None
        );
    }

    #[test]
    fn near_every_anchor() {
        let (positions, targets) = world();
        let placement = Placement {
            anchors: vec![TANK, PLAYER],
            within: 10.0,
            ..Default::default()
        };
        // Too far from the player (and the tank): back towards the middle of them, until it's `RETURN_MARGIN`
        // inside the limit of both (7 yalms from the player, at -3.)
        let goal = placement
            .goal(Vec3A::new(20.0, 0.0, 0.0), &positions, &targets, None, 15.0)
            .unwrap();
        assert!((goal.x - 4.0).abs() < 0.01, "{goal}");
        // A chase stops there too, rather than run off to be sent back.
        let chase = placement.chase_goal(Vec3A::new(20.0, 0.0, 0.0), &positions);
        assert!((chase.x - 4.0).abs() < 0.01, "{chase}");
        assert_eq!(
            placement.chase_goal(Vec3A::new(3.0, 0.0, 0.0), &positions),
            Vec3A::new(3.0, 0.0, 0.0)
        );
        // Close enough to both: stays.
        assert_eq!(
            placement.goal(Vec3A::new(2.0, 0.0, 0.0), &positions, &targets, None, 15.0),
            None
        );
        // A spot behind the enemy that would take it too far from the player isn't taken.
        let both = Placement {
            behind: Some((TANK, 6.0)),
            ..placement
        };
        assert_eq!(
            both.goal(Vec3A::new(2.0, 0.0, 0.0), &positions, &targets, None, 15.0),
            None
        );
    }

    /// A spot of its own: it goes there and stays, whatever its anchors, and its chases stop there.
    #[test]
    fn holding_a_spot() {
        let (positions, targets) = world();
        let spot = Vec3A::new(5.0, 0.0, 10.0);
        let placement = Placement {
            hold: Some(spot),
            anchors: vec![PLAYER],
            within: 1.0,
            ..Default::default()
        };
        assert!(!placement.is_empty());
        let far = Vec3A::new(20.0, 0.0, 0.0);
        assert_eq!(
            placement.goal(far, &positions, &targets, None, 15.0),
            Some(spot)
        );
        assert_eq!(placement.goal(spot, &positions, &targets, None, 15.0), None);
        assert_eq!(placement.chase_goal(far, &positions), spot);
    }

    /// Off to the boss's side, beyond its edge; round further where that isn't walkable; kept while the boss is in
    /// reach and not too far, given up when it isn't.
    #[test]
    fn a_spot_round_the_boss() {
        let boss = Vec3A::new(0.0, 0.0, 0.0);
        // Facing +z (rotation 0): 90 degrees round is +x.
        let spot = boss_fight_spot(boss, 0.0, 3.5, 90.0, 6.0, |_| true).unwrap();
        assert!((spot.x - 9.5).abs() < 0.01 && spot.z.abs() < 0.01, "{spot}");
        // +x not walkable: the next round is 20 degrees further.
        let spot = boss_fight_spot(boss, 0.0, 3.5, 90.0, 6.0, |at| at.z.abs() > 1.0).unwrap();
        assert!(
            (flat(spot, boss) - 9.5).abs() < 0.01 && spot.z.abs() > 1.0,
            "{spot}"
        );
        assert!(boss_fight_spot(boss, 0.0, 3.5, 90.0, 6.0, |_| false).is_none());

        let spot = Vec3A::new(9.5, 0.0, 0.0);
        assert!(keeps_spot(spot, boss, 3.5, 12.0, 18.5));
        // The boss turned and stepped a few yalms: kept.
        assert!(keeps_spot(
            spot,
            Vec3A::new(-3.0, 0.0, 4.0),
            3.5,
            12.0,
            18.5
        ));
        // Dragged off beyond `keep`, or onto the spot: a new one.
        assert!(!keeps_spot(
            spot,
            Vec3A::new(-9.0, 0.0, 0.0),
            3.5,
            12.0,
            18.5
        ));
        assert!(!keeps_spot(
            spot,
            Vec3A::new(9.0, 0.0, 0.0),
            3.5,
            12.0,
            18.5
        ));
    }
}
