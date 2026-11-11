//! Questing world bots (stage 2 of [World Bots](../../../../docs/src/fork/world-bots.md)): bots shown as players that
//! level from 1 to 100 by doing quests, as players do, along the quest paths of Questionable
//! (<https://github.com/PunishXIV/Questionable>, AGPL-3.0 as Kawari is). Its files hold the game's text (quest and NPC
//! names in their file names and comments), so none of it is in git: the server reads a checkout of it at runtime
//! (`$KAWARI_DATA/questionable`, which ArchonEngine ControlPanel downloads), as it reads the game's files.
//!
//! * `QuestPaths`: each quest's steps from Questionable (territory, position, the NPC or object, what's done there.)
//! * `Routes`: the quests a bot levels through, from the game's Quest sheet: its level, the quests before it, and only
//!   those any class can take, done in towns and the open world (a duty or solo duty is time away.)
//! * `Questing`: a bot's place on its route; `world_bots` walks, teleports, talks, fights and levels it.

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use glam::Vec3A;
use icarus::Quest::QuestSheet;
use kawari::common::TerritoryIntendedUse;
use serde_json::Value;

use crate::GameData;

/// Where Questionable's checkout is read from: `$KAWARI_DATA/questionable` (the container's `/kawari/data`), or
/// `questionable` in the working directory.
pub fn questionable_path() -> PathBuf {
    match std::env::var_os("KAWARI_DATA") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir).join("questionable"),
        _ => PathBuf::from("questionable"),
    }
}

/// What's done at a step (Questionable's InteractionType, read to what a bot shows of it.)
#[derive(Debug, Clone, PartialEq)]
pub enum StepKind {
    /// Talking to someone or using something there: AcceptQuest, Interact, CompleteQuest, attuning, and the like.
    Talk,
    /// Just getting there.
    WalkTo,
    /// Killing these enemies (BNpcBase IDs; none listed for those the quest spawns itself.)
    Combat(Vec<u32>),
    /// A duty (time away, as in the Duty Finder.)
    Duty,
    /// A solo duty or quest battle (time away.)
    SoloDuty,
    /// An emote at someone.
    Emote,
    /// Anything else (an item used, a line said, a jump): a moment there.
    Other,
}

impl StepKind {
    fn of(interaction: &str, step: &Value) -> Self {
        match interaction {
            "AcceptQuest"
            | "CompleteQuest"
            | "Interact"
            | "AttuneAetheryte"
            | "AttuneAethernetShard"
            | "AttuneAetherCurrent"
            | "RegisterFreeOrFavoredAetheryte"
            | "PurchaseItem" => StepKind::Talk,
            "WalkTo" => StepKind::WalkTo,
            "Combat" => {
                let mut enemies: Vec<u32> = step
                    .get("KillEnemyDataIds")
                    .and_then(Value::as_array)
                    .map(|ids| {
                        ids.iter()
                            .filter_map(Value::as_u64)
                            .map(|id| id as u32)
                            .collect()
                    })
                    .unwrap_or_default();
                if let Some(complex) = step.get("ComplexCombatData").and_then(Value::as_array) {
                    enemies.extend(
                        complex
                            .iter()
                            .filter_map(|enemy| enemy.get("DataId").and_then(Value::as_u64))
                            .map(|id| id as u32),
                    );
                }
                enemies.sort();
                enemies.dedup();
                StepKind::Combat(enemies)
            }
            "Duty" => StepKind::Duty,
            "SinglePlayerDuty" => StepKind::SoloDuty,
            "Emote" => StepKind::Emote,
            _ => StepKind::Other,
        }
    }
}

/// One step of a quest's path.
#[derive(Debug, Clone, PartialEq)]
pub struct QStep {
    pub sequence: u8,
    pub territory: u16,
    pub at: Option<Vec3A>,
    /// The NPC, object or enemy there (ENpcBase, EObj or BNpcBase ID; 0 for none.)
    pub data_id: u32,
    pub kind: StepKind,
}

/// Every quest's steps from Questionable, by quest ID (the Quest row less 65536, as its file names have it.)
#[derive(Debug, Default)]
pub struct QuestPaths {
    pub quests: HashMap<u32, Vec<QStep>>,
}

