//! Names from a ClientStructs checkout (FFXIVClientStructs, read at run time, never stored): which known function a
//! packet's data is handed to (its `MemberFunction` signature matching at that address), and the fields of the packet
//! structs ClientStructs describes (`[FieldOffset]`).

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;

use iced_x86::Register;
use kawari::exe_scan::{Exe, Pattern, parse_pattern};

struct Function {
    name: String,
    pattern: Pattern,
    is_static: bool,
    /// Parameter types, in order (without `this`.)
    parameters: Vec<String>,
}

pub struct ClientStructs {
    functions: Vec<Function>,
    /// Struct name → (offset, field name, field type), by offset.
    structs: HashMap<String, Vec<(i64, String, String)>>,
    matched: RefCell<HashMap<usize, Option<usize>>>,
}

impl ClientStructs {
    /// Reads every .cs file under `root`.
    pub fn read(root: &Path) -> Result<Self, String> {
        let mut files = Vec::new();
        collect(root, &mut files);
        if files.is_empty() {
            return Err(format!("no .cs files under {}", root.display()));
        }
        let mut functions = Vec::new();
        let mut structs: HashMap<String, Vec<(i64, String, String)>> = HashMap::new();
        for file in files {
            let Ok(text) = std::fs::read_to_string(&file) else {
                continue;
            };
            let mut current = String::new();
            let mut pending: Option<String> = None;
            for line in text.lines() {
                let line = line.trim();
                if let Some(at) = line.find("struct ")
                    && (line.contains("public") || line.contains("internal"))
                    && !line.contains('(')
                    && !line.ends_with(';')
                {
                    current = line[at + 7..]
                        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                        .next()
                        .unwrap_or("")
                        .to_string();
                }
                if let Some(rest) = line.strip_prefix("[MemberFunction(\"") {
                    pending = rest.split('"').next().map(str::to_string);
                    continue;
                }
                if let Some(pattern) = pending.take() {
                    if line.starts_with('[') {
                        pending = Some(pattern); // another attribute in between
                        continue;
                    }
                    if let Some(open) = line.find('(') {
                        let head = &line[..open];
                        let name = head.split_whitespace().last().unwrap_or("").to_string();
                        let params_text = line[open + 1..].split(')').next().unwrap_or("");
                        let parameters = params_text
                            .split(',')
                            .filter(|p| !p.trim().is_empty())
                            .map(|p| {
                                let mut words: Vec<&str> = p.split_whitespace().collect();
                                words.pop(); // the parameter's name
                                words.last().copied().unwrap_or("").to_string()
                            })
                            .collect();
                        if let Some(pattern) = parse_pattern(&pattern) {
                            functions.push(Function {
                                name: format!("{current}.{name}"),
                                pattern,
                                is_static: head.contains(" static "),
                                parameters,
                            });
                        }
                    }
                }
                if let Some(rest) = line.strip_prefix("[FieldOffset(") {
                    let offset_text = rest.split([')', ',']).next().unwrap_or("").trim();
                    let offset = if let Some(hex) = offset_text.strip_prefix("0x") {
                        i64::from_str_radix(hex, 16).ok()
                    } else {
                        offset_text.parse().ok()
                    };
                    let Some(offset) = offset else { continue };
                    let Some(declaration) = rest.split(']').nth(1) else {
                        continue;
                    };
                    let declaration = declaration.split(['=', ';']).next().unwrap_or("");
                    let words: Vec<&str> = declaration.split_whitespace().collect();
                    if words.len() < 2 {
                        continue;
                    }
                    let name = words[words.len() - 1].trim_start_matches('_');
                    let name = name[..1].to_uppercase() + &name[1..];
                    let ty = words[words.len() - 2].to_string();
                    structs
                        .entry(current.clone())
                        .or_default()
                        .push((offset, name, ty));
                }
            }
        }
        for fields in structs.values_mut() {
            fields.sort_by_key(|f| f.0);
            fields.dedup_by_key(|f| f.0);
        }
        Ok(Self {
            functions,
            structs,
            matched: RefCell::new(HashMap::new()),
        })
    }

