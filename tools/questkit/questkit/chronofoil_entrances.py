"""Duty entrances in Chronofoil captures, checked against what the server knows of them.

A duty's players come in at its entrance circle: an EObj bound to the spot they stand on, which retail's server
hides with ActorControl 106 [7] at DutyCommence and shows again after a wipe. Kawari finds the circle by its EObj ID
(`ENTRANCE_CIRCLE_IDS` in core/src/common/game.rs) and turns players the way retail did by territory
(resources/data/entrance_facing.json). A duty whose circle isn't in the list comes in at a random pop range of the
whole duty, and its circle never hides: the Stone Vigil (Bob's bundle 20261107-205303), then Castrum Meridianum
(Bob's mark of 2026-11-19, "the starting spawn point isn't even slightly set-up") with five runs of it in hand.

`observe` reads every duty entry out of a capture: the territory and ContentFinderCondition, where retail put the
player (ZoneInit's position), which way the client said it faced in its first position update, and the circle (the
first object with a bound layout that retail hid with ActorControl 106 [7] in the duty). `check` says where the
server doesn't know the circle or the facing. The observations are numbers only, kept in
`tools/questkit/questkit/data/entrances.json` so the test suite runs the check without the captures
(`questkit chronofoil <captures> --layers --save` adds a capture's).

Layouts (Kawari's, core/src/ipc/zone): ZoneInit territory u16 @2, ContentFinderCondition u16 @6, position 3 × f32
@104. SpawnObject base u32 @4, actor u32 @8, layout u32 @12, bound layout u32 @24, position 3 × f32 @52.
ActorControl category u16 @0, first parameter u32 @4 (the object is the segment's source actor).
UpdatePositionHandler(Instance) rotation f32 @0.
"""

from __future__ import annotations

import json
import math
import re
import struct
from pathlib import Path

from . import repo
from .chronofoil import Names

OBSERVATIONS = Path(__file__).resolve().parent / "data" / "entrances.json"
GAME_RS = repo.ROOT / "core" / "src" / "common" / "game.rs"
ENTRANCE_FACING = repo.ROOT / "resources" / "data" / "entrance_facing.json"
HIDE = (106, 7)
# The first position update counts as the entrance's facing only this soon after the zone loaded (as
# entrance_facing.json's were taken), before the player could have turned.
FACING_WITHIN = 3.0
# A duty entry is read this long: retail hid the slowest circle 108 s in (a guildhest's.)
DUTY_WINDOW = 600.0
# The circle stands by where players come in (2.8 to 6.3 yalms in the captures.)
CIRCLE_WITHIN = 15.0


def observe(capture, tables: dict) -> list[dict]:
    """Each duty entry in a capture: where retail put the player, which way they faced, and the circle it hid."""
    names = Names(tables, capture.game_version)
    entries: list[dict] = []
    current = None
    objects: dict[int, tuple] = {}

    def finish():
        if current is not None:
            entries.append(current)

    for segment in capture.segments(protocols=("zone",)):
        name = names.name(segment)
        d = segment.data
        if segment.dir == "out" and name == "ZoneInit" and len(d) >= 116:
            finish()
            territory, cfc = struct.unpack_from("<H", d, 2)[0], struct.unpack_from("<H", d, 6)[0]
            current = None
            objects = {}
            if cfc:
                current = {
                    "capture": capture.id[:8],
                    "territory": territory,
                    "content_finder_condition": cfc,
                    "position": [round(v, 3) for v in struct.unpack_from("<3f", d, 104)],
                    "facing": None,
                    "circle": None,
                    "_t": segment.t,
                }
            continue
        if current is None or segment.t - current["_t"] > DUTY_WINDOW:
            continue
        if segment.dir == "out" and name == "SpawnObject" and len(d) >= 64:
            base, actor, layout = struct.unpack_from("<III", d, 4)
            bound = struct.unpack_from("<I", d, 24)[0]
            objects[actor] = (base, layout, bound, struct.unpack_from("<3f", d, 52))
        elif (segment.dir == "out" and name == "ActorControl" and len(d) >= 8 and current["circle"] is None
              and segment.src in objects and struct.unpack_from("<HxxI", d, 0) == HIDE):
            base, layout, bound, where = objects[segment.src]
            if bound and math.dist(where, current["position"]) <= CIRCLE_WITHIN:
                current["circle"] = {"base": base, "layout": layout, "bound": bound,
                                     "hidden_after": round(segment.t - current["_t"], 1)}
        elif (segment.dir == "in" and name in ("UpdatePositionHandler", "UpdatePositionHandlerInstance")
              and current["facing"] is None and len(d) >= 4):
            current["facing"] = (round(struct.unpack_from("<f", d, 0)[0], 4)
                                 if segment.t - current["_t"] <= FACING_WITHIN else "late")
    finish()
    for entry in entries:
        del entry["_t"]
        if entry["facing"] == "late":
            entry["facing"] = None
    return entries


