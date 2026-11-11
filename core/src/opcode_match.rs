//! Finds every opcode again after a game update, from the new `ffxiv_dx11.exe` alone (docs/src/fork/opcodes.md).
//!
//! While the opcodes are known (the game is SUPPORTED_GAME_VERSION and opcodes.yml and fork_opcodes.yml are current),
//! [`fingerprint`] records how the client's code treats each packet: for a server packet, the code its opcode leads to
//! in the client's zone dispatcher (and, where several opcodes share a case, further in); for a client packet, the
//! function that builds it. Only hashes of the instructions' shape (registers, small numbers, no addresses) are kept,
//! with where the code was, so nothing of the game is stored. After an update, [`match_fingerprints`] finds the same
//! code in the new executable and reads off the new opcodes.
//!
//! Server packets: the client reads the opcode out of the packet (`movzx r, word [packet + 2]`) and branches on it
//! with compares and jump tables. [`walk`] follows that code with a concrete opcode, keeping track of which registers
//! hold the opcode (or numbers made from it) and the packet pointer, and records each branch the opcode decides.
//! Where the path ends tells packets apart. Client packets: the code that builds one writes its opcode and then its
//! size with the IPC header next to it (`mov [rsp+x], opcode`, `mov [rsp+x+8], size + 16`), and every packet's size
//! is in opcodes.yml.
//!
//! Matching is exact first (the same code, the same number of opcodes sharing it), then by order: the game's
//! functions stay in the same order from one version to the next, so already matched code predicts where the rest
//! moved, and the closest code near there is taken. Nothing ambiguous is guessed: an opcode that can't be told apart
//! from others comes back as a list of candidates.

use std::collections::{BTreeMap, HashMap, HashSet};

use iced_x86::{
    Decoder, DecoderOptions, FlowControl, Instruction, InstructionInfoFactory, Mnemonic, OpAccess,
    OpKind, Register,
};
use serde_derive::{Deserialize, Serialize};

use crate::exe_scan::Exe;

/// Opcodes are below this.
const OPCODE_LIMIT: u16 = 0x400;
/// Server opcodes the dispatcher is walked with.
const SERVER_OPCODES: std::ops::Range<u16> = 1..OPCODE_LIMIT;
/// Client opcodes looked for where packets are built.
const CLIENT_OPCODES: std::ops::Range<i64> = 100..OPCODE_LIMIT as i64;
/// How many instructions a case's fingerprint covers (and a call in it, one level down.)
const CASE_LENGTH: usize = 40;
const CALLEE_LENGTH: usize = 30;
/// How many a path's end covers.
const END_LENGTH: usize = 60;
/// How many a function that builds a client packet covers.
const FUNCTION_LENGTH: usize = 300;
/// Numbers from here up are addresses or offsets that move between versions, and are left out of fingerprints.
const CASE_MASK_FROM: u64 = 0x1000;
const FUNCTION_MASK_FROM: u64 = 0x100;
/// How far from where the order predicts matched code may be, and how alike it must be.
const SERVER_ORDER_WINDOW: usize = 0x200;
const CLIENT_ORDER_WINDOW: usize = 0x40;
const ORDER_SIMILARITY: f32 = 0.5;
const CLOSE_SIMILARITY: f32 = 0.7;
const CLEAR_MARGIN: f32 = 0.05;

// --- the file -----------------------------------------------------------------------------------------------------

/// What's recorded about the opcodes of one game version (resources/data/opcode_fingerprints.json.)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Fingerprints {
    /// The game version they were recorded on.
    pub game_version: String,
    /// How they were made: since `OWN_OPCODE_FORMAT`, a number in a lone opcode's code equal to that opcode is
    /// recorded as "its opcode" (0 for files from before, which are compared without it.)
    #[serde(default)]
    pub format: u32,
    /// The zone dispatcher's start.
    pub dispatcher: Prints,
    pub dispatcher_at: usize,
    pub server: Vec<ServerPrint>,
    pub client: Vec<ClientPrint>,
}

/// Instruction hashes, written as one hex string.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Prints(pub Vec<u32>);

impl serde::Serialize for Prints {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let text: String = self.0.iter().map(|hash| format!("{hash:08x}")).collect();
        serializer.serialize_str(&text)
    }
}

impl<'de> serde::Deserialize<'de> for Prints {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = <String as serde::Deserialize>::deserialize(deserializer)?;
        if !text.len().is_multiple_of(8) {
            return Err(serde::de::Error::custom(
                "fingerprint length isn't a multiple of 8",
            ));
        }
        (0..text.len())
            .step_by(8)
            .map(|at| u32::from_str_radix(&text[at..at + 8], 16).map_err(serde::de::Error::custom))
            .collect::<Result<Vec<_>, _>>()
            .map(Prints)
    }
}

/// A server packet: its case in the dispatcher, where its path ends, and (if its case and end are shared) where a
/// second dispatcher sends it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPrint {
    pub kind: String,
    pub name: String,
    pub opcode: u16,
    pub case: Prints,
    /// How many opcodes share the case.
    pub case_group: usize,
    /// The function the case calls (or jumps to), for the order.
    pub handler_at: usize,
    pub end: Prints,
    /// How many of the case's opcodes share the end.
    pub end_group: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sub: Option<SubPrint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubPrint {
    pub end: Prints,
    pub group: usize,
}

/// A client packet: each function that builds it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientPrint {
    pub kind: String,
    pub name: String,
    pub opcode: u16,
    pub size: u32,
    pub sites: Vec<SitePrint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitePrint {
    pub function: Prints,
    pub function_at: usize,
}

/// A packet to fingerprint: its list (e.g. ServerZoneIpcType), name, opcode and size (without headers.)
#[derive(Debug, Clone)]
pub struct KnownOpcode {
    pub kind: String,
    pub name: String,
    pub opcode: u16,
    pub size: u32,
}

/// What matching found for one packet.
#[derive(Debug, Clone, PartialEq)]
pub enum Found {
    Opcode(u16),
    /// One of these; the client's code can't tell them apart.
    OneOf(Vec<u16>),
    NotFound,
}

/// Where the world server writes the opcodes it found for a newer game version (`<version>.json`), and fingerprints
/// it recorded (`fingerprints-<version>.json`), relative to where the servers run.
pub const RESULTS_FOLDER: &str = "auto-opcodes";

/// Game versions besides SUPPORTED_GAME_VERSION whose opcodes the world server found, with their expansions'
/// versions: the patch server accepts these as current.
pub fn accepted_game_versions() -> Vec<(String, Vec<String>)> {
    let Ok(entries) = std::fs::read_dir(RESULTS_FOLDER) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter(|entry| {
            !entry
                .file_name()
                .to_string_lossy()
                .starts_with("fingerprints-")
        })
        .filter_map(|entry| {
            let text = std::fs::read_to_string(entry.path()).ok()?;
            let value: serde_json::Value = serde_json::from_str(&text).ok()?;
            let version = value.get("game_version")?.as_str()?.to_string();
            let expansions = value
                .get("expansions")?
                .as_array()?
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect();
            Some((version, expansions))
        })
        .collect()
}

/// The zone opcodes in opcodes.yml and fork_opcodes.yml (their text), with their sizes. Fork entries without an
/// opcode, or not found on `game_version`, are left out.
pub fn known_from_yaml(
    opcodes: &str,
    fork_opcodes: &str,
    game_version: &str,
) -> Result<Vec<KnownOpcode>, String> {
    let mut known = Vec::new();
    for (text, fork) in [(opcodes, false), (fork_opcodes, true)] {
        let lists: serde_yaml_ng::Value =
            serde_yaml_ng::from_str(text).map_err(|err| err.to_string())?;
        let Some(lists) = lists.as_mapping() else {
            continue;
        };
        for (kind, entries) in lists {
            let Some(kind) = kind.as_str() else { continue };
            if kind != "ServerZoneIpcType" && kind != "ClientZoneIpcType" {
                continue;
            }
            for entry in entries.as_sequence().into_iter().flatten() {
                let (Some(name), Some(opcode), Some(size)) = (
                    entry.get("name").and_then(|v| v.as_str()),
                    entry.get("opcode").and_then(|v| v.as_u64()),
                    entry.get("size").and_then(|v| v.as_u64()),
                ) else {
                    continue;
                };
                if fork && entry.get("found_on").and_then(|v| v.as_str()) != Some(game_version) {
                    continue;
                }
                // A fork correction of upstream's opcode (`corrects`, the number upstream still has; core/build.rs.)
                if fork && let Some(wrong) = entry.get("corrects").and_then(|v| v.as_u64()) {
                    if let Some(upstream) = known.iter_mut().find(|k: &&mut KnownOpcode| {
                        k.kind == kind && k.name == name && k.opcode as u64 == wrong
                    }) {
                        upstream.opcode = opcode as u16;
                    }
                    continue;
                }
                // A name in both is upstream's.
                if known
                    .iter()
                    .any(|k: &KnownOpcode| k.kind == kind && k.name == name)
                {
                    continue;
                }
                known.push(KnownOpcode {
                    kind: kind.to_string(),
                    name: name.to_string(),
                    opcode: opcode as u16,
                    size: size as u32,
                });
            }
        }
    }
    Ok(known)
}

