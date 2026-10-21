"""questkit battle-sim: runs a quest battle's director script against what happened in the same battle on retail.

A **battle fixture** (tools/questkit/battles/<quest battle>.json) is made from a retail run (a Chronofoil capture's
battle, `questkit chronofoil --battle N.V`, or a Kawari Recorder battle file) and holds only numbers:

    {
      "quest_battle": 37, "source": "...",
      "npcs": {"4077147": {"spawn": null, "death": 140.9, "hp": [[0, 100], [42.0, 74.3], ...]}, ...},
      "scenes": {"6": 0.3},                       # how long the client took over each of the director's scenes
      "events": [[0.0, "sequence", 2], [5.1, "talk", 114], [48.0, "spawn", 4077151], ..., [140.9, "won"]],
      "tolerance": 2.0
    }

Times are seconds from the fight starting (the director's scene 60000, which is when Kawari calls `onCommence`).
`spawn` is when the NPC appeared (null: it was there when the fight started), `death` when its health reached 0 (when
Kawari calls `onActorDeath`; retail's Defeated comes about 2 s later), `hp` its health in percent over time. The events are retail's director sequence changes, battle talk (by text ID), the NPCs that
arrived during the fight, and "won" ("Duty Complete", the director's scene 60001).

The simulator runs the real script with content/lib/Battle.lua in Lua 5.4: `onSetup`, `onCommence` at 0, then
`onUpdate` every half second, as the server does. An NPC spawned outside `onUpdate` (in `onCommence`, `onActorDeath`
or `onSceneFinished`) isn't in the next update's health yet: the server spawns it only after that update (its
director tasks run after `onUpdate`), so a script reading `health[npc]` for it then sees nil. Each NPC the script spawns follows retail's health curve and falls
when it fell on retail, counted from when the script spawned it (so a wave the script brings in late also falls late),
and the script hears `onActorDeath` for it. A director scene the script plays finishes after as long as the client
took on retail (`onSceneFinished`). Then the script's sequences, battle talk, spawns and win are matched to retail's,
in order, each within `tolerance` seconds. It also checks that everything the script spawns is in a drop-in layer.

What it doesn't simulate: positions (everyone stands at the origin, so "nearest" choices are arbitrary), enmity and
who hits whom (the health curves are retail's whatever the script's targeting does), and losing. The player's health
follows retail's curve when the fixture has one (`player`, for lines said as the player's health drops), full otherwise.
"""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from pathlib import Path

from . import repo

BATTLES = repo.ROOT / "tools" / "questkit" / "battles"
QUEST_BATTLE_SCRIPTS = repo.SCRIPTS / "content" / "questbattle"
DROPINS = repo.ROOT / "resources" / "dropins"
TICK = 0.5
SCENE_COMMENCE = 60000
SCENE_COMPLETE = 60001
DONE_SEQUENCES = (0xFE, 0xFF)  # set by the server once a battle is over, not by the script
DEFAULT_TOLERANCE = 2.0
AFTER_LAST_EVENT = 30.0  # how long to keep simulating after retail's last event


# --- making a fixture from a retail run -----------------------------------------------------------------------------

def _director_events(run: dict, kind: str) -> list[tuple[float, str]]:
    return [(e["S"], e["Text"]) for e in run.get("Events", []) if e.get("Kind") == kind]


def _number_after(text: str, word: str) -> int | None:
    parts = text.split()
    try:
        return int(parts[parts.index(word) + 1], 0)
    except (ValueError, IndexError):
        return None


