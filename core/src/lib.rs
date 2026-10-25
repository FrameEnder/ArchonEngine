//! A server replacement for a certain MMO.

/// Common functions, structures used in parsing and also useful for the servers.
/// Reads opcodes out of the game's executable.
pub mod exe_scan;

/// Finding every opcode again after a game update.
pub mod opcode_match;

/// Writing fork-opcode finders from an executable whose opcodes are known.
pub mod finder_gen;

pub mod common;

/// Config management.
#[cfg(feature = "server")]
pub mod config;

/// Everything packet parsing related.
pub mod packet;

#[rustfmt::skip]
#[doc(hidden)]
pub mod constants;
#[rustfmt::skip]
#[doc(hidden)]
pub mod opcodes;
#[rustfmt::skip]
#[doc(hidden)]
pub mod festivals;

/// IPC
pub mod ipc;
