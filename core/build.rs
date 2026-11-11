use std::{collections::HashMap, path::PathBuf};

use serde_yaml_ng::Value;

fn format_value(value: &Value) -> String {
    match value {
        Value::Number(number) => number.as_u64().unwrap_or_default().to_string(),
        Value::String(string) => format!("\"{string}\""),
        Value::Sequence(values) => {
            let mut values_str = Vec::new();
            for value in values {
                values_str.push(format_value(value));
            }

            format!("[{}]", values_str.join(", "))
        }
        _ => unimplemented!(),
    }
}

/// One of the fork's own opcodes (see `resources/data/fork_opcodes.yml`.)
struct ForkOpcode {
    kind: String,
    name: String,
    size: u64,
    opcode: Option<u64>,
    found_on: Option<String>,
    current: bool,
    how_to_find: String,
    /// The opcode the generated enum uses for it.
    used_opcode: u64,
    /// Rust for its `exe_scan::Finder`s.
    finders: Vec<String>,
}

/// Rust for the `exe_scan::Finder`s in a fork_opcodes.yml entry's `find` list.
fn fork_finders(entry: &Value, name: &str, known: &[Value]) -> Vec<String> {
    let Some(finders) = entry.get("find").and_then(|find| find.as_sequence()) else {
        return Vec::new();
    };
    finders
        .iter()
        .map(|finder| {
            let field = |kind: &Value, field: &str| kind.get(field).cloned();
            if let Some(immediate) = finder.get("immediate") {
                let pattern = field(immediate, "pattern")
                    .and_then(|p| p.as_str().map(str::to_string))
                    .unwrap_or_else(|| panic!("fork_opcodes.yml: {name}'s immediate finder has no pattern"));
                let offset = field(immediate, "offset").and_then(|o| o.as_u64()).unwrap_or(0);
                let width = field(immediate, "width").and_then(|w| w.as_u64()).unwrap_or(2);
                // `with` names a packet in the same list; its current opcode is what to look for.
                let with_opcode = field(immediate, "with").and_then(|w| w.as_str().map(str::to_string)).map(|with| {
                    known
                        .iter()
                        .find(|opcode| opcode.get("name").and_then(|n| n.as_str()) == Some(with.as_str()))
                        .and_then(|opcode| opcode.get("opcode")?.as_u64())
                        .unwrap_or_else(|| panic!("fork_opcodes.yml: {name}'s finder needs {with}, which isn't a known opcode in its list"))
                });
                format!(
                    "crate::exe_scan::Finder::Immediate {{ pattern: {pattern:?}, offset: {offset}, width: {width}, with_opcode: {with_opcode:?} }}"
                )
            } else if let Some(handler) = finder.get("handler") {
                let pattern = field(handler, "pattern")
                    .and_then(|p| p.as_str().map(str::to_string))
                    .unwrap_or_else(|| panic!("fork_opcodes.yml: {name}'s handler finder has no pattern"));
                let offset = field(handler, "offset").and_then(|o| o.as_i64()).unwrap_or(0);
                format!("crate::exe_scan::Finder::Handler {{ pattern: {pattern:?}, offset: {offset} }}")
            } else {
                panic!("fork_opcodes.yml: {name} has a finder that's neither immediate nor handler");
            }
        })
        .collect()
}

/// The game version Kawari supports (SUPPORTED_GAME_VERSION in constants.yml.)
fn supported_game_version() -> String {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("../resources/data/constants.yml");
    let yml: Value = serde_yaml_ng::from_str(&std::fs::read_to_string(d).unwrap()).unwrap();
    yml.get("SUPPORTED_GAME_VERSION")
        .and_then(|version| version.as_str())
        .expect("constants.yml has no SUPPORTED_GAME_VERSION")
        .to_string()
}

