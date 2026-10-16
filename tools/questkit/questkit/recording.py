"""Reading Kawari Recorder exports (the zip from /krec export, or a folder it was unpacked to).

What questkit uses from them:
* quests.json: every change to the player's quests on retail (accepted, variables, sequence, completed), with who
  they last targeted, and
* packets.json: the server packets addressed to a quest or director, of which the EventNotice ones are the quest's
  notices (the progress pop-ups) with their arguments.

Recordings contain game text (quest, NPC and zone names). Nothing read from them is committed except numbers.
"""

from __future__ import annotations

import io
import json
import zipfile
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path

from . import repo


def parse_time(text: str) -> float:
    """Recorder timestamps (ISO 8601 with 7 fractional digits and Z) as Unix seconds."""
    text = text.strip()
    if text.endswith("Z"):
        text = text[:-1] + "+00:00"
    # Python only takes up to 6 fractional digits.
    if "." in text:
        head, rest = text.split(".", 1)
        digits = ""
        while rest and rest[0].isdigit():
            digits += rest[0]
            rest = rest[1:]
        text = f"{head}.{digits[:6].ljust(6, '0')}{rest}"
    moment = datetime.fromisoformat(text)
    if moment.tzinfo is None:
        moment = moment.replace(tzinfo=timezone.utc)
    return moment.timestamp()


@dataclass
class QuestEvent:
    """One change to a quest, as recorded on retail."""

    time: float
    kind: str  # accepted, variables, sequence, completed, abandoned
    sequence: int
    variables: bytes  # UI8A..UI8F
    target_base: int | None = None
    target_layout: int | None = None
    target_kind: str | None = None  # EventNpc, EventObj, BattleNpc...
    target_seconds_ago: float | None = None
    exp: int | None = None
    gil: int | None = None
    raw: dict = field(default_factory=dict, repr=False)


@dataclass
class Notice:
    """An EventNotice addressed to a quest: the progress pop-up, or a notice as a quest moves on."""

    time: float
    handler: int
    notice: int
    args: list[int]

    @property
    def quest(self) -> int:
        return self.handler - repo.QUEST_HANDLER_BASE

    def as_list(self) -> list:
        return [self.notice, self.args]


def parse_notice(data: bytes) -> tuple[int, int, list[int]] | None:
    """An EventNotice's data (without the IPC header): handler u32, notice u8, argument count u8, 2 bytes padding,
    then the arguments as u32. Checked byte for byte against retail (see fork_opcodes.yml)."""
    if len(data) < 8:
        return None
    handler = int.from_bytes(data[0:4], "little")
    notice = data[4]
    count = data[5]
    if count > 8 or len(data) < 8 + 4 * count:
        return None
    args = [int.from_bytes(data[8 + 4 * i: 12 + 4 * i], "little") for i in range(count)]
    return handler, notice, args


