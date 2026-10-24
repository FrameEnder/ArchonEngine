use glam::Vec3A;
use physis::{
    race::{Gender, Race, Tribe},
    savedata::chardat::CustomizeData,
};
use serde::Deserialize;
use std::io::Read;

use crate::zone_connection::PersistentQuest;
use crate::{
    CharaMake, GameData, PlayerData, WorldDatabase,
    glamour::{DRESSER_SLOTS, DresserItem, PLATES, Plate},
    inventory::{Inventory, Item, Storage},
};
use kawari::{common::Position, ipc::zone::GrandCompany};

#[derive(Deserialize)]
struct NameValue {
    value: i32,
}

#[derive(Deserialize)]
struct DayMonthValue {
    day: i32,
    month: i32,
}

#[derive(Deserialize)]
struct ClassJobLevelValue {
    level: i32,
    exp: Option<u32>,
    value: i32,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct InventoryItem {
    slot: i32,
    quantity: i32,
    id: u32,
    crafter_content_id: u64,
    item_flags: u8,
    condition: u16,
    spiritbond_or_collectability: u16,
    glamour_id: u32,
    materia: Vec<u16>,
    materia_grades: Vec<u8>,
    stains: Vec<u8>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct InventoryContainer {
    items: Vec<InventoryItem>,
}

/// The extended plugin's additions (`kawari_extended`; docs/src/fork/auracite.md). Every member is optional, and a
/// section the client hadn't loaded (`*_loaded: false`) is left alone.
#[derive(Deserialize, Default)]
#[serde(default)]
struct Extended {
    version: u32,
    crystals: Option<InventoryContainer>,
    key_items: Option<InventoryContainer>,
    saddlebag: Option<Vec<InventoryContainer>>,
    saddlebag_loaded: Option<bool>,
    active_quests: Option<Vec<ExtendedQuest>>,
    completed_leves: Option<Vec<u8>>,
    completed_legacy_quests: Option<Vec<u8>>,
    unlocked_map_markers: Option<Vec<u8>>,
    gathering_log: Option<Vec<u8>>,
    glamour: Option<ExtendedGlamour>,
    companion: Option<ExtendedCompanion>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct ExtendedQuest {
    /// The quest's event ID (65536 and up) or its Quest row.
    id: u32,
    sequence: u8,
    variables: Vec<u8>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct ExtendedGlamourItem {
    /// The item, + 1,000,000 if HQ.
    item_id: u32,
    stains: Vec<u8>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct ExtendedPlate {
    items: Vec<ExtendedGlamourItem>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct ExtendedGlamour {
    dresser_loaded: Option<bool>,
    dresser: Option<Vec<ExtendedGlamourItem>>,
    plates_loaded: Option<bool>,
    plates: Option<Vec<ExtendedPlate>>,
    armoire_loaded: Option<bool>,
    armoire: Option<Vec<u8>>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct ExtendedCompanion {
    name: Option<String>,
    rank: Option<u8>,
    stars: Option<u8>,
    exp: Option<u32>,
    skill_points: Option<u8>,
    /// The BuddyAction it was last told (3 Follow … 7 Healer Stance.)
    stance: Option<u8>,
    favorite_feed: Option<u8>,
    colour: Option<u8>,
    barding: Option<Vec<u16>>,
    /// Points in the Defender, Attacker and Healer lines.
    skill_lines: Option<Vec<u8>>,
}

/// Whether a section the client may not have loaded should be imported: not when it says it wasn't loaded.
fn loaded(flag: Option<bool>) -> bool {
    flag != Some(false)
}

/// Copies an Auracite container's items into a Kawari container, by slot.
fn fill_container(container: &InventoryContainer, target: &mut dyn Storage) {
    for item in &container.items {
        if item.slot < 0 || item.slot as u32 >= target.max_slots() {
            continue;
        }
        *target.get_slot_mut(item.slot as u16) = Item {
            quantity: item.quantity,
            item_id: item.id,
            crafter_content_id: item.crafter_content_id,
            item_flags: item.item_flags,
            condition: item.condition,
            spiritbond_or_collectability: item.spiritbond_or_collectability,
            glamour_id: item.glamour_id,
            materia: item.materia.clone().try_into().unwrap_or_default(),
            materia_grades: item.materia_grades.clone().try_into().unwrap_or_default(),
            stains: item.stains.clone().try_into().unwrap_or_default(),
            ..Default::default()
        };
    }
}

fn glamour_item(item: &ExtendedGlamourItem) -> DresserItem {
    DresserItem {
        item_id: item.item_id,
        stain0: item.stains.first().copied().unwrap_or_default(),
        stain1: item.stains.get(1).copied().unwrap_or_default(),
    }
}

/// Applies what Kawari has a place for from `kawari_extended` to a character.
fn apply_extended(extended: &Extended, player_data: &mut PlayerData) {
    let inventory = &mut player_data.inventory;
    if let Some(crystals) = &extended.crystals {
        fill_container(crystals, &mut inventory.crystals);
    }
    if let Some(key_items) = &extended.key_items {
        fill_container(key_items, &mut inventory.key_items);
    }
    if loaded(extended.saddlebag_loaded)
        && let Some(pages) = &extended.saddlebag
    {
        for (page, target) in pages.iter().zip(inventory.saddlebag.iter_mut()) {
            fill_container(page, target);
        }
    }

    if let Some(glamour) = &extended.glamour {
        if loaded(glamour.dresser_loaded)
            && let Some(dresser) = &glamour.dresser
        {
            inventory.glamour.dresser = dresser
                .iter()
                .take(DRESSER_SLOTS)
                .map(glamour_item)
                .collect();
        }
        if loaded(glamour.plates_loaded)
            && let Some(plates) = &glamour.plates
        {
            inventory.glamour.plates = plates
                .iter()
                .take(PLATES)
                .map(|plate| {
                    let mut out = Plate::default();
                    for (i, item) in plate.items.iter().take(12).enumerate() {
                        let item = glamour_item(item);
                        out.item_ids[i] = item.item_id;
                        out.stain0[i] = item.stain0;
                        out.stain1[i] = item.stain1;
                    }
                    out
                })
                .collect();
        }
        if loaded(glamour.armoire_loaded)
            && let Some(armoire) = &glamour.armoire
        {
            inventory.glamour.armoire = armoire.clone();
        }
    }

    if let Some(quests) = &extended.active_quests {
        player_data.quest.active.0 = quests
            .iter()
            .filter_map(|quest| {
                let id = if quest.id >= 65536 {
                    quest.id - 65536
                } else {
                    quest.id
                };
                let mut variables = [0u8; 6];
                for (to, from) in variables.iter_mut().zip(&quest.variables) {
                    *to = *from;
                }
                (id > 0 && id <= u16::MAX as u32).then_some(PersistentQuest {
                    id: id as u16,
                    sequence: quest.sequence,
                    variables,
                })
            })
            .collect();
    }
    if let Some(leves) = &extended.completed_leves {
        player_data.quest.completed_levequests.data = leves.clone();
    }
    if let Some(legacy) = &extended.completed_legacy_quests {
        player_data.quest.completed_legacy.data = legacy.clone();
    }
    if let Some(markers) = &extended.unlocked_map_markers {
        player_data.quest.unlocked_map_markers.data = markers.clone();
    }
    if let Some(log) = &extended.gathering_log {
        player_data.quest.gathered_gathering_items.data = log.clone();
    }

    if let Some(companion) = &extended.companion {
        let buddy = &mut player_data.buddy;
        if let Some(name) = &companion.name {
            buddy.name = name.clone();
        }
        if let Some(rank) = companion.rank {
            buddy.rank = rank as i32;
        }
        if let Some(stars) = companion.stars {
            buddy.stars = stars as i32;
        }
        if let Some(exp) = companion.exp {
            buddy.exp = exp as i32;
        }
        if let Some(points) = companion.skill_points {
            buddy.skill_points = points as i32;
        }
        if let Some(stance) = companion.stance.filter(|stance| (3..=7).contains(stance)) {
            buddy.active_command = stance as i32;
        }
        if let Some(feed) = companion.favorite_feed {
            buddy.favorite_feed = feed as i32;
        }
        if let Some(colour) = companion.colour {
            buddy.color = colour as i32;
        }
        if let Some(barding) = &companion.barding {
            for (to, from) in buddy.barding.0.iter_mut().zip(barding) {
                *to = (*from).min(u8::MAX as u16) as u8;
            }
        }
        if let Some(lines) = &companion.skill_lines {
            for (to, from) in buddy.levels.0.iter_mut().zip(lines) {
                *to = *from;
            }
        }
    }
}

#[derive(Deserialize)]
struct Appearance {
    model_type: i32,
    height: i32,
    face_type: i32,
    hair_style: i32,
    has_highlights: bool,
    skin_color: i32,
    eye_color: i32,
    hair_color: i32,
    hair_color2: i32,
    face_features: i32,
    face_features_color: i32,
    eyebrows: i32,
    eye_color2: i32,
    eye_shape: i32,
    nose_shape: i32,
    jaw_shape: i32,
    lip_style: i32,
    lip_color: i32,
    race_feature_size: i32,
    race_feature_type: i32,
    bust_size: i32,
    facepaint: i32,
    facepaint_color: i32,
}

#[derive(Deserialize)]
struct CharacterJson {
    name: String,
    city_state: NameValue,
    nameday: DayMonthValue,
    guardian: NameValue,
    gender: NameValue,
    tribe: NameValue,
    race: NameValue,
    classjob_levels: Vec<ClassJobLevelValue>,
    grand_company: NameValue,
    grand_company_ranks: Vec<u8>,
    title: NameValue,
    voice: i32,

    is_battle_mentor: bool,
    is_trade_mentor: bool,
    is_novice: bool,
    is_returner: bool,

    appearance: Appearance,

    inventory1: InventoryContainer,
    inventory2: InventoryContainer,
    inventory3: InventoryContainer,
    inventory4: InventoryContainer,
    equipped: InventoryContainer,

    currency: InventoryContainer,

    armory_off_hand: InventoryContainer,
    armory_head: InventoryContainer,
    armory_body: InventoryContainer,
    armory_hands: InventoryContainer,
    armory_legs: InventoryContainer,
    armory_feets: InventoryContainer,
    armory_ear: InventoryContainer,
    armory_neck: InventoryContainer,
    armory_wrist: InventoryContainer,
    armory_rings: InventoryContainer,
    armory_soul_crystal: InventoryContainer,
    armory_main_hand: InventoryContainer,

    // unlocks
    unlocks: Vec<u8>,
    seen_active_help: Vec<u8>,
    minions: Vec<u8>,
    mounts: Vec<u8>,
    orchestrion_rolls: Vec<u8>,
    cutscene_seen: Vec<u8>,
    ornaments: Vec<u8>,
    caught_fish: Vec<u8>,
    caught_spearfish: Vec<u8>,
    adventures: Vec<u8>,
    triple_triad_cards: Vec<u8>,
    glasses_styles: Vec<u8>,
    chocobo_taxi_stands: Vec<u8>,
    titles: Vec<u8>,
    unlocked_companion_equip: Vec<u8>,

    // aether currents
    comp_flg_set: Vec<u8>,
    unlocked_aether_currents: Vec<u8>,

    // aetheryte
    unlocked_aetherytes: Vec<u8>,
    homepoint: i32,
    favorite_aetherytes: Vec<u16>,
    free_aetheryte: i32,

    // classjob
    current_class: i32,
    first_class: i32,
    rested_exp: i32,

    // content
    unlocked_special_content: Vec<u8>,
    unlocked_raids: Vec<u8>,
    unlocked_dungeons: Vec<u8>,
    unlocked_guildhests: Vec<u8>,
    unlocked_trials: Vec<u8>,
    unlocked_crystalline_conflicts: Vec<u8>,
    unlocked_frontlines: Vec<u8>,
    cleared_raids: Vec<u8>,
    cleared_dungeons: Vec<u8>,
    cleared_guildhests: Vec<u8>,
    cleared_trials: Vec<u8>,
    cleared_crystalline_conflicts: Vec<u8>,
    cleared_frontlines: Vec<u8>,
    cleared_masked_carnivale: Vec<u8>,
    unlocked_misc_content: Vec<u8>,
    cleared_misc_content: Vec<u8>,

    // quest
    completed_quests: Vec<u8>,

    // volatile
    position_x: f32,
    position_y: f32,
    position_z: f32,
    rotation: f32,
    zone_id: u16,
}

pub enum ImportError {
    CharacterExists,
    ReadError,
    ParseError,
    MissingData,
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let message = match self {
            ImportError::CharacterExists => "Character already exists",
            ImportError::ReadError => "Error while reading files",
            ImportError::ParseError => "Error while parsing files",
            ImportError::MissingData => {
                "Not all required data can be found for some reason. Please report this as a bug!"
            }
        };

        write!(f, "{message}")
    }
}

impl WorldDatabase {
    /// Imports a character from an Auracite backup at `path`.
    pub fn import_character(
        &mut self,
        game_data: &mut GameData,
        service_account_id: u64,
        path: &str,
    ) -> Result<(), ImportError> {
        tracing::info!("Importing character backup from {path}...");

        let file = std::fs::File::open(path).unwrap();

        let mut archive = zip::ZipArchive::new(file).unwrap();

        let character: CharacterJson;
        let extended: Option<serde_json::Value>;
        {
            let mut character_file = archive
                .by_name("character.json")
                .map_err(|_| ImportError::ReadError)?;

            let mut json_string = String::new();
            character_file
                .read_to_string(&mut json_string)
                .map_err(|_| ImportError::ReadError)?;

            character = serde_json::from_str(&json_string).map_err(|_| ImportError::MissingData)?;
            // The extended plugin's object, kept whole (a later Kawari, or an export, can use what this one can't.)
            extended = serde_json::from_str::<serde_json::Value>(&json_string)
                .ok()
                .and_then(|mut json| json.get_mut("kawari_extended").map(std::mem::take))
                .filter(|value| value.is_object());
        }

        if !self.check_is_name_free(&character.name) {
            return Err(ImportError::CharacterExists);
        }

        let customize = CustomizeData {
            race: Race::from_repr(character.race.value as u8).unwrap(),
            gender: Gender::from_repr(character.gender.value as u8).unwrap(),
            age: character.appearance.model_type as u8,
            height: character.appearance.height as u8,
            tribe: Tribe::from_repr(character.tribe.value as u8).unwrap(),
            face: character.appearance.face_type as u8,
            hair: character.appearance.hair_style as u8,
            enable_highlights: character.appearance.has_highlights as u8 == 1,
            skin_tone: character.appearance.skin_color as u8,
            right_eye_color: character.appearance.eye_color2 as u8,
            hair_tone: character.appearance.hair_color as u8,
            highlights: character.appearance.hair_color2 as u8,
            facial_features: character.appearance.face_features as u8,
            facial_feature_color: character.appearance.face_features_color as u8,
            eyebrows: character.appearance.eyebrows as u8,
            left_eye_color: character.appearance.eye_color as u8,
            eyes: character.appearance.eye_shape as u8,
            nose: character.appearance.nose_shape as u8,
            jaw: character.appearance.jaw_shape as u8,
            mouth: character.appearance.lip_style as u8,
            lips_tone_fur_pattern: character.appearance.lip_color as u8,
            race_feature_size: character.appearance.race_feature_size as u8,
            race_feature_type: character.appearance.race_feature_type as u8,
            bust: character.appearance.bust_size as u8,
            face_paint: character.appearance.facepaint as u8,
            face_paint_color: character.appearance.facepaint_color as u8,
        };

        let chara_make = CharaMake {
            customize,
            voice_id: character.voice,
            guardian: character.guardian.value,
            birth_month: character.nameday.month,
            birth_day: character.nameday.day,
            classjob_id: character.current_class,
            unk2: 1,
        };

        let (_, actor_id) = self.create_player_data(
            service_account_id,
            &character.name,
            &chara_make.to_json(),
            character.city_state.value as u8,
            character.zone_id,
            Inventory::default(),
            game_data,
        );

        let mut player_data = self.find_player_data(actor_id, game_data);

        // import jobs
        for classjob in &character.classjob_levels {
            // find the array index of the job
            let index = game_data
                .get_exp_array_index(classjob.value as u16)
                .ok_or(ImportError::ParseError)?;

            player_data.classjob.levels.0[index as usize] = classjob.level as u16;
            if let Some(exp) = classjob.exp {
                player_data.classjob.exp.0[index as usize] = exp as i32;
            }
        }

        player_data.grand_company.active_company =
            GrandCompany::from_repr(character.grand_company.value as usize).unwrap_or_default();
        player_data.grand_company.company_ranks.0 = character.grand_company_ranks;
        player_data.volatile.title = character.title.value;

        // mentor status
        player_data.mentor.is_battle = character.is_battle_mentor as i32;
        player_data.mentor.is_trade = character.is_trade_mentor as i32;
        player_data.mentor.is_novice = character.is_novice as i32;
        player_data.mentor.is_returner = character.is_returner as i32;

        // import inventory
        fill_container(&character.inventory1, &mut player_data.inventory.pages[0]);
        fill_container(&character.inventory2, &mut player_data.inventory.pages[1]);
        fill_container(&character.inventory3, &mut player_data.inventory.pages[2]);
        fill_container(&character.inventory4, &mut player_data.inventory.pages[3]);
        fill_container(&character.equipped, &mut player_data.inventory.equipped);

        fill_container(&character.currency, &mut player_data.inventory.currency);

        fill_container(
            &character.armory_off_hand,
            &mut player_data.inventory.armoury_off_hand,
        );
        fill_container(
            &character.armory_head,
            &mut player_data.inventory.armoury_head,
        );
        fill_container(
            &character.armory_body,
            &mut player_data.inventory.armoury_body,
        );
        fill_container(
            &character.armory_hands,
            &mut player_data.inventory.armoury_hands,
        );
        fill_container(
            &character.armory_legs,
            &mut player_data.inventory.armoury_legs,
        );
        fill_container(
            &character.armory_feets,
            &mut player_data.inventory.armoury_feet,
        );
        fill_container(
            &character.armory_ear,
            &mut player_data.inventory.armoury_earring,
        );
        fill_container(
            &character.armory_neck,
            &mut player_data.inventory.armoury_necklace,
        );
        fill_container(
            &character.armory_wrist,
            &mut player_data.inventory.armoury_bracelet,
        );
        fill_container(
            &character.armory_rings,
            &mut player_data.inventory.armoury_rings,
        );
        fill_container(
            &character.armory_soul_crystal,
            &mut player_data.inventory.armoury_soul_crystal,
        );
        fill_container(
            &character.armory_main_hand,
            &mut player_data.inventory.armoury_main_hand,
        );

        // unlocks
        player_data.unlock.unlocks.data = character.unlocks.clone();
        player_data.unlock.seen_active_help.data = character.seen_active_help.clone();
        player_data.unlock.minions.data = character.minions.clone();
        player_data.unlock.mounts.data = character.mounts.clone();
        player_data.unlock.orchestrion_rolls.data = character.orchestrion_rolls.clone();
        player_data.unlock.cutscene_seen.data = character.cutscene_seen.clone();
        player_data.unlock.ornaments.data = character.ornaments.clone();
        player_data.unlock.caught_fish.data = character.caught_fish.clone();
        player_data.unlock.caught_spearfish.data = character.caught_spearfish.clone();
        player_data.unlock.adventures.data = character.adventures.clone();
        player_data.unlock.triple_triad_cards.data = character.triple_triad_cards.clone();
        player_data.unlock.glasses_styles.data = character.glasses_styles.clone();
        player_data.unlock.chocobo_taxi_stands.data = character.chocobo_taxi_stands.clone();
        player_data.unlock.titles.data = character.titles.clone();
        player_data.companion.unlocked_equip.data = character.unlocked_companion_equip.clone();

        // aether current
        player_data.aether_current.unlocked.data = character.unlocked_aether_currents.clone();
        player_data.aether_current.comp_flg_set.data = character.comp_flg_set.clone();

        // aetheryte
        player_data.aetheryte.unlocked.data = character.unlocked_aetherytes.clone();
        player_data.aetheryte.homepoint = character.homepoint;
        player_data.aetheryte.favorite_aetherytes.0 = character.favorite_aetherytes;
        player_data.aetheryte.free_aetheryte = character.free_aetheryte;

        // classjob
        player_data.classjob.current_class = character.current_class;
        player_data.classjob.first_class = character.first_class;
        player_data.classjob.rested_exp = character.rested_exp;

        // content
        player_data.content.unlocked_special_content.data =
            character.unlocked_special_content.clone();
        player_data.content.unlocked_raids.data = character.unlocked_raids.clone();
        player_data.content.unlocked_dungeons.data = character.unlocked_dungeons.clone();
        player_data.content.unlocked_guildhests.data = character.unlocked_guildhests.clone();
        player_data.content.unlocked_trials.data = character.unlocked_trials.clone();
        player_data.content.unlocked_crystalline_conflicts.data =
            character.unlocked_crystalline_conflicts.clone();
        player_data.content.unlocked_frontlines.data = character.unlocked_frontlines.clone();
        player_data.content.cleared_raids.data = character.cleared_raids.clone();
        player_data.content.cleared_dungeons.data = character.cleared_dungeons.clone();
        player_data.content.cleared_guildhests.data = character.cleared_guildhests.clone();
        player_data.content.cleared_trials.data = character.cleared_trials.clone();
        player_data.content.cleared_crystalline_conflicts.data =
            character.cleared_crystalline_conflicts.clone();
        player_data.content.cleared_frontlines.data = character.cleared_frontlines.clone();
        player_data.content.cleared_masked_carnivale.data =
            character.cleared_masked_carnivale.clone();
        player_data.content.unlocked_misc_content.data = character.unlocked_misc_content.clone();
        player_data.content.cleared_misc_content.data = character.cleared_misc_content.clone();

        // quest
        player_data.quest.completed.data = character.completed_quests.clone();

        // volatile
        player_data.volatile.position = Position(Vec3A::from_array([
            character.position_x,
            character.position_y,
            character.position_z,
        ]));
        player_data.volatile.rotation = character.rotation as f64;

        if let Some(raw) = &extended {
            match serde_json::from_value::<Extended>(raw.clone()) {
                Ok(parsed) => apply_extended(&parsed, &mut player_data),
                Err(error) => {
                    tracing::warn!(
                        "The backup's kawari_extended couldn't be read ({error}); it's kept as it is."
                    )
                }
            }
        }

        self.commit_player_data(&player_data);
        if let Some(raw) = &extended {
            self.save_auracite_extended(player_data.character.content_id as u64, &raw.to_string());
        }

        // Clean up file
        let _ = std::fs::remove_file(path); // It's okay if this fails

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Made-up values in the extended plugin's shape (docs/src/fork/auracite.md.)
    const SAMPLE: &str = r#"{
        "version": 1,
        "plugin_version": "test",
        "crystals": { "items": [ { "slot": 1, "quantity": 120, "id": 3 } ] },
        "key_items": { "items": [ { "slot": 0, "quantity": 1, "id": 2000001 } ] },
        "saddlebag_loaded": true,
        "saddlebag": [ { "items": [ { "slot": 2, "quantity": 5, "id": 4868 } ] }, { "items": [] } ],
        "active_quests": [ { "id": 66213, "sequence": 2, "flags": 0, "variables": [1, 0, 0, 0, 0, 3], "accepting_class_job": 0 } ],
        "completed_leves": [1, 2, 3],
        "completed_legacy_quests": [9],
        "unlocked_map_markers": [4, 5],
        "gathering_log": [255, 0, 1],
        "glamour": {
            "dresser_loaded": true,
            "dresser": [ { "item_id": 1026447, "stains": [36, 0] }, { "item_id": 0, "stains": [0, 0] } ],
            "plates_loaded": false,
            "plates": [],
            "armoire_loaded": true,
            "armoire": [0, 16]
        },
        "companion": { "name": "Test", "rank": 6, "stars": 1, "exp": 487338, "skill_points": 0, "stance": 6,
                       "favorite_feed": 2, "colour": 36, "barding": [101, 101, 101], "skill_lines": [0, 6, 0] },
        "hotbars": null,
        "something_new": { "a": 1 }
    }"#;

    #[test]
    fn extended_is_applied() {
        let extended: Extended = serde_json::from_str(SAMPLE).unwrap();
        let mut player = PlayerData::default();
        player.inventory.glamour.plates = vec![Plate {
            item_ids: [7; 12],
            ..Default::default()
        }];
        apply_extended(&extended, &mut player);

        assert_eq!(player.inventory.crystals.get_slot(1).item_id, 3);
        assert_eq!(player.inventory.crystals.get_slot(1).quantity, 120);
        assert_eq!(player.inventory.key_items.get_slot(0).item_id, 2000001);
        assert_eq!(player.inventory.saddlebag[0].get_slot(2).quantity, 5);

        let quest = &player.quest.active.0[0];
        assert_eq!(
            (quest.id, quest.sequence, quest.variables),
            (677, 2, [1, 0, 0, 0, 0, 3])
        );
        assert_eq!(player.quest.completed_levequests.data, vec![1, 2, 3]);
        assert_eq!(player.quest.gathered_gathering_items.data, vec![255, 0, 1]);
        assert_eq!(player.quest.completed_legacy.data, vec![9]);
        assert_eq!(player.quest.unlocked_map_markers.data, vec![4, 5]);

        assert_eq!(player.inventory.glamour.dresser[0].item_id, 1026447);
        assert_eq!(player.inventory.glamour.dresser[0].stain0, 36);
        // Plates weren't loaded: what was there stays.
        assert_eq!(player.inventory.glamour.plates[0].item_ids[0], 7);
        assert_eq!(player.inventory.glamour.armoire, vec![0, 16]);

        assert_eq!(player.buddy.name, "Test");
        assert_eq!(player.buddy.rank, 6);
        assert_eq!(player.buddy.stars, 1);
        assert_eq!(player.buddy.exp, 487338);
        assert_eq!(player.buddy.active_command, 6);
        assert_eq!(player.buddy.barding.0, vec![101, 101, 101]);
        assert_eq!(player.buddy.levels.0, vec![0, 6, 0]);
    }

    #[test]
    fn missing_sections_change_nothing() {
        let extended: Extended = serde_json::from_str(r#"{"version": 1}"#).unwrap();
        let mut player = PlayerData::default();
        player.buddy.name = "Kept".to_string();
        apply_extended(&extended, &mut player);
        assert_eq!(player.buddy.name, "Kept");
        assert!(player.quest.active.0.is_empty());
    }
}
