"""What public projects know about duties, as numbers the tools can read without a network.

Two sources, each read from a checkout and written to `questkit/data/public/` (`questkit sources --build`):

* **Boss Mod** (github.com/awgil/ffxiv_bossmod, BSD-3-Clause): one module a boss, with its battle NPCs (`OID`, the
  BNpcBase), its actions (`AID`, each with who uses it on whom, its cast time and its shape as a comment), the components
  that say what each action is (an area with its shape, a raidwide, a tankbuster, a stack, a knockback, a gaze...) and
  the duty it's in (`ModuleInfo`: a ContentFinderCondition, a quest, a FATE...).
* **cactbot** (github.com/OverlayPlugin/cactbot, Apache-2.0): a timeline a duty, each line a time and the action that
  comes then (`ui/raidboss/data/**/*.txt`), with the territory it's for (`zoneId` in the `.ts` beside it).

Only IDs, times and sizes are kept, with the projects' own identifiers for actions (no game text: cactbot's ability
names, which are the game's, are left out). `docs/src/fork/sources.md` credits both.
"""

from __future__ import annotations

import json
import re
import statistics
from pathlib import Path

DATA = Path(__file__).parent / "data" / "public"

# --- Boss Mod ---

_ENUM = re.compile(r"public enum (OID|AID|SID|IconID|TetherID)\s*:\s*uint\s*\{(.*?)\n\}", re.S)
_ENTRY = re.compile(r"^\s*(\w+)\s*=\s*(0x[0-9A-Fa-f]+|\d+)\s*,?\s*(?://\s*(.*))?$", re.M)
_INFO = re.compile(r"\[ModuleInfo\((.*?)\)\]", re.S)
_CLASS = re.compile(
    r"class\s+(\w+)\s*\(BossModule module\)\s*:\s*Components\.(\w+)\(module\s*,\s*(.*?)\)\s*(?:;|\{)", re.S)
_AID_REF = re.compile(r"AID\.(\w+)")
_SHAPE = re.compile(r"new AOEShape(\w+)\(([^()]*(?:\([^()]*\)[^()]*)*)\)")
_NUMBER = re.compile(r"-?\d+(?:\.\d+)?")
_CAST = re.compile(r"(\d+(?:\.\d+)?)(?:\+(\d+(?:\.\d+)?))?s cast")
_ARENA = re.compile(r"BossModule\(init,\s*new\(([-\d.f]+),\s*([-\d.f]+)\),\s*new ArenaBounds(\w+)\(([^)]*)\)")


def _number(text: str) -> float:
    return float(text.rstrip("f"))


def _shape(kind: str, arguments: str) -> dict | None:
    """Boss Mod's AOEShape as the server's: a cone's angle is its whole width (Boss Mod gives half), a rectangle's
    width its whole width (Boss Mod gives half), lengths in yalms."""
    numbers = [float(n) for n in _NUMBER.findall(arguments.replace("f,", ",").replace("f)", ")"))]
    if kind == "Circle" and numbers:
        return {"shape": "circle", "radius": numbers[0]}
    if kind == "Cone" and len(numbers) >= 2:
        return {"shape": "cone", "radius": numbers[0], "angle": numbers[1] * 2}
    if kind == "Rect" and len(numbers) >= 2:
        shape = {"shape": "rect", "length": numbers[0], "width": numbers[1] * 2}
        if len(numbers) >= 3 and numbers[2]:
            shape["behind"] = numbers[2]
        return shape
    if kind == "Donut" and len(numbers) >= 2:
        return {"shape": "donut", "inner": numbers[0], "radius": numbers[1]}
    if kind == "DonutSector" and len(numbers) >= 3:
        return {"shape": "donut_sector", "inner": numbers[0], "radius": numbers[1], "angle": numbers[2] * 2}
    if kind == "Cross" and len(numbers) >= 2:
        return {"shape": "cross", "length": numbers[0], "width": numbers[1] * 2}
    return None


