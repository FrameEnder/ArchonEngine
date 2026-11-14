"""How retail spawned each battle NPC in instanced content (quest battles, duties), from Chronofoil captures: the
registry `questkit lint` holds the drop-ins' quest battle layers to (`tools/questkit/retail_spawns.json`, made by
`questkit chronofoil <captures> --spawns`).

A drop-in layer is written once, from one export, by a script or by hand; what it leaves out never shows until Bob
plays it. Fool Me Twice's boss (quest battle 63, layout 4270036) rode a Magitek Reaper on retail (CommonSpawn's
current_mount 14) and stood about on foot in Kawari (Bob's bundle 20261114-015635), because nothing carried the mount
from the capture to the layer. With every NPC's spawn here, lint compares each field a drop-in can say.

For each territory entered with a ContentFinderCondition (ZoneInit's at 6), each battle NPC (ObjectKind 2) with a
layout ID and no owner, as first seen: BNpcBase, BNpcName, level, maximum HP, character data flags (SpawnNpc's at 8:
0x1 passive, 0x2 aggressive), display flags, Mount, ModelChara, and how many visits saw it. Numbers only.

Since 2026-11-14 also what the first registry left out, which Every Little Thing She Does Is Magitek (quest battle
64) got wrong for it (Bob's mark of that day: "the magitek reaper shouldn't move, and Biggs, and Wedge are gone"):

* **event NPCs** (ObjectKind 3) with a layout ID, `"kind": 3` and their ENpcBase: retail spawned the two who talk
  beside the reaper in all three runs, and the script had them as voices with no one there;
* whether a battle NPC ever **moved** (an ActorMove from it) or **acted** (an action's effect from it) in any visit:
  the reaper did neither in three runs, and the script had it as a tank that walked up to the boss.
"""

from __future__ import annotations

import json
import struct
from pathlib import Path

from . import repo
from .chronofoil import Names

OUT = repo.ROOT / "tools" / "questkit" / "retail_spawns.json"
COMMON = 16  # CommonSpawn in SpawnNpc
BATTLE_NPC = 2
EVENT_NPC = 3
FROM_THE_START = 5.0  # seconds into a visit: there as it began
NO_OWNER = (0, 0xE0000000)


def spawn_fields(data: bytes) -> dict:
    """A SpawnNpc's fields a drop-in battle NPC carries (see the module's description for the offsets' sources:
    `core/src/ipc/zone/server/common_spawn.rs`)."""
    c = COMMON
    return {
        "base": struct.unpack_from("<I", data, c + 48)[0],
        "name": struct.unpack_from("<I", data, c + 52)[0],
        "level": data[c + 118],
        "hp": struct.unpack_from("<I", data, c + 76)[0],
        "flags": data[8],
        "display": struct.unpack_from("<I", data, c + 84)[0],
        "mount": struct.unpack_from("<H", data, c + 100)[0],
        "model_chara": struct.unpack_from("<H", data, c + 96)[0],
    }


def collect(captures, tables: dict) -> dict[str, dict[str, dict]]:
    """territory -> layout -> fields (with `visits`), from every capture."""
    out: dict[str, dict[str, dict]] = {}
    for capture in captures:
        names = Names(tables, capture.game_version)
        zone, content, seen_here, entered = None, 0, set(), 0.0
        actors: dict[int, dict] = {}  # this visit's actors -> their entries
        for segment in capture.segments(protocols=("zone",)):
            if segment.dir != "out":
                continue
            name = names.name(segment)
            data = segment.data
            if name in ("ZoneInit", "InitZone") and len(data) >= 8:
                zone, content = struct.unpack_from("<HxxH", data, 2)
                seen_here = set()
                actors = {}
                entered = segment.t
            elif name == "SpawnNpc" and content and len(data) > COMMON + 508 \
                    and data[COMMON + 113] in (BATTLE_NPC, EVENT_NPC):
                layout = struct.unpack_from("<I", data, COMMON + 56)[0]
                owner = struct.unpack_from("<I", data, COMMON + 68)[0]
                if not layout or owner not in NO_OWNER:
                    continue
                spots = out.setdefault(str(zone), {})
                entry = spots.get(str(layout))
                if entry is None:
                    if data[COMMON + 113] == EVENT_NPC:
                        # `late`: never there as a visit began (the three who come as a cutscene ends in
                        # quest battle 65), so a run that ended before then hadn't them.
                        fields = {"kind": EVENT_NPC, "base": struct.unpack_from("<I", data, COMMON + 48)[0],
                                  "late": True}
                    else:
                        fields = dict(spawn_fields(data), moved=False, acted=False)
                    entry = spots[str(layout)] = dict(fields, visits=0)
                actors[segment.src] = entry
                if layout not in seen_here:
                    seen_here.add(layout)
                    entry["visits"] += 1
                    if entry.get("late") and segment.t - entered <= FROM_THE_START:
                        del entry["late"]
            elif name == "ActorMove" and segment.src in actors and "moved" in actors[segment.src]:
                actors[segment.src]["moved"] = True
            elif name.startswith(("ActionEffect", "AoeEffect")) and segment.src in actors \
                    and "acted" in actors[segment.src]:
                actors[segment.src]["acted"] = True
    return {zone: dict(sorted(spots.items(), key=lambda kv: int(kv[0])))
            for zone, spots in sorted(out.items(), key=lambda kv: int(kv[0]))}


def save(registry: dict, out: Path = OUT) -> Path:
    """One territory a line, one NPC per entry, so a new capture's changes show in a diff. Merged with what's there:
    a territory a new capture didn't visit keeps its NPCs, and an NPC seen again keeps its first sighting."""
    old = json.loads(out.read_text()) if out.exists() else {}
    for zone, spots in registry.items():
        merged = old.setdefault(zone, {})
        for layout, entry in spots.items():
            if layout in merged:
                merged[layout]["visits"] = max(merged[layout]["visits"], entry["visits"])
                for key in ("moved", "acted"):
                    if key in entry:
                        merged[layout][key] = bool(merged[layout].get(key)) or entry[key]
                if "late" in merged[layout] and "late" not in entry:
                    del merged[layout]["late"]
            else:
                merged[layout] = entry
        old[zone] = dict(sorted(merged.items(), key=lambda kv: int(kv[0])))
    lines = ["{"]
    zones = sorted(old, key=int)
    for i, zone in enumerate(zones):
        lines.append(f"  {json.dumps(zone)}: " + json.dumps(old[zone], separators=(",", ":"))
                     + ("," if i < len(zones) - 1 else ""))
    lines.append("}")
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(lines) + "\n")
    return out
