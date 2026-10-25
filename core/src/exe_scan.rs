//! Reads opcodes out of the game's executable (ffxiv_dx11.exe), for the fork's own opcodes
//! (`resources/data/fork_opcodes.yml`) after a game update.
//!
//! Nothing is run: the executable's code is searched for byte patterns, and a switch over opcodes is read the way the
//! Kawari Probe reads it in the running game (following calls back from a packet's handler to the case that calls it).

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;
use std::sync::OnceLock;

/// The game's executable, laid out as it is in memory, so addresses (RVAs) can be used directly.
pub struct Exe {
    image: Vec<u8>,
    text_start: usize,
    text_end: usize,
    /// Every call or jump in the code, by where it goes (built the first time it's needed.)
    calls: OnceLock<HashMap<usize, Vec<usize>>>,
}

/// How to find an opcode in the executable (`find` in fork_opcodes.yml.)
#[derive(Debug, Clone, Copy)]
pub enum Finder {
    /// The opcode is written into the code itself (e.g. a client packet being built): it's the `width`-byte number
    /// `offset` bytes into every match of `pattern`, and they must all agree. With `with_opcode`, only matches in a
    /// function that also writes that (known) opcode count, which tells apart code that looks the same.
    Immediate {
        pattern: &'static str,
        offset: usize,
        width: u8,
        with_opcode: Option<u16>,
    },
    /// `pattern` (plus `offset`) is somewhere in the packet's handler: the opcode is the switch case that leads there
    /// (e.g. a server packet the client handles.) The pattern must match once.
    Handler {
        pattern: &'static str,
        offset: isize,
    },
}

/// A byte pattern; `None` matches any byte.
pub type Pattern = Vec<Option<u8>>;

/// Parses a pattern like "48 8B ?? 05" (?? or ? is any byte.)
pub fn parse_pattern(pattern: &str) -> Option<Pattern> {
    pattern
        .split_whitespace()
        .map(|byte| {
            if byte.chars().all(|c| c == '?') {
                Some(None)
            } else {
                u8::from_str_radix(byte, 16).ok().map(Some)
            }
        })
        .collect()
}

/// A switch over opcodes: the case at index `i` is opcode `bias + i`.
#[derive(Debug, Clone)]
pub struct Switch {
    /// Where the switch's table is read.
    pub at: usize,
    pub bias: i64,
    /// Each case's code, and the indices that go there.
    pub cases: BTreeMap<usize, Vec<i64>>,
}

/// One way a call leads to `opcodes`: the call at `site`, in the function at `function`, is in their case of a switch
/// with `switch_cases` cases.
#[derive(Debug, Clone, PartialEq)]
pub struct OpcodeCase {
    pub depth: usize,
    pub site: usize,
    pub function: usize,
    pub switch_cases: usize,
    pub opcodes: Vec<i64>,
}

impl Exe {
    /// Reads the executable at `path`.
    pub fn open(path: &Path) -> Result<Self, String> {
        let file =
            std::fs::read(path).map_err(|err| format!("can't read {}: {err}", path.display()))?;
        Self::from_file_bytes(&file)
    }

    /// Lays a PE file out as it would be in memory.
    pub fn from_file_bytes(file: &[u8]) -> Result<Self, String> {
        let u16_at = |at: usize| -> Result<usize, String> {
            file.get(at..at + 2)
                .map(|b| u16::from_le_bytes([b[0], b[1]]) as usize)
                .ok_or("the file is too short".to_string())
        };
        let u32_at = |at: usize| -> Result<usize, String> {
            file.get(at..at + 4)
                .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize)
                .ok_or("the file is too short".to_string())
        };

        if file.get(0..2) != Some(b"MZ") {
            return Err("not an executable (no MZ header)".to_string());
        }
        let nt = u32_at(0x3C)?;
        if file.get(nt..nt + 4) != Some(b"PE\0\0") {
            return Err("not a PE executable".to_string());
        }
        let section_count = u16_at(nt + 6)?;
        let optional_header_size = u16_at(nt + 20)?;
        let optional_header = nt + 24;
        // SizeOfImage is at the same offset in PE32 and PE32+.
        let image_size = u32_at(optional_header + 56)?;
        if image_size == 0 || image_size > 1 << 30 {
            return Err(format!("unexpected image size {image_size}"));
        }
        let headers_size = u32_at(optional_header + 60)?
            .min(file.len())
            .min(image_size);

        let mut image = vec![0u8; image_size];
        image[..headers_size].copy_from_slice(&file[..headers_size]);

