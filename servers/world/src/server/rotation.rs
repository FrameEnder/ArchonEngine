//! Class rotations for NPCs (`Timeline::rotation`): which action an NPC uses next, by its class's rules.
//!
//! The mechanics are the Action sheet's (read at runtime through `GameData::get_action_mechanics`): whether an action
//! is on the GCD (CooldownGroup 58), its recast, and the action it follows in a combo (ActionCombo). The rotation adds
//! what retail's recordings show: the NPC's pace, priorities, how often it re-applies a damage-over-time, which action
//! follows a buff. See `docs/src/fork/combat.md` ("Class rotations").

use std::collections::HashMap;
use std::time::{Duration, Instant};

use kawari::common::{ObjectId, Rotation};

/// The GCD's recast group in the Action sheet.
pub const GCD_GROUP: u8 = 58;
/// How long a combo stays open after its last step (as for players.)
pub const COMBO_WINDOW: Duration = Duration::from_secs(15);
/// An ability isn't used straight after a GCD action, but this long after (weaving, as players do.)
pub const WEAVE_DELAY: Duration = Duration::from_millis(1000);

/// What the Action sheet says about an action, as far as rotations go.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ActionMechanics {
    /// On the GCD (a weaponskill or spell), not an ability.
    pub gcd: bool,
    /// Its recast in seconds.
    pub recast: f32,
    /// The action it follows in a combo (0 for none.)
    pub combo_from: u32,
}

/// Where an NPC is in its rotation.
#[derive(Debug, Clone, Default)]
pub struct RotationState {
    /// Its last GCD action and when.
    last_gcd: Option<(u32, Instant)>,
    /// When each action was last used.
    used: HashMap<u32, Instant>,
    /// When each action was last used on each target (for `refresh`.)
    used_on: HashMap<(u32, ObjectId), Instant>,
    /// A GCD action that has to come next (an entry's `then`.)
    next: Option<u32>,
    /// An ability was used since the last GCD action.
    weaved: bool,
}

/// What the NPC's situation is when it's choosing.
pub struct Situation {
    pub now: Instant,
    pub target: ObjectId,
    /// The target's and its own health, in %.
    pub target_hp: f32,
    pub self_hp: f32,
}

impl RotationState {
    /// The action to use now, if any.
    pub fn choose(
        &self,
        rotation: &Rotation,
        situation: &Situation,
        mechanics: &mut impl FnMut(u32) -> Option<ActionMechanics>,
    ) -> Option<u32> {
        let now = situation.now;
        let pace = Duration::from_secs_f32(rotation.pace.max(0.5));
        let gcd_ready = self
            .last_gcd
            .is_none_or(|(_, at)| now.duration_since(at) >= pace);
        // Abilities go between GCD actions: never before the first one (an NPC opens with a weaponskill or spell.)
        let can_weave = !self.weaved
            && self
                .last_gcd
                .is_some_and(|(_, at)| now.duration_since(at) >= WEAVE_DELAY);

        // A buff's follow-up comes first, as soon as the GCD allows.
        if let Some(next) = self.next {
            return gcd_ready.then_some(next);
        }

        for entry in &rotation.actions {
            let Some(action) = mechanics(entry.action) else {
                continue;
            };
            if action.gcd && !gcd_ready || !action.gcd && !can_weave {
                continue;
            }
            // Its own cooldown: `every`, or the recast for abilities and for GCD actions with a longer one.
            let cooldown = entry
                .every
                .or((!action.gcd || action.recast > rotation.pace).then_some(action.recast));
            if let Some(cooldown) = cooldown
                && self.used.get(&entry.action).is_some_and(|at| {
                    now.duration_since(*at) < Duration::from_secs_f32(cooldown.max(0.0))
                })
            {
                continue;
            }
            if let Some(refresh) = entry.refresh
                && self
                    .used_on
                    .get(&(entry.action, situation.target))
                    .is_some_and(|at| {
                        now.duration_since(*at) < Duration::from_secs_f32(refresh.max(0.0))
                    })
            {
                continue;
            }
            // The next step of a combo only follows the step before it.
            if action.gcd && action.combo_from != 0 {
                let continues = self.last_gcd.is_some_and(|(last, at)| {
                    last == action.combo_from && now.duration_since(at) <= COMBO_WINDOW
                });
                if !continues {
                    continue;
                }
            }
            if entry
                .target_hp_below
                .is_some_and(|below| situation.target_hp >= below)
            {
                continue;
            }
            if entry
                .self_hp_below
                .is_some_and(|below| situation.self_hp >= below)
            {
                continue;
            }
            return Some(entry.action);
        }
        None
    }