def fixture_from_run(run: dict, quest_battle: int, source: str = "") -> dict:
    """A battle fixture from a battle run (`questkit chronofoil --battle`, or a Recorder battle file)."""
    # (The Recorder writes scene numbers signed: 60000 is -5536.)
    scenes = [(t, (_number_after(text, "scene") or 0) & 0xFFFF) for t, text in _director_events(run, "scene")]
    ends = [(t, (_number_after(text, "scene") or 0) & 0xFFFF) for t, text in _director_events(run, "scene end")]
    commence = next((t for t, n in scenes if n == SCENE_COMMENCE), None)
    if commence is None:
        raise ValueError("the run has no scene 60000 (the fight starting); a Chronofoil battle has it")

    def rel(t):
        return round(t - commence, 2)

    actors = {k: a for k, a in run.get("Actors", {}).items() if a.get("Kind") == "battle npc" and a.get("LayoutId")}
    curves: dict[str, list] = {k: [] for k in actors}
    for sample in run.get("Samples", []):
        key = str(sample.get("Id"))
        if key in curves and "Hp" in sample and actors[key].get("MaxHp"):
            pct = round(100 * sample["Hp"] / actors[key]["MaxHp"], 1)
            if not curves[key] or curves[key][-1][1] != pct:
                curves[key].append([rel(sample["S"]), pct])
    for action in run.get("Actions", []):  # each hit says its target's HP too
        for target in action.get("Targets", []):
            key = str(target.get("Id"))
            if key in curves and "Hp" in target and target.get("MaxHp"):
                curves[key].append([rel(action["S"]), round(100 * target["Hp"] / target["MaxHp"], 1)])

    you = next((k for k, a in run.get("Actors", {}).items() if a.get("Kind") == "you" and a.get("MaxHp")), None)
    player: list = []
    if you is not None:
        max_hp = run["Actors"][you]["MaxHp"]
        # By time only, keeping the order of samples at the same moment (a hit's health comes after the one before.)
        points = sorted(([rel(s["S"]), round(100 * s["Hp"] / max_hp, 1)] for s in run.get("Samples", [])
                         if str(s.get("Id")) == you and "Hp" in s), key=lambda p: p[0])
        points = [p for p in points if p[0] >= 0] or points[-1:]
        for point in points:
            if not player or point[1] != player[-1][1]:
                player.append(point)

    npcs, events = {}, []
    for key, actor in actors.items():
        curve = sorted(curves[key])
        before = [p for p in curve if p[0] < 0]
        curve = before[-1:] + [p for p in curve if p[0] >= 0]  # only how it stood when the fight started
        compact = []
        for point in curve:  # keep it small: only where the health changes
            if not compact or point[1] != compact[-1][1]:
                compact.append(point)
        spawn = rel(actor["Seen"])
        # When its health reached 0, which is when Kawari calls onActorDeath; retail's Defeated actor control comes
        # about 2 s later, after the death animation.
        death = rel(actor["Died"]) if "Died" in actor else None
        zero = next((p[0] for p in compact if p[1] <= 0), None)
        if zero is not None and (death is None or zero < death):
            death = zero
        npcs[str(actor["LayoutId"])] = {
            "spawn": spawn if spawn > 1.0 else None,
            "death": death,
            "hp": compact,
        }
        if spawn > 1.0:
            events.append([spawn, "spawn", actor["LayoutId"]])

    last_sequence = None
    for t, text in _director_events(run, "sequence"):
        sequence = _number_after(text, "sequence")
        if t < commence - 1.0 or sequence in DONE_SEQUENCES or sequence == last_sequence:
            last_sequence = sequence if t >= commence - 1.0 else last_sequence
            continue
        last_sequence = sequence
        events.append([max(rel(t), 0.0), "sequence", sequence])
    for t, text in _director_events(run, "battle talk"):
        events.append([rel(t), "talk", _number_after(text, "text")])
    won = next((t for t, n in scenes if n == SCENE_COMPLETE), None)
    if won is not None:
        events.append([rel(won), "won"])

    durations = {}
    for t, n in scenes:
        if n in (SCENE_COMMENCE, SCENE_COMPLETE) or t < commence:
            continue
        end = next((e for e, m in ends if m == n and e >= t), None)
        if end is not None:
            durations[str(n)] = round(end - t, 2)
    events.sort(key=lambda e: (e[0], e[1]))
    fixture = {"quest_battle": quest_battle, "source": source, "tolerance": DEFAULT_TOLERANCE, "npcs": npcs,
               "scenes": durations, "events": events}
    if player:
        fixture["player"] = player
    return fixture


def save(fixture: dict, path: Path | None = None) -> Path:
    path = path or BATTLES / f"{fixture['quest_battle']}.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    npcs = ",\n".join(f'    "{k}": {json.dumps(v)}' for k, v in fixture["npcs"].items())
    events = ",\n".join(f"    {json.dumps(e)}" for e in fixture["events"])
    head = {k: v for k, v in fixture.items() if k not in ("npcs", "events")}
    text = json.dumps(head, indent=2)[:-2] + f',\n  "npcs": {{\n{npcs}\n  }},\n  "events": [\n{events}\n  ]\n}}\n'
    json.loads(text)
    path.write_text(text)
    return path


# --- simulating -------------------------------------------------------------------------------------------------------

def _health_at(curve: list, t: float) -> float:
    if not curve or t <= curve[0][0]:
        return curve[0][1] if curve and t >= 0 else 100.0
    for (a, x), (b, y) in zip(curve, curve[1:]):
        if a <= t <= b:
            return x + (y - x) * (t - a) / (b - a) if b > a else y
    return curve[-1][1]


