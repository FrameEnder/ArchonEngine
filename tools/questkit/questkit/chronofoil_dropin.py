"""A zone's open-world battle NPCs from Chronofoil captures, as a Kawari drop-in (resources/dropins/).

Every SpawnNpc of a battle NPC while the player is in the territory gives its layout ID, BNpcBase and BNpcName, level,
maximum HP, character flags (0x2 = hostile), links and display flags, where it stood and which way it faced. Flying
around a zone on retail with Chronofoil running sees nearly all of its spawn points. Left out: FATE monsters (their
FATE spawns them), NPCs owned by someone (pets, chocobos), spawns without a layout ID or turned from an event NPC
(quest NPCs), and anything in `exclude_bases`. Monsters wander, so each one goes where it was seen nearest the average
of its sightings. A base in `one_spot` (e.g. a hunt mark that moves between spawn points) keeps only its first spot.

Captures carry other players and chat, so the drop-in only has numbers.
"""

from __future__ import annotations

import json
import math
import struct
from pathlib import Path

from .chronofoil import Names

COMMON = 16  # where CommonSpawn starts in SpawnNpc (after gimmick, event NPC, flags, icon, scaling, links...)
BATTLE_NPC = 2
NO_OWNER = (0, 0xE0000000)


def _spawn(data: bytes) -> dict:
    c = COMMON
    quantized = struct.unpack_from("<H", data, c + 98)[0]
    return {
        "layout": struct.unpack_from("<I", data, c + 56)[0],
        "base": struct.unpack_from("<I", data, c + 48)[0],
        "name": struct.unpack_from("<I", data, c + 52)[0],
        "owner": struct.unpack_from("<I", data, c + 68)[0],
        "max_hp": struct.unpack_from("<I", data, c + 76)[0],
        "display": struct.unpack_from("<I", data, c + 84)[0],
        "fate": struct.unpack_from("<H", data, c + 88)[0],
        "rotation": quantized / 65535 * 2 * math.pi - math.pi,  # read_quantized_rotation
        "kind": data[c + 113],
        "level": data[c + 118],
        "position": list(struct.unpack_from("<fff", data, c + 496)),
        "event_npc": struct.unpack_from("<I", data, 4)[0],
        "flags": data[8],
        "max_links": data[11],
        "link_family": data[12],
        "link_range": data[13],
    }


def sightings(captures, tables: dict, territory: int) -> dict[int, list[dict]]:
    """Every battle NPC SpawnNpc in the territory, by layout ID."""
    seen: dict[int, list[dict]] = {}
    for capture in captures:
        names = Names(tables, capture.game_version)
        zone = None
        for segment in capture.segments(protocols=("zone",)):
            if segment.dir != "out":
                continue
            name = names.name(segment)
            if name in ("InitZone", "ZoneInit"):
                zone = struct.unpack_from("<H", segment.data, 2)[0]
            elif name == "SpawnNpc" and zone == territory and len(segment.data) > COMMON + 508:
                spawn = _spawn(segment.data)
                if spawn["kind"] == BATTLE_NPC:
                    seen.setdefault(spawn["layout"], []).append(spawn)
    return seen


def build(seen: dict[int, list[dict]], appends: str, layer: str = "BNPCS_RETAIL", exclude_bases=(),
          one_spot=()) -> tuple[dict, dict]:
    """The drop-in, and how many spawn points were left out for each reason."""
    objects, skipped, placed = [], {}, set()

    def skip(reason):
        skipped[reason] = skipped.get(reason, 0) + 1

    for layout in sorted(seen):
        all_seen = seen[layout]
        last = all_seen[-1]
        if last["owner"] not in NO_OWNER:
            skip("owned by someone")
        elif layout == 0:
            skip("no layout ID")
        elif last["fate"]:
            skip("FATE")
        elif last["event_npc"]:
            skip("event NPC")
        elif last["base"] in exclude_bases:
            skip("excluded")
        elif last["base"] in one_spot and last["base"] in placed:
            skip("another spot of a one-spot NPC")
        else:
            placed.add(last["base"])
            mx = sum(s["position"][0] for s in all_seen) / len(all_seen)
            mz = sum(s["position"][2] for s in all_seen) / len(all_seen)
            best = min(all_seen, key=lambda s: (s["position"][0] - mx) ** 2 + (s["position"][2] - mz) ** 2)
            data = {
                "base_id": last["base"], "gimmick_id": 0, "hostile": bool(last["flags"] & 2),
                "hp": last["max_hp"], "level": last["level"], "link_family": last["link_family"],
                "link_range": last["link_range"], "max_links": last["max_links"], "name_id": last["name"],
                "nonpop": False, "type": "battle_npc",
            }
            if last["display"]:
                data["display_flags"] = last["display"]
            objects.append({
                "data": data, "layout_id": layout,
                "position": {axis: round(best["position"][i], 3) for i, axis in enumerate("xyz")},
                "rotation": round(best["rotation"], 4),
            })
    return {"appends": appends, "layers": [{"name": layer, "objects": objects}]}, skipped


def write(dropin: dict, out: str | Path) -> None:
    Path(out).write_text(json.dumps(dropin, indent=4) + "\n")
