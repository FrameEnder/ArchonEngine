"""Where things are in the Kawari repository, and the bits of it questkit reads.

Everything here reads the repository itself (scripts, fork_opcodes.yml, the scene flags), so questkit always matches
the code it sits next to.
"""

from __future__ import annotations

import re
from pathlib import Path

# tools/questkit/questkit/repo.py -> the repository root
ROOT = Path(__file__).resolve().parents[3]
SCRIPTS = ROOT / "resources" / "scripts"
QUEST_SCRIPTS = SCRIPTS / "events" / "quest"
FORK_OPCODES = ROOT / "resources" / "data" / "fork_opcodes.yml"
SCENE_FLAGS_RS = ROOT / "core" / "src" / "ipc" / "zone" / "server" / "event_scene.rs"
ACTOR_CONTROL_RS = ROOT / "core" / "src" / "ipc" / "zone" / "server" / "actor_control.rs"
FIXTURES = ROOT / "tools" / "questkit" / "fixtures"

# Quest handler IDs are the quest number plus this (the Quest sheet's starting row.)
QUEST_HANDLER_BASE = 0x10000


def quest_script_path(script: str, quest_id: int) -> Path:
    """resources/scripts/events/quest/<id / 100, 3 digits>/<script>.lua"""
    return QUEST_SCRIPTS / f"{quest_id // 100:03}" / f"{script}.lua"


def find_quest_script(quest_id: int) -> Path | None:
    """The fork's script for a quest, found by the quest number at the end of its name (SubFst073_00387.lua)."""
    folder = QUEST_SCRIPTS / f"{quest_id // 100:03}"
    for path in sorted(folder.glob(f"*_{quest_id:05}.lua")):
        return path
    return None


def scene_flags() -> dict[str, int]:
    """The scene flags Kawari gives every Lua state (SceneFlags in event_scene.rs, registered without a prefix)."""
    flags = {}
    text = SCENE_FLAGS_RS.read_text()
    for name, value in re.findall(r"const (\w+) = (0x[0-9A-Fa-f]+);", text):
        flags[name] = int(value, 16)
    # Combinations of the others, e.g. SET_BASE = Self::NO_DEFAULT_CAMERA.bits() | Self::FADE_OUT.bits() | ...
    for name, value in re.findall(r"const (\w+) = ((?:Self::\w+\.bits\(\)\s*\|?\s*)+);", text):
        flags[name] = 0
        for part in re.findall(r"Self::(\w+)\.bits\(\)", value):
            flags[name] |= flags[part]
    return flags


def fork_opcodes() -> list[dict]:
    """The entries of fork_opcodes.yml: name, opcode, found_on, size. A small reader for this one flat file, so
    questkit needs nothing outside the standard library to read it."""
    entries = []
    current = None
    for line in FORK_OPCODES.read_text().splitlines():
        if line.lstrip().startswith("#"):
            continue
        match = re.match(r"^- name: (\S+)", line)
        if match:
            current = {"name": match.group(1)}
            entries.append(current)
            continue
        match = re.match(r"^  (opcode|size|found_on): (\S+)", line)
        if match and current is not None:
            key, value = match.groups()
            current[key] = int(value) if key in ("opcode", "size") else value
    return entries


def notice_opcodes(game_version: str | None) -> set[int]:
    """The EventNotice opcodes (every size) for a game version, from fork_opcodes.yml. Empty if that version's
    aren't known."""
    found = set()
    for entry in fork_opcodes():
        if entry["name"].startswith("EventNotice") and "opcode" in entry:
            if game_version is None or entry.get("found_on") == game_version:
                found.add(entry["opcode"])
    return found


def actor_control_names() -> dict[int, str]:
    """ActorControl categories by number, from Kawari's ActorControlCategory (each variant's brw magic)."""
    try:
        text = ACTOR_CONTROL_RS.read_text()
    except OSError:
        return {}
    start = text.find("pub enum ActorControlCategory")
    if start < 0:
        return {}
    names: dict[int, str] = {}
    magic = None
    for line in text[start:].splitlines()[1:]:
        if line.startswith("}"):
            break
        m = re.match(r"\s*#\[brw\(magic = (\d+)u32\)\]", line)
        if m:
            magic = int(m.group(1))
            continue
        m = re.match(r"\s{4}([A-Z][A-Za-z0-9]*)\b", line)
        if m and magic is not None:
            names.setdefault(magic, m.group(1))
            magic = None
    return names
