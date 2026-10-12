//! A small Lua 5.1 bytecode reader and disassembler, for reading the client's compiled event scripts (`.luab`.)
//!
//! This isn't a decompiler. It names every function it can (e.g. `XxaFst034_03854:OnScene00000`), and lists the
//! methods, globals and constants each one uses, followed by an annotated instruction listing. That's enough to see
//! which scenes exist, what they talk to and which quest variables they touch.

use std::fmt::Write;

/// A constant from a function's constant table.
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constant::Nil => write!(f, "nil"),
            Constant::Boolean(value) => write!(f, "{value}"),
            Constant::Number(value) => {
                if value.fract() == 0.0 && value.abs() < 1e15 {
                    write!(f, "{}", *value as i64)
                } else {
                    write!(f, "{value}")
                }
            }
            Constant::String(value) => write!(f, "{value:?}"),
        }
    }
}

/// A compiled Lua function (prototype.)
#[derive(Debug, Clone, Default)]
pub struct Function {
    pub source: Option<String>,
    pub line_defined: u64,
    pub num_upvalues: u8,
    pub num_params: u8,
    pub is_vararg: u8,
    pub max_stack_size: u8,
    pub code: Vec<u32>,
    pub constants: Vec<Constant>,
    pub protos: Vec<Function>,
    pub locals: Vec<String>,
    pub upvalue_names: Vec<String>,
}

/// The header of a compiled chunk, describing the sizes of the types inside.
#[derive(Debug, Clone, Copy)]
pub struct Header {
    pub version: u8,
    pub little_endian: bool,
    pub int_size: u8,
    pub size_t_size: u8,
    pub instruction_size: u8,
    pub number_size: u8,
    pub integral_numbers: bool,
}

#[derive(Debug)]
pub struct Chunk {
    pub header: Header,
    pub main: Function,
}

const SIGNATURE: &[u8; 4] = b"\x1bLua";

struct Reader<'a> {
    data: &'a [u8],
    position: usize,
    header: Option<Header>,
}

impl<'a> Reader<'a> {
    fn bytes(&mut self, count: usize) -> Result<&'a [u8], String> {
        if self.position + count > self.data.len() {
            return Err(format!(
                "unexpected end of file at offset {} (wanted {count} more bytes)",
                self.position
            ));
        }
        let slice = &self.data[self.position..self.position + count];
        self.position += count;
        Ok(slice)
    }

    fn byte(&mut self) -> Result<u8, String> {
        Ok(self.bytes(1)?[0])
    }

    fn unsigned(&mut self, size: u8) -> Result<u64, String> {
        let header = self.header.unwrap();
        let bytes = self.bytes(size as usize)?;
        let mut value = 0u64;
        if header.little_endian {
            for (i, byte) in bytes.iter().enumerate() {
                value |= (*byte as u64) << (8 * i);
            }
        } else {
            for byte in bytes {
                value = (value << 8) | *byte as u64;
            }
        }
        Ok(value)
    }

    fn int(&mut self) -> Result<u64, String> {
        self.unsigned(self.header.unwrap().int_size)
    }

    fn size_t(&mut self) -> Result<u64, String> {
        self.unsigned(self.header.unwrap().size_t_size)
    }

    fn count(&mut self) -> Result<usize, String> {
        let count = self.int()? as usize;
        // Guard against garbage making us allocate the world.
        if count > self.data.len() {
            return Err(format!(
                "implausible count {count} at offset {}, is this really Lua 5.1 bytecode?",
                self.position
            ));
        }
        Ok(count)
    }

    fn string(&mut self) -> Result<Option<String>, String> {
        let length = self.size_t()? as usize;
        if length == 0 {
            return Ok(None);
        }
        let bytes = self.bytes(length)?;
        // Strings include their trailing NUL.
        let bytes = bytes.strip_suffix(&[0]).unwrap_or(bytes);
        Ok(Some(String::from_utf8_lossy(bytes).into_owned()))
    }

    fn number(&mut self) -> Result<f64, String> {
        let header = self.header.unwrap();
        let raw = self.unsigned(header.number_size)?;
        Ok(if header.integral_numbers {
            // Sign-extend
            let shift = 64 - 8 * header.number_size as u32;
            ((raw << shift) as i64 >> shift) as f64
        } else if header.number_size == 8 {
            f64::from_bits(raw)
        } else if header.number_size == 4 {
            f32::from_bits(raw as u32) as f64
        } else {
            return Err(format!(
                "unsupported lua_Number size {}",
                header.number_size
            ));
        })
    }

    fn function(&mut self, parent_source: Option<&str>) -> Result<Function, String> {
        let source = self.string()?.or(parent_source.map(str::to_string));
        let line_defined = self.int()?;
        let _last_line_defined = self.int()?;
        let num_upvalues = self.byte()?;
        let num_params = self.byte()?;
        let is_vararg = self.byte()?;
        let max_stack_size = self.byte()?;

        let code_count = self.count()?;
        let mut code = Vec::with_capacity(code_count);
        for _ in 0..code_count {
            code.push(self.unsigned(self.header.unwrap().instruction_size)? as u32);
        }

        let constant_count = self.count()?;
        let mut constants = Vec::with_capacity(constant_count);
        for _ in 0..constant_count {
            constants.push(match self.byte()? {
                0 => Constant::Nil,
                1 => Constant::Boolean(self.byte()? != 0),
                3 => Constant::Number(self.number()?),
                4 => Constant::String(self.string()?.unwrap_or_default()),
                other => {
                    return Err(format!(
                        "unknown constant type {other} at offset {}",
                        self.position - 1
                    ));
                }
            });
        }

        let proto_count = self.count()?;
        let mut protos = Vec::with_capacity(proto_count);
        for _ in 0..proto_count {
            protos.push(self.function(source.as_deref())?);
        }

        // Debug information (may be stripped, in which case these are all empty)
        let line_count = self.count()?;
        for _ in 0..line_count {
            self.int()?;
        }
        let local_count = self.count()?;
        let mut locals = Vec::with_capacity(local_count);
        for _ in 0..local_count {
            locals.push(self.string()?.unwrap_or_default());
            self.int()?;
            self.int()?;
        }
        let upvalue_count = self.count()?;
        let mut upvalue_names = Vec::with_capacity(upvalue_count);
        for _ in 0..upvalue_count {
            upvalue_names.push(self.string()?.unwrap_or_default());
        }

        Ok(Function {
            source,
            line_defined,
            num_upvalues,
            num_params,
            is_vararg,
            max_stack_size,
            code,
            constants,
            protos,
            locals,
            upvalue_names,
        })
    }
}

