"""questkit lint: the mistakes that fail silently in game, found in the repository before anyone plays.

* An action an NPC uses with no action script: the server only logs "Action N isn't scripted yet!" and the NPC does
  nothing (actions in timelines: auto-attack, timepoints, rotations, recorded amounts, on death; and the ACTION_*
  numbers in quest battle scripts).
* A battle NPC in a quest battle's drop-in layer (QB_*) with no timeline: it fights with the defaults (auto-attack 870,
  a 0.5 yalm hitbox, stat-based damage) whatever retail did.
* A fork quest script (its header says what it was "Checked against") with no replay fixture, or a quest battle script
  with no battle fixture, so CI doesn't hold it to retail.
* A fixture step still marked "todo".
* A battle NPC in a quest battle's drop-in layer that differs from how retail spawned it (`retail_spawns.json`, made by
  `questkit chronofoil <captures> --spawns`): its BNpcBase, BNpcName, level, HP, character data flags, display flags or
  Mount; and an NPC retail spawned in that territory that no drop-in has. Fool Me Twice's boss rode a Magitek Reaper
  on retail (Mount 14) and stood on foot in Kawari until Bob saw it (bundle 20261114-015635).

Open-world drop-in layers may have NPCs without timelines (most of the Central Shroud's do); those are only counted.
"""

from __future__ import annotations

import json
import re
from dataclasses import dataclass, field
from pathlib import Path

from . import repo

ACTIONS = repo.SCRIPTS / "actions"
TIMELINES = repo.ROOT / "resources" / "timelines"
DROPINS = repo.ROOT / "resources" / "dropins"
QUEST_BATTLES = repo.SCRIPTS / "content" / "questbattle"
BATTLE_FIXTURES = repo.ROOT / "tools" / "questkit" / "battles"
RETAIL_SPAWNS = repo.ROOT / "tools" / "questkit" / "retail_spawns.json"


@dataclass
class Report:
    errors: list[str] = field(default_factory=list)
    notes: list[str] = field(default_factory=list)

    @property
    def ok(self) -> bool:
        return not self.errors


def scripted_actions() -> set[int]:
    """Action IDs with a script (resources/scripts/actions/<id / 100>/<Name>_<id, 5 digits>.lua)."""
    found = set()
    for path in ACTIONS.rglob("*.lua"):
        match = re.search(r"_(\d+)$", path.stem)
        if match:
            found.add(int(match.group(1)))
    return found


def timeline_actions(timeline: dict) -> set[int]:
    actions = set()
    if timeline.get("autoattack_action_id"):
        actions.add(int(timeline["autoattack_action_id"]))
    for point in timeline.get("timepoints", []) + [{"data": d} for d in timeline.get("on_death", [])]:
        data = point.get("data", {})
        if data.get("type") == "action" and data.get("action_id"):
            actions.add(int(data["action_id"]))
    for entry in (timeline.get("rotation") or {}).get("actions", []):
        actions.add(int(entry["action"]))
    actions.update(int(k) for k in timeline.get("recorded_amounts", {}))
    return actions


def timeline_bases() -> dict[int, Path]:
    bases = {}
    for path in TIMELINES.glob("*.json"):
        match = re.search(r"_(\d+)$", path.stem)
        if match:
            bases[int(match.group(1))] = path
    return bases


def run() -> Report:
    report = Report()
    scripted = scripted_actions()

    # Actions NPCs use.
    for path in sorted(TIMELINES.glob("*.json")):
        try:
            timeline = json.loads(path.read_text())
        except json.JSONDecodeError as error:
            report.errors.append(f"{path.relative_to(repo.ROOT)}: not valid JSON ({error})")
            continue
        for action in sorted(timeline_actions(timeline) - scripted):
            report.errors.append(f"{path.relative_to(repo.ROOT)} uses action {action}, which has no script")
    for path in sorted(QUEST_BATTLES.glob("*.lua")):
        for name, value in re.findall(r"^(\w*ACTION\w*)\s*=\s*(?:\w+\s+or\s+)?(\d+)", path.read_text(), re.M):
            if int(value) not in scripted:
                report.errors.append(f"{path.relative_to(repo.ROOT)}: {name} = {value}, which has no script")

    # Battle NPCs without timelines.
    bases = timeline_bases()
    open_world = 0
    for path in sorted(DROPINS.glob("*.json")):
        data = json.loads(path.read_text())
        for layer in data.get("layers", []):
            for obj in layer.get("objects", []):
                info = obj.get("data", {})
                if info.get("type") != "battle_npc" or info.get("base_id") in bases:
                    continue
                if layer["name"].startswith("QB_"):
                    report.errors.append(f"{path.relative_to(repo.ROOT)} layer {layer['name']}: battle NPC "
                                         f"{obj.get('layout_id')} (BNpcBase {info.get('base_id')}) has no timeline")
                else:
                    open_world += 1
    if open_world:
        report.notes.append(f"{open_world} open-world spawn points have no timeline for their kind (defaults)")

    # Quest battle NPCs against how retail spawned them.
    report.errors.extend(spawn_differences())

    # Scripts CI doesn't check against retail.
    fixtures = {int(p.stem) for p in repo.FIXTURES.glob("*.json") if p.stem.isdigit()}
    for path in sorted(repo.QUEST_SCRIPTS.rglob("*.lua")):
        match = re.search(r"_(\d{5})$", path.stem)
        if not match:
            continue
        header = "\n".join(path.read_text().splitlines()[:8])
        if "Checked against" in header and int(match.group(1)) not in fixtures:
            report.errors.append(f"{path.relative_to(repo.ROOT)} says it was checked, but has no fixture "
                                 f"(tools/questkit/fixtures/{int(match.group(1))}.json)")
    battle_fixtures = {int(p.stem) for p in BATTLE_FIXTURES.glob("*.json") if p.stem.isdigit()}
    for path in sorted(QUEST_BATTLES.glob("*.lua")):
        if path.stem.isdigit() and int(path.stem) not in battle_fixtures:
            report.errors.append(f"{path.relative_to(repo.ROOT)} has no battle fixture (questkit battle-sim --make "
                                 f"{path.stem} --run <a retail run>)")

    # Fixtures left unfinished.
    for path in sorted(repo.FIXTURES.glob("*.json")):
        for n, step in enumerate(json.loads(path.read_text()).get("steps", []), 1):
            if "todo" in step:
                report.errors.append(f"{path.relative_to(repo.ROOT)} step {n}: still todo ({step['todo']})")
    return report