class Recording:
    def __init__(self, files: dict[str, bytes], source: str):
        self.source = source
        self._files = files
        self.quests: dict[int, dict] = {}
        self.game_version: str | None = None
        self.packets: list[dict] = []
        self._load()

    @classmethod
    def open(cls, path: str | Path) -> "Recording":
        path = Path(path)
        files: dict[str, bytes] = {}
        if path.is_dir():
            for child in path.iterdir():
                if child.suffix in (".json", ".csv"):
                    files[child.name] = child.read_bytes()
        else:
            with zipfile.ZipFile(path) as archive:
                for name in archive.namelist():
                    base = name.rsplit("/", 1)[-1]
                    if base.endswith((".json", ".csv")):
                        files[base] = archive.read(name)
        if "quests.json" not in files and "packets.json" not in files:
            raise ValueError(f"{path} doesn't look like a Kawari Recorder export (no quests.json or packets.json)")
        return cls(files, str(path))

    def json(self, name: str):
        data = self._files.get(name)
        if data is None:
            return None
        return json.load(io.BytesIO(data))

    def _load(self):
        quests = self.json("quests.json") or {}
        for key, quest in (quests.get("Quests") or {}).items():
            self.quests[int(key)] = quest
        packets = self.json("packets.json") or {}
        self.game_version = packets.get("GameVersion")
        self.packets = packets.get("Timeline") or []

    def quest_events(self, quest_id: int) -> list[QuestEvent]:
        quest = self.quests.get(quest_id)
        if not quest:
            return []
        events = []
        for raw in quest.get("Events", []):
            events.append(QuestEvent(
                time=parse_time(raw["Time"]),
                kind=raw.get("Kind", ""),
                sequence=int(raw.get("Sequence", 0)),
                variables=bytes.fromhex(raw.get("Variables", "000000000000")),
                target_base=raw.get("TargetBaseId"),
                target_layout=raw.get("TargetLayoutId"),
                target_kind=raw.get("TargetKind"),
                target_seconds_ago=raw.get("TargetSecondsAgo"),
                exp=raw.get("ExpGained"),
                gil=raw.get("GilGained"),
                raw=raw,
            ))
        events.sort(key=lambda e: e.time)
        return events

    def notices(self, quest_id: int | None = None, opcodes: set[int] | None = None) -> list[Notice]:
        """The quest notices in the packet recording. Their opcodes come from fork_opcodes.yml for the recording's
        game version, unless given."""
        if opcodes is None:
            opcodes = repo.notice_opcodes(self.game_version)
        found = []
        for packet in self.packets:
            if packet.get("Opcode") not in opcodes:
                continue
            handler_text = packet.get("Handler", "")
            if not handler_text.startswith("quest"):
                continue
            parsed = parse_notice(bytes.fromhex(packet.get("Hex", "")))
            if not parsed:
                continue
            handler, notice, args = parsed
            if handler < repo.QUEST_HANDLER_BASE:
                continue
            item = Notice(parse_time(packet["Time"]), handler, notice, args)
            if quest_id is None or item.quest == quest_id:
                found.append(item)
        found.sort(key=lambda n: n.time)
        return found

    def has_packets_for(self, start: float, end: float) -> bool:
        """Whether the packet recording covers this stretch of time (it keeps only so many packets.)"""
        if not self.packets:
            return False
        first = parse_time(self.packets[0]["Time"])
        last = parse_time(self.packets[-1]["Time"])
        return first <= start and end <= last + 60


class Recordings:
    """Several exports read as one: each quest from the newest export that has it, and the notices from every export
    whose packets cover that quest."""

    def __init__(self, recordings: list[Recording]):
        self.recordings = recordings

    @classmethod
    def open(cls, paths) -> "Recordings":
        return cls([Recording.open(p) for p in paths])

    @property
    def quest_ids(self) -> list[int]:
        ids = set()
        for recording in self.recordings:
            ids.update(recording.quests)
        return sorted(ids)

    def quest_name(self, quest_id: int) -> str:
        for recording in reversed(self.recordings):
            if quest_id in recording.quests:
                return recording.quests[quest_id].get("Name", "")
        return ""

    def quest_events(self, quest_id: int) -> list[QuestEvent]:
        best: list[QuestEvent] = []
        for recording in self.recordings:
            events = recording.quest_events(quest_id)
            if len(events) >= len(best):
                best = events
        return best

    def notices(self, quest_id: int) -> tuple[list[Notice], bool]:
        """The quest's notices, and whether any recording's packets covered the quest at all (if not, an empty list
        means unknown rather than none.)"""
        events = self.quest_events(quest_id)
        if not events:
            return [], False
        start, end = events[0].time - 120, events[-1].time + 5
        seen: dict[tuple, Notice] = {}
        covered = False
        for recording in self.recordings:
            if not recording.has_packets_for(events[0].time, events[-1].time):
                continue
            covered = True
            for notice in recording.notices(quest_id):
                if start <= notice.time <= end:
                    seen[(round(notice.time, 2), notice.notice, tuple(notice.args))] = notice
        return sorted(seen.values(), key=lambda n: n.time), covered
