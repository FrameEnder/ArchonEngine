//! Kawari's own packet layouts, read from core's source with `syn`: each `ServerZoneIpcData` / `ClientZoneIpcData`
//! variant's fields with their offsets and sizes on the wire, following binrw's attributes (pad_before, pad_after,
//! pad_size_to, count, map, ignore, calc, temp, args) and the field types (primitives, arrays, core's structs and
//! enums with a repr.) Server packets are laid out as Kawari writes them, client packets as it reads them. A field
//! whose size can't be worked out (a variable count, an enum told apart by magic) stops the offsets after it.

use std::collections::HashMap;
use std::path::Path;

use syn::{Attribute, Expr, Fields, Item, Lit, Meta, Type};

#[derive(Debug, Clone, Default)]
pub struct KawariField {
    /// Dotted for a field of a field's struct (`common.health_points`.)
    pub name: String,
    pub offset: i64,
    pub size: Option<u32>,
    /// The type on the wire, as written in the source (or a map's wire type.)
    pub ty: String,
    /// A number (not an array, struct or string.)
    pub primitive: bool,
}

pub struct KawariLayouts {
    variants: HashMap<(String, String), Vec<KawariField>>,
}

impl KawariLayouts {
    pub fn variant(&self, list: &str, name: &str) -> Option<&Vec<KawariField>> {
        self.variants.get(&(list.to_string(), name.to_string()))
    }

