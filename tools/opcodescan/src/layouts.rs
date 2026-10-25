//! `kawari-opcodescan <exe> layouts`: each zone packet's layout as the client's own code reads (server packets) or
//! writes (client packets) it, compared with Kawari's structs (docs/src/fork/packet-layouts.md.)
//!
//! Server packets: from each known opcode's case in the client's zone dispatcher (`opcode_match::packet_sites`), the
//! code is followed (both ways at every branch, into the functions the packet's data is passed to, a few calls
//! deep), keeping track of which registers hold the packet's address (or an address inside it), and every read from
//! it is recorded: its offset from the start of the packet's data, its width, how it's read (signed, unsigned, a
//! float, a 16-byte block, a copy), and arrays (a pointer stepped through it in a loop, or an index scaled by the
//! element's size.) Client packets: in the function that builds one, every write into the packet's data.
//!
//! Nothing of the game is kept: offsets, widths and the names this tool derives (from ClientStructs, when given.)

use std::collections::{BTreeMap, HashMap, HashSet};

use iced_x86::{
    Decoder, DecoderOptions, FlowControl, Instruction, InstructionInfoFactory, Mnemonic, OpAccess,
    OpKind, Register,
};
use kawari::exe_scan::Exe;
use kawari::opcode_match::{self, KnownOpcode, PacketSites};
use serde_json::{Value as Json, json};

use crate::clientstructs::ClientStructs;
use crate::kawari_structs::{KawariField, KawariLayouts};

/// How many calls deep the packet's data is followed from the dispatcher's case.
const MAX_DEPTH: usize = 4;
/// Instructions looked at per function visited, and per packet.
const FUNCTION_BUDGET: usize = 3000;
const PACKET_BUDGET: usize = 40000;
/// The IPC header before a server packet's data (the dispatcher's packet register points at the header.)
const SERVER_HEADER: i64 = 0x10;
/// The client's own header before a client packet's data (opcode at +0, size with the IPC header at +8.)
const CLIENT_HEADER: i64 = 0x20;
/// A function called from more places than this is a utility (memcpy, an allocator): not followed.
const UTILITY_CALLERS: usize = 1000;
/// A copy (memcpy) of the packet's data is recorded as one access when it's at least this long.
const COPY_MIN: i64 = 8;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Val {
    /// An address in the packet: this many bytes from the start of its data.
    Ptr(i64),
    Const(i64),
    /// An address in the function's own frame (`lea r, [rsp + x]`), where the packet may be copied.
    Frame(Register, i64),
}

#[derive(Debug, Clone)]
pub struct Access {
    /// From the start of the packet's data.
    pub offset: i64,
    pub size: u32,
    pub write: bool,
    /// u (unsigned), i (signed), f (float), block (16 bytes at once), copy (memcpy), bool (only compared to 0)
    pub ty: &'static str,
    /// An array: the step between elements, and how many when the loop says.
    pub stride: Option<i64>,
    pub count: Option<i64>,
    /// 0 in the dispatcher's case, 1 in the function it calls, ...
    pub depth: usize,
    pub at: usize,
    /// For a client packet: where the value comes from (a constant, or what.)
    pub source: Option<String>,
    /// From ClientStructs: the packet struct's field at this offset, or the function's name it was read in.
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Visited {
    pub function: usize,
    pub depth: usize,
    /// The argument register the packet's data came in, and where in the data it pointed.
    pub argument: Register,
    pub offset: i64,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PacketLayout {
    pub kind: &'static str,
    pub name: String,
    pub opcode: u16,
    pub size: u32,
    pub start: usize,
    pub shared: usize,
    pub accesses: Vec<Access>,
    pub visited: Vec<Visited>,
    pub virtual_calls: usize,
    /// A ClientStructs struct this packet's data is passed as (named fields.)
    pub struct_name: Option<String>,
}

impl PacketLayout {
    /// How far into the data the client reads (or writes.)
    pub fn extent(&self) -> i64 {
        self.accesses
            .iter()
            .map(|a| {
                let elements = match (a.stride, a.count) {
                    (Some(stride), Some(count)) if count > 0 => stride * (count - 1),
                    _ => 0,
                };
                a.offset + a.size as i64 + elements
            })
            .max()
            .unwrap_or(0)
    }

    pub fn confidence(&self) -> &'static str {
        if self.accesses.is_empty() {
            "none"
        } else if self.shared > 1 {
            "low"
        } else if self.accesses.iter().any(|a| a.depth <= 2) {
            "high"
        } else {
            "medium"
        }
    }
}

struct Analyzer<'a> {
    exe: &'a Exe,
    cache: HashMap<usize, Option<Instruction>>,
    info: InstructionInfoFactory,
    names: Option<&'a ClientStructs>,
}

/// One path through a function: where it is, what its registers and stack slots hold.
#[derive(Clone)]
struct PathState {
    pc: usize,
    regs: HashMap<Register, Val>,
    stack: HashMap<(Register, i64), Val>,
    /// Copies of the packet into the frame: (base register, where, the packet offset copied from.)
    copies: Vec<(Register, i64, i64)>,
    /// Registers still holding what the function was called with (not written since), and whether it has called
    /// anything yet: such a register left over from before a call isn't an argument of the next one.
    stale: HashSet<Register>,
    called: bool,
}

