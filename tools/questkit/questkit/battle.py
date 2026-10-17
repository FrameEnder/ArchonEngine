"""Reading a Kawari Recorder battle (battles.json, Recorder 2.4+) and turning it into rules with their evidence.

The scripts behind a duty's director and its NPCs stay on Square Enix's servers, so a battle's logic is inferred from
what everyone did. This lays out what the recording shows so that inference is checked, not guessed:

* who is in it, which side they're on (battalion), and the client's layout data for them (layer, instance name,
  NormalAI, MoveAi, ServerPathId, link group, pop settings);
* the timeline: spawns grouped into waves (with what had just happened: deaths, everyone's HP), deaths, the
  director's sequence and variables, battle talk, cutscenes, map effects and your marks;
* for each NPC, who it targeted and when, and for every change of target which explanations fit (the old target
  died, the new one just spawned, it's the newest/oldest/nearest/weakest enemy, it had just hit them...), counted
  over every change, so a rule like "goes for the newest add" comes with how often it held;
* what each NPC used, how often and on whom, and for heals how hurt the target was (and whether it was the most hurt
  ally at the time);
* who never did anything (stood still, never acted), which is how a missing script shows up.
"""

from __future__ import annotations

import bisect
import gzip
import json
import math
import zipfile
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from pathlib import Path

EFFECT_MISS = 1
EFFECT_DAMAGE = 3
EFFECT_HEAL = 4
DAMAGE_EFFECTS = (3, 5, 6)

# How close in time one thing has to be to another to count as its cause.
DEATH_WINDOW = 1.5
SPAWN_WINDOW = 3.0
HIT_WINDOW = 6.0
WAVE_GAP = 2.0


@dataclass
class Track:
    """One field of one actor over time, from the samples (only changes are recorded)."""

    times: list[float] = field(default_factory=list)
    values: list = field(default_factory=list)

    def add(self, t: float, value):
        self.times.append(t)
        self.values.append(value)

    def at(self, t: float, default=None):
        i = bisect.bisect_right(self.times, t) - 1
        return self.values[i] if i >= 0 else default


