//! Glamours for party bots: a list the server's owner writes, `glamours.txt` in the server's data folder (beside the
//! databases; kept out of git, as the item names in it are the game's; Kawari Web's glamour API writes it), each an outfit for one sex or either, and for some classes or any. The
//! names are matched to items, dyes and classes in the game's data (the Item, Stain and ClassJob sheets) when the file
//! is read, which is again whenever it changes. A bot wears one of the outfits for its sex and class over its gear
//! (`docs/src/fork/party-bots.md`, "Glamours").
//!
//! The file (`tools/glamour-collector`, the Chrome extension, writes it from Eorzea Collection's pages):
//!
//! ```text
//! # Comments start with #. Each glamour starts with a line beginning "==": its name, then (male) or (female) for
//! # the sex it's for ((ANY), (either) or neither: either.)
//! == Seaside captain (female)
//! Class: <Black Mage> | <SCH>
//! Head: <item name>
//! Body: <item name> | <dye>
//! Hands: <item name> | <dye> | <second dye>
//! Legs: <item name>
//! Feet: <item name> | <dye>
//! ```
//!
//! The slot before the colon is optional (the item says where it goes); dyes come after `|`, by the game's names for
//! them. Names may be written in `<...>`, as the extension exports them. `Class:` lists the classes or jobs (names or
//! abbreviations, or role groups such as `Tanks` and `Caster DPS`, `|` between them) the glamour is for; a job and its
//! base class count as one (a Black Mage glamour fits a Thaumaturge too, but a Scholar's doesn't fit a Summoner), and
//! without the line (or with `All Classes`) it fits any class. Weapons and facewear are left out (bots keep their
//! class's weapon.)

use std::{collections::HashMap, path::PathBuf, sync::Mutex, time::SystemTime};

use physis::race::Gender;

use crate::GameData;

/// The list's file name: in the server's data folder, `$KAWARI_DATA` (`/kawari/data` in Kawari Web's container, the
/// folder that's mounted from the host and kept, with the databases), or else the working directory.
pub const GLAMOUR_FILE: &str = "glamours.txt";

/// Where the list is read from (see `GLAMOUR_FILE`.)
pub fn glamour_path() -> PathBuf {
    match std::env::var_os("KAWARI_DATA") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir).join(GLAMOUR_FILE),
        _ => PathBuf::from(GLAMOUR_FILE),
    }
}

/// Who a glamour is for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Wearer {
    Male,
    Female,
    Either,
}

/// A glamour as written: its name, who it's for, the classes it's for (none: any) and each piece's item and dyes by
/// name.
#[derive(Debug, Clone, PartialEq)]
pub struct GlamourText {
    pub name: String,
    pub wearer: Wearer,
    pub classes: Vec<String>,
    pub pieces: Vec<(String, Vec<String>)>,
}

/// A glamour's piece: the item, its EquipSlotCategory and its two dyes (Stain rows, 0 undyed.)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Piece {
    pub item_id: u32,
    pub slot_category: u8,
    pub stains: [u8; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub struct Glamour {
    pub name: String,
    pub wearer: Wearer,
    /// The classes and jobs it's for, each as (ClassJob row, its ClassJobParent) (empty: any class.)
    pub classes: Vec<(u8, u8)>,
    pub pieces: Vec<Piece>,
}

/// Slot names allowed before a piece's colon; weapon lines are skipped.
const SLOTS: [&str; 14] = [
    "head",
    "body",
    "hands",
    "gloves",
    "legs",
    "feet",
    "ears",
    "earrings",
    "neck",
    "necklace",
    "wrists",
    "bracelets",
    "ring",
    "rings",
];
/// Lines left out: weapons (bots keep their class's) and facewear (not gear.)
const SKIPPED_SLOTS: [&str; 8] = [
    "weapon",
    "main hand",
    "mainhand",
    "off hand",
    "offhand",
    "face",
    "facewear",
    "glasses",
];
const CLASS_LINES: [&str; 4] = ["class", "classes", "job", "jobs"];
/// Class names meaning no restriction (Eorzea Collection's "All Classes".)
const ANY_CLASS: [&str; 4] = ["all classes", "all", "any", "any class"];

/// The role groups a combat class is in, by its ClassJob Role (1 tank, 2 melee, 3 ranged, 4 healer) and PrimaryStat
/// (2, dexterity, for the physical ranged): the names Eorzea Collection's "Intended for" uses, and a few more.
fn role_groups(role: u8, primary_stat: u8) -> &'static [&'static str] {
    const DEXTERITY: u8 = 2;
    match (role, primary_stat) {
        (1, _) => &["tanks", "tank", "disciples of war"],
        (2, _) => &["melee dps", "melee", "disciples of war"],
        (3, DEXTERITY) => &["physical ranged dps", "physical ranged", "disciples of war"],
        (3, _) => &[
            "caster dps",
            "casters",
            "caster",
            "magical ranged dps",
            "disciples of magic",
        ],
        (4, _) => &["healers", "healer", "disciples of magic"],
        _ => &[],
    }
}

