"""Reading Kawari's packet captures (world config packet_capture: true, see servers/world/src/capture.rs).

One JSON object per line: t (Unix ms), dir (out = server to client, in, note), src/dst (actor IDs), name (Kawari's
name for the packet), size, hex (the IPC segment including its 16-byte IPC header) and decoded (Kawari's reading).
Files may be gzipped, as in diagnostics bundles.
"""

from __future__ import annotations

import gzip
import json
import re
from dataclasses import dataclass
from pathlib import Path

from .recording import parse_notice

IPC_HEADER = 16

EVENT_PACKETS = (
    "StartTalkEvent", "StartWalkInEvent", "EventStart", "EventFinish", "EnterTerritoryEvent", "EventUnkRequest",
)


@dataclass
class Packet:
    t: float  # Unix seconds
    dir: str
    name: str
    src: int
    dst: int
    size: int
    hex: str
    decoded: str
    text: str = ""  # for notes

    @property
    def data(self) -> bytes:
        """The IPC data, without its header."""
        return bytes.fromhex(self.hex)[IPC_HEADER:]


def open_lines(path: Path):
    if path.suffix == ".gz":
        return gzip.open(path, "rt", encoding="utf-8", errors="replace")
    return open(path, encoding="utf-8", errors="replace")


def load(path: str | Path) -> list[Packet]:
    packets = []
    with open_lines(Path(path)) as lines:
        for line in lines:
            line = line.strip()
            if not line:
                continue
            try:
                raw = json.loads(line)
            except json.JSONDecodeError:
                continue
            packets.append(Packet(
                t=raw.get("t", 0) / 1000.0,
                dir=raw.get("dir", ""),
                name=raw.get("name") or raw.get("kind") or "",
                src=raw.get("src", 0),
                dst=raw.get("dst", 0),
                size=raw.get("size", 0),
                hex=raw.get("hex", ""),
                decoded=raw.get("decoded", ""),
                text=raw.get("text", ""),
            ))
    packets.sort(key=lambda p: p.t)
    return packets


def load_many(paths) -> list[Packet]:
    packets = []
    for path in paths:
        packets.extend(load(path))
    packets.sort(key=lambda p: p.t)
    return packets


def actors(packets: list[Packet]) -> list[int]:
    """The players in a capture (a connection can log in more than one character over its life.)"""
    seen = []
    for packet in packets:
        if packet.name == "PlayerSetup" or packet.name == "InitResponse":
            if packet.src and packet.src not in seen:
                seen.append(packet.src)
    for packet in packets:
        if packet.src and packet.src not in seen and packet.name in ("UpdateQuest", "StartTalkEvent"):
            seen.append(packet.src)
    return seen


# --- Quests -------------------------------------------------------------------------------------------------------

_ACTIVE_QUEST = re.compile(
    r"id: (\d+), sequence: (\d+), flags: (\d+), accept_class_job: (\d+), bitflags: \[([\d, ]*)\]")


@dataclass
class QuestState:
    t: float
    actor: int
    quest: int
    sequence: int
    variables: bytes
    kind: str  # update, accepted, finished, list


def quest_states(packets: list[Packet]) -> list[QuestState]:
    """Every quest state the server sent: each UpdateQuest, the active list at login, accepts and finishes."""
    states = []
    for packet in packets:
        if packet.dir != "out":
            continue
        if packet.name == "UpdateQuest" or packet.name == "QuestActiveList":
            for match in _ACTIVE_QUEST.finditer(packet.decoded):
                quest = int(match.group(1))
                if quest == 0:
                    continue
                values = [int(v) for v in match.group(5).split(",") if v.strip()]
                states.append(QuestState(packet.t, packet.src, quest, int(match.group(2)), bytes(values[:6]),
                                         "update" if packet.name == "UpdateQuest" else "list"))
        elif packet.name in ("AcceptQuest", "FinishQuest"):
            match = re.search(r"quest_id: (\d+)", packet.decoded)
            if match:
                states.append(QuestState(packet.t, packet.src, int(match.group(1)), -1, b"",
                                         "accepted" if packet.name == "AcceptQuest" else "finished"))
    return states


@dataclass
class CapturedNotice:
    t: float
    actor: int
    handler: int
    notice: int
    args: list[int]

    @property
    def quest(self) -> int:
        return self.handler - 0x10000


def notices(packets: list[Packet]) -> list[CapturedNotice]:
    found = []
    for packet in packets:
        if packet.dir == "out" and packet.name.startswith("EventNotice"):
            parsed = parse_notice(packet.data)
            if parsed:
                found.append(CapturedNotice(packet.t, packet.src, *parsed))
    return found


# --- Events -------------------------------------------------------------------------------------------------------

_HANDLER = re.compile(r"HandlerId \{ type: (\w+), id: (\d+) \}")
_SCENE = re.compile(r"scene: (\d+)")


def handler_of(packet: Packet) -> tuple[str, int] | None:
    match = _HANDLER.search(packet.decoded)
    if match:
        return match.group(1), int(match.group(2))
    return None


def is_event_packet(packet: Packet) -> bool:
    name = packet.name
    return (name in EVENT_PACKETS or name.startswith("EventScene") or name.startswith("EventReturn")
            or name.startswith("EventYield") or name.startswith("EventNotice") or name.startswith("EventLogMessage")
            or name in ("AcceptQuest", "FinishQuest", "UpdateQuest"))


def scene_of(packet: Packet) -> int | None:
    match = _SCENE.search(packet.decoded)
    return int(match.group(1)) if match else None
