//! Every opcode, found again after a game update without anyone looking them up (docs/src/fork/opcodes.md).
//!
//! When the game installed in the game folder is newer than SUPPORTED_GAME_VERSION, the opcode fingerprints of the
//! last known version (`kawari::opcode_match`) are matched against its `ffxiv_dx11.exe`, and the opcodes found replace
//! the zone lists' opcodes (`kawari::packet::remap::set_table`). The result is written to `auto-opcodes/<version>.json`,
//! which tells the patch server that version is supported and is what to save in the opcode files. Client packets
//! the match couldn't place are learnt from the first unknown packet of their size, when only one fits.
//!
//! When the installed game is SUPPORTED_GAME_VERSION, its fingerprints are recorded instead (if they're missing or
//! for another version), so the next update finds them.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use parking_lot::Mutex;
use serde::Serialize;

use kawari::constants::SUPPORTED_GAME_VERSION;
use kawari::exe_scan::Exe;
use kawari::opcode_match::{self, Fingerprints, Found, KnownOpcode};
use kawari::opcodes::KNOWN_OPCODES;
use kawari::packet::remap;

/// The fingerprints shipped with the server (recorded on SUPPORTED_GAME_VERSION.)
const SHIPPED: &str = "resources/data/opcode_fingerprints.json";
/// Where results, and fingerprints recorded since, are written.
const FOLDER: &str = opcode_match::RESULTS_FOLDER;
/// The zone lists: everything else (lobby, chat) hasn't changed between versions.
const LISTS: [&str; 2] = ["ServerZoneIpcType", "ClientZoneIpcType"];

/// Client packets not found by the match, waiting to be learnt by size: (generated opcode, name, size.)
static UNPLACED: Mutex<Vec<(u16, &'static str, usize)>> = Mutex::new(Vec::new());

#[derive(Serialize)]
struct Result {
    game_version: String,
    /// The expansions' versions in this game install (for the patch server.)
    expansions: Vec<String>,
    from_fingerprints_of: String,
    /// list → name → the game's opcode.
    opcodes: BTreeMap<String, BTreeMap<String, u16>>,
    /// Opcodes taken from candidates the client's code doesn't tell apart: list → name → (taken, candidates.)
    guessed: BTreeMap<String, BTreeMap<String, (u16, Vec<u16>)>>,
    /// Not found: list → names.
    not_found: BTreeMap<String, Vec<String>>,
}

fn game_version(game: &Path) -> String {
    std::fs::read_to_string(game.join("ffxivgame.ver"))
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn expansions(game: &Path) -> Vec<String> {
    (1..=9)
        .map_while(|n| {
            std::fs::read_to_string(game.join(format!("sqpack/ex{n}/ex{n}.ver")))
                .ok()
                .map(|version| version.trim().to_string())
        })
        .collect()
}

fn version_key(version: &str) -> Vec<u32> {
    version
        .split('.')
        .map(|part| part.parse().unwrap_or(0))
        .collect()
}

/// The newest fingerprints older than `version`: those recorded here since, or the shipped ones.
fn newest_fingerprints(version: &str, shipped: &Path) -> Option<(PathBuf, Fingerprints)> {
    let mut found: Vec<(PathBuf, Fingerprints)> = Vec::new();
    let mut paths = vec![shipped.to_path_buf()];
    if let Ok(entries) = std::fs::read_dir(FOLDER) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("fingerprints-") && name.ends_with(".json") {
                paths.push(entry.path());
            }
        }
    }
    for path in paths {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        match serde_json::from_str::<Fingerprints>(&text) {
            Ok(prints) if version_key(&prints.game_version) < version_key(version) => {
                found.push((path, prints))
            }
            Ok(_) => {}
            Err(err) => tracing::warn!(
                "{} isn't readable opcode fingerprints: {err}",
                path.display()
            ),
        }
    }
    found
        .into_iter()
        .max_by_key(|(_, prints)| version_key(&prints.game_version))
}