def _action_comment(comment: str) -> dict:
    """What an AID's comment says: `Boss->self, 4.0s cast, range 40 ?-degree cone aoe`."""
    out: dict = {}
    if not comment:
        return out
    who = re.match(r"\s*([\w/ ]+?)->([\w/ ]+?)(?:,|$)", comment)
    if who:
        out["source"], out["target"] = who.group(1).strip(), who.group(2).strip()
    cast = _CAST.search(comment)
    if cast:
        out["cast"] = float(cast.group(1))
        if cast.group(2):
            out["cast_after"] = float(cast.group(2))
    elif "no cast" in comment:
        out["cast"] = 0.0
    said = re.search(r"range (\d+(?:\.\d+)?)(?:\+R)?(?:-(\d+(?:\.\d+)?))? ?(?:width (\d+(?:\.\d+)?) )?"
                     r"(?:(\d+|\?)-degree )?(circle|cone|rect|donut|cross)?", comment)
    if said:
        described = {"range": float(said.group(1))}
        if said.group(2):
            described["outer"] = float(said.group(2))
        if said.group(3):
            described["width"] = float(said.group(3))
        if said.group(4) and said.group(4) != "?":
            described["angle"] = float(said.group(4))
        if said.group(5):
            described["shape"] = said.group(5)
        out["described"] = described
    for word in ("raidwide", "tankbuster", "knockback", "stack", "spread", "gaze", "visual", "interruptible",
                 "single-target", "enrage"):
        if word in comment.lower():
            out.setdefault("tags", []).append(word)
    return out


_NAMESPACE = re.compile(r"^\s*namespace\s+([\w.]+)\s*;", re.M)


def read_bossmod_module(text: str, where: str) -> dict | None:
    """One module: the files of one namespace together (a big fight's enums, components and states are in several.)"""
    info = _INFO.search(text)
    enums: dict[str, str] = {}
    for kind, body in _ENUM.findall(text):
        enums[kind] = enums.get(kind, "") + "\n" + body
    if "AID" not in enums and not info:
        return None
    module: dict = {"path": where}
    if info:
        for key, value in re.findall(r"(\w+)\s*=\s*([^,)]+)", info.group(1)):
            value = value.strip()
            if key == "GroupType":
                module["group_type"] = value.rsplit(".", 1)[-1]
            elif key in ("GroupID", "NameID", "SortOrder", "PlanLevel") and re.fullmatch(r"\d+", value):
                module[{"GroupID": "group_id", "NameID": "name_id", "SortOrder": "order",
                        "PlanLevel": "level"}[key]] = int(value)
            elif key == "PrimaryActorOID":
                module["primary"] = value.rsplit(".", 1)[-1]
    arena = _ARENA.search(text)
    if arena:
        module["arena"] = {"x": _number(arena.group(1)), "z": _number(arena.group(2)), "bounds": arena.group(3).lower(),
                           "size": [float(n) for n in _NUMBER.findall(arena.group(4).replace("f", ""))]}
    oids = {}
    for name, value, comment in _ENTRY.findall(enums.get("OID", "")):
        oids[name] = int(value, 0)
    if oids:
        module["oids"] = oids
    actions: dict[str, dict] = {}
    for name, value, comment in _ENTRY.findall(enums.get("AID", "")):
        actions[name] = {"id": int(value, 0), "name": name, **_action_comment(comment)}
    for name, value, _ in _ENTRY.findall(enums.get("SID", "")):
        module.setdefault("statuses", {})[name] = int(value, 0)
    # What each action is, from its component.
    for _, kind, arguments in _CLASS.findall(text):
        shape = _SHAPE.search(arguments)
        for reference in _AID_REF.findall(arguments):
            action = actions.get(reference)
            if action is None:
                continue
            action.setdefault("kinds", [])
            if kind not in action["kinds"]:
                action["kinds"].append(kind)
            if shape and "area" not in action:
                made = _shape(shape.group(1), shape.group(2))
                if made:
                    action["area"] = made
    # A shape marked "TODO: verify" on its line is Boss Mod's guess.
    for line in text.splitlines():
        if "TODO" in line and "AOEShape" in line:
            for reference in _AID_REF.findall(line):
                if reference in actions and "area" in actions[reference]:
                    actions[reference]["area"]["unverified"] = True
    if actions:
        module["actions"] = sorted(actions.values(), key=lambda a: a["id"])
    return _numbers_only(module) if (actions or oids) else None