impl QuestPaths {
    /// Reads every quest file under `dir` (a checkout: its `QuestPaths` folder, or `dir` itself.) Files are found by
    /// their name's number; nothing of their names or text is kept.
    pub fn load(dir: &Path) -> Self {
        let root = if dir.join("QuestPaths").is_dir() {
            dir.join("QuestPaths")
        } else {
            dir.to_path_buf()
        };
        let mut paths = QuestPaths::default();
        let mut folders = vec![root];
        while let Some(folder) = folders.pop() {
            let Ok(entries) = std::fs::read_dir(&folder) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    folders.push(path);
                    continue;
                }
                let Some(id) = quest_id_of(&path) else {
                    continue;
                };
                let Some(steps) = std::fs::read_to_string(&path)
                    .ok()
                    .and_then(|text| {
                        serde_json::from_str::<Value>(text.trim_start_matches('\u{feff}')).ok()
                    })
                    .and_then(|value| Self::parse(&value))
                else {
                    continue;
                };
                paths.quests.insert(id, steps);
            }
        }
        paths
    }

    /// A quest file's steps, in order (its QuestSequence's steps, sequence by sequence.)
    pub fn parse(value: &Value) -> Option<Vec<QStep>> {
        let mut steps = Vec::new();
        for sequence in value.get("QuestSequence")?.as_array()? {
            let number = sequence
                .get("Sequence")
                .and_then(Value::as_u64)
                .unwrap_or(0) as u8;
            for step in sequence
                .get("Steps")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let Some(territory) = step.get("TerritoryId").and_then(Value::as_u64) else {
                    continue;
                };
                let at = step.get("Position").and_then(|position| {
                    Some(Vec3A::new(
                        position.get("X")?.as_f64()? as f32,
                        position.get("Y")?.as_f64()? as f32,
                        position.get("Z")?.as_f64()? as f32,
                    ))
                });
                let interaction = step
                    .get("InteractionType")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                steps.push(QStep {
                    sequence: number,
                    territory: territory as u16,
                    at,
                    data_id: step.get("DataId").and_then(Value::as_u64).unwrap_or(0) as u32,
                    kind: StepKind::of(interaction, step),
                });
            }
        }
        (!steps.is_empty()).then_some(steps)
    }
}

/// A quest file's quest ID: the number its name starts with (`551_….json` is quest 551, Quest row 66087.)
fn quest_id_of(path: &Path) -> Option<u32> {
    if path.extension()? != "json" {
        return None;
    }
    let name = path.file_stem()?.to_str()?;
    let digits: String = name.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() || !name[digits.len()..].starts_with('_') {
        return None;
    }
    digits.parse().ok()
}

/// A quest a bot can level through.
#[derive(Debug, Clone, PartialEq)]
pub struct QuestInfo {
    pub id: u32,
    pub level: u8,
    /// The quests before it (quest IDs.)
    pub previous: Vec<u32>,
    /// Its reward items, for the gear a bot has had from it (the always-given and the ones to choose from.)
    pub rewards: Vec<u32>,
}

/// The quests bots level through, by level.
#[derive(Debug, Default)]
pub struct Routes {
    pub quests: Vec<QuestInfo>,
    pub by_id: HashMap<u32, usize>,
}