struct Run {
    accesses: Vec<Access>,
    visited: Vec<Visited>,
    seen_calls: HashSet<(usize, Vec<(Register, i64)>)>,
    virtual_calls: usize,
    steps: usize,
}

const ARGUMENTS: [Register; 4] = [Register::RCX, Register::RDX, Register::R8, Register::R9];
const VOLATILE: [Register; 7] = [
    Register::RAX,
    Register::RCX,
    Register::RDX,
    Register::R8,
    Register::R9,
    Register::R10,
    Register::R11,
];

impl<'a> Analyzer<'a> {
    fn new(exe: &'a Exe, names: Option<&'a ClientStructs>) -> Self {
        Self {
            exe,
            cache: HashMap::new(),
            info: InstructionInfoFactory::new(),
            names,
        }
    }

    fn at(&mut self, rva: usize) -> Option<Instruction> {
        if let Some(cached) = self.cache.get(&rva) {
            return *cached;
        }
        let (start, end) = self.exe.text();
        let decoded = (start..end).contains(&rva).then(|| {
            let image = self.exe.image();
            let bytes = &image[rva..(rva + 15).min(image.len())];
            Decoder::with_ip(64, bytes, rva as u64, DecoderOptions::NONE).decode()
        });
        let decoded = decoded.filter(|i| !i.is_invalid());
        self.cache.insert(rva, decoded);
        decoded
    }

    /// Follows a function (or a case) from `start` with `regs`, recording what's read from the packet.
    fn follow(
        &mut self,
        run: &mut Run,
        start: usize,
        regs: HashMap<Register, Val>,
        depth: usize,
        write_mode: bool,
    ) {
        let function = self.exe.function_start(start);
        let stale: HashSet<Register> = if depth == 0 {
            HashSet::new()
        } else {
            regs.keys().copied().collect()
        };
        let mut todo = vec![PathState {
            pc: start,
            regs,
            stack: HashMap::new(),
            copies: Vec::new(),
            stale,
            called: false,
        }];
        let mut seen: HashSet<usize> = HashSet::new();
        let mut steps = 0;
        // Pointers stepped in a loop: (register, the value it had, the step.)
        let mut strides: Vec<(i64, i64, Option<i64>)> = Vec::new();
        let first_access = run.accesses.len();
        while let Some(mut state) = todo.pop() {
            loop {
                if !seen.insert(state.pc) || steps >= FUNCTION_BUDGET || run.steps >= PACKET_BUDGET
                {
                    break;
                }
                steps += 1;
                run.steps += 1;
                let Some(ins) = self.at(state.pc) else {
                    break;
                };
                let next = ins.next_ip() as usize;
                self.memory(run, &ins, &state, depth, write_mode);

                match ins.flow_control() {
                    FlowControl::Return | FlowControl::Interrupt | FlowControl::Exception => break,
                    FlowControl::Call => {
                        self.call(run, &ins, &mut state, depth, write_mode);
                        state.pc = next;
                        continue;
                    }
                    FlowControl::IndirectCall => {
                        if ARGUMENTS
                            .iter()
                            .any(|r| matches!(state.regs.get(r), Some(Val::Ptr(_))))
                        {
                            run.virtual_calls += 1;
                        }
                        for r in VOLATILE {
                            state.regs.remove(&r);
                        }
                        state.pc = next;
                        continue;
                    }
                    FlowControl::UnconditionalBranch => {
                        if ins.op0_kind() == OpKind::NearBranch64 {
                            let to = ins.near_branch64() as usize;
                            if self.exe.function_start(to) != function && to != function {
                                // A tail call (passing on what it was called with.)
                                state.called = false;
                                self.call(run, &ins, &mut state, depth, write_mode);
                                break;
                            }
                            state.pc = to;
                            continue;
                        }
                        break;
                    }
                    FlowControl::IndirectBranch => break,
                    FlowControl::ConditionalBranch => {
                        if ins.op0_kind() == OpKind::NearBranch64 {
                            let to = ins.near_branch64() as usize;
                            let mut other = state.clone();
                            other.pc = to;
                            todo.push(other);
                        }
                        state.pc = next;
                        continue;
                    }
                    _ => {}
                }
                self.registers(&ins, &mut state, &mut strides, next);
                state.pc = next;
            }
        }
        // Accesses through a pointer stepped in a loop are arrays.
        for (from, step, count) in strides {
            for access in &mut run.accesses[first_access..] {
                if access.stride.is_none()
                    && access.offset >= from
                    && access.offset < from + step
                    && access.size as i64 <= step
                {
                    access.stride = Some(step);
                    access.count = count;
                }
            }
        }
    }

    fn memory(
        &mut self,
        run: &mut Run,
        ins: &Instruction,
        state: &PathState,
        depth: usize,
        write_mode: bool,
    ) {
        if ins.mnemonic() == Mnemonic::Lea || ins.mnemonic() == Mnemonic::Nop {
            return;
        }
        let info = self.info.info(ins);
        for used in info.used_memory() {
            let base = used.base().full_register();
            let Some(Val::Ptr(p)) = state.regs.get(&base).copied() else {
                continue;
            };
            let write = matches!(
                used.access(),
                OpAccess::Write
                    | OpAccess::CondWrite
                    | OpAccess::ReadWrite
                    | OpAccess::ReadCondWrite
            );
            if write_mode != write && !(write_mode && used.access() == OpAccess::ReadWrite) {
                continue;
            }
            let mut offset = p + used.displacement() as i64;
            let mut stride = None;
            if used.index() != Register::None {
                match state.regs.get(&used.index().full_register()) {
                    Some(Val::Const(c)) => offset += c * used.scale() as i64,
                    _ => stride = Some(used.scale() as i64),
                }
            }
            let size = used.memory_size().size() as u32;
            if size == 0 {
                continue;
            }
            let ty = type_of(ins, size);
            let source = write_mode.then(|| source_of(ins, state)).flatten();
            run.accesses.push(Access {
                offset,
                size,
                write,
                ty,
                stride: stride.filter(|s| *s > 1),
                count: None,
                depth,
                at: ins.ip() as usize,
                source,
                name: None,
            });
        }
    }