/// Adds the fork's own opcodes (`resources/data/fork_opcodes.yml`) to upstream's. Any not found for the supported game
/// version get a placeholder opcode the game never uses, and a build warning.
fn merge_fork_opcodes(opcodes: &mut Value) -> Vec<ForkOpcode> {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("../resources/data/fork_opcodes.yml");
    println!("cargo::rerun-if-changed={}", d.to_str().unwrap());
    let Ok(buffer) = std::fs::read_to_string(&d) else {
        return Vec::new();
    };
    let fork: Value = serde_yaml_ng::from_str(&buffer).unwrap();
    let Some(fork) = fork.as_mapping() else {
        return Vec::new();
    };
    let version = supported_game_version();

    let mut merged = Vec::new();
    for (kind, entries) in fork {
        let kind = kind.as_str().unwrap();
        let Some(list) = opcodes
            .get_mut(kind)
            .and_then(|list| list.as_sequence_mut())
        else {
            panic!("fork_opcodes.yml: {kind} isn't one of the opcode lists in opcodes.yml!");
        };
        let mut used: Vec<u64> = list
            .iter()
            .filter_map(|opcode| opcode.get("opcode")?.as_u64())
            .collect();
        let names: Vec<String> = list
            .iter()
            .filter_map(|opcode| Some(opcode.get("name")?.as_str()?.to_string()))
            .collect();
        // Upstream's entries, which a finder can refer to (`with`.)
        let known_entries: Vec<Value> = list.clone();
        let mut next_placeholder = u16::MAX as u64;

        for entry in entries.as_sequence().unwrap() {
            let text = |field: &str| entry.get(field).and_then(|value| value.as_str());
            let name = text("name").expect("fork_opcodes.yml: an entry has no name");
            let size = entry
                .get("size")
                .and_then(|size| size.as_u64())
                .unwrap_or_else(|| panic!("fork_opcodes.yml: {name} has no size"));
            let opcode = entry.get("opcode").and_then(|opcode| opcode.as_u64());
            let found_on = text("found_on").map(|found_on| found_on.to_string());
            let how_to_find = text("how_to_find").unwrap_or_default().to_string();
            let mut comment = text("comment").unwrap_or_default().to_string();

            // A correction of upstream's opcode for this version (`corrects`: the opcode upstream still has.) It only
            // applies while upstream has that number, so their next update takes over again by itself.
            if let Some(wrong) = entry.get("corrects").and_then(|wrong| wrong.as_u64()) {
                let upstream = list
                    .iter_mut()
                    .find(|known| known.get("name").and_then(|known| known.as_str()) == Some(name));
                let Some(upstream) = upstream else {
                    println!(
                        "cargo::warning=fork_opcodes.yml corrects {kind}::{name}, which upstream's opcodes.yml doesn't have: remove its `corrects`."
                    );
                    continue;
                };
                let theirs = upstream.get("opcode").and_then(|opcode| opcode.as_u64());
                match (theirs, opcode) {
                    (Some(theirs), Some(ours))
                        if theirs == wrong && found_on.as_deref() == Some(version.as_str()) =>
                    {
                        upstream["opcode"] = ours.into();
                        used.retain(|used| *used != theirs);
                        used.push(ours);
                    }
                    _ => println!(
                        "cargo::warning=fork_opcodes.yml's correction of {kind}::{name} no longer applies (upstream has {theirs:?}, the correction was of {wrong} on {}): remove it.",
                        found_on.as_deref().unwrap_or("no version"),
                    ),
                }
                continue;
            }

            if names.iter().any(|known| known == name) {
                println!(
                    "cargo::warning=Upstream's opcodes.yml now has {kind}::{name}, so its entry in fork_opcodes.yml is ignored and can be removed."
                );
                continue;
            }

            let current = opcode.is_some() && found_on.as_deref() == Some(version.as_str());
            let used_opcode = if current {
                opcode.unwrap()
            } else {
                println!(
                    "cargo::warning={kind}::{name} needs finding for {version} (last found: {} on {}). How: {how_to_find}",
                    opcode
                        .map(|opcode| opcode.to_string())
                        .unwrap_or("never".to_string()),
                    found_on.as_deref().unwrap_or("no version"),
                );
                comment.push_str(&format!(" OUTSTANDING: needs finding for {version}."));
                while used.contains(&next_placeholder) {
                    next_placeholder -= 1;
                }
                next_placeholder
            };
            used.push(used_opcode);

            let mut mapping = serde_yaml_ng::Mapping::new();
            mapping.insert("name".into(), name.into());
            mapping.insert("comment".into(), comment.trim().into());
            mapping.insert("opcode".into(), used_opcode.into());
            mapping.insert("size".into(), size.into());
            list.push(Value::Mapping(mapping));

            merged.push(ForkOpcode {
                kind: kind.to_string(),
                name: name.to_string(),
                size,
                opcode,
                found_on,
                current,
                how_to_find,
                used_opcode,
                finders: fork_finders(entry, name, &known_entries),
            });
        }
    }
    merged
}

