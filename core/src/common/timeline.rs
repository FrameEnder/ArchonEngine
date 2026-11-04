use std::collections::BTreeMap;

use serde_derive::{Deserialize, Serialize};

/// A JSON file that appends an existing (and usually empty) LGB.
///
/// These can only add gathering points for now.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Timeline {
    /// Which action is used for the auto-attack. Index into the Action Excel sheet.
    pub autoattack_action_id: u32,
    /// Whether the timeline always plays.
    pub timeline_always_plays: bool,
    /// The timeline points.
    pub timepoints: Vec<Timepoint>,
    /// The points' times count from the NPC's pull (its first target), to the half second, and each action is used
    /// when due whether its target is in reach or not, as retail's bosses keep their schedule: the Copperbell Mines'
    /// Kottos cast at 5.2, 15.3, 29.6 and 39.1 s from its pull in all three of Bob's runs, however long it stood
    /// before. Otherwise they count from its spawn in whole seconds, used only when in reach (upstream's.)
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub from_pull: bool,
    /// A series of actions (to play in sequence) on death.
    #[serde(default)]
    pub on_death: Vec<TimepointData>,
    /// How close (in yalms) this NPC gets to its target before attacking, e.g. further away for casters.
    #[serde(default = "default_attack_range")]
    pub attack_range: f32,
    /// How much this NPC's auto-attack hits players for on retail, by its level (an average, before critical hits.)
    /// Its other actions hit for this times their potency / 100. Levels in between or beyond are scaled by how
    /// ordinary enemies' health grows. Without any, NPC damage is worked out from its stats instead.
    #[serde(default)]
    pub damage_to_players: BTreeMap<u8, f32>,
    /// What this NPC's actions did on retail, by action ID: the average damage (or healing, for a heal) to a player
    /// and to another NPC, as Kawari Recorder's recording of a duty shows it. When there's one for the action and the
    /// kind of target, the hit is scaled to it (then varied, missed, parried or critical like any NPC hit), whatever
    /// the action's script and the NPC's stats would give. This is how allies and enemies in quest battles hurt (and
    /// heal) each other the way they do on retail.
    #[serde(default)]
    pub recorded_amounts: BTreeMap<u32, RecordedAmount>,
    /// This NPC's hitbox radius in yalms (as Kawari Recorder reads it on retail.) Its area attacks reach this much
    /// further, since they're measured from the edge of its hitbox.
    #[serde(default)]
    pub hitbox_radius: f32,
    /// The animation lock retail's ActionEffect carried for each of this NPC's actions, by action ID (the same every
    /// use: Bob's Copperbell Mines captures, e.g. 2.1 for its bosses' attacks, 2.3 or 2.4 for some trash abilities, 1.1
    /// for the helpers' markers). Its auto-attack is always 0.1; an action not here gets a player's lock.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub animation_locks: BTreeMap<u32, f32>,
    /// How this NPC fights with its class's actions, instead of (or as well as) fixed timepoints: see `Rotation`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<Rotation>,
}

/// A class rotation: which actions an NPC uses, in priority order, by the class's own rules. The mechanics come from
/// the Action sheet at runtime (whether an action is on the GCD, its recast, the action it follows in a combo), so an
/// entry is usually just an action ID; the rest is what retail's recordings show (`questkit battle` writes these.)
///
/// Each time the NPC can act, the first entry that's ready is used:
/// * a weaponskill or spell (CooldownGroup 58) when `pace` seconds have passed since its last one, only as the next
///   step of its combo (right after its ActionCombo, within 15 s) if it has one;
/// * an ability (off the GCD) once its recast (or `every`) has passed, at most one between two GCD actions;
/// * `refresh`: not again on the same target for that long (a damage-over-time or debuff, re-applied as it wears off);
/// * `then`: the GCD action that has to follow this one (Life Surge, then Vorpal Thrust);
/// * `target_hp_below` / `self_hp_below`: only then (in %).
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Rotation {
    /// Seconds between this NPC's weaponskills and spells. NPCs act far less often than players (retail's quest battle
    /// allies use one every 3-12 s); questkit reads it from a recording.
    #[serde(default = "default_pace")]
    pub pace: f32,
    pub actions: Vec<RotationAction>,
}

