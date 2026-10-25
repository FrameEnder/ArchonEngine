#!/usr/bin/env python3
"""Reads rules the game client works out itself straight from its executable, for the world server
(`servers/world/src/client_rules.rs`), so they can be re-read after every game patch.

    python3 tools/clientlogic/clientlogic.py <ffxiv_dx11.exe> [--rust]

What it reads (the functions are found by FFXIVClientStructs' signatures, MIT):

* ActionManager.GetMaxCharges(action, level): an action's charges at a level. When its Action row has charges, a
  few actions check traits in turn and take the first learnt one's Value; otherwise the row's MaxCharges.
* ActionManager.GetAdjustedRecastTime(type, action, applyClassMechanics): an action's recast. Before the GCD's speed
  adjustment it adds, for some actions on some classes, a learnt trait's Value × 100 ms (Enhanced Swiftcast: -20 s.)

Both are a tree of comparisons on the action ID (and the class), so each candidate action is run through a small
emulator of that code with each class, answering "is this trait learnt?" with yes until the chain is exhausted.
Nothing of the game is kept: only action, class and trait numbers come out.
"""
import json
import re
import struct
import sys

import capstone

SIG_MAX_CHARGES = "E9 ?? ?? ?? ?? 41 B0 ?? 8B D3"
SIG_ADJUSTED_RECAST = "E8 ?? ?? ?? ?? 85 C0 7E 1C"
CLASS_JOBS = range(0, 45)


class Exe:
    def __init__(self, path):
        self.data = open(path, "rb").read()
        d = self.data
        pe = struct.unpack_from("<I", d, 0x3C)[0]
        nsec = struct.unpack_from("<H", d, pe + 6)[0]
        optsz = struct.unpack_from("<H", d, pe + 20)[0]
        self.base = struct.unpack_from("<Q", d, pe + 24 + 24)[0]
        self.secs = []
        off = pe + 24 + optsz
        for _ in range(nsec):
            name = d[off:off + 8].rstrip(b"\0").decode()
            vsize, va, rsize, raw = struct.unpack_from("<IIII", d, off + 8)
            self.secs.append((name, va, vsize, raw, rsize))
            off += 40
        self.md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_64)
        self.cache = {}

    def read(self, ea, n):
        rva = ea - self.base
        for _, va, vs, raw, rs in self.secs:
            if va <= rva < va + max(vs, rs):
                o = raw + rva - va
                return self.data[o:o + n]
        raise ValueError(hex(ea))

    def find(self, sig):
        parts = sig.split()
        rx = b"".join(b"." if p == "??" else re.escape(bytes([int(p, 16)])) for p in parts)
        _, va, _, raw, rs = next(s for s in self.secs if s[0] == ".text")
        tb = self.data[raw:raw + rs]
        return [self.base + va + m.start() for m in re.finditer(rx, tb, re.S)]

    def resolve(self, sig):
        hits = self.find(sig)
        if len(hits) != 1:
            raise SystemExit(f"signature {sig!r}: {len(hits)} matches")
        h = hits[0]
        if self.read(h, 1)[0] in (0xE8, 0xE9):
            return h + 5 + struct.unpack("<i", self.read(h + 1, 4))[0]
        return h

    def ins(self, ea):
        i = self.cache.get(ea)
        if i is None:
            i = next(self.md.disasm(self.read(ea, 16), ea))
            self.cache[ea] = i
        return i

    def function(self, ea, limit=3000):
        """Instructions from `ea` up to the first int3 padding."""
        out = []
        while len(out) < limit:
            try:
                i = self.ins(ea)
            except StopIteration:
                break
            if i.mnemonic == "int3":
                break
            out.append(i)
            ea += i.size
        return out


REGS = {"al": "rax", "eax": "rax", "ax": "rax", "bl": "rbx", "ebx": "rbx", "cl": "rcx", "ecx": "rcx", "dl": "rdx",
        "edx": "rdx", "esi": "rsi", "sil": "rsi", "edi": "rdi", "dil": "rdi", "ebp": "rbp", "r8d": "r8", "r8b": "r8",
        "r9d": "r9", "r9b": "r9", "r12d": "r12", "r12b": "r12", "r13d": "r13", "r13b": "r13", "r14d": "r14",
        "r14b": "r14", "r15d": "r15", "r15b": "r15"}
JCC = {"je", "jne", "jg", "jl", "jge", "jle", "ja", "jb", "jae", "jbe", "js", "jns"}