/// Parses a compiled Lua 5.1 chunk.
pub fn parse(data: &[u8]) -> Result<Chunk, String> {
    if data.len() < 12 || &data[0..4] != SIGNATURE {
        return Err(format!(
            "not Lua bytecode (expected the signature 1B 4C 75 61, found {:02X?})",
            &data[..data.len().min(4)]
        ));
    }

    let header = Header {
        version: data[4],
        little_endian: data[6] == 1,
        int_size: data[7],
        size_t_size: data[8],
        instruction_size: data[9],
        number_size: data[10],
        integral_numbers: data[11] != 0,
    };
    if header.version != 0x51 {
        return Err(format!(
            "this is Lua bytecode version {:X}.{:X}, only 5.1 is supported",
            header.version >> 4,
            header.version & 0xF
        ));
    }
    if data[5] != 0 {
        return Err(format!("unsupported bytecode format {}", data[5]));
    }
    for (name, size) in [
        ("int", header.int_size),
        ("size_t", header.size_t_size),
        ("Instruction", header.instruction_size),
    ] {
        if !(1..=8).contains(&size) {
            return Err(format!("unsupported sizeof({name}) of {size}"));
        }
    }

    let mut reader = Reader {
        data,
        position: 12,
        header: Some(header),
    };
    let main = reader.function(None)?;
    Ok(Chunk { header, main })
}

const OPCODES: [&str; 38] = [
    "MOVE",
    "LOADK",
    "LOADBOOL",
    "LOADNIL",
    "GETUPVAL",
    "GETGLOBAL",
    "GETTABLE",
    "SETGLOBAL",
    "SETUPVAL",
    "SETTABLE",
    "NEWTABLE",
    "SELF",
    "ADD",
    "SUB",
    "MUL",
    "DIV",
    "MOD",
    "POW",
    "UNM",
    "NOT",
    "LEN",
    "CONCAT",
    "JMP",
    "EQ",
    "LT",
    "LE",
    "TEST",
    "TESTSET",
    "CALL",
    "TAILCALL",
    "RETURN",
    "FORLOOP",
    "FORPREP",
    "TFORLOOP",
    "SETLIST",
    "CLOSE",
    "CLOSURE",
    "VARARG",
];