def _numbers_only(module: dict) -> dict:
    """Boss Mod names its actors, actions and statuses after the game's text; the extract keeps numbers (the game's
    data names them at runtime): an actor is `Boss`, `Helper` or `actor<BNpcBase>`, an action its row, with
    `auto-attack` and `teleport` among its tags where its name said so."""
    oids = module.get("oids", {})

    def actor(name: str) -> str:
        if name in ("Boss", "Helper") or name not in oids:
            return name if name in ("Boss", "Helper", "player", "players", "self", "location", "none") else "other"
        return f"actor{oids[name]}"

    for action in module.get("actions", []):
        name = action.pop("name")
        for word, tag in (("AutoAttack", "auto-attack"), ("Teleport", "teleport")):
            if name.startswith(word) and tag not in action.get("tags", []):
                action.setdefault("tags", []).append(tag)
        if "source" in action:
            action["source"] = "/".join(actor(part.strip()) for part in action["source"].split("/"))
        if "target" in action:
            action["target"] = "/".join(actor(part.strip()) for part in action["target"].split("/"))
    if "primary" in module:
        module["primary"] = actor(module["primary"])
    if oids:
        module["oids"] = {actor(name): base for name, base in oids.items()}
    if "statuses" in module:
        module["statuses"] = sorted(set(module["statuses"].values()))
    return module


def read_bossmod(checkout: str | Path) -> dict:
    """Every module under `BossMod.Modules/` of a checkout of Boss Mod."""
    root = Path(checkout)
    by_namespace: dict[str, list[tuple[str, str]]] = {}
    for path in sorted((root / "BossMod.Modules").rglob("*.cs")):
        text = path.read_text(encoding="utf-8-sig", errors="replace")
        namespace = _NAMESPACE.search(text)
        where = str(path.relative_to(root)).removeprefix("BossMod.Modules/")
        by_namespace.setdefault(namespace.group(1) if namespace else where, []).append((where, text))
    modules = []
    for files in by_namespace.values():
        # The module's place is its folder when it has several files, else its file.
        where = files[0][0] if len(files) == 1 else files[0][0].rsplit("/", 1)[0]
        module = read_bossmod_module("\n".join(text for _, text in files), where)
        if module:
            modules.append(module)
    modules.sort(key=lambda m: m["path"])
    return {"about": "Boss Mod's modules (github.com/awgil/ffxiv_bossmod, BSD-3-Clause, see docs/src/fork/sources.md): "
                     "battle NPCs (BNpcBase), actions with their casts and shapes (a cone's angle and a rectangle's "
                     "width are whole widths here), what each is, and the duty. Numbers only: Boss Mod's names for "
                     "actors, actions and statuses (the game's text) are left out. Made by `questkit sources --build`.",
            "modules": modules}


# --- Boss Mod's job actions ---

_JOB_ACTION = re.compile(r"^\s*\w+ = (\d+), // L(\d+), ([^\n]*)", re.M)


