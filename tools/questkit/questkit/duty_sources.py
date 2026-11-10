"""Duties against what the public projects know (`public_sources.py`): an audit of the ones the fork has, and a draft
of one it hasn't.

* `audit(repo)`: every timeline in `resources/timelines/` whose battle NPC a Boss Mod module has: the module's actions
  the fork never mentions (in the timeline, the duty's scripts or `action_areas.json`), and cones whose recorded angle
  isn't Boss Mod's.
* `draft(content_finder_condition, ...)`: a duty's bosses as timeline files in the fork's format, from Boss Mod's
  actions (who uses what, casts, shapes) and cactbot's timing where it has the territory, with what's missing said.
  Damage amounts, HP and levels aren't in either: a capture of the duty (`questkit chronofoil --battle`) or the game's
  data fills those.
"""

from __future__ import annotations

import json
import re
import statistics
from collections import Counter
from pathlib import Path

from .public_sources import Public

AUTO_ATTACKS = {870, 871, 872, 873}
# A battle NPC in this many modules is the game's invisible helper (0x233C), not a boss.
SHARED_BASE = 6


def _fork_mentions(repo: Path) -> tuple[dict[int, set[int]], str]:
    """Each timeline's actions by its BNpcBase, and all the text of the duty and quest battle scripts."""
    by_base: dict[int, set[int]] = {}
    for path in sorted((repo / "resources" / "timelines").glob("*.json")):
        base = re.search(r"_(\d+)\.json$", path.name)
        if not base:
            continue
        text = path.read_text(encoding="utf-8")
        actions = {int(n) for n in re.findall(r'"action(?:_id)?"\s*:\s*(\d+)', text)}
        actions |= {int(n) for n in re.findall(r'"autoattack_action_id"\s*:\s*(\d+)', text)}
        try:
            actions |= {int(k) for k in json.loads(text).get("recorded_amounts", {})}
        except (ValueError, AttributeError):
            pass
        actions |= {int(n) for n in re.findall(r"\b(\d{2,5}) \(", json.loads(text).get("comment", "") or "")}
        by_base.setdefault(int(base.group(1)), set()).update(actions)
    scripts = "\n".join(path.read_text(encoding="utf-8", errors="replace")
                        for path in sorted((repo / "resources" / "scripts" / "content").rglob("*.lua")))
    return by_base, scripts


def audit(repo: str | Path, public: Public | None = None) -> str:
    repo = Path(repo)
    public = public or Public()
    by_base, scripts = _fork_mentions(repo)
    areas = json.loads((repo / "resources" / "data" / "action_areas.json").read_text(encoding="utf-8"))
    recorded = {int(k): v for k, v in areas.get("cone_angles", {}).items()}
    lines = ["# The fork's duties against Boss Mod and cactbot", "",
             "Made by `questkit sources --audit`. A boss's actions Boss Mod lists that the fork's timeline, duty "
             "scripts and action data never mention, and cones recorded at another angle than Boss Mod's. Boss Mod's "
             "`visual` actions (a cast that only shows, its damage another action's) and auto-attacks are left out.",
             ""]
    checked = missing_total = 0
    for base in sorted(by_base):
        modules = [m for m in public.by_base.get(base, []) if len(public.by_base[base]) < SHARED_BASE]
        for module in modules:
            checked += 1
            ours = by_base[base]
            names = {name for name, oid in module.get("oids", {}).items() if oid == base}
            missing, angles = [], []
            for action in module.get("actions", []):
                number = action["id"]
                if number in AUTO_ATTACKS or {"visual", "auto-attack", "teleport"} & set(action.get("tags", [])):
                    continue
                # Only what this battle NPC uses (the module's other actors have their own timelines, or none.)
                if action.get("source") and not names & set(action["source"].split("/")):
                    continue
                area = action.get("area")
                if area and area["shape"] == "cone" and number in recorded and abs(recorded[number] - area["angle"]) > 1:
                    angles.append(f"{number}: recorded {recorded[number]:g}, Boss Mod {area['angle']:g}"
                                  + (" (its guess)" if area.get("unverified") else ""))
                if number in ours or re.search(rf"\b{number}\b", scripts):
                    continue
                what = []
                if "cast" in action:
                    what.append(f"{action['cast']:g} s cast" if action["cast"] else "no cast")
                if area:
                    what.append(_area_words(area))
                what += action.get("kinds", [])[:2]
                missing.append(f"{number} ({', '.join(what) or 'nothing said'})")
            if missing or angles:
                missing_total += len(missing)
                lines.append(f"* **BNpcBase {base}** (`{module['path']}`, duty {module.get('group_id', '?')}):")
                if missing:
                    lines.append(f"  * never mentioned: {'; '.join(missing)}")
                if angles:
                    lines.append(f"  * cone angles: {'; '.join(angles)}")
    lines += ["", f"{checked} timelines have a Boss Mod module; {missing_total} of their actions are never mentioned."]
    return "\n".join(lines) + "\n"