const OP_MOVE: u32 = 0;
const OP_GETGLOBAL: u32 = 5;
const OP_GETTABLE: u32 = 6;
const OP_SETGLOBAL: u32 = 7;
const OP_SETTABLE: u32 = 9;
const OP_SELF: u32 = 11;
const OP_CLOSURE: u32 = 36;

/// Constants are referenced by setting this bit in B or C.
const BIT_RK: u32 = 1 << 8;
const MAX_ARG_SBX: i64 = (1 << 17) - 1;

/// A decoded instruction.
#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub opcode: u32,
    pub a: u32,
    pub b: u32,
    pub c: u32,
    pub bx: u32,
    pub sbx: i64,
}

impl Instruction {
    pub fn decode(raw: u32) -> Self {
        let bx = raw >> 14;
        Self {
            opcode: raw & 0x3F,
            a: (raw >> 6) & 0xFF,
            c: (raw >> 14) & 0x1FF,
            b: (raw >> 23) & 0x1FF,
            bx,
            sbx: bx as i64 - MAX_ARG_SBX,
        }
    }

    pub fn name(&self) -> &'static str {
        OPCODES.get(self.opcode as usize).copied().unwrap_or("???")
    }
}

/// Describes a register-or-constant operand.
fn rk(function: &Function, value: u32) -> String {
    if value & BIT_RK != 0 {
        match function.constants.get((value & !BIT_RK) as usize) {
            Some(constant) => constant.to_string(),
            None => format!("K{}", value & !BIT_RK),
        }
    } else {
        format!("R{value}")
    }
}

fn constant_string(function: &Function, index: u32) -> Option<&str> {
    match function.constants.get(index as usize) {
        Some(Constant::String(value)) => Some(value),
        _ => None,
    }
}

fn rk_string(function: &Function, value: u32) -> Option<&str> {
    if value & BIT_RK != 0 {
        constant_string(function, value & !BIT_RK)
    } else {
        None
    }
}

/// Works out the names of a function's child prototypes from how they're assigned
/// (e.g. `function Quest:OnScene00000()` becomes `Quest:OnScene00000`.)
fn name_children(function: &Function) -> Vec<Option<String>> {
    let mut names = vec![None; function.protos.len()];
    // What each register currently holds, as far as we can tell.
    let mut closure_in: Vec<Option<usize>> = vec![None; 256];
    let mut global_in: Vec<Option<String>> = vec![None; 256];

    for raw in &function.code {
        let instruction = Instruction::decode(*raw);
        let a = instruction.a as usize;
        match instruction.opcode {
            OP_CLOSURE => {
                closure_in[a] = Some(instruction.bx as usize);
                global_in[a] = None;
            }
            OP_GETGLOBAL => {
                global_in[a] = constant_string(function, instruction.bx).map(str::to_string);
                closure_in[a] = None;
            }
            OP_MOVE => {
                let b = instruction.b as usize;
                closure_in[a] = closure_in[b];
                global_in[a] = global_in[b].clone();
            }
            OP_SETGLOBAL => {
                if let Some(proto) = closure_in[a]
                    && let Some(name) = constant_string(function, instruction.bx)
                    && let Some(slot) = names.get_mut(proto)
                {
                    *slot = Some(name.to_string());
                }
            }
            OP_SETTABLE => {
                let c = instruction.c;
                if c & BIT_RK == 0
                    && let Some(proto) = closure_in[c as usize]
                    && let Some(key) = rk_string(function, instruction.b)
                    && let Some(slot) = names.get_mut(proto)
                {
                    *slot = Some(match &global_in[a] {
                        Some(table) => format!("{table}:{key}"),
                        None => key.to_string(),
                    });
                }
            }
            _ => {
                // Anything else writing to A invalidates what we knew about it.
                if instruction.opcode != OP_SETTABLE {
                    closure_in[a] = None;
                    global_in[a] = None;
                }
            }
        }
    }

    names
}

/// The interesting things a function refers to.
#[derive(Debug, Default)]
pub struct Summary {
    /// Methods called with `:` (e.g. `GetQuestUI8AL`, `PlayCutScene`.)
    pub methods: Vec<String>,
    /// Globals read.
    pub globals: Vec<String>,
    /// Fields read with a constant key (e.g. `self.ACTOR0`.)
    pub fields: Vec<String>,
    /// Numeric constants.
    pub numbers: Vec<String>,
    /// Other string constants.
    pub strings: Vec<String>,
}

