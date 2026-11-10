"""The ledger: one pass over a Chronofoil capture into typed events, numbers only (docs/src/fork/capture-pipeline.md,
stage B). Every fact table reads this, never the raw capture, so a packet's layout is written once, here.

Nobody is named: an actor is a small number, counted per capture in the order they appear (0 is the capturing player),
and a `spawn` event says what it is (`self`, `player` with its job and level, `npc` with its BNpcBase and BNpcName,
`pet` with its owner). Entity IDs, account IDs, names and chat never leave the decoder.

Events are dicts; `e` is the kind, `t` the Unix time, `a` the actor the packet is about:

| `e` | From | Carries |
|---|---|---|
| `zone` | ZoneInit | `territory`, `content` (ContentFinderCondition) |
| `class` | UpdateClassInfo | `job`, `lv`, `class_lv`, `synced` |
| `spawn` | SpawnNpc, SpawnPlayer | `k`, `job`, `lv`, `hp`, `p`; `base`, `name`, `owner` for NPCs |
| `despawn` | DeleteActor | |
| `request` | the client's ActionRequest(GroundTargeted) | `action`, `type`, `seq`, `on` |
| `cast` | ActorCast | `action`, `type`, `time`, `on`, `p`, `r` |
| `effect` | ActionEffect1, AoeEffect8-32 | `action`, `type`, `seq` (the request's, for one's own), `lock` (animation lock), `p`, `r`, `hits`: each `on`, `p` and `fx` (kind, value, five parameters) |
| `result` | EffectResult(Basic) | `hp`, and from the full one `max`, `mp`, `shield`, `st`: statuses given (id, param, seconds, by whom) |
| `statuses` | StatusEffectList* | `st` as above, when it differs from the last one sent for that actor; `hp`, `mp`, `shield` |
| `vitals` | UpdateHpMpTp | `hp`, `mp` |
| `gauge` | ActorGauge | `job`, `b` (the 15 bytes, hex) |
| `recast` | UpdateRecastTimes | `groups`: {recast group: [elapsed, total]} for those running |
| `control` | ActorControl, -Self, -Target | `pk` (which), `cat`, `args`, `on` |
| `trigger` | the client's ClientTrigger | `cat`, `args`, `on` |
| `condition` | Condition | `flags` (the set condition numbers) |
| `first_attack` | FirstAttack | `kind`, `by` |
| `loot` | LootMessage | `kind`, `args` |
| `director` | DirectorVars | `handler`, `seq`, `branch`, `vars` |
| `coverage` | (the last event) | per direction: packets by name, unnamed by opcode and size, wrong sizes |

An argument that is an actor's entity ID is written as `"a<n>"`; one that is an entity ID of nobody seen as `"a?"`.
"""

from __future__ import annotations

import gzip
import json
import os
import struct
from collections import Counter
from pathlib import Path

from .chronofoil import EFFECTS, Capture, Names
from .chronofoil_battle import SPAWN, _effects, _packed, _rotation

# Raise when an event's layout changes: cached ledgers of an older number are remade.
VERSION = 2
NOBODY = 0xE0000000


def _is_entity(value: int) -> bool:
    """Players' entity IDs are 0x10xxxxxx; NPCs', pets' and objects' 0x40xxxxxx."""
    return value >> 24 in (0x10, 0x40)


