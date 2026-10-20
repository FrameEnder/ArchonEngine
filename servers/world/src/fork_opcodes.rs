//! Finding the fork's own opcodes (`resources/data/fork_opcodes.yml`) again after a game update: by reading them out
//! of the game's executable when the server starts, or with help from the server when the game sends one.

use parking_lot::Mutex;
use std::path::Path;

use kawari::constants::SUPPORTED_GAME_VERSION;
use kawari::exe_scan::{Exe, check_opcode, find_opcode};
use kawari::opcodes::{FORK_OPCODES, ForkOpcode, outstanding_fork_opcodes};
use kawari::packet::remap;

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

    for opcode in FORK_OPCODES
        .iter()
        .filter(|opcode| !opcode.finders.is_empty())
    {
        if let (true, Some(known)) = (opcode.current, opcode.opcode) {
            match check_opcode(&exe, opcode, known) {
                Ok(()) => tracing::info!("{}: the game agrees it's {known}.", opcode.name),
                Err(err) => tracing::warn!(
                    "{}: fork_opcodes.yml says {known}, but its finder disagrees ({err}). It's still used, but the finder should be updated.",
                    opcode.name
                ),
            }
            continue;
        }

        match find_opcode(&exe, opcode) {
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
    // After a game update, a packet the opcode match didn't place may be learnt from it.
    crate::auto_opcodes::learn_unknown_packet(kind, opcode, size);
    let opcode = kawari::packet::remap::real_unknown(opcode);
    for candidate in
        outstanding_fork_opcodes(kind, size).filter(|candidate| found(candidate).is_none())
    {
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
