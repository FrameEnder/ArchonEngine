//! Finding the fork's own opcodes (`resources/data/fork_opcodes.yml`) again after a game update: by reading them out
//! of the game's executable when the server starts, or with help from the server when the game sends one.

use std::path::Path;
use parking_lot::Mutex;

use kawari::constants::SUPPORTED_GAME_VERSION;
use kawari::exe_scan::{Exe, Finder, parse_pattern};
use kawari::opcodes::{FORK_OPCODES, ForkOpcode, is_known_opcode, outstanding_fork_opcodes};
use kawari::packet::remap;

/// How deep to follow calls back from a handler to the switch over opcodes.
const HANDLER_SEARCH_DEPTH: usize = 4;

/// Opcodes read from the executable this run: (name, opcode).
static FOUND: Mutex<Vec<(&'static str, u16)>> = Mutex::new(Vec::new());

fn found(opcode: &ForkOpcode) -> Option<u16> {
    FOUND
        .lock()
        .iter()
        .find(|(name, _)| *name == opcode.name)
        .map(|(_, found)| *found)
}

/// The fork opcodes that still aren't known for this game version (not in fork_opcodes.yml, nor read this run.)
pub fn outstanding() -> impl Iterator<Item = &'static ForkOpcode> {
    FORK_OPCODES
        .iter()
        .filter(|opcode| !opcode.current && found(opcode).is_none())
}

/// Reads the fork's opcodes out of the game's executable in `game_path`: outstanding ones are used from then on, and
/// known ones check their finders still work.
pub fn scan_executable(game_path: &str) {
    if FORK_OPCODES.iter().all(|opcode| opcode.finders.is_empty()) {
        return;
    }

    let game = Path::new(game_path);
    let exe_path = game.join("ffxiv_dx11.exe");
    if !exe_path.exists() {
        tracing::info!(
            "There's no ffxiv_dx11.exe in {game_path}, so the fork's opcodes can't be read from it."
        );
        return;
    }
    // Opcodes read from a different version would be wrong.
    let exe_version = std::fs::read_to_string(game.join("ffxivgame.ver")).unwrap_or_default();
    if exe_version.trim() != SUPPORTED_GAME_VERSION {
        tracing::warn!(
            "The game in {game_path} is version {:?}, not {SUPPORTED_GAME_VERSION}, so the fork's opcodes aren't read from it.",
            exe_version.trim()
        );
        return;
    }
    let exe = match Exe::open(&exe_path) {
        Ok(exe) => exe,
        Err(err) => {
            tracing::warn!("Couldn't read the fork's opcodes from the game: {err}");
            return;
        }
    };

    for opcode in FORK_OPCODES.iter().filter(|opcode| !opcode.finders.is_empty()) {
        let result = find_opcode(&exe, opcode);
        if opcode.current {
            match result {
                Ok(read) if Some(read) == opcode.opcode => {
                    tracing::info!("{}: the game agrees it's {read}.", opcode.name)
                }
                Ok(read) => tracing::warn!(
                    "{}: fork_opcodes.yml says {}, but the game reads as {read}. Its finder may need updating.",
                    opcode.name,
                    opcode.opcode.unwrap_or_default()
                ),
                Err(err) => tracing::warn!(
                    "{}: its finder doesn't work on this version anymore ({err}). It's still known, but should be updated.",
                    opcode.name
                ),
            }
            continue;
        }

        match result {
            Ok(read) => {
                remap::set_real_opcode(opcode.kind, opcode.used_opcode, read);
                FOUND.lock().push((opcode.name, read));
                tracing::info!(
                    "{}::{} read from the game: {read}. It's used from now on. To keep it, put `opcode: {read}` and `found_on: {SUPPORTED_GAME_VERSION}` in its entry in resources/data/fork_opcodes.yml.",
                    opcode.kind,
                    opcode.name
                );
            }
            Err(err) => tracing::warn!(
                "{}::{} couldn't be read from the game: {err}",
                opcode.kind,
                opcode.name
            ),
        }
    }
}

/// Tries `opcode`'s finders in order.
pub fn find_opcode(exe: &Exe, opcode: &ForkOpcode) -> Result<u16, String> {
    let mut errors = Vec::new();
    for finder in opcode.finders {
        match run_finder(exe, finder) {
            Ok(read) if is_known_opcode(opcode.kind, read) && Some(read) != opcode.opcode => {
                errors.push(format!("it read {read}, but that's already another packet's"))
            }
            Ok(read) => return Ok(read),
            Err(err) => errors.push(err),
        }
    }
    if errors.is_empty() {
        errors.push("it has no finders".to_string());
    }
    Err(errors.join("; "))
}