/// The generated `FORK_OPCODES` list, for the server to say which opcodes still need finding and what to do.
fn fork_opcodes_code(fork_opcodes: &[ForkOpcode], opcodes: &Value) -> String {
    let mut code = String::from(
        "/// One of the fork's own opcodes, from `resources/data/fork_opcodes.yml`.
#[derive(Debug, Clone, Copy)]
pub struct ForkOpcode {
    /// The opcode list it's in, e.g. ClientZoneIpcType.
    pub kind: &'static str,
    pub name: &'static str,
    /// The size of its data, without headers.
    pub size: u32,
    /// The opcode as it was last found, if ever.
    pub opcode: Option<u16>,
    /// The game version it was last found on.
    pub found_on: Option<&'static str>,
    /// Whether it's known for the supported game version. If not, a placeholder opcode stands in for it.
    pub current: bool,
    /// What to do in game to make the client send (or the server need) it.
    pub how_to_find: &'static str,
    /// The opcode the generated enum uses for it (a placeholder, if it isn't current.)
    pub used_opcode: u16,
    /// How to read it from the game's executable.
    pub finders: &'static [crate::exe_scan::Finder],
}

/// The fork's own opcodes.
pub const FORK_OPCODES: &[ForkOpcode] = &[
",
    );
    for opcode in fork_opcodes {
        code.push_str(&format!(
            "ForkOpcode {{ kind: {:?}, name: {:?}, size: {}, opcode: {:?}, found_on: {:?}, current: {}, how_to_find: {:?}, used_opcode: {}, finders: &[{}] }},\n",
            opcode.kind,
            opcode.name,
            opcode.size,
            opcode.opcode.map(|opcode| opcode as u16),
            opcode.found_on.as_deref(),
            opcode.current,
            opcode.how_to_find,
            opcode.used_opcode,
            opcode.finders.join(", "),
        ));
    }
    code.push_str(
        "];

/// The fork opcodes still to be found for this game version that a packet in `kind` (e.g. ClientZoneIpcType) with
/// `size` bytes of data could be.
pub fn outstanding_fork_opcodes(kind: &str, size: usize) -> impl Iterator<Item = &'static ForkOpcode> + '_ {
    FORK_OPCODES
        .iter()
        .filter(move |opcode| !opcode.current && opcode.kind == kind && opcode.size as usize == size)
}
",
    );

    // Every opcode in use in each list (placeholders included), so one read from the executable can't take one that's
    // known, and finders can be checked against upstream's.
    code.push_str(
        "\n/// Every opcode in use: (list, name, opcode.) Outstanding fork opcodes have their placeholder.\npub const KNOWN_OPCODES: &[(&str, &str, u16)] = &[\n",
    );
    for (kind, list) in opcodes.as_mapping().unwrap() {
        for opcode in list.as_sequence().unwrap() {
            let (Some(name), Some(number)) = (
                opcode.get("name").and_then(|name| name.as_str()),
                opcode.get("opcode").and_then(|number| number.as_u64()),
            ) else {
                continue;
            };
            code.push_str(&format!(
                "({:?}, {name:?}, {number}),\n",
                kind.as_str().unwrap()
            ));
        }
    }
    code.push_str(
        "];

/// Whether `opcode` is already some packet's opcode in the `kind` opcode list (e.g. ClientZoneIpcType.)
pub fn is_known_opcode(kind: &str, opcode: u16) -> bool {
    KNOWN_OPCODES.iter().any(|(k, _, number)| *k == kind && *number == opcode)
}
",
    );

    // Each packet's size, for matching opcodes to a new game version (opcode_match.)
    code.push_str(
        "\n/// Every packet's size (its data, without headers): (list, name, size.)\npub const OPCODE_SIZES: &[(&str, &str, u32)] = &[\n",
    );
    for (kind, list) in opcodes.as_mapping().unwrap() {
        for opcode in list.as_sequence().unwrap() {
            let (Some(name), Some(size)) = (
                opcode.get("name").and_then(|name| name.as_str()),
                opcode.get("size").and_then(|size| size.as_u64()),
            ) else {
                continue;
            };
            code.push_str(&format!(
                "({:?}, {name:?}, {size}),\n",
                kind.as_str().unwrap()
            ));
        }
    }
    code.push_str(
        "];

/// The size of the packet `name` in the `kind` opcode list (its data, without headers.)
pub fn opcode_size(kind: &str, name: &str) -> Option<usize> {
    OPCODE_SIZES.iter().find(|(k, n, _)| *k == kind && *n == name).map(|(_, _, size)| *size as usize)
}
",
    );
    code
}

fn main() {
    // Add link search directory for Oodle
    println!(
        "cargo:rustc-link-search={}/../oodle",
        env!("CARGO_MANIFEST_DIR")
    );

    // Generate IPC opcodes
    {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("../resources/data/opcodes.yml");

        println!("cargo::rerun-if-changed={}", d.to_str().unwrap());

        let mut output_str =
            "//!This file is automatically generated from `resources/data/opcodes.yml`! Do not edit this manually or commit to the repository.\nuse binrw::binrw;\nuse crate::packet::PredefinedOpcode;\n".to_string();

        let opcodes_buffer = std::fs::read_to_string(d).unwrap();
        let mut json: Value = serde_yaml_ng::from_str(&opcodes_buffer).unwrap();
        // Our fork's own opcodes (fork_opcodes.yml) join upstream's.
        let fork_opcodes = merge_fork_opcodes(&mut json);
        for element in json.as_mapping().unwrap() {
            let key = element.0.as_str().unwrap_or_default();
            let opcodes = element.1.as_sequence().unwrap();
            let mut seen_opcodes = Vec::new();

            let mut output_md = String::new();

            output_md.push_str("| Opcode | Size (in bytes) | Name | Comment |\n");
            output_md.push_str("| --- | --- | --- | --- |\n");

            // beginning
            output_str.push_str("#[binrw]\n");
            output_str.push_str("#[derive(Clone, PartialEq, Debug)]\n");
            output_str.push_str(&format!("pub enum {key} {{\n"));

            for opcode in opcodes {
                let opcode = opcode.as_mapping().unwrap();
                let size = opcode.get("size").unwrap().as_u64().unwrap();
                let name = opcode.get("name").unwrap().as_str().unwrap();
                let comment = if let Some(comment) = opcode.get("comment") {
                    let comment = comment.as_str().unwrap_or_default();
                    output_str.push_str(&format!("/// {comment}\n"));

                    comment
                } else {
                    ""
                };

                let opcode = opcode.get("opcode").unwrap().as_u64().unwrap();

                if !seen_opcodes.contains(&opcode) {
                    seen_opcodes.push(opcode);
                } else {
                    panic!("{name} has the same opcode as another, this isn't possible!");
                }

                output_str.push_str(&format!("#[brw(magic = {opcode}u16)]\n"));
                output_str.push_str(&format!("{name},\n"));

                // NOTE: This makes the assumption that the Data type is just the key but with Data at the end!
                let data_type_name = key.replace("Type", "Data");

                let module = if key.contains("Chat") {
                    "chat::"
                } else if key.contains("Zone") {
                    "zone::"
                } else if key.contains("Lobby") {
                    "lobby::"
                } else {
                    "" // For the custom IPC I guess?
                };

                output_md.push_str(&format!("| {opcode} (0x{opcode:X}) | {size} | [{name}](self::{module}{data_type_name}::{name}) | {comment} |\n"));
            }

            output_str.push_str("Unknown(u16),\n");

            // end
            output_str.push_str("}\n\n");

            output_str.push_str(&format!("impl Default for {key} {{\n"));
            output_str.push_str("fn default () -> Self {\n");
            output_str.push_str("Self::Unknown(0)\n");
            output_str.push_str("}\n");
            output_str.push_str("}\n");

            // Used for item ops/mail constants because const fns in traits doesn't exist yet
            output_str.push_str(&format!("impl {key} {{\n"));
            output_str.push_str("/// Returns the integer opcode.\n");
            output_str.push_str("pub(crate) const fn get_opcode(&self) -> u16 {\n");
            output_str.push_str("match self {\n");

            for opcode in opcodes {
                let opcode = opcode.as_mapping().unwrap();
                let name = opcode.get("name").unwrap().as_str().unwrap();
                let opcode = opcode.get("opcode").unwrap().as_u64().unwrap();

                output_str.push_str(&format!("{key}::{name} => {opcode},\n"));
            }

            output_str.push_str(&format!("{key}::Unknown(opcode) => *opcode,\n"));

            output_str.push_str("}\n\n");
            output_str.push_str("}\n\n");
            output_str.push_str("}\n");

            output_str.push_str(&format!("impl PredefinedOpcode for {key} {{\n"));

            // sizes
            output_str.push_str("fn calc_size(&self) -> u32 {\n");
            output_str.push_str("match self {\n");

            for opcode in opcodes {
                let opcode = opcode.as_mapping().unwrap();
                let name = opcode.get("name").unwrap().as_str().unwrap();
                let size = opcode.get("size").unwrap().as_u64().unwrap();

                output_str.push_str(&format!("{key}::{name} => {size},\n"));
            }

            output_str.push_str(&format!("{key}::Unknown(_) => unreachable!(),\n"));

            output_str.push_str("}\n\n");
            output_str.push_str("}\n\n");

            // names
            output_str.push_str("fn get_name(&self) -> &'static str {\n");
            output_str.push_str("match self {\n");

            for opcode in opcodes {
                let opcode = opcode.as_mapping().unwrap();
                let name = opcode.get("name").unwrap().as_str().unwrap();

                output_str.push_str(&format!("{key}::{name} => \"{name}\",\n"));
            }

            output_str.push_str(&format!("{key}::Unknown(_) => \"Unknown\",\n"));

            output_str.push_str("}\n\n");
            output_str.push_str("}\n\n");

            // opcodes
            output_str.push_str("/// Returns the integer opcode.\n");
            output_str.push_str("fn get_opcode(&self) -> u16 {\n");
            output_str.push_str("self.get_opcode()\n");
            output_str.push_str("}\n\n");

            // comments
            output_str.push_str("fn get_comment(&self) -> Option<&'static str> {\n");
            output_str.push_str("match self {\n");

            for opcode in opcodes {
                let opcode = opcode.as_mapping().unwrap();
                let name = opcode.get("name").unwrap().as_str().unwrap();

                if let Some(comment) = opcode.get("comment") {
                    let comment = comment.as_str().unwrap_or_default();
                    output_str.push_str(&format!("{key}::{name} => Some(\"{comment}\"),\n"));
                }
            }

            output_str.push_str("_ => None,\n");

            output_str.push_str("}\n\n");
            output_str.push_str("}\n\n");

            // end impl
            output_str.push_str("}\n\n");

            std::fs::write(format!("src/doc_{}.md", key.to_lowercase()), output_md)
                .expect("Failed to write opcodes file!");
        }

        output_str.push_str(&fork_opcodes_code(&fork_opcodes, &json));

        std::fs::write("src/opcodes.rs", output_str).expect("Failed to write opcodes file!");
    }

    // Generate constants
    {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("../resources/data/constants.yml");

        println!("cargo::rerun-if-changed={}", d.to_str().unwrap());

        let mut type_map: HashMap<&str, &str> = HashMap::new();
        type_map.insert("LOBBY_GAME_VERSION", "u32");
        type_map.insert("OBFUSCATION_ENABLED_MODE", "u8");
        type_map.insert("OBFUSCATION_TABLE_RADIXES", "[i32; 3]");
        type_map.insert("OBFUSCATION_TABLE_MAX", "[i32; 3]");
        type_map.insert("SUPPORTED_BOOT_VERSION", "&str");
        type_map.insert("SUPPORTED_GAME_VERSION", "&str");
        type_map.insert("LEVEL_MAIN", "[u32; 100]");

        let mut output_str =
            "//! This file is automatically generated from `resources/data/constants.yml`! Do not edit it manually or commit to the repository.\n"
                .to_string();

        let constants_buffer = std::fs::read_to_string(d).unwrap();
        let yml: Value = serde_yaml_ng::from_str(&constants_buffer).unwrap();
        for (key, value) in yml.as_mapping().unwrap() {
            let mut value_type = type_map
                .get(key.clone().as_str().unwrap())
                .unwrap_or(&"usize")
                .to_string();

            // This is special because it also defines how many expansions we support.
            if key == "SUPPORTED_EXPAC_VERSIONS" {
                let max_expansions = value.as_sequence().unwrap().len();
                output_str.push_str(&format!(
                    "pub const MAX_EXPANSION: usize = {max_expansions};\n"
                ));
                value_type = format!("[&str; {max_expansions}]");
            }

            let value_display = format_value(value);

            output_str.push_str(&format!(
                "pub const {}: {value_type} = {};\n",
                key.as_str().unwrap(),
                value_display
            ));
        }

        std::fs::write("src/constants.rs", output_str).expect("Failed to write constants file!");
    }

    // Generate festivals
    {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("../resources/data/festivals.yml");

        println!("cargo::rerun-if-changed={}", d.to_str().unwrap());

        let mut output_str =
            "//! This file is automatically generated from `resources/data/festivals.yml`! Do not edit it manually or commit to the repository.\n"
                .to_string();

        output_str.push_str("pub fn festival_name(id: u16) -> Option<&'static str> {\n");
        output_str.push_str("match id {\n");

        let festivals_buffer = std::fs::read_to_string(d).unwrap();
        let yml: Value = serde_yaml_ng::from_str(&festivals_buffer).unwrap();
        for value in yml.as_sequence().unwrap() {
            output_str.push_str(&format!(
                "{} => Some(\"{}\"),\n",
                value.get("Id").unwrap().as_i64().unwrap(),
                value.get("Name").unwrap().as_str().unwrap()
            ));
        }

        output_str.push_str("_ => None,\n");
        output_str.push_str("}\n");
        output_str.push_str("}\n");

        output_str.push_str("pub fn festival_list() -> &'static [(u16, &'static str)] {\n");
        output_str.push_str("&[\n");
        for value in yml.as_sequence().unwrap() {
            output_str.push_str(&format!(
                "({}, \"{}\"),\n",
                value.get("Id").unwrap().as_i64().unwrap(),
                value.get("Name").unwrap().as_str().unwrap()
            ));
        }
        output_str.push_str("]\n");
        output_str.push_str("}\n");

        std::fs::write("src/festivals.rs", output_str).expect("Failed to write constants file!");
    }
}
