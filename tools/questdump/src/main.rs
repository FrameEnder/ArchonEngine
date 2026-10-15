//! Dumps quest data from your game install, for scripting quests from ground truth.
//!
//! For each quest this writes its Quest sheet data (NPCs, objectives, the named scene numbers), its compiled
//! client script (`.luab`), and a readable listing of that script showing every scene and what it touches.
//!
//! The scripts are Square Enix's game files, so keep the dumps to yourself (the default folder is gitignored.)

mod luab;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use icarus::{
    BNpcName::BNpcNameSheet, ClassJobCategory::ClassJobCategorySheet,
    ENpcResident::ENpcResidentSheet, EObjName::EObjNameSheet, ExVersion::ExVersionSheet,
    JournalCategory::JournalCategorySheet, JournalGenre::JournalGenreSheet,
    JournalSection::JournalSectionSheet, PlaceName::PlaceNameSheet, Quest::QuestSheet,
    ScenarioTree::ScenarioTreeSheet,
};
use physis::Language;
use kawari::config::get_config;
use physis::Platform;
use physis::resource::{ResourceResolver, SqPackResource, UnpackedResource};
use physis::sqpack::{Hash, SqPackIndex};
use serde_json::{Value, json};

const USAGE: &str = "\
Dumps quest data and the client's quest scripts from your game install.

USAGE:
    kawari-questdump dump <QUEST>... [--out DIR] [--listing]
    kawari-questdump chain <QUEST> <COUNT> [--out DIR] [--listing]
    kawari-questdump extract <GAME PATH>... [--out DIR]
    kawari-questdump index [--out DIR]
    kawari-questdump disasm <FILE.luab> [--listing]

QUEST can be a quest ID (3854 or 69390), a script ID (XxaFst034_03854) or the quest's name in quotes.

COMMANDS:
    dump      Dump the given quests.
    chain     Dump the COUNT quests that follow QUEST in the same journal genre (e.g. the main story.)
    extract   Copy any game file out as-is (e.g. game_script/opening/OpeningGridania.luab.) A game_script file
              can also be given by its name alone (e.g. ManFst005Btl.luab) when its folder isn't known.
    index     Write quest-index.json: every quest's name, level, prerequisites, issuer, area, expansion, journal
              section and whether it's main scenario, for tools that need the whole quest graph (Kawari Web
              uses it to search, to unlock prerequisites and for its quest progress page.)
    disasm    Print a readable listing of a compiled script you already have.

OPTIONS:
    --out DIR     Where to write the dumps (default: quest-dumps)
    --listing     Include every instruction, not just each function's summary.
    --game DIR    The game's \"game\" folder (the one containing sqpack.) By default it's found the same way
                  the server finds it: config.yaml in the current directory, or auto-detection.";

struct Options {
    out: PathBuf,
    listing: bool,
    game: Option<String>,
    positional: Vec<String>,
}

fn parse_options(args: &[String]) -> Result<Options, String> {
    let mut options = Options {
        out: PathBuf::from("quest-dumps"),
        listing: false,
        game: None,
        positional: Vec::new(),
    };
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--out" => {
                options.out = PathBuf::from(iter.next().ok_or("--out needs a directory")?);
            }
            "--listing" => options.listing = true,
            "--game" => {
                options.game = Some(iter.next().ok_or("--game needs a directory")?.clone());
            }
            _ => options.positional.push(arg.clone()),
        }
    }
    Ok(options)
}

/// One quest, as much as tools need to find it and to walk the quest graph.
struct QuestEntry {
    /// Row ID in the Quest sheet (the quest ID plus `QuestSheet::STARTING_ROW`.)
    row_id: u32,
    script: String,
    name: String,
    genre: u32,
    /// Row IDs of the quests that unlock this one.
    previous: Vec<u32>,
    /// How `previous` combine: 2 means any one of them, anything else means all of them.
    previous_join: u8,
    level: u16,
    issuer: u32,
    place: u16,
    class_category: u8,
    expansion: u8,
}

/// Everything we need from the game data, read once.
struct GameData {
    resolver: ResourceResolver,
    quests: QuestSheet,
    /// Every quest row, for lookups by script id/name and walking chains.
    index: Vec<QuestEntry>,
    npc_names: HashMap<u32, String>,
    eobj_names: HashMap<u32, String>,
    bnpc_names: HashMap<u32, String>,
    place_names: HashMap<u32, String>,
    class_categories: HashMap<u32, String>,
    journal: Journal,
}