def _area_words(area: dict) -> str:
    shape = area["shape"]
    if shape == "circle":
        words = f"circle {area['radius']:g}"
    elif shape == "cone":
        words = f"cone {area['radius']:g}, {area['angle']:g} degrees"
    elif shape == "rect":
        words = f"rectangle {area['length']:g} by {area['width']:g}"
    elif shape == "donut":
        words = f"ring {area['inner']:g}-{area['radius']:g}"
    elif shape == "cross":
        words = f"cross {area['length']:g} by {area['width']:g}"
    else:
        words = shape
    return words + (" (Boss Mod's guess)" if area.get("unverified") else "")


def _median(values: list) -> float:
    return statistics.median(values) if values else 0.0


def capture_facts(ledgers, content_finder_condition: int) -> dict:
    """What Bob's captures of a duty hold, from their ledgers (`ledger.py`): its territory, the runs, and each battle
    NPC by BNpcBase: level, HP, how many were seen, and each of its actions (uses, cast time, the median time between
    uses, the median damage to players and to NPCs, the most it hit at once)."""
    from .ledger_facts import Cast

    bases: dict[int, dict] = {}
    runs: list[str] = []
    territory = None
    for capture, events in ledgers:
        cast = Cast()
        inside = False
        last: dict[tuple, float] = {}
        for event in events:
            cast.see(event)
            kind = event["e"]
            if kind == "zone":
                inside = event["content"] == content_finder_condition
                last.clear()
                if inside:
                    territory = event["territory"]
                    runs.append(capture.id[:8])
                continue
            if not inside:
                continue
            if kind == "spawn" and event["k"] == "npc" and event.get("kind") == 2:
                entry = bases.setdefault(event["base"], {"name": event["name"], "levels": Counter(), "hp": Counter(),
                                                         "seen": 0, "runs": set(), "actions": {}})
                entry["levels"][event["lv"]] += 1
                entry["hp"][event["max"]] += 1
                entry["seen"] += 1
                entry["runs"].add(len(runs))
            elif kind in ("cast", "effect"):
                source = cast.who.get(event["a"])
                if not source or source.get("k") != "npc" or source["base"] not in bases:
                    continue
                if event.get("type", 1) != 1:
                    continue
                action = bases[source["base"]]["actions"].setdefault(event["action"], {
                    "uses": 0, "casts": Counter(), "gaps": [], "players": [], "npcs": [], "most_hit": 0,
                    "spots": Counter()})
                if kind == "cast":
                    action["casts"][event["time"]] += 1
                    # Where it was cast from (a helper's spot is where the director puts the attack.)
                    where = cast.who[event["a"]].get("p") if event["p"] == [0.0, 0.0, 0.0] else event["p"]
                    if where:
                        action["spots"][(round(where[0]), round(where[2]))] += 1
                    continue
                action["uses"] += 1
                key = (event["a"], event["action"])
                if key in last and event["t"] - last[key] < 120:
                    action["gaps"].append(event["t"] - last[key])
                last[key] = event["t"]
                action["most_hit"] = max(action["most_hit"], len(event["hits"]))
                for hit in event["hits"]:
                    amounts = [fx[1] for fx in hit["fx"] if fx[0] == 3]
                    if amounts:
                        action["players" if cast.player(hit["on"]) else "npcs"].append(sum(amounts))
    out: dict = {"territory": territory, "runs": runs, "bases": {}}
    for base, entry in sorted(bases.items()):
        actions = {}
        for number, action in sorted(entry["actions"].items()):
            if not action["uses"]:
                continue
            fact = {"uses": action["uses"], "most_hit": action["most_hit"]}
            if action["casts"]:
                fact["cast"] = action["casts"].most_common(1)[0][0]
            if len(action["gaps"]) >= 2:
                fact["every"] = round(_median(action["gaps"]), 1)
            if action["spots"]:
                fact["spots"] = [list(spot) for spot, _ in action["spots"].most_common(8)]
                fact["spot_count"] = len(action["spots"])
            for side in ("players", "npcs"):
                if action[side]:
                    fact[side] = round(_median(action[side]))
            actions[number] = fact
        out["bases"][base] = {"name": entry["name"], "level": entry["levels"].most_common(1)[0][0],
                              "hp": entry["hp"].most_common(1)[0][0], "seen": entry["seen"],
                              "runs": len(entry["runs"]), "actions": actions}
    return out