/// Lower case, one kind of apostrophe, single spaces: how names are matched.
pub fn normalise(name: &str) -> String {
    name.trim()
        .replace(['\u{2019}', '\u{2018}', '`'], "'")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// A name as written, without spaces around it or the `<...>` the extension puts round it.
fn unbracket(name: &str) -> &str {
    let name = name.trim();
    name.strip_prefix('<')
        .and_then(|inner| inner.strip_suffix('>'))
        .map(str::trim)
        .unwrap_or(name)
}

/// The names between `|`, without brackets or empty ones.
fn names(text: &str) -> Vec<String> {
    text.split('|')
        .map(unbracket)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .collect()
}

/// Reads the file's text into glamours by name (see the module's description.)
pub fn parse(text: &str) -> Vec<GlamourText> {
    let mut glamours: Vec<GlamourText> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(header) = line.strip_prefix("==") {
            let header = header.trim().trim_end_matches('=').trim();
            let lower = header.to_lowercase();
            let (wearer, tag) = [
                (Wearer::Male, "(male)"),
                (Wearer::Female, "(female)"),
                (Wearer::Either, "(any)"),
                (Wearer::Either, "(either)"),
            ]
            .into_iter()
            .find(|(_, tag)| lower.ends_with(tag))
            .unwrap_or((Wearer::Either, ""));
            glamours.push(GlamourText {
                name: header[..header.len() - tag.len()].trim().to_string(),
                wearer,
                classes: Vec::new(),
                pieces: Vec::new(),
            });
            continue;
        }
        let Some(glamour) = glamours.last_mut() else {
            continue;
        };
        let mut rest = line;
        if let Some((slot, after)) = line.split_once(':') {
            let slot = normalise(slot);
            if SKIPPED_SLOTS.contains(&slot.as_str()) {
                continue;
            }
            if CLASS_LINES.contains(&slot.as_str()) {
                glamour.classes.extend(names(after));
                continue;
            }
            if SLOTS.contains(&slot.as_str()) {
                rest = after;
            }
        }
        let mut parts = names(rest).into_iter();
        let Some(item) = parts.next() else {
            continue;
        };
        glamour.pieces.push((item, parts.collect()));
    }
    glamours.retain(|glamour| !glamour.pieces.is_empty());
    glamours
}

/// Names to IDs: equippable items by name (with their EquipSlotCategory), dyes by name, and combat classes and jobs by
/// name, English name and abbreviation, and role groups ("tanks", "caster dps") by name, each to its classes (ClassJob
/// rows with their ClassJobParent.)
pub struct Names {
    pub items: HashMap<String, (u32, u8)>,
    pub stains: HashMap<String, u8>,
    pub classes: HashMap<String, Vec<(u8, u8)>>,
}

