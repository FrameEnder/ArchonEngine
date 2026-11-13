"""The fork's duty scripts against AutoDuty's duty paths (github.com/ffxivcode/AutoDuty).

AutoDuty is a Dalamud plugin that runs duties; its `AutoDuty/Paths/(<TerritoryType>) <name>.json` hold, for 254
duties, a player's way through: where to walk, the objects to use in order (`Interactable`, with the EObj's ID: a
switch, a key, the door it opens), the keys needed (`ConditionAction` "ItemCount;<item>;..."), where each boss is
fought (`Boss`), and the treasure coffers on the way (`TreasureCoffer`). It's unlicensed (its README: default
copyright), so nothing of it goes in git, extract or otherwise: this reads a checkout and writes reports and drafts
outside the repository, numbers only.

What it gives:

* a check of each of the fork's duty scripts (the table in `docs/src/fork/dungeons.md`: territory and script): every
  EObj AutoDuty uses and every key item it waits on is a number the script knows;
* a draft for a duty not yet scripted: the objects in order with where they are, where the bosses are fought, the
  coffers, the keys. It's one player's route (where they stood, not where enemies are) and doesn't replace a recording.
"""

from __future__ import annotations

import json
import re
from dataclasses import dataclass, field
from pathlib import Path

from . import repo

FILE = re.compile(r"^\((\d+)\) (.*)\.json$")
EOBJ = range(2_000_000, 3_000_000)
ITEM_COUNT = re.compile(r"ItemCount;(\d+)")
NUMBER = re.compile(r"\b(\d{3,8})\b")


@dataclass
class Action:
    name: str
    tag: str
    position: tuple[float, float, float] | None
    arguments: list[str]

    @property
    def eobj(self) -> int | None:
        for argument in self.arguments:
            if argument.strip().isdigit() and int(argument) in EOBJ:
                return int(argument)
        return None


@dataclass
class DutyPath:
    territory: int
    file: str
    actions: list[Action] = field(default_factory=list)

    def of(self, *names: str) -> list[Action]:
        return [a for a in self.actions if a.name in names]

    @property
    def eobjs(self) -> list[int]:
        out = []
        for action in self.of("Interactable"):
            if action.eobj and action.eobj not in out:
                out.append(action.eobj)
        return out

    @property
    def key_items(self) -> list[int]:
        out = []
        for action in self.of("ConditionAction"):
            for argument in action.arguments:
                for item in ITEM_COUNT.findall(argument):
                    if int(item) not in out:
                        out.append(int(item))
        return out


def _position(action: dict) -> tuple[float, float, float] | None:
    p = action.get("Position") or action.get("position")
    if not p:
        return None
    x, y, z = (p.get(k, p.get(k.lower(), 0.0)) for k in ("X", "Y", "Z"))
    return None if (x, y, z) == (0.0, 0.0, 0.0) else (x, y, z)


def read_paths(checkout: str | Path) -> dict[int, list[DutyPath]]:
    """Every duty path in a checkout (its `AutoDuty/Paths`, or that folder itself), by TerritoryType; a duty's main
    path first (the variants for other roles or wall-to-wall pulls have a 「...」 or "W2W" in their names)."""
    root = Path(checkout)
    for sub in ("AutoDuty/Paths", "Paths"):
        if (root / sub).is_dir():
            root = root / sub
            break
    paths: dict[int, list[DutyPath]] = {}
    for file in sorted(root.glob("*.json")):
        match = FILE.match(file.name)
        if not match:
            continue
        try:
            data = json.loads(file.read_text(encoding="utf-8-sig"))
        except (OSError, ValueError):
            continue
        path = DutyPath(int(match.group(1)), file.name)
        for action in data.get("Actions") or data.get("actions") or []:
            path.actions.append(Action(
                name=str(action.get("Name", action.get("name", ""))),
                tag=str(action.get("Tag", action.get("tag", ""))),
                position=_position(action),
                arguments=[str(a) for a in (action.get("Arguments") or action.get("arguments") or [])],
            ))
        paths.setdefault(path.territory, []).append(path)
    for variants in paths.values():
        variants.sort(key=lambda p: ("「" in p.file or "W2W" in p.file, p.file))
    return paths