fn default_pace() -> f32 {
    2.5
}

/// One entry of a `Rotation`.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone, Default)]
pub struct RotationAction {
    pub action: u32,
    /// Use it at most this often, in seconds (instead of its recast.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub every: Option<f32>,
    /// Not again on the same target for this many seconds (its status's duration, less a little.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh: Option<f32>,
    /// The GCD action that must come next.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub then: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_hp_below: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub self_hp_below: Option<f32>,
    /// Use it on a living NPC of this BNpcBase on its own side (the lowest layout ID of them) instead of its target, and
    /// only while there is one: the Akoman's drain (1127) on the yodomi that feed it in quest battle 17 (Bob's
    /// Chronofoil capture df6f5af9.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_ally: Option<u32>,
    /// Where the numbers came from (ignored by the server.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

/// See `Timeline::recorded_amounts`.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone, Default)]
pub struct RecordedAmount {
    /// The average amount against players.
    #[serde(default)]
    pub players: Option<f32>,
    /// The average amount against other NPCs.
    #[serde(default)]
    pub npcs: Option<f32>,
    /// The average against players by the NPC's level, for a kind met at several levels: the tainted earth sprite
    /// (BNpcBase 294) hit for 7 at level 5 in Trial by Earth and 20 at level 20 in Sylphie's Trials (Bob's Chronofoil
    /// capture b23b91ef). Its level's comes first, then `players`, then the nearest level's.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub players_by_level: BTreeMap<u8, f32>,
}

impl Timeline {
    /// What retail's recording says this NPC's `action_id` does to a player (`to_player`) or another NPC, if known.
    pub fn recorded_amount(&self, action_id: u32, to_player: bool) -> Option<f32> {
        let recorded = self.recorded_amounts.get(&action_id)?;
        if to_player {
            recorded.players
        } else {
            recorded.npcs
        }
    }

    /// `recorded_amount` for an NPC of `level`: against players, its level's `players_by_level` when there is one.
    pub fn recorded_amount_at(&self, action_id: u32, to_player: bool, level: u8) -> Option<f32> {
        let recorded = self.recorded_amounts.get(&action_id)?;
        if !to_player {
            return recorded.npcs;
        }
        if let Some(amount) = recorded.players_by_level.get(&level) {
            return Some(*amount);
        }
        recorded.players.or_else(|| {
            recorded
                .players_by_level
                .iter()
                .min_by_key(|(recorded, _)| (**recorded as i32 - level as i32).abs())
                .map(|(_, amount)| *amount)
        })
    }

    /// The average auto-attack damage this NPC does to players at `level`, if it's known.
    ///
    /// `health_at` is ordinary enemies' health by level, used to scale from the nearest recorded level.
    pub fn auto_attack_damage(
        &self,
        level: u8,
        health_at: impl Fn(u8) -> Option<u32>,
    ) -> Option<f32> {
        if let Some(damage) = self.damage_to_players.get(&level) {
            return Some(*damage);
        }

        let (nearest, damage) = self
            .damage_to_players
            .iter()
            .min_by_key(|(recorded, _)| (**recorded as i32 - level as i32).abs())?;
        let ratio = health_at(level)? as f32 / health_at(*nearest)? as f32;
        Some(damage * ratio)
    }
}

fn default_attack_range() -> f32 {
    crate::common::MINIMUM_PATHFINDING_DISTANCE
}

impl Timeline {
    /// Duration of the entire timeline in seconds.
    pub fn duration(&self) -> i32 {
        let mut duration = 0;
        for point in &self.timepoints {
            duration = duration.max(point.time as i32);
        }

        duration
    }

    /// Returns all points happening at the specified time, if any.
    pub fn points_at(&self, point: i32) -> Vec<&Timepoint> {
        self.timepoints
            .iter()
            .filter(|x| x.time == point as f32)
            .collect()
    }

    /// A `from_pull` timeline's points due in the tick from `from` up to (not including) `to` seconds after the pull:
    /// each at its `time`, then every `every` seconds after it if it repeats.
    pub fn points_between(&self, from: f32, to: f32) -> Vec<&Timepoint> {
        self.points_between_after(from, to, None)
    }