class Battle:
    def __init__(self, run: dict):
        self.run = run
        self.actors: dict[int, dict] = {int(k): v for k, v in (run.get("Actors") or {}).items()}
        self.samples = run.get("Samples") or []
        self.actions = run.get("Actions") or []
        self.events = run.get("Events") or []
        self.controls = run.get("Controls") or []
        self.layers = run.get("Layers") or []
        self.you = next((i for i, a in self.actors.items() if a.get("Kind") == "you"), None)
        self.action_info = {int(k): v for k, v in (run.get("ActionInfo") or {}).items()}
        self.tethers: dict[int, Track] = defaultdict(Track)

        self.hp: dict[int, Track] = defaultdict(Track)
        self.pos: dict[int, Track] = defaultdict(Track)
        self.target: dict[int, Track] = defaultdict(Track)
        self.dead: dict[int, Track] = defaultdict(Track)
        for s in self.samples:
            i, t = s["Id"], s["S"]
            if "Hp" in s:
                self.hp[i].add(t, s["Hp"])
            if "P" in s:
                self.pos[i].add(t, s["P"])
            if "T" in s:
                self.target[i].add(t, s["T"])
            if "Dead" in s:
                self.dead[i].add(t, s["Dead"])
            if "Tethers" in s:
                self.tethers[i].add(t, s["Tethers"])

        self.layout: dict[int, tuple[dict, dict]] = {}
        for layer in self.layers:
            for obj in layer.get("Objects") or []:
                self.layout.setdefault(obj.get("Id"), (layer, obj))

        self.length = max(
            [s["S"] for s in self.samples] + [a["S"] for a in self.actions] + [e["S"] for e in self.events] + [0]
        )

    # ---- who's who ----

    def engaged(self, i: int) -> bool:
        """Whether this NPC is really part of your fight: it dies here, loses a real share of its HP (5% or more),
        you hit it, or it goes after you."""
        if getattr(self, "_engaged", None) is None:
            hit = {t["Id"] for act in self.actions if act["Source"] == self.you for t in act.get("Targets") or []}
            after_you = {i for i, track in self.target.items() if self.you in track.values}
            died = {i for i, a in self.actors.items() if a.get("Died") is not None}
            died |= {i for i, track in self.dead.items() if any(track.values)}
            hurt = {i for i, track in self.hp.items() if self.actors.get(i, {}).get("MaxHp")
                    and min(v for v in track.values if v is not None) <= 0.95 * self.actors[i]["MaxHp"]}
            self._engaged = hit | after_you | died | hurt
        return i in self._engaged

    @property
    def in_fight(self) -> set[int]:
        """Everyone linked to you by hits, heals or targets, directly or through others: your fight, without the
        scenery (NPCs fighting each other in the background, as in To Guard a Guardian, never touch it.) Everyone,
        when there's no "you"."""
        if getattr(self, "_in_fight", None) is not None:
            return self._in_fight
        if self.you is None:
            self._in_fight = set(self.actors)
            return self._in_fight
        links: dict[int, set[int]] = defaultdict(set)
        for act in self.actions:
            for target in act.get("Targets") or []:
                if target["Id"] != act["Source"]:
                    links[act["Source"]].add(target["Id"])
                    links[target["Id"]].add(act["Source"])
        for i, track in self.target.items():
            for value in track.values:
                if value and value != i:
                    links[i].add(value)
                    links[value].add(i)
        seen, todo = {self.you}, [self.you]
        while todo:
            for other in links[todo.pop()]:
                if other not in seen:
                    seen.add(other)
                    todo.append(other)
        self._in_fight = seen
        return seen

    def name(self, i: int | None) -> str:
        if not i:
            return "no one"
        a = self.actors.get(i)
        if a is None:
            return f"#{i:X}"
        if a.get("Kind") == "you":
            return "you"
        if a.get("Kind") == "player":
            return f"player #{i:X}"
        label = a.get("Name") or f"base {a.get('BaseId')}"
        return f"{label} [{a['LayoutId']}]" if a.get("LayoutId") else label

    def is_fighter(self, i: int) -> bool:
        a = self.actors.get(i, {})
        return a.get("Kind") in ("you", "player", "battle npc") and a.get("MaxHp", 0) > 0

    def side(self, i: int):
        return self.actors.get(i, {}).get("Battalion")

    def hostile(self, a: int, b: int) -> bool:
        """Different battalions fight each other (the Battalion sheet has the exceptions; this is the usual case)."""
        return self.is_fighter(a) and self.is_fighter(b) and self.side(a) != self.side(b)

    def alive(self, i: int, t: float) -> bool:
        a = self.actors.get(i)
        if a is None or a.get("Seen", 0) > t or a.get("LastSeen", 0) < t - 0.5:
            return False
        if self.dead[i].at(t, False):
            return False
        return (self.hp[i].at(t, 1) or 0) > 0

    def hp_percent(self, i: int, t: float) -> float | None:
        a = self.actors.get(i, {})
        hp = self.hp[i].at(t)
        if hp is None or not a.get("MaxHp"):
            return None
        return 100.0 * hp / a["MaxHp"]

    def distance(self, a: int, b: int, t: float) -> float | None:
        pa, pb = self.pos[a].at(t), self.pos[b].at(t)
        if pa is None or pb is None:
            return None
        return math.dist(pa, pb)

    def layout_info(self, layout_id: int) -> dict:
        """The client's layout settings for a layout ID: its layer, its instance name and the AI-related fields."""
        found = self.layout.get(layout_id)
        if not found:
            return {}
        layer, obj = found
        info = {"layer": layer.get("Name", ""), "instance": obj.get("Name", "")}
        data = obj.get("Data") or {}
        for key in ("NormalAI", "MoveAi", "ServerPathId", "LinkGroup", "LinkFamily", "LinkRange", "ActiveType",
                    "PopEvent", "Nonpop", "BoundInstanceID", "SenseRangeRate", "WanderingRange", "Route",
                    "EventGroup", "PopInterval", "TerritoryRange"):
            value = _find(data, key)
            if value not in (None, 0, 0.0, ""):
                info[key] = value
        return info

    def action_name(self, action: int) -> str:
        info = self.action_info.get(action) or {}
        return f"{action} ({info['Name']})" if info.get("Name") else str(action)

    # ---- what happened ----

    def spawn_times(self) -> dict[int, float]:
        return {i: a.get("Seen", 0.0) for i, a in self.actors.items()}

    def deaths(self) -> list[tuple[float, int]]:
        out = []
        for i, a in self.actors.items():
            if a.get("Died") is not None:
                out.append((a["Died"], i))
        return sorted(out)

    def waves(self) -> list[tuple[float, list[int]]]:
        """Battle NPCs that appeared together (within WAVE_GAP of each other), after the start."""
        spawns = sorted((a.get("Seen", 0.0), i) for i, a in self.actors.items()
                        if a.get("Kind") == "battle npc" and a.get("MaxHp", 0) > 0)
        waves: list[tuple[float, list[int]]] = []
        last = None
        for t, i in spawns:
            if waves and last is not None and t - last <= WAVE_GAP:
                waves[-1][1].append(i)
            else:
                waves.append((t, [i]))
            last = t
        return waves

    def target_segments(self, i: int) -> list[tuple[float, float, int]]:
        track = self.target[i]
        end = self.actors.get(i, {}).get("LastSeen", self.length)
        out = []
        for k, (t, target) in enumerate(zip(track.times, track.values)):
            until = track.times[k + 1] if k + 1 < len(track.times) else end
            out.append((t, until, target))
        return out

    def hits_on(self, victim: int, by: int, before: float, window: float) -> bool:
        for a in self.actions:
            if before - window <= a["S"] <= before and a["Source"] == by:
                if any(h["Id"] == victim for h in a["Targets"]):
                    return True
        return False

    def switch_reasons(self, i: int, t: float, old: int, new: int) -> list[str]:
        """Which explanations fit NPC i changing its target from old to new at time t."""
        reasons = []
        if old and not self.alive(old, t + 0.1):
            reasons.append("old target dead or gone")
        elif old and any(abs(d - t) <= DEATH_WINDOW and who == old for d, who in self.deaths()):
            reasons.append("old target dead or gone")
        if not old:
            reasons.append("had no target")
        spawned = self.actors.get(new, {}).get("Seen", -99)
        if 0 <= t - spawned <= SPAWN_WINDOW:
            reasons.append("new one had just appeared")

        candidates = [c for c in self.actors if c != i and self.hostile(i, c) and self.alive(c, t)]
        if new in candidates and len(candidates) > 1:
            by_spawn = sorted(candidates, key=lambda c: self.actors[c].get("Seen", 0))
            if by_spawn[-1] == new:
                reasons.append("newest enemy")
            if by_spawn[0] == new:
                reasons.append("oldest enemy")
            distances = {c: self.distance(i, c, t) for c in candidates}
            known = {c: d for c, d in distances.items() if d is not None}
            if known and min(known, key=known.get) == new:
                reasons.append("nearest enemy")
            hps = {c: self.hp_percent(c, t) for c in candidates}
            hps = {c: h for c, h in hps.items() if h is not None}
            if hps and min(hps, key=hps.get) == new:
                reasons.append("most hurt enemy")
            current = {c: self.hp[c].at(t) for c in candidates}
            current = {c: h for c, h in current.items() if h is not None}
            if current and max(current, key=current.get) == new:
                reasons.append("healthiest enemy")
            if new == self.you:
                reasons.append("the player")
            elif self.actors.get(new, {}).get("Kind") == "battle npc" and self.you in candidates:
                reasons.append("an NPC over the player")
        if self.hits_on(i, new, t, HIT_WINDOW):
            reasons.append("new one had just hit it")
        for ally in self.actors:
            if ally != i and not self.hostile(i, ally) and self.hits_on(ally, new, t, HIT_WINDOW):
                reasons.append(f"new one had just hit {self.name(ally)}")
                break
        new_target = self.target[new].at(t)
        if new_target and not self.hostile(i, new_target) and new_target != i:
            reasons.append(f"new one was after {self.name(new_target)}")
        return reasons


# ---- Class mechanics: NPCs fight with their class's actions, by its rules ----

GCD_GROUP = 58
COMBO_WINDOW = 15.0
AUTO_ATTACK_CATEGORY = 1


def _num(value, default=0):
    try:
        return int(value)
    except (TypeError, ValueError):
        return default


class Mechanics:
    """The Action sheet's class mechanics for one action, from a recording's ActionInfo or a class kit."""

    def __init__(self, action: int, info: dict):
        self.action = action
        self.name = info.get("Name", "")
        self.gcd = _num(info.get("CooldownGroup")) == GCD_GROUP or _num(info.get("AdditionalCooldownGroup")) == GCD_GROUP
        self.recast = _num(info.get("Recast100ms")) / 10
        self.cast = _num(info.get("Cast100ms")) / 10
        self.combo_from = _num(info.get("ActionCombo"))
        self.level = _num(info.get("ClassJobLevel"))
        self.class_job = _num(info.get("ClassJob"))
        self.buff = _num(info.get("StatusGainSelf"))
        self.proc = _num(info.get("ActionProcStatus"))
        self.auto_attack = _num(info.get("ActionCategory")) == AUTO_ATTACK_CATEGORY

    def label(self) -> str:
        return f"{self.action} ({self.name})" if self.name else str(self.action)


