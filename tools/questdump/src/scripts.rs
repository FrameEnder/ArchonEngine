//! `kawari-questdump scripts <FOLDER>`: every client script of a kind at once, for checking the server's scripts
//! against them (`questkit clientcheck`). The kinds are the ones whose script names a sheet lists:
//!
//! * `custom`: CustomTalk (NPC conversations, menus and the shops they open), `game_script/custom/<id / 100>/`;
//! * `opening`: Opening (the city openings), `game_script/opening/`.
//!
//! Each script is written as `<out>/<folder>/<path>.luab` with its listing (`.txt`) and its sheet row (`.json`: the
//! row, its handler ID and its Lua parameters), plus `<out>/<folder>/index.json` listing them all with whether the
//! server has a script of the same name. The scripts are Square Enix's game files: keep them out of git.

use std::path::{Path, PathBuf};

use icarus::{CustomTalk::CustomTalkSheet, Opening::OpeningSheet};
use kawari::config::get_config;
use serde_json::{Value, json};

use crate::{GameData, Options, write_listing};

/// The folders this command knows.
pub const FOLDERS: [&str; 2] = ["custom", "opening"];

/// One script to dump: where it is in the game and where the server would have its own.
struct Script {
    name: String,
    row_id: u32,
    handler_id: u32,
    /// Its path under `game_script/<folder>/` and under the server's `events/<...>/`, without extension.
    relative: String,
    /// Where the server keeps its own script of this name, under `resources/scripts/`.
    server_path: String,
    params: Vec<Value>,
}

/// The folder a custom script's number puts it in (the server's `folder_from_script_id`): 107 is in 001.
fn numbered_folder(name: &str) -> Option<String> {
    let number: u32 = name.get(name.len().checked_sub(5)?..)?.parse().ok()?;
    Some(format!("{:03}", number / 100))
}

fn custom_scripts(data: &mut GameData) -> Result<Vec<Script>, String> {
    let language = get_config().world.language();
    let sheet = CustomTalkSheet::read_from(&mut data.resolver, language)
        .map_err(|err| format!("couldn't read the CustomTalk sheet: {err:?}"))?;
    let mut scripts = Vec::new();
    for (row_id, row) in sheet.into_iter().flatten_subrows() {
        let name = row.Name.to_string();
        let Some(folder) = numbered_folder(&name) else {
            continue;
        };
        let params = row
            .Script
            .iter()
            .filter(|element| !element.ScriptInstruction.is_empty())
            .map(|element| {
                let name = element.ScriptInstruction.to_string();
                let mut entry = json!({ "name": name, "value": element.ScriptArg });
                if let Some(label) = data.describe_value(&name, element.ScriptArg) {
                    entry["label"] = json!(label);
                }
                entry
            })
            .collect();
        scripts.push(Script {
            relative: format!("{folder}/{name}"),
            server_path: format!("events/custom/{folder}/{name}.lua"),
            name,
            row_id,
            handler_id: row_id,
            params,
        });
    }
    Ok(scripts)
}

fn opening_scripts(data: &mut GameData) -> Result<Vec<Script>, String> {
    let sheet = OpeningSheet::read_from(&mut data.resolver, physis::Language::None)
        .map_err(|err| format!("couldn't read the Opening sheet: {err:?}"))?;
    let mut scripts = Vec::new();
    for (row_id, row) in sheet.into_iter().flatten_subrows() {
        let name = row.Name.to_string();
        if name.is_empty() {
            continue;
        }
        let params = row
            .Variables
            .iter()
            .filter(|element| !element.Name.is_empty())
            .map(|element| json!({ "name": element.Name.to_string(), "value": element.Value }))
            .collect();
        scripts.push(Script {
            relative: name.clone(),
            server_path: format!("events/quest/opening/{name}.lua"),
            name,
            row_id,
            handler_id: row_id,
            params,
        });
    }
    Ok(scripts)
}

/// Dumps every script of `folder`; with `server_scripts` (the server's `resources/scripts`), only those it has.
pub fn dump(
    data: &mut GameData,
    folder: &str,
    server_scripts: Option<&Path>,
    options: &Options,
) -> Result<(), String> {
    let scripts = match folder {
        "custom" => custom_scripts(data)?,
        "opening" => opening_scripts(data)?,
        other => {
            return Err(format!(
                "{other:?} isn't a folder this knows (it knows {})",
                FOLDERS.join(", ")
            ));
        }
    };
    let root = options.out.join(folder);
    std::fs::create_dir_all(&root)
        .map_err(|err| format!("couldn't create {}: {err}", root.display()))?;

    let mut index = Vec::new();
    let (mut written, mut missing) = (0, 0);
    for script in &scripts {
        let on_server = server_scripts.map(|dir| dir.join(&script.server_path).is_file());
        if on_server == Some(false) {
            continue;
        }
        let game_path = format!("game_script/{folder}/{}.luab", script.relative);
        let destination: PathBuf = root.join(format!("{}.luab", script.relative));
        let found = match data.resolver.read(&game_path) {
            Ok(bytes) => {
                if let Some(parent) = destination.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|err| format!("couldn't create {}: {err}", parent.display()))?;
                }
                std::fs::write(&destination, &bytes)
                    .map_err(|err| format!("couldn't write {}: {err}", destination.display()))?;
                write_listing(&bytes, &destination.with_extension("txt"), options.listing);
                let row = json!({
                    "name": script.name,
                    "row_id": script.row_id,
                    "handler_id": script.handler_id,
                    "client_script": game_path,
                    "server_script": script.server_path,
                    "params": script.params,
                });
                std::fs::write(
                    destination.with_extension("json"),
                    serde_json::to_string_pretty(&row).unwrap(),
                )
                .map_err(|err| format!("couldn't write the row: {err}"))?;
                written += 1;
                true
            }
            Err(_) => {
                missing += 1;
                false
            }
        };
        index.push(json!({
            "name": script.name,
            "handler_id": script.handler_id,
            "path": format!("{}.luab", script.relative),
            "server_script": script.server_path,
            "on_server": on_server,
            "found": found,
        }));
    }
    std::fs::write(
        root.join("index.json"),
        serde_json::to_string_pretty(&index).unwrap(),
    )
    .map_err(|err| format!("couldn't write index.json: {err}"))?;
    println!(
        "Wrote {written} {folder} scripts to {} ({missing} the sheet names weren't in the game files)",
        root.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folders() {
        assert_eq!(
            numbered_folder("CmnDefAchievementReward_00107").as_deref(),
            Some("001")
        );
        assert_eq!(
            numbered_folder("CmnFatShopField_00723").as_deref(),
            Some("007")
        );
        assert_eq!(numbered_folder("NoNumber"), None);
    }
}