/// Where quests go in the journal, and what the game calls each part: JournalGenre (e.g. Gridanian Sidequests) belongs
/// to a JournalCategory, which belongs to a JournalSection (e.g. Sidequests.) Plus the expansions' names, and which
/// quests are the main scenario (ScenarioTree.)
#[derive(Default)]
struct Journal {
    /// JournalGenre: (name, JournalCategory)
    genres: HashMap<u32, (String, u32)>,
    /// JournalCategory: (name, JournalSection)
    categories: HashMap<u32, (String, u32)>,
    sections: HashMap<u32, String>,
    expansions: HashMap<u32, String>,
    /// Quest row IDs in ScenarioTree.
    main_scenario: std::collections::HashSet<u32>,
}

impl Journal {
    fn read(resolver: &mut ResourceResolver, language: Language) -> Self {
        let mut journal = Self::default();
        if let Ok(sheet) = JournalGenreSheet::read_from(resolver, language) {
            for (id, row) in sheet.into_iter().flatten_subrows() {
                journal
                    .genres
                    .insert(id, (row.Name.to_string(), row.JournalCategory as u32));
            }
        }
        if let Ok(sheet) = JournalCategorySheet::read_from(resolver, language) {
            for (id, row) in sheet.into_iter().flatten_subrows() {
                journal
                    .categories
                    .insert(id, (row.Name.to_string(), row.JournalSection as u32));
            }
        }
        if let Ok(sheet) = JournalSectionSheet::read_from(resolver, language) {
            for (id, row) in sheet.into_iter().flatten_subrows() {
                journal.sections.insert(id, row.Name.to_string());
            }
        }
        if let Ok(sheet) = ExVersionSheet::read_from(resolver, language) {
            for (id, row) in sheet.into_iter().flatten_subrows() {
                journal.expansions.insert(id, row.Name.to_string());
            }
        }
        // Not every sheet is translated; ScenarioTree may only be readable without a language.
        let scenario = ScenarioTreeSheet::read_from(resolver, language)
            .or_else(|_| ScenarioTreeSheet::read_from(resolver, Language::None));
        match scenario {
            Ok(sheet) => {
                for (id, _) in sheet.into_iter().flatten_subrows() {
                    journal.main_scenario.insert(id);
                }
            }
            Err(err) => eprintln!("Couldn't read the ScenarioTree sheet, so no quest is marked main scenario: {err:?}"),
        }
        journal
    }
}

