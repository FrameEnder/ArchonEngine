"""How often an enemy's rotation would cast each action, against how often retail's did, in every duty fixture.

Bob's run of Castrum Meridianum on 2026-11-21: "the area of effect data it's far too common … this should've been
obvious from just comparing the retail captures". The duty drafts (`questkit sources --duty N --captures …`) gave an
action `every` only when one NPC had used it at least three times, so an area attack retail's Imperial casters used
once or twice a fight had none and came every turn the NPC was free: 14570's 29171, cast 1 to 2 times in each of five
retail runs, was cast 29 times in 240 s by each caster on the server. This reads the casts in each duty fixture
(`tools/questkit/duties/*.json`: who cast what when, each NPC's base, pull and death) and each NPC's timeline
(`resources/timelines/*_<base>.json`), and says where the rotation would cast an action far more often than retail did
over the same lives, or before retail ever did.

A rotation entry's most uses over a fight of L seconds (servers/world/src/server/rotation.rs): with `every` e and
`not_before` n, 1 + (L - n) // e once L reaches n; with no `every`, a turn of every other `pace` (it shares the free
turns with the NPC's other actions; an upper bound would be every turn). Retail's are counted per life from its pull
(the NPC's fight; the server counts `not_before` from its first action, which is the pull in the fixtures to a second
or so.) Only actions with a cast bar are in fixtures (`casts`, ActorCast), which are the ones that show: instant
weaponskills aren't read.
"""

from __future__ import annotations

import json
import math
import re
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path
from statistics import median

from . import repo

DUTIES = repo.ROOT / "tools" / "questkit" / "duties"
TIMELINES = repo.ROOT / "resources" / "timelines"
# An action is flagged when the rotation would use it more than this many times retail's count over the same lives
# (and at least TOO_MANY_OVER more): rotations are judged by their most, retail's runs differ.
TOO_MANY = 2.0
TOO_MANY_OVER = 3
# `chance` is given when fewer than this share of the lives that fought long enough used it, and there were this many.
CHANCE_BELOW = 0.8
CHANCE_LIVES = 4
# A first use past this many times the median of its kind's is left out of `first_uses` (`usual`.)
LATE_AFTER = 3.0
# A rotation's first use may come this much before retail's first (seconds): the pull is read to about a second.
EARLY_SLACK = 5.0


@dataclass
class Rate:
    base: int
    action: int
    timeline: str
    lives: int = 0  # retail lives of the base that fought (were pulled)
    retail_uses: int = 0
    rotation_uses: float = 0
    retail_first: float | None = None  # earliest use after a pull, seconds
    rotation_first: float | None = None
    problems: list[str] = field(default_factory=list)
    actions: list[int] = field(default_factory=list)  # the entry's action, or its slot's


def timelines_by_base() -> dict[int, Path]:
    found = {}
    for path in TIMELINES.glob("*.json"):
        match = re.search(r"_(\d+)\.json$", path.name)
        if match:
            found[int(match.group(1))] = path
    return found


def waits(entry: dict) -> float:
    """The soonest an entry comes in a fight: the earliest of its `first_uses` (one is drawn each fight), or its
    `not_before`."""
    if entry.get("first_uses"):
        return min(entry["first_uses"])
    return entry.get("not_before", 0.0) or 0.0


