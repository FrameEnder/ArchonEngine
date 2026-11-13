"""The fork's quest scripts against Questionable's quest paths (github.com/PunishXIV/Questionable, AGPL-3.0).

Questionable is a Dalamud plugin that does quests; its `QuestPaths/` hold, for 4,389 quests, each sequence's steps:
whom or what is talked to or used (`DataId`: an ENpc or EObj), where (`Position`, `TerritoryId`), what's done there
(`InteractionType`), the key item used (`ItemId`), the enemies fought, and the quest's variables as its authors saw
them on retail (`CompletionQuestVariablesFlags`, and comments starting "QW:"). Its files hold the game's text (quest
and NPC names), so nothing of it goes in git: this reads a checkout (`$KAWARI_DATA/questionable`, as the world bots
do) and writes its report outside the repository. Only numbers come out of it.

What's checked, for each quest script that has a path:

* every ENpc and EObj Questionable talks to or uses is one of the script's constants (ACTOR0, EOBJECT2 ...) that the
  script refers to, with the quest's values from its fixtures (`tools/questkit/fixtures/<quest>*.json`, `params`) or
  a quest dump (`quest-dumps/*/quest.json`);
* every key item Questionable uses is one of the script's ITEM constants;
* every sequence Questionable goes through is one the script's header speaks of ("Sequence 1: ...").

A finding is something to look at, not a fault: Questionable takes the shortest way, so a script can rightly use
more than it does; and a quest with no known values can't have its targets checked (the report says which).
"""

from __future__ import annotations

import json
import re
from dataclasses import dataclass, field
from pathlib import Path

from . import repo

# Talking to or using something there (Questionable's InteractionType); the rest (WalkTo, Jump, Duty ...) aren't.
TARGETED = {"AcceptQuest", "CompleteQuest", "Interact", "UseItem", "Combat", "EquipItem", "Emote", "Say",
            "Action", "SinglePlayerDuty", "AttuneObject", "Snipe", "StatusOff"}
# ENpc and EObj rows: what a quest script names with ACTOR and EOBJECT.
ENPC = range(1_000_000, 2_000_000)
EOBJ = range(2_000_000, 3_000_000)
CONSTANT = re.compile(r"\b([A-Z][A-Z_]*?\d+)\b")
SEQUENCE = re.compile(r"\b[Ss]equences? ((?:\d+(?:\s*(?:-|–|to|and|,|or)\s*)?)+)")
QUEST_FILE = re.compile(r"^(\d+)_")


@dataclass
class Step:
    sequence: int
    interaction: str
    data_id: int | None
    item: int | None
    territory: int | None
    position: tuple[float, float, float] | None
    variables: list | None = None
    note: str | None = None
    # Not this quest's business: taking up or handing in another quest (PickUpQuestId, TurnInQuestId), or getting
    # somewhere (TargetTerritoryId: a door, a ferry, an aethernet shard.)
    elsewhere: bool = False


@dataclass
class Path_:
    quest: int
    steps: list[Step] = field(default_factory=list)

    @property
    def sequences(self) -> list[int]:
        return sorted({s.sequence for s in self.steps})


def read_paths(checkout: str | Path) -> dict[int, Path_]:
    """Every quest path in a checkout of Questionable (its `QuestPaths/`, or the checkout itself.)"""
    root = Path(checkout)
    if (root / "QuestPaths").is_dir():
        root = root / "QuestPaths"
    paths: dict[int, Path_] = {}
    for file in root.rglob("*.json"):
        match = QUEST_FILE.match(file.name)
        if not match:
            continue
        try:
            data = json.loads(file.read_text(encoding="utf-8-sig"))
        except (OSError, ValueError):
            continue
        path = Path_(int(match.group(1)))
        for sequence in data.get("QuestSequence", []):
            for step in sequence.get("Steps", []):
                position = step.get("Position")
                note = step.get("$")
                path.steps.append(Step(
                    sequence=int(sequence.get("Sequence", 0)),
                    interaction=step.get("InteractionType", ""),
                    data_id=step.get("DataId"),
                    item=step.get("ItemId"),
                    territory=step.get("TerritoryId"),
                    position=(position["X"], position["Y"], position["Z"]) if position else None,
                    variables=step.get("CompletionQuestVariablesFlags"),
                    # Only the quest variables they saw ("QW: 16 0 1 16 0 16"), never the comments' words.
                    note=note if isinstance(note, str) and re.fullmatch(r"(QW: )?[\d ]+", note) else None,
                    elsewhere=any(key in step for key in ("PickUpQuestId", "TurnInQuestId", "TargetTerritoryId")),
                ))
        paths[path.quest] = path
    return paths