// --- reading code -------------------------------------------------------------------------------------------------

struct Code<'a> {
    exe: &'a Exe,
    cache: std::cell::RefCell<HashMap<usize, Option<Instruction>>>,
}

impl<'a> Code<'a> {
    fn new(exe: &'a Exe) -> Self {
        Self {
            exe,
            cache: Default::default(),
        }
    }

    fn at(&self, rva: usize) -> Option<Instruction> {
        if let Some(cached) = self.cache.borrow().get(&rva) {
            return *cached;
        }
        let (start, end) = self.exe.text();
        let decoded = (start..end).contains(&rva).then(|| {
            let bytes = &self.exe.image()[rva..(rva + 15).min(self.exe.image().len())];
            let mut decoder = Decoder::with_ip(64, bytes, rva as u64, DecoderOptions::NONE);
            decoder.decode()
        });
        let decoded = decoded.filter(|instruction| !instruction.is_invalid());
        self.cache.borrow_mut().insert(rva, decoded);
        decoded
    }

    fn in_text(&self, rva: u64) -> bool {
        let (start, end) = self.exe.text();
        (start as u64..end as u64).contains(&rva)
    }

    fn read(&self, rva: u64, size: usize, signed: bool) -> Option<u64> {
        let bytes = self.exe.image().get(rva as usize..rva as usize + size)?;
        let mut value = 0u64;
        for (i, byte) in bytes.iter().enumerate() {
            value |= (*byte as u64) << (8 * i);
        }
        if signed && size < 8 && value >> (size * 8 - 1) & 1 == 1 {
            value |= u64::MAX << (size * 8);
        }
        Some(value)
    }
}

/// FNV-1a, so fingerprints are the same on every machine and build.
fn hash(parts: &[u64]) -> u32 {
    let mut h: u32 = 0x811C_9DC5;
    for part in parts {
        for byte in part.to_le_bytes() {
            h ^= byte as u32;
            h = h.wrapping_mul(0x0100_0193);
        }
    }
    h
}

const MASKED: u64 = u64::MAX;
/// A number equal to the packet's own opcode (`token`.)
const OWN_OPCODE: u64 = u64::MAX - 2;
/// Fingerprints made with `OWN_OPCODE` (`Fingerprints::format`.) Before it, small handlers that pass the client's packet
/// code their own opcode and size (`mov r8d, opcode`, e.g. EnmityList's and HaterList's) were alike but never the same
/// from one version to the next, and the order put 24 server packets one function off on 7.58, where a new packet's
/// handler had been put in among them (2026-11-11.)
pub const OWN_OPCODE_FORMAT: u32 = 2;

fn is_immediate(kind: OpKind) -> bool {
    matches!(
        kind,
        OpKind::Immediate8
            | OpKind::Immediate8_2nd
            | OpKind::Immediate16
            | OpKind::Immediate32
            | OpKind::Immediate64
            | OpKind::Immediate8to16
            | OpKind::Immediate8to32
            | OpKind::Immediate8to64
            | OpKind::Immediate32to64
    )
}

/// An instruction's shape: what it does, its registers and small numbers, without addresses; a number equal to `own`
/// (the packet's opcode, when the code is that one packet's) as `OWN_OPCODE`.
fn token(instruction: &Instruction, mask_from: u64, own: Option<u16>) -> u32 {
    let small = |value: u64| {
        if own.is_some_and(|own| value == own as u64) {
            return OWN_OPCODE;
        }
        let magnitude = (value as i64).unsigned_abs();
        if magnitude < mask_from { value } else { MASKED }
    };
    let mut parts = vec![instruction.mnemonic() as u64, instruction.op_count() as u64];
    for i in 0..instruction.op_count() {
        match instruction.op_kind(i) {
            OpKind::Register => parts.push(instruction.op_register(i) as u64),
            OpKind::NearBranch16 | OpKind::NearBranch32 | OpKind::NearBranch64 => {
                parts.push(MASKED - 1)
            }
            OpKind::Memory => {
                parts.push(instruction.memory_size() as u64);
                parts.push(instruction.memory_base() as u64);
                parts.push(instruction.memory_index() as u64);
                parts.push(instruction.memory_index_scale() as u64);
                if instruction.is_ip_rel_memory_operand() {
                    parts.push(MASKED);
                } else {
                    parts.push(small(instruction.memory_displacement64()));
                }
            }
            OpKind::Immediate8
            | OpKind::Immediate8_2nd
            | OpKind::Immediate16
            | OpKind::Immediate32
            | OpKind::Immediate64
            | OpKind::Immediate8to16
            | OpKind::Immediate8to32
            | OpKind::Immediate8to64
            | OpKind::Immediate32to64 => parts.push(small(instruction.immediate(i))),
            other => parts.push(0x1000 + other as u64),
        }
    }
    hash(&parts)
}

/// The code from `rva` up to the first return, jump or int3 (at most `length` instructions); calls one level down
/// count as their callee's fingerprint, `depth` levels down. `own`: the opcode, when this is one packet's code.
fn fingerprint_code(
    code: &Code,
    rva: usize,
    depth: usize,
    length: usize,
    own: Option<u16>,
) -> Prints {
    let mut prints = Vec::new();
    let mut at = rva;
    while prints.len() < length {
        let Some(instruction) = code.at(at) else {
            break;
        };
        if instruction.mnemonic() == Mnemonic::Call
            && depth > 0
            && instruction.op0_kind() == OpKind::NearBranch64
        {
            let callee = fingerprint_code(
                code,
                instruction.near_branch64() as usize,
                depth - 1,
                CALLEE_LENGTH,
                own,
            );
            let mut parts = vec![Mnemonic::Call as u64];
            parts.extend(callee.0.iter().map(|hash| *hash as u64));
            prints.push(hash(&parts));
        } else {
            prints.push(token(&instruction, CASE_MASK_FROM, own));
        }
        if matches!(
            instruction.mnemonic(),
            Mnemonic::Ret | Mnemonic::Jmp | Mnemonic::Int3
        ) {
            break;
        }
        at = instruction.next_ip() as usize;
    }
    Prints(prints)
}

/// A function that builds a client packet, with the opcode's own write marked.
fn fingerprint_function(code: &Code, start: usize, site: usize) -> Prints {
    let mut prints = Vec::new();
    let mut at = start;
    while prints.len() < FUNCTION_LENGTH {
        let Some(instruction) = code.at(at) else {
            break;
        };
        if instruction.mnemonic() == Mnemonic::Int3 {
            break;
        }
        prints.push(if at == site {
            0x0C0D_E000
        } else {
            token(&instruction, FUNCTION_MASK_FROM, None)
        });
        at = instruction.next_ip() as usize;
    }
    Prints(prints)
}

/// How alike two fingerprints are, from 0 to 1: twice the instructions in common (in order, Ratcliff/Obershelp, as
/// Python's difflib) over both lengths.
pub fn similarity(a: &[u32], b: &[u32]) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let mut positions: HashMap<u32, Vec<usize>> = HashMap::new();
    for (j, hash) in b.iter().enumerate() {
        positions.entry(*hash).or_default().push(j);
    }
    let mut matched = 0;
    let mut todo = vec![(0, a.len(), 0, b.len())];
    while let Some((alo, ahi, blo, bhi)) = todo.pop() {
        // The longest run in common inside the ranges.
        let (mut best_i, mut best_j, mut best) = (alo, blo, 0);
        let mut lengths: HashMap<usize, usize> = HashMap::new();
        for i in alo..ahi {
            let mut next = HashMap::new();
            if let Some(js) = positions.get(&a[i]) {
                for &j in js {
                    if j < blo {
                        continue;
                    }
                    if j >= bhi {
                        break;
                    }
                    let k = lengths.get(&(j.wrapping_sub(1))).copied().unwrap_or(0) + 1;
                    next.insert(j, k);
                    if k > best {
                        best_i = i + 1 - k;
                        best_j = j + 1 - k;
                        best = k;
                    }
                }
            }
            lengths = next;
        }
        if best == 0 {
            continue;
        }
        matched += best;
        todo.push((alo, best_i, blo, best_j));
        todo.push((best_i + best, ahi, best_j + best, bhi));
    }
    2.0 * matched as f32 / (a.len() + b.len()) as f32
}

// --- following an opcode through the code -------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
enum Holds {
    Opcode,
    Number,
    Packet,
}

#[derive(Debug, Clone, Copy)]
struct Value {
    value: u64,
    holds: Holds,
}

#[derive(Debug, Clone, Copy)]
struct Flags {
    sub: Option<bool>, // Some(true) after sub/cmp/dec, Some(false) after add/inc, None after logic
    a: u64,
    b: u64,
    result: u64,
    bits: u32,
    from_opcode: bool,
}

#[derive(Debug, Clone)]
struct State {
    pc: usize,
    registers: HashMap<Register, Value>,
    flags: Option<Flags>,
    /// Calls followed: where to return to, the registers then, and how many decisions had been made.
    stack: Vec<(usize, HashMap<Register, Value>, usize)>,
    /// Each branch the opcode decided: (where, to.)
    decisions: Vec<(usize, usize)>,
}