    /// Notes that `action` was used on `target`.
    pub fn record(
        &mut self,
        rotation: &Rotation,
        action: u32,
        target: ObjectId,
        now: Instant,
        mechanics: Option<ActionMechanics>,
    ) {
        self.used.insert(action, now);
        self.used_on.insert((action, target), now);
        let gcd = mechanics.is_some_and(|m| m.gcd);
        if gcd {
            self.last_gcd = Some((action, now));
            self.weaved = false;
            if self.next == Some(action) {
                self.next = None;
            }
        } else {
            self.weaved = true;
        }
        if let Some(then) = rotation
            .actions
            .iter()
            .find(|entry| entry.action == action)
            .and_then(|entry| entry.then)
        {
            self.next = Some(then);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kawari::common::RotationAction;

    // A lancer, the way Miraudont fights: Life Surge (ability, 45 s) then Vorpal Thrust; Leg Sweep (ability, 20 s);
    // True Thrust -> Vorpal Thrust as the combo; a GCD every 3 s.
    const TRUE_THRUST: u32 = 75;
    const VORPAL_THRUST: u32 = 78;
    const LEG_SWEEP: u32 = 82;
    const LIFE_SURGE: u32 = 83;
    const THUNDER: u32 = 144;

    fn mechanics(action: u32) -> Option<ActionMechanics> {
        Some(match action {
            TRUE_THRUST => ActionMechanics {
                gcd: true,
                recast: 2.5,
                combo_from: 0,
            },
            VORPAL_THRUST => ActionMechanics {
                gcd: true,
                recast: 2.5,
                combo_from: TRUE_THRUST,
            },
            LEG_SWEEP => ActionMechanics {
                gcd: false,
                recast: 20.0,
                combo_from: 0,
            },
            LIFE_SURGE => ActionMechanics {
                gcd: false,
                recast: 45.0,
                combo_from: 0,
            },
            THUNDER => ActionMechanics {
                gcd: true,
                recast: 2.5,
                combo_from: 0,
            },
            _ => return None,
        })
    }

    fn lancer() -> Rotation {
        Rotation {
            pace: 3.0,
            actions: vec![
                RotationAction {
                    action: LIFE_SURGE,
                    then: Some(VORPAL_THRUST),
                    ..Default::default()
                },
                RotationAction {
                    action: LEG_SWEEP,
                    ..Default::default()
                },
                RotationAction {
                    action: VORPAL_THRUST,
                    ..Default::default()
                },
                RotationAction {
                    action: TRUE_THRUST,
                    ..Default::default()
                },
            ],
        }
    }

    /// Runs the rotation for `seconds`, acting every half second like the server, and returns what it used when.
    fn simulate(rotation: &Rotation, seconds: u64) -> Vec<(f32, u32)> {
        let start = Instant::now();
        let mut state = RotationState::default();
        let mut used = Vec::new();
        for tick in 0..seconds * 2 {
            let now = start + Duration::from_millis(tick * 500);
            let situation = Situation {
                now,
                target: ObjectId(1),
                target_hp: 100.0,
                self_hp: 100.0,
            };
            if let Some(action) = state.choose(rotation, &situation, &mut mechanics) {
                state.record(rotation, action, ObjectId(1), now, mechanics(action));
                used.push((tick as f32 / 2.0, action));
            }
        }
        used
    }

    #[test]
    fn combo_follows_its_step_and_abilities_weave() {
        let used = simulate(&lancer(), 12);
        let actions: Vec<u32> = used.iter().map(|(_, a)| *a).collect();
        // True Thrust opens (Vorpal can't without it), then Life Surge and Leg Sweep are woven in a second after GCD
        // actions, and Life Surge's follow-up is Vorpal Thrust, which then can't repeat until True Thrust again.
        assert_eq!(actions[0], TRUE_THRUST);
        assert_eq!(actions[1], LIFE_SURGE);
        assert_eq!(actions[2], VORPAL_THRUST);
        assert!(actions.contains(&LEG_SWEEP));
        for pair in actions.windows(2) {
            assert!(!(pair[0] == VORPAL_THRUST && pair[1] == VORPAL_THRUST));
        }
        // GCD actions are 3 s apart.
        let gcds: Vec<f32> = used
            .iter()
            .filter(|(_, a)| mechanics(*a).unwrap().gcd)
            .map(|(t, _)| *t)
            .collect();
        for pair in gcds.windows(2) {
            assert!(pair[1] - pair[0] >= 3.0, "{gcds:?}");
        }
        // One ability between two GCD actions at most, and a second after the GCD.
        for (i, (t, a)) in used.iter().enumerate() {
            if !mechanics(*a).unwrap().gcd {
                let before = used[..i]
                    .iter()
                    .rev()
                    .find(|(_, b)| mechanics(*b).unwrap().gcd)
                    .unwrap();
                assert!(t - before.0 >= 1.0);
                assert!(mechanics(used[i - 1].1).unwrap().gcd);
            }
        }
    }

    #[test]
    fn abilities_wait_for_their_recast() {
        let used = simulate(&lancer(), 60);
        let sweeps: Vec<f32> = used
            .iter()
            .filter(|(_, a)| *a == LEG_SWEEP)
            .map(|(t, _)| *t)
            .collect();
        assert!(sweeps.len() >= 2);
        for pair in sweeps.windows(2) {
            assert!(pair[1] - pair[0] >= 20.0);
        }
        let surges = used.iter().filter(|(_, a)| *a == LIFE_SURGE).count();
        assert_eq!(surges, 2); // at 0 and 45 s
    }

    #[test]
    fn refresh_waits_per_target_and_hp_conditions() {
        let rotation = Rotation {
            pace: 2.5,
            actions: vec![
                RotationAction {
                    action: THUNDER,
                    refresh: Some(18.0),
                    ..Default::default()
                },
                RotationAction {
                    action: TRUE_THRUST,
                    ..Default::default()
                },
            ],
        };
        let used = simulate(&rotation, 45);
        let thunders: Vec<f32> = used
            .iter()
            .filter(|(_, a)| *a == THUNDER)
            .map(|(t, _)| *t)
            .collect();
        // Re-applied on the first GCD slot (every 2.5 s) once 18 s have passed.
        assert_eq!(thunders, vec![0.0, 20.0, 40.0]);

        // A new target gets it straight away.
        let start = Instant::now();
        let mut state = RotationState::default();
        state.record(&rotation, THUNDER, ObjectId(1), start, mechanics(THUNDER));
        let later = start + Duration::from_secs(3);
        let on_other = Situation {
            now: later,
            target: ObjectId(2),
            target_hp: 100.0,
            self_hp: 100.0,
        };
        assert_eq!(
            state.choose(&rotation, &on_other, &mut mechanics),
            Some(THUNDER)
        );
        let on_same = Situation {
            now: later,
            target: ObjectId(1),
            target_hp: 100.0,
            self_hp: 100.0,
        };
        assert_eq!(
            state.choose(&rotation, &on_same, &mut mechanics),
            Some(TRUE_THRUST)
        );

        let finisher = Rotation {
            pace: 2.5,
            actions: vec![RotationAction {
                action: TRUE_THRUST,
                target_hp_below: Some(20.0),
                ..Default::default()
            }],
        };
        let state = RotationState::default();
        let healthy = Situation {
            now: start,
            target: ObjectId(1),
            target_hp: 50.0,
            self_hp: 100.0,
        };
        assert_eq!(state.choose(&finisher, &healthy, &mut mechanics), None);
        let low = Situation {
            now: start,
            target: ObjectId(1),
            target_hp: 10.0,
            self_hp: 100.0,
        };
        assert_eq!(
            state.choose(&finisher, &low, &mut mechanics),
            Some(TRUE_THRUST)
        );
    }
}
