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
//! kawari-opcodescan <exe> check-handlers <switch table RVA in hex> <opcode list, e.g. ServerZoneIpcType>
//!     Checks the handler method against every known opcode in the list: from each opcode's case in the switch to the
//!     function it calls, and back again.

use std::path::{Path, PathBuf};

use kawari::exe_scan::{Exe, check_opcode, find_opcode};
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
        Some(other) => eprintln!("Unknown command {other}"),
    }
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