    fn call(
        &mut self,
        run: &mut Run,
        ins: &Instruction,
        state: &mut PathState,
        depth: usize,
        write_mode: bool,
    ) {
        let target = (ins.op0_kind() == OpKind::NearBranch64).then(|| ins.near_branch64() as usize);
        // Its arguments: as many as ClientStructs says it takes, when it names it; a register left over from what this
        // function was called with only before its first call; never a memory utility.
        let name = target.and_then(|t| {
            self.names
                .and_then(|names| names.function_name(self.exe, t))
        });
        // Called from all over the game: a utility (memcpy, an allocator), not where the packet is understood.
        let common = target.is_some_and(|t| self.exe.calls_to(t).len() > UTILITY_CALLERS);
        let utility = common
            || name.as_ref().is_some_and(|n| {
                let n = n.to_lowercase();
                [
                    "memset", "memcpy", "memmove", "free", "alloc", "dtor", "destroy",
                ]
                .iter()
                .any(|u| n.contains(u))
            });
        let taken = target
            .and_then(|t| self.names.and_then(|names| names.argument_count(t)))
            .unwrap_or(ARGUMENTS.len());
        let args: Vec<(Register, i64)> = ARGUMENTS
            .iter()
            .take(taken)
            .filter(|r| !(state.called && state.stale.contains(r)))
            .filter_map(|r| match state.regs.get(r) {
                Some(Val::Ptr(p)) => Some((*r, *p)),
                _ => None,
            })
            .filter(|_| !utility)
            .collect();
        // memcpy(somewhere, packet + p, n): one copy of n bytes.
        if let (Some(Val::Ptr(from)), Some(Val::Const(n))) = (
            state.regs.get(&Register::RDX),
            state.regs.get(&Register::R8),
        ) && !matches!(state.regs.get(&Register::RCX), Some(Val::Ptr(_)))
            && (COPY_MIN..=0x4000).contains(n)
            && !write_mode
        {
            run.accesses.push(Access {
                offset: *from,
                size: *n as u32,
                write: false,
                ty: "copy",
                stride: None,
                count: None,
                depth,
                at: ins.ip() as usize,
                source: None,
                name: None,
            });
        } else if let Some(target) = target
            && !args.is_empty()
            && depth < MAX_DEPTH
            && !write_mode
            && run.seen_calls.insert((target, args.clone()))
        {
            for (register, offset) in &args {
                run.visited.push(Visited {
                    function: target,
                    depth: depth + 1,
                    argument: *register,
                    offset: *offset,
                    name: name.clone(),
                });
            }
            let regs: HashMap<Register, Val> =
                args.iter().map(|(r, p)| (*r, Val::Ptr(*p))).collect();
            self.follow(run, target, regs, depth + 1, false);
        }
        note_copy(state);
        let probe = matches!(state.regs.get(&Register::RAX), Some(Val::Const(n)) if *n >= 0x1000);
        for r in VOLATILE {
            if !probe || matches!(r, Register::RAX | Register::R10 | Register::R11) {
                state.regs.remove(&r);
                state.stale.remove(&r);
            }
        }
        if !probe {
            state.called = true;
        }
    }