impl GameData {
    fn open(game: Option<&str>) -> Result<Self, String> {
        let config = get_config();
        let mut resolver = ResourceResolver::new();
        for path in &config.filesystem.additional_search_paths {
            resolver.add_source(UnpackedResource::from_existing(path));
        }
        let game_path = match game {
            Some(path) => path.to_string(),
            None => config.filesystem.game_path(),
        };
        if !Path::new(&game_path).join("sqpack").is_dir() {
            return Err(format!(
                "{game_path} doesn't look like the game folder (there's no sqpack folder inside it)"
            ));
        }
        eprintln!("Reading game data from {game_path}");
        resolver.add_source(SqPackResource::from_existing(&game_path));

        let language = config.world.language();
        let quests = QuestSheet::read_from(&mut resolver, language)
            .map_err(|err| format!("couldn't read the Quest sheet: {err:?}"))?;

        let mut index = Vec::new();
        for (row_id, row) in quests.into_iter().flatten_subrows() {
            if row.Id.is_empty() {
                continue;
            }
            index.push(QuestEntry {
                row_id,
                script: row.Id.to_string(),
                name: row.Name.to_string(),
                genre: row.JournalGenre,
                previous: row
                    .PreviousQuest
                    .iter()
                    .copied()
                    .filter(|x| *x != 0)
                    .collect(),
                previous_join: row.PreviousQuestJoin,
                level: row.ClassJobLevel[0],
                issuer: row.IssuerStart,
                place: row.PlaceName,
                class_category: row.ClassJobCategory0,
                expansion: row.Expansion,
            });
        }

        // Names make the dumps much easier to read, but aren't required.
        let mut npc_names = HashMap::new();
        if let Ok(sheet) = ENpcResidentSheet::read_from(&mut resolver, language) {
            for (id, row) in sheet.into_iter().flatten_subrows() {
                if !row.Singular.is_empty() {
                    npc_names.insert(id, row.Singular.to_string());
                }
            }
        }
        let mut eobj_names = HashMap::new();
        if let Ok(sheet) = EObjNameSheet::read_from(&mut resolver, language) {
            for (id, row) in sheet.into_iter().flatten_subrows() {
                if !row.Singular.is_empty() {
                    eobj_names.insert(id, row.Singular.to_string());
                }
            }
        }
        let mut bnpc_names = HashMap::new();
        if let Ok(sheet) = BNpcNameSheet::read_from(&mut resolver, language) {
            for (id, row) in sheet.into_iter().flatten_subrows() {
                if !row.Singular.is_empty() {
                    bnpc_names.insert(id, row.Singular.to_string());
                }
            }
        }

        let mut place_names = HashMap::new();
        if let Ok(sheet) = PlaceNameSheet::read_from(&mut resolver, language) {
            for (id, row) in sheet.into_iter().flatten_subrows() {
                if !row.Name.is_empty() {
                    place_names.insert(id, row.Name.to_string());
                }
            }
        }
        let mut class_categories = HashMap::new();
        if let Ok(sheet) = ClassJobCategorySheet::read_from(&mut resolver, language) {
            for (id, row) in sheet.into_iter().flatten_subrows() {
                if !row.Name.is_empty() {
                    class_categories.insert(id, row.Name.to_string());
                }
            }
        }

        let journal = Journal::read(&mut resolver, language);

        Ok(Self {
            resolver,
            quests,
            index,
            npc_names,
            eobj_names,
            bnpc_names,
            place_names,
            class_categories,
            journal,
        })
    }