def class_profile(b: "Battle", i: int) -> dict | None:
    """How an NPC fought with its class's actions: its class, pace, combos, abilities, buffs and their follow-ups,
    re-applied debuffs, what its kit had that it didn't use, and a `rotation` for its timeline built from all that."""
    mine = sorted((x for x in b.actions if x["Source"] == i), key=lambda x: x["S"])
    if not mine:
        return None
    mech = {a: Mechanics(a, b.action_info.get(a) or {}) for a in {x["Action"] for x in mine}}
    if not any(m.name or m.recast or m.gcd for m in mech.values()):
        return None  # a recording without ActionInfo (before Recorder 2.6): nothing to go on
    used = [x for x in mine if not mech[x["Action"]].auto_attack]
    autos = sorted({x["Action"] for x in mine if mech[x["Action"]].auto_attack})
    classes = Counter(mech[x["Action"]].class_job for x in used if mech[x["Action"]].class_job > 0)
    class_job = classes.most_common(1)[0][0] if classes else 0
    kit = (b.run.get("ClassKits") or {}).get(str(class_job)) or {}
    profile = {"class_job": class_job, "class": kit.get("Abbreviation") or (f"ClassJob {class_job}" if class_job else ""),
               "auto_attacks": autos, "facts": [], "rotation": None}
    facts = profile["facts"]
    if not used:
        facts.append("Only auto-attacks.")
        return profile

    gcd_uses = [x for x in used if mech[x["Action"]].gcd]
    gcd_times = [x["S"] for x in gcd_uses]
    gaps = [round(y - x, 2) for x, y in zip(gcd_times, gcd_times[1:]) if y - x < 30]
    pace = round(_median(gaps), 1) if gaps else 2.5
    if gaps:
        facts.append(f"Its weaponskills and spells come every {pace} s (median of {len(gaps)} gaps, "
                     f"{min(gaps)}–{max(gaps)} s); a player's GCD is 2.5 s.")

    # Combos: does each step follow the step before it?
    for action, m in sorted(mech.items()):
        if not m.gcd or not m.combo_from:
            continue
        uses = [k for k, x in enumerate(gcd_uses) if x["Action"] == action]
        follows = sum(1 for k in uses if k > 0 and gcd_uses[k - 1]["Action"] == m.combo_from
                      and gcd_uses[k]["S"] - gcd_uses[k - 1]["S"] <= COMBO_WINDOW)
        before = mech.get(m.combo_from)
        facts.append(f"Combo: {m.label()} follows {before.label() if before else m.combo_from} in {follows}/{len(uses)} uses.")

    entries = []
    # Abilities: on cooldown, or every so often?
    for action, m in sorted(mech.items(), key=lambda kv: -kv[1].recast):
        if m.gcd or m.auto_attack:
            continue
        times = [x["S"] for x in used if x["Action"] == action]
        intervals = [round(y - x, 1) for x, y in zip(times, times[1:])]
        entry = {"action": action}
        if intervals:
            median = _median(intervals)
            if m.recast and median <= m.recast * 1.3:
                facts.append(f"{m.label()}: on cooldown (every {median} s; its recast is {m.recast} s).")
            else:
                facts.append(f"{m.label()}: every {median} s (its recast is {m.recast} s), so not on cooldown.")
                entry["every"] = round(median, 1)
        else:
            facts.append(f"{m.label()}: once (recast {m.recast} s).")
        # A buff: which GCD action came next?
        nexts = Counter()
        for x in (y for y in used if y["Action"] == action):
            after = next((g for g in gcd_uses if g["S"] > x["S"]), None)
            if after:
                nexts[after["Action"]] += 1
        if nexts:
            follow, count = nexts.most_common(1)[0]
            total = sum(nexts.values())
            if total >= 2 and count / total >= 0.8:
                facts.append(f"  then {mech[follow].label()} ({count}/{total} times)")
                entry["then"] = follow
        entry["comment"] = f"{len(times)} uses"
        entries.append(entry)

    # GCD actions: re-applied to the same target after a while (a damage-over-time or debuff)?
    gcd_entries = []
    for action, m in mech.items():
        if not m.gcd:
            continue
        per_target = defaultdict(list)
        for x in gcd_uses:
            if x["Action"] == action:
                for h in x["Targets"]:
                    per_target[h["Id"]].append(x["S"])
        same_target = [round(y - x, 1) for times in per_target.values() for x, y in zip(times, times[1:])]
        entry = {"action": action, "comment": f"{sum(len(t) for t in per_target.values())} uses"}
        if same_target and min(same_target) >= max(pace * 2.5, 6):
            refresh = round(min(same_target), 1)
            facts.append(f"{m.label()}: never again on the same target within {refresh} s (a debuff or damage over "
                         f"time, re-applied as it wears off).")
            entry["refresh"] = refresh
        gcd_entries.append((m, entry))
    # Priority: re-applied ones first, then combo finishers (deepest step first), then the rest.
    def depth(m):
        d, seen, cur = 0, set(), m
        while cur and cur.combo_from and cur.combo_from not in seen:
            seen.add(cur.combo_from)
            d += 1
            cur = mech.get(cur.combo_from)
        return d
    gcd_entries.sort(key=lambda me: (0 if "refresh" in me[1] else 1, -depth(me[0]), -me[0].recast))
    entries += [e for _m, e in gcd_entries]

    # What its class kit had at its level that it never used.
    level = b.actors.get(i, {}).get("Level") or 0
    unused = []
    for key, info in (kit.get("Actions") or {}).items():
        m = Mechanics(int(key), info)
        if m.level and m.level <= level and m.action not in mech:
            unused.append(m)
    if unused:
        facts.append(f"Its class kit at level {level} also has (never used here): "
                     + ", ".join(m.label() for m in sorted(unused, key=lambda m: m.level)[:15])
                     + (" …" if len(unused) > 15 else ""))
    profile["rotation"] = {"pace": pace, "actions": entries}
    return profile


# ---- The director's outline: what each ally is there to do, phase by phase ----

# A target-choice explanation, and the Battle.lua order it means.
ORDERS = {"oldest enemy": "oldest", "newest enemy": "newest", "nearest enemy": "nearest",
          "most hurt enemy": "lowest_hp", "healthiest enemy": "highest_hp"}


# ClassJob's Role column: what each class is for.
CLASS_ROLES = {1: "tank", 2: "dps", 3: "dps", 4: "support"}


def class_role(b: "Battle", i: int) -> tuple[str | None, str]:
    """The role an NPC's class has (from its class kit's ClassJob row), and a word on how sure that is."""
    profile = class_profile(b, i)
    if not profile or not profile["class_job"]:
        return None, ""
    kit = (b.run.get("ClassKits") or {}).get(str(profile["class_job"])) or {}
    role = CLASS_ROLES.get(_num((kit.get("Details") or {}).get("Role")))
    return role, f"{profile['class']}'s role" if role else ""