def _health_then(curve: list, t: float) -> float:
    """The last health seen at or before t: a player's health jumps (a hit, a potion), so it isn't interpolated."""
    value = 100.0
    for at, pct in curve:
        if at > t:
            break
        value = pct
    return value


def dropin_layouts() -> set[int]:
    layouts = set()
    for path in DROPINS.glob("*.json"):
        try:
            data = json.loads(path.read_text())
        except json.JSONDecodeError:
            continue
        for layer in data.get("layers", []):
            for obj in layer.get("objects", []):
                layouts.add(obj.get("layout_id"))
    return layouts


@dataclass
class SimResult:
    quest_battle: int
    script: Path | None
    ok: bool = False
    problems: list[str] = field(default_factory=list)
    matched: list[tuple] = field(default_factory=list)  # (kind, value, retail time, script time)
    calls: list[tuple] = field(default_factory=list)  # (time, name, args)
    error: str | None = None


def simulate(fixture: dict, script: Path | None = None) -> SimResult:
    try:
        from lupa.lua54 import LuaRuntime
    except ImportError:
        return SimResult(fixture["quest_battle"], script, error="battle-sim needs lupa: pip install lupa")
    number = fixture["quest_battle"]
    script = script or QUEST_BATTLE_SCRIPTS / f"{number}.lua"
    result = SimResult(number, script)
    if not Path(script).exists():
        result.error = f"no script {script}"
        return result

    lua = LuaRuntime(unpack_returned_tuples=True)
    clock = [0.0]
    calls = result.calls

    def log(name, *args):
        calls.append((clock[0], name, [dict(a.items()) if hasattr(a, "items") else a for a in args]))

    lua.globals().LOG = log
    lua.execute((repo.SCRIPTS / "content" / "lib" / "Battle.lua").read_text())
    director = lua.eval("""setmetatable({}, { __index = function(_, name)
        return function(_, ...) LOG(name, ...) end
    end })""")
    try:
        lua.execute(Path(script).read_text())
    except Exception as error:  # Lua errors
        result.error = f"script error: {error}"
        return result
    g = lua.globals()

    npcs = {int(k): v for k, v in fixture["npcs"].items()}
    spawned: dict[int, float] = {}  # layout -> when the script spawned it
    alive: set[int] = set()
    dead: set[int] = set()
    scenes_due: list[tuple[float, int]] = []
    seen_calls = 0
    # Spawned outside onUpdate: not in the next update's health yet (the server spawns them after that update.)
    not_yet: set[int] = set()
    end = max([e[0] for e in fixture["events"]] or [0]) + AFTER_LAST_EVENT
    known = dropin_layouts()

    def follow_calls(outside_update: bool = True):
        nonlocal seen_calls
        for t, name, args in calls[seen_calls:]:
            if name == "spawn_bnpc" and args and args[0] not in spawned:
                layout = int(args[0])
                spawned[layout] = t
                alive.add(layout)
                if outside_update:
                    not_yet.add(layout)
                if layout not in known:
                    result.problems.append(f"spawns {layout}, which no drop-in has")
                elif layout not in npcs:
                    result.problems.append(f"spawns {layout} at {t:.1f} s, which retail never had")
            elif name == "despawn_bnpc" and args:
                alive.discard(int(args[0]))
            elif name == "play_scene" and args:
                scene = int(args[0])
                scenes_due.append((t + fixture.get("scenes", {}).get(str(scene), 1.0), scene))
        seen_calls = len(calls)

    try:
        # Before the opening cutscene (onSetup): not compared, retail's sequence 1 comes then too.
        clock[0] = -1.0
        if g.onSetup is not None:
            g.onSetup(director)
        clock[0] = 0.0
        g.onCommence(director)
        follow_calls()
        t = 0.0
        while t <= end:
            clock[0] = t
            for due in [d for d in scenes_due if d[0] <= t]:
                scenes_due.remove(due)
                if g.onSceneFinished is not None:
                    g.onSceneFinished(director, due[1])
            follow_calls()
            # Deaths, by retail's clock counted from when the script spawned each one.
            for layout in sorted(alive):
                info = npcs.get(layout)
                if info is None or info.get("death") is None:
                    continue
                offset = spawned[layout] - (info["spawn"] or 0.0)
                if t >= info["death"] + offset:
                    alive.discard(layout)
                    dead.add(layout)
                    if g.onActorDeath is not None:
                        g.onActorDeath(director, layout, None)
                    follow_calls()
            health, info_table = {}, {}
            for layout in alive - not_yet:
                spec = npcs.get(layout)
                offset = spawned[layout] - ((spec or {}).get("spawn") or 0.0)
                pct = _health_at(spec["hp"], t - offset) if spec else 100.0
                health[layout] = max(pct, 0.1)
                info_table[layout] = lua.table_from({"hp": pct, "max_hp": 100, "x": 0, "y": 0, "z": 0})
            player_pct = _health_then(fixture["player"], t) if fixture.get("player") else 100.0
            info_table["player"] = lua.table_from({"hp": player_pct, "max_hp": 100, "x": 0, "y": 0, "z": 0})
            g.onUpdate(director, lua.table_from(health), player_pct, lua.table_from(info_table))
            not_yet.clear()
            follow_calls(outside_update=False)
            t = round(t + TICK, 2)
    except Exception as error:  # Lua errors
        result.error = f"script error at {clock[0]:.1f} s: {str(error).strip().splitlines()[0]}"
        return result

    result.problems += compare(fixture, calls, result)
    result.ok = not result.problems
    return result