    fn registers(
        &mut self,
        ins: &Instruction,
        state: &mut PathState,
        strides: &mut Vec<(i64, i64, Option<i64>)>,
        next: usize,
    ) {
        let dest = (ins.op_count() > 0 && ins.op0_kind() == OpKind::Register)
            .then(|| ins.op0_register().full_register());
        let mnemonic = ins.mnemonic();
        if let Some(dest) = dest {
            state.stale.remove(&dest);
        }
        let stack_slot = |ins: &Instruction| -> Option<(Register, i64)> {
            let base = ins.memory_base().full_register();
            (matches!(base, Register::RSP | Register::RBP) && ins.memory_index() == Register::None)
                .then(|| (base, ins.memory_displacement64() as i64))
        };
        let mut handled = false;
        match mnemonic {
            Mnemonic::Mov | Mnemonic::Movzx | Mnemonic::Movsx | Mnemonic::Movsxd
                if ins.op_count() == 2 =>
            {
                if let Some(dest) = dest {
                    let value = match ins.op1_kind() {
                        OpKind::Register => {
                            state.regs.get(&ins.op1_register().full_register()).copied()
                        }
                        OpKind::Memory => {
                            stack_slot(ins).and_then(|slot| state.stack.get(&slot).copied())
                        }
                        kind if is_immediate(kind) => Some(Val::Const(ins.immediate(1) as i64)),
                        _ => None,
                    };
                    // A pointer only survives a 64-bit move.
                    let value = value
                        .filter(|v| matches!(v, Val::Const(_)) || ins.op0_register().size() == 8);
                    match value {
                        Some(v) => {
                            state.regs.insert(dest, v);
                        }
                        None => {
                            state.regs.remove(&dest);
                        }
                    }
                    handled = true;
                } else if ins.op0_kind() == OpKind::Memory
                    && ins.op1_kind() == OpKind::Register
                    && let Some(slot) = stack_slot(ins)
                {
                    match state.regs.get(&ins.op1_register().full_register()).copied() {
                        Some(v) => {
                            state.stack.insert(slot, v);
                        }
                        None => {
                            state.stack.remove(&slot);
                        }
                    }
                    handled = true;
                }
            }
            Mnemonic::Lea => {
                if let Some(dest) = dest {
                    let base = ins.memory_base().full_register();
                    let frame_copy = matches!(base, Register::RSP | Register::RBP)
                        .then(|| {
                            let at = ins.memory_displacement64() as i64;
                            state
                                .copies
                                .iter()
                                .filter(|(b, from, _)| {
                                    *b == base && *from <= at && at < from + 0x4000
                                })
                                .max_by_key(|(_, from, _)| *from)
                                .map(|(_, from, p)| Val::Ptr(p + at - from))
                                .unwrap_or(Val::Frame(base, at))
                        })
                        .filter(|_| {
                            ins.memory_index() == Register::None && !state.regs.contains_key(&base)
                        });
                    let value = match state.regs.get(&base) {
                        _ if frame_copy.is_some() => frame_copy,
                        Some(Val::Ptr(p)) => {
                            let mut p = p + ins.memory_displacement64() as i64;
                            match ins.memory_index() {
                                Register::None => Some(Val::Ptr(p)),
                                index => match state.regs.get(&index.full_register()) {
                                    Some(Val::Const(c)) => {
                                        p += c * ins.memory_index_scale() as i64;
                                        Some(Val::Ptr(p))
                                    }
                                    _ => None,
                                },
                            }
                        }
                        _ => None,
                    };
                    if let (Some(Val::Ptr(old)), Some(Val::Ptr(new))) =
                        (state.regs.get(&dest).copied(), value)
                        && base == dest
                        && new > old
                    {
                        strides.push((old, new - old, self.loop_count(next)));
                    }
                    match value {
                        Some(v) if ins.op0_register().size() == 8 => {
                            state.regs.insert(dest, v);
                        }
                        _ => {
                            state.regs.remove(&dest);
                        }
                    }
                    handled = true;
                }
            }
            _ if dest == Some(Register::RSP) => {
                // The stack pointer stands for the function's frame (a client packet's buffer is in it): its
                // prologue's adjustments aren't followed, and nothing else changes it.
                handled = true;
            }
            Mnemonic::Add | Mnemonic::Sub if dest.is_some() && is_immediate(ins.op1_kind()) => {
                let dest = dest.unwrap();
                let k = ins.immediate(1) as i64;
                let k = if mnemonic == Mnemonic::Sub { -k } else { k };
                match state.regs.get(&dest).copied() {
                    Some(Val::Ptr(p)) => {
                        if k > 0 {
                            strides.push((p, k, self.loop_count(next)));
                        }
                        state.regs.insert(dest, Val::Ptr(p + k));
                    }
                    Some(Val::Const(c)) => {
                        state.regs.insert(dest, Val::Const(c + k));
                    }
                    Some(Val::Frame(base, at)) => {
                        state.regs.insert(dest, Val::Frame(base, at + k));
                    }
                    None => {}
                }
                handled = true;
            }
            Mnemonic::Xor
                if dest.is_some()
                    && ins.op1_kind() == OpKind::Register
                    && ins.op1_register().full_register() == dest.unwrap() =>
            {
                state.regs.insert(dest.unwrap(), Val::Const(0));
                handled = true;
            }
            _ => {}
        }
        if handled {
            return;
        }
        let info = self.info.info(ins);
        for used in info.used_registers() {
            if matches!(
                used.access(),
                OpAccess::Write
                    | OpAccess::CondWrite
                    | OpAccess::ReadWrite
                    | OpAccess::ReadCondWrite
            ) && used.register().full_register() != Register::RSP
            {
                state.regs.remove(&used.register().full_register());
            }
        }
    }

