"""Replay fixtures: what a quest did on retail, step by step, as numbers only.

A fixture (tools/questkit/fixtures/<quest>.json) is made from a Kawari Recorder export and the quest's dump
(kawari-questdump), and holds no game text, so it can be committed:

    {
      "quest": 387, "script": "SubFst073_00387",
      "source": "...where the recording came from...",
      "params": {"ACTOR0": 1000503, ...},          # the Quest sheet's Lua parameters (numbers)
      "steps": [
        {"talk": 1000503, "expect": {"sequence": 1, "vars": "000000000000", "notices": []}},
        {"kill": [12, 3838969], "expect": {...}},
        {"range": 3842570},                         # walking into an event range
        {"enter_territory": 148, "expect": {...}},  # zoning in (quests that play a scene on arrival)
        {"battle_won": 4, "expect": {...}},         # a quest battle's director winning it
        {"talk": 1000470, "results": {"24": [1, 0]}, "expect": {"completed": true}}
      ]
    }

`expect` is the quest's state after the step: its sequence and its six variable bytes (UI8A..UI8F) as hex, or
"completed". `notices` is the notices the step sends, as [notice, [args]]; it's left out (not checked) where the
recording didn't capture packets. A step without `expect` only has to run without errors.

`results` sets what the client answers to a scene (every scene answers [1, 0] otherwise): a list, or
{"results": [...], "error": n} for a scene whose answer is its error code (50 is "No" in The Gridanian Envoy's
boarding yes/nos; retail's "Yes" there is error 0 with results [0, 0]).

Recorder 2.0 and later record who the player last targeted, which says what each step was. Older recordings don't,
so their steps come out as {"todo": ...} placeholders to fill in by hand.
"""

from __future__ import annotations

import json
from pathlib import Path

from . import repo
from .recording import Recordings

MERGE_SECONDS = 3.0
NOTICE_LEAD = 0.5  # notices arrive just before the quest change they go with


def load_dump(dumps: Path, quest: int) -> dict | None:
    """A quest's quest.json from a kawari-questdump output folder (quest-dumps/<script>/quest.json)."""
    dumps = Path(dumps)
    for path in sorted(dumps.glob("*/quest.json")):
        try:
            data = json.loads(path.read_text())
        except json.JSONDecodeError:
            continue
        if data.get("quest_id") == quest:
            return data
    return None


def numeric_params(dump: dict) -> dict[str, int]:
    params = {}
    for param in dump.get("params", []):
        value = param.get("value")
        if isinstance(value, int) and isinstance(param.get("name"), str):
            params[param["name"]] = value
    return params


def _group(events):
    """Recorded changes that came from one action: the same target, close together."""
    groups = []
    for event in events:
        if groups:
            last = groups[-1][-1]
            same_target = event.target_layout and event.target_layout == last.target_layout
            if same_target and event.time - last.time <= MERGE_SECONDS and event.kind != "completed":
                groups[-1].append(event)
                continue
        groups.append([event])
    return groups


def build(quest: int, recordings: Recordings, dump: dict, source: str = "") -> dict:
    events = recordings.quest_events(quest)
    if not events:
        raise ValueError(f"quest {quest} isn't in the recording")
    notices, covered = recordings.notices(quest)
    params = numeric_params(dump)
    groups = _group(events)

    # Each notice goes with the first change that comes after it (or with it.)
    assigned: list[list] = [[] for _ in groups]
    for notice in notices:
        for index, group in enumerate(groups):
            if group[-1].time >= notice.time - NOTICE_LEAD:
                assigned[index].append(notice.as_list())
                break

    battle = "QUESTBATTLE0" in params
    by_value = {}
    for name, value in params.items():
        by_value.setdefault(value, []).append(name)

    steps = []
    for index, group in enumerate(groups):
        last = group[-1]
        first = group[0]
        step: dict = {}
        kind = first.target_kind
        fresh = first.target_seconds_ago is not None and first.target_seconds_ago <= 10 and first.target_base
        if kind in ("EventNpc", "EventObj") and fresh:
            step["talk"] = first.target_base
        elif kind == "BattleNpc" and fresh:
            step["kill"] = [first.target_base, first.target_layout]
        elif battle and last.kind == "sequence" and index > 0 and steps and \
                last.raw.get("TerritoryId") != groups[index - 1][-1].raw.get("TerritoryId"):
            step["battle_won"] = last.sequence
        else:
            step["todo"] = (f"{last.kind} in territory {last.raw.get('TerritoryId')}: the recording doesn't say "
                            f"what the player did (talk, kill, range, enter_territory or battle_won)")
        names = by_value.get(step.get("talk")) or by_value.get((step.get("kill") or [None])[0])
        if names:
            step["what"] = "/".join(sorted(names))
        expect: dict = {}
        if last.kind == "completed":
            expect["completed"] = True
        else:
            expect["sequence"] = last.sequence
            expect["vars"] = last.variables.hex()
        if covered:
            expect["notices"] = assigned[index]
        step["expect"] = expect
        steps.append(step)

    return {
        "quest": quest,
        "script": dump.get("script", ""),
        "source": source,
        "params": params,
        "steps": steps,
    }


def fixture_path(quest: int) -> Path:
    return repo.FIXTURES / f"{quest}.json"


def save(fixture: dict, path: Path | None = None) -> Path:
    path = path or fixture_path(fixture["quest"])
    path.parent.mkdir(parents=True, exist_ok=True)
    text = dumps(fixture)
    path.write_text(text + "\n")
    return path


def dumps(fixture: dict) -> str:
    """JSON with lists of numbers ([notice, [args]], kills) kept on one line, so a fixture reads step by step."""
    import re

    text = json.dumps(fixture, indent=2)
    tokens: list[str] = []

    def keep(match):
        tokens.append("[" + ", ".join(part.strip() for part in match.group(1).split(",") if part.strip()) + "]")
        return f"\x00{len(tokens) - 1}\x00"

    while True:
        new = re.sub(r"\[([^\[\]{}\x00]*(?:\x00\d+\x00[^\[\]{}\x00]*)*)\]", keep, text)
        if new == text:
            break
        text = new
    while "\x00" in text:
        text = re.sub(r"\x00(\d+)\x00", lambda m: tokens[int(m.group(1))], text)
    return text


def load(path: Path) -> dict:
    return json.loads(Path(path).read_text())


def all_fixtures() -> list[Path]:
    return sorted(repo.FIXTURES.glob("*.json"), key=lambda p: int(p.stem) if p.stem.isdigit() else 0)