        let mut text = None;
        let mut section = optional_header + optional_header_size;
        for _ in 0..section_count {
            let name: Vec<u8> = file
                .get(section..section + 8)
                .ok_or("the section table is cut off")?
                .iter()
                .copied()
                .take_while(|b| *b != 0)
                .collect();
            let virtual_size = u32_at(section + 8)?;
            let virtual_address = u32_at(section + 12)?;
            let raw_size = u32_at(section + 16)?;
            let raw_pointer = u32_at(section + 20)?;

            let size = raw_size.min(virtual_size.max(raw_size));
            if let (Some(from), Some(to)) = (
                file.get(raw_pointer..raw_pointer + size),
                image.get_mut(virtual_address..virtual_address + size),
            ) {
                to.copy_from_slice(from);
            }
            if name == b".text" {
                text = Some((
                    virtual_address,
                    virtual_address + virtual_size.max(raw_size),
                ));
            }
            section += 40;
        }

        let (text_start, text_end) = text.ok_or("no .text section")?;
        Ok(Self {
            text_end: text_end.min(image.len()),
            image,
            text_start,
            calls: OnceLock::new(),
        })
    }

    /// Lays out raw code as a .text section starting at `text_start`, for tests.
    pub fn from_code(code: &[u8], text_start: usize) -> Self {
        let mut image = vec![0xCCu8; text_start];
        image.extend_from_slice(code);
        let text_end = image.len();
        image.extend_from_slice(&[0xCC; 64]);
        Self {
            image,
            text_start,
            text_end,
            calls: OnceLock::new(),
        }
    }

    /// Where the code is: (start, end) RVAs of .text.
    pub fn text(&self) -> (usize, usize) {
        (self.text_start, self.text_end)
    }

    pub fn image(&self) -> &[u8] {
        &self.image
    }

    pub fn in_text(&self, rva: i64) -> bool {
        rva >= self.text_start as i64 && rva < self.text_end as i64
    }

    pub fn i32_at(&self, rva: usize) -> Option<i32> {
        self.image
            .get(rva..rva + 4)
            .map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub fn u16_at(&self, rva: usize) -> Option<u16> {
        self.image
            .get(rva..rva + 2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
    }

    fn byte(&self, rva: usize) -> u8 {
        self.image.get(rva).copied().unwrap_or(0)
    }

    /// Everywhere in the code `pattern` matches.
    pub fn find(&self, pattern: &Pattern) -> Vec<usize> {
        let Some(first) = pattern.iter().position(|b| b.is_some()) else {
            return Vec::new();
        };
        let first_byte = pattern[first].unwrap();
        let code = &self.image[..self.text_end];
        let mut found = Vec::new();
        let mut at = self.text_start + first;
        while at + pattern.len() - first <= code.len() {
            match code[at..].iter().position(|b| *b == first_byte) {
                None => break,
                Some(offset) => {
                    let start = at + offset - first;
                    if start + pattern.len() <= code.len()
                        && pattern
                            .iter()
                            .enumerate()
                            .all(|(i, b)| b.is_none_or(|b| code[start + i] == b))
                    {
                        found.push(start);
                    }
                    at += offset + 1;
                }
            }
        }
        found
    }

    /// Every call or jump to `target`.
    pub fn calls_to(&self, target: usize) -> Vec<usize> {
        let calls = self.calls.get_or_init(|| {
            let mut calls: HashMap<usize, Vec<usize>> = HashMap::new();
            for i in self.text_start..self.text_end.saturating_sub(5) {
                let op = self.image[i];
                if (op == 0xE8 || op == 0xE9)
                    && let Some(offset) = self.i32_at(i + 1)
                {
                    let to = i as i64 + 5 + offset as i64;
                    if self.in_text(to) {
                        calls.entry(to as usize).or_default().push(i);
                    }
                }
            }
            calls
        });
        calls.get(&target).cloned().unwrap_or_default()
    }

    /// The start of the function containing `rva`: MSVC pads functions with int3 and aligns them to 16 bytes.
    pub fn function_start(&self, rva: usize) -> usize {
        let mut j = rva;
        while j > self.text_start {
            if j.is_multiple_of(16) && self.byte(j - 1) == 0xCC && self.byte(j) != 0xCC {
                return j;
            }
            j -= 1;
        }
        self.text_start
    }

    /// Whether the function containing `rva` has `value` in it as a 32-bit number (e.g. an opcode it writes.)
    pub fn function_has_u32(&self, rva: usize, value: u32) -> bool {
        let start = self.function_start(rva);
        let end = self.function_end(start);
        let needle = value.to_le_bytes();
        self.image[start..end]
            .windows(4)
            .any(|window| window == needle)
    }

    fn function_end(&self, start: usize) -> usize {
        let limit = (start + 0x80000).min(self.text_end);
        for j in start..limit.saturating_sub(2) {
            if self.byte(j) == 0xCC && self.byte(j + 1) == 0xCC && self.byte(j + 2) == 0xCC {
                return j;
            }
        }
        limit
    }

    fn rex(&self, at: usize) -> usize {
        if (0x40..=0x4F).contains(&self.byte(at)) {
            at + 1
        } else {
            at
        }
    }

    /// The jump tables in the function between `start` and `end`.
    pub fn switches(&self, start: usize, end: usize) -> Vec<Switch> {
        let mut switches = Vec::new();
        let mut p = start;
        while p + 8 < end {
            let at = p;
            p += 1;

            // mov r32, [base + index*4 + table]
            let q = self.rex(at);
            if self.byte(q) != 0x8B {
                continue;
            }
            let modrm = self.byte(q + 1);
            if modrm >> 6 != 2 || modrm & 7 != 4 || self.byte(q + 2) >> 6 != 2 {
                continue;
            }
            let Some(table) = self.i32_at(q + 3) else {
                continue;
            };
            if table <= 0
                || table as usize + 4 >= self.image.len()
                || !self.in_text(self.i32_at(table as usize).unwrap_or(0) as i64)
            {
                continue;
            }
            let table = table as usize;

            // An optional byte table before it: movzx r32, byte [base + index + table]
            let mut byte_table = None;
            let mut b = at.saturating_sub(1);
            while b > at.saturating_sub(24) && b >= start {
                let r = self.rex(b);
                if self.byte(r) == 0x0F
                    && self.byte(r + 1) == 0xB6
                    && self.byte(r + 2) >> 6 == 2
                    && self.byte(r + 2) & 7 == 4
                    && self.byte(r + 3) >> 6 == 0
                    && let Some(t) = self.i32_at(r + 4)
                    && t > 0
                    && (t as usize) < self.image.len()
                {
                    byte_table = Some(t as usize);
                    break;
                }
                b -= 1;
            }

            // The bounds check before it: cmp r32, imm; ja
            let mut limit: i64 = -1;
            let mut cmp_at = 0;
            let mut b = at.saturating_sub(1);
            while b > at.saturating_sub(80) && b >= start {
                if self.byte(b) == 0x3D {
                    limit = self.i32_at(b + 1).unwrap_or(-1) as i64;
                    cmp_at = b;
                    break;
                }
                if (self.byte(b) == 0x81 || self.byte(b) == 0x83)
                    && self.byte(b + 1) >> 6 == 3
                    && (self.byte(b + 1) >> 3) & 7 == 7
                {
                    limit = if self.byte(b) == 0x81 {
                        self.i32_at(b + 2).unwrap_or(-1) as i64
                    } else {
                        self.byte(b + 2) as i8 as i64
                    };
                    cmp_at = b;
                    break;
                }
                b -= 1;
            }
            if !(0..=4096).contains(&limit) {
                continue;
            }

            // How the opcode was shifted to make the index: lea r32, [reg - x], sub r32, x or add r32, x
            let mut bias: i64 = 0;
            let mut b = cmp_at.saturating_sub(1);
            while b > cmp_at.saturating_sub(48) && b >= start {
                let r = self.rex(b);
                let (op, modrm) = (self.byte(r), self.byte(r + 1));
                if op == 0x8D && modrm >> 6 == 1 && modrm & 7 != 4 {
                    bias = -(self.byte(r + 2) as i8 as i64);
                    break;
                }
                if op == 0x8D && modrm >> 6 == 2 && modrm & 7 != 4 {
                    bias = -(self.i32_at(r + 2).unwrap_or(0) as i64);
                    break;
                }
                if op == 0x2D {
                    bias = self.i32_at(r + 1).unwrap_or(0) as i64;
                    break;
                }
                if op == 0x05 {
                    bias = -(self.i32_at(r + 1).unwrap_or(0) as i64);
                    break;
                }
                if op == 0x83 && modrm >> 6 == 3 {
                    let ext = (modrm >> 3) & 7;
                    if ext == 5 {
                        bias = self.byte(r + 2) as i8 as i64;
                        break;
                    }
                    if ext == 0 {
                        bias = -(self.byte(r + 2) as i8 as i64);
                        break;
                    }
                }
                if op == 0x81 && modrm >> 6 == 3 {
                    let ext = (modrm >> 3) & 7;
                    if ext == 5 {
                        bias = self.i32_at(r + 2).unwrap_or(0) as i64;
                        break;
                    }
                    if ext == 0 {
                        bias = -(self.i32_at(r + 2).unwrap_or(0) as i64);
                        break;
                    }
                }
                b -= 1;
            }

            let mut cases: BTreeMap<usize, Vec<i64>> = BTreeMap::new();
            for i in 0..=limit {
                let slot = match byte_table {
                    Some(bt) => self.byte(bt + i as usize) as usize,
                    None => i as usize,
                };
                let Some(target) = self.i32_at(table + slot * 4) else {
                    break;
                };
                if !self.in_text(target as i64) {
                    break;
                }
                cases.entry(target as usize).or_default().push(i);
            }
            if cases.len() > 1 {
                switches.push(Switch { at, bias, cases });
            }
        }
        switches
    }

    /// The opcodes whose case (in a switch of the function at `function`) contains `site`, with that switch's size.
    fn opcodes_at(&self, site: usize, function: usize) -> Vec<(usize, Vec<i64>)> {
        let mut result = Vec::new();
        for switch in self.switches(function, self.function_end(function)) {
            let Some((&case, indices)) = switch.cases.range(..=site).next_back() else {
                continue;
            };
            if site - case > 0x300 {
                continue;
            }
            let opcodes = indices.iter().map(|i| switch.bias + i).collect();
            result.push((switch.cases.len(), opcodes));
        }
        result
    }

    /// The function a switch case calls first, e.g. a packet's handler from its case in the dispatcher.
    pub fn first_call_in_case(&self, case: usize) -> Option<usize> {
        let mut at = case;
        while at < case + 96 && at + 5 <= self.text_end {
            match self.byte(at) {
                0xE8 => {
                    let to = at as i64 + 5 + self.i32_at(at + 1)? as i64;
                    return self.in_text(to).then_some(to as usize);
                }
                // The case ends: ret, or a jump on.
                0xC3 | 0xE9 | 0xEB => return None,
                _ => at += 1,
            }
        }
        None
    }

    /// Follows calls back from the function at `handler` (up to `depth` levels) to the switch cases that lead to it.
    pub fn opcode_cases(&self, handler: usize, depth: usize) -> Vec<OpcodeCase> {
        let mut cases = Vec::new();
        let mut seen = HashSet::new();
        let mut level = vec![handler];
        for d in 0..depth {
            if level.is_empty() {
                break;
            }
            let mut next = Vec::new();
            for target in level {
                for site in self.calls_to(target) {
                    let function = self.function_start(site);
                    let found = self.opcodes_at(site, function);
                    if found.is_empty() && seen.insert(function) {
                        next.push(function);
                    }
                    for (switch_cases, mut opcodes) in found {
                        opcodes.sort();
                        opcodes.dedup();
                        cases.push(OpcodeCase {
                            depth: d,
                            site,
                            function,
                            switch_cases,
                            opcodes,
                        });
                    }
                }
            }
            level = next;
        }
        cases
    }
}

/// How deep to follow calls back from a handler to the switch over opcodes.
const HANDLER_SEARCH_DEPTH: usize = 4;

/// Tries `opcode`'s finders in order. Only a finder that reads exactly one opcode finds it.
pub fn find_opcode(exe: &Exe, opcode: &crate::opcodes::ForkOpcode) -> Result<u16, String> {
    let mut errors = Vec::new();
    for finder in opcode.finders {
        match finder_candidates(exe, finder) {
            Ok(candidates) => match candidates.as_slice() {
                [read] if crate::opcodes::is_known_opcode(opcode.kind, *read) && Some(*read) != opcode.opcode => {
                    errors.push(format!("it read {read}, but that's already another packet's"))
                }
                [read] => return Ok(*read),
                _ => errors.push(format!(
                    "it's one of {candidates:?} (they share their code), so a recording has to say which"
                )),
            },
            Err(err) => errors.push(err),
        }
    }
    if errors.is_empty() {
        errors.push("it has no finders".to_string());
    }
    Err(errors.join("; "))
}

/// Checks a known opcode against its finders: Ok if a finder's candidates include it.
pub fn check_opcode(
    exe: &Exe,
    opcode: &crate::opcodes::ForkOpcode,
    known: u16,
) -> Result<(), String> {
    let mut errors = Vec::new();
    for finder in opcode.finders {
        match finder_candidates(exe, finder) {
            Ok(candidates) if candidates.contains(&known) => return Ok(()),
            Ok(candidates) => errors.push(format!("it reads {candidates:?}")),
            Err(err) => errors.push(err),
        }
    }
    Err(errors.join("; "))
}

/// The opcodes a finder reads (more than one when packets share their code, e.g. one handler for several sizes.)
pub fn finder_candidates(exe: &Exe, finder: &Finder) -> Result<Vec<u16>, String> {
    let to_u16 = |values: Vec<i64>| -> Result<Vec<u16>, String> {
        values
            .into_iter()
            .map(|value| u16::try_from(value).map_err(|_| format!("{value} isn't an opcode")))
            .collect()
    };
    match *finder {
        Finder::Immediate {
            pattern,
            offset,
            width,
            with_opcode,
        } => {
            let parsed = parse_pattern(pattern).ok_or(format!("bad pattern {pattern:?}"))?;
            let mut matches = exe.find(&parsed);
            if matches.is_empty() {
                return Err(format!("{pattern:?} isn't in the game anymore"));
            }
            if let Some(with_opcode) = with_opcode {
                matches.retain(|at| exe.function_has_u32(*at, with_opcode as u32));
                if matches.is_empty() {
                    return Err(format!(
                        "{pattern:?} is in the game, but not alongside opcode {with_opcode}"
                    ));
                }
            }
            let mut values: Vec<i64> = matches
                .iter()
                .filter_map(|at| match width {
                    2 => exe.u16_at(at + offset).map(i64::from),
                    _ => exe.i32_at(at + offset).map(i64::from),
                })
                .collect();
            values.sort();
            values.dedup();
            to_u16(values)
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
            to_u16(opcodes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patterns() {
        assert_eq!(
            parse_pattern("48 8B ?? 05"),
            Some(vec![Some(0x48), Some(0x8B), None, Some(5)])
        );
        assert_eq!(parse_pattern("48 zz"), None);

        let exe = Exe::from_code(
            &[0x90, 0x48, 0x8B, 0x11, 0x05, 0x48, 0x8B, 0x22, 0x05],
            0x1000,
        );
        assert_eq!(
            exe.find(&parse_pattern("48 8B ? 05").unwrap()),
            vec![0x1001, 0x1005]
        );
        assert_eq!(exe.find(&parse_pattern("8B 22").unwrap()), vec![0x1006]);
    }

    /// A tiny dispatcher: `switch (opcode - 100)` over cases 100..=102, where 101 calls a handler.
    fn dispatcher() -> (Exe, usize) {
        let base = 0x1000;
        let mut code = vec![0xCCu8; 16];
        // The dispatcher, at 0x1010:
        let dispatcher = base + code.len();
        code.extend_from_slice(&[0x8D, 0x41, 0x9C]); // lea eax, [rcx - 100]
        code.extend_from_slice(&[0x83, 0xF8, 0x02]); // cmp eax, 2
        code.extend_from_slice(&[0x77, 0x20]); // ja default
        let table_mov = code.len();
        code.extend_from_slice(&[0x8B, 0x84, 0x82, 0, 0, 0, 0]); // mov eax, [rdx + rax*4 + table]
        code.extend_from_slice(&[0xFF, 0xE0]); // jmp rax
        let case = |code: &Vec<u8>| base + code.len();
        let case100 = case(&code);
        code.extend_from_slice(&[0xC3]);
        let case101 = case(&code);
        let call_at = code.len();
        code.extend_from_slice(&[0xE8, 0, 0, 0, 0, 0xC3]); // call handler; ret
        let case102 = case(&code);
        code.extend_from_slice(&[0xC3]);
        while !code.len().is_multiple_of(16) {
            code.push(0xCC);
        }
        code.extend_from_slice(&[0xCC; 16]);
        // The handler.
        let handler = base + code.len();
        code.extend_from_slice(&[0x90, 0xC3]);
        code.extend_from_slice(&[0xCC; 16]);
        // The jump table.
        let table = base + code.len();
        for target in [case100, case101, case102] {
            code.extend_from_slice(&(target as u32).to_le_bytes());
        }

        code[table_mov + 3..table_mov + 7].copy_from_slice(&(table as u32).to_le_bytes());
        let call_rva = base + call_at;
        let offset = handler as i64 - (call_rva as i64 + 5);
        code[call_at + 1..call_at + 5].copy_from_slice(&(offset as i32).to_le_bytes());

        assert_eq!(dispatcher, 0x1010);
        (Exe::from_code(&code, base), handler)
    }

    #[test]
    fn reads_the_opcode_from_its_switch() {
        let (exe, handler) = dispatcher();
        let cases = exe.opcode_cases(handler, 3);
        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0].opcodes, vec![101]);
        assert_eq!(cases[0].switch_cases, 3);
        assert_eq!(cases[0].function, 0x1010);
    }
}