const ARGUMENT_REGISTERS: [Register; 4] =
    [Register::RCX, Register::RDX, Register::R8, Register::R9];
const KEPT_REGISTERS: [Register; 8] = [
    Register::RBX,
    Register::RSI,
    Register::RDI,
    Register::RBP,
    Register::R12,
    Register::R13,
    Register::R14,
    Register::R15,
];

enum Step {
    Next,
    End,
    /// A branch the opcode doesn't decide: (taken, not taken.)
    Either(usize, usize),
}

/// Follows the code from `start` for `opcode`, with the packet in `packet`. Returns the decisions the opcode made and
/// where the last one led (`start` if none.) `max_depth` is how many calls deep to follow.
fn walk(
    code: &Code,
    start: usize,
    opcode: u16,
    packet: Register,
    max_depth: usize,
    max_forks: usize,
) -> (Vec<usize>, usize) {
    const BUDGET: usize = 6000;
    let mut first = State {
        pc: start,
        registers: HashMap::new(),
        flags: None,
        stack: Vec::new(),
        decisions: Vec::new(),
    };
    first.registers.insert(
        packet,
        Value {
            value: 0,
            holds: Holds::Packet,
        },
    );
    let mut todo = vec![first];
    let mut done: Vec<State> = Vec::new();
    let mut forks = 0;
    let mut steps = 0;
    let mut info = InstructionInfoFactory::new();
    while let Some(mut state) = todo.pop() {
        loop {
            steps += 1;
            if steps > BUDGET {
                done.push(state);
                break;
            }
            let Some(instruction) = code.at(state.pc) else {
                done.push(state);
                break;
            };
            match step(code, &instruction, &mut state, opcode, max_depth, &mut info) {
                Step::Next => {}
                Step::End => {
                    done.push(state);
                    break;
                }
                Step::Either(taken, not_taken) => {
                    // A call that hasn't decided anything yet is skipped rather than explored.
                    if let Some((ret, saved, decided)) = state.stack.last().cloned()
                        && decided == state.decisions.len()
                    {
                        state.stack.pop();
                        state.registers = saved;
                        state.registers.remove(&Register::RAX);
                        state.flags = None;
                        state.pc = ret;
                        continue;
                    }
                    if forks >= max_forks {
                        done.push(state);
                        break;
                    }
                    forks += 1;
                    let mut other = state.clone();
                    other.pc = taken;
                    todo.push(other);
                    state.pc = not_taken;
                }
            }
        }
    }
    // The path that went furthest (the first of them, as explored.)
    let mut best: Option<&State> = None;
    for state in &done {
        if best.is_none_or(|best| state.decisions.len() > best.decisions.len()) {
            best = Some(state);
        }
    }
    let best = best.expect("at least one path");
    let end = best.decisions.last().map(|(_, to)| *to).unwrap_or(start);
    (best.decisions.iter().map(|(at, _)| *at).collect(), end)
}

