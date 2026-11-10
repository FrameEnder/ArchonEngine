//! Class rotations for NPCs (`Timeline::rotation`): which action an NPC uses next, by its class's rules.
//!
//! The mechanics are the Action sheet's (read at runtime through `GameData::get_action_mechanics`): whether an action
//! is on the GCD (CooldownGroup 58), its recast, and the action it follows in a combo (ActionCombo). The rotation adds
//! what retail's recordings show: the NPC's pace, priorities, how often it re-applies a damage-over-time, which action
//! follows a buff. See `docs/src/fork/combat.md` ("Class rotations").

use std::collections::HashMap;
use std::time::{Duration, Instant};

use kawari::common::{ObjectId, Rotation, RotationAction};

/// The GCD's recast group in the Action sheet.
pub const GCD_GROUP: u8 = 58;
/// How long a combo stays open after its last step: 30 s since 7.0 (as for players.)
pub const COMBO_WINDOW: Duration = Duration::from_secs(30);
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
    /// It's a combo's step (it follows one, or one follows it): only these carry a combo on or break it, as retail's
    /// (a Gunbreaker's Gnashing Fang or Burst Strike between Brutal Shell and Solid Barrel leaves the combo be; retail's
    /// Gunbreakers at 80-89 in Bob's captures used Keen Edge 279 times, Brutal Shell 268 and Solid Barrel 253 with 123
    /// Gnashing Fangs, 75 Burst Strikes and 60 Sonic Breaks among them.)
    pub combo: bool,
}

/// Where an NPC is in its rotation.
#[derive(Debug, Clone, Default)]
pub struct RotationState {
    /// Its last GCD action and when.
    last_gcd: Option<(u32, Instant)>,
    /// The GCD action before that one and when.
    previous_gcd: Option<(u32, Instant)>,
    /// Its last combo step and the one before (`ActionMechanics::combo`): what the last one continues
    /// (`continues_combo`.)
    last_combo: Option<(u32, Instant)>,
    previous_combo: Option<(u32, Instant)>,
    /// When each action was last used.
    used: HashMap<u32, Instant>,
    /// When each action was last used on each target (for `refresh`.)
    used_on: HashMap<(u32, ObjectId), Instant>,
    /// A GCD action that has to come next (an entry's `then`.)
    next: Option<u32>,
    /// An ability was used since the last GCD action.
    weaved: bool,
    /// How many abilities since the last GCD action (party bots' rotation scripts weave up to two.)
    weaves: u8,
}

/// What the NPC's situation is when it's choosing.
pub struct Situation {
    pub now: Instant,
    pub target: ObjectId,
    /// The target's and its own health, in %.
    pub target_hp: f32,
    pub self_hp: f32,
    /// The NPCs on its own side by BNpcBase, one of each (the lowest layout ID), for entries used `on_ally`.
    pub allies: HashMap<u32, ObjectId>,
}

impl Situation {
    /// Whom an entry is used on: its ally for one `on_ally`, otherwise the NPC's target.
    pub fn target_for(&self, entry: &RotationAction) -> Option<ObjectId> {
        match entry.on_ally {
            Some(base) => self.allies.get(&base).copied(),
            None => Some(self.target),
        }
    }
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
        // A kit with nothing on the GCD (NPC actions in no recast group, such as the corsair thaumaturges' Fire, 966)
        // isn't woven between GCD actions it doesn't have: each action goes when its own cooldown allows, at most
        // once per `pace`. Kept to the weaving rule, they never acted (Bob's bundle of 2026-10-22; retail cast Fire
        // every 3.1 s.)
        let no_gcd = !rotation
            .actions
            .iter()
            .any(|entry| mechanics(entry.action).is_some_and(|action| action.gcd));