def known_circles() -> set[int]:
    """`ENTRANCE_CIRCLE_IDS` as the server has it."""
    text = GAME_RS.read_text()
    found = re.search(r"pub const ENTRANCE_CIRCLE_IDS: \[u32; \d+\] = \[(.*?)\];", text, re.S)
    if not found:
        raise ValueError(f"ENTRANCE_CIRCLE_IDS isn't in {GAME_RS}")
    body = re.sub(r"//[^\n]*", "", found.group(1))
    return {int(number) for number in re.findall(r"\d+", body)}


def known_facings() -> dict[int, float]:
    return {int(territory): facing
            for territory, facing in json.loads(ENTRANCE_FACING.read_text())["territories"].items()}


def check(observations: list[dict]) -> list[str]:
    """Where retail hid a circle the server doesn't know, or turned players a way it doesn't know or knows otherwise."""
    circles = known_circles()
    facings = known_facings()
    problems = []
    for seen in observations:
        where = (f"capture {seen['capture']}, territory {seen['territory']} "
                 f"(ContentFinderCondition {seen['content_finder_condition']})")
        circle = seen.get("circle")
        if circle and circle["base"] not in circles:
            problems.append(f"{where}: retail's entrance circle is EObj {circle['base']} (layout {circle['layout']}, "
                            f"hidden {circle['hidden_after']} s in; players at {seen['position']}); "
                            "ENTRANCE_CIRCLE_IDS hasn't it, so players come in at a random pop range")
        facing = seen.get("facing")
        if facing is not None:
            ours = facings.get(seen["territory"])
            if ours is None:
                problems.append(f"{where}: retail turned players to {facing}; entrance_facing.json has no entry")
            elif abs(ours - facing) > 0.001:
                problems.append(f"{where}: retail turned players to {facing}; entrance_facing.json says {ours}")
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
        "comment": "Every duty entry in the Chronofoil captures read so far: where retail put the player, the "
                   "facing in the client's first position update (null when it came more than 3 s in), and the "
                   "entrance circle retail hid with ActorControl 106 [7] (numbers only; `questkit chronofoil "
                   "<captures> --layers --save` adds a capture's). The test suite checks ENTRANCE_CIRCLE_IDS "
                   "(core/src/common/game.rs) has each circle and resources/data/entrance_facing.json each facing.",
        "observations": observations,
    }, indent=1)
    OBSERVATIONS.write_text(text + "\n")


def report(observations: list[dict]) -> str:
    lines = []
    for seen in observations:
        circle = seen.get("circle")
        lines.append(f"{seen['capture']}  territory {seen['territory']:>4}  CFC {seen['content_finder_condition']:>5}  "
                     f"at {seen['position']}  facing {seen.get('facing')}  "
                     f"circle {circle['base'] if circle else '-'}")
    return "\n".join(lines)
