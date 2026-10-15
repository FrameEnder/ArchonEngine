//! Help finding the fork's own opcodes (`resources/data/fork_opcodes.yml`) again after a game update.

use kawari::constants::SUPPORTED_GAME_VERSION;
use kawari::opcodes::{FORK_OPCODES, ForkOpcode, outstanding_fork_opcodes};

/// The fork opcodes that still need finding for this game version.
pub fn outstanding() -> impl Iterator<Item = &'static ForkOpcode> {
    FORK_OPCODES.iter().filter(|opcode| !opcode.current)
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
    for candidate in outstanding_fork_opcodes(kind, size) {
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