class Decoder:
    def __init__(self, capture: Capture, tables: dict):
        self.capture = capture
        self.names = Names(tables, capture.game_version)
        self.actors: dict[int, int] = {}
        self.you: int | None = None
        self.positions: dict[int, list[float]] = {}
        self.last_statuses: dict[int, list] = {}
        self.named: dict[str, Counter] = {"out": Counter(), "in": Counter()}
        self.unnamed: dict[str, Counter] = {"out": Counter(), "in": Counter()}
        self.wrong: Counter = Counter()

    def actor(self, entity: int) -> int | None:
        if entity in (0, NOBODY, 0xFFFFFFFF):
            return None
        if entity not in self.actors:
            self.actors[entity] = len(self.actors)
        return self.actors[entity]

    def argument(self, value: int):
        if value in self.actors:
            return f"a{self.actors[value]}"
        return "a?" if _is_entity(value) else value

    def statuses(self, data: bytes, at: int, count: int, size: int, lead: int) -> list:
        """Status entries of `size` bytes whose id is `lead` bytes in: [id, param, seconds, by whom]."""
        out = []
        for k in range(count):
            offset = at + k * size + lead
            if offset + 12 > len(data):
                break
            status, param = struct.unpack_from("<HH", data, offset)
            if not status:
                continue
            duration, source = struct.unpack_from("<fI", data, offset + (6 if lead else 4))
            out.append([status, param, round(duration, 2), self.actor(source)])
        return out

    def events(self):
        names = self.names
        for segment in self.capture.segments(protocols=("zone",)):
            name = names.name(segment)
            data, t = segment.data, round(segment.t, 3)
            if name.startswith("Unknown"):
                self.unnamed[segment.dir][f"{segment.opcode}/{len(data)}"] += 1
                continue
            self.named[segment.dir][name] += 1
            expected = names.size(segment)
            if expected is not None and expected != len(data):
                self.wrong[name] += 1
            try:
                if segment.dir == "out":
                    if self.you is None and segment.dst:
                        self.you = segment.dst
                        self.actors[self.you] = 0
                    yield from self.server(name, segment.src, data, t)
                else:
                    yield from self.client(name, data, t)
            except struct.error:
                continue
        yield {"e": "coverage", "version": self.capture.game_version, "table": names.known,
               "named": {d: dict(c.most_common()) for d, c in self.named.items()},
               "unnamed": {d: dict(c.most_common()) for d, c in self.unnamed.items()},
               "wrong": dict(self.wrong.most_common())}

    def client(self, name: str, data: bytes, t: float):
        if name == "ActionRequest" and len(data) >= 24:
            action, _, kind, sequence = struct.unpack_from("<IBBH", data, 0)
            target, = struct.unpack_from("<I", data, 16)
            yield {"e": "request", "t": t, "a": 0, "action": action, "type": kind, "seq": sequence,
                   "on": self.actor(target)}
        elif name == "ActionRequestGroundTargeted" and len(data) >= 24:
            action, _, kind, sequence = struct.unpack_from("<IBBH", data, 0)
            yield {"e": "request", "t": t, "a": 0, "action": action, "type": kind, "seq": sequence, "on": None,
                   "ground": [round(v, 2) for v in struct.unpack_from("<fff", data, 12)]}
        elif name == "ClientTrigger" and len(data) >= 32:
            category, = struct.unpack_from("<I", data, 0)
            target, = struct.unpack_from("<I", data, 24)
            yield {"e": "trigger", "t": t, "a": 0, "cat": category,
                   "args": [self.argument(v) for v in struct.unpack_from("<IIIII", data, 4)], "on": self.actor(target)}
        elif name in ("UpdatePositionHandler", "UpdatePositionHandlerInstance") and self.you is not None:
            at = 8 if name == "UpdatePositionHandler" else 12
            if len(data) >= at + 12:
                self.positions[self.you] = [round(v, 2) for v in struct.unpack_from("<fff", data, at)]

    def spawn(self, t: float, entity: int, data: bytes, offset: int, player: bool):
        base, name_id = struct.unpack_from("<II", data, offset + 48)
        owner, = struct.unpack_from("<I", data, offset + 68)
        max_hp, hp = struct.unpack_from("<II", data, offset + 76)
        position = [round(v, 2) for v in struct.unpack_from("<fff", data, offset + 496)]
        self.positions[entity] = position
        event = {"e": "spawn", "t": t, "a": self.actor(entity), "job": data[offset + 119], "lv": data[offset + 118],
                 "hp": hp, "max": max_hp, "p": position}
        if player:
            event["k"] = "self" if entity == self.you else "player"
        else:
            owned = self.actor(owner)
            event.update(k="pet" if owned is not None else "npc", base=base, name=name_id, kind=data[offset + 113],
                         battalion=data[offset + 117])
            if owned is not None:
                event["owner"] = owned
        return event

    def server(self, name: str, src: int, data: bytes, t: float):
        a = self.actor(src)
        if name == "ZoneInit" and len(data) >= 20:
            territory, _, content = struct.unpack_from("<HHH", data, 2)
            self.last_statuses.clear()
            yield {"e": "zone", "t": t, "territory": territory, "content": content}
        elif name == "UpdateClassInfo" and len(data) >= 8:
            level, class_level, synced = struct.unpack_from("<HHH", data, 2)
            yield {"e": "class", "t": t, "a": 0, "job": data[0], "lv": level, "class_lv": class_level, "synced": synced}
        elif name == "SpawnNpc" and len(data) >= SPAWN + 508:
            yield self.spawn(t, src, data, SPAWN, False)
        elif name == "SpawnPlayer" and len(data) >= 32 + 508:
            yield self.spawn(t, src, data, 32, True)
        elif name == "DeleteActor" and len(data) >= 8:
            gone, = struct.unpack_from("<I", data, 4)
            if gone in self.actors:
                yield {"e": "despawn", "t": t, "a": self.actors[gone]}
        elif name == "ActorMove" and len(data) >= 12:
            x, y, z = struct.unpack_from("<HHH", data, 6)
            self.positions[src] = [_packed(x), _packed(y), _packed(z)]
        elif name == "ActorSetPos" and len(data) >= 20:
            self.positions[src] = [round(v, 2) for v in struct.unpack_from("<fff", data, 8)]
        elif name == "ActorCast" and len(data) >= 30:
            _, kind, _, action, cast_time, target, rotation, _ = struct.unpack_from("<HBBIfIHH", data, 0)
            x, y, z = struct.unpack_from("<HHH", data, 24)
            yield {"e": "cast", "t": t, "a": a, "action": action, "type": kind, "time": round(cast_time, 2),
                   "on": self.actor(target),
                   "p": [_packed(x), _packed(y), _packed(z)], "r": _rotation(rotation)}
        elif name in EFFECTS and len(data) >= 42:
            action, = struct.unpack_from("<I", data, 8)
            lock, = struct.unpack_from("<f", data, 16)
            sequence, rotation = struct.unpack_from("<HH", data, 24)
            slots = 1 if name == "ActionEffect1" else int(name.removeprefix("AoeEffect"))
            effects_at, targets_at = 42, 42 + slots * 64 + 6
            hits = []
            for k in range(min(data[33], slots)):
                target, = struct.unpack_from("<I", data, targets_at + k * 8)
                hits.append({"on": self.actor(target), "p": self.positions.get(target),
                             "fx": _effects(data[effects_at + k * 64:effects_at + (k + 1) * 64])})
            yield {"e": "effect", "t": t, "a": a, "action": action, "type": data[31], "seq": sequence,
                   "lock": round(lock, 2), "p": self.positions.get(src), "r": _rotation(rotation), "hits": hits}
        elif name == "EffectResult" and len(data) >= 32:
            hp, max_hp, mp = struct.unpack_from("<IIH", data, 12)
            yield {"e": "result", "t": t, "a": a, "hp": hp, "max": max_hp, "mp": mp, "shield": data[24],
                   "st": self.statuses(data, 28, min(data[25], 4), 16, 2)}
        elif name == "EffectResultBasic" and len(data) >= 16:
            yield {"e": "result", "t": t, "a": a, "hp": struct.unpack_from("<I", data, 12)[0]}
        elif name.startswith("StatusEffectList") and len(data) >= 20 + 12:
            # The plain list: classjob, level, flags, HP, max, MP, max, shield, then 30 statuses of 12 bytes from 20.
            # (The other kinds' statuses start elsewhere; only the plain one is read.)
            if name != "StatusEffectList":
                return
            hp, max_hp, mp = struct.unpack_from("<IIH", data, 4)
            listed = self.statuses(data, 20, 30, 12, 0)
            key = [entry[:2] for entry in listed]
            if self.last_statuses.get(src) != key:
                self.last_statuses[src] = key
                yield {"e": "statuses", "t": t, "a": a, "hp": hp, "max": max_hp, "mp": mp, "shield": data[16],
                       "st": listed}
        elif name == "UpdateHpMpTp" and len(data) >= 6:
            hp, mp = struct.unpack_from("<IH", data, 0)
            yield {"e": "vitals", "t": t, "a": a, "hp": hp, "mp": mp}
        elif name == "ActorGauge" and len(data) >= 16:
            yield {"e": "gauge", "t": t, "a": a, "job": data[0], "b": data[1:16].hex()}
        elif name == "UpdateRecastTimes" and len(data) >= 640:
            # 80 recast groups: each one's seconds elapsed, then each one's whole recast (two arrays of floats.)
            elapsed = struct.unpack_from("<80f", data, 0)
            total = struct.unpack_from("<80f", data, 320)
            yield {"e": "recast", "t": t, "a": a,
                   "groups": {str(k): [round(elapsed[k], 2), round(total[k], 2)] for k in range(80) if total[k]}}
        elif name in ("ActorControl", "ActorControlSelf", "ActorControlTarget") and len(data) >= 20:
            category, = struct.unpack_from("<H", data, 0)
            count = 6 if name == "ActorControlSelf" and len(data) >= 28 else 4
            event = {"e": "control", "t": t, "a": a, "pk": name.removeprefix("ActorControl") or "All", "cat": category,
                     "args": [self.argument(v) for v in struct.unpack_from(f"<{count}I", data, 4)]}
            if name == "ActorControlTarget" and len(data) >= 28:
                event["on"] = self.actor(struct.unpack_from("<I", data, 24)[0])
            yield event
        elif name == "Condition" and len(data) >= 8:
            bits = int.from_bytes(data[:12], "little")
            yield {"e": "condition", "t": t, "a": 0, "flags": [k for k in range(96) if bits >> k & 1]}
        elif name == "FirstAttack" and len(data) >= 12:
            kind, _, tagger = struct.unpack_from("<III", data, 0)
            yield {"e": "first_attack", "t": t, "a": a, "kind": kind, "by": self.actor(tagger)}
        elif name == "LootMessage" and len(data) >= 32:
            kind, *args = struct.unpack_from("<8I", data, 0)
            yield {"e": "loot", "t": t, "a": a, "kind": kind, "args": [self.argument(v) for v in args]}
        elif name == "DirectorVars" and len(data) >= 16:
            handler, sequence, branch = struct.unpack_from("<IBB", data, 0)
            yield {"e": "director", "t": t, "handler": handler, "seq": sequence, "branch": branch,
                   "vars": list(data[6:16])}


def events(capture: Capture, tables: dict):
    """The capture's events, in order (the last is `coverage`.)"""
    return Decoder(capture, tables).events()


def cache_folder() -> Path:
    return Path(os.environ.get("QUESTKIT_LEDGERS") or Path.home() / ".cache" / "questkit" / "ledgers")


def cached(capture: Capture, tables: dict, folder: Path | None = None):
    """The capture's events from its cached ledger (`<capture ID>.v<VERSION>.jsonl.gz`), made on first use. Unfinished
    captures (still being written) aren't cached."""
    if not capture.finished:
        yield from events(capture, tables)
        return
    folder = folder or cache_folder()
    path = folder / f"{capture.id}.v{VERSION}.jsonl.gz"
    if not path.exists():
        folder.mkdir(parents=True, exist_ok=True)
        partial = path.with_suffix(".part")
        with gzip.open(partial, "wt", compresslevel=3) as f:
            for event in events(capture, tables):
                f.write(json.dumps(event, separators=(",", ":")) + "\n")
        partial.replace(path)
    with gzip.open(path, "rt") as f:
        for line in f:
            yield json.loads(line)