/// Matches each glamour's items, dyes and classes to the game's (`names`): returns the glamours, and what wasn't
/// found. A glamour whose classes are all unknown is left out (rather than worn by any class.)
pub fn resolve(texts: &[GlamourText], names: &Names) -> (Vec<Glamour>, Vec<String>) {
    let mut glamours = Vec::new();
    let mut unknown = Vec::new();
    for text in texts {
        let any_class = text
            .classes
            .iter()
            .any(|class| ANY_CLASS.contains(&normalise(class).as_str()));
        let mut classes = Vec::new();
        if !any_class {
            for class in &text.classes {
                match names.classes.get(&normalise(class)) {
                    Some(found) => {
                        for class in found {
                            if !classes.contains(class) {
                                classes.push(*class);
                            }
                        }
                    }
                    None => unknown.push(format!("{}: class {class:?}", text.name)),
                }
            }
            if !text.classes.is_empty() && classes.is_empty() {
                continue;
            }
        }
        let mut pieces = Vec::new();
        for (item, dyes) in &text.pieces {
            let Some(&(item_id, slot_category)) = names.items.get(&normalise(item)) else {
                unknown.push(format!("{}: item {item:?}", text.name));
                continue;
            };
            let mut stains = [0u8; 2];
            for (index, dye) in dyes.iter().take(2).enumerate() {
                let dye_name = normalise(dye);
                if matches!(dye_name.as_str(), "undyed" | "no dye" | "none" | "-") {
                    continue;
                }
                match names.stains.get(&dye_name) {
                    Some(stain) => stains[index] = *stain,
                    None => unknown.push(format!("{}: dye {dye:?}", text.name)),
                }
            }
            pieces.push(Piece {
                item_id,
                slot_category,
                stains,
            });
        }
        if !pieces.is_empty() {
            glamours.push(Glamour {
                name: text.name.clone(),
                wearer: text.wearer,
                classes,
                pieces,
            });
        }
    }
    (glamours, unknown)
}

impl GameData {
    /// Every equippable item by its (normalised) name, with its EquipSlotCategory, every dye by name, and every combat
    /// class and job by name, English name and abbreviation. Where two items share a name, the later row is kept.
    pub fn glamour_names(&mut self) -> Names {
        let mut items = HashMap::new();
        for (id, row) in self.item_sheet.into_iter().flatten_subrows() {
            let name = row.Name.to_string();
            if row.EquipSlotCategory == 0 || name.is_empty() {
                continue;
            }
            items.insert(normalise(&name), (id, row.EquipSlotCategory));
        }
        let mut stains = HashMap::new();
        if let Ok(sheet) = icarus::Stain::StainSheet::read_from(
            &mut self.resource,
            kawari::config::get_config().world.language(),
        ) {
            for (id, row) in sheet.into_iter().flatten_subrows() {
                let name = row.Name.to_string();
                if !name.is_empty() && id <= u8::MAX as u32 {
                    stains.insert(normalise(&name), id as u8);
                }
            }
        }
        let mut classes: HashMap<String, Vec<(u8, u8)>> = HashMap::new();
        for (id, row) in self.classjob_sheet.into_iter().flatten_subrows() {
            if row.Role == 0 || id > u8::MAX as u32 {
                continue;
            }
            let class = (id as u8, row.ClassJobParent);
            for name in [&row.Name, &row.NameEnglish, &row.Abbreviation] {
                let name = normalise(&name.to_string());
                if !name.is_empty() {
                    classes.insert(name, vec![class]);
                }
            }
            for group in role_groups(row.Role, row.PrimaryStat) {
                classes.entry(group.to_string()).or_default().push(class);
            }
        }
        Names {
            items,
            stains,
            classes,
        }
    }
}

/// The glamours read, and the file's modification time they were read at.
static LOADED: Mutex<Option<(Option<SystemTime>, Vec<Glamour>)>> = Mutex::new(None);