def _timeline(actions: dict, auto: int | None, comment: str) -> dict:
    """A timeline in the fork's format from captured actions ({action: capture_facts' fact})."""
    amounts, rotation, gaps = {}, [], []
    for number, fact in actions.items():
        dealt = {side: fact[side] for side in ("players", "npcs") if side in fact}
        if dealt:
            amounts[str(number)] = dealt
        if number in AUTO_ATTACKS or number == auto:
            continue
        entry: dict = {"action": number}
        if "every" in fact:
            entry["every"] = fact["every"]
            gaps.append(fact["every"])
        entry["comment"] = f"{fact['uses']} uses" + (f"; {fact['cast']:g} s cast" if fact.get("cast") else "") + (
            "" if dealt else "; no damage") + (f"; hit {fact['most_hit']} at once" if fact["most_hit"] > 1 else "")
        rotation.append(entry)
    return {"autoattack_action_id": auto or 870, "recorded_amounts": amounts, "timeline_always_plays": False,
            "timepoints": [], "comment": comment,
            "rotation": {"pace": round(min(gaps), 1) if gaps else 8.0, "actions": rotation}}


def draft(content_finder_condition: int, territory: int | None = None, public: Public | None = None,
          captured: dict | None = None) -> dict:
    """The duty's bosses as timeline files (name: JSON text) and a report (`README.md`). With `captured`
    (`capture_facts`), what retail did fills them in: each action's time between uses and its damage, the boss's level
    and HP, and a timeline for every other enemy of the duty that fought."""
    public = public or Public()
    modules = public.duty(content_finder_condition)
    seen = (captured or {}).get("bases", {})
    territory = territory or (captured or {}).get("territory")
    timing = public.timing(territory) if territory else {}
    files: dict[str, str] = {}
    report = [f"# Duty {content_finder_condition}: a draft from Boss Mod and cactbot"
              + (" and Bob's captures" if seen else ""), ""]
    if captured is not None:
        report.append(f"Captures: {len(captured['runs'])} runs ({', '.join(sorted(set(captured['runs']))) or 'none'}), "
                      f"territory {captured['territory']}." if captured["runs"] else "Captures: no run of this duty.")
    if not modules:
        report.append("Boss Mod has no module for this duty.")
    if territory and not timing:
        report.append(f"cactbot has no timeline for territory {territory}: every `every` below is missing.")
    used: set[int] = set()
    for module in modules:
        oids = module.get("oids", {})
        primary = module.get("primary", "Boss")
        base = oids.get(primary)
        if base is None:
            continue
        report += ["", f"## `{module['path']}` (BNpcBase {base}, BNpcName {module.get('name_id', '?')})", ""]
        retail = seen.get(base)
        used.add(base)
        if retail:
            report.append(f"* Retail: level {retail['level']}, {retail['hp']} HP, in {retail['runs']} runs.")
        elif captured is not None and captured["runs"]:
            report.append("* Retail: this BNpcBase isn't in the captures (the duty's older or newer version?).")
        if "arena" in module:
            arena = module["arena"]
            report.append(f"* Arena: centre ({arena['x']:g}, {arena['z']:g}), {arena['bounds']} {arena['size']}.")
        others = {name: oid for name, oid in oids.items() if name != primary}
        if others:
            report.append("* Other actors (BNpcBase): " + ", ".join(f"{name} {oid}" for name, oid in others.items())
                          + ". Their actions are the director's to place (helpers) or their own timelines (adds).")
        actions, said, todo = [], [], []
        amounts: dict = {}
        auto = None
        for action in module.get("actions", []):
            number = action["id"]
            source = action.get("source", primary)
            if number in AUTO_ATTACKS or "auto-attack" in action.get("tags", []):
                auto = auto or number
                swing = retail["actions"].get(number) if retail else None
                if swing and ("players" in swing or "npcs" in swing):
                    amounts[str(number)] = {side: swing[side] for side in ("players", "npcs") if side in swing}
                continue
            words = [f"{action['cast']:g} s cast" if action.get("cast") else "no cast"]
            if "area" in action:
                words.append(_area_words(action["area"]))
            if action.get("target"):
                words.append(f"on {action['target']}")
            words += action.get("kinds", [])
            words += action.get("tags", [])
            when = timing.get(number)
            if when:
                words.append(f"cactbot: first at {when['first']:g} s" + (f", every {when['every']:g} s" if "every" in when
                                                                         else f", {when['uses']} uses"))
            # Whose action it is in the captures: the boss's, or the other actor's (a helper's, an add's).
            whose = retail if source == primary else seen.get(oids.get(source.split("/")[0]))
            did = whose["actions"].get(number) if whose else None
            if did:
                words.append(f"retail: {did['uses']} uses" + (f", every {did['every']:g} s" if "every" in did else "")
                             + (f", {did['players']} to players" if "players" in did else "")
                             + (f", {did['npcs']} to NPCs" if "npcs" in did else "")
                             + (f", cast from {did['spot_count']} spots (x, z): "
                                + " ".join(f"({x}, {z})" for x, z in did["spots"])
                                if source != primary and did.get("spots") else ""))
                if source != primary:
                    said.append(f"  * {source}: {number} ({', '.join(words)})")
                    continue
                dealt = {side: did[side] for side in ("players", "npcs") if side in did}
                if dealt:
                    amounts[str(number)] = dealt
                if "every" in did:
                    when = {"every": did["every"], "first": 0}
            elif whose:
                words.append("retail: not used in the captures")
            line = f"{number} ({', '.join(words)})"
            if source != primary:
                said.append(f"  * {source}: {line}")
                continue
            said.append(f"  * {line}")
            if {"visual", "teleport"} & set(action.get("tags", [])):
                continue
            entry: dict = {"action": number}
            if when and "every" in when:
                entry["every"] = when["every"]
            else:
                todo.append(number)
            entry["comment"] = "; ".join(words)
            actions.append(entry)
        report.append("* Actions:")
        report += said
        if todo:
            report.append(f"* No timing for {', '.join(map(str, todo))}: a capture gives it (`questkit chronofoil "
                          f"--battle`), else the rotation's pace spreads them.")
        if retail:
            known = {a["id"] for a in module.get("actions", [])}
            extra = {n: f for n, f in retail["actions"].items() if n not in known}
            if extra:
                report.append("* In the captures, not in Boss Mod's module: " + "; ".join(
                    f"{n} ({f['uses']} uses" + (f", {f['players']} to players" if "players" in f else "") + ")"
                    for n, f in extra.items()) + ".")
                for number, fact in extra.items():
                    if number in AUTO_ATTACKS:
                        auto = auto or number
                    else:
                        actions.append({"action": number, **({"every": fact["every"]} if "every" in fact else {}),
                                        "comment": f"retail: {fact['uses']} uses; not in Boss Mod's module"})
                    dealt = {side: fact[side] for side in ("players", "npcs") if side in fact}
                    if dealt:
                        amounts[str(number)] = dealt
        else:
            report.append("* Not in either source: its level, HP and what each action deals (the game's data and a "
                          "capture).")
        timeline = {
            "autoattack_action_id": auto or 870,
            "from_pull": True,
            "recorded_amounts": amounts,
            "rotation": {"pace": 8.0, "actions": actions,
                         "comment": "A draft: the pace is a guess until a capture gives the gaps."},
            "timepoints": [],
            "timeline_always_plays": False,
            "comment": f"A draft from public sources, not yet checked against retail: ContentFinderCondition "
                       f"{content_finder_condition}, BNpcBase {base}, BNpcName {module.get('name_id', '?')}. Actions, "
                       f"casts and shapes from Boss Mod's {module['path']} (awgil/ffxiv_bossmod, BSD-3-Clause)"
                       + (f"; timing from cactbot's timeline for territory {territory} (OverlayPlugin/cactbot, "
                          f"Apache-2.0)" if timing else "")
                       + (f". Retail (Bob's Chronofoil captures {', '.join(sorted(set(captured['runs'])))}): level "
                          f"{retail['level']}, {retail['hp']} HP; `players` and `npcs` are the median damage to each, "
                          f"`every` the median time between uses." if retail else ". No damage amounts."),
        }
        stem = re.sub(r"\W", "", module["path"].rsplit("/", 1)[-1].removesuffix(".cs"))
        files[f"{stem}_{base}.json"] = json.dumps(timeline, indent=4) + "\n"
    # The duty's other enemies that fought, from the captures alone.
    # (The invisible helpers, in many modules, are the bosses' own attacks: listed with them above.)
    helpers = {module.get("oids", {}).get("Helper") for module in modules}
    others = {base: entry for base, entry in seen.items() if base not in used and entry["actions"]
              and base not in helpers and len(public.by_base.get(base, [])) < SHARED_BASE}
    if others:
        report += ["", "## The duty's other enemies (from the captures alone)", "",
                   "| BNpcBase | BNpcName | Level | HP | Seen | Actions (uses; every; to players) |", "|---|---|---|---|---|---|"]
        for base, entry in others.items():
            auto = next((n for n in entry["actions"] if n in AUTO_ATTACKS), None)
            report.append(f"| {base} | {entry['name']} | {entry['level']} | {entry['hp']} | {entry['seen']} | " + "; ".join(
                f"{n} ({f['uses']}" + (f"; {f['every']:g} s" if "every" in f else "") + (
                    f"; {f['players']}" if "players" in f else "") + ")" for n, f in entry["actions"].items()) + " |")
            files[f"Duty{content_finder_condition}Enemy_{base}.json"] = json.dumps(_timeline(
                entry["actions"], auto,
                f"A draft from Bob's Chronofoil captures ({', '.join(sorted(set(captured['runs'])))}), not yet checked "
                f"in game: ContentFinderCondition {content_finder_condition}, BNpcBase {base}, BNpcName "
                f"{entry['name']}, level {entry['level']}, {entry['hp']} HP, {entry['seen']} seen in {entry['runs']} "
                f"runs. `players` and `npcs` are the median damage to each, `every` the median time between uses."),
                indent=4) + "\n"
    files["README.md"] = "\n".join(report) + "\n"
    return files


