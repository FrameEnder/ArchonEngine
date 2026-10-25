//! Writing fork-opcode finders (`find` in `resources/data/fork_opcodes.yml`) from the executable of a version whose
//! opcodes are known, so an opcode found once (from a capture, say) is found by itself after the next game update.
//!
//! For a client packet, the instruction in the function that builds it writing its opcode (`mov [rsp+x], opcode`)
//! with the instructions around it becomes an `immediate` finder; for a server packet, a stretch of its handler becomes
//! a `handler` finder. Addresses (RIP-relative operands, calls and jumps, big displacements and numbers) are left as
//! `??` so the pattern survives the code moving, and the stretch grows until it matches only once. Each finder is
//! checked the way the server will run it (`exe_scan::finder_candidates`) before it's offered. `kawari-opcodescan
//! <exe> suggest-finders` prints them.

use iced_x86::{Decoder, DecoderOptions, FlowControl, Instruction, OpKind};

use crate::exe_scan::{Exe, Finder, finder_candidates, parse_pattern};

/// The most instructions a function is read for: a handler, or a packet's builder (some build it late in a long
/// function: EquipGearsetGlamour's.)
const FUNCTION_LIMIT: usize = 400;
const BUILDER_LIMIT: usize = 6000;
/// The most instructions either side of the opcode's write an `immediate` finder takes.
const IMMEDIATE_REACH: usize = 12;
/// The most instructions a `handler` finder takes, and the fewest fixed bytes either kind may have.
const HANDLER_LENGTH: usize = 24;
const MIN_FIXED_BYTES: usize = 8;
/// Displacements and numbers at least this big are addresses (or may move), so they're left out.
const BIG: u64 = 0x1000;

/// A finder written for an opcode, and what it reads in the executable it was written from.
#[derive(Debug, Clone, PartialEq)]
pub struct Suggestion {
    /// `immediate` or `handler`.
    pub kind: &'static str,
    pub pattern: String,
    /// For `immediate`: where the opcode is in the match, and how many bytes it is.
    pub offset: usize,
    pub width: u8,
    /// Where the match is (RVA.)
    pub at: usize,
}

impl Suggestion {
    /// The finder as the server runs it (the pattern is leaked: finders are `'static`, and only a handful are made.)
    pub fn finder(&self) -> Finder {
        let pattern: &'static str = Box::leak(self.pattern.clone().into_boxed_str());
        match self.kind {
            "immediate" => Finder::Immediate {
                pattern,
                offset: self.offset,
                width: self.width,
                with_opcode: None,
            },
            _ => Finder::Handler { pattern, offset: 0 },
        }
    }

    /// The finder as it goes in fork_opcodes.yml (under `find:`).
    pub fn yaml(&self) -> String {
        match self.kind {
            "immediate" => format!(
                "  - immediate:\n      pattern: {}\n      offset: {}\n      width: {}\n",
                self.pattern, self.offset, self.width
            ),
            _ => format!("  - handler:\n      pattern: {}\n", self.pattern),
        }
    }
}

/// One decoded instruction with its bytes, the ones that may move marked.
struct Piece {
    at: usize,
    bytes: Vec<Option<u8>>,
}

/// The instructions of the function at `start`, as far as its end (a return or jump out followed by padding) or
/// `FUNCTION_LIMIT`.
fn function_pieces(exe: &Exe, start: usize) -> Vec<(Piece, Instruction)> {
    function_pieces_up_to(exe, start, FUNCTION_LIMIT)
}

fn function_pieces_up_to(exe: &Exe, start: usize, limit: usize) -> Vec<(Piece, Instruction)> {
    let (_, text_end) = exe.text();
    let end = (start + limit * 15).min(text_end);
    let Some(bytes) = exe.image().get(start..end) else {
        return Vec::new();
    };
    let mut decoder = Decoder::with_ip(64, bytes, start as u64, DecoderOptions::NONE);
    let mut pieces = Vec::new();
    let mut instruction = Instruction::default();
    while decoder.can_decode() && pieces.len() < limit {
        decoder.decode_out(&mut instruction);
        if instruction.is_invalid() {
            break;
        }
        let at = instruction.ip() as usize;
        let len = instruction.len();
        let raw = &exe.image()[at..at + len];
        let offsets = decoder.get_constant_offsets(&instruction);
        let mut masked: Vec<Option<u8>> = raw.iter().copied().map(Some).collect();
        let mut mask = |offset: usize, size: usize| {
            for byte in masked.iter_mut().skip(offset).take(size) {
                *byte = None;
            }
        };
        if offsets.has_displacement() {
            let displacement = instruction.memory_displacement64();
            let far = instruction.is_ip_rel_memory_operand()
                || (offsets.displacement_size() >= 4
                    && (displacement as i64).unsigned_abs() >= BIG);
            if far {
                mask(offsets.displacement_offset(), offsets.displacement_size());
            }
        }
        let branch = matches!(
            instruction.flow_control(),
            FlowControl::Call
                | FlowControl::UnconditionalBranch
                | FlowControl::ConditionalBranch
                | FlowControl::XbeginXabortXend
        ) && (0..instruction.op_count()).any(|i| {
            matches!(
                instruction.op_kind(i),
                OpKind::NearBranch16 | OpKind::NearBranch32 | OpKind::NearBranch64
            )
        });
        if offsets.has_immediate() {
            let value = immediate(&instruction).unwrap_or(0);
            if branch || value >= BIG && value < u64::MAX - BIG {
                mask(offsets.immediate_offset(), offsets.immediate_size());
            }
        }
        if offsets.has_immediate2() {
            mask(offsets.immediate_offset2(), offsets.immediate_size2());
        }
        let flow = instruction.flow_control();
        pieces.push((Piece { at, bytes: masked }, instruction));
        // The function ends at a return, or a jump on, with padding after it.
        if matches!(flow, FlowControl::Return | FlowControl::UnconditionalBranch)
            && exe.image().get(at + len) == Some(&0xCC)
        {
            break;
        }
    }
    pieces
}