def acting_role(b: "Battle", i: int, start: float, end: float) -> tuple[str | None, list[str]]:
    """The role an NPC played in a stretch of the fight, from what it did: taking most of the enemies' hits is a
    tank's part (a DPS taking the toughest enemy on plays it too); healing as much as it attacks is a support's; a DPS
    that heals now and then (Papalymo's Cure) is still a DPS, the most versatile role."""
    facts = []
    heals_others = sum(1 for x in b.actions if x["Source"] == i and start <= x["S"] < end
                       for h in x["Targets"] if h["Id"] != i and not b.hostile(i, h["Id"])
                       and any(e[0] == EFFECT_HEAL for e in h["Effects"]))
    hits_taken = Counter()
    for x in b.actions:
        if start <= x["S"] < end and b.actors.get(x["Source"], {}).get("Kind") == "battle npc":
            for h in x["Targets"]:
                if b.hostile(x["Source"], h["Id"]) and any(e[0] in DAMAGE_EFFECTS for e in h["Effects"]):
                    hits_taken[h["Id"]] += 1
    side = [a for a in b.actors if b.is_fighter(a) and not b.hostile(i, a)]
    taken = hits_taken[i]
    total = sum(hits_taken[a] for a in side)
    attacks = sum(1 for x in b.actions if x["Source"] == i and start <= x["S"] < end
                  for h in x["Targets"] if b.hostile(i, h["Id"]) and any(e[0] in DAMAGE_EFFECTS for e in h["Effects"]))
    if heals_others >= 2:
        facts.append(f"it healed others {heals_others} times (and attacked {attacks} times)")
    if total >= 10:
        facts.append(f"it took {taken / total:.0%} of the hits on its side")
    if total >= 10 and taken / total >= 0.5 and len(side) > 2:
        return "tank", facts
    if heals_others >= 2 and heals_others * 2 >= attacks:
        return "support", facts
    if heals_others >= 2:
        facts.append("a DPS with some support (it heals, but mostly attacks)")
    return "dps", facts


def phases(b: "Battle") -> list[tuple[float, float, str]]:
    """The fight cut where the director's sequence changes: (start, end, name) each."""
    cuts = [(0.0, "start")]
    for e in b.events:
        if e["Kind"] == "sequence":
            name = e["Text"].split()[-1]
            if not cuts or cuts[-1][1] != f"sequence {name}":
                cuts.append((e["S"], f"sequence {name}"))
    out = []
    for k, (t, name) in enumerate(cuts):
        end = cuts[k + 1][0] if k + 1 < len(cuts) else b.length
        if end - t >= 2:
            out.append((t, end, name))
    return out


def _boss_at(b: "Battle", ally: int, t: float):
    """The enemy with the most max HP still standing: what this fight's "boss" is at that moment."""
    candidates = [c for c in b.actors if b.hostile(ally, c) and b.alive(c, t) and b.actors[c].get("Kind") == "battle npc"
                  and c in b.in_fight]
    if not candidates:
        return None, []
    # The boss is one you're fighting: it dies, you hit it or it comes after you. Scenery with huge HP (fighting in
    # the background, there to look busy) does none of those.
    engaged = [c for c in candidates if b.engaged(c)]
    if engaged:
        candidates = engaged
    boss = max(candidates, key=lambda c: b.actors[c].get("MaxHp", 0))
    return boss, [c for c in candidates if c != boss]


def ally_outline(b: "Battle", ally: int, start: float, end: float) -> dict:
    """What one ally did in one phase, as a Battle.lua role with its evidence."""
    a = b.actors[ally]
    role, facts = {}, []
    by_class, why = class_role(b, ally)
    acting, acting_facts = acting_role(b, ally, start, end)
    facts += acting_facts
    if by_class and acting and by_class != acting:
        facts.append(f"its class is a {by_class} ({why}) but it acted as a {acting} here")
    elif by_class:
        facts.append(f"a {by_class} ({why})")
    if acting or by_class:
        role["role"] = acting or by_class
    # Focus: on the boss, or on the adds, while there were both?
    on_boss = on_adds = 0
    t = start
    while t < end:
        target = b.target[ally].at(t)
        boss, adds = _boss_at(b, ally, t)
        if target and boss and adds and b.alive(ally, t):
            if target == boss:
                on_boss += 1
            elif target in adds:
                on_adds += 1
        t += 0.5
    total = on_boss + on_adds
    if total >= 6:
        share = on_boss / total
        if share >= 0.7:
            role["focus"] = ["boss", "adds"]
        elif share <= 0.3:
            role["focus"] = ["adds", "boss"]
        facts.append(f"while there was a boss and adds, it was on the boss {share:.0%} of the time")
    # Order: which explanation fits its target choices here, judged within the group it was choosing from (the adds,
    # when it picked an add: the boss arriving first doesn't make every pick "not the oldest".)
    reasons, switches, previous = Counter(), 0, b.target[ally].at(start - 0.01) or 0
    for s0, _e, target in b.target_segments(ally):
        if s0 < start or s0 >= end:
            previous = target
            continue
        if target and target != previous:
            switches += 1
            boss, adds = _boss_at(b, ally, s0)
            group = adds if target in adds else [target]
            if len(group) > 1:
                reasons.update(_order_reasons(b, ally, s0, target, group))
            else:
                reasons["the only one of its kind"] += 1
        previous = target
    fitting = [(reasons[r], o) for r, o in ORDERS.items() if reasons[r]]
    choices = switches - reasons["the only one of its kind"]
    if choices >= 2 and fitting:
        count, order = max(fitting)
        if count / choices >= 0.6:
            role["order"] = order
    if switches:
        facts.append(f"its {switches} target changes: " + ", ".join(f"{r} {c}/{switches}" for r, c in reasons.most_common(5)))
    # Range: how far from its target when it acted.
    distances = []
    for x in b.actions:
        if x["Source"] == ally and start <= x["S"] < end and x["Targets"]:
            target = x["Targets"][0]["Id"]
            if b.hostile(ally, target):
                d = b.distance(ally, target, x["S"])
                if d is not None:
                    distances.append(d)
    if len(distances) >= 3:
        median = _median(distances)
        # Melee, or a caster's distance to the nearest 5 yalms (so a phase isn't "different" over a yalm or two.)
        role["range"] = 3 if median <= 5 else 5 * round(median / 5)
        facts.append(f"it attacked from {median:.1f} yalms (median of {len(distances)})")
    # Staying near you.
    if b.you:
        near = []
        t = start
        while t < end:
            d = b.distance(ally, b.you, t)
            if d is not None and b.alive(ally, t):
                near.append(d)
            t += 1.0
        if len(near) >= 5:
            near.sort()
            p90 = near[int(len(near) * 0.9) - 1]
            facts.append(f"it stayed within {p90:.0f} yalms of you 90% of the time")
            if p90 <= 20 and role.get("range", 0) >= 6:
                role["keep_near"] = {"within": 5 * round((p90 + 2) / 5 + 0.49)}
    # Heals.
    healed = defaultdict(list)
    for x in b.actions:
        if x["Source"] == ally and start <= x["S"] < end:
            for h in x["Targets"]:
                if any(e[0] == EFFECT_HEAL for e in h["Effects"]) and h.get("MaxHp"):
                    who = "player" if h["Id"] == b.you else h["Id"]
                    healed[(x["Action"], who)].append((x["S"], 100 * h["Hp"] / h["MaxHp"]))
    heals = []
    for (action, who), uses in healed.items():
        times = [u[0] for u in uses]
        gaps = [y - x for x, y in zip(times, times[1:])]
        below = round(max(u[1] for u in uses) + 5)
        heal = {"action": action, "targets": [who], "below": min(below, 95)}
        if gaps:
            heal["every"] = round(min(gaps), 1)
        heals.append(heal)
        facts.append(f"it healed {b.name(who) if who != 'player' else 'you'} with {b.action_name(action)} "
                     f"{len(uses)} times, at {min(u[1] for u in uses):.0f}–{max(u[1] for u in uses):.0f}% HP")
    if heals:
        role["heals"] = heals
    return {"role": role, "facts": facts}