impl Routes {
    /// The quests of `paths` a bot can do: in the Quest sheet with a level of 1-100, any class's (ClassJobCategory 1
    /// or none), not repeatable, not a festival's, a beast tribe's, a delivery, a daily pool's, a Grand Company's
    /// or one needing a house, and with every step in a town or the open world, a duty or a solo duty.
    pub fn plan(paths: &QuestPaths, game_data: &mut GameData) -> Self {
        let mut uses: HashMap<u16, bool> = HashMap::new();
        let mut quests = Vec::new();
        for (id, steps) in &paths.quests {
            let Some(row) = game_data.quest_sheet.row(QuestSheet::STARTING_ROW + id) else {
                continue;
            };
            let level = row.ClassJobLevel[0];
            if !(1..=100).contains(&level)
                || row.ClassJobCategory0 > 1
                || row.IsRepeatable
                || row.Festival != 0
                || row.BeastTribe != 0
                || row.DeliveryQuest != 0
                || row.DailyQuestPool != 0
                || row.GrandCompany != 0
                || row.IsHouseRequired
            {
                continue;
            }
            let walkable = steps.iter().all(|step| {
                matches!(step.kind, StepKind::Duty | StepKind::SoloDuty)
                    || *uses.entry(step.territory).or_insert_with(|| {
                        matches!(
                            game_data.get_intended_use(step.territory as u32),
                            Some(TerritoryIntendedUse::Town | TerritoryIntendedUse::Overworld)
                        )
                    })
            });
            if !walkable {
                continue;
            }
            let previous = row
                .PreviousQuest
                .iter()
                .filter(|quest| **quest > QuestSheet::STARTING_ROW)
                .map(|quest| quest - QuestSheet::STARTING_ROW)
                .collect();
            let rewards = row
                .Reward
                .iter()
                .chain(row.OptionalItemReward.iter())
                .copied()
                .filter(|item| *item != 0)
                .collect();
            quests.push(QuestInfo {
                id: *id,
                level: level as u8,
                previous,
                rewards,
            });
        }
        Self::from_quests(quests)
    }

    pub fn from_quests(mut quests: Vec<QuestInfo>) -> Self {
        quests.sort_by_key(|quest| (quest.level, quest.id));
        let by_id = quests
            .iter()
            .enumerate()
            .map(|(index, quest)| (quest.id, index))
            .collect();
        Routes { quests, by_id }
    }

    pub fn get(&self, id: u32) -> Option<&QuestInfo> {
        self.by_id.get(&id).map(|index| &self.quests[*index])
    }

    /// A quest to start a route of `levels` on: one of the band's first levels whose first step nobody's on, with
    /// fewer than `per_quest` bots on it (`busy`: quest ID → how many.)
    pub fn start(
        &self,
        levels: (u8, u8),
        busy: &HashMap<u32, usize>,
        per_quest: usize,
        step_taken: impl Fn(u32, usize) -> bool,
    ) -> Option<u32> {
        let low = levels.0;
        let high = (levels.0 + 2).min(levels.1);
        let choices: Vec<u32> = self
            .quests
            .iter()
            .filter(|quest| (low..=high).contains(&quest.level))
            .filter(|quest| busy.get(&quest.id).copied().unwrap_or(0) < per_quest)
            .filter(|quest| !step_taken(quest.id, 0))
            .map(|quest| quest.id)
            .collect();
        (!choices.is_empty()).then(|| choices[fastrand::usize(..choices.len())])
    }

    /// The next quest after `done` for a bot at `level` within `levels`, not among `done_before`: the quest that follows
    /// it (has it before it) most often, else one of its level or the next few near where it is (`near`: its first
    /// step's territory is `territory`) or anywhere; none with `per_quest` bots on it already. None at the band's end.
    #[allow(clippy::too_many_arguments)]
    pub fn next(
        &self,
        done: u32,
        level: u8,
        levels: (u8, u8),
        done_before: &HashSet<u32>,
        busy: &HashMap<u32, usize>,
        per_quest: usize,
        first_territory: impl Fn(u32) -> Option<u16>,
        territory: u16,
    ) -> Option<u32> {
        let open = |quest: &&QuestInfo| {
            quest.level <= levels.1
                && quest.level >= levels.0
                && !done_before.contains(&quest.id)
                && quest.id != done
                && busy.get(&quest.id).copied().unwrap_or(0) < per_quest
        };
        let following: Vec<u32> = self
            .quests
            .iter()
            .filter(open)
            .filter(|quest| quest.previous.contains(&done) && quest.level <= level + 2)
            .map(|quest| quest.id)
            .collect();
        if !following.is_empty() && fastrand::f32() < 0.8 {
            return Some(following[fastrand::usize(..following.len())]);
        }
        let near_level: Vec<&QuestInfo> = self
            .quests
            .iter()
            .filter(open)
            .filter(|quest| quest.level >= level && quest.level <= level + 2)
            .collect();
        let here: Vec<u32> = near_level
            .iter()
            .filter(|quest| first_territory(quest.id) == Some(territory))
            .map(|quest| quest.id)
            .collect();
        if !here.is_empty() && fastrand::f32() < 0.7 {
            return Some(here[fastrand::usize(..here.len())]);
        }
        if !near_level.is_empty() {
            return Some(near_level[fastrand::usize(..near_level.len())].id);
        }
        // Nothing at its level: the lowest still open above it.
        self.quests
            .iter()
            .filter(open)
            .find(|quest| quest.level > level)
            .map(|quest| quest.id)
    }
}