def server_data(public: Public | None = None) -> dict:
    """`resources/data/public_action_areas.json`: what the server takes from Boss Mod where the game's files and the
    captures say nothing (gamedata.rs: `public_cone_angle`, `public_donut_hole`.)"""
    public = public or Public()
    # Angles outside 15-340 degrees are a shape this reader took wrongly (or one the server has no cone for.)
    cones = {action: angle for action, angle in public.cone_angles().items() if 15 <= angle <= 340}
    holes = {}
    for action in public.by_action:
        area = public.area(action)
        if area and area["shape"] == "donut" and 0 < area["inner"] < area["radius"] and not area.get("unverified"):
            holes[action] = round(area["inner"] / area["radius"], 4)
    return {
        "comment": "Enemies' area attacks as Boss Mod has them (github.com/awgil/ffxiv_bossmod, BSD-3-Clause; made by "
                   "`questkit sources --server-data` from tools/questkit/questkit/data/public/bossmod.json): "
                   "`cone_angles`, a cone's whole width in degrees by Action row (those Boss Mod marks as unverified "
                   "left out), and `donut_holes`, a ring's hole as a share of its outer radius. gamedata.rs uses them "
                   "after what was recorded on retail (action_areas.json) and the omen's own path, before its "
                   "defaults (90 degrees; half).",
        "cone_angles": {str(k): cones[k] for k in sorted(cones)},
        "donut_holes": {str(k): holes[k] for k in sorted(holes)},
    }
