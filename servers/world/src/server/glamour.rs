//! Glamours for party bots: a list the server's owner writes, `glamours.txt` beside `config.yaml` (kept out of git,
//! as the item names in it are the game's), each an outfit for one sex or either. The names are matched to items and
//! dyes in the game's data (the Item and Stain sheets) when the file is read, which is again whenever it changes. A bot
//! wears one of the outfits for its sex over its gear (`docs/src/fork/party-bots.md`, "Glamours").
//!
//! The file:
//!
//! ```text
//! # Comments start with #. Each glamour starts with a line beginning "==": its name, then (male) or (female) for
//! # the sex it's for (neither: either.)
//! == Seaside captain (female)
//! Head: <item name>
//! Body: <item name> | <dye>
//! Hands: <item name> | <dye> | <second dye>
//! Legs: <item name>
//! Feet: <item name> | <dye>
//! ```
//!
//! The slot before the colon is optional (the item says where it goes); dyes come after `|`, by the game's names for
//! them. Weapons are left out (bots keep their class's.)

use std::{collections::HashMap, path::Path, sync::Mutex, time::SystemTime};

use physis::race::Gender;

use crate::GameData;

/// Where the list is read from, relative to the server's working directory (where `config.yaml` is.)
pub const GLAMOUR_FILE: &str = "glamours.txt";

/// Who a glamour is for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Wearer {
    Male,
    Female,
    Either,
}

/// A glamour as written: its name, who it's for, and each piece's item and dyes by name.
#[derive(Debug, Clone, PartialEq)]
pub struct GlamourText {
    pub name: String,
    pub wearer: Wearer,
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
const WEAPON_SLOTS: [&str; 5] = ["weapon", "main hand", "mainhand", "off hand", "offhand"];

/// Lower case, one kind of apostrophe, single spaces: how names are matched.
pub fn normalise(name: &str) -> String {
    name.trim()
        .replace(['\u{2019}', '\u{2018}', '`'], "'")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
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
            let (wearer, tag) = if lower.ends_with("(male)") {
                (Wearer::Male, "(male)")
            } else if lower.ends_with("(female)") {
                (Wearer::Female, "(female)")
            } else {
                (Wearer::Either, "")
            };
            glamours.push(GlamourText {
                name: header[..header.len() - tag.len()].trim().to_string(),
                wearer,
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
            if WEAPON_SLOTS.contains(&slot.as_str()) {
                continue;
            }
            if SLOTS.contains(&slot.as_str()) {
                rest = after;
            }
        }
        let mut parts = rest.split('|').map(str::trim);
        let item = parts.next().unwrap_or_default().to_string();
        if item.is_empty() {
            continue;
        }
        let dyes = parts
            .filter(|dye| !dye.is_empty())
            .map(str::to_string)
            .collect();
        glamour.pieces.push((item, dyes));
    }
    glamours.retain(|glamour| !glamour.pieces.is_empty());
    glamours
}

/// Names to IDs: equippable items by name (with their EquipSlotCategory) and dyes by name.
pub struct Names {
    pub items: HashMap<String, (u32, u8)>,
    pub stains: HashMap<String, u8>,
}

/// Matches each piece's item and dyes to the game's (`names`): returns the glamours, and what wasn't found.
pub fn resolve(texts: &[GlamourText], names: &Names) -> (Vec<Glamour>, Vec<String>) {
    let mut glamours = Vec::new();
    let mut unknown = Vec::new();
    for text in texts {
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
                pieces,
            });
        }
    }
    (glamours, unknown)
}

impl GameData {
    /// Every equippable item by its (normalised) name, with its EquipSlotCategory, and every dye by name. Where two
    /// items share a name, the later row is kept.
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
        Names { items, stains }
    }
}

/// The glamours read, and the file's modification time they were read at.
static LOADED: Mutex<Option<(Option<SystemTime>, Vec<Glamour>)>> = Mutex::new(None);

/// The glamours from `GLAMOUR_FILE`, read again when the file has changed. None (and nothing logged) without the file.
pub fn glamours(game_data: &mut GameData) -> Vec<Glamour> {
    let path = Path::new(GLAMOUR_FILE);
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
                "Party bot glamours: {} read from {GLAMOUR_FILE} ({} male, {} female, {} either)",
                glamours.len(),
                count(Wearer::Male),
                count(Wearer::Female),
                count(Wearer::Either)
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

/// One of the glamours for this sex (or either), at random.
pub fn pick(glamours: &[Glamour], gender: Gender) -> Option<&Glamour> {
    let fits: Vec<&Glamour> = glamours
        .iter()
        .filter(|glamour| match glamour.wearer {
            Wearer::Either => true,
            Wearer::Male => gender == Gender::Male,
            Wearer::Female => gender == Gender::Female,
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
        let names = Names {
            items: HashMap::from([
                ("tricorne".to_string(), (100, 3)),
                ("long coat".to_string(), (200, 4)),
                ("fishing cap".to_string(), (300, 3)),
                ("waders".to_string(), (400, 7)),
            ]),
            stains: HashMap::from([("soot black".to_string(), 2), ("wine red".to_string(), 30)]),
        };
        let (glamours, unknown) = resolve(&parse(TEXT), &names);
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
        assert!(pick(&glamours, Gender::Male).is_some_and(|g| g.name == "Fisher"));
        assert_eq!(normalise("  Wine\u{2019}s   RED "), "wine's red");
    }
}