fn step(
    code: &Code,
    instruction: &Instruction,
    state: &mut State,
    opcode: u16,
    max_depth: usize,
    info: &mut InstructionInfoFactory,
) -> Step {
    let next = instruction.next_ip() as usize;
    state.pc = next;
    let mnemonic = instruction.mnemonic();

    let full = |register: Register| register.full_register();
    let get = |state: &State, i: u32| -> Option<Value> {
        match instruction.op_kind(i) {
            OpKind::Register => state
                .registers
                .get(&full(instruction.op_register(i)))
                .copied(),
            OpKind::Immediate8
            | OpKind::Immediate16
            | OpKind::Immediate32
            | OpKind::Immediate64
            | OpKind::Immediate8to16
            | OpKind::Immediate8to32
            | OpKind::Immediate8to64
            | OpKind::Immediate32to64 => Some(Value {
                value: instruction.immediate(i),
                holds: Holds::Number,
            }),
            _ => None,
        }
    };
    let put = |state: &mut State, register: Register, value: Option<Value>| {
        let size = register.size();
        let register = full(register);
        if register == Register::RSP || register == Register::None {
            return;
        }
        match value {
            None => {
                state.registers.remove(&register);
            }
            Some(mut value) => {
                if size == 4 && value.holds != Holds::Packet {
                    value.value &= 0xFFFF_FFFF;
                }
                if size < 4 && value.holds != Holds::Packet {
                    value.value &= (1u64 << (size * 8)) - 1;
                }
                state.registers.insert(register, value);
            }
        }
    };
    // A memory operand's address, when it's known: a number, or an offset into the packet.
    let address = |state: &State| -> Option<Value> {
        if instruction.segment_prefix() != Register::None {
            return None;
        }
        if instruction.is_ip_rel_memory_operand() {
            return Some(Value {
                value: instruction.ip_rel_memory_address(),
                holds: Holds::Number,
            });
        }
        let base = instruction.memory_base();
        let index = instruction.memory_index();
        let zero = Value {
            value: 0,
            holds: Holds::Number,
        };
        let b = if base == Register::None {
            zero
        } else {
            *state.registers.get(&full(base))?
        };
        let x = if index == Register::None {
            zero
        } else {
            *state.registers.get(&full(index))?
        };
        let displacement = instruction.memory_displacement64();
        if b.holds == Holds::Packet && index == Register::None {
            return Some(Value {
                value: displacement,
                holds: Holds::Packet,
            });
        }
        if b.holds == Holds::Packet || x.holds == Holds::Packet {
            return None;
        }
        let holds = if b.holds == Holds::Opcode || x.holds == Holds::Opcode {
            Holds::Opcode
        } else {
            Holds::Number
        };
        Some(Value {
            value: b
                .value
                .wrapping_add(
                    x.value
                        .wrapping_mul(instruction.memory_index_scale() as u64),
                )
                .wrapping_add(displacement),
            holds,
        })
    };

    let destination_register = (instruction.op_count() > 0
        && instruction.op0_kind() == OpKind::Register)
        .then(|| instruction.op0_register());

    match mnemonic {
        Mnemonic::Mov | Mnemonic::Movzx | Mnemonic::Movsx | Mnemonic::Movsxd
            if destination_register.is_some() =>
        {
            let destination = destination_register.unwrap();
            if instruction.op1_kind() == OpKind::Memory {
                let size = instruction.memory_size().size();
                let value = match address(state) {
                    Some(Value {
                        value: 2,
                        holds: Holds::Packet,
                    }) if size == 2 => Some(Value {
                        value: opcode as u64,
                        holds: Holds::Opcode,
                    }),
                    // A table read with the opcode: what's in the executable there.
                    Some(Value {
                        value,
                        holds: Holds::Opcode,
                    }) => code
                        .read(
                            value,
                            size,
                            matches!(mnemonic, Mnemonic::Movsx | Mnemonic::Movsxd),
                        )
                        .map(|value| Value {
                            value,
                            holds: Holds::Opcode,
                        }),
                    _ => None,
                };
                put(state, destination, value);
            } else {
                let mut value = get(state, 1);
                if let Some(v) = value
                    && v.holds == Holds::Packet
                    && destination.size() != 8
                {
                    value = None;
                }
                if mnemonic == Mnemonic::Movsxd
                    && let Some(v) = value.as_mut()
                    && v.holds != Holds::Packet
                {
                    v.value = v.value as u32 as i32 as i64 as u64;
                }
                if mnemonic == Mnemonic::Movsx
                    && let Some(v) = value.as_mut()
                    && v.holds != Holds::Packet
                {
                    let bits = instruction.op1_register().size() * 8;
                    let shift = 64 - bits;
                    v.value = (((v.value << shift) as i64) >> shift) as u64;
                }
                put(state, destination, value);
            }
            Step::Next
        }
        Mnemonic::Lea if destination_register.is_some() => {
            let value = address(state).filter(|value| value.holds != Holds::Packet);
            put(state, destination_register.unwrap(), value);
            Step::Next
        }
        Mnemonic::Add
        | Mnemonic::Sub
        | Mnemonic::Cmp
        | Mnemonic::And
        | Mnemonic::Test
        | Mnemonic::Inc
        | Mnemonic::Dec
        | Mnemonic::Xor
        | Mnemonic::Or
            if destination_register.is_some() =>
        {
            let destination = destination_register.unwrap();
            let compares = matches!(mnemonic, Mnemonic::Cmp | Mnemonic::Test);
            if mnemonic == Mnemonic::Xor
                && instruction.op1_kind() == OpKind::Register
                && full(instruction.op1_register()) == full(destination)
            {
                put(
                    state,
                    destination,
                    Some(Value {
                        value: 0,
                        holds: Holds::Number,
                    }),
                );
                state.flags = None;
                return Step::Next;
            }
            let a = get(state, 0);
            let b = if matches!(mnemonic, Mnemonic::Inc | Mnemonic::Dec) {
                Some(Value {
                    value: 1,
                    holds: Holds::Number,
                })
            } else {
                get(state, 1)
            };
            let (Some(a), Some(b)) = (a, b) else {
                if !compares {
                    put(state, destination, None);
                }
                state.flags = None;
                return Step::Next;
            };
            if a.holds == Holds::Packet || b.holds == Holds::Packet {
                if !compares {
                    put(state, destination, None);
                }
                state.flags = None;
                return Step::Next;
            }
            let bits = destination.size() as u32 * 8;
            let mask = if bits == 64 {
                u64::MAX
            } else {
                (1u64 << bits) - 1
            };
            let (av, bv) = (a.value & mask, b.value & mask);
            let from_opcode = a.holds == Holds::Opcode || b.holds == Holds::Opcode;
            let (result, sub) = match mnemonic {
                Mnemonic::Add | Mnemonic::Inc => (av.wrapping_add(bv) & mask, Some(false)),
                Mnemonic::Sub | Mnemonic::Cmp | Mnemonic::Dec => {
                    (av.wrapping_sub(bv) & mask, Some(true))
                }
                Mnemonic::And | Mnemonic::Test => (av & bv, None),
                Mnemonic::Or => (av | bv, None),
                _ => (av ^ bv, None),
            };
            state.flags = Some(Flags {
                sub,
                a: av,
                b: bv,
                result,
                bits,
                from_opcode,
            });
            if !compares {
                put(
                    state,
                    destination,
                    Some(Value {
                        value: result,
                        holds: if from_opcode {
                            Holds::Opcode
                        } else {
                            Holds::Number
                        },
                    }),
                );
            }
            Step::Next
        }
        Mnemonic::Jmp => match instruction.op0_kind() {
            OpKind::NearBranch64 => {
                state.pc = instruction.near_branch64() as usize;
                Step::Next
            }
            OpKind::Register => match get(state, 0) {
                Some(Value {
                    value,
                    holds: Holds::Opcode,
                }) if code.in_text(value) => {
                    state
                        .decisions
                        .push((instruction.ip() as usize, value as usize));
                    state.pc = value as usize;
                    Step::Next
                }
                _ => Step::End,
            },
            _ => Step::End,
        },
        _ if instruction.flow_control() == FlowControl::ConditionalBranch => {
            if instruction.op0_kind() != OpKind::NearBranch64 {
                return Step::End;
            }
            let taken = instruction.near_branch64() as usize;
            let Some(flags) = state.flags.filter(|flags| flags.from_opcode) else {
                return Step::Either(taken, next);
            };
            let zf = flags.result == 0;
            let sf = flags.result >> (flags.bits - 1) & 1 == 1;
            let (cf, of) = match flags.sub {
                Some(true) => (
                    flags.a < flags.b,
                    ((flags.a ^ flags.b) & (flags.a ^ flags.result)) >> (flags.bits - 1) & 1 == 1,
                ),
                Some(false) => {
                    let mask = if flags.bits == 64 {
                        u64::MAX
                    } else {
                        (1u64 << flags.bits) - 1
                    };
                    (
                        (flags.a as u128 + flags.b as u128) > mask as u128,
                        (!(flags.a ^ flags.b) & (flags.a ^ flags.result)) >> (flags.bits - 1) & 1
                            == 1,
                    )
                }
                None => (false, false),
            };
            let condition = match mnemonic {
                Mnemonic::Je => zf,
                Mnemonic::Jne => !zf,
                Mnemonic::Ja => !cf && !zf,
                Mnemonic::Jae => !cf,
                Mnemonic::Jb => cf,
                Mnemonic::Jbe => cf || zf,
                Mnemonic::Jg => !zf && sf == of,
                Mnemonic::Jge => sf == of,
                Mnemonic::Jl => sf != of,
                Mnemonic::Jle => zf || sf != of,
                Mnemonic::Js => sf,
                Mnemonic::Jns => !sf,
                _ => return Step::Either(taken, next),
            };
            let to = if condition { taken } else { next };
            state.decisions.push((instruction.ip() as usize, to));
            state.pc = to;
            Step::Next
        }
        Mnemonic::Call => {
            let passes = ARGUMENT_REGISTERS.iter().any(|register| {
                state
                    .registers
                    .get(register)
                    .is_some_and(|value| value.holds != Holds::Number)
            });
            if instruction.op0_kind() == OpKind::NearBranch64
                && passes
                && state.stack.len() < max_depth
                && code.in_text(instruction.near_branch64())
            {
                state
                    .stack
                    .push((next, state.registers.clone(), state.decisions.len()));
                state.pc = instruction.near_branch64() as usize;
            } else {
                state.registers.remove(&Register::RAX);
                state.flags = None;
            }
            Step::Next
        }
        Mnemonic::Ret => {
            let Some((ret, saved, _)) = state.stack.pop() else {
                return Step::End;
            };
            for register in KEPT_REGISTERS {
                match saved.get(&register) {
                    Some(value) => {
                        state.registers.insert(register, *value);
                    }
                    None => {
                        state.registers.remove(&register);
                    }
                }
            }
            state.registers.remove(&Register::RAX);
            state.flags = None;
            state.pc = ret;
            Step::Next
        }
        // Sign-extending the accumulator (cdqe is how a switch index often gets to 64 bits.)
        Mnemonic::Cdqe | Mnemonic::Cwde => {
            let value = state.registers.get(&Register::RAX).copied();
            let value = value.filter(|v| v.holds != Holds::Packet).map(|mut v| {
                v.value = if mnemonic == Mnemonic::Cdqe {
                    v.value as u32 as i32 as i64 as u64
                } else {
                    v.value as u16 as i16 as i32 as u32 as u64
                };
                v
            });
            match value {
                Some(value) => {
                    state.registers.insert(Register::RAX, value);
                }
                None => {
                    state.registers.remove(&Register::RAX);
                }
            }
            Step::Next
        }
        Mnemonic::Int3 | Mnemonic::Ud2 | Mnemonic::Hlt => Step::End,
        _ => {
            // Anything else: forget the registers and flags it writes.
            let written: Vec<Register> = info
                .info(instruction)
                .used_registers()
                .iter()
                .filter(|used| {
                    matches!(
                        used.access(),
                        OpAccess::Write
                            | OpAccess::ReadWrite
                            | OpAccess::CondWrite
                            | OpAccess::ReadCondWrite
                    )
                })
                .map(|used| used.register())
                .collect();
            for register in written {
                put(state, register, None);
            }
            if instruction.rflags_modified() != 0 {
                state.flags = None;
            }
            Step::Next
        }
    }
}

/// The register a function reads the packet's opcode through (`movzx r, word [reg + 2]`), near its start.
fn packet_register(code: &Code, start: usize) -> Option<Register> {
    let mut at = start;
    for _ in 0..300 {
        let instruction = code.at(at)?;
        if let Some(register) = opcode_read(&instruction) {
            return Some(register);
        }
        if instruction.mnemonic() == Mnemonic::Int3 {
            return None;
        }
        at = instruction.next_ip() as usize;
    }
    None
}

/// `movzx r32, word [reg + 2]`: the base register.
fn opcode_read(instruction: &Instruction) -> Option<Register> {
    (instruction.mnemonic() == Mnemonic::Movzx
        && instruction.op_count() == 2
        && instruction.op1_kind() == OpKind::Memory
        && instruction.memory_size().size() == 2
        && instruction.memory_displacement64() == 2
        && instruction.memory_index() == Register::None
        && instruction.memory_base() != Register::None
        && instruction.memory_base() != Register::RIP)
        .then(|| instruction.memory_base().full_register())
}

// --- server packets -----------------------------------------------------------------------------------------------

/// What the dispatcher does with each opcode: its case (where it goes in the dispatcher itself), and where its path
/// ends (following calls.)
struct ServerPaths {
    case: BTreeMap<u16, usize>,
    end: BTreeMap<u16, usize>,
}

fn server_paths(code: &Code, dispatcher: usize) -> Option<ServerPaths> {
    let packet = packet_register(code, dispatcher)?;
    let mut case = BTreeMap::new();
    let mut end = BTreeMap::new();
    for opcode in SERVER_OPCODES {
        case.insert(opcode, walk(code, dispatcher, opcode, packet, 0, 24).1);
        end.insert(opcode, walk(code, dispatcher, opcode, packet, 5, 24).1);
    }
    Some(ServerPaths { case, end })
}

/// Each opcode's case and end in the dispatcher at `dispatcher`, for checking the walk (kawari-opcodescan paths.)
pub fn debug_paths(exe: &Exe, dispatcher: usize) -> Vec<(u16, usize, usize)> {
    let code = Code::new(exe);
    match server_paths(&code, dispatcher) {
        Some(paths) => paths
            .case
            .iter()
            .map(|(opcode, case)| (*opcode, *case, paths.end[opcode]))
            .collect(),
        None => Vec::new(),
    }
}

fn groups(paths: &BTreeMap<u16, usize>) -> BTreeMap<usize, Vec<u16>> {
    let mut groups: BTreeMap<usize, Vec<u16>> = BTreeMap::new();
    for (opcode, to) in paths {
        groups.entry(*to).or_default().push(*opcode);
    }
    groups
}