fn push_unique(list: &mut Vec<String>, value: String) {
    if !list.contains(&value) {
        list.push(value);
    }
}

pub fn summarize(function: &Function) -> Summary {
    let mut summary = Summary::default();
    let mut used_names = Vec::new();

    for raw in &function.code {
        let instruction = Instruction::decode(*raw);
        match instruction.opcode {
            OP_SELF => {
                if let Some(name) = rk_string(function, instruction.c) {
                    push_unique(&mut summary.methods, name.to_string());
                    used_names.push(name.to_string());
                }
            }
            OP_GETGLOBAL => {
                if let Some(name) = constant_string(function, instruction.bx) {
                    push_unique(&mut summary.globals, name.to_string());
                    used_names.push(name.to_string());
                }
            }
            OP_GETTABLE => {
                if let Some(name) = rk_string(function, instruction.c) {
                    push_unique(&mut summary.fields, name.to_string());
                    used_names.push(name.to_string());
                }
            }
            _ => {}
        }
    }

    for constant in &function.constants {
        match constant {
            Constant::Number(_) => push_unique(&mut summary.numbers, constant.to_string()),
            Constant::String(value) if !used_names.contains(value) => {
                push_unique(&mut summary.strings, value.clone())
            }
            _ => {}
        }
    }

    summary
}

/// Annotated listing of a single instruction.
pub fn describe(function: &Function, instruction: &Instruction, pc: usize) -> String {
    let (a, b, c) = (instruction.a, instruction.b, instruction.c);
    let constant = |index: u32| {
        function
            .constants
            .get(index as usize)
            .map(|k| k.to_string())
            .unwrap_or_else(|| format!("K{index}"))
    };
    match instruction.opcode {
        0 => format!("R{a} = R{b}"),
        1 => format!("R{a} = {}", constant(instruction.bx)),
        2 => format!(
            "R{a} = {}{}",
            b != 0,
            if c != 0 { "; skip next" } else { "" }
        ),
        3 => format!("R{a}..R{b} = nil"),
        4 => format!(
            "R{a} = upvalue {}",
            function
                .upvalue_names
                .get(b as usize)
                .cloned()
                .unwrap_or_else(|| format!("U{b}"))
        ),
        5 => format!("R{a} = {}", constant(instruction.bx).trim_matches('"')),
        6 => format!("R{a} = R{b}[{}]", rk(function, c)),
        7 => format!("{} = R{a}", constant(instruction.bx).trim_matches('"')),
        8 => format!("upvalue U{b} = R{a}"),
        9 => format!("R{a}[{}] = {}", rk(function, b), rk(function, c)),
        10 => format!("R{a} = {{}}"),
        11 => format!(
            "R{} = R{b}; R{a} = R{b}:{}",
            a + 1,
            rk(function, c).trim_matches('"')
        ),
        12..=17 => {
            let operator = ["+", "-", "*", "/", "%", "^"][(instruction.opcode - 12) as usize];
            format!("R{a} = {} {operator} {}", rk(function, b), rk(function, c))
        }
        18 => format!("R{a} = -R{b}"),
        19 => format!("R{a} = not R{b}"),
        20 => format!("R{a} = #R{b}"),
        21 => format!("R{a} = R{b} .. ... .. R{c}"),
        22 => format!("jump to {}", pc as i64 + 1 + instruction.sbx + 1),
        23..=25 => {
            let operator = ["==", "<", "<="][(instruction.opcode - 23) as usize];
            format!(
                "if ({} {operator} {}) ~= {} then skip next",
                rk(function, b),
                rk(function, c),
                a != 0
            )
        }
        26 => format!("if not (R{a} <=> {c}) then skip next"),
        27 => format!("if R{b} <=> {c} then R{a} = R{b} else skip next"),
        28 | 29 => {
            let args = if b == 0 {
                "...".to_string()
            } else {
                format!("{} args", b - 1)
            };
            let results = if c == 0 {
                "...".to_string()
            } else {
                format!("{} results", c as i64 - 1)
            };
            if instruction.opcode == 29 {
                format!("return R{a}({args})")
            } else {
                format!("R{a}({args}) -> {results}")
            }
        }
        30 => {
            if b == 0 {
                format!("return R{a}...")
            } else if b == 1 {
                "return".to_string()
            } else {
                format!("return R{a}..R{}", a + b - 2)
            }
        }
        31 => format!(
            "for loop R{a}, jump to {}",
            pc as i64 + 1 + instruction.sbx + 1
        ),
        32 => format!(
            "for prep R{a}, jump to {}",
            pc as i64 + 1 + instruction.sbx + 1
        ),
        33 => format!("generic for R{a}, {c} results"),
        34 => format!("R{a}[...] = R{}..R{}", a + 1, a + b),
        35 => format!("close upvalues >= R{a}"),
        36 => format!("R{a} = closure of function {}", instruction.bx),
        37 => format!("R{a}.. = ..."),
        _ => format!("A={a} B={b} C={c}"),
    }
}