    /// Where the dispatcher holds the packet when it reaches each of `cases`: its code followed from the start with
    /// the packet in `packet`, the registers holding it noted at each case reached directly (a compare tree), and at
    /// its jump tables (under the key 0) for the cases reached through one.
    fn case_states(
        &mut self,
        dispatcher: usize,
        packet: Register,
        cases: &HashSet<usize>,
    ) -> HashMap<usize, HashMap<Register, Val>> {
        let mut found: HashMap<usize, HashMap<Register, Val>> = HashMap::new();
        let mut todo = vec![PathState {
            pc: dispatcher,
            regs: HashMap::from([(packet, Val::Ptr(-SERVER_HEADER))]),
            stack: HashMap::new(),
            copies: Vec::new(),
            stale: HashSet::new(),
            called: false,
        }];
        let mut seen: HashSet<usize> = HashSet::new();
        let mut steps = 0;
        let only_pointers = |regs: &HashMap<Register, Val>| -> HashMap<Register, Val> {
            regs.iter()
                .filter(|(_, v)| matches!(v, Val::Ptr(_)))
                .map(|(r, v)| (*r, *v))
                .collect()
        };
        while let Some(mut state) = todo.pop() {
            loop {
                if steps > 60000 || !seen.insert(state.pc) {
                    break;
                }
                steps += 1;
                if cases.contains(&state.pc) {
                    found
                        .entry(state.pc)
                        .or_insert_with(|| only_pointers(&state.regs));
                    break;
                }
                let Some(ins) = self.at(state.pc) else { break };
                let next = ins.next_ip() as usize;
                match ins.flow_control() {
                    FlowControl::Return | FlowControl::Interrupt | FlowControl::Exception => break,
                    FlowControl::IndirectBranch => {
                        found.entry(0).or_insert_with(|| only_pointers(&state.regs));
                        break;
                    }
                    FlowControl::Call | FlowControl::IndirectCall => {
                        // The stack probe (`mov eax, frame size` then a call) only changes rax, r10 and r11.
                        note_copy(&mut state);
                        let probe = matches!(state.regs.get(&Register::RAX), Some(Val::Const(n)) if *n >= 0x1000);
                        for r in VOLATILE {
                            if !probe || matches!(r, Register::RAX | Register::R10 | Register::R11)
                            {
                                state.regs.remove(&r);
                            }
                        }
                        state.pc = next;
                        continue;
                    }
                    FlowControl::UnconditionalBranch => {
                        if ins.op0_kind() == OpKind::NearBranch64 {
                            state.pc = ins.near_branch64() as usize;
                            continue;
                        }
                        break;
                    }
                    FlowControl::ConditionalBranch => {
                        if ins.op0_kind() == OpKind::NearBranch64 {
                            let mut other = state.clone();
                            other.pc = ins.near_branch64() as usize;
                            todo.push(other);
                        }
                        state.pc = next;
                        continue;
                    }
                    _ => {}
                }
                let mut strides = Vec::new();
                self.registers(&ins, &mut state, &mut strides, next);
                state.pc = next;
            }
        }
        found
    }

    /// Registers a function copies `register` into before its first branch, of those a call keeps (or `register`
    /// itself if there's none.)
    fn prologue_copies(&mut self, function: usize, register: Register) -> Vec<Register> {
        const KEPT: [Register; 8] = [
            Register::RBX,
            Register::RBP,
            Register::RSI,
            Register::RDI,
            Register::R12,
            Register::R13,
            Register::R14,
            Register::R15,
        ];
        let mut holding = vec![register];
        let mut at = function;
        for _ in 0..200 {
            let Some(ins) = self.at(at) else { break };
            if matches!(
                ins.flow_control(),
                FlowControl::ConditionalBranch | FlowControl::Return | FlowControl::IndirectBranch
            ) {
                break;
            }
            if ins.mnemonic() == Mnemonic::Mov
                && ins.op_count() == 2
                && ins.op0_kind() == OpKind::Register
                && ins.op1_kind() == OpKind::Register
                && ins.op0_register().size() == 8
            {
                let from = ins.op1_register().full_register();
                let to = ins.op0_register().full_register();
                holding.retain(|r| *r != to);
                if holding.contains(&from) {
                    holding.push(to);
                }
            }
            at = ins.next_ip() as usize;
        }
        let kept: Vec<Register> = holding
            .iter()
            .copied()
            .filter(|r| KEPT.contains(r))
            .collect();
        if kept.is_empty() { holding } else { kept }
    }

    /// A loop's count: a compare with a number just after a pointer is stepped, before a jump back.
    fn loop_count(&mut self, from: usize) -> Option<i64> {
        let mut at = from;
        let mut count = None;
        for _ in 0..8 {
            let ins = self.at(at)?;
            if ins.mnemonic() == Mnemonic::Cmp && is_immediate(ins.op1_kind()) {
                count = Some(ins.immediate(1) as i64);
            }
            if ins.flow_control() == FlowControl::ConditionalBranch {
                return count.filter(|_| (ins.near_branch64() as usize) < from);
            }
            at = ins.next_ip() as usize;
        }
        None
    }
}

/// A call that copies the packet into the frame (`rcx` an address in it, `rdx` in the packet): noted, so the copy is
/// followed as the packet.
fn note_copy(state: &mut PathState) {
    if let (Some(Val::Frame(base, at)), Some(Val::Ptr(p))) = (
        state.regs.get(&Register::RCX).copied(),
        state.regs.get(&Register::RDX).copied(),
    ) {
        state.copies.push((base, at, p));
    }
}

fn is_immediate(kind: OpKind) -> bool {
    matches!(
        kind,
        OpKind::Immediate8
            | OpKind::Immediate16
            | OpKind::Immediate32
            | OpKind::Immediate64
            | OpKind::Immediate8to16
            | OpKind::Immediate8to32
            | OpKind::Immediate8to64
            | OpKind::Immediate32to64
    )
}