/// The levels of band `band` (0-9): 1-10, 11-20, … 91-100.
pub fn band_levels(band: u8) -> (u8, u8) {
    (band * 10 + 1, band * 10 + 10)
}

/// A questing bot's place on its route.
#[derive(Debug, Clone, PartialEq)]
pub struct Questing {
    /// Its ten levels (0-9: 1-10 … 91-100.)
    pub band: u8,
    pub quest: u32,
    /// The step of the quest's path it's on.
    pub step: usize,
    /// Whether it has done what's done at that step (the next decision moves on.)
    pub step_done: bool,
    /// The quests it's done on this route.
    pub done: HashSet<u32>,
    /// Gear it has had from quests (item IDs), and its attacks for its class and level (action ID, range.)
    pub rewards: Vec<u32>,
    pub attacks: Vec<(u32, i8)>,
}

/// The class a questing bot at `level` levels as: before 30 a starting class, then the jobs, a later expansion's from
/// the level its story reaches (ClassJob rows: 1-7, 26, 29 the classes; 19-25, 27, 28, 30 A Realm Reborn's jobs;
/// 31-33 Heavensward's at 50; 34, 35 Stormblood's at 60; 37, 38 Shadowbringers' at 70; 39, 40 Endwalker's at 80; 41,
/// 42 Dawntrail's at 80.)
pub fn questing_class(level: u8) -> u8 {
    const CLASSES: [u8; 9] = [1, 2, 3, 4, 5, 6, 7, 26, 29];
    const JOBS: [(u8, u8); 22] = [
        (19, 30),
        (20, 30),
        (21, 30),
        (22, 30),
        (23, 30),
        (24, 30),
        (25, 30),
        (27, 30),
        (28, 30),
        (30, 30),
        (31, 50),
        (32, 50),
        (33, 50),
        (34, 60),
        (35, 60),
        (37, 70),
        (38, 70),
        (39, 80),
        (40, 80),
        (41, 80),
        (42, 80),
        (36, 255),
    ];
    if level < 30 {
        return CLASSES[fastrand::usize(..CLASSES.len())];
    }
    let open: Vec<u8> = JOBS
        .iter()
        .filter(|(_, from)| *from <= level)
        .map(|(job, _)| *job)
        .collect();
    open[fastrand::usize(..open.len())]
}

