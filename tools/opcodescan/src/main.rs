//! Reads opcodes out of the game's executable, the same way the world server does at startup.
//!
//! kawari-opcodescan <game folder or ffxiv_dx11.exe>
//!     Runs every fork opcode's finders (resources/data/fork_opcodes.yml) and prints what they read.
//! kawari-opcodescan <exe> switches [minimum cases]
//!     Lists the switches in the game's code with at least that many cases (default 200), e.g. packet dispatchers.
//! kawari-opcodescan <exe> case <switch table RVA in hex> <opcode>
//!     Prints the code address a switch sends an opcode to.
//! kawari-opcodescan <exe> handler <function RVA in hex>
//!     Follows calls back from a function to the switch cases (opcodes) that lead to it.
//! kawari-opcodescan <exe> fingerprint <out.json> <opcodes.yml> <fork_opcodes.yml> <game version>
//!     Records the fingerprints of every zone opcode in those files (as they are on that game version) from this
//!     executable, for finding them in the next version (resources/data/opcode_fingerprints.json.)
//! kawari-opcodescan <exe> match <fingerprints.json>
//!     Finds the fingerprinted opcodes in this executable, and prints them as JSON: {list: {name: opcode, or a list
//!     of candidates, or null}}; client packets it builds with another size than this build has go to stderr.
//! kawari-opcodescan <exe> suggest-finders <fingerprints.json> [other exe] [all]
//!     Writes finders for the fork's opcodes that have none (every one with `all`), from this executable (the
//!     fingerprints must have been recorded from it), checked to read each opcode here; with another executable (another
//!     game version), also what each reads there. Prints them as fork_opcodes.yml `find:` entries.
//! kawari-opcodescan <exe> check-handlers <switch table RVA in hex> <opcode list, e.g. ServerZoneIpcType>
//!     Checks the handler method against every known opcode in the list: from each opcode's case in the switch to the
//!     function it calls, and back again.
//! kawari-opcodescan <exe> layouts <opcodes.yml> <fork_opcodes.yml> <game version> <core/src> <out folder>
//!         [<ClientStructs folder>]
//!     Reads every known zone packet's layout from the client's code (what its handler reads, what the code that
//!     builds a client packet writes), compares it with Kawari's structs (read from core's source), and writes
//!     layouts.json and layouts.md into the out folder (docs/src/fork/packet-layouts.md.)

use std::path::{Path, PathBuf};

mod clientstructs;
mod kawari_structs;
mod layouts;