def read_job_actions(checkout: str | Path) -> dict:
    """Each job's actions from `BossMod/ActionQueue/<role>/<JOB>.cs`: the level it's learnt at, whether it's on the
    global cooldown, its recast and cast in seconds, and whether it's used on oneself or a friend. What the rotation
    simulator (`rotation_sim.py`) needs to pace a bot as the server does."""
    jobs = {}
    for path in sorted((Path(checkout) / "BossMod" / "ActionQueue").glob("*/*.cs")):
        actions = []
        for number, level, said in _JOB_ACTION.findall(path.read_text(encoding="utf-8-sig", errors="replace")):
            recast = re.search(r"([\d.]+)s CD", said)
            cast = re.search(r"([\d.]+)s cast", said)
            actions.append({"id": int(number), "lv": int(level),
                            # On the global cooldown as the server counts it (`get_action_mechanics`: the Action
                            # row's own CooldownGroup). One with its own recast that also sets the global cooldown
                            # off ("group 9/57": the Summoner's demi-summons, Phlegma) isn't, there or here.
                            "gcd": "GCD" in said or not recast,
                            "recast": float(recast.group(1)) if recast else 2.5,
                            "cast": float(cast.group(1)) if cast else 0.0,
                            "friendly": "Hostile" not in said})
        if actions:
            jobs[path.stem] = sorted(actions, key=lambda a: (a["lv"], a["id"]))
    return {"about": "Boss Mod's job action lists (github.com/awgil/ffxiv_bossmod, BSD-3-Clause, BossMod/ActionQueue): "
                     "by job, each action's level, whether it's on the global cooldown, its recast and cast time, "
                     "and whether it's for oneself or friends. Numbers only. Made by `questkit sources --build`.",
            "jobs": jobs}


# --- cactbot ---

_ZONE = re.compile(r"zoneId:\s*ZoneId\.(\w+)")
_ZONE_ID = re.compile(r"^\s*'(\w+)':\s*(\d+),", re.M)
_LINE = re.compile(r"^(\d+(?:\.\d+)?)\s+(?:\"[^\"]*\"|label \"[^\"]*\")\s*(.*)$")
_ABILITY = re.compile(r"(Ability|StartsUsing)\s*\{[^}]*?id:\s*(\[[^\]]*\]|\"[^\"]*\")")
_JUMP = re.compile(r"\b(?:force)?jump\s+(\d+(?:\.\d+)?|\"[^\"]*\")")


def _ids(text: str) -> list[int]:
    """The action IDs of a cactbot `id:` (hex, maybe several, maybe a pattern like `5B[56]`: those are left out)."""
    ids = []
    for token in re.findall(r"[0-9A-Fa-f|()\[\]?.]+", text.strip('[]"').replace('"', " ").replace(",", " ")):
        for part in token.strip("()").split("|"):
            if re.fullmatch(r"[0-9A-Fa-f]{2,5}", part):
                ids.append(int(part, 16))
    return ids


def read_cactbot_timeline(path: Path) -> list[dict]:
    entries = []
    for raw in path.read_text(encoding="utf-8", errors="replace").splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or line.startswith("hideall"):
            continue
        matched = _LINE.match(line)
        if not matched:
            continue
        ability = _ABILITY.search(matched.group(2))
        if not ability:
            continue
        ids = _ids(ability.group(2))
        if not ids:
            continue
        entry = {"t": float(matched.group(1)), "ids": ids}
        if ability.group(1) == "StartsUsing":
            entry["cast_start"] = True
        jump = _JUMP.search(matched.group(2))
        if jump and jump.group(1)[0].isdigit():
            entry["jump"] = float(jump.group(1))
        entries.append(entry)
    return entries


def read_cactbot(checkout: str | Path) -> dict:
    root = Path(checkout)
    zone_ids = dict((name, int(number)) for name, number in
                    _ZONE_ID.findall((root / "resources" / "zone_id.ts").read_text(encoding="utf-8")))
    timelines = []
    for path in sorted((root / "ui" / "raidboss" / "data").rglob("*.txt")):
        script = path.with_suffix(".ts")
        if not script.exists():
            continue
        zone = _ZONE.search(script.read_text(encoding="utf-8", errors="replace"))
        entries = read_cactbot_timeline(path)
        if not zone or not entries:
            continue
        timelines.append({"path": str(path.relative_to(root / "ui" / "raidboss" / "data")),
                          "territory": zone_ids.get(zone.group(1)), "entries": entries})
    return {"about": "cactbot's raidboss timelines (github.com/OverlayPlugin/cactbot, Apache-2.0, see "
                     "docs/src/fork/sources.md): the time of each action in a duty, by territory. Ability names (the "
                     "game's text) are left out. Made by `questkit sources --build`.",
            "timelines": timelines}


# --- reading them back ---

