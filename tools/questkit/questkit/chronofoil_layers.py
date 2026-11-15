"""Layer filter keys in Chronofoil captures, checked against `resources/data/layer_filters.json`.

Retail's server tells the client which of a zone's layer filters to show (ZoneInit's and ActorSetPos'
`transition_territory_filter_key`), by where the story is; Kawari picks it from the table. Every key retail sent is
an observation the table has to give back: the territory, the key, and the quests as the character had them at that
moment (accepted with their sequence, completed). `observe` reads them out of a capture; `check` asks the table (as
the server's `layer_filters.rs` reads it) for each and says where it gives another key. The observations are numbers
only, kept in `tools/questkit/questkit/data/layer_keys.json` so the check runs without the captures
(`questkit chronofoil <captures> --layers --save` adds a capture's; the test suite runs the check.)

Bob, 2026-11-15: The Black Wolf's Ultimatum (522) didn't move the Waking Sands on. Captures of the quest had been
in hand for three days with the key in them; nothing compared them with the table.

Layouts (Kawari's, core/src/ipc/zone): ZoneInit territory u16 @2, key u32 @8. ActorSetPos key u32 @4.
QuestActiveList: 30 of (quest u16, sequence u8, …) 16 bytes each. QuestCompleteList: a bitmask, quest `q` at byte
`q // 8`, bit `0x80 >> (q % 8)`. AcceptQuest / FinishQuest: quest u16. UpdateQuest: quest u16 @4, sequence u8 @6.
"""

from __future__ import annotations

import json
import struct
from pathlib import Path

from . import repo
from .chronofoil import Names

OBSERVATIONS = Path(__file__).resolve().parent / "data" / "layer_keys.json"
COMPLETED_BYTES = 750  # at least; only as much as the packet has is read


def observe(capture, tables: dict) -> list[dict]:
    """Each time a capture's server named a layer filter: where, which, and the quests then."""
    names = Names(tables, capture.game_version)
    active: dict[int, int] = {}
    completed: set[int] = set()
    know_completed = False
    territory = 0
    last = None
    seen: list[dict] = []

    def note(key: int, where: str, t: float):
        nonlocal last
        state = (territory, key, last)
        if seen and seen[-1]["_state"] == state:
            return
        seen.append({
            "capture": capture.id[:8],
            "territory": territory,
            "key": key,
            "where": where,
            "active": dict(sorted(active.items())),
            "completed": sorted(completed) if know_completed else None,
            "after": list(last) if last else None,
            "_state": state,
        })

    for segment in capture.segments(protocols=("zone",)):
        if segment.dir != "out":
            continue
        name = names.name(segment)
        d = segment.data
        if name == "ZoneInit" and len(d) >= 12:
            territory = struct.unpack_from("<H", d, 2)[0]
            key = struct.unpack_from("<I", d, 8)[0]
            if key:
                note(key, "ZoneInit", segment.t)
        elif name == "ActorSetPos" and len(d) >= 8 and segment.src == segment.dst:
            key = struct.unpack_from("<I", d, 4)[0]
            if key:
                note(key, "ActorSetPos", segment.t)
        elif name == "QuestActiveList":
            active = {}
            for i in range(0, min(len(d), 30 * 16), 16):
                quest, sequence = struct.unpack_from("<HB", d, i)
                if quest:
                    active[quest] = sequence
        elif name == "QuestCompleteList":
            know_completed = True
            completed = {byte * 8 + bit for byte in range(min(len(d), COMPLETED_BYTES)) for bit in range(8)
                         if d[byte] & (0x80 >> bit)}
        elif name == "AcceptQuest" and len(d) >= 2:
            quest = struct.unpack_from("<H", d, 0)[0]
            active.setdefault(quest, 0)
            last = ("accept", quest, active[quest])
        elif name == "UpdateQuest" and len(d) >= 7:
            quest, sequence = struct.unpack_from("<H", d, 4)[0], d[6]
            if quest:
                active[quest] = sequence
                last = ("update", quest, sequence)
        elif name == "FinishQuest" and len(d) >= 2:
            quest = struct.unpack_from("<H", d, 0)[0]
            active.pop(quest, None)
            completed.add(quest)
            last = ("finish", quest, 255)
    for entry in seen:
        del entry["_state"]
    return seen


def table_key(territory: int, active: dict[int, int], completed, entries=None) -> int:
    """The key the server gives (`layer_filters.rs`: the last entry whose quest has been reached.)"""
    if entries is None:
        entries = repo.layer_filters().get(territory, [])
    completed = set(completed or ())
    key = 0
    for entry in entries:
        quest = entry.get("quest")
        if quest is not None:
            done = quest in completed
            open_far_enough = quest in active and (entry.get("sequence") is None
                                                   or active[quest] >= entry["sequence"])
            if not (done or (not entry.get("completed") and open_far_enough)):
                continue
        key = entry["key"]
    return key


def check(observations: list[dict]) -> list[str]:
    """Where the table gives another key than retail did. An observation without the character's completed quests
    (a capture begun after login) is checked only when the table's answer doesn't hang on one."""
    table = repo.layer_filters()
    problems = []
    for seen in observations:
        territory, key = seen["territory"], seen["key"]
        active = {int(quest): sequence for quest, sequence in seen["active"].items()}
        entries = table.get(territory)
        where = f"capture {seen['capture']}, territory {territory}, {seen['where']}"
        after = seen.get("after")
        then = f" after quest {after[1]} {after[0]} (sequence {after[2]})" if after else ""
        if entries is None:
            problems.append(f"{where}: retail sent key {key}{then}; the table has no territory {territory}")
            continue
        if key not in {entry["key"] for entry in entries}:
            problems.append(f"{where}: retail sent key {key}{then}; the table has no entry for it")
            continue
        if seen["completed"] is None:
            continue
        ours = table_key(territory, active, seen["completed"], entries)
        if ours != key:
            problems.append(f"{where}: retail sent key {key}{then}; the table gives {ours}")
    return problems


def load() -> list[dict]:
    if not OBSERVATIONS.exists():
        return []
    return json.loads(OBSERVATIONS.read_text())["observations"]


def merge(old: list[dict], new: list[dict]) -> list[dict]:
    """The kept observations with a capture's read again: its old ones replaced."""
    captures = {seen["capture"] for seen in new}
    kept = [seen for seen in old if seen["capture"] not in captures]
    return sorted(kept + new, key=lambda seen: (seen["territory"], seen["capture"]))


def save(observations: list[dict]):
    OBSERVATIONS.parent.mkdir(parents=True, exist_ok=True)
    text = json.dumps({
        "comment": "Every layer filter key retail's server named in the Chronofoil captures read so far, with the "
                   "quests as the character had them then (numbers only; `questkit chronofoil <captures> --layers "
                   "--save` adds a capture's). `completed` is null where the capture began after login, so the "
                   "completed quests weren't seen. The test suite checks resources/data/layer_filters.json gives "
                   "each key back.",
        "observations": observations,
    }, indent=1)
    OBSERVATIONS.write_text(text + "\n")


def report(observations: list[dict]) -> str:
    lines = []
    for seen in observations:
        after = seen.get("after")
        then = f"after {after[0]} of {after[1]} at {after[2]}" if after else "at login"
        lines.append(f"{seen['capture']}  territory {seen['territory']:>4}  key {seen['key']:>6}  "
                     f"{seen['where']:<11}  {then}")
    return "\n".join(lines)