use kawari::exe_scan::{Exe, check_opcode, find_opcode};
use kawari::opcode_match;
use kawari::opcodes::{FORK_OPCODES, KNOWN_OPCODES};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(path) = args.first() else {
        eprintln!(
            "Usage: kawari-opcodescan <game folder or ffxiv_dx11.exe> [switches [min] | case <table> <opcode> | handler <rva>]"
        );
        std::process::exit(1);
    };
    let mut path = PathBuf::from(path);
    if path.is_dir() {
        path.push("ffxiv_dx11.exe");
    }
    let exe = match Exe::open(&path) {
        Ok(exe) => exe,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    };

    let hex = |text: &str| usize::from_str_radix(text.trim_start_matches("0x"), 16).expect("not a hex number");
    match args.get(1).map(String::as_str) {
        None => scan(&exe, &path),
        Some("switches") => {
            let min: usize = args.get(2).and_then(|min| min.parse().ok()).unwrap_or(200);
            let (start, end) = exe.text();
            for switch in exe.switches(start, end) {
                let indices: usize = switch.cases.values().map(Vec::len).sum();
                if indices >= min {
                    println!(
                        "switch at 0x{:X} in function 0x{:X}: opcodes {}..={} ({} distinct cases)",
                        switch.at,
                        exe.function_start(switch.at),
                        switch.bias,
                        switch.bias + indices as i64 - 1,
                        switch.cases.len()
                    );
                }
            }
        }
        Some("case") => {
            let at = hex(&args[2]);
            let opcode: i64 = args[3].parse().expect("not an opcode");
            let function = exe.function_start(at);
            for switch in exe.switches(function, exe.text().1) {
                if switch.at != at {
                    continue;
                }
                for (target, indices) in &switch.cases {
                    if indices.iter().any(|i| switch.bias + i == opcode) {
                        println!("opcode {opcode} goes to 0x{target:X}");
                    }
                }
                return;
            }
            println!("no switch at 0x{at:X}");
        }
        Some("handler") => {
            for case in exe.opcode_cases(hex(&args[2]), 4) {
                println!(
                    "depth {}: call at 0x{:X} in function 0x{:X} (a switch with {} cases) is the case for opcode(s) {:?}",
                    case.depth, case.site, case.function, case.switch_cases, case.opcodes
                );
            }
        }
        Some("check-handlers") => check_handlers(&exe, hex(&args[2]), &args[3]),
        Some("fingerprint") => {
            let read = |path: &String| {
                std::fs::read_to_string(path).unwrap_or_else(|err| panic!("{path}: {err}"))
            };
            let known = opcode_match::known_from_yaml(&read(&args[3]), &read(&args[4]), &args[5])
                .expect("couldn't read the opcodes");
            let prints =
                opcode_match::fingerprint(&exe, &args[5], &known).expect("couldn't fingerprint");
            let client_sites: usize = prints
                .client
                .iter()
                .filter(|print| !print.sites.is_empty())
                .count();
            std::fs::write(&args[2], serde_json::to_string_pretty(&prints).unwrap())
                .expect("couldn't write");
            println!(
                "{}: {} server opcodes ({} with a second dispatcher), {} client opcodes ({} with a function that builds them)",
                args[2],
                prints.server.len(),
                prints
                    .server
                    .iter()
                    .filter(|print| print.sub.is_some())
                    .count(),
                prints.client.len(),
                client_sites
            );
        }
        Some("paths") => {
            for (opcode, case, end) in opcode_match::debug_paths(&exe, hex(&args[2])) {
                println!("{opcode} 0x{case:X} 0x{end:X}");
            }
        }
        Some("match") => {
            let text = std::fs::read_to_string(&args[2]).expect("can't read the fingerprints");
            let prints: opcode_match::Fingerprints =
                serde_json::from_str(&text).expect("not fingerprints");
            let mut out = serde_json::Map::new();
            for ((kind, name), found) in opcode_match::match_fingerprints(&exe, &prints) {
                let value = match found {
                    opcode_match::Found::Opcode(opcode) => serde_json::json!(opcode),
                    opcode_match::Found::OneOf(candidates) => serde_json::json!(candidates),
                    opcode_match::Found::NotFound => serde_json::Value::Null,
                };
                out.entry(kind)
                    .or_insert_with(|| serde_json::Value::Object(Default::default()))
                    .as_object_mut()
                    .unwrap()
                    .insert(name, value);
            }
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
            // Client packets now built with another size than this build has: their layout changed.
            let client: std::collections::BTreeMap<String, u16> = out
                .get("ClientZoneIpcType")
                .and_then(|list| list.as_object())
                .map(|list| {
                    list.iter()
                        .filter_map(|(name, value)| Some((name.clone(), value.as_u64()? as u16)))
                        .collect()
                })
                .unwrap_or_default();
            let sizes: std::collections::BTreeMap<String, u32> = client
                .keys()
                .filter_map(|name| {
                    kawari::opcodes::opcode_size("ClientZoneIpcType", name)
                        .map(|size| (name.clone(), size as u32))
                })
                .collect();
            for (name, old, new) in opcode_match::client_size_changes(&exe, &client, &sizes) {
                eprintln!(
                    "ClientZoneIpcType::{name}: built with {new:?} bytes here, {old} in this build"
                );
            }
        }
        Some("suggest-finders") => {
            let text = std::fs::read_to_string(&args[2]).expect("can't read the fingerprints");
            let prints: opcode_match::Fingerprints =
                serde_json::from_str(&text).expect("not fingerprints");
            let other = args
                .get(3)
                .filter(|arg| arg.as_str() != "all")
                .map(|path| Exe::open(Path::new(path)).expect("can't read the other executable"));
            let all = args.iter().any(|arg| arg == "all");
            suggest_finders(&exe, &prints, other.as_ref(), all);
        }
        Some("layouts") => {
            let read = |path: &String| {
                std::fs::read_to_string(path).unwrap_or_else(|err| panic!("{path}: {err}"))
            };
            let known = opcode_match::known_from_yaml(&read(&args[2]), &read(&args[3]), &args[4])
                .expect("couldn't read the opcodes");
            let kawari =
                kawari_structs::read(Path::new(&args[5])).expect("couldn't read core's source");
            let names = args.get(7).map(|dir| {
                clientstructs::ClientStructs::read(Path::new(dir))
                    .expect("couldn't read ClientStructs")
            });
            let found =
                layouts::layouts(&exe, &known, names.as_ref()).expect("couldn't read the layouts");
            let findings = layouts::compare(&found, &kawari);
            let out = PathBuf::from(&args[6]);
            std::fs::create_dir_all(&out).expect("couldn't make the out folder");
            std::fs::write(
                out.join("layouts.json"),
                serde_json::to_string_pretty(&layouts::to_json(&found, &findings, &args[4]))
                    .unwrap(),
            )
            .expect("couldn't write layouts.json");
            std::fs::write(
                out.join("layouts.md"),
                layouts::to_markdown(&found, &findings, &args[4], &kawari),
            )
            .expect("couldn't write layouts.md");
            for kind in ["server", "client"] {
                let of_kind: Vec<&layouts::PacketLayout> =
                    found.iter().filter(|l| l.kind == kind).collect();
                let count = |c: &str| of_kind.iter().filter(|l| l.confidence() == c).count();
                println!(
                    "{kind} packets: {} mapped ({} high, {} medium, {} low/shared, {} nothing found)",
                    of_kind.len(),
                    count("high"),
                    count("medium"),
                    count("low"),
                    count("none")
                );
            }
            for list in ["ServerZoneIpcData", "ClientZoneIpcData"] {
                let (complete, all) = kawari.complete(list);
                println!("{list}: {complete} of {all} Kawari variants laid out to the end");
            }
            println!(
                "{} findings ({} confident); written to {}",
                findings.len(),
                findings.iter().filter(|f| f.confident).count(),
                out.display()
            );
        }
        Some(other) => eprintln!("Unknown command {other}"),
    }
}