def _order_reasons(b: "Battle", ally: int, t: float, new: int, group: list[int]) -> list[str]:
    """Which orders pick `new` out of `group` at time t."""
    reasons = []
    by_spawn = sorted(group, key=lambda c: b.actors[c].get("Seen", 0))
    if by_spawn[0] == new:
        reasons.append("oldest enemy")
    if by_spawn[-1] == new:
        reasons.append("newest enemy")
    distances = {c: b.distance(ally, c, t) for c in group}
    distances = {c: d for c, d in distances.items() if d is not None}
    if distances and min(distances, key=distances.get) == new:
        reasons.append("nearest enemy")
    percents = {c: b.hp_percent(c, t) for c in group}
    percents = {c: v for c, v in percents.items() if v is not None}
    if percents and len(set(percents.values())) > 1 and min(percents, key=percents.get) == new:
        reasons.append("most hurt enemy")
    current = {c: b.hp[c].at(t) for c in group}
    current = {c: v for c, v in current.items() if v is not None}
    if current and len(set(current.values())) > 1 and max(current, key=current.get) == new:
        reasons.append("healthiest enemy")
    return reasons


def telegraph_shape(info: dict) -> str | None:
    """An action's area as Kawari reads it (gamedata.rs `ActionInfo::area_shape`): CastType 2 a circle on its target
    (or the caster with no range), 5 a circle around the caster, 3/13 a cone, 4/8/12 a line; EffectRange its size."""
    cast_type, size = _num(info.get("CastType")), _num(info.get("EffectRange"))
    width = max(_num(info.get("XAxisModifier")), 1)
    if size == 0:
        return None
    if cast_type == 2:
        return f"circle {size} y" + (" on its target" if _num(info.get("Range")) > 0 else " around itself")
    if cast_type == 5:
        return f"circle {size} y around itself"
    if cast_type in (3, 13):
        return f"cone {size} y"
    if cast_type in (4, 8, 12):
        return f"line {size} y long, {width} y wide"
    return f"cast type {cast_type}, size {size}"


def telegraphs(b: "Battle") -> str:
    """Every enemy cast with an area (a ground circle, cone or line) and its indicator: shape and size, cast time,
    omen, how often, and where it was aimed (from the cast events.)"""
    rows = defaultdict(list)
    for e in b.events:
        if e["Kind"] != "cast" or not e.get("Id"):
            continue
        parts = e["Text"].split()
        try:
            action = int(parts[parts.index("action") + 1])
        except (ValueError, IndexError):
            continue
        rows[(e["Id"], action)].append(e)
    lines = []
    for (caster, action), casts in sorted(rows.items(), key=lambda kv: kv[1][0]["S"]):
        info = b.action_info.get(action) or {}
        shape = telegraph_shape(info)
        if shape is None and not _num(info.get("Omen")):
            continue
        times = [c["S"] for c in casts]
        gaps = [round(y - x, 1) for x, y in zip(times, times[1:])]
        aimed = "at its target" if all(" on 0 " not in c["Text"] for c in casts) else "at the ground"
        lines.append(f"* {b.name(caster)}: {b.action_name(action)}, {shape or 'no area'}, omen {_num(info.get('Omen'))}, "
                     f"{_num(info.get('Cast100ms')) / 10} s cast, {len(casts)}×"
                     + (f" every {_median(gaps)} s" if gaps else "") + f", first at {times[0]:.0f} s, aimed {aimed}")
    if not lines:
        return ""
    return "\n".join(["## Telegraphs", "", "Enemy casts with an area and their indicators (Kawari's action scripts "
                       "need the shape, size and cast time; the omen is what the client draws):", ""] + lines)


def _lua(value) -> str:
    if isinstance(value, dict):
        return "{ " + ", ".join(f"{k} = {_lua(v)}" for k, v in value.items()) + " }"
    if isinstance(value, list):
        return "{ " + ", ".join(_lua(v) for v in value) + " }"
    if isinstance(value, str):
        return "Battle.PLAYER" if value == "player" else f'"{value}"'
    return str(value)


def outline(b: "Battle") -> str:
    """A draft of the battle's outline for content/lib/Battle.lua: each ally's role in each director phase, with what
    the recording shows behind it. Roles that change between phases become Battle:change calls."""
    lines: list[str] = []
    w = lines.append
    allies = [i for i, a in b.actors.items() if a.get("Kind") == "battle npc" and b.you
              and not b.hostile(i, b.you) and a.get("MaxHp") and i in b.in_fight]
    if not allies:
        return ""
    w("## The director's outline (draft)")
    w("")
    w("What each ally was there to do in each phase of the fight (the director's sequence), read from what it did: "
      "who it went for first (boss or adds), in what order, from how far, how near it stayed to you, and whom it "
      "healed. As `content/lib/Battle.lua` roles; check the evidence under each before using it.")
    w("")
    previous: dict[int, dict] = {}
    for start, end, name in phases(b):
        present = [i for i in allies if any(b.alive(i, t / 2) for t in range(int(start * 2), int(end * 2)))]
        if not present:
            continue
        w(f"### {name} ({start:.0f}–{end:.0f} s)")
        w("")
        changes = {}
        for i in present:
            result = ally_outline(b, i, start, end)
            role = result["role"]
            w(f"* {b.name(i)}: " + ("; ".join(result["facts"]) if result["facts"] else "nothing to go on"))
            if role and role != previous.get(i):
                changes[i] = role
                previous[i] = role
        w("")
        if changes:
            w("```lua")
            if name == "start":
                for i, role in changes.items():
                    w(f"BATTLE:ally({b.actors[i].get('LayoutId')}, {_lua(role)}) -- {b.name(i)}")
            else:
                w(f'{{ 0, "change", "{name}", {{')
                for i, role in changes.items():
                    w(f"    [{b.actors[i].get('LayoutId')}] = {_lua(role)}, -- {b.name(i)}")
                w("} },")
            w("```")
            w("")
    return "\n".join(lines)