/// The zone opcodes the server knows (as generated), with their sizes.
fn known() -> Vec<KnownOpcode> {
    let sizes = sizes();
    KNOWN_OPCODES
        .iter()
        .filter(|(kind, _, _)| LISTS.contains(kind))
        .filter_map(|(kind, name, opcode)| {
            sizes.get(&(*kind, *name)).map(|size| KnownOpcode {
                kind: kind.to_string(),
                name: name.to_string(),
                opcode: *opcode,
                size: *size as u32,
            })
        })
        .collect()
}

/// Each zone packet's size (without headers), from the opcode files the server was built with.
fn sizes() -> BTreeMap<(&'static str, &'static str), usize> {
    let mut sizes = BTreeMap::new();
    for (kind, name, _) in KNOWN_OPCODES
        .iter()
        .filter(|(kind, _, _)| LISTS.contains(kind))
    {
        if let Some(size) = kawari::opcodes::opcode_size(kind, name) {
            sizes.insert((*kind, *name), size);
        }
    }
    sizes
}

/// Matches the opcodes to the game in `game_path`, if it's newer than the server's opcode files, or records its
/// fingerprints if it's the version they're for.
pub fn adapt_to_game(game_path: &str) {
    adapt(game_path, SUPPORTED_GAME_VERSION, Path::new(SHIPPED));
}

/// `adapt_to_game` with the server's version and shipped fingerprints given (for tests.)
fn adapt(game_path: &str, supported: &str, shipped: &Path) -> Option<Result> {
    let game = Path::new(game_path);
    let version = game_version(game);
    if version.is_empty() {
        return None;
    }
    let exe_path = game.join("ffxiv_dx11.exe");
    if version == supported {
        record_fingerprints(&exe_path, &version, shipped);
        return None;
    }
    if version_key(&version) < version_key(supported) {
        tracing::warn!(
            "The game in {game_path} is {version}, older than {supported}: its opcodes aren't looked for."
        );
        return None;
    }

    let Some((prints_path, prints)) = newest_fingerprints(&version, shipped) else {
        tracing::warn!(
            "The game in {game_path} is {version}, newer than {supported}, but there are no opcode fingerprints to find its opcodes with ({} is missing.)",
            shipped.display()
        );
        return None;
    };
    let exe = match Exe::open(&exe_path) {
        Ok(exe) => exe,
        Err(err) => {
            tracing::warn!(
                "The game is {version}, but its executable can't be read to find its opcodes: {err}"
            );
            return None;
        }
    };
    tracing::info!(
        "The game is {version}, newer than {supported}: finding its opcodes with the fingerprints of {} ({})...",
        prints.game_version,
        prints_path.display()
    );
    let found = opcode_match::match_fingerprints(&exe, &prints);
    let result = apply(&version, game, &prints, &found);

    let exact: usize = result.opcodes.values().map(BTreeMap::len).sum();
    let guessed: usize = result.guessed.values().map(BTreeMap::len).sum();
    let missing: usize = result.not_found.values().map(Vec::len).sum();
    tracing::info!(
        "Opcodes for {version}: {} found, {guessed} of them taken from candidates the client handles alike, {missing} not found. Written to {FOLDER}/{version}.json.",
        exact
    );
    for (kind, names) in &result.guessed {
        for (name, (taken, candidates)) in names {
            tracing::warn!(
                "{kind}::{name}: the client's code doesn't tell {candidates:?} apart, so {taken} is used. If it doesn't work, try the others with !rawpacket."
            );
        }
    }
    for (kind, names) in &result.not_found {
        if kind.starts_with("Client") {
            tracing::warn!(
                "{kind}: not found yet, learnt from the first unknown packet of their size: {}",
                names.join(", ")
            );
        } else {
            tracing::warn!("{kind}: not found, switched off: {}", names.join(", "));
        }
    }

    // Record this version's fingerprints from what was found for sure, for the next update.
    let sure: Vec<KnownOpcode> = known()
        .into_iter()
        .filter_map(|mut k| {
            let real = *result.opcodes.get(&k.kind)?.get(&k.name)?;
            let guessed = result
                .guessed
                .get(&k.kind)
                .is_some_and(|names| names.contains_key(&k.name));
            (!guessed).then(|| {
                k.opcode = real;
                k
            })
        })
        .collect();
    write_fingerprints(&exe, &version, &sure);
    Some(result)
}