    /// `points_between` for the clock started by `anchor` (an action with `restarts`; `None` is the pull.)
    pub fn points_between_after(&self, from: f32, to: f32, anchor: Option<u32>) -> Vec<&Timepoint> {
        self.timepoints
            .iter()
            .filter(|point| point.hp_below.is_none() && point.after == anchor)
            .filter(|point| match point.every {
                Some(every) if every > 0.0 => {
                    let since = (from - point.time).max(0.0);
                    let next = point.time + (since / every).ceil() * every;
                    next >= from && next < to
                }
                _ => point.time >= from && point.time < to,
            })
            .collect()
    }
}

/// Represents a point on the timeline.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Timepoint {
    /// Seconds: from the NPC's spawn (whole), or from its pull with `Timeline::from_pull` (to the half second.)
    pub time: f32,
    /// With `Timeline::from_pull`: it comes again every this many seconds after `time` (Kottos' 28545 every 81 s,
    /// its 28543, 28546 and 28547 every 37), each point keeping its own period.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub every: Option<f32>,
    /// With `Timeline::from_pull`: its time counts from the last use of this action (one with `restarts`) instead of
    /// the pull, and it only comes while that's the last such: the Copperbell Mines' second boss's 28462 came 22.3 s
    /// after each of its 28463s, and its opening 28461 never once a 28463 had come first (Bob's three runs.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<u32>,
    /// With `Timeline::from_pull`: instead of a time, it comes once, as soon as the NPC's health is at or below this
    /// percentage (not mid-cast): the second boss's 28463 at 80, 50 and 25 % (75.5-76.9, 42.6-49.8 and 14.8-24.6 % as
    /// it began, in the three runs, its timing different in each.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hp_below: Option<f32>,
    /// Its use starts the clock again for the points `after` it (see `after`.)
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub restarts: bool,
    pub data: TimepointData,
}

/// Whom a timeline's action is used on.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone, Copy, Default)]
#[serde(rename_all = "snake_case")]
pub enum TimepointTarget {
    /// Its target (the one it's fighting.)
    #[default]
    Target,
    /// Someone else on its enmity list, at random (Kottos' 28547: a healer, a ninja, a lancer in the three runs,
    /// never the one it was fighting.) Its target when there's no one else.
    Other,
    /// Itself (an area around it, cast on itself: Kottos' 28546.)
    Itself,
    /// No one (0xE0000000), cast where it stands: Kottos' 28543.
    Nobody,
}

/// Which way an action cast on itself (`TimepointTarget::Itself`, an area in front of or around it) goes.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone, Copy, Default)]
#[serde(rename_all = "snake_case")]
pub enum TimepointFacing {
    /// Towards the one it's fighting.
    #[default]
    Target,
    /// Towards the nearest player, whoever it's fighting: The Lominsan Way's 1769 aimed every Rive (1299, a line) at
    /// the player, within a degree, while it fought the one it was there to kill (Bob's Chronofoil captures 44df02fc and
    /// c9145fdc: 19 casts in seven runs.)
    Player,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
#[serde(tag = "type")]
pub enum TimepointData {
    /// Represents doing an action.
    #[serde(rename = "action")]
    Action {
        /// Index into the Action Excel sheet.
        action_id: u32,
        /// A lock-on marker (Lockon sheet row) put over the target as the cast begins, as retail marks a tank-buster's
        /// target (218 on the Copperbell Mines' three bosses' busters: ActorControl 34 from the target.)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        lockon: Option<u32>,
        /// Whom it's used on (its target unless said.)
        #[serde(default, skip_serializing_if = "is_default_target")]
        target: TimepointTarget,
        /// For one cast on itself, which way it faces (its target unless said.)
        #[serde(default, skip_serializing_if = "is_default_facing")]
        facing: TimepointFacing,
    },
    /// Animates timelines for the gimmick this actor is bound to, such as Giant Clams.
    #[serde(rename = "timeline_state")]
    TimelineState { states: Vec<u32> },
    /// Changes the invulnerability state of this NPC.
    #[serde(rename = "invulnerability")]
    Invulnerability { invulnerable: bool },
}

fn is_default_target(target: &TimepointTarget) -> bool {
    *target == TimepointTarget::Target
}

fn is_default_facing(facing: &TimepointFacing) -> bool {
    *facing == TimepointFacing::Target
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorded_amount_by_level() {
        // The tainted earth sprite: 7 at level 5 (Trial by Earth), 21 at level 20 (Sylphie's Trials.)
        let timeline: Timeline = serde_json::from_str(
            r#"{"autoattack_action_id": 0, "timeline_always_plays": false, "timepoints": [],
                "recorded_amounts": {"970": {"players": 7.0, "npcs": 3.0, "players_by_level": {"5": 7.0, "20": 21.0}},
                                     "971": {"players_by_level": {"15": 16.0, "20": 20.0}}}}"#,
        )
        .unwrap();
        assert_eq!(timeline.recorded_amount_at(970, true, 20), Some(21.0));
        assert_eq!(timeline.recorded_amount_at(970, true, 12), Some(7.0)); // not recorded: `players`
        assert_eq!(timeline.recorded_amount_at(970, false, 20), Some(3.0));
        assert_eq!(timeline.recorded_amount_at(971, true, 18), Some(20.0)); // no `players`: the nearest level's
        assert_eq!(timeline.recorded_amount_at(1, true, 20), None);
    }