/// The glamours from `glamour_path()`, read again when the file has changed. None (and nothing logged) without the
/// file.
pub fn glamours(game_data: &mut GameData) -> Vec<Glamour> {
    let path = glamour_path();
    let path = path.as_path();
    let modified = std::fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok();
    let mut loaded = LOADED
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some((at, glamours)) = loaded.as_ref()
        && *at == modified
    {
        return glamours.clone();
    }
    let glamours = match std::fs::read_to_string(path) {
        Ok(text) => {
            let texts = parse(&text);
            let names = game_data.glamour_names();
            let (glamours, unknown) = resolve(&texts, &names);
            let count = |wearer: Wearer| glamours.iter().filter(|g| g.wearer == wearer).count();
            tracing::info!(
                "Party bot glamours: {} read from {} ({} male, {} female, {} either; {} for some classes only)",
                glamours.len(),
                path.display(),
                count(Wearer::Male),
                count(Wearer::Female),
                count(Wearer::Either),
                glamours.iter().filter(|g| !g.classes.is_empty()).count()
            );
            for missing in &unknown {
                tracing::warn!(
                    "Party bot glamours: no such {missing} in the game's data, left out"
                );
            }
            glamours
        }
        Err(_) => Vec::new(),
    };
    *loaded = Some((modified, glamours.clone()));
    glamours
}

/// Whether a glamour listed for `listed` (a ClassJob row and its parent) fits `classjob` (whose parent is `parent`):
/// the same class, the job of a listed base class, or the base class of a listed job. Jobs sharing a base class
/// (Scholar and Summoner) don't fit each other.
fn fits_class(listed: (u8, u8), classjob: u8, parent: u8) -> bool {
    let (id, listed_parent) = listed;
    id == classjob || id == parent || listed_parent == classjob
}

