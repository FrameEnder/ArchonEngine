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
}

impl Placement {
    pub fn is_empty(&self) -> bool {
        self.anchors.is_empty() && self.behind.is_none()
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
            return Some(anchors.iter().copied().sum::<Vec3A>() / anchors.len() as f32);
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
        // Too far from the player (and the tank): back to the middle of them.
        let goal = placement.goal(Vec3A::new(20.0, 0.0, 0.0), &positions, &targets, None, 15.0);
        assert_eq!(goal, Some(Vec3A::new(-1.5, 0.0, 0.0)));
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
}