    #[test]
    fn test_simple_example() {
        let json =
            std::fs::read_to_string("../resources/data/tests/example_timeline.json").unwrap();
        let timeline: Timeline = serde_json::from_str(&json).unwrap();

        assert_eq!(
            timeline,
            Timeline {
                autoattack_action_id: 872,
                timeline_always_plays: false,
                on_death: Vec::default(),
                attack_range: crate::common::MINIMUM_PATHFINDING_DISTANCE,
                damage_to_players: BTreeMap::default(),
                recorded_amounts: BTreeMap::default(),
                hitbox_radius: 0.0,
                animation_locks: BTreeMap::default(),
                rotation: None,
                from_pull: false,
                timepoints: vec![Timepoint {
                    time: 20.0,
                    every: None,
                    after: None,
                    hp_below: None,
                    restarts: false,
                    data: TimepointData::Action {
                        action_id: 872,
                        lockon: None,
                        target: TimepointTarget::Target,
                        facing: TimepointFacing::Target,
                    }
                }]
            }
        );
    }

    /// Every timeline in resources/timelines reads, and the Copperbell Mines' last boss marks its tank-buster's target.
    #[test]
    fn every_timeline_reads() {
        for entry in std::fs::read_dir("../resources/timelines")
            .unwrap()
            .flatten()
        {
            let json = std::fs::read_to_string(entry.path()).unwrap();
            let timeline: Result<Timeline, _> = serde_json::from_str(&json);
            assert!(timeline.is_ok(), "{:?}: {:?}", entry.path(), timeline.err());
        }
        let json =
            std::fs::read_to_string("../resources/timelines/CopperbellBoss3_14537.json").unwrap();
        let timeline: Timeline = serde_json::from_str(&json).unwrap();
        assert!(timeline.timepoints.iter().any(|point| matches!(
            point.data,
            TimepointData::Action {
                action_id: 28764,
                lockon: Some(218),
                ..
            }
        )));
    }

    /// Kottos' timeline gives retail's schedule: the ticks its casts begin in, from its pull (Bob's three Copperbell
    /// Mines runs: 5.2, 15.3, 29.6, 39.1, 52.3, 66.6, 76.0, 86.2, 99.4.)
    #[test]
    fn kottos_schedule() {
        let json =
            std::fs::read_to_string("../resources/timelines/CopperbellBoss1_14460.json").unwrap();
        let timeline: Timeline = serde_json::from_str(&json).unwrap();
        assert!(timeline.from_pull);
        let mut casts = Vec::new();
        for tick in 0..200 {
            let from = tick as f32 * 0.5;
            for point in timeline.points_between(from, from + 0.5) {
                if let TimepointData::Action { action_id, .. } = point.data {
                    casts.push((from, action_id));
                }
            }
        }
        assert_eq!(
            casts,
            vec![
                (5.0, 28545),
                (15.0, 28543),
                (29.5, 28546),
                (39.0, 28547),
                (52.0, 28543),
                (66.5, 28546),
                (76.0, 28547),
                (86.0, 28545),
                (99.0, 28543),
            ]
        );
    }