def script_events(calls: list[tuple]) -> list[list]:
    events = []
    for t, name, args in calls:
        if t < 0:
            continue
        if name == "set_sequence" and args and args[0] not in DONE_SEQUENCES:
            events.append([t, "sequence", int(args[0])])
        elif name == "battle_talk" and len(args) >= 3:
            events.append([t, "talk", int(args[2])])
        elif name == "spawn_bnpc" and args:
            events.append([t, "spawn", int(args[0])])
        elif name == "complete_quest_battle":
            events.append([t, "won"])
    return events


def compare(fixture: dict, calls: list[tuple], result: SimResult) -> list[str]:
    """Matches retail's events to the script's: the same kind and value, in order, within the tolerance."""
    tolerance = fixture.get("tolerance", DEFAULT_TOLERANCE)
    mine = script_events(calls)
    initial = {int(k) for k, v in fixture["npcs"].items() if v.get("spawn") is None}
    mine = [e for e in mine if not (e[1] == "spawn" and e[2] in initial and e[0] == 0.0)]
    used = [False] * len(mine)
    problems = []
    for event in fixture["events"]:
        t, kind = event[0], event[1]
        value = event[2] if len(event) > 2 else None
        found = None
        for i, other in enumerate(mine):
            if not used[i] and other[1] == kind and (other[2] if len(other) > 2 else None) == value:
                found = i
                break
        label = f"{kind} {value}" if value is not None else kind
        if found is None:
            problems.append(f"{label} at {t:.1f} s on retail: the script never did it")
            continue
        used[found] = True
        result.matched.append((kind, value, t, mine[found][0]))
        # A fixture can allow one event more slack (`tolerances`, by label) where the runs disagree.
        if abs(mine[found][0] - t) > fixture.get("tolerances", {}).get(label, tolerance):
            problems.append(f"{label}: {mine[found][0]:.1f} s, retail {t:.1f} s")
    for i, other in enumerate(mine):
        if not used[i]:
            label = f"{other[1]} {other[2]}" if len(other) > 2 else other[1]
            problems.append(f"{label} at {other[0]:.1f} s: retail didn't")
    return problems


def all_fixtures() -> list[Path]:
    return sorted(BATTLES.glob("*.json"), key=lambda p: int(p.stem) if p.stem.isdigit() else 0)


def run_all(numbers: list[int] | None = None) -> list[SimResult]:
    results = []
    for path in all_fixtures():
        fixture = json.loads(path.read_text())
        if numbers and fixture["quest_battle"] not in numbers:
            continue
        results.append(simulate(fixture))
    return results


def format_results(results: list[SimResult], verbose: bool = False) -> str:
    lines = []
    for result in results:
        if result.error:
            lines.append(f"FAIL quest battle {result.quest_battle}: {result.error}")
            continue
        status = "ok  " if result.ok else "FAIL"
        lines.append(f"{status} quest battle {result.quest_battle}: {len(result.matched)} events matched retail"
                     + ("" if result.ok else f", {len(result.problems)} problems"))
        for problem in result.problems:
            lines.append(f"       {problem}")
        if verbose:
            for kind, value, retail, mine in result.matched:
                label = f"{kind} {value}" if value is not None else kind
                lines.append(f"       {label:20} retail {retail:7.1f} s   script {mine:7.1f} s   ({mine - retail:+.1f})")
    passed = sum(1 for r in results if r.ok)
    lines.append(f"{passed} of {len(results)} quest battles match retail")
    return "\n".join(lines)