/// Reads an opcode with one finder.
pub fn run_finder(exe: &Exe, finder: &Finder) -> Result<u16, String> {
    match *finder {
        Finder::Immediate {
            pattern,
            offset,
            width,
        } => {
            let parsed = parse_pattern(pattern).ok_or(format!("bad pattern {pattern:?}"))?;
            let matches = exe.find(&parsed);
            if matches.is_empty() {
                return Err(format!("{pattern:?} isn't in the game anymore"));
            }
            let mut values: Vec<u32> = matches
                .iter()
                .filter_map(|at| match width {
                    2 => exe.u16_at(at + offset).map(u32::from),
                    _ => exe.i32_at(at + offset).map(|value| value as u32),
                })
                .collect();
            values.sort();
            values.dedup();
            match values.as_slice() {
                [value] => u16::try_from(*value).map_err(|_| format!("{value} isn't an opcode")),
                _ => Err(format!("{pattern:?} matches in places that disagree: {values:?}")),
            }
        }
        Finder::Handler { pattern, offset } => {
            let parsed = parse_pattern(pattern).ok_or(format!("bad pattern {pattern:?}"))?;
            let matches = exe.find(&parsed);
            let [at] = matches.as_slice() else {
                return Err(format!(
                    "{pattern:?} should match once, but matches {} times",
                    matches.len()
                ));
            };
            let handler = exe.function_start((*at as isize + offset) as usize);
            let cases = exe.opcode_cases(handler, HANDLER_SEARCH_DEPTH);
            // The main dispatcher is the switch with the most cases.
            let Some(biggest) = cases.iter().map(|case| case.switch_cases).max() else {
                return Err("no switch over opcodes leads to its handler".to_string());
            };
            let mut opcodes: Vec<i64> = cases
                .iter()
                .filter(|case| case.switch_cases == biggest)
                .flat_map(|case| case.opcodes.iter().copied())
                .collect();
            opcodes.sort();
            opcodes.dedup();
            match opcodes.as_slice() {
                [opcode] => u16::try_from(*opcode).map_err(|_| format!("{opcode} isn't an opcode")),
                _ => Err(format!("its handler is reached from several opcodes: {opcodes:?}")),
            }
        }
    }
}

/// Logs which fork opcodes still need finding (and how), when the server starts.
pub fn log_outstanding() {
    let mut any = false;
    for opcode in outstanding() {
        any = true;
        tracing::warn!(
            "{}::{} isn't known for {SUPPORTED_GAME_VERSION} yet (last found: {}), so it's switched off. To find it: {}",
            opcode.kind,
            opcode.name,
            last_found(opcode),
            opcode.how_to_find
        );
    }
    if any {
        tracing::warn!(
            "When you do that, the server says which unknown packet it was. See resources/data/fork_opcodes.yml."
        );
    }
}

/// When an unknown packet arrives, says which outstanding fork opcode it could be, going by its size.
/// `kind` is the opcode list, e.g. ClientZoneIpcType, and `size` the size of its data.
pub fn hint_unknown_packet(kind: &str, opcode: u16, size: usize) {
    for candidate in outstanding_fork_opcodes(kind, size).filter(|candidate| found(candidate).is_none()) {
        tracing::warn!(
            "Unknown packet {opcode} is the size of {} ({size} bytes), which needs finding for {SUPPORTED_GAME_VERSION}. If you just did this: {} -- then it's probably that. Put `opcode: {opcode}` and `found_on: {SUPPORTED_GAME_VERSION}` in its entry in resources/data/fork_opcodes.yml.",
            candidate.name,
            candidate.how_to_find
        );
    }
}

/// A line per fork opcode, for the !forkopcodes command.
pub fn status_lines() -> Vec<String> {
    if FORK_OPCODES.is_empty() {
        return vec!["The fork has no opcodes of its own.".to_string()];
    }
    FORK_OPCODES
        .iter()
        .map(|opcode| {
            if opcode.current {
                format!(
                    "{} ({}): {} -- found for this version.",
                    opcode.name,
                    opcode.kind,
                    opcode.opcode.unwrap_or_default()
                )
            } else if let Some(read) = found(opcode) {
                format!(
                    "{} ({}): {read} -- read from the game this run (not yet saved in fork_opcodes.yml.)",
                    opcode.name, opcode.kind
                )
            } else {
                format!(
                    "{} ({}): NEEDS FINDING (last found: {}). {}",
                    opcode.name,
                    opcode.kind,
                    last_found(opcode),
                    opcode.how_to_find
                )
            }
        })
        .collect()
}

fn last_found(opcode: &ForkOpcode) -> String {
    match (opcode.opcode, opcode.found_on) {
        (Some(number), Some(version)) => format!("{number} on {version}"),
        (Some(number), None) => format!("{number}, version unknown"),
        _ => "never".to_string(),
    }
}
