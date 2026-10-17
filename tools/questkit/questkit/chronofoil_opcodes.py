"""How the opcode tables for older game versions in data/chronofoil_opcodes.json were made.

Chronofoil captures are only readable with their own game version's opcodes, and Kawari only knows the version it
supports. The saved tables come from:

* **Kawari's opcodes.yml as it was** on a version it supported (`KAWARI_COMMITS`): exact, with sizes.
* **A hotfix away from one of those** (`SHIFTED`): a hotfix reshuffles most opcodes. FFXIVOpcodes
  (github.com/karashiiro/FFXIVOpcodes, by patch name) has about a third of Kawari's packets under its own names, for
  every patch; an opcode that's X in FFXIVOpcodes on Kawari's version and Y on the hotfix is Y in the hotfix.
* **Sizes** (`infer_by_size`): a Kawari packet whose size no other packet has, seen under exactly one opcode that's
  still unnamed in that version's captures.
* **Contents** (`CONTENT_PICKS`): checked by hand where size wasn't enough, e.g. DirectorVars (24 bytes, like a dozen
  others) is the only one whose first four bytes are a director (0x8003xxxx).

Every packet's size in the captures was then checked against the table (`questkit chronofoil --index` warns about
packets that aren't the size the table says). Regenerate with `build(...)` (needs a full clone of FFXIVOpcodes and
Kawari's history back to the commits below); new versions only need `questkit chronofoil --save-opcodes` before
upstream's next patch update is merged.
"""

from __future__ import annotations

import json
import subprocess
from collections import Counter, defaultdict

# Kawari commits whose opcodes.yml is that game version's (SUPPORTED_GAME_VERSION in constants.yml).
KAWARI_COMMITS = {
    "2026.06.10.0000.0000": "964cc04",  # 7.51h
    "2026.09.15.0000.0000": "31bdd47",  # 7.56h
}
# FFXIVOpcodes commits by patch name.
FFXIVOPCODES_COMMITS = {"7.51h": "f38d969", "7.51h2": "517bec8", "7.56": "040fafa", "7.56h": "38a7415"}
# version: (Kawari's version it's shifted from, its FFXIVOpcodes patch, this version's FFXIVOpcodes patch)
SHIFTED = {
    "2026.06.18.0000.0000": ("2026.06.10.0000.0000", "7.51h", "7.51h2"),
    "2026.09.01.0000.0000": ("2026.09.15.0000.0000", "7.56h", "7.56"),
}
# Picked by their contents in Bob's captures (2026-10-17): DirectorVars starts with the director's ID.
CONTENT_PICKS = {
    "2026.06.18.0000.0000": {("ServerZoneIpcType", "DirectorVars"): (208, 24)},
    "2026.09.01.0000.0000": {("ServerZoneIpcType", "DirectorVars"): (668, 24)},
}


def _git_show(repo: str, rev: str, path: str) -> str:
    return subprocess.run(["git", "-C", repo, "show", f"{rev}:{path}"], capture_output=True, text=True,
                          check=True).stdout


def kawari_table(repo: str, rev: str) -> dict:
    import yaml

    table = {}
    for kind, entries in yaml.safe_load(_git_show(repo, rev, "resources/data/opcodes.yml")).items():
        for entry in entries or []:
            if entry.get("opcode") is not None:
                table[(kind, entry["name"])] = (entry["opcode"], entry.get("size"))
    return table


def ffxivopcodes_table(repo: str, rev: str) -> dict:
    data = json.loads(_git_show(repo, rev, "opcodes.json"))
    lists = next(e for e in data if e["region"] == "Global")["lists"]
    return {(kind, e["name"]): e["opcode"] for kind, entries in lists.items() for e in entries}


def shift(base: dict, frm: dict, to: dict) -> dict:
    by_opcode = defaultdict(dict)
    for (kind, name), opcode in frm.items():
        by_opcode[kind][opcode] = name
    out = {}
    for (kind, name), (opcode, size) in base.items():
        theirs = by_opcode[kind].get(opcode)
        if theirs and (kind, theirs) in to:
            out[(kind, name)] = (to[(kind, theirs)], size)
    return out


def infer_by_size(table: dict, reference: dict, seen: dict) -> dict:
    """seen: {(kind, opcode): Counter(size)} from captures of this version."""
    used = {(kind, opcode) for (kind, _), (opcode, _) in table.items()}
    free = defaultdict(list)
    for (kind, opcode), sizes in seen.items():
        if (kind, opcode) in used:
            continue
        size, count = sizes.most_common(1)[0]
        if count >= 0.9 * sum(sizes.values()):
            free[(kind, size)].append(opcode)
    shared = Counter((kind, size) for (kind, _), (_, size) in reference.items())
    out = {}
    for (kind, name), (_, size) in reference.items():
        if (kind, name) in table or size is None or shared[(kind, size)] != 1:
            continue
        candidates = free.get((kind, size), [])
        if len(candidates) == 1:
            out[(kind, name)] = (candidates[0], size)
    return out


def seen_sizes(captures, version: str) -> dict:
    from .chronofoil import KINDS

    seen = defaultdict(Counter)
    for capture in captures:
        if capture.game_version != version:
            continue
        for segment in capture.segments():
            kind = KINDS.get((segment.protocol, segment.dir))
            if kind:
                seen[(kind, segment.opcode)][len(segment.data)] += 1
    return seen


def build(kawari_repo: str, ffxivopcodes_repo: str, captures, reference: dict) -> dict:
    """{version: {kind: {opcode: [name, size, how]}}}. `reference` is Kawari's current table ((kind, name) ->
    (opcode, size)), used for the sizes of packets the older tables don't have."""
    versions: dict = {}
    kawari = {version: kawari_table(kawari_repo, rev) for version, rev in KAWARI_COMMITS.items()}
    theirs = {patch: ffxivopcodes_table(ffxivopcodes_repo, rev) for patch, rev in FFXIVOPCODES_COMMITS.items()}
    for version, table in kawari.items():
        versions[version] = {key: (*value, f"kawari@{KAWARI_COMMITS[version]}") for key, value in table.items()}
    for version, (base, frm, to) in SHIFTED.items():
        table = {key: (*value, f"shifted {frm}->{to}") for key, value in
                 shift(kawari[base], theirs[frm], theirs[to]).items()}
        for key, value in CONTENT_PICKS.get(version, {}).items():
            table[key] = (*value, "contents")
        plain = {key: value[:2] for key, value in table.items()}
        for key, value in infer_by_size(plain, reference, seen_sizes(captures, version)).items():
            table[key] = (*value, "size")
        versions[version] = table
    out = {}
    for version, table in versions.items():
        kinds = defaultdict(dict)
        for (kind, name), (opcode, size, how) in table.items():
            kinds[kind][str(opcode)] = [name, size, how]
        out[version] = {kind: dict(sorted(entries.items(), key=lambda e: int(e[0]))) for kind, entries in kinds.items()}
    return out