def _find(data, key):
    """A field anywhere in a flattened layout object (fields of its parents are nested under ParentData)."""
    if isinstance(data, dict):
        if key in data:
            return data[key]
        for value in data.values():
            found = _find(value, key)
            if found is not None:
                return found
    return None


def _fmt(t: float) -> str:
    return f"{t:7.1f}s"


def _median(values):
    values = sorted(values)
    if not values:
        return None
    mid = len(values) // 2
    return values[mid] if len(values) % 2 else (values[mid - 1] + values[mid]) / 2


def report(run: dict, npc_filter: str | None = None) -> str:
    b = Battle(run)
    lines: list[str] = []
    w = lines.append
    w(f"# {run.get('Zone', '')} {run.get('Content', '')}".rstrip())
    w("")
    if run.get("_source") or run.get("_label"):
        w(" · ".join(x for x in (f"`{run.get('_source', '')}`" if run.get("_source") else "",
                                 f"label: {run['_label']}" if run.get("_label") else "",
                                 str(run.get("Start", ""))) if x))
        w("")
    w(f"Territory {run.get('TerritoryId')} · content finder {run.get('ContentFinderCondition')} · "
      f"{b.length:.0f} s · {len(b.actors)} actors · {len(b.samples)} samples · {len(b.actions)} actions · "
      f"{len(b.controls)} actor controls · {len(b.layers)} layout layers" + (" · TRUNCATED" if run.get("Truncated") else ""))
    w("")

    fighters = sorted((i for i in b.actors if b.is_fighter(i)), key=lambda i: (b.actors[i].get("Seen", 0), i))

    failed = {k: v for k, v in (run.get("Hooks") or {}).items() if str(v).startswith("FAILED")}
    if failed:
        w("**Missing from this recording** (the Recorder's hooks that weren't working): "
          + "; ".join(f"{k}: {v}" for k, v in failed.items()))
        w("")
    setup = run.get("Setup")
    if setup:
        w("## Your setup")
        w("")
        sync = f", synced to {setup.get('SyncedLevel')}" if setup.get("LevelSynced") else ""
        w(f"* {setup.get('JobName')} level {setup.get('Level')}{sync}, item level about {setup.get('ItemLevel')}, "
          f"HP {setup.get('Hp')}/{setup.get('MaxHp')}")
        settings = [name for key, name in (("Unrestricted", "unrestricted party"), ("MinimumItemLevel", "minimum item level"),
                                           ("LevelSync", "level sync"), ("ExplorerMode", "explorer mode")) if setup.get(key)]
        if settings:
            w("* Duty Finder settings: " + ", ".join(settings))
        party = setup.get("Party") or []
        if len(party) > 1:
            w("* Party: " + ", ".join(f"job {j} level {lv}{' (you)' if me else ''}" for j, lv, me in party))
        statuses = setup.get("Statuses") or []
        if statuses:
            w("* Statuses at the start: " + ", ".join(str(x[0]) + (f" ({x[1]})" if x[1] else "") for x in statuses)
              + " (status IDs: the Echo is 42, 239 or 3240 depending on the content)")
        w("")

    # ---- cast ----
    w("## Who's in it")
    w("")
    w("Sides are battalions: the same number fights on the same side.")
    w("")
    w("| Who | Battalion | Level | Max HP | Appears | Dies | Layer · instance | Layout settings |")
    w("|---|---|---|---|---|---|---|---|")
    for i in fighters:
        a = b.actors[i]
        info = b.layout_info(a.get("LayoutId", 0))
        where = " · ".join(x for x in (info.pop("layer", ""), info.pop("instance", "")) if x)
        settings = ", ".join(f"{k} {v}" for k, v in info.items())
        died = f"{a['Died']:.1f}" if a.get("Died") is not None else ""
        w(f"| {b.name(i)} | {a.get('Battalion')} | {a.get('Level')} | {a.get('MaxHp')} | {a.get('Seen', 0):.1f} | "
          f"{died} | {where} | {settings} |")
    w("")

    # ---- timeline ----
    w("## Timeline")
    w("")
    deaths = b.deaths()
    timeline: list[tuple[float, str]] = []
    for t, wave in b.waves():
        before = [who for d, who in deaths if 0 <= t - d <= SPAWN_WINDOW]
        context = []
        if before:
            context.append("just after " + ", ".join(b.name(x) for x in before) + " died")
        hurt = []
        for x in fighters:
            if x not in wave and b.alive(x, t - 0.01) and b.actors[x].get("Kind") == "battle npc":
                hp = b.hp_percent(x, t - 0.01)
                if hp is not None and hp < 97.5:
                    hurt.append((hp, x))
        if hurt:
            # The most hurt first: an HP threshold that brings a wave is usually the boss's, and the boss is low.
            hurt.sort()
            context.append("HP then: " + ", ".join(f"{b.name(x)} {hp:.0f}%" for hp, x in hurt[:8])
                           + (f" (+{len(hurt) - 8} more)" if len(hurt) > 8 else ""))
        timeline.append((t, f"**{len(wave)} appear**: " + ", ".join(b.name(x) for x in wave)
                         + (f" ({'; '.join(context)})" if context else "")))
    for t, who in deaths:
        timeline.append((t, f"{b.name(who)} dies"))
    interesting = {"sequence", "variables", "battle talk", "scene", "map effect", "mark", "auto mark", "objective",
                   "todos", "director", "npc chat", "system", "object", "fast move", "end"}
    for e in b.events:
        if e["Kind"] in interesting:
            text = e["Text"]
            if e["Kind"] in ("scene", "object", "fast move", "auto mark") and e.get("Id"):
                text += f" ({b.name(e['Id'])})"
            timeline.append((e["S"], f"{e['Kind']}: {text}"))
    for i, track in b.tethers.items():
        previous = []
        for t, value in zip(track.times, track.values):
            if value != previous:
                if value:
                    timeline.append((t, f"tether: {b.name(i)} → " + ", ".join(
                        f"{b.name(target)} (Channeling {kind})" for kind, target in value)))
                elif previous:
                    timeline.append((t, f"tether gone: {b.name(i)}"))
            previous = value
    for t, text in sorted(timeline, key=lambda x: x[0]):
        w(f"* {_fmt(t)} {text}")
    w("")

    enmity = run.get("Enmity") or []
    if enmity:
        w("## Your enmity")
        w("")
        w("Who was at the top of the enemy list (the enemies after you, first one first), when that changed:")
        w("")
        top = None
        for sample in enmity:
            haters = [h[0] for h in sample.get("Haters") or []]
            first = haters[0] if haters else None
            if first != top:
                w(f"* {_fmt(sample['S'])} " + (", ".join(b.name(h) for h in haters[:5]) if haters else "no one on you"))
                top = first
        w("")

    controls = run.get("Controls") or []
    if controls:
        from .repo import actor_control_names
        names = actor_control_names()
        counts = Counter((c["Category"]) for c in controls)
        w("## Actor controls")
        w("")
        w("Every ActorControl the server sent, by category (names from Kawari's ActorControlCategory):")
        w("")
        for category, count in counts.most_common():
            w(f"* {category} {names.get(category, '(unknown to Kawari)')}: {count}")
        w("")

    # ---- each NPC ----
    w("## Each NPC")
    w("")
    for i in fighters:
        a = b.actors[i]
        if a.get("Kind") != "battle npc":
            continue
        if npc_filter and npc_filter.lower() not in b.name(i).lower():
            continue
        w(f"### {b.name(i)}")
        w("")
        mine = [x for x in b.actions if x["Source"] == i]
        spawn = a.get("Position") or (b.pos[i].values[0] if b.pos[i].values else None)
        roam = 0.0
        if spawn:
            roam = max((math.dist(spawn, p) for p in b.pos[i].values), default=0.0)
        if not mine:
            w(f"* **Never acted** (moved at most {roam:.1f} yalms from where it appeared).")
        else:
            w(f"* {len(mine)} actions; moved at most {roam:.1f} yalms from where it appeared.")

        segments = b.target_segments(i)
        if any(t for _s, _e, t in segments):
            w("* Targets: " + "; ".join(f"{s:.1f}–{e:.1f}s {b.name(t)}" for s, e, t in segments if t or len(segments) > 1))
        reasons = Counter()
        switches = 0
        previous = 0
        for s, _e, t in segments:
            if t and t != previous:
                switches += 1
                for reason in b.switch_reasons(i, s, previous, t):
                    reasons[reason] += 1
            previous = t
        if switches:
            w(f"* Of its {switches} target choices, the explanations that fit (how many times):")
            for reason, count in reasons.most_common():
                w(f"  * {reason}: {count}/{switches}")

        profile = class_profile(b, i)
        if profile:
            who = f"fights as **{profile['class']}**" if profile["class"] else "uses no class's actions (its own skills)"
            autos = ", ".join(b.action_name(a) for a in profile["auto_attacks"])
            w(f"* Class: {who}" + (f"; auto-attack {autos}" if autos else ""))
            for fact in profile["facts"]:
                w(f"  * {fact}" if not fact.startswith("  ") else f"    * {fact.strip()}")
            if profile["rotation"]:
                w("* Rotation for its timeline (`\"rotation\"`), from the above:")
                w("")
                w("```json")
                w(json.dumps(profile["rotation"], indent=1))
                w("```")
                w("")

        by_action = defaultdict(list)
        for x in mine:
            by_action[x["Action"]].append(x)
        for action, uses in sorted(by_action.items(), key=lambda kv: -len(kv[1])):
            times = [u["S"] for u in uses]
            gaps = [round(b2 - a2, 1) for a2, b2 in zip(times, times[1:])]
            on = Counter()
            amounts = defaultdict(list)
            heals = []
            for u in uses:
                for h in u["Targets"]:
                    who = "itself" if h["Id"] == i else ("you" if h["Id"] == b.you else
                                                       ("an enemy" if b.hostile(i, h["Id"]) else "an ally"))
                    on[who] += 1
                    for effect in h["Effects"]:
                        kind, value = effect[0], effect[1]
                        if kind in DAMAGE_EFFECTS:
                            amounts["damage"].append(value)
                        elif kind == EFFECT_HEAL:
                            amounts["heal"].append(value)
                            heals.append((u["S"], h))
                        elif kind == EFFECT_MISS:
                            amounts["miss"].append(0)
            parts = [f"action {b.action_name(action)} ×{len(uses)}"]
            if gaps:
                parts.append(f"every {_median(gaps)} s (min {min(gaps)}, max {max(gaps)})")
            parts.append("on " + ", ".join(f"{k} {v}" for k, v in on.most_common()))
            for kind, values in amounts.items():
                if kind == "miss":
                    parts.append(f"{len(values)} misses")
                else:
                    parts.append(f"{kind} {min(values)}–{max(values)} (median {_median(values)})")
            w("* " + " · ".join(parts))
            if heals:
                percents = []
                most_hurt = 0
                for t, h in heals:
                    if h.get("MaxHp"):
                        percents.append(100 * h["Hp"] / h["MaxHp"])
                    allies = [x for x in b.actors if b.is_fighter(x) and not b.hostile(i, x) and b.alive(x, t)]
                    hps = {x: b.hp_percent(x, t - 0.3) for x in allies}
                    hps = {x: v for x, v in hps.items() if v is not None}
                    if hps and min(hps, key=hps.get) == h["Id"]:
                        most_hurt += 1
                if percents:
                    w(f"  * healed targets were at {min(percents):.0f}–{max(percents):.0f}% HP "
                      f"(median {_median(percents):.0f}%); the most hurt of its side {most_hurt}/{len(heals)} times")
                who_healed = Counter(b.name(h["Id"]) for _t, h in heals)
                w("  * healed: " + ", ".join(f"{k} {v}" for k, v in who_healed.most_common()))
        w("")

    drawn = telegraphs(b)
    if drawn:
        w(drawn)
        w("")

    drafted = outline(b)
    if drafted:
        w(drafted)
        w("")

    idle = [i for i in fighters if b.actors[i].get("Kind") == "battle npc" and not any(x["Source"] == i for x in b.actions)]
    if idle:
        w("## Never acted")
        w("")
        w("Battle NPCs that never used an action. Scenery, or something Kawari has to script that the recording "
          "can't show (check their layer and layout settings).")
        w("")
        for i in idle:
            w(f"* {b.name(i)}")
        w("")
    return "\n".join(lines)


