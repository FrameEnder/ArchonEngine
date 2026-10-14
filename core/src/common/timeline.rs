use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

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
}

impl Timeline {
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
                timepoints: vec![Timepoint {
                    time: 20,
                    data: TimepointData::Action { action_id: 872 }
                }]
            }
        );
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
