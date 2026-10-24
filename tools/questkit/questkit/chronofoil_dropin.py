"""A zone's open-world battle NPCs from Chronofoil captures, as a Kawari drop-in (resources/dropins/).

Every SpawnNpc of a battle NPC while the player is in the territory gives its layout ID, BNpcBase and BNpcName, level,
maximum HP, character flags (0x2 = hostile), links and display flags, where it stood and which way it faced. Flying
around a zone on retail with Chronofoil running sees nearly all of its spawn points. Left out: FATE monsters (their
FATE spawns them), NPCs owned by someone (pets, chocobos), spawns without a layout ID or turned from an event NPC
(quest NPCs), and anything in `exclude_bases`. Monsters wander, so each one goes where it was seen nearest the average
of its sightings. A base in `one_spot` (e.g. a hunt mark that moves between spawn points) keeps only its first spot.

Guards (character data flags 0x4) keep that flag and their Behavior row (`character_data_flags`, `behavior`). Each
sighting also records whether the NPC moved while it was idle (an ActorMove more than `MOVED` yalms from where it
appeared, while it wasn't fighting: no target, cast or action of its own), and for how long it was seen: a spot is
`stationary` when it was seen idle for `IDLE_SECONDS` or more without moving and never seen idle and moving, and so
are the unseen (or briefly seen) spots of a kind that never moved in two or more such sightings.

`gathering_points`/`build_gathering` do the same for gathering points (SpawnObject of kind 6, which the client's
layouts don't have): every position of every point, with retail's spawn arguments (`spawn_args`: integrity, the
point's phase slot, which of its positions this is, how many it has.)

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
GUARD = 0x4  # character data flags of a friendly battle NPC with a talk (city guards)
MOVED = 1.0  # yalms from where it appeared
IDLE_SECONDS = 60
# Packets that mean the NPC (their sender) was busy fighting, not wandering.
BUSY = ("ActorControlTarget", "ActorCast", "ActionEffect1", "AoeEffect8", "AoeEffect16", "AoeEffect24", "AoeEffect32")


def _packed(value: int) -> float:
    return value / 65535 * 2000 - 1000  # read_packed_position


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
        "behavior": struct.unpack_from("<H", data, c + 94)[0],
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
    """Every battle NPC SpawnNpc in the territory, by layout ID, each with how long it was seen (`seconds`), whether it
    moved while idle (`moved`) and whether it fought (`busy`)."""
    seen: dict[int, list[dict]] = {}
    for capture in captures:
        names = Names(tables, capture.game_version)
        zone = None
        live: dict[int, dict] = {}  # actor ID -> its sighting

        def end_all(t):
            for sighting in live.values():
                sighting["seconds"] = t - sighting["since"]
            live.clear()

        last_t = 0.0
        for segment in capture.segments(protocols=("zone",)):
            last_t = segment.t
            if segment.dir != "out":
                continue
            name = names.name(segment)
            data = segment.data
            if name in ("InitZone", "ZoneInit"):
                end_all(segment.t)
                zone = struct.unpack_from("<H", data, 2)[0]
            elif name == "SpawnNpc" and zone == territory and len(data) > COMMON + 508:
                spawn = _spawn(data)
                if spawn["kind"] == BATTLE_NPC:
                    spawn.update(since=segment.t, seconds=0.0, moved=False, busy=False)
                    seen.setdefault(spawn["layout"], []).append(spawn)
                    if segment.src in live:
                        live[segment.src]["seconds"] = segment.t - live[segment.src]["since"]
                    live[segment.src] = spawn
            elif name == "DeleteActor" and len(data) >= 8:
                # The actor's ID is in the packet (spawn index, 3 empty, actor ID), not its sender.
                actor = struct.unpack_from("<I", data, 4)[0]
                if actor in live:
                    live[actor]["seconds"] = segment.t - live[actor]["since"]
                    del live[actor]
            elif segment.src in live:
                sighting = live[segment.src]
                if name == "ActorMove" and len(data) >= 12:
                    x, _, z = (_packed(v) for v in struct.unpack_from("<HHH", data, 6))
                    start = sighting["position"]
                    if not sighting["busy"] and math.hypot(x - start[0], z - start[2]) > MOVED:
                        sighting["moved"] = True
                elif name in BUSY:
                    sighting["busy"] = True
        end_all(last_t)
    return seen


def _idle(sighting: dict) -> bool:
    """Seen idle long enough to tell whether it wanders."""
    return not sighting["busy"] and (sighting["moved"] or sighting["seconds"] >= IDLE_SECONDS)


def stationary_spots(seen: dict[int, list[dict]]) -> set[int]:
    """The layout IDs that stand still (see the module's description.)"""
    still, wanders = set(), set()
    kinds: dict[int, list[bool]] = {}
    for layout, sightings in seen.items():
        idle = [s for s in sightings if _idle(s)]
        if any(s["moved"] for s in idle):
            wanders.add(layout)
        elif idle:
            still.add(layout)
        for s in idle:
            kinds.setdefault(s["base"], []).append(s["moved"])
    never_moving = {base for base, moved in kinds.items() if len(moved) >= 2 and not any(moved)}
    for layout, sightings in seen.items():
        if layout not in wanders and layout not in still and sightings[-1]["base"] in never_moving:
            still.add(layout)
    return still


def build(seen: dict[int, list[dict]], appends: str, layer: str = "BNPCS_RETAIL", exclude_bases=(),
          one_spot=()) -> tuple[dict, dict]:
    """The drop-in, and how many spawn points were left out for each reason."""
    objects, skipped, placed = [], {}, set()
    still = stationary_spots(seen)

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
            if layout in still:
                data["stationary"] = True
            if last["flags"] & GUARD:
                data["character_data_flags"] = GUARD
                data["behavior"] = last["behavior"]
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


GATHERING_POINT = 6  # ObjectKind


def gathering_points(captures, tables: dict, territory: int) -> dict[int, dict]:
    """Every gathering point SpawnObject in the territory, by layout ID (the last sighting of each)."""
    seen: dict[int, dict] = {}
    for capture in captures:
        names = Names(tables, capture.game_version)
        zone = None
        for segment in capture.segments(protocols=("zone",)):
            if segment.dir != "out":
                continue
            name = names.name(segment)
            data = segment.data
            if name in ("InitZone", "ZoneInit"):
                zone = struct.unpack_from("<H", data, 2)[0]
            elif name == "SpawnObject" and zone == territory and len(data) >= 64 and data[1] == GATHERING_POINT:
                base, _, layout = struct.unpack_from("<III", data, 4)
                seen[layout] = {
                    "base": base,
                    "args1": struct.unpack_from("<I", data, 44)[0],
                    "position": list(struct.unpack_from("<fff", data, 52)),
                }
    return seen


def build_gathering(seen: dict[int, dict], appends: str, layer: str = "GATHERING_RETAIL") -> tuple[dict, dict]:
    """The drop-in, and how many points were left out for each reason. Points sent with no integrity (levequest
    points, a spearfishing spot) only appear for their leve or quest, so they're left out."""
    objects, skipped = [], {}
    for layout in sorted(seen):
        point = seen[layout]
        if point["args1"] & 0xFF == 0:
            skipped["no integrity (leve or quest)"] = skipped.get("no integrity (leve or quest)", 0) + 1
            continue
        objects.append({
            "data": {"base_id": point["base"], "spawn_args": point["args1"], "type": "gathering_point"},
            "layout_id": layout,
            "position": {axis: round(point["position"][i], 3) for i, axis in enumerate("xyz")},
            "rotation": 0.0,
        })
    return {"appends": appends, "layers": [{"name": layer, "objects": objects}]}, skipped