/// How an access reads its value: floats only where it's float arithmetic, a float compare or conversion, or a
/// single-precision move (8-byte xmm moves are how the compiler copies two numbers at once.)
fn type_of(ins: &Instruction, size: u32) -> &'static str {
    if size == 16 || size == 32 {
        return "block";
    }
    let name = format!("{:?}", ins.mnemonic()).to_lowercase();
    let float_op = [
        "addss",
        "subss",
        "mulss",
        "divss",
        "comiss",
        "ucomiss",
        "maxss",
        "minss",
        "sqrtss",
        "cvtss2sd",
        "cvtss2si",
        "cvttss2si",
        "movss",
        "vmovss",
        "vaddss",
        "vsubss",
        "vmulss",
        "vdivss",
        "vcomiss",
        "vucomiss",
        "addsd",
        "subsd",
        "mulsd",
        "divsd",
        "comisd",
        "ucomisd",
        "cvtsd2ss",
        "cvtsd2si",
        "cvttsd2si",
    ];
    if float_op.contains(&name.as_str()) && (size == 4 || size == 8) {
        return "f";
    }
    match ins.mnemonic() {
        Mnemonic::Movsx | Mnemonic::Movsxd => "i",
        Mnemonic::Cmp | Mnemonic::Test
            if size == 1 && is_immediate(ins.op1_kind()) && ins.immediate(1) == 0 =>
        {
            "bool"
        }
        _ => "u",
    }
}

/// Where a client packet's written value comes from.
fn source_of(ins: &Instruction, state: &PathState) -> Option<String> {
    if ins.op_count() < 2 {
        return None;
    }
    Some(match ins.op1_kind() {
        kind if is_immediate(kind) => format!("constant {}", ins.immediate(1) as i64),
        OpKind::Register => match state.regs.get(&ins.op1_register().full_register()) {
            Some(Val::Const(c)) => format!("constant {c}"),
            _ => "variable".to_string(),
        },
        _ => "variable".to_string(),
    })
}