    /// Finds a quest's row ID from an ID, script ID or name.
    fn resolve(&self, query: &str) -> Result<u32, String> {
        if let Ok(number) = query.parse::<u32>() {
            let row_id = if number < QuestSheet::STARTING_ROW {
                number + QuestSheet::STARTING_ROW
            } else {
                number
            };
            if self.index.iter().any(|entry| entry.row_id == row_id) {
                return Ok(row_id);
            }
            return Err(format!("there's no quest with ID {query}"));
        }

        let lowered = query.to_lowercase();
        let matches: Vec<_> = self
            .index
            .iter()
            .filter(|entry| {
                entry.script.to_lowercase() == lowered || entry.name.to_lowercase() == lowered
            })
            .collect();
        match matches.as_slice() {
            [] => Err(format!("couldn't find a quest called {query:?}")),
            [only] => Ok(only.row_id),
            many => Err(format!(
                "{query:?} matches several quests, use one of their IDs instead: {}",
                many.iter()
                    .map(|entry| format!(
                        "{} ({})",
                        entry.row_id - QuestSheet::STARTING_ROW,
                        entry.script
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }

    /// Follows the quests that come after `start` in the same journal genre, in order.
    fn chain(&self, start: u32, count: usize) -> Vec<u32> {
        let genre = self
            .index
            .iter()
            .find(|entry| entry.row_id == start)
            .map(|entry| entry.genre)
            .unwrap_or_default();

        let mut result = Vec::new();
        let mut frontier = vec![start];
        let mut seen = vec![start];
        while let Some(current) = frontier.first().copied() {
            frontier.remove(0);
            let mut next: Vec<_> = self
                .index
                .iter()
                .filter(|entry| entry.previous.contains(&current) && entry.genre == genre)
                .filter(|entry| !seen.contains(&entry.row_id))
                .collect();
            next.sort_by_key(|entry| (entry.level, entry.row_id));
            for entry in next {
                seen.push(entry.row_id);
                result.push(entry.row_id);
                frontier.push(entry.row_id);
                if result.len() >= count {
                    return result;
                }
            }
        }
        result
    }

    /// The whole quest graph as JSON, one small object per quest. IDs are quest IDs, not row IDs.
    fn index_json(&self) -> Value {
        let quests: Vec<Value> = self
            .index
            .iter()
            .map(|entry| {
                let mut quest = json!({
                    "id": entry.row_id - QuestSheet::STARTING_ROW,
                    "script": entry.script,
                    "name": entry.name,
                    "level": entry.level,
                    "genre": entry.genre,
                    "previous": entry.previous.iter().map(|x| x.saturating_sub(QuestSheet::STARTING_ROW)).collect::<Vec<_>>(),
                    "previous_join": entry.previous_join,
                    "issuer": entry.issuer,
                    "place": entry.place,
                    "class_category": entry.class_category,
                    "expansion": entry.expansion,
                });
                if let Some(name) = self.npc_names.get(&entry.issuer) {
                    quest["issuer_name"] = json!(name);
                }
                if let Some(name) = self.place_names.get(&(entry.place as u32)) {
                    quest["place_name"] = json!(name);
                }
                if let Some(name) = self.class_categories.get(&(entry.class_category as u32)) {
                    quest["class_category_name"] = json!(name);
                }

                // Where it is in the journal (format 2.)
                let journal = &self.journal;
                quest["main_scenario"] = json!(journal.main_scenario.contains(&entry.row_id));
                if let Some(name) = journal.expansions.get(&(entry.expansion as u32)) {
                    quest["expansion_name"] = json!(name);
                }
                if let Some((genre_name, category)) = journal.genres.get(&entry.genre) {
                    quest["genre_name"] = json!(genre_name);
                    quest["category"] = json!(category);
                    if let Some((category_name, section)) = journal.categories.get(category) {
                        quest["category_name"] = json!(category_name);
                        quest["section"] = json!(section);
                        if let Some(section_name) = journal.sections.get(section) {
                            quest["section_name"] = json!(section_name);
                        }
                    }
                }
                quest
            })
            .collect();
        json!({ "format": 2, "quests": quests })
    }

    /// Gives a readable name to a quest variable's value, where we can.
    fn describe_value(&self, name: &str, value: u32) -> Option<String> {
        let kind = name.trim_end_matches(|c: char| c.is_ascii_digit());
        match kind {
            "ACTOR" => self.npc_names.get(&value).cloned(),
            "EOBJECT" => self.eobj_names.get(&value).cloned(),
            "ENEMY" => self.bnpc_names.get(&value).cloned(),
            _ => None,
        }
        .or_else(|| {
            // Listeners mix NPCs, objects and enemies.
            if name == "listener" {
                self.npc_names
                    .get(&value)
                    .or(self.eobj_names.get(&value))
                    .or(self.bnpc_names.get(&value))
                    .cloned()
            } else {
                None
            }
        })
    }
}

/// The client script for a quest lives at `game_script/quest/<script id / 100>/<script name>.luab`.
fn quest_script_path(script_name: &str) -> Option<String> {
    let script_id: u32 = script_name
        .get(script_name.len().checked_sub(5)?..)?
        .parse()
        .ok()?;
    Some(format!(
        "game_script/quest/{:03}/{script_name}.luab",
        script_id / 100
    ))
}

fn quest_json(data: &GameData, row_id: u32) -> Option<Value> {
    let row = data.quests.row(row_id)?;
    let named = |name: &str, value: u32| {
        let mut entry = json!({ "name": name, "value": value });
        if let Some(label) = data.describe_value(name, value) {
            entry["label"] = json!(label);
        }
        entry
    };

    let params: Vec<Value> = row
        .QuestParams
        .iter()
        .filter(|param| !param.ScriptInstruction.is_empty())
        .map(|param| named(&param.ScriptInstruction.to_string(), param.ScriptArg))
        .collect();

    let todos: Vec<Value> = row
        .TodoParams
        .iter()
        .enumerate()
        .filter(|(_, todo)| todo.ToDoCompleteSeq != 0 || todo.ToDoQty != 0)
        .map(|(index, todo)| {
            json!({
                "index": index,
                "complete_sequence": todo.ToDoCompleteSeq,
                "quantity": todo.ToDoQty,
                "countable": todo.CountableNum,
                "locations": todo.ToDoLocation.iter().filter(|x| **x != 0).collect::<Vec<_>>(),
            })
        })
        .collect();

    let listeners: Vec<Value> = row
        .QuestListenerParams
        .iter()
        .filter(|listener| listener.Listener != 0)
        .map(|listener| {
            let mut entry = json!({
                "listener": listener.Listener,
                "spawn_sequence": listener.ActorSpawnSeq,
                "despawn_sequence": listener.ActorDespawnSeq,
                "quest_uint8a": listener.QuestUInt8A,
                "condition_type": listener.ConditionType,
                "condition_value": listener.ConditionValue,
                "condition_operator": listener.ConditionOperator,
                "behavior": listener.Behavior,
                "visible": listener.VisibleBool,
                "condition": listener.ConditionBool,
                "item": listener.ItemBool,
                "announce": listener.AnnounceBool,
                "behavior_bool": listener.BehaviorBool,
                "accept": listener.AcceptBool,
                "qualified": listener.QualifiedBool,
                "can_target": listener.CanTargetBool,
            });
            if let Some(label) = data.describe_value("listener", listener.Listener) {
                entry["label"] = json!(label);
            }
            entry
        })
        .collect();

    Some(json!({
        "quest_id": row_id - QuestSheet::STARTING_ROW,
        "event_id": row_id,
        "script": row.Id.to_string(),
        "name": row.Name.to_string(),
        "level": row.ClassJobLevel[0],
        "journal_genre": row.JournalGenre,
        "previous_quests": row.PreviousQuest.iter().filter(|x| **x != 0).map(|x| x - QuestSheet::STARTING_ROW).collect::<Vec<_>>(),
        "previous_join": row.PreviousQuestJoin,
        "place": data.place_names.get(&(row.PlaceName as u32)),
        "class_category": data.class_categories.get(&(row.ClassJobCategory0 as u32)),
        "issuer_start": named("ACTOR", row.IssuerStart),
        "target_end": named("ACTOR", row.TargetEnd),
        "client_script": quest_script_path(&row.Id.to_string()),
        "params": params,
        "todos": todos,
        "listeners": listeners,
        "rewards": {
            "gil": row.GilReward,
            "exp_factor": row.ExpFactor,
            "items": row.Reward.iter().zip(row.ItemCountReward.iter()).filter(|(id, _)| **id != 0).map(|(id, count)| json!({ "item": id, "count": count })).collect::<Vec<_>>(),
            "optional_items": row.OptionalItemReward.iter().zip(row.OptionalItemCountReward.iter()).filter(|(id, _)| **id != 0).map(|(id, count)| json!({ "item": id, "count": count })).collect::<Vec<_>>(),
        },
    }))
}

/// Folders scripts are known (or likely) to live in, to name the folder a file was found in.
const SCRIPT_FOLDERS: &[&str] = &[
    "game_script/battle",
    "game_script/content",
    "game_script/custom",
    "game_script/fate",
    "game_script/guild_order",
    "game_script/leve",
    "game_script/opening",
    "game_script/quest",
    "game_script/questbattle",
    "game_script/raid",
    "game_script/shop",
    "game_script/story",
    "game_script/system",
    "game_script/warp",
];

/// Finds a game_script file by name in every index (the index only stores hashes of the folder and the file name, so
/// the file name alone is enough to find it), and copies it out. Returns how many copies were found.
fn find_file(options: &Options, file_name: &str) -> Result<usize, String> {
    let config = get_config();
    let game_path = match options.game.as_deref() {
        Some(path) => path.to_string(),
        None => config.filesystem.game_path(),
    };

    // game_script is category 0x0B, in the base game and every expansion folder.
    let sqpack = Path::new(&game_path).join("sqpack");
    let mut indices: Vec<PathBuf> = std::fs::read_dir(&sqpack)
        .map_err(|err| format!("couldn't read {}: {err}", sqpack.display()))?
        .flatten()
        .map(|entry| entry.path().join("0b0000.win32.index"))
        .filter(|path| path.is_file())
        .collect();
    indices.sort();

    let mut resource = SqPackResource::from_existing(&game_path);
    let name_hash = SqPackIndex::calculate_partial_hash(file_name);
    let mut found = 0;
    for index_path in &indices {
        let Some(index) = SqPackIndex::from_existing(Platform::Win32, index_path) else {
            continue;
        };
        for entry in &index.entries {
            let Hash::SplitPath { name, path } = entry.hash else {
                continue;
            };
            if name != name_hash {
                continue;
            }
            let folder = SCRIPT_FOLDERS
                .iter()
                .find(|folder| SqPackIndex::calculate_partial_hash(folder) == path)
                .map(|folder| folder.to_string())
                .unwrap_or_else(|| format!("an unknown folder (hash {path:#010x})"));
            println!("{file_name}: found in {folder}, {}", index_path.display());

            match resource.read_from_hash(index_path, entry.hash) {
                Ok(bytes) => {
                    found += 1;
                    let destination = options.out.join(file_name);
                    std::fs::write(&destination, &bytes).map_err(|err| {
                        format!("couldn't write {}: {err}", destination.display())
                    })?;
                    let summary = if file_name.ends_with(".luab") {
                        write_listing(&bytes, &destination.with_extension("txt"), options.listing)
                    } else {
                        String::new()
                    };
                    println!("  -> {} {summary}", destination.display());
                }
                Err(err) => println!("  couldn't read it: {err:?}"),
            }
        }
    }
    if found == 0 {
        println!("{file_name}: not found in any game_script index");
    }
    Ok(found)
}

/// Writes a listing of a compiled script next to it. Returns a one-line summary.
fn write_listing(bytes: &[u8], destination: &Path, listing: bool) -> String {
    match luab::parse(bytes) {
        Ok(chunk) => {
            let text = luab::disassemble(&chunk, listing);
            if let Err(err) = std::fs::write(destination, &text) {
                return format!("couldn't write {}: {err}", destination.display());
            }
            let scenes = luab::scene_functions(&chunk);
            format!("{} scene functions", scenes.len())
        }
        Err(err) => {
            // Still leave something useful behind: every readable string in the file.
            let strings = printable_strings(bytes);
            let _ = std::fs::write(
                destination,
                format!(
                    "; Couldn't read this as Lua 5.1 bytecode: {err}\n; Printable strings found instead:\n{}\n",
                    strings.join("\n")
                ),
            );
            format!("unreadable bytecode ({err}), wrote its strings instead")
        }
    }
}

fn printable_strings(bytes: &[u8]) -> Vec<String> {
    let mut found = Vec::new();
    let mut current = Vec::new();
    for byte in bytes {
        if byte.is_ascii_graphic() || *byte == b' ' {
            current.push(*byte);
        } else {
            if current.len() >= 4 {
                found.push(String::from_utf8_lossy(&current).into_owned());
            }
            current.clear();
        }
    }
    if current.len() >= 4 {
        found.push(String::from_utf8_lossy(&current).into_owned());
    }
    found
}

fn dump_quests(data: &mut GameData, row_ids: &[u32], options: &Options) -> Result<(), String> {
    std::fs::create_dir_all(&options.out)
        .map_err(|err| format!("couldn't create {}: {err}", options.out.display()))?;

    let mut index_lines = Vec::new();
    for row_id in row_ids {
        let Some(quest) = quest_json(data, *row_id) else {
            eprintln!("Skipping {row_id}, it isn't in the Quest sheet");
            continue;
        };
        let script = quest["script"].as_str().unwrap_or_default().to_string();
        let name = quest["name"].as_str().unwrap_or_default().to_string();
        let folder = options.out.join(&script);
        std::fs::create_dir_all(&folder)
            .map_err(|err| format!("couldn't create {}: {err}", folder.display()))?;

        let json_text = serde_json::to_string_pretty(&quest).unwrap();
        std::fs::write(folder.join("quest.json"), json_text)
            .map_err(|err| format!("couldn't write quest.json: {err}"))?;

        let script_status = match quest_script_path(&script) {
            Some(game_path) => match data.resolver.read(&game_path) {
                Ok(bytes) => {
                    let _ = std::fs::write(folder.join(format!("{script}.luab")), &bytes);
                    write_listing(
                        &bytes,
                        &folder.join(format!("{script}.txt")),
                        options.listing,
                    )
                }
                Err(err) => format!("no client script at {game_path} ({err:?})"),
            },
            None => "couldn't work out its client script path".to_string(),
        };

        let line = format!(
            "{:>5}  {:<18} lv{:<3} {:<40} {script_status}",
            row_id - QuestSheet::STARTING_ROW,
            script,
            quest["level"],
            name
        );
        println!("{line}");
        index_lines.push(line);
    }

    std::fs::write(options.out.join("index.txt"), index_lines.join("\n") + "\n")
        .map_err(|err| format!("couldn't write index.txt: {err}"))?;
    println!(
        "\nWrote {} quests to {}",
        index_lines.len(),
        options.out.display()
    );
    Ok(())
}

fn run(args: &[String]) -> Result<(), String> {
    let Some(command) = args.first() else {
        return Err(USAGE.to_string());
    };
    let options = parse_options(&args[1..])?;

    match command.as_str() {
        "dump" => {
            if options.positional.is_empty() {
                return Err(USAGE.to_string());
            }
            let mut data = GameData::open(options.game.as_deref())?;
            let row_ids = options
                .positional
                .iter()
                .map(|query| data.resolve(query))
                .collect::<Result<Vec<_>, _>>()?;
            dump_quests(&mut data, &row_ids, &options)
        }
        "chain" => {
            let [start, count] = options.positional.as_slice() else {
                return Err(USAGE.to_string());
            };
            let count: usize = count
                .parse()
                .map_err(|_| format!("{count:?} isn't a number"))?;
            let mut data = GameData::open(options.game.as_deref())?;
            let start = data.resolve(start)?;
            let row_ids = data.chain(start, count);
            if row_ids.is_empty() {
                return Err("no quests follow that one".to_string());
            }
            dump_quests(&mut data, &row_ids, &options)
        }
        "extract" => {
            if options.positional.is_empty() {
                return Err(USAGE.to_string());
            }
            let mut data = GameData::open(options.game.as_deref())?;
            std::fs::create_dir_all(&options.out)
                .map_err(|err| format!("couldn't create {}: {err}", options.out.display()))?;
            // Paths that don't exist are skipped, so several guesses at where a file lives can be tried at once.
            let mut found = 0;
            for game_path in &options.positional {
                // Just a file name: look it up in the game_script indices.
                if !game_path.contains('/') {
                    found += find_file(&options, game_path)?;
                    continue;
                }
                let Ok(bytes) = data.resolver.read(game_path) else {
                    println!("{game_path}: not found");
                    continue;
                };
                found += 1;
                let file_name = Path::new(game_path)
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "file".to_string());
                let destination = options.out.join(&file_name);
                std::fs::write(&destination, &bytes)
                    .map_err(|err| format!("couldn't write {}: {err}", destination.display()))?;
                let mut summary = String::new();
                if file_name.ends_with(".luab") {
                    let listing_path = destination.with_extension("txt");
                    summary = format!(
                        ", {}",
                        write_listing(&bytes, &listing_path, options.listing)
                    );
                }
                println!("{game_path} -> {}{summary}", destination.display());
            }
            if found == 0 {
                return Err("none of those files exist".to_string());
            }
            Ok(())
        }
        "index" => {
            let data = GameData::open(options.game.as_deref())?;
            std::fs::create_dir_all(&options.out)
                .map_err(|err| format!("couldn't create {}: {err}", options.out.display()))?;
            let destination = options.out.join("quest-index.json");
            let index = data.index_json();
            std::fs::write(&destination, serde_json::to_string(&index).unwrap())
                .map_err(|err| format!("couldn't write {}: {err}", destination.display()))?;
            println!(
                "Wrote {} quests to {}",
                data.index.len(),
                destination.display()
            );
            Ok(())
        }
        "disasm" => {
            let [file] = options.positional.as_slice() else {
                return Err(USAGE.to_string());
            };
            let bytes =
                std::fs::read(file).map_err(|err| format!("couldn't read {file}: {err}"))?;
            let chunk = luab::parse(&bytes)?;
            print!("{}", luab::disassemble(&chunk, options.listing));
            Ok(())
        }
        "help" | "--help" | "-h" => {
            println!("{USAGE}");
            Ok(())
        }
        other => Err(format!("unknown command {other:?}\n\n{USAGE}")),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(err) = run(&args) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_paths() {
        assert_eq!(
            quest_script_path("XxaFst034_03854").as_deref(),
            Some("game_script/quest/038/XxaFst034_03854.luab")
        );
        assert_eq!(
            quest_script_path("SubFst005_00028").as_deref(),
            Some("game_script/quest/000/SubFst005_00028.luab")
        );
        assert_eq!(quest_script_path("bad"), None);
    }

    #[test]
    fn options() {
        let args: Vec<String> = ["123", "--out", "somewhere", "--listing", "456"]
            .iter()
            .map(|x| x.to_string())
            .collect();
        let options = parse_options(&args).unwrap();
        assert_eq!(options.out, PathBuf::from("somewhere"));
        assert!(options.listing);
        assert_eq!(options.positional, vec!["123", "456"]);
    }
}