def spawn_differences(registry: dict | None = None, dropins: dict | None = None) -> list[str]:
    """Each quest battle drop-in NPC's fields that differ from retail's spawn of it, and the NPCs retail spawned in a
    quest battle's territory that no drop-in has. `registry`: territory -> layout -> fields (`retail_spawns.json`);
    `dropins`: file name -> drop-in (`resources/dropins`)."""
    if registry is None:
        registry = json.loads(RETAIL_SPAWNS.read_text()) if RETAIL_SPAWNS.exists() else {}
    if dropins is None:
        dropins = {p.name: json.loads(p.read_text()) for p in sorted(DROPINS.glob("*.json"))}
    in_dropins: set[int] = set()
    quest_battle: list[tuple[str, str, dict]] = []
    for file, data in dropins.items():
        for layer in data.get("layers", []):
            for obj in layer.get("objects", []):
                in_dropins.add(obj.get("layout_id"))
                if layer["name"].startswith("QB_") and obj.get("data", {}).get("type") == "battle_npc":
                    quest_battle.append((file, layer["name"], obj))

    problems = []
    territories: set[str] = set()
    for file, layer, obj in quest_battle:
        layout = str(obj.get("layout_id"))
        info = obj["data"]
        found = [(zone, spots[layout]) for zone, spots in registry.items() if layout in spots]
        # A layout ID in two territories (copies of one zone): the one with the same BNpcBase.
        found = [f for f in found if f[1]["base"] == info.get("base_id")] or found
        if not found:
            continue
        zone, retail = found[0]
        territories.add(zone)
        ours = {
            "base": info.get("base_id"), "name": info.get("name_id"), "level": info.get("level"),
            "hp": info.get("hp", retail["hp"]),
            "flags": info.get("character_data_flags", 2 if info.get("hostile") else 0),
            "display": info.get("display_flags", 0), "mount": info.get("mount") or 0,
        }
        for key, value in ours.items():
            if value != retail[key]:
                problems.append(f"resources/dropins/{file} layer {layer}: battle NPC {layout} has {key} {value}, "
                                f"retail spawned it with {retail[key]} (territory {zone})")
    for zone in sorted(territories, key=int):
        for layout, retail in registry[zone].items():
            if int(layout) not in in_dropins:
                problems.append(f"territory {zone}: retail spawned battle NPC {layout} (BNpcBase {retail['base']}, "
                                f"BNpcName {retail['name']}), which no drop-in has")
    return problems


# A drop-in battle NPC's field for each of `retail_spawns.json`'s (`spawn_differences`).
SPAWN_FIELDS = {"base": "base_id", "name": "name_id", "level": "level", "hp": "hp", "flags": "character_data_flags",
                "display": "display_flags", "mount": "mount"}


def fix_spawns(registry: dict | None = None) -> dict[str, int]:
    """Writes retail's spawn of each quest battle drop-in NPC into it (the fields `spawn_differences` compares): how
    many NPCs changed in each file. Lines it doesn't change stay as they were (two files write small floats in another
    form than Python's json does)."""
    import difflib

    if registry is None:
        registry = json.loads(RETAIL_SPAWNS.read_text())
    changed: dict[str, int] = {}
    for path in sorted(DROPINS.glob("*.json")):
        text = path.read_text()
        data = json.loads(text)
        count = 0
        for layer in data.get("layers", []):
            if not layer["name"].startswith("QB_"):
                continue
            for obj in layer.get("objects", []):
                info = obj.get("data", {})
                layout = str(obj.get("layout_id"))
                if info.get("type") != "battle_npc":
                    continue
                found = [spots[layout] for spots in registry.values() if layout in spots]
                found = [f for f in found if f["base"] == info.get("base_id")] or found
                if not found:
                    continue
                retail, before = found[0], dict(info)
                for key, field_name in SPAWN_FIELDS.items():
                    value = retail[key]
                    if key in ("mount", "display") and not value:
                        info.pop(field_name, None)
                    else:
                        info[field_name] = value
                info["hostile"] = bool(retail["flags"] & 2)
                if info != before:
                    count += 1
        if not count:
            continue
        normalized = (json.dumps(json.loads(text), indent=4) + "\n").splitlines()
        original = text.splitlines()
        new = (json.dumps(data, indent=4) + "\n").splitlines()
        if len(normalized) == len(original):
            out = []
            for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(None, normalized, new, autojunk=False).get_opcodes():
                out.extend(original[i1:i2] if tag == "equal" else new[j1:j2])
            new = out
        path.write_text("\n".join(new) + ("\n" if text.endswith("\n") else ""))
        changed[path.name] = count
    return changed


def format_report(report: Report) -> str:
    lines = [f"error: {e}" for e in report.errors] + [f"note: {n}" for n in report.notes]
    lines.append("lint: ok" if report.ok else f"lint: {len(report.errors)} errors")
    return "\n".join(lines)
