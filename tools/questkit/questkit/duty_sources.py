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


def draft(content_finder_condition: int, territory: int | None = None, public: Public | None = None) -> dict:
    """The duty's bosses as timeline files (name: JSON text) and a report (`README.md`)."""
    public = public or Public()
    modules = public.duty(content_finder_condition)
    timing = public.timing(territory) if territory else {}
    files: dict[str, str] = {}
    report = [f"# Duty {content_finder_condition}: a draft from Boss Mod and cactbot", ""]
    if not modules:
        report.append("Boss Mod has no module for this duty.")
    if territory and not timing:
        report.append(f"cactbot has no timeline for territory {territory}: every `every` below is missing.")
    for module in modules:
        oids = module.get("oids", {})
        primary = module.get("primary", "Boss")
        base = oids.get(primary)
        if base is None:
            continue
        report += ["", f"## `{module['path']}` (BNpcBase {base}, BNpcName {module.get('name_id', '?')})", ""]
        if "arena" in module:
            arena = module["arena"]
            report.append(f"* Arena: centre ({arena['x']:g}, {arena['z']:g}), {arena['bounds']} {arena['size']}.")
        others = {name: oid for name, oid in oids.items() if name != primary}
        if others:
            report.append("* Other actors (BNpcBase): " + ", ".join(f"{name} {oid}" for name, oid in others.items())
                          + ". Their actions are the director's to place (helpers) or their own timelines (adds).")
        actions, said, todo = [], [], []
        auto = None
        for action in module.get("actions", []):
            number = action["id"]
            source = action.get("source", primary)
            if number in AUTO_ATTACKS or "auto-attack" in action.get("tags", []):
                auto = auto or number
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
        report.append("* Not in either source: its level, HP and what each action deals (the game's data and a capture).")
        timeline = {
            "autoattack_action_id": auto or 870,
            "from_pull": True,
            "recorded_amounts": {},
            "rotation": {"pace": 8.0, "actions": actions,
                         "comment": "A draft: the pace is a guess until a capture gives the gaps."},
            "timepoints": [],
            "timeline_always_plays": False,
            "comment": f"A draft from public sources, not yet checked against retail: ContentFinderCondition "
                       f"{content_finder_condition}, BNpcBase {base}, BNpcName {module.get('name_id', '?')}. Actions, "
                       f"casts and shapes from Boss Mod's {module['path']} (awgil/ffxiv_bossmod, BSD-3-Clause)"
                       + (f"; timing from cactbot's timeline for territory {territory} (OverlayPlugin/cactbot, "
                          f"Apache-2.0)" if timing else "") + ". No damage amounts.",
        }
        stem = re.sub(r"\W", "", module["path"].rsplit("/", 1)[-1].removesuffix(".cs"))
        files[f"{stem}_{base}.json"] = json.dumps(timeline, indent=4) + "\n"
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
