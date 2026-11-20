"""Is a boss's event set off by its clock or by its health? Read across retail runs of the same fight.

Bob, 2026-11-20 (Castrum Meridianum's first boss): "most things I've observed around timings for events and add
spawns as well as phases in the fight are usually timed by the bosses health not on an actual timer … check for this
behavior more often". This puts both side by side for every event of a boss in every run given: each cast of an
action (the first, second, … of it), each battle talk, each wave (battle NPCs first seen together during the fight),
each map effect, at seconds from the pull and at the boss's health then. An event the clock sets off comes at nearly
the same second in every run, its health spread wide (a slow party leaves the boss healthier); one its health sets off
comes at nearly the same health, its seconds spread wide. The verdict says which spread is the narrower for its kind,
from the runs given: two runs can't tell much; the more and the more different the parties, the better.

Run exports (`questkit chronofoil <capture> --battle <visit> --out run.json.gz`) are what it reads: `Actors` (BaseId,
MaxHp, Seen, Died), `Samples` (health over time), `Actions`, `Events` (casts, talks, map effects).
"""

from __future__ import annotations

import bisect
import gzip
import json
import re
from collections import defaultdict
from pathlib import Path
from statistics import median

# A boss's clock in retail keeps to a few seconds between runs (its pull is read to about a second); its health line
# to a few percent (the boss is struck many times a second.) The spreads are weighed against these.
CLOCK_SLACK = 3.0  # seconds
HEALTH_SLACK = 3.0  # percent
# Battle NPCs first seen within this of each other in the fight are one wave.
WAVE_WITHIN = 1.0
# Kinds of event read (from a run's `Events`.)
EVENT_KINDS = ("battle talk", "map effect")


def load(path) -> dict:
    with gzip.open(path) if str(path).endswith(".gz") else open(path) as handle:
        return json.load(handle)


def fight(run: dict, boss_base: int) -> list[tuple[str, float, float | None]]:
    """The boss's fight in one run: (event, seconds from its pull, its health % then) in order."""
    actors = {int(key): actor for key, actor in run["Actors"].items()}
    boss = next((key for key, actor in actors.items() if actor.get("BaseId") == boss_base), None)
    if boss is None:
        return []
    maximum = actors[boss].get("MaxHp") or 1
    health = sorted((sample["S"], sample["Hp"]) for sample in run["Samples"]
                    if sample["Id"] == boss and sample.get("Hp") is not None)
    involved = [action["S"] for action in run["Actions"]
                if action["Source"] == boss or any(target["Id"] == boss for target in action["Targets"])]
    if not involved:
        return []
    pull = min(involved)
    end = actors[boss].get("Died") or max(involved)

    def health_at(t: float):
        index = bisect.bisect_right(health, (t, float("inf"))) - 1
        return round(health[index][1] / maximum * 100, 1) if index >= 0 else None

    events = []
    counts: dict[str, int] = defaultdict(int)
    for event in run["Events"]:
        if not pull - 1 <= event["S"] <= end:
            continue
        if event["Kind"] == "cast" and event["Id"] == boss:
            found = re.match(r"action (\d+)", event["Text"])
            if not found:
                continue
            name = f"cast {found.group(1)}"
        elif event["Kind"] in EVENT_KINDS:
            name = event["Kind"] + " " + re.sub(r" \d+ ms$", "", event["Text"]).strip()
        else:
            continue
        counts[name] += 1
        events.append((f"{name} #{counts[name]}", event["S"] - pull, health_at(event["S"])))

    # Waves: battle NPCs (not the boss, not Duty Support's) first seen together after the pull.
    seen = sorted((actor["Seen"], key) for key, actor in actors.items()
                  if actor.get("Kind") == "battle npc" and key != boss and actor.get("Seen") is not None
                  and pull < actor["Seen"] <= end and actor.get("BaseId") not in (14658, 14659, 14660)
                  and actor.get("LayoutId"))
    waves: list[list[tuple[float, int]]] = []
    for at, key in seen:
        if waves and at - waves[-1][0][0] <= WAVE_WITHIN:
            waves[-1].append((at, key))
        else:
            waves.append([(at, key)])
    for number, wave in enumerate(waves, 1):
        at = wave[0][0]
        layouts = sorted(actors[key]["LayoutId"] for _, key in wave)
        events.append((f"wave #{number} ({len(wave)}: {layouts[0]}…)", at - pull, health_at(at)))
    return sorted(events, key=lambda event: event[1])


def verdict(times: list[float], healths: list[float]) -> str:
    """Which the event goes by, from its spreads across runs."""
    if len(times) < 2:
        return "one run"
    time_spread = max(times) - min(times)
    health_spread = max(healths) - min(healths) if len(healths) == len(times) else None
    if health_spread is None:
        return "?"
    clock = time_spread / CLOCK_SLACK
    life = health_spread / HEALTH_SLACK
    if clock <= 1 and life > 1.5:
        return "clock"
    if life <= 1 and clock > 1.5:
        return "health"
    if clock <= 1 and life <= 1:
        return "either (the runs went alike)"
    return "neither alone (" + ("rather the clock" if clock < life else "rather health") + ")"


def compare(paths: list, boss_base: int) -> list[dict]:
    """Every event of the boss's fight seen in at least one run, with its seconds and health in each run."""
    by_event: dict[str, list[tuple[str, float, float | None]]] = defaultdict(list)
    order: dict[str, float] = {}
    for path in paths:
        name = Path(path).name.split(".")[0]
        for event, t, health in fight(load(path), boss_base):
            by_event[event].append((name, t, health))
            order.setdefault(event, t)
    rows = []
    for event in sorted(by_event, key=lambda event: median(t for _, t, _ in by_event[event])):
        runs = by_event[event]
        times = [t for _, t, _ in runs]
        healths = [health for _, _, health in runs if health is not None]
        rows.append({
            "event": event,
            "runs": len(runs),
            "seconds": [round(t, 1) for t in times],
            "health": healths,
            "verdict": verdict(times, healths),
        })
    return rows


def report(rows: list[dict]) -> str:
    lines = [f"{'event':58} {'runs':>4}  {'seconds from pull':30} {'boss health %':34} verdict"]
    for row in rows:
        seconds = " ".join(f"{t:.1f}" for t in row["seconds"])
        health = " ".join(f"{h:.1f}" for h in row["health"])
        lines.append(f"{row['event'][:58]:58} {row['runs']:>4}  {seconds[:30]:30} {health[:34]:34} "
                     f"{row['verdict']}")
    return "\n".join(lines)