def most_uses(entry: dict, pace: float, seconds: float) -> int:
    """The most times a rotation entry is used in a fight of `seconds` (rotation.rs's rules, see the module's text)."""
    start = waits(entry)
    if seconds < start:
        return 0
    every = entry.get("every")
    if every is None:
        every = 2 * max(pace, 0.5)
    return 1 + int((seconds - start) // max(every, 0.5))


def lives(fixtures: list[Path] | None = None):
    """(fixture name, base, layout, pulled, end, [(seconds from pull, action)]) for every life that fought."""
    for path in sorted(fixtures or DUTIES.glob("*-*.json")):
        fixture = json.loads(path.read_text())
        casts = defaultdict(list)
        for at, layout, action in fixture.get("casts", []):
            casts[str(layout)].append((at, action))
        for layout, npc_lives in fixture.get("npcs", {}).items():
            for life in npc_lives:
                base, pulled = life.get("base"), life.get("pulled")
                if base is None or pulled is None:
                    continue
                end = life.get("death") or life.get("gone")
                if end is None:
                    continue
                used = [(at - pulled, action) for at, action in casts.get(layout, []) if pulled - 1 <= at <= end]
                yield path.stem, base, int(layout), pulled, end, used


def groups(rotation: dict) -> list[tuple[str, list[dict]]]:
    """A rotation's entries as the turns they take: one entry each, or the entries of one `slot` together."""
    out: list[tuple[str, list[dict]]] = []
    slots: dict[str, list[dict]] = {}
    for entry in rotation.get("actions", []):
        slot = entry.get("slot")
        if slot is None:
            out.append((str(entry["action"]), [entry]))
        elif slot in slots:
            slots[slot].append(entry)
        else:
            slots[slot] = [entry]
            out.append((f"slot {slot}", slots[slot]))
    return out


def compare(fixtures: list[Path] | None = None) -> list[Rate]:
    """Each rotation entry (or slot) of each fixture's NPCs: retail's uses against the rotation's most over the same
    lives (times its `chance`, when it has one: what it comes to on average.)"""
    paths = timelines_by_base()
    loaded: dict[int, dict] = {}
    rates: dict[tuple[int, str], Rate] = {}
    cast_somewhere = set()
    for _, base, _, pulled, end, used in lives(fixtures):
        cast_somewhere |= {(base, action) for _, action in used}
        path = paths.get(base)
        if path is None:
            continue
        timeline = loaded.setdefault(base, json.loads(path.read_text()))
        rotation = timeline.get("rotation")
        if not rotation:
            continue
        seconds = end - pulled
        for name, entries in groups(rotation):
            actions = [entry["action"] for entry in entries]
            rate = rates.setdefault((base, name), Rate(base, actions[0], path.name, actions=actions))
            rate.lives += 1
            mine = [at for at, cast in used if cast in actions]
            rate.retail_uses += len(mine)
            if mine:
                first = max(min(mine), 0.0)
                rate.retail_first = first if rate.retail_first is None else min(rate.retail_first, first)
            entry = entries[0]
            would = most_uses(entry, rotation.get("pace", 3.0), seconds) * entry.get("chance", 1.0)
            rate.rotation_uses += would
            if would:
                first = waits(entry)
                rate.rotation_first = first if rate.rotation_first is None else min(rate.rotation_first, first)
    out = []
    for _, rate in sorted(rates.items()):
        # Only actions retail cast with a bar (the fixtures' `casts`); an instant one is never in them.
        if not any((rate.base, action) in cast_somewhere for action in rate.actions):
            continue
        rate.rotation_uses = round(rate.rotation_uses, 1)
        if rate.rotation_uses > max(rate.retail_uses * TOO_MANY, rate.retail_uses + TOO_MANY_OVER):
            rate.problems.append(f"{rate.rotation_uses:g} uses at most over {rate.lives} lives where retail's cast it "
                                 f"{rate.retail_uses} times: needs `every` (and `not_before`) from retail's")
        if (rate.retail_first is not None and rate.rotation_first is not None
                and rate.rotation_first + EARLY_SLACK < rate.retail_first):
            rate.problems.append(f"first use {rate.rotation_first:g} s into a fight; retail's earliest was "
                                 f"{rate.retail_first:.1f} s")
        out.append(rate)
    return out


def retail_numbers(base: int, actions: list[int], fixtures: list[Path] | None = None) -> dict:
    """What retail's lives of `base` say of `actions` (one, or a slot's): the median time between one NPC's uses
    (`every`), or, never used twice by one, the longest life any of them had (once a fight, as far as anyone saw); the
    median first use after the pull (`not_before`: half came later, half earlier, the earliest at `earliest`); and
    `chance`, the share of the lives that lived past `not_before` that used it at all, when under CHANCE_BELOW (and
    there are CHANCE_LIVES of them to tell)."""
    gaps, firsts, spans = [], [], []
    for _, life_base, _, pulled, end, used in lives(fixtures):
        if life_base != base:
            continue
        spans.append(end - pulled)
        mine = sorted(at for at, cast in used if cast in actions)
        if not mine:
            continue
        firsts.append(max(mine[0], 0.0))
        gaps += [b - a for a, b in zip(mine, mine[1:])]
    if not firsts:
        return {}
    uses, users = len(firsts) + len(gaps), len(firsts)
    firsts = usual(firsts)
    numbers = {"uses": uses, "lives": users,
               "not_before": round(median(firsts), 1), "earliest": round(min(firsts), 1),
               "latest": round(max(firsts), 1), "first_uses": sorted(round(first, 1) for first in firsts)}
    if gaps:
        numbers["every"] = round(median(gaps), 1)
        numbers["from"] = f"the median of {len(gaps)} times between one's uses"
    else:
        numbers["every"] = float(math.ceil(max(spans)))
        numbers["from"] = "never twice by one: the longest any of them fought"
    lived = sum(1 for span in spans if span >= numbers["not_before"])
    if lived >= CHANCE_LIVES and users / lived < CHANCE_BELOW:
        numbers["chance"] = round(users / lived, 2)
        numbers["lived"] = lived
    return numbers


def usual(firsts: list[float]) -> list[float]:
    """First uses without the far-late ones (past LATE_AFTER times the median, with three or more): an NPC that had
    someone targeted long before it fought. The Praetorium's 14562, 14563 and 14560 in Bob's run 6320ae56 first cast
    29180, 29175 and 29174 94 to 128 s after their pull where the others did 7 to 45 s in; drawn, one such would make
    an NPC hold its attack a minute and a half."""
    if len(firsts) < 3:
        return firsts
    middle = median(firsts)
    return [first for first in firsts if first <= LATE_AFTER * middle]


def fix(rates: list[Rate], fixtures: list[Path] | None = None, when: str = "") -> list[str]:
    """Gives each flagged entry (or slot) retail's `every`, `not_before` and `chance` (`retail_numbers`), in its
    timeline file; returns what it changed."""
    changed = []
    by_file: dict[str, list[Rate]] = defaultdict(list)
    for rate in rates:
        if rate.problems:
            by_file[rate.timeline].append(rate)
    for name, flagged in sorted(by_file.items()):
        path = TIMELINES / name
        text = path.read_text()
        timeline = json.loads(text)
        for rate in flagged:
            numbers = retail_numbers(rate.base, rate.actions, fixtures)
            if not numbers:
                continue
            too_many = any("uses at most" in problem for problem in rate.problems)
            for entry in timeline["rotation"]["actions"]:
                if entry["action"] not in rate.actions:
                    continue
                kept = not (too_many or "every" not in entry)
                if not kept:
                    entry["every"] = numbers["every"]
                # Each NPC draws one of retail's own first uses each fight (`first_uses`), so those of a kind
                # don't all begin at one second (Bob, 2026-11-21: "have every caster of a kind vary like retail").
                entry.pop("not_before", None)
                entry["first_uses"] = numbers["first_uses"]
                if "chance" in numbers:
                    entry["chance"] = numbers["chance"]
                if entry["action"] == rate.actions[0]:
                    chance = (f"; used by {numbers['lives']} of the {numbers['lived']} that fought past then "
                              f"(`chance` {numbers['chance']:g})" if "chance" in numbers else "")
                    note = (f"Retail (questkit rates{', ' + when if when else ''}: {numbers['uses']} casts by "
                            f"{numbers['lives']} of the duty fixtures' NPCs): first {numbers['earliest']:g} to "
                            f"{numbers['latest']:g} s after the pull (median {numbers['not_before']:g}; `first_uses`: "
                            f"each NPC draws one of retail's each fight), "
                            + (f"`every` kept (retail's: {numbers['every']:g} s, {numbers['from']})" if kept else
                               f"`every` {numbers['every']:g} s ({numbers['from']})")
                            + f"{chance}. Before, the rotation would have cast it {rate.rotation_uses:g} "
                            f"times at most over lives retail's cast it {rate.retail_uses}.")
                    entry["comment"] = (entry.get("comment", "") + ". " + note).lstrip(". ")
                if "comment" in entry:
                    # The comment last, as the files keep it.
                    entry["comment"] = entry.pop("comment")
                changed.append(f"{name}: {entry['action']}: every {entry['every']:g}, first "
                               f"{numbers['earliest']:g}-{numbers['latest']:g}" + (f", chance {numbers['chance']:g}"
                                                                 if "chance" in numbers else ""))
        indent = 4 if text.startswith("{\n    ") else 2
        path.write_text(json.dumps(timeline, indent=indent, ensure_ascii=False) + "\n")
    return changed


def report(rates: list[Rate], everything: bool = False) -> str:
    lines = [f"{'timeline':40} {'action':>6} {'lives':>5} {'retail':>6} {'most':>5} {'first':>6}  problems"]
    for rate in rates:
        if not everything and not rate.problems:
            continue
        first = "-" if rate.retail_first is None else f"{rate.retail_first:.1f}"
        lines.append(f"{rate.timeline[:40]:40} {rate.action:>6} {rate.lives:>5} {rate.retail_uses:>6} "
                     f"{rate.rotation_uses:>5g} {first:>6}  {'; '.join(rate.problems)}")
    return "\n".join(lines)