/// Installs what was found and writes it down.
fn apply(
    version: &str,
    game: &Path,
    prints: &Fingerprints,
    found: &BTreeMap<(String, String), Found>,
) -> Result {
    let mut result = Result {
        game_version: version.to_string(),
        expansions: expansions(game),
        from_fingerprints_of: prints.game_version.clone(),
        opcodes: BTreeMap::new(),
        guessed: BTreeMap::new(),
        not_found: BTreeMap::new(),
    };
    let sizes = sizes();

    // Packets that share candidates take different ones, in the order of their names.
    let mut sharing: BTreeMap<(String, Vec<u16>), Vec<String>> = BTreeMap::new();
    for ((kind, name), value) in found {
        if let Found::OneOf(candidates) = value {
            sharing
                .entry((kind.clone(), candidates.clone()))
                .or_default()
                .push(name.clone());
        }
    }
    let mut taken_from: BTreeMap<(String, String), u16> = BTreeMap::new();
    for ((kind, candidates), names) in &sharing {
        for (name, opcode) in names.iter().zip(candidates.iter()) {
            taken_from.insert((kind.clone(), name.clone()), *opcode);
        }
    }

    UNPLACED.lock().clear();
    for kind in LISTS {
        let mut pairs = Vec::new();
        for (k, name, ours) in KNOWN_OPCODES.iter().filter(|(k, _, _)| *k == kind) {
            let key = (kind.to_string(), name.to_string());
            let real = match found.get(&key) {
                Some(Found::Opcode(real)) => Some(*real),
                Some(Found::OneOf(candidates)) => taken_from.get(&key).copied().inspect(|taken| {
                    result
                        .guessed
                        .entry(kind.to_string())
                        .or_default()
                        .insert(name.to_string(), (*taken, candidates.clone()));
                }),
                _ => None,
            };
            match real {
                Some(real) => {
                    pairs.push((*ours, real));
                    result
                        .opcodes
                        .entry(kind.to_string())
                        .or_default()
                        .insert(name.to_string(), real);
                }
                None => {
                    result
                        .not_found
                        .entry(kind.to_string())
                        .or_default()
                        .push(name.to_string());
                    if kind.starts_with("Client")
                        && let Some(size) = sizes.get(&(*k, *name))
                    {
                        UNPLACED.lock().push((*ours, name, *size));
                    }
                }
            }
        }
        remap::set_table(kind, &pairs);
    }

    let path = Path::new(FOLDER).join(format!("{version}.json"));
    let written = std::fs::create_dir_all(FOLDER).and_then(|_| {
        std::fs::write(
            &path,
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )
    });
    if let Err(err) = written {
        tracing::warn!("Couldn't write {}: {err}", path.display());
    }
    result
}

fn record_fingerprints(exe_path: &Path, version: &str, shipped: &Path) {
    let shipped_is_current = std::fs::read_to_string(shipped)
        .ok()
        .and_then(|text| serde_json::from_str::<Fingerprints>(&text).ok())
        .is_some_and(|prints| prints.game_version == version);
    let recorded = Path::new(FOLDER).join(format!("fingerprints-{version}.json"));
    if shipped_is_current || recorded.exists() {
        return;
    }
    let Ok(exe) = Exe::open(exe_path) else { return };
    // Only fork opcodes found for this version count (outstanding ones have placeholders.)
    let known: Vec<KnownOpcode> = known()
        .into_iter()
        .filter(|k| {
            kawari::opcodes::FORK_OPCODES
                .iter()
                .find(|fork| fork.kind == k.kind && fork.name == k.name)
                .is_none_or(|fork| fork.current)
        })
        .collect();
    write_fingerprints(&exe, version, &known);
}