/// The first function a case calls, or jumps to outside the dispatcher (`cases`: where its cases are), or 0 when it
/// has neither (it can't be placed by order then.)
fn handler(code: &Code, case: usize, cases: &std::ops::Range<usize>) -> usize {
    let mut at = case;
    for _ in 0..CASE_LENGTH {
        let Some(instruction) = code.at(at) else {
            break;
        };
        if instruction.op_count() > 0 && instruction.op0_kind() == OpKind::NearBranch64 {
            let to = instruction.near_branch64() as usize;
            match instruction.mnemonic() {
                Mnemonic::Call => return to,
                Mnemonic::Jmp if !cases.contains(&to) => return to,
                Mnemonic::Jmp => break,
                _ => {}
            }
        }
        if matches!(instruction.mnemonic(), Mnemonic::Ret | Mnemonic::Int3) {
            break;
        }
        at = instruction.next_ip() as usize;
    }
    0
}

/// Where a dispatcher's cases are, and a little after (where they jump back to.)
fn case_range(groups: &BTreeMap<usize, Vec<u16>>, dispatcher: usize) -> std::ops::Range<usize> {
    let first = groups
        .keys()
        .next()
        .copied()
        .unwrap_or(dispatcher)
        .min(dispatcher);
    let last = groups.keys().next_back().copied().unwrap_or(dispatcher);
    first..last + 0x3000
}

/// The opcode dispatcher among the executable's switches: the function with the biggest switch over opcodes.
fn dispatchers(exe: &Exe) -> Vec<usize> {
    let (start, end) = exe.text();
    let mut found: Vec<(usize, usize)> = exe
        .switches(start, end)
        .into_iter()
        .filter(|switch| switch.bias >= 0 && switch.bias < OPCODE_LIMIT as i64)
        .map(|switch| {
            (
                switch.cases.values().map(Vec::len).sum::<usize>(),
                exe.function_start(switch.at),
            )
        })
        .filter(|(cases, _)| *cases >= 200)
        .collect();
    found.sort();
    found.dedup_by_key(|(_, function)| *function);
    found
        .into_iter()
        .rev()
        .map(|(_, function)| function)
        .collect()
}

/// Functions that read a packet's opcode and compare it with opcode-sized numbers: second dispatchers, where the
/// zone dispatcher passes a packet on to a part of the game that looks at the opcode again.
fn second_dispatchers(code: &Code) -> Vec<usize> {
    let exe = code.exe;
    let (start, end) = exe.text();
    let image = exe.image();
    let mut found = HashSet::new();
    let mut at = start;
    while at + 4 < end {
        // 0F B7 /r with mod 01 (disp8) and displacement 2, maybe after a REX prefix
        if image[at] == 0x0F
            && image[at + 1] == 0xB7
            && (0x40..0x80).contains(&image[at + 2])
            && image[at + 3] == 2
        {
            let from = if (0x40..0x50).contains(&image[at - 1]) {
                at - 1
            } else {
                at
            };
            if let Some(instruction) = code.at(from)
                && opcode_read(&instruction).is_some()
            {
                let mut p = instruction.next_ip() as usize;
                for _ in 0..12 {
                    let Some(next) = code.at(p) else { break };
                    let compares = matches!(
                        next.mnemonic(),
                        Mnemonic::Cmp | Mnemonic::Sub | Mnemonic::Add
                    ) && (0..next.op_count()).any(|i| {
                        is_immediate(next.op_kind(i))
                            && (100..OPCODE_LIMIT as i64)
                                .contains(&(next.immediate(i) as i64).abs())
                    });
                    let lea = next.mnemonic() == Mnemonic::Lea
                        && (100..OPCODE_LIMIT as i64)
                            .contains(&(next.memory_displacement64() as i64).abs());
                    if compares || lea {
                        found.insert(exe.function_start(from));
                        break;
                    }
                    p = next.next_ip() as usize;
                }
            }
        }
        at += 1;
    }
    let mut found: Vec<usize> = found.into_iter().collect();
    found.sort();
    found
}

/// The second dispatcher that tells most of `members` apart: (dispatcher, where each one ends.)
fn best_split(
    code: &Code,
    dispatchers: &[usize],
    members: &[u16],
) -> Option<BTreeMap<u16, Option<usize>>> {
    let mut best: Option<(usize, BTreeMap<u16, Option<usize>>)> = None;
    for &dispatcher in dispatchers {
        for packet in ARGUMENT_REGISTERS {
            let mut ends = BTreeMap::new();
            for &opcode in members {
                let (decisions, end) = walk(code, dispatcher, opcode, packet, 5, 8);
                ends.insert(opcode, (!decisions.is_empty()).then_some(end));
            }
            let distinct: HashSet<usize> = ends.values().flatten().copied().collect();
            if distinct.len() >= 2
                && best
                    .as_ref()
                    .is_none_or(|(count, _)| distinct.len() > *count)
            {
                best = Some((distinct.len(), ends));
            }
        }
    }
    best.map(|(_, ends)| ends)
}

// --- client packets -----------------------------------------------------------------------------------------------

/// Every place a client packet is built: (where the opcode is written, the opcode, the size written after it.)
fn client_sites(code: &Code) -> Vec<(usize, u16, u32)> {
    let exe = code.exe;
    let (start, end) = exe.text();
    let image = exe.image();
    let mut sites = Vec::new();
    let mut at = start;
    while at + 8 < end {
        // C7 /0 with a memory operand (mod 01 or 10), maybe after a REX prefix
        if image[at] == 0xC7 && matches!(image[at + 1] >> 6, 1 | 2) && image[at + 1] >> 3 & 7 == 0 {
            for from in [at, at.saturating_sub(1)] {
                let Some(instruction) = code.at(from) else {
                    continue;
                };
                if instruction.mnemonic() != Mnemonic::Mov
                    || instruction.op0_kind() != OpKind::Memory
                    || instruction.memory_size().size() != 4
                    || !matches!(instruction.op1_kind(), OpKind::Immediate32)
                {
                    continue;
                }
                let opcode = instruction.immediate32() as i64;
                if !CLIENT_OPCODES.contains(&opcode) {
                    break;
                }
                let base = instruction.memory_base();
                let displacement = instruction.memory_displacement64();
                let mut p = instruction.next_ip() as usize;
                for _ in 0..10 {
                    let Some(next) = code.at(p) else { break };
                    if next.mnemonic() == Mnemonic::Mov
                        && next.op_count() == 2
                        && next.op0_kind() == OpKind::Memory
                        && next.memory_base() == base
                        && next.memory_displacement64() == displacement.wrapping_add(8)
                        && is_immediate(next.op1_kind())
                    {
                        sites.push((from, opcode as u16, next.immediate(1) as u32));
                        break;
                    }
                    p = next.next_ip() as usize;
                }
                break;
            }
        }
        at += 1;
    }
    sites.sort();
    sites.dedup();
    sites
}

/// The sizes each client opcode is sent with in this executable (data only, without headers), from every function
/// that builds a packet (writes an opcode, then its size with the 16-byte IPC header): opcode → sizes.
pub fn client_sizes(exe: &Exe) -> BTreeMap<u16, std::collections::BTreeSet<u32>> {
    let code = Code::new(exe);
    let mut sizes: BTreeMap<u16, std::collections::BTreeSet<u32>> = BTreeMap::new();
    for (_, opcode, with_header) in client_sites(&code) {
        if let Some(size) = with_header.checked_sub(16) {
            sizes.entry(opcode).or_default().insert(size);
        }
    }
    sizes
}

/// Client packets whose size changed: (name, size the server has, sizes this executable builds it with), for each
/// one of `opcodes` (name → its opcode in this executable) that is built here only with other sizes. A new size
/// means the packet's layout changed with the update, which no opcode table fixes.
pub fn client_size_changes(
    exe: &Exe,
    opcodes: &BTreeMap<String, u16>,
    known_sizes: &BTreeMap<String, u32>,
) -> Vec<(String, u32, Vec<u32>)> {
    let built = client_sizes(exe);
    let mut changes = Vec::new();
    for (name, opcode) in opcodes {
        let (Some(known), Some(sizes)) = (known_sizes.get(name), built.get(opcode)) else {
            continue;
        };
        if !sizes.contains(known) {
            changes.push((name.clone(), *known, sizes.iter().copied().collect()));
        }
    }
    changes
}

// --- recording ----------------------------------------------------------------------------------------------------