def runs_in(recording) -> list[dict]:
    book = recording.json("battles.json") or {}
    return book.get("Runs") or []


# ---- Finding battles: Recorder 2.5 saves each one to its own battles/<start>-<territory>-<what>.json.gz ----

def _add_runs(out: list[dict], data: dict, source: str, labels: dict[str, str]):
    """A battle file (one run) or a 2.4 battles.json (Runs: [...])."""
    if "Runs" in data:
        for n, run in enumerate(data.get("Runs") or []):
            run.setdefault("_source", f"{source}#{n}")
            out.append(run)
    elif "Actors" in data:
        name = source.rsplit("/", 1)[-1]
        data["_source"] = source
        label = labels.get(name) or data.get("Label") or ""
        if label:
            data["_label"] = label
        out.append(data)


def _labels(index: dict | None) -> dict[str, str]:
    return {b.get("File", ""): b.get("Label", "") for b in (index or {}).get("Battles") or []}


def load_runs(paths) -> list[dict]:
    """Every battle in these: Recorder exports (zip), the Recorder's battles folder (or an unpacked export), single
    battle files (.json.gz) and 2.4's battles.json. Oldest first."""
    runs: list[dict] = []
    for path in paths:
        path = Path(path)
        if path.is_dir():
            folders = [path, path / "battles"]
            labels: dict[str, str] = {}
            for folder in folders:
                index = folder / "index.json"
                if index.exists():
                    labels.update(_labels(json.loads(index.read_text())))
            for folder in folders:
                if not folder.is_dir():
                    continue
                for file in sorted(folder.glob("*.json.gz")):
                    _add_runs(runs, json.loads(gzip.decompress(file.read_bytes())), str(file), labels)
                legacy = folder / "battles.json"
                if legacy.exists():
                    _add_runs(runs, json.loads(legacy.read_text()), str(legacy), labels)
        elif path.suffix == ".zip":
            with zipfile.ZipFile(path) as archive:
                names = archive.namelist()
                labels = {}
                for name in names:
                    if name.endswith("index.json") and "battles" in name:
                        labels.update(_labels(json.loads(archive.read(name))))
                for name in sorted(names):
                    if name.endswith(".json.gz"):
                        _add_runs(runs, json.loads(gzip.decompress(archive.read(name))), f"{path}:{name}", labels)
                    elif name.rsplit("/", 1)[-1] == "battles.json":
                        _add_runs(runs, json.loads(archive.read(name)), f"{path}:{name}", labels)
        elif path.name.endswith(".gz"):
            _add_runs(runs, json.loads(gzip.decompress(path.read_bytes())), str(path), {})
        else:
            _add_runs(runs, json.loads(path.read_text()), str(path), {})
    runs.sort(key=lambda run: str(run.get("Start", "")))
    return runs