    /// How many variants of `list` were laid out to the end.
    pub fn complete(&self, list: &str) -> (usize, usize) {
        let all: Vec<&Vec<KawariField>> = self
            .variants
            .iter()
            .filter(|((l, _), _)| l == list)
            .map(|(_, f)| f)
            .collect();
        (
            all.iter()
                .filter(|f| f.iter().all(|f| f.size.is_some()))
                .count(),
            all.len(),
        )
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Side {
    /// As Kawari writes it (server packets.)
    Write,
    /// As Kawari reads it (client packets.)
    Read,
}

struct Source {
    structs: HashMap<String, syn::ItemStruct>,
    enums: HashMap<String, syn::ItemEnum>,
    consts: HashMap<String, i64>,
    /// Function name → (first parameter's type, return type), for maps.
    functions: HashMap<String, (Option<Type>, Option<Type>)>,
}

/// Reads core's source under `core_src` and lays out both lists.
pub fn read(core_src: &Path) -> Result<KawariLayouts, String> {
    let mut files = Vec::new();
    collect(core_src, &mut files);
    let mut source = Source {
        structs: HashMap::new(),
        enums: HashMap::new(),
        consts: HashMap::new(),
        functions: HashMap::new(),
    };
    for file in &files {
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        let Ok(parsed) = syn::parse_file(&text) else {
            continue;
        };
        gather(&parsed.items, &mut source);
    }
    let mut variants = HashMap::new();
    for (list, side) in [
        ("ServerZoneIpcData", Side::Write),
        ("ClientZoneIpcData", Side::Read),
    ] {
        let Some(item) = source.enums.get(list).cloned() else {
            return Err(format!("{list} wasn't found in {}", core_src.display()));
        };
        for variant in &item.variants {
            let mut fields = Vec::new();
            let mut offset = Some(0i64);
            let args = args_of(&variant.attrs);
            match &variant.fields {
                Fields::Named(named) => {
                    for field in &named.named {
                        let name = field
                            .ident
                            .as_ref()
                            .map(|i| i.to_string())
                            .unwrap_or_default();
                        lay_field(
                            &source,
                            &name,
                            &field.ty,
                            &field.attrs,
                            &args,
                            side,
                            &mut offset,
                            &mut fields,
                            0,
                        );
                    }
                }
                Fields::Unnamed(unnamed) if unnamed.unnamed.len() == 1 => {
                    let field = &unnamed.unnamed[0];
                    let mut field_args = args.clone();
                    field_args.extend(args_of(&field.attrs));
                    if let Some(name) = type_name(&field.ty)
                        && let Some(item) = source.structs.get(&name).cloned()
                    {
                        lay_struct(
                            &source,
                            &item,
                            "",
                            &field_args,
                            side,
                            &mut offset,
                            &mut fields,
                            0,
                        );
                    } else {
                        lay_field(
                            &source,
                            "0",
                            &field.ty,
                            &field.attrs,
                            &field_args,
                            side,
                            &mut offset,
                            &mut fields,
                            0,
                        );
                    }
                }
                _ => {}
            }
            variants.insert((list.to_string(), variant.ident.to_string()), fields);
        }
    }
    Ok(KawariLayouts { variants })
}

fn collect(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn gather(items: &[Item], source: &mut Source) {
    for item in items {
        match item {
            Item::Struct(s) => {
                source.structs.insert(s.ident.to_string(), s.clone());
            }
            Item::Enum(e) => {
                source.enums.insert(e.ident.to_string(), e.clone());
            }
            Item::Const(c) => {
                if let Some(value) = eval(&c.expr, &HashMap::new(), &source.consts) {
                    source.consts.insert(c.ident.to_string(), value);
                }
            }
            Item::Fn(f) => {
                let first = f.sig.inputs.iter().find_map(|input| match input {
                    syn::FnArg::Typed(t) => Some((*t.ty).clone()),
                    _ => None,
                });
                let ret = match &f.sig.output {
                    syn::ReturnType::Type(_, ty) => Some((**ty).clone()),
                    _ => None,
                };
                source
                    .functions
                    .insert(f.sig.ident.to_string(), (first, ret));
            }
            Item::Mod(m) => {
                if let Some((_, items)) = &m.content {
                    gather(items, source);
                }
            }
            Item::Impl(i) => {
                for item in &i.items {
                    if let syn::ImplItem::Fn(f) = item {
                        let first = f.sig.inputs.iter().find_map(|input| match input {
                            syn::FnArg::Typed(t) => Some((*t.ty).clone()),
                            _ => None,
                        });
                        let ret = match &f.sig.output {
                            syn::ReturnType::Type(_, ty) => Some((**ty).clone()),
                            _ => None,
                        };
                        source
                            .functions
                            .entry(f.sig.ident.to_string())
                            .or_insert((first, ret));
                    }
                }
            }
            _ => {}
        }
    }
}

/// binrw's `args { name: value }` on a variant or field, as numbers.
fn args_of(attrs: &[Attribute]) -> HashMap<String, i64> {
    let mut args = HashMap::new();
    for attr in attrs {
        let text = attr.meta.to_token_stream_string();
        if let Some(at) = text.find("args") {
            let rest = &text[at..];
            if let (Some(open), Some(close)) = (rest.find('{'), rest.find('}')) {
                for pair in rest[open + 1..close].split(',') {
                    let mut parts = pair.split(':');
                    if let (Some(k), Some(v)) = (parts.next(), parts.next())
                        && let Ok(v) = v.trim().parse::<i64>()
                    {
                        args.insert(k.trim().to_string(), v);
                    }
                }
            }
        }
    }
    args
}

trait TokenString {
    fn to_token_stream_string(&self) -> String;
}

impl TokenString for Meta {
    fn to_token_stream_string(&self) -> String {
        quote::ToTokens::to_token_stream(self).to_string()
    }
}

/// One binrw attribute's settings on a field.
#[derive(Default)]
struct BinrwAttrs {
    pad_before: Option<Expr>,
    pad_after: Option<Expr>,
    pad_size_to: Option<Expr>,
    count: Option<Expr>,
    read_map: Option<Expr>,
    write_map: Option<Expr>,
    skip_read: bool,
    skip_write: bool,
    conditional: bool,
}

fn binrw_attrs(attrs: &[Attribute]) -> BinrwAttrs {
    let mut out = BinrwAttrs::default();
    for attr in attrs {
        let path = attr
            .path()
            .get_ident()
            .map(|i| i.to_string())
            .unwrap_or_default();
        if !matches!(path.as_str(), "br" | "bw" | "brw") {
            continue;
        }
        let read = path != "bw";
        let write = path != "br";
        let Ok(list) = attr
            .parse_args_with(syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated)
        else {
            continue;
        };
        for meta in list {
            let key = meta
                .path()
                .get_ident()
                .map(|i| i.to_string())
                .unwrap_or_default();
            let value = match &meta {
                Meta::NameValue(nv) => Some(nv.value.clone()),
                _ => None,
            };
            match key.as_str() {
                "pad_before" => out.pad_before = value,
                "pad_after" => out.pad_after = value,
                "pad_size_to" => out.pad_size_to = value,
                "count" => out.count = value,
                "map" | "try_map" => {
                    if read {
                        out.read_map = value.clone();
                    }
                    if write {
                        out.write_map = value;
                    }
                }
                "ignore" | "calc" | "default" => {
                    // `br(calc)`/`br(ignore)` skip reading, `bw(ignore)` skips writing.
                    if path == "br" {
                        out.skip_read = true;
                    } else if path == "bw" && key == "ignore" {
                        out.skip_write = true;
                    } else if path == "brw" {
                        out.skip_read = true;
                        out.skip_write = true;
                    }
                }
                "if" => out.conditional = true,
                _ => {}
            }
        }
    }
    out
}

/// A small number expression: literals, consts, binrw args, + - * /, parentheses, casts.
fn eval(expr: &Expr, args: &HashMap<String, i64>, consts: &HashMap<String, i64>) -> Option<i64> {
    match expr {
        Expr::Lit(lit) => match &lit.lit {
            Lit::Int(i) => i.base10_parse().ok(),
            _ => None,
        },
        Expr::Path(p) => {
            let name = p.path.segments.last()?.ident.to_string();
            args.get(&name).or_else(|| consts.get(&name)).copied()
        }
        Expr::Paren(p) => eval(&p.expr, args, consts),
        Expr::Group(g) => eval(&g.expr, args, consts),
        Expr::Cast(c) => eval(&c.expr, args, consts),
        Expr::Binary(b) => {
            let l = eval(&b.left, args, consts)?;
            let r = eval(&b.right, args, consts)?;
            match b.op {
                syn::BinOp::Add(_) => Some(l + r),
                syn::BinOp::Sub(_) => Some(l - r),
                syn::BinOp::Mul(_) => Some(l * r),
                syn::BinOp::Div(_) if r != 0 => Some(l / r),
                _ => None,
            }
        }
        _ => None,
    }
}

fn type_name(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
        _ => None,
    }
}

fn type_text(ty: &Type) -> String {
    quote::ToTokens::to_token_stream(ty)
        .to_string()
        .replace(' ', "")
}

/// The wire type of a map: a turbofish's type (`read_bool_from::<u32>`), a closure's annotated parameter, or the
/// function's first parameter (reading) or return type (writing.)
fn map_wire_type(source: &Source, map: &Expr, side: Side) -> Option<Type> {
    match map {
        Expr::Path(p) => {
            let last = p.path.segments.last()?;
            if let syn::PathArguments::AngleBracketed(args) = &last.arguments
                && let Some(syn::GenericArgument::Type(ty)) = args.args.first()
            {
                return Some(ty.clone());
            }
            let (first, ret) = source.functions.get(&last.ident.to_string())?;
            match side {
                Side::Read => first.clone(),
                Side::Write => ret.clone(),
            }
        }
        Expr::Closure(c) => match side {
            Side::Read => c.inputs.first().and_then(|pat| match pat {
                syn::Pat::Type(t) => Some((*t.ty).clone()),
                _ => None,
            }),
            Side::Write => match &c.output {
                syn::ReturnType::Type(_, ty) => Some((**ty).clone()),
                _ => None,
            },
        },
        _ => None,
    }
}

fn primitive_size(name: &str) -> Option<u32> {
    Some(match name {
        "u8" | "i8" | "bool" => 1,
        "u16" | "i16" => 2,
        "u32" | "i32" | "f32" => 4,
        "u64" | "i64" | "f64" => 8,
        "u128" | "i128" => 16,
        _ => return None,
    })
}

/// The size of a type with no field attributes (None if it varies or isn't known.)
fn type_size(
    source: &Source,
    ty: &Type,
    args: &HashMap<String, i64>,
    side: Side,
    depth: usize,
) -> Option<u32> {
    if depth > 12 {
        return None;
    }
    match ty {
        Type::Array(a) => {
            let n = eval(&a.len, args, &source.consts)?;
            Some(type_size(source, &a.elem, args, side, depth + 1)? * n as u32)
        }
        Type::Tuple(t) if t.elems.is_empty() => Some(0),
        Type::Path(_) => {
            let name = type_name(ty)?;
            if let Some(size) = primitive_size(&name) {
                return Some(size);
            }
            if let Some(item) = source.structs.get(&name) {
                let mut offset = Some(0i64);
                let mut fields = Vec::new();
                lay_struct(
                    source,
                    item,
                    "",
                    args,
                    side,
                    &mut offset,
                    &mut fields,
                    depth + 1,
                );
                return offset.map(|o| o as u32);
            }
            if let Some(item) = source.enums.get(&name) {
                for attr in &item.attrs {
                    let text = attr.meta.to_token_stream_string().replace(' ', "");
                    for (repr, size) in [
                        ("u8", 1),
                        ("u16", 2),
                        ("u32", 4),
                        ("u64", 8),
                        ("i8", 1),
                        ("i16", 2),
                        ("i32", 4),
                    ] {
                        if text.contains(&format!("repr={repr}")) || text == format!("repr({repr})")
                        {
                            return Some(size);
                        }
                    }
                }
            }
            None
        }
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn lay_struct(
    source: &Source,
    item: &syn::ItemStruct,
    prefix: &str,
    args: &HashMap<String, i64>,
    side: Side,
    offset: &mut Option<i64>,
    fields: &mut Vec<KawariField>,
    depth: usize,
) {
    match &item.fields {
        Fields::Named(named) => {
            for field in &named.named {
                let name = field
                    .ident
                    .as_ref()
                    .map(|i| i.to_string())
                    .unwrap_or_default();
                let name = if prefix.is_empty() {
                    name
                } else {
                    format!("{prefix}.{name}")
                };
                lay_field(
                    source,
                    &name,
                    &field.ty,
                    &field.attrs,
                    args,
                    side,
                    offset,
                    fields,
                    depth,
                );
            }
        }
        Fields::Unnamed(unnamed) => {
            for (i, field) in unnamed.unnamed.iter().enumerate() {
                let name = if prefix.is_empty() {
                    i.to_string()
                } else {
                    format!("{prefix}.{i}")
                };
                lay_field(
                    source,
                    &name,
                    &field.ty,
                    &field.attrs,
                    args,
                    side,
                    offset,
                    fields,
                    depth,
                );
            }
        }
        Fields::Unit => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn lay_field(
    source: &Source,
    name: &str,
    ty: &Type,
    attrs: &[Attribute],
    args: &HashMap<String, i64>,
    side: Side,
    offset: &mut Option<i64>,
    fields: &mut Vec<KawariField>,
    depth: usize,
) {
    let a = binrw_attrs(attrs);
    let skipped = match side {
        Side::Read => a.skip_read,
        Side::Write => a.skip_write,
    };
    if skipped {
        return;
    }
    let Some(start) = *offset else {
        fields.push(KawariField {
            name: name.to_string(),
            offset: -1,
            size: None,
            ty: type_text(ty),
            primitive: false,
        });
        return;
    };
    let mut at = start;
    if let Some(pad) = &a.pad_before {
        match eval(pad, args, &source.consts) {
            Some(n) => at += n,
            None => {
                *offset = None;
                return;
            }
        }
    }
    let map = match side {
        Side::Read => a.read_map.as_ref(),
        Side::Write => a.write_map.as_ref(),
    };
    let wire = map
        .and_then(|m| map_wire_type(source, m, side))
        .unwrap_or_else(|| ty.clone());
    // A Vec or a string: its count, or its padded size.
    let wire_name = type_name(&wire).unwrap_or_default();
    let mut size = if wire_name == "Vec" || wire_name == "String" {
        let element = match &wire {
            Type::Path(p) => match &p.path.segments.last().unwrap().arguments {
                syn::PathArguments::AngleBracketed(g) => g.args.first().and_then(|a| match a {
                    syn::GenericArgument::Type(t) => Some(t.clone()),
                    _ => None,
                }),
                _ => None,
            },
            _ => None,
        };
        match (&a.count, element) {
            (Some(count), Some(element)) => eval(count, args, &source.consts).and_then(|n| {
                type_size(source, &element, args, side, depth + 1).map(|s| s * n as u32)
            }),
            _ => None,
        }
    } else {
        type_size(source, &wire, args, side, depth)
    };
    if let Some(padded) = &a.pad_size_to {
        // pad_size_to pads up to that size; a field already as big stays as it is.
        if let Some(n) = eval(padded, args, &source.consts) {
            size = Some(size.map_or(n as u32, |s| s.max(n as u32)));
        }
    }
    if a.conditional {
        size = None;
    }
    // A struct's own fields, so an `unk` inside it can be found.
    let nested = (map.is_none() && depth < 6)
        .then(|| type_name(&wire))
        .flatten()
        .and_then(|n| source.structs.get(&n).cloned())
        .filter(|_| size.is_some() && !matches!(wire, Type::Array(_)));
    if let Some(item) = nested {
        let mut inner = Some(at);
        let before = fields.len();
        lay_struct(
            source,
            &item,
            name,
            args,
            side,
            &mut inner,
            fields,
            depth + 1,
        );
        if fields.len() == before {
            fields.push(KawariField {
                name: name.to_string(),
                offset: at,
                size,
                ty: type_text(&wire),
                primitive: false,
            });
        }
    } else {
        fields.push(KawariField {
            name: name.to_string(),
            offset: at,
            size,
            ty: type_text(&wire),
            primitive: primitive_size(&wire_name).is_some() && !matches!(wire, Type::Array(_)),
        });
    }
    let Some(size) = size else {
        *offset = None;
        return;
    };
    at += size as i64;
    if let Some(pad) = &a.pad_after {
        match eval(pad, args, &source.consts) {
            Some(n) => at += n,
            None => {
                *offset = None;
                return;
            }
        }
    }
    *offset = Some(at);
}