    /// The second boss's points after the pull only while no 28463 has come, and its 28462 22 s after each.
    #[test]
    fn second_boss_restarts() {
        let json =
            std::fs::read_to_string("../resources/timelines/CopperbellBoss2_14448.json").unwrap();
        let timeline: Timeline = serde_json::from_str(&json).unwrap();
        let actions = |points: Vec<&Timepoint>| -> Vec<u32> {
            points
                .iter()
                .filter_map(|point| match point.data {
                    TimepointData::Action { action_id, .. } => Some(action_id),
                    _ => None,
                })
                .collect()
        };
        assert_eq!(
            actions(timeline.points_between_after(6.0, 6.5, None)),
            vec![28462]
        );
        assert_eq!(
            actions(timeline.points_between_after(17.5, 18.0, None)),
            vec![28461]
        );
        assert!(
            timeline
                .points_between_after(17.5, 18.0, Some(28463))
                .is_empty()
        );
        assert_eq!(
            actions(timeline.points_between_after(22.0, 22.5, Some(28463))),
            vec![28462]
        );
        assert_eq!(
            timeline
                .timepoints
                .iter()
                .filter(|point| point.hp_below.is_some() && point.restarts)
                .count(),
            3
        );
    }

    /// A pull-relative timeline's points come at their time, then every `every` seconds after it.
    #[test]
    fn points_every_so_often() {
        let timeline: Timeline = serde_json::from_str(
            r#"{"autoattack_action_id": 870, "timeline_always_plays": false, "from_pull": true, "timepoints": [
                {"time": 5.2, "every": 81, "data": {"type": "action", "action_id": 28545}},
                {"time": 15.3, "every": 37, "data": {"type": "action", "action_id": 28543, "target": "nobody"}}
            ]}"#,
        )
        .unwrap();
        let due = |from: f32| -> Vec<u32> {
            timeline
                .points_between(from, from + 0.5)
                .iter()
                .filter_map(|point| match point.data {
                    TimepointData::Action { action_id, .. } => Some(action_id),
                    _ => None,
                })
                .collect()
        };
        assert_eq!(due(5.0), vec![28545]);
        assert_eq!(due(5.5), Vec::<u32>::new());
        assert_eq!(due(15.0), vec![28543]);
        assert_eq!(due(52.0), vec![28543]);
        assert_eq!(due(86.0), vec![28545]);
        assert_eq!(due(0.0), Vec::<u32>::new());
    }

    #[test]
    fn recorded_amounts_by_target() {
        let timeline: Timeline = serde_json::from_str(
            r#"{"autoattack_action_id": 873, "timeline_always_plays": false, "timepoints": [],
                "recorded_amounts": {"873": {"npcs": 15.4}, "120": {"players": 95.0, "npcs": 87.0}}}"#,
        )
        .unwrap();
        assert_eq!(timeline.recorded_amount(873, false), Some(15.4));
        assert_eq!(timeline.recorded_amount(873, true), None);
        assert_eq!(timeline.recorded_amount(120, true), Some(95.0));
        assert_eq!(timeline.recorded_amount(7, false), None);
    }

    #[test]
    fn auto_attack_damage_scales_from_the_nearest_level() {
        let timeline: Timeline = serde_json::from_str(
            r#"{"autoattack_action_id": 870, "timeline_always_plays": false, "timepoints": [],
                "damage_to_players": {"10": 8.5, "12": 12.4}}"#,
        )
        .unwrap();
        let health = |level: u8| Some(level as u32 * 10);

        assert_eq!(timeline.auto_attack_damage(10, health), Some(8.5));
        assert_eq!(
            timeline.auto_attack_damage(13, health),
            Some(12.4 * 130.0 / 120.0)
        );
        assert_eq!(
            timeline.auto_attack_damage(5, health),
            Some(8.5 * 50.0 / 100.0)
        );

        let none: Timeline = serde_json::from_str(
            r#"{"autoattack_action_id": 870, "timeline_always_plays": false, "timepoints": []}"#,
        )
        .unwrap();
        assert_eq!(none.auto_attack_damage(10, health), None);
    }
}