/// An instruction's immediate operand, as the instruction uses it (sign extended); not `immediate64`, which is only
/// right for 64-bit immediates.
fn immediate(instruction: &Instruction) -> Option<u64> {
    (0..instruction.op_count())
        .find(|i| {
            matches!(
                instruction.op_kind(*i),
                OpKind::Immediate8
                    | OpKind::Immediate16
                    | OpKind::Immediate32
                    | OpKind::Immediate64
                    | OpKind::Immediate8to16
                    | OpKind::Immediate8to32
                    | OpKind::Immediate8to64
                    | OpKind::Immediate32to64
            )
        })
        .map(|i| instruction.immediate(i))
}

fn pattern_text(bytes: &[Option<u8>]) -> String {
    bytes
        .iter()
        .map(|byte| byte.map_or("??".to_string(), |b| format!("{b:02X}")))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The places a pattern matches, narrowed as it grows: found with one search of the executable once it has enough
/// fixed bytes, then only those places are checked again (searching all 50 MB for every longer pattern was too slow.)
struct Narrowing {
    places: Option<Vec<usize>>,
}

impl Narrowing {
    fn new() -> Self {
        Self { places: None }
    }

    /// The places `bytes` (starting where the stretch starts) still matches, if it has enough fixed bytes yet.
    fn narrow(&mut self, exe: &Exe, bytes: &[Option<u8>]) -> Option<&[usize]> {
        if bytes.iter().filter(|byte| byte.is_some()).count() < MIN_FIXED_BYTES {
            return None;
        }
        let image = exe.image();
        let fits = |at: usize| {
            image.get(at..at + bytes.len()).is_some_and(|here| {
                here.iter()
                    .zip(bytes)
                    .all(|(byte, want)| want.is_none_or(|want| *byte == want))
            })
        };
        let places = match self.places.take() {
            None => exe.find(&bytes.to_vec()),
            Some(places) => places.into_iter().filter(|at| fits(*at)).collect(),
        };
        self.places = Some(places);
        self.places.as_deref()
    }
}

/// An `immediate` finder for a client packet with `opcode`, built by the function at `function` (from its
/// fingerprint's `function_at`), checked to read exactly `opcode` here.
pub fn suggest_immediate(exe: &Exe, function: usize, opcode: u16) -> Option<Suggestion> {
    let pieces = function_pieces_up_to(exe, function, BUILDER_LIMIT);
    // Where the function writes the opcode: a mov of the number into memory.
    let writes: Vec<usize> = pieces
        .iter()
        .enumerate()
        .filter(|(_, (_, instruction))| {
            instruction.op_count() == 2
                && instruction.op0_kind() == OpKind::Memory
                && matches!(
                    instruction.op1_kind(),
                    OpKind::Immediate16 | OpKind::Immediate32 | OpKind::Immediate32to64
                )
                && immediate(instruction) == Some(opcode as u64)
        })
        .map(|(i, _)| i)
        .collect();
    for write in writes {
        let (piece, instruction) = &pieces[write];
        // The opcode's bytes in the instruction (the last 2 or 4 bytes: the immediate comes last.)
        let width: u8 = if instruction.op1_kind() == OpKind::Immediate16 {
            2
        } else {
            4
        };
        let opcode_offset = piece.bytes.len() - width as usize;
        for before in 0..=IMMEDIATE_REACH.min(write) {
            let from = write - before;
            let mut narrowing = Narrowing::new();
            for after in 0..=IMMEDIATE_REACH {
                let to = write + after + 1;
                if to > pieces.len() {
                    break;
                }
                let mut bytes: Vec<Option<u8>> = Vec::new();
                let mut offset = 0;
                for (i, (piece, _)) in pieces[from..to].iter().enumerate() {
                    if from + i == write {
                        offset = bytes.len() + opcode_offset;
                    }
                    bytes.extend(piece.bytes.iter().copied());
                }
                for byte in bytes.iter_mut().skip(offset).take(width as usize) {
                    *byte = None;
                }
                if narrowing.narrow(exe, &bytes).map(<[usize]>::len) != Some(1) {
                    continue;
                }
                let suggestion = Suggestion {
                    kind: "immediate",
                    pattern: pattern_text(&bytes),
                    offset,
                    width,
                    at: pieces[from].0.at,
                };
                if finder_candidates(exe, &suggestion.finder()).ok().as_deref() == Some(&[opcode]) {
                    return Some(suggestion);
                }
                break;
            }
        }
    }
    None
}

/// A `handler` finder for a server packet with `opcode` whose handler is the function at `handler` (from its
/// fingerprint's `handler_at`), checked to lead to exactly `opcode` here: the earliest stretch of the handler that's
/// unique in the executable. Many handlers only pass the packet on (a few instructions, the same as others'), so the
/// functions they call or jump to are tried too, two levels down: the finder follows calls back up from wherever its
/// pattern is (`exe_scan::opcode_cases`), and the check makes sure that still leads only to this opcode.
pub fn suggest_handler(exe: &Exe, handler: usize, opcode: u16) -> Option<Suggestion> {
    let mut functions = vec![handler];
    let mut seen = std::collections::HashSet::from([handler]);
    let mut level = vec![handler];
    for _ in 0..HANDLER_DEPTH {
        let mut next = Vec::new();
        for function in level {
            for (_, instruction) in function_pieces(exe, function) {
                if !matches!(
                    instruction.flow_control(),
                    FlowControl::Call | FlowControl::UnconditionalBranch
                ) || instruction.op0_kind() != OpKind::NearBranch64
                {
                    continue;
                }
                let target = instruction.near_branch_target() as usize;
                // A jump within the function isn't another function; one called from all over is a utility.
                if exe.function_start(target) != target || exe.calls_to(target).len() > 8 {
                    continue;
                }
                if seen.insert(target) {
                    next.push(target);
                    functions.push(target);
                }
            }
        }
        level = next;
    }
    functions
        .into_iter()
        .take(HANDLER_FUNCTIONS)
        .find_map(|function| suggest_in_function(exe, function, opcode))
}

/// How many levels of callees `suggest_handler` tries, and how many functions in all.
const HANDLER_DEPTH: usize = 2;
const HANDLER_FUNCTIONS: usize = 16;

fn suggest_in_function(exe: &Exe, function: usize, opcode: u16) -> Option<Suggestion> {
    let pieces = function_pieces(exe, function);
    for start in 0..pieces.len() {
        let mut bytes: Vec<Option<u8>> = Vec::new();
        let mut narrowing = Narrowing::new();
        for (piece, _) in pieces.iter().skip(start).take(HANDLER_LENGTH) {
            bytes.extend(piece.bytes.iter().copied());
            match narrowing.narrow(exe, &bytes).map(<[usize]>::len) {
                Some(1) => {}
                Some(0) => break,
                _ => continue,
            }
            // Trailing wildcards add nothing.
            while bytes.last() == Some(&None) {
                bytes.pop();
            }
            let suggestion = Suggestion {
                kind: "handler",
                pattern: pattern_text(&bytes),
                offset: 0,
                width: 0,
                at: pieces[start].0.at,
            };
            if finder_candidates(exe, &suggestion.finder()).ok().as_deref() == Some(&[opcode]) {
                return Some(suggestion);
            }
            break;
        }
    }
    None
}

/// What a finder reads in another executable (another game version): the opcodes, or why it read nothing.
pub fn try_finder(exe: &Exe, suggestion: &Suggestion) -> Result<Vec<u16>, String> {
    parse_pattern(&suggestion.pattern).ok_or("bad pattern")?;
    finder_candidates(exe, &suggestion.finder())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tiny client "builder": writes opcode 0x1234 and a size, then returns, padded.
    #[test]
    fn immediate_finder_from_a_builder() {
        let mut code = vec![0xCCu8; 16];
        let function = 0x1000 + code.len();
        code.extend_from_slice(&[0x48, 0x83, 0xEC, 0x28]); // sub rsp, 28h
        code.extend_from_slice(&[0xC7, 0x44, 0x24, 0x20, 0x34, 0x12, 0x00, 0x00]); // mov dword [rsp+20h], 1234h
        code.extend_from_slice(&[0x48, 0xC7, 0x44, 0x24, 0x28, 0x18, 0x00, 0x00, 0x00]); // mov qword [rsp+28h], 18h
        code.extend_from_slice(&[0x48, 0x83, 0xC4, 0x28, 0xC3]); // add rsp, 28h; ret
        code.extend_from_slice(&[0xCC; 16]);
        let exe = Exe::from_code(&code, 0x1000);
        let suggestion = suggest_immediate(&exe, function, 0x1234).expect("a finder");
        assert_eq!(suggestion.kind, "immediate");
        assert_eq!(
            finder_candidates(&exe, &suggestion.finder()),
            Ok(vec![0x1234])
        );
        assert!(suggestion.pattern.contains("C7 44 24 20 ?? ?? ?? ??"));
    }
}
