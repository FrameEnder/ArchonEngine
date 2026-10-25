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
            duration = duration.max(point.time);
        }

        duration
    }

    /// Returns all points happening at the specified time, if any.
    pub fn points_at(&self, point: i32) -> Vec<&Timepoint> {
        self.timepoints.iter().filter(|x| x.time == point).collect()
    }
}

/// Represents a point on the timeline.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Timepoint {
    pub time: i32,
    pub data: TimepointData,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
#[serde(tag = "type")]
pub enum TimepointData {
    /// Represents doing an action.
    #[serde(rename = "action")]
    Action {
        /// Index into the Action Excel sheet.
        action_id: u32,
    },
    /// Animates timelines for the gimmick this actor is bound to, such as Giant Clams.
    #[serde(rename = "timeline_state")]
    TimelineState { states: Vec<u32> },
    /// Changes the invulnerability state of this NPC.
    #[serde(rename = "invulnerability")]
    Invulnerability { invulnerable: bool },
}

#[cfg(test)]
mod tests {
    use super::*;

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
                timepoints: vec![Timepoint {
                    time: 20,
                    data: TimepointData::Action { action_id: 872 }
                }]
            }
        );
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