def run(exe, start, regs, calls, stops, limit=4000):
    """Follows one path: register moves, adds, subtractions, compares, tests and branches on known values; other
    instructions make their destination unknown, and a branch on unknown flags ends the path. `calls` stubs callees."""
    ea, flags, events = start, None, []
    for _ in range(limit):
        if ea in stops:
            return ea, events
        i = exe.ins(ea)
        m, nxt = i.mnemonic, ea + i.size
        ops = [o.strip() for o in i.op_str.split(",")] if i.op_str else []

        def val(o):
            if re.fullmatch(r"-?0x[0-9a-f]+|-?\d+", o):
                return int(o, 0) & 0xFFFFFFFF
            return regs.get(REGS.get(o, o))

        dst = REGS.get(ops[0], ops[0]) if ops else None
        if m == "ret":
            return ea, events
        if m in ("mov", "movzx", "movsx") and "[" not in ops[0]:
            regs[dst] = None if "[" in ops[1] else val(ops[1])
        elif m in ("sub", "add") and "[" not in i.op_str:
            a, b = regs.get(dst), val(ops[1])
            r = None if a is None or b is None else ((a - b) if m == "sub" else (a + b)) & 0xFFFFFFFF
            regs[dst], flags = r, (None if r is None else (r, 0))
        elif m == "cmp":
            a, b = (None, None) if "[" in i.op_str else (val(ops[0]), val(ops[1]))
            flags = None if a is None or b is None else (a, b)
        elif m == "test":
            a = None if "[" in i.op_str else val(ops[0])
            b = a if ops[1] == ops[0] else (None if "[" in i.op_str else val(ops[1]))
            flags = None if a is None or b is None else (a & b, 0)
        elif m == "xor" and ops[0] == ops[1]:
            regs[dst], flags = 0, (0, 0)
        elif m == "jmp":
            try:
                ea = int(ops[0], 0)
                continue
            except ValueError:
                return ea, events
        elif m in JCC:
            if flags is None:
                events.append(("unknown", ea))
                return ea, events
            a, b = flags
            sa, sb = (a - (1 << 32) if a & 0x80000000 else a), (b - (1 << 32) if b & 0x80000000 else b)
            taken = {"je": a == b, "jne": a != b, "jg": sa > sb, "jl": sa < sb, "jge": sa >= sb, "jle": sa <= sb,
                     "ja": a > b, "jb": a < b, "jae": a >= b, "jbe": a <= b, "js": sa < 0, "jns": sa >= 0}[m]
            ea = int(ops[0], 0) if taken else nxt
            continue
        elif m == "call":
            try:
                target = int(ops[0], 0)
            except ValueError:
                target = None
            stub = calls.get(target)
            if stub is not None and stub(regs, events) == "stop":
                return ea, events
            if stub is None:
                regs["rax"] = None
        elif ops and "[" not in ops[0]:
            regs[dst] = None
        ea = nxt
    events.append(("limit", ea))
    return ea, events


def callee_after(fn, pattern):
    """The call target most often followed (next instruction) by an instruction matching `pattern`."""
    counts = {}
    for a, b in zip(fn, fn[1:]):
        if a.mnemonic == "call" and re.match(pattern, f"{b.mnemonic} {b.op_str}"):
            try:
                t = int(a.op_str, 0)
            except ValueError:
                continue
            counts[t] = counts.get(t, 0) + 1
    return max(counts, key=counts.get) if counts else None


def trait_check(fn):
    """The trait check: the call preceded (within three instructions) by `mov edx, <trait>` and followed by `cmp al, 1`."""
    counts = {}
    for k, i in enumerate(fn):
        if i.mnemonic == "call" and k + 1 < len(fn) and f"{fn[k + 1].mnemonic} {fn[k + 1].op_str}" == "cmp al, 1":
            if any(f"{p.mnemonic} {p.op_str}".startswith("mov edx, 0x") for p in fn[max(0, k - 4):k]):
                t = int(i.op_str, 0)
                counts[t] = counts.get(t, 0) + 1
    return max(counts, key=counts.get)