/// One of the glamours for this sex (or either) and class (`classjob`, whose ClassJobParent is `parent`; or any), at
/// random.
pub fn pick(glamours: &[Glamour], gender: Gender, classjob: u8, parent: u8) -> Option<&Glamour> {
    let fits: Vec<&Glamour> = glamours
        .iter()
        .filter(|glamour| match glamour.wearer {
            Wearer::Either => true,
            Wearer::Male => gender == Gender::Male,
            Wearer::Female => gender == Gender::Female,
        })
        .filter(|glamour| {
            glamour.classes.is_empty()
                || glamour
                    .classes
                    .iter()
                    .any(|&listed| fits_class(listed, classjob, parent))
        })
        .collect();
    (!fits.is_empty()).then(|| fits[fastrand::usize(..fits.len())])
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = "# A comment
== Captain (female)
Head: Tricorne | Soot Black
Body: Long Coat | Wine Red | Snow White
Weapon: Cutlass
Feet: Boots

== Fisher (male)
Fishing Cap
Legs: Waders | undyed

==Nothing here (female)
";

    fn names() -> Names {
        Names {
            items: HashMap::from([
                ("tricorne".to_string(), (100, 3)),
                ("long coat".to_string(), (200, 4)),
                ("fishing cap".to_string(), (300, 3)),
                ("waders".to_string(), (400, 7)),
                ("circlet".to_string(), (500, 3)),
                ("kote".to_string(), (600, 5)),
            ]),
            stains: HashMap::from([("soot black".to_string(), 2), ("wine red".to_string(), 30)]),
            classes: HashMap::from([
                ("black mage".to_string(), vec![(25, 7)]),
                ("blm".to_string(), vec![(25, 7)]),
                ("thaumaturge".to_string(), vec![(7, 7)]),
                ("scholar".to_string(), vec![(28, 26)]),
                ("tanks".to_string(), vec![(1, 1), (19, 1), (32, 32)]),
            ]),
        }
    }

    #[test]
    fn parses_the_list() {
        let glamours = parse(TEXT);
        assert_eq!(glamours.len(), 2);
        assert_eq!(glamours[0].name, "Captain");
        assert_eq!(glamours[0].wearer, Wearer::Female);
        assert_eq!(
            glamours[0].pieces,
            vec![
                ("Tricorne".to_string(), vec!["Soot Black".to_string()]),
                (
                    "Long Coat".to_string(),
                    vec!["Wine Red".to_string(), "Snow White".to_string()]
                ),
                ("Boots".to_string(), vec![]),
            ]
        );
        assert_eq!(glamours[1].wearer, Wearer::Male);
        assert_eq!(glamours[1].pieces[0].0, "Fishing Cap");
    }

    #[test]
    fn resolves_names_and_reports_the_rest() {
        let (glamours, unknown) = resolve(&parse(TEXT), &names());
        assert_eq!(glamours[0].pieces[0].stains, [2, 0]);
        assert_eq!(glamours[0].pieces[1].stains, [30, 0]);
        assert_eq!(glamours[1].pieces[1].stains, [0, 0]);
        assert_eq!(
            unknown,
            vec![
                "Captain: dye \"Snow White\"".to_string(),
                "Captain: item \"Boots\"".to_string()
            ]
        );
        assert!(pick(&glamours, Gender::Male, 1, 1).is_some_and(|g| g.name == "Fisher"));
        assert_eq!(normalise("  Wine\u{2019}s   RED "), "wine's red");
    }

    #[test]
    fn reads_the_collectors_export() {
        let text = "# Lines starting with # are notes.

== Vampire Reverie (ANY)
Class: <BLM> | <Scholar>
Head: <Circlet> | <Soot Black>
Hands: <Kote> | <Soot Black> | <undyed>
Weapon: <Rod>

== Unknown class (male)
Class: <Nobody>
Head: <Circlet>
";
        let texts = parse(text);
        assert_eq!(texts.len(), 2);
        assert_eq!(texts[0].name, "Vampire Reverie");
        assert_eq!(texts[0].wearer, Wearer::Either);
        assert_eq!(
            texts[0].classes,
            vec!["BLM".to_string(), "Scholar".to_string()]
        );
        assert_eq!(
            texts[0].pieces,
            vec![
                ("Circlet".to_string(), vec!["Soot Black".to_string()]),
                (
                    "Kote".to_string(),
                    vec!["Soot Black".to_string(), "undyed".to_string()]
                ),
            ]
        );

        let (glamours, unknown) = resolve(&texts, &names());
        assert_eq!(glamours.len(), 1);
        assert_eq!(glamours[0].classes, vec![(25, 7), (28, 26)]);
        assert_eq!(glamours[0].pieces[1].stains, [2, 0]);
        assert_eq!(unknown, vec!["Unknown class: class \"Nobody\"".to_string()]);
        // Black Mage and its Thaumaturge, Scholar and its Arcanist wear it; a Summoner (another Arcanist job) and a
        // Gladiator don't.
        assert!(pick(&glamours, Gender::Female, 25, 7).is_some());
        assert!(pick(&glamours, Gender::Female, 7, 7).is_some());
        assert!(pick(&glamours, Gender::Male, 28, 26).is_some());
        assert!(pick(&glamours, Gender::Male, 26, 26).is_some());
        assert!(pick(&glamours, Gender::Male, 27, 26).is_none());
        assert!(pick(&glamours, Gender::Male, 1, 1).is_none());
        // A glamour for Thaumaturge fits its Black Mage too.
        assert!(fits_class((7, 7), 25, 7));
    }

    #[test]
    fn role_groups_all_classes_and_facewear() {
        let text = "== Knight (male)
Class: <Tanks>
Head: <Circlet>
Face: <Circlet>

== Anyone (ANY)
Class: <All Classes>
Hands: <Kote>
";
        let texts = parse(text);
        assert_eq!(texts[0].pieces.len(), 1);
        let (glamours, unknown) = resolve(&texts, &names());
        assert!(unknown.is_empty());
        assert_eq!(glamours[0].classes, vec![(1, 1), (19, 1), (32, 32)]);
        assert!(glamours[1].classes.is_empty());
        let knight = &glamours[..1];
        assert!(pick(knight, Gender::Male, 19, 1).is_some());
        assert!(pick(knight, Gender::Male, 32, 32).is_some());
        assert!(pick(knight, Gender::Male, 25, 7).is_none());
        assert!(role_groups(3, 2).contains(&"physical ranged dps"));
        assert!(role_groups(3, 4).contains(&"caster dps"));
    }
}