def known_params(quest: int, dumps: Path | None = None) -> dict[str, int]:
    """The quest's constants' values from its fixtures and, if there is one, its quest dump."""
    params: dict[str, int] = {}
    for fixture in sorted(repo.FIXTURES.glob(f"{quest}.json")) + sorted(repo.FIXTURES.glob(f"{quest}-*.json")):
        try:
            params.update({k: v for k, v in json.loads(fixture.read_text()).get("params", {}).items()
                           if isinstance(v, int)})
        except (OSError, ValueError):
            continue
    if dumps and dumps.is_dir():
        for dump in dumps.glob(f"*_{quest:05}/quest.json"):
            try:
                for param in json.loads(dump.read_text()).get("params", []):
                    if isinstance(param.get("value"), int):
                        params.setdefault(param["name"], param["value"])
            except (OSError, ValueError):
                continue
    return params


def quest_scripts() -> dict[int, Path]:
    """Every quest script by quest number (its file name's last five digits.)"""
    scripts = {}
    for file in repo.QUEST_SCRIPTS.glob("*/*.lua"):
        match = re.search(r"_(\d{5})$", file.stem)
        if match:
            scripts[int(match.group(1))] = file
    return scripts


def header(text: str) -> str:
    """The script's leading comment block."""
    lines = []
    for line in text.splitlines():
        if line.startswith("--"):
            lines.append(line)
        elif line.strip():
            break
    return "\n".join(lines)


def header_sequences(text: str) -> set[int]:
    """The sequences a script's header speaks of: "Sequence 1:", "Sequences 2, 4, 6:", "Sequences 1-3"."""
    found: set[int] = set()
    for listed in SEQUENCE.findall(header(text)):
        for part in re.split(r"\s*(?:,|and|or)\s*", listed):
            bounds = re.split(r"\s*(?:-|–|to)\s*", part.strip())
            numbers = [int(x) for x in bounds if x.isdigit()]
            if len(numbers) == 2 and 0 <= numbers[1] - numbers[0] <= 32:
                found.update(range(numbers[0], numbers[1] + 1))
            else:
                found.update(numbers)
    return found


@dataclass
class Finding:
    quest: int
    script: str
    kind: str
    detail: str


def check_quest(quest: int, script: Path, path: Path_, dumps: Path | None = None) -> tuple[list[Finding], bool]:
    """The findings for one quest, and whether its targets could be checked (its constants' values known.)"""
    text = script.read_text(encoding="utf-8")
    params = known_params(quest, dumps)
    used = {name for name in CONSTANT.findall(text)}
    values = {value for name, value in params.items() if name in used}
    findings = []

    def note(kind: str, detail: str) -> None:
        findings.append(Finding(quest, script.stem, kind, detail))

    if params:
        for step in path.steps:
            if step.interaction not in TARGETED or not step.data_id or step.elsewhere:
                continue
            if step.data_id not in ENPC and step.data_id not in EOBJ:
                continue
            if step.data_id in values:
                continue
            named = [name for name, value in params.items() if value == step.data_id]
            if named:
                note("target unused", f"sequence {step.sequence}: {step.interaction} {step.data_id} is "
                                      f"{', '.join(named)}, which the script never refers to")
            else:
                note("target unknown", f"sequence {step.sequence}: {step.interaction} {step.data_id} is none of "
                                       f"the quest's constants")
        items = {value for name, value in params.items() if name.startswith("ITEM") and name in used}
        for step in path.steps:
            # (One used on nothing is the player's own: Gysahl greens for the companion, a ticket, a coffer.)
            if not step.item or step.interaction != "UseItem" or step.elsewhere or step.item in items \
                    or not step.data_id:
                continue
            # The quest's own item (ITEM0, RITEM0 ...) that the script never mentions; an item used on a quest
            # target that's none of the quest's. Others (a teleport ticket, opening a reward coffer) aren't its.
            named = [name for name, value in params.items() if value == step.item]
            if named and not any(name in used for name in named):
                note("item", f"sequence {step.sequence}: uses item {step.item} ({', '.join(named)}), which the "
                             f"script never refers to")
            elif not named and step.data_id and items:
                note("item", f"sequence {step.sequence}: uses item {step.item} on {step.data_id}; it's none of the "
                             f"script's ITEMs")
    spoken = header_sequences(text)
    if spoken:
        for sequence in path.sequences:
            if sequence not in spoken:
                note("sequence", f"Questionable goes through sequence {sequence}; the script's header doesn't "
                                 f"speak of it (it does of {sorted(spoken)})")
    return findings, bool(params)