def candidates(fn):
    """Action IDs the code compares with: `cmp <reg>, n`, and the running totals of `sub ecx, n` chains."""
    out, acc = set(), None
    for i in fn:
        m, s = i.mnemonic, i.op_str
        ops = [o.strip() for o in s.split(",")] if s else []
        if m == "mov" and len(ops) == 2 and ops[1] in ("ebx", "r13d", "esi", "edi"):
            acc = 0
        elif m == "sub" and acc is not None and re.fullmatch(r"0x[0-9a-f]+|\d+", ops[1]):
            acc += int(ops[1], 0)
            out.add(acc)
        elif m == "cmp" and len(ops) == 2 and re.fullmatch(r"0x[0-9a-f]+|\d+", ops[1]) and "[" not in ops[0]:
            n = int(ops[1], 0)
            out.add(n)
            if acc is not None:
                out.add(acc + n)
        elif m == "lea" and re.search(r"\[\w+ - (0x[0-9a-f]+)\]", s):
            base = int(re.search(r"\[\w+ - (0x[0-9a-f]+)\]", s).group(1), 0)
            out.update(base + d for d in range(0, 40))
    return sorted(c for c in out if 0 < c < 0x10000)


def max_charges(exe):
    fn_ea = exe.resolve(SIG_MAX_CHARGES)
    fn = exe.function(fn_ea)
    trait = trait_check(fn)
    # After `cmp byte [rax + 0x30], 0` (the row's MaxCharges): `jne <tree>`; the default is `mov eax, 1; jmp <end>`.
    k = next(n for n, i in enumerate(fn) if i.mnemonic == "cmp" and "+ 0x30], 0" in i.op_str)
    tree = int(fn[k + 1].op_str, 0)
    end = int(fn[k + 3].op_str, 0)
    rules = {}
    for action in candidates(fn):
        chain = []
        while True:
            def learnt(regs, events):
                events.append(regs.get("rdx"))
                regs["rax"] = 0 if regs.get("rdx") in chain else 1
            stop, events = run(exe, tree, {"rbx": action, "rdi": 0}, {trait: learnt}, {end})
            new = [t for t in events if isinstance(t, int) and t not in chain]
            if not new:
                break
            chain.append(new[0])
        if chain:
            rules[action] = chain
    return rules


def recast_traits(exe):
    fn_ea = exe.resolve(SIG_ADJUSTED_RECAST)
    fn = exe.function(fn_ea)
    trait = trait_check(fn)
    classjob = callee_after(fn, r"cmp eax, 0x[0-9a-f]+")
    charges_fn = exe.resolve(SIG_MAX_CHARGES)
    stop = next(i.address for i in fn if i.mnemonic == "call" and i.op_str == hex(charges_fn))
    # The action branch: `movzx eax, word [rsi + 0x16]` (Recast100ms) just before the first class check.
    start = next(i.address for n, i in enumerate(fn)
                 if i.op_str.endswith("word ptr [rsi + 0x16]") and i.address < stop
                 and any(j.mnemonic == "call" and j.op_str == hex(classjob) for j in fn[n:n + 6]))
    rules = {}
    for action in candidates([i for i in fn if start <= i.address < stop]):
        per_job = {}
        for job in CLASS_JOBS:
            def get_job(regs, events, job=job):
                regs["rax"] = job

            def learnt(regs, events):
                events.append(regs.get("rdx"))
                regs["rax"] = 1
            end, events = run(exe, start, {"rbx": action, "rsi": 1}, {classjob: get_job, trait: learnt}, {stop})
            traits = [t for t in events if isinstance(t, int)]
            if end != stop:
                raise SystemExit(f"recast: action {action} class {job} stopped at {end:#x}: {events}")
            for t in traits:
                per_job.setdefault(t, set()).add(job)
        for t, jobs in per_job.items():
            rules.setdefault(action, []).append((t, None if len(jobs) == len(CLASS_JOBS) else sorted(jobs)))
    return rules


def main():
    if len(sys.argv) < 2:
        raise SystemExit(__doc__)
    exe = Exe(sys.argv[1])
    charges = max_charges(exe)
    recasts = recast_traits(exe)
    if "--rust" in sys.argv:
        print("pub const MAX_CHARGE_TRAITS: &[(u32, &[u32])] = &[")
        for a, chain in sorted(charges.items()):
            print(f"    ({a}, &{chain}),")
        print("];")
        print("pub const RECAST_TRAITS: &[(u32, u32, &[u8])] = &[")
        for a, items in sorted(recasts.items()):
            for t, jobs in items:
                print(f"    ({a}, {t}, &{jobs or []}),")
        print("];")
    else:
        print(json.dumps({"max_charges": charges, "recast_traits": {a: [[t, j] for t, j in v] for a, v in recasts.items()}},
                         indent=1))


if __name__ == "__main__":
    main()
