"""questkit lint: the mistakes that fail silently in game, found in the repository before anyone plays.

* An action an NPC uses with no action script: the server only logs "Action N isn't scripted yet!" and the NPC does
  nothing (actions in timelines: auto-attack, timepoints, rotations, recorded amounts, on death; and the ACTION_*
  numbers in quest battle scripts).
* A battle NPC in a quest battle's drop-in layer (QB_*) with no timeline: it fights with the defaults (auto-attack 870,
  a 0.5 yalm hitbox, stat-based damage) whatever retail did.
* A fork quest script (its header says what it was "Checked against") with no replay fixture, or a quest battle script
  with no battle fixture, so CI doesn't hold it to retail.
* A fixture step still marked "todo".

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


def format_report(report: Report) -> str:
    lines = [f"error: {e}" for e in report.errors] + [f"note: {n}" for n in report.notes]
    lines.append("lint: ok" if report.ok else f"lint: {len(report.errors)} errors")
    return "\n".join(lines)