/// Every known packet's layout from `exe`.
pub fn layouts(
    exe: &Exe,
    known: &[KnownOpcode],
    names: Option<&ClientStructs>,
) -> Result<Vec<PacketLayout>, String> {
    let sites: PacketSites = opcode_match::packet_sites(exe, known)?;
    let mut analyzer = Analyzer::new(exe, names);
    let mut out = Vec::new();
    // The registers the dispatcher keeps the packet in through its cases: copies made before its first branch, into
    // registers a call keeps (the one it arrived in is reused.)
    let packet_copies = analyzer.prologue_copies(sites.dispatcher, sites.packet);
    let case_states = analyzer.case_states(
        sites.dispatcher,
        sites.packet,
        &sites
            .server
            .iter()
            .map(|s| s.case)
            .collect::<HashSet<usize>>(),
    );

    for site in &sites.server {
        let mut run = Run {
            accesses: Vec::new(),
            visited: Vec::new(),
            seen_calls: HashSet::new(),
            virtual_calls: 0,
            steps: 0,
        };
        let regs: HashMap<Register, Val> = case_states
            .get(&site.case)
            .or(case_states.get(&0))
            .cloned()
            .filter(|regs| !regs.is_empty())
            .unwrap_or_else(|| {
                packet_copies
                    .iter()
                    .map(|r| (*r, Val::Ptr(-SERVER_HEADER)))
                    .collect()
            });
        analyzer.follow(&mut run, site.case, regs, 0, false);
        run.accesses.retain(|a| a.offset >= 0 && !a.write);
        out.push(finish(
            "server",
            &site.name,
            site.opcode,
            site.size,
            site.case,
            site.case_group,
            run,
            names,
        ));
    }

    // Client packets: the writes into the packet's data in the function that builds it (each site of one packet
    // together.)
    let mut by_name: BTreeMap<(String, u16, u32), Vec<(usize, usize)>> = BTreeMap::new();
    for site in &sites.client {
        by_name
            .entry((site.name.clone(), site.opcode, site.size))
            .or_default()
            .push((site.site, site.function));
    }
    for ((name, opcode, size), places) in by_name {
        let mut run = Run {
            accesses: Vec::new(),
            visited: Vec::new(),
            seen_calls: HashSet::new(),
            virtual_calls: 0,
            steps: 0,
        };
        for (site, function) in &places {
            let Some(ins) = analyzer.at(*site) else {
                continue;
            };
            let base = ins.memory_base().full_register();
            let header = ins.memory_displacement64() as i64;
            let data = header + CLIENT_HEADER;
            // The building function's frame: its base register points `data` bytes before the packet's data.
            let regs = HashMap::from([(base, Val::Ptr(-data))]);
            let mut sub = Run {
                accesses: Vec::new(),
                visited: Vec::new(),
                seen_calls: HashSet::new(),
                virtual_calls: 0,
                steps: 0,
            };
            analyzer.follow(&mut sub, *function, regs, 0, true);
            run.accesses.extend(
                sub.accesses
                    .into_iter()
                    .filter(|a| a.offset >= 0 && a.offset < size as i64 && a.write),
            );
        }
        let start = places.first().map(|(_, f)| *f).unwrap_or(0);
        out.push(finish(
            "client",
            &name,
            opcode,
            size,
            start,
            places.len(),
            run,
            names,
        ));
    }
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
fn finish(
    kind: &'static str,
    name: &str,
    opcode: u16,
    size: u32,
    start: usize,
    shared: usize,
    run: Run,
    names: Option<&ClientStructs>,
) -> PacketLayout {
    let mut accesses = run.accesses;
    accesses.sort_by_key(|a| (a.offset, a.size, a.depth));
    // One line per (offset, size, kind), the shallowest kept.
    let mut seen: HashSet<(i64, u32, &'static str)> = HashSet::new();
    accesses.retain(|a| seen.insert((a.offset, a.size, a.ty)));

    // A ClientStructs packet struct: named by its handler's parameter, or by the packet's own name.
    let struct_name = names.and_then(|names| {
        run.visited
            .iter()
            .filter(|v| v.offset == 0)
            .find_map(|v| names.parameter_struct(v.function, v.argument))
            .or_else(|| names.packet_struct_named(name))
    });
    if let (Some(names), Some(struct_name)) = (names, &struct_name) {
        for access in &mut accesses {
            access.name = names.field_at(struct_name, access.offset);
        }
    }
    let shared = if kind == "server" { shared } else { 1 };
    PacketLayout {
        kind,
        name: name.to_string(),
        opcode,
        size,
        start,
        shared,
        accesses,
        visited: run.visited,
        virtual_calls: run.virtual_calls,
        struct_name,
    }
}

// --- comparing with Kawari ----------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Finding {
    pub packet: String,
    pub kind: &'static str,
    /// size, width, type, unknown-used, padding-read
    pub what: &'static str,
    pub offset: i64,
    pub detail: String,
    pub confident: bool,
}

/// What differs between the client's layouts and Kawari's structs.
pub fn compare(layouts: &[PacketLayout], kawari: &KawariLayouts) -> Vec<Finding> {
    let mut findings = Vec::new();
    for layout in layouts {
        let list = if layout.kind == "server" {
            "ServerZoneIpcData"
        } else {
            "ClientZoneIpcData"
        };
        let confident = layout.confidence() == "high";
        let extent = layout.extent();
        let extent = layout
            .accesses
            .iter()
            .filter(|a| a.ty != "block")
            .map(|a| a.offset + a.size as i64)
            .max()
            .unwrap_or(0)
            .max(if layout.kind == "server" { extent } else { 0 });
        if extent > layout.size as i64 && layout.shared <= 1 {
            findings.push(Finding {
                packet: layout.name.clone(),
                kind: layout.kind,
                what: "size",
                offset: extent,
                detail: format!(
                    "the client {} up to byte {extent:#x}, past Kawari's size {:#x}",
                    if layout.kind == "server" {
                        "reads"
                    } else {
                        "writes"
                    },
                    layout.size
                ),
                // opcodes.yml's sizes are retail's: reading past one more likely means a handler shared with a
                // bigger packet than a wrong size.
                confident: false,
            });
        }
        let Some(fields) = kawari.variant(list, &layout.name) else {
            continue;
        };
        for access in &layout.accesses {
            if access.stride.is_some() || access.ty == "copy" || access.ty == "block" {
                continue;
            }
            let field = fields.iter().find(|f| {
                f.size.is_some_and(|size| {
                    access.offset >= f.offset && access.offset < f.offset + size as i64
                })
            });
            match field {
                None => {
                    if fields.iter().all(|f| f.size.is_some()) && access.offset < layout.size as i64
                    {
                        findings.push(Finding {
                            packet: layout.name.clone(),
                            kind: layout.kind,
                            what: "padding-read",
                            offset: access.offset,
                            detail: format!(
                                "the client {} {} byte(s) ({}) at {:#x}, which Kawari has as padding",
                                if access.write { "writes" } else { "reads" },
                                access.size,
                                access.ty,
                                access.offset
                            ),
                            confident: confident && access.depth <= 2,
                        });
                    }
                }
                Some(field) => {
                    if is_unknown(&field.name) {
                        findings.push(Finding {
                            packet: layout.name.clone(),
                            kind: layout.kind,
                            what: "unknown-used",
                            offset: access.offset,
                            detail: format!(
                                "`{}` ({} at {:#x}): the client {} {} byte(s) ({}) at {:#x}{}",
                                field.name,
                                field.ty,
                                field.offset,
                                if access.write { "writes" } else { "reads" },
                                access.size,
                                access.ty,
                                access.offset,
                                access
                                    .name
                                    .as_ref()
                                    .map(|n| format!(" (ClientStructs: {n})"))
                                    .unwrap_or_default()
                            ),
                            confident: confident && access.depth <= 2,
                        });
                    } else if field.primitive
                        && access.offset == field.offset
                        && field.size.is_some_and(|size| size != access.size)
                        // A wider read is the compiler loading two fields at once.
                        && field.size.is_some_and(|size| access.size < size)
                    {
                        findings.push(Finding {
                            packet: layout.name.clone(),
                            kind: layout.kind,
                            what: "width",
                            offset: access.offset,
                            detail: format!(
                                "`{}` is {} ({} bytes) in Kawari; the client {} {} byte(s) ({}) there",
                                field.name,
                                field.ty,
                                field.size.unwrap_or(0),
                                if access.write { "writes" } else { "reads" },
                                access.size,
                                access.ty
                            ),
                            // A narrower read is often a flag tested in a wider field: only a lead.
                            confident: false,
                        });
                    } else if field.primitive
                        && access.offset == field.offset
                        && access.ty == "f"
                        && !field.ty.starts_with('f')
                    {
                        findings.push(Finding {
                            packet: layout.name.clone(),
                            kind: layout.kind,
                            what: "type",
                            offset: access.offset,
                            detail: format!(
                                "`{}` is {} in Kawari; the client reads it as a float",
                                field.name, field.ty
                            ),
                            confident: confident && access.depth <= 2,
                        });
                    }
                }
            }
        }
    }
    findings
}

fn is_unknown(name: &str) -> bool {
    let name = name.trim_start_matches('_').to_lowercase();
    name.starts_with("unk") || name.starts_with("padding") || name.starts_with("unknown")
}

// --- output -------------------------------------------------------------------------------------------------------

pub fn to_json(layouts: &[PacketLayout], findings: &[Finding], game: &str) -> Json {
    let packets: Vec<Json> = layouts
        .iter()
        .map(|l| {
            json!({
                "kind": l.kind,
                "name": l.name,
                "opcode": l.opcode,
                "size": l.size,
                "confidence": l.confidence(),
                "shared_with": l.shared,
                "extent": l.extent(),
                "struct": l.struct_name,
                "functions": l.visited.iter().map(|v| json!({
                    "rva": format!("{:#x}", v.function), "depth": v.depth,
                    "argument": format!("{:?}", v.argument), "offset": v.offset, "name": v.name,
                })).collect::<Vec<_>>(),
                "start": format!("{:#x}", l.start),
                "virtual_calls": l.virtual_calls,
                "fields": l.accesses.iter().map(|a| json!({
                    "offset": a.offset, "size": a.size, "type": a.ty, "write": a.write,
                    "stride": a.stride, "count": a.count, "depth": a.depth,
                    "source": a.source, "name": a.name, "at": format!("{:#x}", a.at),
                })).collect::<Vec<_>>(),
            })
        })
        .collect();
    json!({
        "game_version": game,
        "packets": packets,
        "findings": findings.iter().map(|f| json!({
            "packet": f.packet, "kind": f.kind, "what": f.what, "offset": f.offset,
            "detail": f.detail, "confident": f.confident,
        })).collect::<Vec<_>>(),
    })
}

pub fn to_markdown(
    layouts: &[PacketLayout],
    findings: &[Finding],
    game: &str,
    kawari: &KawariLayouts,
) -> String {
    let mut md = String::new();
    let count = |kind: &str, confidence: &str| {
        layouts
            .iter()
            .filter(|l| l.kind == kind && l.confidence() == confidence)
            .count()
    };
    md.push_str(&format!(
        "# Packet layouts read from the client ({game})\n\n"
    ));
    md.push_str(
        "Generated by `kawari-opcodescan <exe> layouts` (docs/src/fork/packet-layouts.md).\n\n",
    );
    md.push_str("| | high | medium | low (shared case) | nothing found |\n|---|---|---|---|---|\n");
    for kind in ["server", "client"] {
        md.push_str(&format!(
            "| {kind} | {} | {} | {} | {} |\n",
            count(kind, "high"),
            count(kind, "medium"),
            count(kind, "low"),
            count(kind, "none")
        ));
    }
    md.push_str("\n## Findings\n\n| Packet | | What | Detail |\n|---|---|---|---|\n");
    for f in findings {
        md.push_str(&format!(
            "| {} ({}) | {} | {} | {} |\n",
            f.packet,
            f.kind,
            if f.confident { "confident" } else { "check" },
            f.what,
            f.detail.replace('|', "\\|")
        ));
    }
    md.push_str("\n## Packets\n");
    for l in layouts {
        md.push_str(&format!(
            "\n### {} ({}, opcode {}, {} bytes): {}\n\n",
            l.name,
            l.kind,
            l.opcode,
            l.size,
            l.confidence()
        ));
        if l.shared > 1 && l.kind == "server" {
            md.push_str(&format!(
                "Its case is shared by {} opcodes: what's read there is read for all of them.\n\n",
                l.shared
            ));
        }
        if let Some(s) = &l.struct_name {
            md.push_str(&format!("ClientStructs: `{s}`.\n\n"));
        }
        let named: Vec<String> = l
            .visited
            .iter()
            .filter_map(|v| v.name.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        if !named.is_empty() {
            md.push_str(&format!("Handled in: {}.\n\n", named.join(", ")));
        }
        if l.accesses.is_empty() {
            md.push_str("Nothing read from its data was found (it may be handed on through a virtual call).\n");
            continue;
        }
        let kawari_fields: Vec<KawariField> = kawari
            .variant(
                if l.kind == "server" {
                    "ServerZoneIpcData"
                } else {
                    "ClientZoneIpcData"
                },
                &l.name,
            )
            .cloned()
            .unwrap_or_default();
        md.push_str("| Offset | Size | Type | Array | Depth | Kawari | Notes |\n|---|---|---|---|---|---|---|\n");
        for a in &l.accesses {
            let kawari_field = kawari_fields
                .iter()
                .find(|f| {
                    f.size.is_some_and(|size| {
                        a.offset >= f.offset && a.offset < f.offset + size as i64
                    })
                })
                .map(|f| {
                    if a.offset == f.offset {
                        format!("`{}` ({})", f.name, f.ty)
                    } else {
                        format!("`{}` +{:#x}", f.name, a.offset - f.offset)
                    }
                })
                .unwrap_or_default();
            let array = match (a.stride, a.count) {
                (Some(s), Some(c)) => format!("{c} × {s}"),
                (Some(s), None) => format!("step {s}"),
                _ => String::new(),
            };
            let notes: Vec<String> = [a.name.clone(), a.source.clone()]
                .into_iter()
                .flatten()
                .collect();
            md.push_str(&format!(
                "| {:#x} | {} | {} | {} | {} | {} | {} |\n",
                a.offset,
                a.size,
                a.ty,
                array,
                a.depth,
                kawari_field,
                notes.join("; ")
            ));
        }
    }
    md
}