fn disassemble_function(
    out: &mut String,
    function: &Function,
    name: &str,
    path: &str,
    listing: bool,
) {
    let _ = writeln!(
        out,
        "\n== {name}  [function {path}, {} params{}, {} upvalues, stack {}, {} instructions{}]",
        function.num_params,
        if function.is_vararg != 0 {
            " + ..."
        } else {
            ""
        },
        function.num_upvalues,
        function.max_stack_size,
        function.code.len(),
        match (&function.source, function.line_defined) {
            (Some(source), line) if line > 0 => format!(", {source}:{line}"),
            _ => String::new(),
        }
    );
    if !function.locals.is_empty() {
        let _ = writeln!(out, "   locals   {}", function.locals.join(", "));
    }

    let summary = summarize(function);
    for (label, list) in [
        ("methods", &summary.methods),
        ("globals", &summary.globals),
        ("fields", &summary.fields),
        ("numbers", &summary.numbers),
        ("strings", &summary.strings),
    ] {
        if !list.is_empty() {
            let _ = writeln!(out, "   {label:8} {}", list.join(", "));
        }
    }

    if listing {
        for (pc, raw) in function.code.iter().enumerate() {
            let instruction = Instruction::decode(*raw);
            let _ = writeln!(
                out,
                "   {:>4}  {:<10} {}",
                pc + 1,
                instruction.name(),
                describe(function, &instruction, pc)
            );
        }
    }

    let child_names = name_children(function);
    for (index, child) in function.protos.iter().enumerate() {
        let child_name = child_names[index]
            .clone()
            .unwrap_or_else(|| format!("{name}/<anonymous {index}>"));
        disassemble_function(out, child, &child_name, &format!("{path}.{index}"), listing);
    }
}

/// Returns the names of every function in the chunk that looks like a scene (`OnScene00000` etc.)
pub fn scene_functions(chunk: &Chunk) -> Vec<String> {
    fn walk(function: &Function, found: &mut Vec<String>) {
        for (index, name) in name_children(function).into_iter().enumerate() {
            if let Some(name) = name
                && name.contains("OnScene")
            {
                found.push(name);
            }
            walk(&function.protos[index], found);
        }
    }
    let mut found = Vec::new();
    walk(&chunk.main, &mut found);
    found
}