/// Finders for the fork's opcodes, from this executable's fingerprints (`kawari::finder_gen`.)
fn suggest_finders(exe: &Exe, prints: &opcode_match::Fingerprints, other: Option<&Exe>, all: bool) {
    use kawari::finder_gen::{suggest_handler, suggest_immediate, try_finder};
    let (mut written, mut failed) = (0, 0);
    for opcode in FORK_OPCODES {
        if !all && !opcode.finders.is_empty() {
            continue;
        }
        let Some(known) = opcode.opcode.filter(|_| opcode.current) else {
            println!("# {}: not known on this version, skipped", opcode.name);
            continue;
        };
        let suggestion = if opcode.kind.starts_with("Server") {
            prints
                .server
                .iter()
                .find(|print| print.name == opcode.name && print.opcode == known)
                .and_then(|print| suggest_handler(exe, print.handler_at, known))
        } else {
            prints
                .client
                .iter()
                .find(|print| print.name == opcode.name && print.opcode == known)
                .and_then(|print| {
                    print
                        .sites
                        .iter()
                        .find_map(|site| suggest_immediate(exe, site.function_at, known))
                })
        };
        let Some(suggestion) = suggestion else {
            println!(
                "# {} ({known}): no finder could be written (its code isn't unique)",
                opcode.name
            );
            failed += 1;
            continue;
        };
        written += 1;
        let elsewhere = other.map(|other| match try_finder(other, &suggestion) {
            Ok(read) => format!("reads {read:?} in the other executable"),
            Err(err) => format!("in the other executable: {err}"),
        });
        println!(
            "# {} ({known}), at 0x{:X}{}",
            opcode.name,
            suggestion.at,
            elsewhere
                .map(|text| format!("; {text}"))
                .unwrap_or_default()
        );
        print!("{}", suggestion.yaml());
    }
    println!("# {written} written, {failed} without");
}

fn scan(exe: &Exe, path: &Path) {
    println!("{}", path.display());
    for opcode in FORK_OPCODES {
        if opcode.finders.is_empty() {
            println!("{}::{}: no finders", opcode.kind, opcode.name);
            continue;
        }
        if let (true, Some(known)) = (opcode.current, opcode.opcode) {
            match check_opcode(exe, opcode, known) {
                Ok(()) => println!("{}::{}: {known}, confirmed by its finder", opcode.kind, opcode.name),
                Err(err) => println!("{}::{}: {known}, but its finder disagrees: {err}", opcode.kind, opcode.name),
            }
            continue;
        }
        match find_opcode(exe, opcode) {
            Ok(read) => println!("{}::{}: {read} (read from the game)", opcode.kind, opcode.name),
            Err(err) => println!("{}::{}: not found: {err}", opcode.kind, opcode.name),
        }
    }
}

/// From each known opcode's case to its handler, and back: does the handler finder get the opcode again?
fn check_handlers(exe: &Exe, switch_at: usize, kind: &str) {
    let function = exe.function_start(switch_at);
    let Some(switch) = exe
        .switches(function, exe.text().1)
        .into_iter()
        .find(|switch| switch.at == switch_at)
    else {
        println!("no switch at 0x{switch_at:X}");
        return;
    };

    let (mut exact, mut shared, mut wrong, mut no_handler, mut not_in_switch) = (0, 0, 0, 0, 0);
    for (_, name, opcode) in KNOWN_OPCODES.iter().filter(|(k, _, _)| *k == kind) {
        let Some((case, _)) = switch
            .cases
            .iter()
            .find(|(_, indices)| indices.iter().any(|i| switch.bias + i == *opcode as i64))
        else {
            not_in_switch += 1;
            continue;
        };
        // A call that's made from all over the game is a utility (e.g. getting a module), not this packet's handler.
        let Some(handler) = exe
            .first_call_in_case(*case)
            .filter(|handler| exe.calls_to(*handler).len() <= 50)
        else {
            no_handler += 1;
            continue;
        };
        let cases = exe.opcode_cases(handler, 4);
        let biggest = cases.iter().map(|case| case.switch_cases).max().unwrap_or(0);
        let mut back: Vec<i64> = cases
            .iter()
            .filter(|case| case.switch_cases == biggest)
            .flat_map(|case| case.opcodes.iter().copied())
            .collect();
        back.sort();
        back.dedup();
        if back == [*opcode as i64] {
            exact += 1;
        } else if back.contains(&(*opcode as i64)) {
            shared += 1;
            println!("shared: {name} ({opcode}) has the same handler as {back:?}");
        } else {
            wrong += 1;
            println!("WRONG: {name} ({opcode}) came back as {back:?}");
        }
    }
    println!(
        "{kind}: {exact} exact, {shared} sharing a handler, {wrong} wrong, {no_handler} without a handler of their own to start from, {not_in_switch} not in this switch"
    );
}