/// The gear a questing bot wears: for each slot, the best of what its quests gave it (`rewards`) and what any player
/// of its level would have bought or been given a few levels back (`base`), the best three times in four, else the
/// next best; a slot's quest piece is favoured over an equal other one, so pieces of a quest's set stay together.
pub fn quest_gear(base: &[(u8, u16, u32)], rewards: &[(u8, u16, u32)]) -> Vec<(u8, u32)> {
    let mut slots: HashMap<u8, Vec<(u16, bool, u32)>> = HashMap::new();
    for (slot, item_level, id) in base {
        slots
            .entry(*slot)
            .or_default()
            .push((*item_level, false, *id));
    }
    for (slot, item_level, id) in rewards {
        slots
            .entry(*slot)
            .or_default()
            .push((*item_level, true, *id));
    }
    let mut gear = Vec::new();
    for (slot, mut candidates) in slots {
        candidates.sort_by(|a, b| b.cmp(a));
        candidates.dedup_by_key(|candidate| candidate.2);
        let pick = if candidates.len() > 1 && fastrand::f32() >= 0.75 {
            1
        } else {
            0
        };
        gear.push((slot, candidates[pick].2));
    }
    gear.sort();
    gear
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A Questionable quest file's steps, read as a bot does them (the numbers of quest 100's, without its text.)
    #[test]
    fn a_quest_file_is_read() {
        let value: Value = serde_json::from_str(
            r#"{"QuestSequence": [
                {"Sequence": 0, "Steps": [{"DataId": 1009813, "Position": {"X": 55.4, "Y": 31.2, "Z": -763.4},
                  "TerritoryId": 156, "InteractionType": "AcceptQuest"}]},
                {"Sequence": 3, "Steps": [{"DataId": 1009835, "Position": {"X": -103.4, "Y": -1.3, "Z": -615.9},
                  "TerritoryId": 156, "InteractionType": "Combat", "KillEnemyDataIds": [3565, 4619]}]},
                {"Sequence": 255, "Steps": [{"TerritoryId": 156, "InteractionType": "Duty"},
                  {"TerritoryId": 156, "InteractionType": "Say"}]}
            ]}"#,
        )
        .unwrap();
        let steps = QuestPaths::parse(&value).unwrap();
        assert_eq!(steps.len(), 4);
        assert_eq!(steps[0].kind, StepKind::Talk);
        assert_eq!(steps[0].territory, 156);
        assert_eq!(steps[1].kind, StepKind::Combat(vec![3565, 4619]));
        assert_eq!(steps[1].sequence, 3);
        assert_eq!(steps[2].kind, StepKind::Duty);
        assert_eq!(steps[2].at, None);
        assert_eq!(steps[3].kind, StepKind::Other);
        assert_eq!(
            quest_id_of(Path::new("QuestPaths/x/551_Anything.json")),
            Some(551)
        );
        assert_eq!(quest_id_of(Path::new("QuestPaths/quest-v1.json")), None);
    }

    fn quest(id: u32, level: u8, previous: &[u32]) -> QuestInfo {
        QuestInfo {
            id,
            level,
            previous: previous.to_vec(),
            rewards: Vec::new(),
        }
    }

    /// A route keeps to its band, follows a quest's chain, and no more than `per_quest` bots take one quest.
    #[test]
    fn routes_keep_to_their_band_and_share_quests() {
        fastrand::seed(1);
        let routes = Routes::from_quests(vec![
            quest(1, 1, &[]),
            quest(2, 2, &[1]),
            quest(3, 3, &[]),
            quest(4, 9, &[]),
            quest(5, 12, &[]),
        ]);
        let busy = HashMap::from([(1, 3usize)]);
        for _ in 0..20 {
            let start = routes
                .start(band_levels(0), &busy, 3, |_, _| false)
                .unwrap();
            assert!(start == 2 || start == 3, "{start}");
        }
        assert_eq!(
            routes.start(band_levels(0), &HashMap::new(), 3, |quest, _| quest != 3),
            Some(3)
        );
        // After 3 at level 9, only 4 is left in the band; after 4, none (the route's over.)
        let done = HashSet::from([1, 2, 3]);
        let next = routes.next(3, 9, band_levels(0), &done, &HashMap::new(), 3, |_| None, 0);
        assert_eq!(next, Some(4));
        let done = HashSet::from([1, 2, 3, 4]);
        let next = routes.next(
            4,
            10,
            band_levels(0),
            &done,
            &HashMap::new(),
            3,
            |_| None,
            0,
        );
        assert_eq!(next, None);
    }

    #[test]
    fn classes_by_level() {
        for _ in 0..50 {
            assert!([1, 2, 3, 4, 5, 6, 7, 26, 29].contains(&questing_class(12)));
            let job = questing_class(55);
            assert!((19..=33).contains(&job) && job != 26 && job != 29, "{job}");
        }
    }

    /// Quest gear over what a player would have had anyway, slot by slot.
    #[test]
    fn quest_gear_is_worn() {
        fastrand::seed(2);
        let base = [(3, 10, 100), (4, 10, 101)];
        let rewards = [(4, 12, 200), (5, 11, 201)];
        let mut body_from_quest = 0;
        for _ in 0..40 {
            let gear = quest_gear(&base, &rewards);
            assert!(gear.contains(&(3, 100)) && gear.contains(&(5, 201)));
            if gear.contains(&(4, 200)) {
                body_from_quest += 1;
            }
        }
        assert!(body_from_quest > 20);
    }
}
