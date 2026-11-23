"""Which of a duty's enemies fought from one spot on retail (`questkit chronofoil <captures> --spots`).

Retail's Magitek Vanguard F-1 (Castrum Meridianum's second boss) walked to its arena's exact middle in the 3 s after
its pull and never moved again, every cast and auto-attack from there; Kawari's chased the bots out of its arena (Bob's
marks of 2026-11-22). A timeline says so with `fights_from` (core/src/common/timeline.rs). This reads each duty visit
of the captures (positions from ActorMove and ActorSetPos, as `chronofoil_battle` does) and, for every enemy that cast
at least `MIN_CASTS` times, tells:

* held: from its first cast to its last sighting it stayed within `HELD_WITHIN` yalms of one spot, for `MIN_HELD`
  seconds or more;
* walked there: before its first cast it moved more than `WALKED` yalms from where it was when pulled (its first
  target) to that spot, and came to rest there. One that held the spot where its tank found it (the Black Eft, which
  lunged at its tank and came back) only had no reason to move: that isn't `fights_from`;
* its arena's middle: the spot is within `AT_MIDDLE` of the middle of its arena as Boss Mod has it (its BNpcBase's
  modules' arenas). The Vanguard's was 0.03 yalms off; Haukke Manor's Manor Claviger walked 7.4 yalms and held a spot
  1.4 off hers, which is where a tank leaves a boss, and allies (Papalymo, Galfrid, Kuplu Kopo) stand to cast, with no
  arena of their own. Only one at its arena's middle fights from it.

and checks its timeline (`resources/timelines/*_<BNpcBase>.json`): one that walked to a spot and held it without
`fights_from` there is MISSING; a `fights_from` more than `HELD_WITHIN` off the spot is OFF. Numbers only.
"""

from __future__ import annotations

import json
import math
import statistics
from pathlib import Path

from . import repo
from .chronofoil import Capture, summarize
from .chronofoil_battle import run_from_visit

MIN_CASTS = 3
AT_MIDDLE = 1.0
HELPER_HP = 100
HELD_WITHIN = 0.5
MIN_HELD = 30.0
WALKED = 2.0


def _arena_middles(base: int) -> list[tuple[float, float]]:
    from .public_sources import Public

    if not hasattr(_arena_middles, "public"):
        _arena_middles.public = Public()
    return [(module["arena"]["x"], module["arena"]["z"]) for module in _arena_middles.public.by_base.get(base, [])
            if "arena" in module]


def _timeline(base: int) -> tuple[Path, dict] | None:
    for path in sorted((repo.ROOT / "resources" / "timelines").glob(f"*_{base}.json")):
        try:
            return path, json.loads(path.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            return None
    return None


def observe(capture: Capture, tables: dict) -> list[dict]:
    """Every casting enemy of the capture's duty visits, with where it held (if it did) and whether it walked there."""
    seen = []
    for visit in summarize(capture, tables).visits:
        if not visit.content:
            continue
        run = run_from_visit(capture, tables, visit)
        casts: dict[int, list[float]] = {}
        for event in run["Events"]:
            if event.get("Kind") == "cast":
                casts.setdefault(event["Id"], []).append(event["S"])
        for key, actor in run["Actors"].items():
            actor_id = int(key)
            # Not the helpers and launchers (44 HP, BNpcBase 9020 and 6358): the director puts those where they go.
            if actor["Kind"] != "battle npc" or actor["MaxHp"] <= HELPER_HP or len(casts.get(actor_id, [])) < MIN_CASTS:
                continue
            first = casts[actor_id][0]
            samples = [s for s in run["Samples"] if s["Id"] == actor_id]
            positions = [(s["S"], s["P"]) for s in samples if "P" in s]
            after = [(t, p) for t, p in positions if t >= first]
            if len(after) < 3:
                continue
            spot = (statistics.median(p[0] for _, p in after), statistics.median(p[1] for _, p in after),
                    statistics.median(p[2] for _, p in after))
            off = max(math.hypot(p[0] - spot[0], p[2] - spot[2]) for _, p in after)
            span = after[-1][0] - first
            pulled = next((s["S"] for s in samples if s.get("T")), None)
            before = [p for t, p in positions if pulled is not None and pulled <= t < first]
            at_pull = next((p for t, p in reversed(positions) if pulled is not None and t <= pulled), None)
            start = at_pull or (before[0] if before else actor["Position"])
            walked = math.hypot(start[0] - spot[0], start[2] - spot[2])
            seen.append({
                "capture": capture.id[:8], "territory": visit.territory, "content": visit.content,
                "base": actor["BaseId"], "hp": actor["MaxHp"], "casts": len(casts[actor_id]),
                "spot": [round(v, 2) for v in spot], "off": round(off, 2), "span": round(span, 1),
                "held": off <= HELD_WITHIN and span >= MIN_HELD, "walked": round(walked, 2),
            })
    return seen


def report(seen: list[dict]) -> tuple[str, list[str]]:
    """Each enemy, by duty, and what its timeline is missing (MISSING / OFF lines.)"""
    lines, problems = [], []
    for entry in sorted(seen, key=lambda e: (e["content"], e["base"], e["capture"])):
        middles = _arena_middles(entry["base"])
        at_middle = any(math.hypot(entry["spot"][0] - x, entry["spot"][2] - z) <= AT_MIDDLE for x, z in middles)
        holds = entry["held"] and entry["walked"] > WALKED and at_middle
        what = ("walked %.1f yalms to its arena's middle (%.2f, %.2f, %.2f) and fought from there"
                % (entry["walked"], *entry["spot"]) if holds else
                "walked %.1f yalms to (%.2f, %.2f, %.2f) and held it, not its arena's middle%s" % (
                    entry["walked"], *entry["spot"], "" if middles else " (no arena known)")
                if entry["held"] and entry["walked"] > WALKED else
                "held where it stood (%.1f yalms from its pull's spot)" % entry["walked"] if entry["held"] else
                "too short a fight to tell (%.0f s)" % entry["span"] if entry["off"] <= HELD_WITHIN else
                "moved about (%.1f yalms off its middle)" % entry["off"])
        lines.append(f"duty {entry['content']} (territory {entry['territory']}) BNpcBase {entry['base']} "
                     f"({entry['hp']} HP, {entry['casts']} casts over {entry['span']:.0f} s, capture "
                     f"{entry['capture']}): {what}")
        found = _timeline(entry["base"])
        fights_from = found[1].get("fights_from") if found else None
        if holds and fights_from is None:
            problems.append(f"BNpcBase {entry['base']}: retail fought from {entry['spot']}; its timeline "
                            f"({found[0].name if found else 'none'}) has no fights_from")
        elif fights_from is not None and math.hypot(fights_from[0] - entry["spot"][0],
                                                    fights_from[2] - entry["spot"][2]) > HELD_WITHIN and entry["held"]:
            problems.append(f"BNpcBase {entry['base']}: fights_from {fights_from} is off retail's {entry['spot']}")
    return "\n".join(lines), problems