/// Produces a readable listing of a compiled chunk.
pub fn disassemble(chunk: &Chunk, listing: bool) -> String {
    let mut out = String::new();
    let header = chunk.header;
    let _ = writeln!(
        out,
        "; Lua {}.{} bytecode, {} endian, int={} size_t={} number={}{}",
        header.version >> 4,
        header.version & 0xF,
        if header.little_endian {
            "little"
        } else {
            "big"
        },
        header.int_size,
        header.size_t_size,
        header.number_size,
        if header.integral_numbers {
            " (integral)"
        } else {
            ""
        }
    );

    let scenes = scene_functions(chunk);
    if !scenes.is_empty() {
        let _ = writeln!(out, "; scenes: {}", scenes.join(", "));
    }

    disassemble_function(&mut out, &chunk.main, "<main>", "0", listing);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a tiny chunk by hand: `Quest = {}; function Quest:OnScene00001(a) a:Talk(1000100) end`
    /// laid out like luac 5.1 would on a 64-bit little-endian machine.
    fn sample_chunk() -> Vec<u8> {
        fn int(out: &mut Vec<u8>, value: u32) {
            out.extend_from_slice(&value.to_le_bytes());
        }
        fn size_t(out: &mut Vec<u8>, value: u64) {
            out.extend_from_slice(&value.to_le_bytes());
        }
        fn string(out: &mut Vec<u8>, value: &str) {
            size_t(out, value.len() as u64 + 1);
            out.extend_from_slice(value.as_bytes());
            out.push(0);
        }
        fn instruction(op: u32, a: u32, b: u32, c: u32) -> u32 {
            op | (a << 6) | (c << 14) | (b << 23)
        }
        fn instruction_bx(op: u32, a: u32, bx: u32) -> u32 {
            op | (a << 6) | (bx << 14)
        }
        fn no_debug(out: &mut Vec<u8>) {
            int(out, 0);
            int(out, 0);
            int(out, 0);
        }

        let mut out = b"\x1bLua\x51\x00\x01\x04\x08\x04\x08\x00".to_vec();

        // main
        string(&mut out, "@quest.lua");
        int(&mut out, 0);
        int(&mut out, 0);
        out.extend_from_slice(&[0, 0, 2, 2]);
        let code = [
            instruction(10, 0, 0, 0),         // R0 = {}
            instruction_bx(7, 0, 0),          // Quest = R0
            instruction_bx(5, 0, 0),          // R0 = Quest
            instruction_bx(36, 1, 0),         // R1 = closure 0
            instruction(9, 0, BIT_RK | 1, 1), // R0["OnScene00001"] = R1
            instruction(30, 0, 1, 0),         // return
        ];
        int(&mut out, code.len() as u32);
        for op in code {
            int(&mut out, op);
        }
        int(&mut out, 2);
        out.push(4);
        string(&mut out, "Quest");
        out.push(4);
        string(&mut out, "OnScene00001");

        // child: function(self, a) a:Talk(1000100) end
        int(&mut out, 1);
        size_t(&mut out, 0);
        int(&mut out, 1);
        int(&mut out, 1);
        out.extend_from_slice(&[0, 2, 0, 4]);
        let code = [
            instruction(11, 2, 1, BIT_RK), // R3 = R1; R2 = R1:Talk
            instruction_bx(1, 4, 1),       // R4 = 1000100
            instruction(28, 2, 3, 1),      // R2(R3, R4)
            instruction(30, 0, 1, 0),      // return
        ];
        int(&mut out, code.len() as u32);
        for op in code {
            int(&mut out, op);
        }
        int(&mut out, 2);
        out.push(4);
        string(&mut out, "Talk");
        out.push(3);
        out.extend_from_slice(&1000100f64.to_le_bytes());
        int(&mut out, 0); // no protos
        no_debug(&mut out);

        no_debug(&mut out);
        out
    }

    #[test]
    fn reads_hand_built_chunk() {
        let chunk = parse(&sample_chunk()).unwrap();
        assert_eq!(chunk.main.protos.len(), 1);
        assert_eq!(scene_functions(&chunk), vec!["Quest:OnScene00001"]);

        let summary = summarize(&chunk.main.protos[0]);
        assert_eq!(summary.methods, vec!["Talk"]);
        assert_eq!(summary.numbers, vec!["1000100"]);

        let listing = disassemble(&chunk, true);
        assert!(listing.contains("== Quest:OnScene00001"));
        assert!(listing.contains("R1:Talk"));
    }

    /// `tests/fixtures/sample.lua`, compiled with luac 5.1 (with and without debug info.)
    #[test]
    fn reads_real_luac_output() {
        for file in ["sample_stripped.luab", "sample_debug.luab"] {
            let path = format!("{}/tests/fixtures/{file}", env!("CARGO_MANIFEST_DIR"));
            let chunk = parse(&std::fs::read(path).unwrap()).unwrap();
            assert_eq!(
                scene_functions(&chunk),
                vec![
                    "SubTst001_00001:OnScene00000",
                    "SubTst001_00001:OnScene00001",
                    "SubTst001_00001:OnScene00100"
                ]
            );

            let scene1 = summarize(&chunk.main.protos[1]);
            assert_eq!(
                scene1.methods,
                vec!["GetQuestUI8AL", "SetQuestUI8AL", "PlayCutScene", "Wait"]
            );
            assert!(scene1.numbers.contains(&"62".to_string()));
            assert!(scene1.numbers.contains(&"0.5".to_string()));

            // Nested functions are listed too.
            assert_eq!(chunk.main.protos[2].protos.len(), 1);
        }
    }

    #[test]
    fn rejects_non_lua() {
        assert!(parse(b"not lua at all").is_err());
        assert!(parse(b"\x1bLua\x52\x00\x01\x04\x08\x04\x08\x00").is_err());
    }
}