    fn matching(&self, exe: &Exe, rva: usize) -> Option<usize> {
        if let Some(found) = self.matched.borrow().get(&rva) {
            return *found;
        }
        let image = exe.image();
        let found = self.functions.iter().position(|f| {
            // Call-site signatures (a call to the function) can't be checked at the function itself.
            f.pattern.first() != Some(&Some(0xE8))
                && f.pattern.first() != Some(&Some(0xE9))
                && rva + f.pattern.len() <= image.len()
                && f.pattern
                    .iter()
                    .enumerate()
                    .all(|(i, b)| b.is_none_or(|b| image[rva + i] == b))
        });
        self.matched.borrow_mut().insert(rva, found);
        found
    }

    /// The ClientStructs name of the function at `rva`, if a signature matches there.
    pub fn function_name(&self, exe: &Exe, rva: usize) -> Option<String> {
        self.matching(exe, rva)
            .map(|i| self.functions[i].name.clone())
    }

    /// The ClientStructs struct a function at `rva` takes in `register`, if it's one with fields (only for functions
    /// already named by `function_name`.)
    pub fn parameter_struct(&self, function: usize, register: Register) -> Option<String> {
        let index = self.matched.borrow().get(&function).copied().flatten()?;
        let f = &self.functions[index];
        let position = [Register::RCX, Register::RDX, Register::R8, Register::R9]
            .iter()
            .position(|r| *r == register)?;
        let parameter = if f.is_static {
            position
        } else {
            position.checked_sub(1)?
        };
        let ty = f.parameters.get(parameter)?.trim_end_matches('*');
        let ty = ty.rsplit('.').next().unwrap_or(ty);
        self.structs.contains_key(ty).then(|| ty.to_string())
    }

    /// How many argument registers the function at `rva` takes (`this` counted), for functions already named.
    pub fn argument_count(&self, rva: usize) -> Option<usize> {
        let index = self.matched.borrow().get(&rva).copied().flatten()?;
        let f = &self.functions[index];
        Some(f.parameters.len() + usize::from(!f.is_static))
    }

    /// A struct named after the packet (e.g. `ActorCastPacket` for ActorCast.)
    pub fn packet_struct_named(&self, packet: &str) -> Option<String> {
        let wanted = format!("{packet}packet").to_lowercase();
        self.structs
            .keys()
            .find(|name| name.to_lowercase() == wanted)
            .cloned()
    }

    /// The field of `struct_name` at `offset` (or inside it: "Field+0x4".)
    pub fn field_at(&self, struct_name: &str, offset: i64) -> Option<String> {
        let fields = self.structs.get(struct_name)?;
        let (at, name, ty) = fields.iter().rev().find(|(at, ..)| *at <= offset)?;
        if *at == offset {
            return Some(format!("{name} ({ty})"));
        }
        // Inside it only as far as its type is known to reach (a number, or a fixed array of them.)
        let reach = type_size(ty).unwrap_or(0);
        (offset - at < reach).then(|| format!("{name}+{:#x}", offset - at))
    }
}

/// How many bytes a ClientStructs field's type takes, for numbers and fixed arrays of them.
fn type_size(ty: &str) -> Option<i64> {
    let number = |t: &str| -> Option<i64> {
        Some(match t {
            "byte" | "sbyte" | "bool" => 1,
            "ushort" | "short" | "char" => 2,
            "uint" | "int" | "float" => 4,
            "ulong" | "long" | "double" | "nint" | "nuint" => 8,
            t if t.ends_with('*') => 8,
            _ => return None,
        })
    };
    if let Some(rest) = ty.strip_prefix("FixedSizeArray") {
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        let inner = rest[digits.len()..]
            .trim_start_matches('<')
            .trim_end_matches('>');
        return Some(digits.parse::<i64>().ok()? * number(inner)?);
    }
    number(ty)
}

fn collect(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.extension().is_some_and(|e| e == "cs") {
            out.push(path);
        }
    }
}