def check(checkout: str | Path, only: list[int] | None = None, dumps: Path | None = None) -> dict:
    paths = read_paths(checkout)
    scripts = quest_scripts()
    dumps = dumps or (repo.ROOT / "quest-dumps")
    findings: list[Finding] = []
    checked, unchecked, no_path = [], [], []
    for quest, script in sorted(scripts.items()):
        if only and quest not in only:
            continue
        path = paths.get(quest)
        if not path:
            no_path.append(quest)
            continue
        found, targets = check_quest(quest, script, path, dumps)
        findings += found
        (checked if targets else unchecked).append(quest)
    return {"paths": len(paths), "checked": checked, "unchecked": unchecked, "no_path": no_path,
            "findings": findings}


def report(result: dict, everything: bool = False) -> str:
    lines = ["# Quest scripts against Questionable's paths", ""]
    lines.append(f"{result['paths']} paths; {len(result['checked'])} scripts checked with their constants' values, "
                 f"{len(result['unchecked'])} only by sequence (no fixture or dump gives their values), "
                 f"{len(result['no_path'])} with no path.")
    lines.append("")
    by_quest: dict[int, list[Finding]] = {}
    for finding in result["findings"]:
        by_quest.setdefault(finding.quest, []).append(finding)
    for quest in sorted(by_quest):
        items = by_quest[quest]
        lines.append(f"## {quest} ({items[0].script})")
        for finding in items:
            lines.append(f"* {finding.kind}: {finding.detail}")
        lines.append("")
    if result["unchecked"]:
        lines.append("## Not checked by target (no constants' values: a quest dump or a fixture gives them)")
        lines.append(" ".join(str(q) for q in result["unchecked"]))
        lines.append("")
    if result["no_path"] and everything:
        lines.append("## No Questionable path")
        lines.append(" ".join(str(q) for q in result["no_path"]))
    return "\n".join(lines) + "\n"


def retail_hints(checkout: str | Path, quest: int) -> list[str]:
    """For a quest about to be scripted: Questionable's steps by sequence, numbers only (targets, items, the quest
    variables seen on retail, positions)."""
    path = read_paths(checkout).get(quest)
    if not path:
        return [f"No Questionable path for {quest}."]
    out = []
    for step in path.steps:
        if step.interaction == "WalkTo" and not step.data_id:
            continue
        where = f" at ({step.position[0]:.1f}, {step.position[1]:.1f}, {step.position[2]:.1f}) in {step.territory}" \
            if step.position else ""
        extra = []
        if step.item:
            extra.append(f"item {step.item}")
        if step.variables and any(v for v in step.variables if v):
            extra.append(f"variables after {step.variables}")
        if step.note:
            extra.append(step.note)
        out.append(f"sequence {step.sequence}: {step.interaction} {step.data_id or ''}{where}"
                   + (f" ({'; '.join(extra)})" if extra else ""))
    return out
