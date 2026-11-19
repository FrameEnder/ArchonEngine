//! How an enemy takes a player's status a second and third time, as retail's did
//! (`resources/data/status_resistance.json`, read from Bob's and Connor's Chronofoil captures on 2026-11-18.)
//!
//! Kawari gave every status in full every time, so a White Mage's Holy (Stun, 4 s, every 2.5 s) held a pack still for
//! as long as it was cast: at Cape Westwind the four that come for the boss stood stunned from 273 s to 307 s and
//! "didn't attack for awhile" (Bob, 2026-11-18). Retail's four refuse Stun outright (effect 55 with the status's ID
//! where effect 14 would give it, 59 of 59), and any other enemy takes it for the full time, half, a quarter, and then
//! not at all.

use std::{
    collections::HashMap,
    sync::OnceLock,
    time::{Duration, Instant},
};

use kawari::common::ObjectId;
use serde::Deserialize;

/// How one status is taken again: the share of its time each use gets, how old a use is before it counts towards the
/// next, and how long after the last that took hold it's all forgotten.
#[derive(Debug, Clone, Deserialize)]
pub struct Diminishing {
    pub steps: Vec<f32>,
    pub counted_after: f32,
    pub forget_after: f32,
}

#[derive(Debug, Default, Deserialize)]
struct Table {
    #[serde(default)]
    immune: HashMap<String, Vec<u32>>,
    #[serde(default)]
    diminishing: HashMap<String, Diminishing>,
}

fn table() -> &'static Table {
    static TABLE: OnceLock<Table> = OnceLock::new();
    TABLE.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../../../resources/data/status_resistance.json"
        ))
        .unwrap_or_default()
    })
}

/// What became of a status given to an enemy.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Taken {
    /// For this long (its own time, or a share of it.)
    For(f32),
    /// Not at all: the action shows the status with "no effect" (effect 55.)
    Refused,
}

/// When each enemy last took each status (`Instance::status_resistance`.)
#[derive(Debug, Default)]
pub struct Resistance {
    taken: HashMap<(ObjectId, u16), Vec<Instant>>,
}

impl Resistance {
    /// The enemy `target` (of BNpcBase `base_id`) is given `status` for `duration` seconds at `now`.
    pub fn take(
        &mut self,
        base_id: u32,
        target: ObjectId,
        status: u16,
        duration: f32,
        now: Instant,
    ) -> Taken {
        let table = table();
        let key = status.to_string();
        if table
            .immune
            .get(&key)
            .is_some_and(|bases| bases.contains(&base_id))
        {
            return Taken::Refused;
        }
        let Some(rule) = table.diminishing.get(&key) else {
            return Taken::For(duration);
        };
        let taken = self.taken.entry((target, status)).or_default();
        if taken.last().is_some_and(|last| {
            now.duration_since(*last) >= Duration::from_secs_f32(rule.forget_after)
        }) {
            taken.clear();
        }
        let counted = taken
            .iter()
            .filter(|at| now.duration_since(**at) >= Duration::from_secs_f32(rule.counted_after))
            .count();
        match rule.steps.get(counted) {
            Some(share) => {
                taken.push(now);
                Taken::For(duration * share)
            }
            None => Taken::Refused,
        }
    }

    /// An actor gone: nothing of it is kept.
    pub fn forget(&mut self, target: ObjectId) {
        self.taken.retain(|(id, _), _| *id != target);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STUN: u16 = 2;

    /// Retail's runs of Holy on one enemy (Bob's Chronofoil capture 50e72631; seconds, then the Stun's time or its
    /// refusal): "0.0 4, 7.6 2, 12.5 1, 15.0 refused", and with two 2.5 s apart "0.0 4, 2.5 4, 10.0 1, 12.4 1, 15.0
    /// refused".
    #[test]
    fn stun_is_taken_for_less_each_time_and_then_not_at_all() {
        let start = Instant::now();
        let at = |seconds: f32| start + Duration::from_secs_f32(seconds);
        let enemy = ObjectId(0x4000_1000);
        let mut resistance = Resistance::default();
        for (seconds, taken) in [
            (0.0, Taken::For(4.0)),
            (7.6, Taken::For(2.0)),
            (12.5, Taken::For(1.0)),
            (15.0, Taken::Refused),
            (17.4, Taken::Refused),
            (60.0, Taken::Refused),
        ] {
            assert_eq!(
                resistance.take(14576, enemy, STUN, 4.0, at(seconds)),
                taken,
                "{seconds}"
            );
        }
        let other = ObjectId(0x4000_1001);
        for (seconds, taken) in [
            (0.0, Taken::For(4.0)),
            (2.4, Taken::For(4.0)),
            (10.0, Taken::For(1.0)),
            (12.4, Taken::For(1.0)),
            (15.0, Taken::Refused),
        ] {
            assert_eq!(
                resistance.take(14572, other, STUN, 4.0, at(seconds)),
                taken,
                "{seconds}"
            );
        }
        // Forgotten a while after the last that took hold (a guess: retail's wasn't seen to end.)
        assert_eq!(
            resistance.take(14572, other, STUN, 4.0, at(80.0)),
            Taken::For(4.0)
        );
        // A status the table doesn't name is given in full every time.
        for seconds in [0.0, 1.0, 2.0, 30.0] {
            assert_eq!(
                resistance.take(14572, other, 9, 20.0, at(seconds)),
                Taken::For(20.0)
            );
        }
    }

    /// Cape Westwind's four refused every Stun, the first included (59 of 59 in Bob's and Connor's captures.)
    #[test]
    fn some_enemies_refuse_stun_outright() {
        let mut resistance = Resistance::default();
        let now = Instant::now();
        // Cape Westwind's (59 of 59), and the two that refused their only one: Castrum Meridianum's last boss and
        // the Praetorium's last boss's four (Kawari Witness's `refuses:2`, Bob's captures deea324c and 6320ae56.)
        for base in [14582, 14583, 14542, 14455] {
            assert_eq!(
                resistance.take(base, ObjectId(0x4000_2000), STUN, 4.0, now),
                Taken::Refused
            );
        }
    }
}