fn write_fingerprints(exe: &Exe, version: &str, known: &[KnownOpcode]) {
    match opcode_match::fingerprint(exe, version, known) {
        Ok(prints) => {
            let path = Path::new(FOLDER).join(format!("fingerprints-{version}.json"));
            let written = std::fs::create_dir_all(FOLDER).and_then(|_| {
                std::fs::write(&path, serde_json::to_string(&prints).unwrap_or_default())
            });
            match written {
                Ok(()) => tracing::info!(
                    "Recorded the opcode fingerprints of {version} in {} (for finding the opcodes after the next game update.)",
                    path.display()
                ),
                Err(err) => tracing::warn!("Couldn't write {}: {err}", path.display()),
            }
        }
        Err(err) => tracing::warn!("Couldn't record the opcode fingerprints of {version}: {err}"),
    }
}

/// An unknown client packet arrived (its game opcode and size): if only one packet not found yet has that size, it's
/// that one from now on.
pub fn learn_unknown_packet(kind: &str, opcode: u16, size: usize) {
    if !remap::has_table(kind) {
        return;
    }
    let opcode = remap::real_unknown(opcode);
    let mut unplaced = UNPLACED.lock();
    let fitting: Vec<usize> = unplaced
        .iter()
        .enumerate()
        .filter(|(_, (_, _, wanted))| *wanted == size)
        .map(|(i, _)| i)
        .collect();
    match fitting.as_slice() {
        [i] => {
            let (ours, name, _) = unplaced[*i];
            if remap::add_to_table(kind, ours, opcode) {
                unplaced.remove(*i);
                tracing::info!(
                    "{kind}::{name} is {opcode} on this game version (the only packet not found yet that's {size} bytes); used from now on. The next one is understood."
                );
            }
        }
        [] => {}
        many => tracing::info!(
            "Unknown packet {opcode} ({size} bytes) could be any of {:?}, which aren't found yet.",
            many.iter().map(|i| unplaced[*i].1).collect::<Vec<_>>()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// With a game folder (KAWARI_TEST_GAME, a version newer than KAWARI_TEST_SUPPORTED) and that older version's
    /// fingerprints (KAWARI_TEST_FINGERPRINTS): every opcode found is the one this build has, when the build's opcode
    /// files are for the game's version. Needs Square Enix's executable, so it's not run by default:
    /// `cargo test -p kawari-world adapts_to_a_newer_game -- --ignored`.
    #[test]
    #[ignore]
    fn adapts_to_a_newer_game() {
        let (Ok(game), Ok(prints), Ok(supported)) = (
            std::env::var("KAWARI_TEST_GAME"),
            std::env::var("KAWARI_TEST_FINGERPRINTS"),
            std::env::var("KAWARI_TEST_SUPPORTED"),
        ) else {
            panic!("set KAWARI_TEST_GAME, KAWARI_TEST_FINGERPRINTS and KAWARI_TEST_SUPPORTED");
        };
        let work = std::env::temp_dir().join("kawari-auto-opcodes-test");
        let _ = std::fs::remove_dir_all(&work);
        std::fs::create_dir_all(&work).unwrap();
        std::env::set_current_dir(&work).unwrap();

        let result =
            adapt(&game, &supported, Path::new(&prints)).expect("the opcodes were looked for");
        let mut checked = 0;
        for (kind, names) in &result.opcodes {
            for (name, real) in names {
                if result
                    .guessed
                    .get(kind)
                    .is_some_and(|guessed| guessed.contains_key(name))
                {
                    continue;
                }
                let ours = KNOWN_OPCODES
                    .iter()
                    .find(|(k, n, _)| k == kind && n == name)
                    .unwrap()
                    .2;
                assert_eq!(*real, ours, "{kind}::{name}");
                checked += 1;
            }
        }
        println!(
            "{checked} opcodes as this build has them; {} guessed, {} not found",
            result.guessed.values().map(BTreeMap::len).sum::<usize>(),
            result.not_found.values().map(Vec::len).sum::<usize>()
        );
        // Installed: a found client opcode reads as itself, an opcode nothing has is unknown.
        assert!(remap::has_table("ClientZoneIpcType"));
        assert!(
            work.join(FOLDER)
                .join(format!("{}.json", result.game_version))
                .exists()
        );
        assert!(
            work.join(FOLDER)
                .join(format!("fingerprints-{}.json", result.game_version))
                .exists()
        );
        assert!(
            opcode_match::accepted_game_versions()
                .iter()
                .any(|(version, _)| *version == result.game_version)
        );
    }
}