        for entry in &rotation.actions {
            let Some(action) = mechanics(entry.action) else {
                continue;
            };
            // One used on an ally waits for that ally to be there.
            let Some(target) = situation.target_for(entry) else {
                continue;
            };
            if !no_gcd && (action.gcd && !gcd_ready || !action.gcd && !can_weave) {
                continue;
            }
            // Its own cooldown: `every`, or the recast for abilities and for GCD actions with a longer one.
            let cooldown = if no_gcd {
                Some(entry.every.unwrap_or(action.recast.max(rotation.pace)))
            } else {
                entry
                    .every
                    .or((!action.gcd || action.recast > rotation.pace).then_some(action.recast))
            };
            if let Some(cooldown) = cooldown
                && self.used.get(&entry.action).is_some_and(|at| {
                    now.duration_since(*at) < Duration::from_secs_f32(cooldown.max(0.0))
                })
            {
                continue;
            }
            if let Some(refresh) = entry.refresh
                && self.used_on.get(&(entry.action, target)).is_some_and(|at| {
                    now.duration_since(*at) < Duration::from_secs_f32(refresh.max(0.0))
                })
            {
                continue;
            }
            // The next step of a combo only follows the step before it, when the NPC has that step at all: retail's
            // NPCs use a combo's later steps on their own when their kit lacks the step before (Underneath the
            // Sultantree's stranger: Rage of Halone, whose ActionCombo is Riot Blade, without Riot Blade.)
            if action.gcd
                && action.combo_from != 0
                && rotation
                    .actions
                    .iter()
                    .any(|other| other.action == action.combo_from)
            {
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

    /// When it last used `action`.
    pub fn used_at(&self, action: u32) -> Option<Instant> {
        self.used.get(&action).copied()
    }

    /// Its last GCD action and when.
    pub fn last_gcd(&self) -> Option<(u32, Instant)> {
        self.last_gcd
    }

    /// Abilities used since its last GCD action.
    pub fn weaves(&self) -> u8 {
        self.weaves
    }

    /// Its last combo step and when (GCD actions outside combos leave it be.)
    pub fn last_combo(&self) -> Option<(u32, Instant)> {
        self.last_combo
    }

    /// Whether its last combo step, `action`, carries on a combo from `combo_from` (the step before it, within
    /// `COMBO_WINDOW`), so it gets the step's combo potency as a player's would. The choice is noted (`record`) before
    /// the action goes off, so the step before is the combo step before the last.
    pub fn continues_combo(&self, action: u32, combo_from: u32, now: Instant) -> bool {
        combo_from != 0
            && self.last_combo.is_some_and(|(last, _)| last == action)
            && self.previous_combo.is_some_and(|(previous, at)| {
                previous == combo_from && now.duration_since(at) <= COMBO_WINDOW
            })
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
        if gcd && mechanics.is_some_and(|m| m.combo) {
            self.previous_combo = self.last_combo;
            self.last_combo = Some((action, now));
        }
        if gcd {
            self.previous_gcd = self.last_gcd;
            self.last_gcd = Some((action, now));
            self.weaved = false;
            self.weaves = 0;
            if self.next == Some(action) {
                self.next = None;
            }
        } else {
            self.weaved = true;
            self.weaves = self.weaves.saturating_add(1);
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
                combo: true,
            },
            VORPAL_THRUST => ActionMechanics {
                gcd: true,
                recast: 2.5,
                combo_from: TRUE_THRUST,
                combo: true,
            },
            LEG_SWEEP => ActionMechanics {
                gcd: false,
                recast: 20.0,
                combo_from: 0,
                combo: false,
            },
            LIFE_SURGE => ActionMechanics {
                gcd: false,
                recast: 45.0,
                combo_from: 0,
                combo: false,
            },
            THUNDER => ActionMechanics {
                gcd: true,
                recast: 2.5,
                combo_from: 0,
                combo: false,
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
                allies: HashMap::new(),
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
    fn a_combo_step_knows_it_continues_one() {
        let rotation = lancer();
        let start = Instant::now();
        let mut state = RotationState::default();
        state.record(
            &rotation,
            TRUE_THRUST,
            ObjectId(1),
            start,
            mechanics(TRUE_THRUST),
        );
        assert!(!state.continues_combo(TRUE_THRUST, 0, start));
        let later = start + Duration::from_secs(3);
        // An ability between doesn't break it.
        state.record(
            &rotation,
            LIFE_SURGE,
            ObjectId(1),
            later,
            mechanics(LIFE_SURGE),
        );
        state.record(
            &rotation,
            VORPAL_THRUST,
            ObjectId(1),
            later,
            mechanics(VORPAL_THRUST),
        );
        assert!(state.continues_combo(VORPAL_THRUST, TRUE_THRUST, later));
        // Too long after its step, it doesn't.
        let mut late = RotationState::default();
        late.record(
            &rotation,
            TRUE_THRUST,
            ObjectId(1),
            start,
            mechanics(TRUE_THRUST),
        );
        let much_later = start + COMBO_WINDOW + Duration::from_secs(1);
        late.record(
            &rotation,
            VORPAL_THRUST,
            ObjectId(1),
            much_later,
            mechanics(VORPAL_THRUST),
        );
        assert!(!late.continues_combo(VORPAL_THRUST, TRUE_THRUST, much_later));
    }

    #[test]
    fn combo_step_without_its_step_stands_alone() {
        // Without True Thrust in its kit, Vorpal Thrust is used on its own, every GCD.
        let rotation = Rotation {
            pace: 3.0,
            actions: vec![RotationAction {
                action: VORPAL_THRUST,
                ..Default::default()
            }],
        };
        let used = simulate(&rotation, 10);
        assert!(used.len() >= 3, "{used:?}");
        assert!(used.iter().all(|(_, a)| *a == VORPAL_THRUST));
    }

    #[test]
    fn a_kit_without_gcd_actions_still_acts() {
        // Only an ability (Leg Sweep, 20 s recast) and a 3 s pace: it's used every time its recast allows, from the
        // start, as the corsair thaumaturges' Fire (no recast group) is on retail.
        let rotation = Rotation {
            pace: 3.0,
            actions: vec![RotationAction {
                action: LEG_SWEEP,
                ..Default::default()
            }],
        };
        let used = simulate(&rotation, 45);
        let times: Vec<f32> = used.iter().map(|(t, _)| *t).collect();
        assert_eq!(times, vec![0.0, 20.0, 40.0], "{used:?}");
        // With `every`, that instead.
        let rotation = Rotation {
            pace: 3.0,
            actions: vec![RotationAction {
                action: LEG_SWEEP,
                every: Some(3.0),
                ..Default::default()
            }],
        };
        assert_eq!(simulate(&rotation, 10).len(), 4);
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
            allies: HashMap::new(),
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
            allies: HashMap::new(),
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
            allies: HashMap::new(),
        };
        assert_eq!(state.choose(&finisher, &healthy, &mut mechanics), None);
        let low = Situation {
            now: start,
            target: ObjectId(1),
            target_hp: 10.0,
            self_hp: 100.0,
            allies: HashMap::new(),
        };
        assert_eq!(
            state.choose(&finisher, &low, &mut mechanics),
            Some(TRUE_THRUST)
        );
    }
    #[test]
    fn on_ally_waits_for_the_ally_and_goes_on_it() {
        // The Akoman's drain on a yodomi (quest battle 17): first while one is out, otherwise its usual action.
        const YODOMI: u32 = 1710;
        let rotation = Rotation {
            pace: 2.5,
            actions: vec![
                RotationAction {
                    action: THUNDER,
                    on_ally: Some(YODOMI),
                    every: Some(26.0),
                    ..Default::default()
                },
                RotationAction {
                    action: TRUE_THRUST,
                    ..Default::default()
                },
            ],
        };
        let state = RotationState::default();
        let start = Instant::now();
        let alone = Situation {
            now: start,
            target: ObjectId(1),
            target_hp: 100.0,
            self_hp: 100.0,
            allies: HashMap::new(),
        };
        assert_eq!(
            state.choose(&rotation, &alone, &mut mechanics),
            Some(TRUE_THRUST)
        );
        let with_yodomi = Situation {
            allies: HashMap::from([(YODOMI, ObjectId(7))]),
            ..alone
        };
        assert_eq!(
            state.choose(&rotation, &with_yodomi, &mut mechanics),
            Some(THUNDER)
        );
        assert_eq!(
            with_yodomi.target_for(&rotation.actions[0]),
            Some(ObjectId(7))
        );
        assert_eq!(
            with_yodomi.target_for(&rotation.actions[1]),
            Some(ObjectId(1))
        );
    }
}