/// Records the fingerprints of `known` (server lists end in Server..., client ones in Client...) in `exe`, the game
/// version they're right for.
pub fn fingerprint(
    exe: &Exe,
    game_version: &str,
    known: &[KnownOpcode],
) -> Result<Fingerprints, String> {
    let code = Code::new(exe);
    let server_known: Vec<&KnownOpcode> = known
        .iter()
        .filter(|k| k.kind == "ServerZoneIpcType")
        .collect();
    let client_known: Vec<&KnownOpcode> = known
        .iter()
        .filter(|k| k.kind == "ClientZoneIpcType")
        .collect();

    // The zone dispatcher is the big switch whose cases the known opcodes use most distinctly.
    let mut chosen = None;
    for dispatcher in dispatchers(exe) {
        let Some(paths) = server_paths(&code, dispatcher) else {
            continue;
        };
        let case_groups = groups(&paths.case);
        let distinct: HashSet<usize> = server_known
            .iter()
            .filter_map(|k| paths.case.get(&k.opcode))
            .copied()
            .collect();
        let biggest = case_groups.values().map(Vec::len).max().unwrap_or(0);
        if distinct.len() * 2 > server_known.len() && biggest < SERVER_OPCODES.len() / 2 {
            chosen = Some((dispatcher, paths));
            break;
        }
    }
    let (dispatcher, paths) = chosen.ok_or("no zone dispatcher found")?;
    let case_groups = groups(&paths.case);
    let range = case_range(&case_groups, dispatcher);
    let second = second_dispatchers(&code);

    let mut server = Vec::new();
    for k in &server_known {
        let Some(&case) = paths.case.get(&k.opcode) else {
            continue;
        };
        let members = &case_groups[&case];
        let end = paths.end[&k.opcode];
        let end_group = members
            .iter()
            .filter(|member| paths.end[member] == end)
            .count();
        server.push(ServerPrint {
            kind: k.kind.clone(),
            name: k.name.clone(),
            opcode: k.opcode,
            case: fingerprint_code(
                &code,
                case,
                1,
                CASE_LENGTH,
                (members.len() == 1).then_some(k.opcode),
            ),
            case_group: members.len(),
            handler_at: handler(&code, case, &range),
            end: fingerprint_code(
                &code,
                end,
                1,
                END_LENGTH,
                (end_group == 1).then_some(k.opcode),
            ),
            end_group,
            sub: None,
        });
    }
    // Opcodes still sharing their end with another: where a second dispatcher sends them.
    let mut shared: BTreeMap<(usize, usize), Vec<u16>> = BTreeMap::new();
    for k in &server_known {
        if let Some(&case) = paths.case.get(&k.opcode) {
            let end = paths.end[&k.opcode];
            let members: Vec<u16> = case_groups[&case]
                .iter()
                .copied()
                .filter(|m| paths.end[m] == end)
                .collect();
            if members.len() > 1 && members.len() <= 64 {
                shared.insert((case, end), members);
            }
        }
    }
    for members in shared.values() {
        let Some(ends) = best_split(&code, &second, members) else {
            continue;
        };
        for print in server
            .iter_mut()
            .filter(|print| members.contains(&print.opcode))
        {
            if let Some(Some(end)) = ends.get(&print.opcode) {
                print.sub = Some(SubPrint {
                    end: fingerprint_code(&code, *end, 1, END_LENGTH, None),
                    group: ends.values().filter(|other| **other == Some(*end)).count(),
                });
            }
        }
    }

    let sites = client_sites(&code);
    let mut client = Vec::new();
    for k in &client_known {
        let with_header = k.size + 16;
        let prints: Vec<SitePrint> = sites
            .iter()
            .filter(|(_, opcode, size)| *opcode == k.opcode && *size == with_header)
            .map(|(site, _, _)| {
                let function = exe.function_start(*site);
                SitePrint {
                    function: fingerprint_function(&code, function, *site),
                    function_at: function,
                }
            })
            .collect();
        client.push(ClientPrint {
            kind: k.kind.clone(),
            name: k.name.clone(),
            opcode: k.opcode,
            size: k.size,
            sites: prints,
        });
    }

    Ok(Fingerprints {
        game_version: game_version.to_string(),
        format: OWN_OPCODE_FORMAT,
        dispatcher: fingerprint_code(&code, dispatcher, 0, CASE_LENGTH, None),
        dispatcher_at: dispatcher,
        server,
        client,
    })
}

// --- where packets are handled (kawari-opcodescan layouts) --------------------------------------------------------

/// A known server packet's case in the client's zone dispatcher.
#[derive(Debug, Clone)]
pub struct ServerSite {
    pub name: String,
    pub opcode: u16,
    pub size: u32,
    /// Where the dispatcher sends its opcode, the packet (its IPC header) still in `PacketSites::packet`.
    pub case: usize,
    /// How many opcodes share that case (more than one: what's read there is read for all of them.)
    pub case_group: usize,
    /// Where its path ends further in (following calls), where it doesn't share it with others.
    pub end: usize,
}

/// A known client packet's opcode write in the function that builds it.
#[derive(Debug, Clone)]
pub struct ClientSite {
    pub name: String,
    pub opcode: u16,
    pub size: u32,
    /// The `mov [base+x], opcode` (the client's packet header: opcode, then the size with the header at +8; the
    /// packet's data from +0x20.)
    pub site: usize,
    pub function: usize,
}

/// Where the known zone packets are handled (server) and built (client) in an executable.
#[derive(Debug, Clone)]
pub struct PacketSites {
    pub dispatcher: usize,
    /// The register the dispatcher holds the packet (its IPC header) in.
    pub packet: Register,
    pub server: Vec<ServerSite>,
    pub client: Vec<ClientSite>,
}

/// Finds where each of `known` is handled or built in `exe` (the dispatcher is chosen as `fingerprint` does.)
pub fn packet_sites(exe: &Exe, known: &[KnownOpcode]) -> Result<PacketSites, String> {
    let code = Code::new(exe);
    let server_known: Vec<&KnownOpcode> = known
        .iter()
        .filter(|k| k.kind == "ServerZoneIpcType")
        .collect();
    let mut chosen = None;
    for dispatcher in dispatchers(exe) {
        let Some(paths) = server_paths(&code, dispatcher) else {
            continue;
        };
        let distinct: HashSet<usize> = server_known
            .iter()
            .filter_map(|k| paths.case.get(&k.opcode))
            .copied()
            .collect();
        let biggest = groups(&paths.case)
            .values()
            .map(Vec::len)
            .max()
            .unwrap_or(0);
        if distinct.len() * 2 > server_known.len() && biggest < SERVER_OPCODES.len() / 2 {
            chosen = Some((dispatcher, paths));
            break;
        }
    }
    let (dispatcher, paths) = chosen.ok_or("no zone dispatcher found")?;
    let packet = packet_register(&code, dispatcher).ok_or("the dispatcher reads no opcode")?;
    let case_groups = groups(&paths.case);
    let server = server_known
        .iter()
        .filter_map(|k| {
            let case = *paths.case.get(&k.opcode)?;
            Some(ServerSite {
                name: k.name.clone(),
                opcode: k.opcode,
                size: k.size,
                case,
                case_group: case_groups[&case].len(),
                end: paths.end[&k.opcode],
            })
        })
        .collect();
    let sites = client_sites(&code);
    let client = known
        .iter()
        .filter(|k| k.kind == "ClientZoneIpcType")
        .flat_map(|k| {
            sites
                .iter()
                .filter(move |(_, opcode, size)| *opcode == k.opcode && *size == k.size + 16)
                .map(move |(site, _, _)| ClientSite {
                    name: k.name.clone(),
                    opcode: k.opcode,
                    size: k.size,
                    site: *site,
                    function: exe.function_start(*site),
                })
        })
        .collect();
    Ok(PacketSites {
        dispatcher,
        packet,
        server,
        client,
    })
}

// --- matching -----------------------------------------------------------------------------------------------------

/// The closest of `candidates` to `print`, if it's close enough and clearly the closest.
fn clearly_closest<T: Copy>(candidates: impl Iterator<Item = (T, f32)>) -> Option<T> {
    let mut scored: Vec<(f32, T)> = candidates.map(|(t, score)| (score, t)).collect();
    scored.sort_by(|a, b| b.0.total_cmp(&a.0));
    match scored.as_slice() {
        [(best, t)] if *best >= CLOSE_SIMILARITY => Some(*t),
        [(best, t), (second, _), ..]
            if *best >= CLOSE_SIMILARITY && best - second >= CLEAR_MARGIN =>
        {
            Some(*t)
        }
        _ => None,
    }
}

/// Finds the fingerprinted opcodes in `exe`: (list, name) → what was found.
pub fn match_fingerprints(exe: &Exe, prints: &Fingerprints) -> BTreeMap<(String, String), Found> {
    let code = Code::new(exe);
    let mut found = BTreeMap::new();
    match_server(&code, prints, &mut found);
    match_client(&code, prints, &mut found);
    found
}