def build(bossmod: str | Path | None, cactbot: str | Path | None, out: Path = DATA) -> dict:
    out.mkdir(parents=True, exist_ok=True)
    made = {}
    if bossmod:
        data = read_bossmod(bossmod)
        _write(out / "bossmod.json", data, "modules")
        made["bossmod"] = len(data["modules"])
        if (Path(bossmod) / "BossMod" / "ActionQueue").is_dir():
            jobs = read_job_actions(bossmod)
            lines = ["{", f' "about": {json.dumps(jobs["about"])},', ' "jobs": {']
            lines.append(",\n".join(f'  {json.dumps(job)}: ' + json.dumps(actions, separators=(",", ":"), sort_keys=True)
                                    for job, actions in jobs["jobs"].items()))
            lines += [" }", "}"]
            (out / "job_actions.json").write_text("\n".join(lines) + "\n", encoding="utf-8")
            made["jobs"] = len(jobs["jobs"])
    if cactbot:
        data = read_cactbot(cactbot)
        _write(out / "cactbot.json", data, "timelines")
        made["cactbot"] = len(data["timelines"])
    return made


def _write(path: Path, data: dict, key: str) -> None:
    """One module or timeline a line, so a new checkout's changes show in a diff."""
    lines = ["{", f' "about": {json.dumps(data["about"])},', f' "{key}": [']
    rows = [json.dumps(row, separators=(",", ":"), sort_keys=True) for row in data[key]]
    lines.append(",\n".join("  " + row for row in rows))
    lines += [" ]", "}"]
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


class Public:
    """The two extracts, with what the tools ask of them."""

    def __init__(self, folder: Path = DATA):
        self.modules = self._load(folder / "bossmod.json", "modules")
        self.timelines = self._load(folder / "cactbot.json", "timelines")
        self.by_action: dict[int, list[tuple[dict, dict]]] = {}
        self.by_base: dict[int, list[dict]] = {}
        for module in self.modules:
            for action in module.get("actions", []):
                self.by_action.setdefault(action["id"], []).append((module, action))
            for base in set(module.get("oids", {}).values()):
                self.by_base.setdefault(base, []).append(module)

    @staticmethod
    def _load(path: Path, key: str) -> list:
        if not path.exists():
            return []
        return json.loads(path.read_text(encoding="utf-8"))[key]

    def duty(self, content_finder_condition: int) -> list[dict]:
        """A duty's bosses, in Boss Mod's order (its files are numbered)."""
        return sorted((m for m in self.modules
                       if m.get("group_type") == "CFC" and m.get("group_id") == content_finder_condition),
                      key=lambda m: m["path"])

    def area(self, action: int) -> dict | None:
        """An action's shape, if every module that has it says the same."""
        areas = [a["area"] for _, a in self.by_action.get(action, []) if "area" in a]
        if areas and all(area == areas[0] for area in areas):
            return areas[0]
        return None

    def cone_angles(self, verified_only: bool = True) -> dict[int, float]:
        """Every cone's whole angle by action (those Boss Mod marks as unverified left out by default.)"""
        angles = {}
        for action in self.by_action:
            area = self.area(action)
            if area and area["shape"] == "cone" and not (verified_only and area.get("unverified")):
                angles[action] = area["angle"]
        return angles

    def timing(self, territory: int) -> dict[int, dict]:
        """Each action of a territory's cactbot timelines: when it first comes and the median time between its uses
        (within one timeline's phases, loops unrolled once.)"""
        out: dict[int, dict] = {}
        for timeline in self.timelines:
            if timeline.get("territory") != territory:
                continue
            times: dict[int, list[float]] = {}
            for entry in timeline["entries"]:
                for action in entry["ids"]:
                    times.setdefault(action, []).append(entry["t"])
            for action, at in times.items():
                at.sort()
                gaps = [b - a for a, b in zip(at, at[1:]) if 0 < b - a < 120]
                found = {"first": at[0], "uses": len(at)}
                if gaps:
                    found["every"] = round(statistics.median(gaps), 1)
                out.setdefault(action, found)
        return out