# ---- The same battle recorded several times ----

def npc_key(actor: dict):
    """The same NPC across runs: its layout ID, or its BNpcBase and name when it has none."""
    return actor.get("LayoutId") or (actor.get("BaseId"), actor.get("Name"))


def npc_summary(b: Battle, i: int) -> dict:
    """What one NPC did in one run, in numbers that add up across runs."""
    reasons = Counter()
    switches = 0
    previous = 0
    for s, _e, t in b.target_segments(i):
        if t and t != previous:
            switches += 1
            reasons.update(b.switch_reasons(i, s, previous, t))
        previous = t
    actions = defaultdict(list)
    heal_percents = []
    heals = 0
    most_hurt = 0
    for x in b.actions:
        if x["Source"] != i:
            continue
        actions[x["Action"]].append(x["S"])
        for h in x["Targets"]:
            if any(e[0] == EFFECT_HEAL for e in h["Effects"]):
                heals += 1
                if h.get("MaxHp"):
                    heal_percents.append(100 * h["Hp"] / h["MaxHp"])
                allies = [a for a in b.actors if b.is_fighter(a) and not b.hostile(i, a) and b.alive(a, x["S"])]
                hps = {a: b.hp_percent(a, x["S"] - 0.3) for a in allies}
                hps = {a: v for a, v in hps.items() if v is not None}
                if hps and min(hps, key=hps.get) == h["Id"]:
                    most_hurt += 1
    return {"switches": switches, "reasons": reasons, "actions": dict(actions), "heals": heals,
            "heal_percents": heal_percents, "most_hurt": most_hurt}


def across(runs: list[dict]) -> str:
    """For battles recorded more than once: each NPC's evidence added up over every run, so a rule that holds in all
    of them stands out from one that held once."""
    groups = defaultdict(list)
    for run in runs:
        groups[(run.get("TerritoryId"), run.get("ContentFinderCondition"))].append(run)
    lines: list[str] = []
    w = lines.append
    for (_territory, _cfc), group in groups.items():
        if len(group) < 2:
            continue
        first = group[0]
        w(f"# Across {len(group)} runs: {first.get('Zone', '')} {first.get('Content', '')}".rstrip())
        w("")
        for run in group:
            label = f" ({run['_label']})" if run.get("_label") else ""
            w(f"* {run.get('Start', '')} `{run.get('_source', '')}`{label}")
        w("")
        totals: dict = {}
        for run in group:
            b = Battle(run)
            for i, actor in b.actors.items():
                if actor.get("Kind") != "battle npc" or not actor.get("MaxHp"):
                    continue
                key = npc_key(actor)
                t = totals.setdefault(key, {"name": b.name(i), "runs": 0, "idle": 0, "switches": 0,
                                            "reasons": Counter(), "uses": Counter(), "gaps": defaultdict(list),
                                            "heals": 0, "most_hurt": 0, "heal_percents": [], "deaths": []})
                summary = npc_summary(b, i)
                t["runs"] += 1
                t["idle"] += 0 if summary["actions"] else 1
                t["switches"] += summary["switches"]
                t["reasons"].update(summary["reasons"])
                for action, times in summary["actions"].items():
                    t["uses"][action] += len(times)
                    t["gaps"][action] += [round(y - x, 1) for x, y in zip(times, times[1:])]
                t["heals"] += summary["heals"]
                t["most_hurt"] += summary["most_hurt"]
                t["heal_percents"] += summary["heal_percents"]
                if actor.get("Died") is not None:
                    t["deaths"].append(actor["Died"])
        for key, t in sorted(totals.items(), key=lambda kv: str(kv[1]["name"])):
            w(f"## {t['name']}")
            w("")
            w(f"* In {t['runs']}/{len(group)} runs" + (f"; never acted in {t['idle']}" if t["idle"] else "")
              + (f"; died at {', '.join(f'{d:.0f}' for d in t['deaths'])} s" if t["deaths"] else ""))
            if t["switches"]:
                w(f"* {t['switches']} target choices in all; explanations that fit:")
                for reason, count in t["reasons"].most_common():
                    w(f"  * {reason}: {count}/{t['switches']}")
            for action, count in t["uses"].most_common():
                gaps = t["gaps"][action]
                every = f", every {_median(gaps)} s (min {min(gaps)}, max {max(gaps)})" if gaps else ""
                w(f"* action {action}: {count / t['runs']:.1f} a run{every}")
            if t["heals"]:
                percents = t["heal_percents"]
                at = f" at {min(percents):.0f}–{max(percents):.0f}% HP (median {_median(percents):.0f}%)" if percents else ""
                w(f"* {t['heals']} heals{at}; the most hurt of its side {t['most_hurt']}/{t['heals']} times")
            w("")
    return "\n".join(lines)