fn match_server(code: &Code, prints: &Fingerprints, found: &mut BTreeMap<(String, String), Found>) {
    let not_found = |found: &mut BTreeMap<(String, String), Found>| {
        for print in &prints.server {
            found.insert((print.kind.clone(), print.name.clone()), Found::NotFound);
        }
    };
    // The dispatcher: the big switch that starts most like the recorded one.
    let candidates = dispatchers(code.exe);
    let Some(dispatcher) = candidates
        .iter()
        .map(|&d| {
            (
                d,
                similarity(
                    &prints.dispatcher.0,
                    &fingerprint_code(code, d, 0, CASE_LENGTH, None).0,
                ),
            )
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(d, _)| d)
    else {
        not_found(found);
        return;
    };
    let Some(paths) = server_paths(code, dispatcher) else {
        not_found(found);
        return;
    };
    let case_groups = groups(&paths.case);
    // A lone opcode's number in its own code counts as "its opcode", as recorded (`OWN_OPCODE_FORMAT`.)
    let own_opcode = prints.format >= OWN_OPCODE_FORMAT;
    let lone = |members: &[u16]| match members {
        [only] if own_opcode => Some(*only),
        _ => None,
    };
    let case_prints: BTreeMap<usize, Prints> = case_groups
        .iter()
        .map(|(&case, members)| {
            (
                case,
                fingerprint_code(code, case, 1, CASE_LENGTH, lone(members)),
            )
        })
        .collect();
    let range = case_range(&case_groups, dispatcher);
    let handlers: BTreeMap<usize, usize> = case_groups
        .keys()
        .map(|&case| (case, handler(code, case, &range)))
        .collect();

    // 1. Cases by their code (one group of old opcodes → one new case.)
    let mut old_cases: BTreeMap<(Prints, usize, usize), Vec<&ServerPrint>> = BTreeMap::new();
    for print in &prints.server {
        old_cases
            .entry((print.case.clone(), print.case_group, print.handler_at))
            .or_default()
            .push(print);
    }
    let mut case_of: HashMap<(Prints, usize, usize), usize> = HashMap::new();
    let mut by_print: HashMap<&Prints, Vec<usize>> = HashMap::new();
    for (case, print) in &case_prints {
        by_print.entry(print).or_default().push(*case);
    }
    // An exact match is unique both ways: old code that's the same in two places is placed by order instead.
    let mut old_count: HashMap<(&Prints, usize), usize> = HashMap::new();
    for key in old_cases.keys() {
        *old_count.entry((&key.0, key.1)).or_default() += 1;
    }
    for key in old_cases.keys() {
        if old_count[&(&key.0, key.1)] > 1 {
            continue;
        }
        let matching: Vec<usize> = by_print
            .get(&key.0)
            .map(|cases| {
                cases
                    .iter()
                    .copied()
                    .filter(|case| case_groups[case].len() == key.1)
                    .collect()
            })
            .unwrap_or_default();
        if let [case] = matching.as_slice() {
            case_of.insert(key.clone(), *case);
        }
    }
    // 2. The rest by order: matched handlers predict where the others moved.
    for _ in 0..3 {
        let mut anchors: Vec<(usize, usize)> = case_of
            .iter()
            .filter(|(key, case)| key.2 != 0 && handlers[*case] != 0)
            .map(|(key, case)| (key.2, handlers[case]))
            .collect();
        anchors.sort();
        let used: HashSet<usize> = case_of.values().copied().collect();
        let mut added = Vec::new();
        for key in old_cases
            .keys()
            .filter(|key| key.2 != 0 && !case_of.contains_key(*key))
        {
            let i = anchors.partition_point(|(old, _)| *old < key.2);
            let mut best: Option<((bool, f32, i64), usize)> = None;
            for j in [i.wrapping_sub(1), i] {
                let Some(&(old, new)) = anchors.get(j) else {
                    continue;
                };
                let predicted = (key.2 as i64 - old as i64 + new as i64) as usize;
                for (case, members) in &case_groups {
                    if used.contains(case) || members.len() != key.1 || handlers[case] == 0 {
                        continue;
                    }
                    let distance = handlers[case].abs_diff(predicted);
                    if distance > SERVER_ORDER_WINDOW {
                        continue;
                    }
                    let score = similarity(&key.0.0, &case_prints[case].0);
                    if score < ORDER_SIMILARITY {
                        continue;
                    }
                    // Exactly where the order says first, then the most alike, then the nearest.
                    let rank = (distance == 0, score, -(distance as i64));
                    if best.as_ref().is_none_or(|(b, _)| {
                        rank.0 > b.0 || (rank.0 == b.0 && (rank.1, rank.2) > (b.1, b.2))
                    }) {
                        best = Some((rank, *case));
                    }
                }
            }
            if let Some((_, case)) = best {
                added.push((key.clone(), case));
            }
        }
        if added.is_empty() {
            break;
        }
        let mut taken: HashSet<usize> = used;
        for (key, case) in added {
            if taken.insert(case) {
                case_of.insert(key, case);
            }
        }
    }

    // 3. Within a case shared by several opcodes: their ends, then second dispatchers.
    let second = std::cell::OnceCell::new();
    for (key, olds) in &old_cases {
        let Some(&case) = case_of.get(key) else {
            for print in olds {
                found.insert((print.kind.clone(), print.name.clone()), Found::NotFound);
            }
            continue;
        };
        let members = &case_groups[&case];
        if members.len() == 1 {
            for print in olds {
                found.insert(
                    (print.kind.clone(), print.name.clone()),
                    Found::Opcode(members[0]),
                );
            }
            continue;
        }
        let mut new_ends: BTreeMap<usize, Vec<u16>> = BTreeMap::new();
        for member in members {
            new_ends.entry(paths.end[member]).or_default().push(*member);
        }
        let end_prints: BTreeMap<usize, Prints> = new_ends
            .iter()
            .map(|(&end, members)| {
                (
                    end,
                    fingerprint_code(code, end, 1, END_LENGTH, lone(members)),
                )
            })
            .collect();
        let mut used = HashSet::new();
        let mut ends_of: HashMap<&Prints, usize> = HashMap::new();
        // exact first, then clearly closest
        let mut olds_by_end: BTreeMap<(&Prints, usize), Vec<&ServerPrint>> = BTreeMap::new();
        for print in olds {
            olds_by_end
                .entry((&print.end, print.end_group))
                .or_default()
                .push(print);
        }
        for (end_print, group) in olds_by_end.keys() {
            let exact: Vec<usize> = new_ends
                .iter()
                .filter(|(end, m)| {
                    m.len() == *group && !used.contains(*end) && end_prints[*end] == **end_print
                })
                .map(|(end, _)| *end)
                .collect();
            if let [end] = exact.as_slice() {
                used.insert(*end);
                ends_of.insert(end_print, *end);
            }
        }
        // Only exact ends count here: the ends of one shared handler are often a few bytes apart in one compare tree,
        // running on into each other, so how alike they are says little (10.16 to 10.06's inventory packets.) Nor does
        // their order: a compare tree is laid out by the opcodes' values, which change (tried: it swapped the two
        // InventoryTransactions between 10.06 and 10.16.)
        let leftover: Vec<u16> = new_ends
            .iter()
            .filter(|(end, _)| !used.contains(*end))
            .flat_map(|(_, m)| m.iter().copied())
            .collect();
        for (_, olds_here) in olds_by_end {
            let end = ends_of.get(&olds_here[0].end).copied();
            let candidates: Vec<u16> = match end {
                _ if new_ends.len() == 1 => members.clone(),
                Some(end) => new_ends[&end].clone(),
                None => leftover.clone(),
            };
            if candidates.len() > 1 && olds_here.iter().any(|print| print.sub.is_some()) {
                // a second dispatcher may tell them apart
                let second = second.get_or_init(|| second_dispatchers(code));
                if let Some(ends) = best_split(code, second, &candidates) {
                    let mut sub_groups: BTreeMap<usize, Vec<u16>> = BTreeMap::new();
                    for (opcode, end) in &ends {
                        if let Some(end) = end {
                            sub_groups.entry(*end).or_default().push(*opcode);
                        }
                    }
                    let sub_prints: BTreeMap<usize, Prints> = sub_groups
                        .keys()
                        .map(|&end| (end, fingerprint_code(code, end, 1, END_LENGTH, None)))
                        .collect();
                    let mut sub_used = HashSet::new();
                    for print in &olds_here {
                        let Some(sub) = &print.sub else {
                            found.insert(
                                (print.kind.clone(), print.name.clone()),
                                Found::OneOf(candidates.clone()),
                            );
                            continue;
                        };
                        let exact: Vec<usize> = sub_groups
                            .iter()
                            .filter(|(end, m)| m.len() == sub.group && sub_prints[*end] == sub.end)
                            .map(|(end, _)| *end)
                            .collect();
                        let pick = if let [end] = exact.as_slice() {
                            Some(*end)
                        } else {
                            clearly_closest(
                                sub_groups
                                    .iter()
                                    .filter(|(end, m)| {
                                        m.len() == sub.group && !sub_used.contains(*end)
                                    })
                                    .map(|(end, _)| {
                                        (*end, similarity(&sub.end.0, &sub_prints[end].0))
                                    }),
                            )
                        };
                        match pick {
                            Some(end) => {
                                sub_used.insert(end);
                                found.insert(
                                    (print.kind.clone(), print.name.clone()),
                                    one(&sub_groups[&end]),
                                );
                            }
                            None => {
                                found.insert(
                                    (print.kind.clone(), print.name.clone()),
                                    Found::OneOf(candidates.clone()),
                                );
                            }
                        }
                    }
                    continue;
                }
            }
            for print in olds_here {
                found.insert((print.kind.clone(), print.name.clone()), one(&candidates));
            }
        }
    }
}

fn one(candidates: &[u16]) -> Found {
    match candidates {
        [] => Found::NotFound,
        [opcode] => Found::Opcode(*opcode),
        many => Found::OneOf(many.to_vec()),
    }
}

fn match_client(code: &Code, prints: &Fingerprints, found: &mut BTreeMap<(String, String), Found>) {
    let sites = client_sites(code);
    let site_prints: Vec<(usize, u16, u32, usize, Prints)> = sites
        .iter()
        .map(|&(site, opcode, size)| {
            let function = code.exe.function_start(site);
            (
                site,
                opcode,
                size,
                function,
                fingerprint_function(code, function, site),
            )
        })
        .collect();

    // Old functions that look the same build different packets: those don't count as exact.
    let mut old_owners: HashMap<(&Prints, u32), HashSet<u16>> = HashMap::new();
    for print in &prints.client {
        for site in &print.sites {
            old_owners
                .entry((&site.function, print.size))
                .or_default()
                .insert(print.opcode);
        }
    }
    let mut result: BTreeMap<&str, Option<u16>> = BTreeMap::new();
    for print in &prints.client {
        let with_header = print.size + 16;
        let mut votes: BTreeMap<u16, usize> = BTreeMap::new();
        for site in &print.sites {
            let candidates: Vec<&(usize, u16, u32, usize, Prints)> =
                site_prints.iter().filter(|s| s.2 == with_header).collect();
            let exact: HashSet<u16> = candidates
                .iter()
                .filter(|s| s.4 == site.function)
                .map(|s| s.1)
                .collect();
            if exact.len() == 1 && old_owners[&(&site.function, print.size)].len() == 1 {
                *votes.entry(*exact.iter().next().unwrap()).or_default() += 3;
                continue;
            }
            let mut scored: Vec<(f32, u16)> = candidates
                .iter()
                .map(|s| (similarity(&site.function.0, &s.4.0), s.1))
                .collect();
            scored.sort_by(|a, b| b.0.total_cmp(&a.0));
            match scored.as_slice() {
                [(best, opcode)] if *best >= CLOSE_SIMILARITY => {
                    *votes.entry(*opcode).or_default() += 1
                }
                [(best, opcode), (second, other), ..]
                    if *best >= CLOSE_SIMILARITY
                        && (best - second >= CLEAR_MARGIN || opcode == other) =>
                {
                    *votes.entry(*opcode).or_default() += 1
                }
                _ => {}
            }
        }
        result.insert(&print.name, winner(&votes));
    }

    // The rest by order: functions found predict where the others moved.
    let mut anchors: Vec<(usize, usize)> = Vec::new();
    for print in &prints.client {
        if let Some(Some(opcode)) = result.get(print.name.as_str()) {
            for site in &print.sites {
                for s in site_prints
                    .iter()
                    .filter(|s| s.1 == *opcode && s.2 == print.size + 16)
                {
                    anchors.push((site.function_at, s.3));
                }
            }
        }
    }
    anchors.sort();
    anchors.dedup();
    for print in &prints.client {
        if result.get(print.name.as_str()).copied().flatten().is_some() {
            continue;
        }
        let mut votes: BTreeMap<u16, usize> = BTreeMap::new();
        for site in &print.sites {
            let i = anchors.partition_point(|(old, _)| *old < site.function_at);
            for j in [i.wrapping_sub(1), i] {
                let Some(&(old, new)) = anchors.get(j) else {
                    continue;
                };
                let predicted = (site.function_at as i64 - old as i64 + new as i64) as usize;
                for s in site_prints.iter().filter(|s| s.2 == print.size + 16) {
                    if s.3.abs_diff(predicted) <= CLIENT_ORDER_WINDOW
                        && similarity(&site.function.0, &s.4.0) >= ORDER_SIMILARITY
                    {
                        *votes.entry(s.1).or_default() += 1;
                    }
                }
            }
        }
        result.insert(&print.name, winner(&votes));
    }

    for print in &prints.client {
        let value = match result.get(print.name.as_str()).copied().flatten() {
            Some(opcode) => Found::Opcode(opcode),
            None => Found::NotFound,
        };
        found.insert((print.kind.clone(), print.name.clone()), value);
    }
}

fn winner(votes: &BTreeMap<u16, usize>) -> Option<u16> {
    let mut sorted: Vec<(usize, u16)> = votes
        .iter()
        .map(|(opcode, count)| (*count, *opcode))
        .collect();
    sorted.sort_by(|a, b| b.cmp(a));
    match sorted.as_slice() {
        [(_, opcode)] => Some(*opcode),
        [(first, opcode), (second, _), ..] if first > second => Some(*opcode),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fork correction of upstream's opcode (`corrects` in fork_opcodes.yml) replaces it while upstream still has the
    /// number it corrects, and is ignored once upstream changes it (7.58's EditSearchInfo: upstream kept 424.)
    #[test]
    fn a_correction_replaces_upstreams_wrong_opcode() {
        let upstream = "ClientZoneIpcType:\n- name: EditSearchInfo\n  opcode: 424\n  size: 216\n";
        let fork = "ClientZoneIpcType:\n- name: EditSearchInfo\n  corrects: 424\n  opcode: 645\n  size: 216\n  found_on: v2\n";
        let opcode = |upstream: &str, version: &str| {
            known_from_yaml(upstream, fork, version).unwrap()[0].opcode
        };
        assert_eq!(opcode(upstream, "v2"), 645);
        // Another version: the correction isn't for it.
        assert_eq!(opcode(upstream, "v3"), 424);
        // Upstream fixed it (or moved on): theirs.
        assert_eq!(opcode(&upstream.replace("424", "700"), "v2"), 700);
    }

    /// The shipped fingerprints (`resources/data/opcode_fingerprints.json`) are what finds every opcode after the next
    /// game update, so on the version they were recorded on they must have every fork opcode known there, at the same
    /// opcode, and no packet at an opcode it no longer has (re-record them with `kawari-opcodescan <exe> fingerprint`
    /// when an opcode is added or corrected.)
    #[test]
    fn shipped_fingerprints_cover_every_opcode() {
        let prints: Fingerprints = serde_json::from_str(include_str!(
            "../../resources/data/opcode_fingerprints.json"
        ))
        .expect("the shipped fingerprints don't parse");
        let printed = |kind: &str, name: &str| {
            prints
                .server
                .iter()
                .map(|print| (&print.kind, &print.name, print.opcode))
                .chain(
                    prints
                        .client
                        .iter()
                        .map(|print| (&print.kind, &print.name, print.opcode)),
                )
                .find(|(k, n, _)| *k == kind && *n == name)
                .map(|(_, _, opcode)| opcode)
        };
        for (kind, name, opcode) in crate::opcodes::KNOWN_OPCODES {
            if let Some(printed) = printed(kind, name) {
                assert_eq!(
                    printed, *opcode,
                    "{kind}::{name} is fingerprinted as {printed} but is {opcode} now: re-record the fingerprints"
                );
            }
        }
        if prints.game_version != crate::constants::SUPPORTED_GAME_VERSION {
            return; // recorded on another version: nothing to compare the fork's opcodes with
        }
        for fork in crate::opcodes::FORK_OPCODES {
            if !fork.current || !fork.kind.contains("Zone") {
                continue;
            }
            assert!(
                printed(fork.kind, fork.name).is_some(),
                "{}::{} has no fingerprint: record them again (kawari-opcodescan <exe> fingerprint ...)",
                fork.kind,
                fork.name
            );
        }
    }

    #[test]
    fn similarity_is_difflibs() {
        // difflib.SequenceMatcher(None, "abcd", "bcde").ratio() == 0.75
        assert_eq!(similarity(&[1, 2, 3, 4], &[2, 3, 4, 5]), 0.75);
        assert_eq!(similarity(&[1, 2, 3], &[1, 2, 3]), 1.0);
        assert_eq!(similarity(&[1, 2], &[3, 4]), 0.0);
    }

    #[test]
    fn prints_round_trip() {
        let prints = Prints(vec![0, 0xDEAD_BEEF, 7]);
        let text = serde_json::to_string(&prints).unwrap();
        assert_eq!(text, "\"00000000deadbeef00000007\"");
        assert_eq!(serde_json::from_str::<Prints>(&text).unwrap(), prints);
    }

    /// The walk reads the opcode from the packet and follows compares to where each opcode goes.
    #[test]
    fn walks_compares() {
        // movzx eax, word [r8+2]; cmp eax, 0x12C; je +2; int3; int3; (je lands on) nop; ret
        let code_bytes = [
            0x41, 0x0F, 0xB7, 0x40, 0x02, // movzx eax, word ptr [r8+2]
            0x3D, 0x2C, 0x01, 0x00, 0x00, // cmp eax, 300
            0x74, 0x02, // je +2
            0x90, 0xC3, // nop; ret
            0x90, 0xC3, // nop; ret
        ];
        let exe = Exe::from_code(&code_bytes, 0x1000);
        let code = Code::new(&exe);
        assert_eq!(packet_register(&code, 0x1000), Some(Register::R8));
        let (decisions, end) = walk(&code, 0x1000, 300, Register::R8, 0, 4);
        assert_eq!(decisions, vec![0x100A]);
        assert_eq!(end, 0x100E);
        let (_, end) = walk(&code, 0x1000, 301, Register::R8, 0, 4);
        assert_eq!(end, 0x100C);
    }
}