def fork_duties() -> dict[int, Path]:
    """The fork's duty scripts by TerritoryType, from `docs/src/fork/dungeons.md`'s table."""
    table = (repo.ROOT / "docs" / "src" / "fork" / "dungeons.md").read_text(encoding="utf-8")
    duties = {}
    for line in table.splitlines():
        cells = [c.strip() for c in line.split("|")]
        if len(cells) < 5:
            continue
        script = re.search(r"`content/([\w/]+\.lua)`", line)
        territory = next((int(c) for c in cells if c.isdigit() and 100 <= int(c) <= 9999), None)
        if script and territory:
            duties[territory] = repo.SCRIPTS / "content" / script.group(1)
    return duties


def check(checkout: str | Path, only: list[int] | None = None) -> dict:
    paths = read_paths(checkout)
    results = []
    for territory, script in sorted(fork_duties().items()):
        if only and territory not in only:
            continue
        if not script.exists() or territory not in paths:
            results.append({"territory": territory, "script": script.name, "path": None, "missing": [], "keys": []})
            continue
        path = paths[territory][0]
        known = {int(n) for n in NUMBER.findall(script.read_text(encoding="utf-8"))}
        results.append({
            "territory": territory, "script": script.name, "path": path.file,
            "eobjs": path.eobjs, "missing": [e for e in path.eobjs if e not in known],
            "keys": [k for k in path.key_items if k not in known],
            "bosses": len(path.of("Boss")), "coffers": len(path.of("TreasureCoffer")),
        })
    return {"paths": len(paths), "duties": results}


def report(result: dict) -> str:
    lines = ["# Duty scripts against AutoDuty's paths", "",
             f"{result['paths']} duties have paths. For each of the fork's: the EObjs AutoDuty uses and the key items "
             "it waits on that the script never names (a door, switch or key the script may not handle).", ""]
    for duty in result["duties"]:
        if not duty["path"]:
            lines.append(f"* {duty['territory']} `{duty['script']}`: no AutoDuty path")
            continue
        state = "all known" if not duty["missing"] and not duty["keys"] else ""
        lines.append(f"* {duty['territory']} `{duty['script']}`: {len(duty['eobjs'])} objects, {duty['bosses']} "
                     f"bosses, {duty['coffers']} coffers on the way; {state}")
        if duty["missing"]:
            lines.append(f"  * objects the script never names: {duty['missing']}")
        if duty["keys"]:
            lines.append(f"  * key items the script never names: {duty['keys']}")
    return "\n".join(lines) + "\n"


def draft(checkout: str | Path, territory: int) -> str:
    """A duty's route from AutoDuty, numbers only: for starting a duty that isn't scripted yet."""
    paths = read_paths(checkout).get(territory)
    if not paths:
        return f"No AutoDuty path for territory {territory}.\n"
    path = paths[0]

    def at(action: Action) -> str:
        return f"({action.position[0]:.1f}, {action.position[1]:.1f}, {action.position[2]:.1f})" \
            if action.position else "(no position)"

    lines = [f"# Territory {territory}: AutoDuty's route", "",
             "One player's way through (AutoDuty, github.com/ffxivcode/AutoDuty, unlicensed: keep this out of git). "
             "Positions are where they stood, not where enemies are.", "", "## In order", ""]
    boss = 0
    for action in path.actions:
        if action.name == "Interactable" and action.eobj:
            lines.append(f"* object {action.eobj} at {at(action)}")
        elif action.name == "ConditionAction":
            items = [int(x) for a in action.arguments for x in ITEM_COUNT.findall(a)]
            if items:
                lines.append(f"* needs key item {', '.join(map(str, items))} here")
        elif action.name == "Boss":
            boss += 1
            spot = next((a for a in action.arguments if a.startswith("<")), "")
            lines.append(f"* boss {boss} fought at {at(action)}" + (f", arena {spot}" if spot else ""))
        elif action.name == "TreasureCoffer":
            lines.append(f"* treasure coffer at {at(action)}")
        elif action.name in ("Revival",):
            lines.append(f"* where a fallen player comes back: {at(action)}")
    lines += ["", f"Objects: {path.eobjs}", f"Key items: {path.key_items}", ""]
    return "\n".join(lines)
