"""Every hit and heal the capturing player dealt, with what retail's damage formula needs, as JSON lines: the player's
class and level (UpdateClassInfo: level, the class's own level, the synced level), their PlayerStats, their main hand
(UpdateItem, container 1000 slot 0: item ID and flags), the action, the effect's amount and flags (damage: critical
0x20 and direct hit 0x40 in its first parameter; heals: critical 0x20 in its second), how many targets it hit, and
the target's level and BNpcBase (its SpawnNpc). The territory and the statuses on the player then (StatusEffectList)
come with each, so instance statuses (a quest battle's role status) can be told apart. Numbers only: no names.

`questkit chronofoil <captures> --hits hits.jsonl.gz` writes it; docs/src/fork/combat.md has the fit made from it.
"""

from __future__ import annotations

import gzip
import json
import struct

from .chronofoil import EFFECTS, Capture, Names
from .chronofoil_battle import SPAWN, _effects

STATS = ["str", "dex", "vit", "int", "mnd", "pie", "hp", "mp", "tp", "gp", "cp", "delay", "tnc", "ap", "def", "dh",
         "eva", "mdef", "crit", "amp", "hmp", "elem", "det", "sks", "sps", "haste", "craft", "ctrl", "gath", "perc",
         "bstr", "bdex", "bvit", "bint", "bmnd", "bpie"]
# StatusEffectList: 20 bytes of the actor's own (class, level, HP, MP...), then 30 statuses of 12 bytes (ID, param,
# time left, source.)
STATUS_LIST_AT = 20
EQUIPPED = 1000


def hits(capture: Capture, tables: dict):
    names = Names(tables, capture.game_version)
    you = None
    territory = None
    class_info = None
    stats = None
    weapon = None
    statuses: list[int] = []
    targets: dict[int, tuple[int, int]] = {}
    for segment in capture.segments(protocols=("zone",)):
        if segment.dir != "out":
            continue
        name = names.name(segment)
        data = segment.data
        if you is None and segment.dst:
            you = segment.dst
        if name == "ZoneInit" and len(data) >= 4:
            territory, = struct.unpack_from("<H", data, 2)
            statuses = []
        elif name == "UpdateClassInfo" and len(data) >= 8:
            level, class_level, synced = struct.unpack_from("<HHH", data, 2)
            class_info = {"class": data[0], "level": level, "class_level": class_level, "synced": synced}
        elif name == "PlayerStats" and len(data) >= 4 * len(STATS):
            stats = dict(zip(STATS, struct.unpack_from(f"<{len(STATS)}I", data, 0)))
        elif name == "UpdateItem" and len(data) >= 40:
            container, slot = struct.unpack_from("<HH", data, 8)
            if container == EQUIPPED and slot == 0:
                weapon = [struct.unpack_from("<I", data, 16)[0], data[32]]
        elif name == "SpawnNpc" and len(data) >= SPAWN + 508:
            base, = struct.unpack_from("<I", data, SPAWN + 48)
            targets[segment.src] = (data[SPAWN + 118], base)
        elif name == "StatusEffectList" and segment.src == you and len(data) >= STATUS_LIST_AT + 30 * 12:
            statuses = [i for i in (struct.unpack_from("<H", data, STATUS_LIST_AT + k * 12)[0] for k in range(30)) if i]
        elif name in EFFECTS and segment.src == you and len(data) >= 42:
            action, = struct.unpack_from("<I", data, 8)
            count = data[33]
            slots = 1 if name == "ActionEffect1" else int(name.removeprefix("AoeEffect"))
            effects_at, targets_at = 42, 42 + slots * 64 + 6
            for k in range(min(count, slots)):
                target, = struct.unpack_from("<I", data, targets_at + k * 8)
                for effect in _effects(data[effects_at + k * 64:effects_at + (k + 1) * 64]):
                    if effect[0] not in (3, 4):  # damage, heal
                        continue
                    level_base = targets.get(target)
                    yield {
                        "capture": capture.id[:8], "t": round(segment.t, 2), "territory": territory,
                        "action": action, "kind": "damage" if effect[0] == 3 else "heal", "amount": effect[1],
                        "params": effect[2:], "self": target == you, "targets": count,
                        "target_level": level_base[0] if level_base else None,
                        "target_base": level_base[1] if level_base else None,
                        "class": class_info, "stats": stats, "weapon": weapon, "statuses": statuses,
                    }


def write(captures: list[Capture], tables: dict, out: str) -> int:
    count = 0
    with gzip.open(out, "wt") as f:
        for capture in captures:
            for hit in hits(capture, tables):
                f.write(json.dumps(hit) + "\n")
                count += 1
    return count
