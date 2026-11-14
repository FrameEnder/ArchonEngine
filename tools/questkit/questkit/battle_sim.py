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
A battle whose director played scenes of its own between the opening and the fight (The Lominsan Way's scene 2, 16 s
of battle talk before scene 60000) has them in `lead_in` (`{"2": 16.04}`, how long the client took over each): Kawari
calls `onCommence` as the opening ends, so the simulator does so when retail's first of them began (`lead_in_from`;
without it, as long before 0 as they took together), and the script plays them from there (`onSceneFinished` starts the fight); what retail did meanwhile (its sequence, battle talk) comes at negative
times.
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

Positions: a fixture can hold retail's (`positions`: `{"player": [[t, x, y, z], ...], "<layout>": [...]}`, an NPC's
times counted like its health), which the script sees in `info` as the server gives them, for steps that wait for the
player to come close (Victory in Peril's second pair and cutscene). Without them everyone stands at the origin, so
"nearest" choices are arbitrary and anything waiting for the player to come close happens at once.

Casts: a fixture can hold the NPCs' casts (`casts`: `[[t, layout, action], ...]`, from retail's ActorCast, timed like
their health), which the script hears as `onActorCast(director, layout, action, 0)`, as the server calls it when an
NPC begins an action (the magitek vanguard's Drill Cannons move battles 46-48 on). Instant actions aren't in it.

Object talks: a fixture can hold when the player talked to an event object the director brought out (`object_talks`:
`[[t, base, bar], ...]`, from retail's StartTalkEvent on it, and how long the event action that followed took), which
the script hears as `onObjectTalk(director, 1, 2, base)` if it has spawned that object by then (`spawn_eobj`); an
`event_action` it answers with ends `bar` seconds later (`onEventActionCast(director, 1, 2)`), as the server's does (In
the Eyes of Gods and Men's rosary, quest battle 60.)

Pools: NPCs retail brought out at random from a few, over and over (quest battle 86's bombs: one of three layouts each
time), are listed in `pools` (`[[4317589, 4317648, 4317650]]`). A spawn of any of a pool matches a retail spawn of any
other of it, the script can have several out at once, and they fall when the script has them fall (their retail deaths
aren't used.)

What it doesn't simulate: enmity and who hits whom (the health curves are retail's whatever the script's targeting does), and losing. The player's health
follows retail's curve when the fixture has one (`player`, for lines said as the player's health drops), full otherwise.
"""

from __future__ import annotations

import json
import struct
from dataclasses import dataclass, field
from pathlib import Path

from . import repo

BATTLES = repo.ROOT / "tools" / "questkit" / "battles"
QUEST_BATTLE_SCRIPTS = repo.SCRIPTS / "content" / "questbattle"
DROPINS = repo.ROOT / "resources" / "dropins"
TICK = 0.5
SCENE_COMMENCE = 60000
CONTROL_SET_STATUS = 2  # ActorControl category: the actor's status (2: dead)
STATUS_DEAD = 2
SCENE_COMPLETE = 60001
DONE_SEQUENCES = (0xFE, 0xFF)  # set by the server once a battle is over, not by the script
DEFAULT_TOLERANCE = 2.0
AFTER_LAST_EVENT = 30.0  # how long to keep simulating after retail's last event
SETUP_CLOCK = -1000.0  # onSetup's time: before the opening, never compared


# --- making a fixture from a retail run -----------------------------------------------------------------------------

def _director_events(run: dict, kind: str) -> list[tuple[float, str]]:
    return [(e["S"], e["Text"]) for e in run.get("Events", []) if e.get("Kind") == kind]


def _talk_events(run: dict) -> list[tuple[float, str]]:
    """The battle talks: the run's own "battle talk" events (a Chronofoil battle has them), or, in a Kawari Recorder
    battle file (2.7 logs no such event), its DirectorPopupMessage packets (by its game version's opcode), read as
    chronofoil_battle does."""
    talks = _director_events(run, "battle talk")
    if talks or not run.get("Packets"):
        return talks
    opcodes = repo.server_opcodes("DirectorPopupMessage", run.get("GameVersion"))
    for packet in run["Packets"]:
        data = bytes.fromhex(packet.get("Hex", ""))
        if packet.get("Op") not in opcodes or len(data) < 28:
            continue
        npc_name, text, duration, icon = struct.unpack_from("<IIII", data, 12)
        talks.append((packet["S"], f"text {text} name {npc_name} icon {icon} {duration} ms"))
    return sorted(talks)


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

    # The director's own scenes between the opening (the first scene) and the fight: how long each took.
    lead_in = {}
    for t, n in scenes[1:]:
        if t >= commence or n in (SCENE_COMMENCE, SCENE_COMPLETE):
            continue
        end = next((e for e, m in ends if m == n and e >= t), None)
        if end is not None:
            lead_in[str(n)] = round(end - t, 2)
    lead_in_start = next((t for t, n in scenes[1:] if str(n) in lead_in), None)

    actors = {k: a for k, a in run.get("Actors", {}).items() if a.get("Kind") == "battle npc" and a.get("LayoutId")}
    curves: dict[str, list] = {k: [] for k in actors}

    def percent(hp, max_hp):
        # A few HP left isn't 0 (In the Eyes of Gods and Men's boss stood at 1 of 9150 for 3 s, Bob's Chronofoil capture
        # 3c56e09f: rounded, it fell then.)
        return max(round(100 * hp / max_hp, 1), 0.1) if hp > 0 else 0.0

    for sample in run.get("Samples", []):
        key = str(sample.get("Id"))
        if key in curves and "Hp" in sample and actors[key].get("MaxHp"):
            pct = percent(sample["Hp"], actors[key]["MaxHp"])
            if not curves[key] or curves[key][-1][1] != pct:
                curves[key].append([rel(sample["S"]), pct])
    for action in run.get("Actions", []):  # each hit says its target's HP too
        for target in action.get("Targets", []):
            key = str(target.get("Id"))
            if key in curves and "Hp" in target and target.get("MaxHp"):
                curves[key].append([rel(action["S"]), percent(target["Hp"], target["MaxHp"])])

    # An ActorControl 2 with 2 (its status: dead) is when it fell, even when the killing blow's own HP never came: the
    # boss of Notorious Biggs stood at 25 HP (0.2 %) until it, 2.1 s before Defeated (Bob's Chronofoil capture
    # da424662 of 2026-11-12); a curve that stopped above 0 put its death at Defeated, 2 s late.
    for control in run.get("Controls", []):
        key = str(control.get("Id"))
        if key in curves and control.get("Category") == CONTROL_SET_STATUS and \
                (control.get("Params") or [None])[0] == STATUS_DEAD:
            curves[key].append([rel(control["S"]), 0.0])

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
    # By when each was seen: a layout ID retail spawned twice (Trial by Wind's 4248510) is the first one's NPC, and the
    # second only a spawn event.
    for key, actor in sorted(actors.items(), key=lambda item: item[1]["Seen"]):
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
        if str(actor["LayoutId"]) in npcs:
            if spawn > 1.0:
                events.append([spawn, "spawn", actor["LayoutId"]])
            continue
        npcs[str(actor["LayoutId"])] = {
            "spawn": spawn if spawn > 1.0 else None,
            "death": death,
            "hp": compact,
        }
        if spawn > 1.0:
            events.append([spawn, "spawn", actor["LayoutId"]])

    last_sequence = None
    sequences = _director_events(run, "sequence")
    for n, (t, text) in enumerate(sequences):
        sequence = _number_after(text, "sequence")
        # One set before the fight and replaced by the time it starts is the script's onSetup (Trial by Wind's
        # sequence 1, 0.4 s before scene 60000 and sequence 2.)
        # One set with a lead-in scene is the script's, at its time (The Lominsan Way's sequence 2.)
        in_lead_in = lead_in_start is not None and lead_in_start - 0.2 <= t < commence - 1.0
        replaced = t < commence and any(t < later <= commence for later, _ in sequences[n + 1:])
        if not in_lead_in and (t < commence - 1.0 or replaced) or sequence in DONE_SEQUENCES \
                or sequence == last_sequence:
            last_sequence = sequence if t >= commence - 1.0 else last_sequence
            continue
        last_sequence = sequence
        events.append([rel(t) if in_lead_in else max(rel(t), 0.0), "sequence", sequence])
    for t, text in _talk_events(run):
        events.append([rel(t), "talk", _number_after(text, "text")])
    won = next((t for t, n in scenes if n == SCENE_COMPLETE), None)
    if won is not None:
        events.append([rel(won), "won"])

    # The NPCs' casts (ActorCast: "action N ..."), by layout ID.
    casts = []
    for event in run.get("Events", []):
        if event.get("Kind") != "cast":
            continue
        actor = actors.get(str(event.get("Id")))
        action = _number_after(event.get("Text", ""), "action")
        if actor is None or not actor.get("LayoutId") or action is None or rel(event["S"]) < 0:
            continue
        casts.append([rel(event["S"]), actor["LayoutId"], action])

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
    if lead_in:
        fixture["lead_in"] = lead_in
        fixture["lead_in_from"] = rel(lead_in_start)
    if player:
        fixture["player"] = player
    if casts:
        fixture["casts"] = casts
    return fixture


def save(fixture: dict, path: Path | None = None) -> Path:
    path = path or BATTLES / f"{fixture['quest_battle']}.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    npcs = ",\n".join(f'    "{k}": {json.dumps(v)}' for k, v in fixture["npcs"].items())
    events = ",\n".join(f"    {json.dumps(e)}" for e in fixture["events"])
    head = {k: v for k, v in fixture.items() if k not in ("npcs", "events", "positions", "casts")}
    text = json.dumps(head, indent=2)[:-2] + f',\n  "npcs": {{\n{npcs}\n  }},\n  "events": [\n{events}\n  ]'
    if fixture.get("positions"):
        positions = ",\n".join(f'    "{k}": {json.dumps(v)}' for k, v in fixture["positions"].items())
        text += f',\n  "positions": {{\n{positions}\n  }}'
    if fixture.get("casts"):
        casts = ",\n".join(f"    {json.dumps(c)}" for c in fixture["casts"])
        text += f',\n  "casts": [\n{casts}\n  ]'
    text += "\n}\n"
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


def _position_then(path: list | None, t: float) -> tuple[float, float, float]:
    """Where someone was at t: their last recorded position (the origin without any.)"""
    position = (0.0, 0.0, 0.0)
    for at, x, y, z in path or []:
        if at > t:
            break
        position = (x, y, z)
    return position


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
    positions = fixture.get("positions", {})
    spawned: dict[int, float] = {}  # layout -> when the script spawned it
    alive: set[int] = set()
    dead: set[int] = set()
    scenes_due: list[tuple[float, int]] = []
    seen_calls = 0
    # Spawned outside onUpdate: not in the next update's health yet (the server spawns them after that update.)
    not_yet: set[int] = set()
    end = max([e[0] for e in fixture["events"]] or [0]) + AFTER_LAST_EVENT
    # Kawari's onCommence comes as the opening ends: before the director's lead-in scenes, if retail had any.
    lead_in = fixture.get("lead_in", {})
    pooled = pool_of(fixture)
    # When retail's first lead-in scene began (`lead_in_from`), else as long before the fight as they took together
    # (fixtures made before 2026-11-08: The Lominsan Way's fight came as its scene 2 ended.)
    start = round(fixture.get("lead_in_from", -sum(lead_in.values())) * 2) / 2
    known = dropin_layouts()
    standing, _ = retail_line_up(fixture)  # the event NPCs retail had there, which a fixture's NPCs don't list
    cast_heard: set[tuple] = set()
    eobjs: set[int] = set()  # the event objects the script brought out (spawn_eobj), by base
    talks_due = [list(talk) for talk in fixture.get("object_talks", [])]
    bars_due: list[float] = []  # when the event actions the script started on an object talk end

    def follow_calls(outside_update: bool = True):
        nonlocal seen_calls
        for t, name, args in calls[seen_calls:]:
            if name == "spawn_bnpc" and args and (args[0] not in spawned or int(args[0]) in pooled):
                layout = int(args[0])
                # Brought out before the fight (a lead-in): its health follows retail's from the fight's start.
                spawned[layout] = max(t, 0.0)
                alive.add(layout)
                if outside_update:
                    not_yet.add(layout)
                if layout not in known:
                    result.problems.append(f"spawns {layout}, which no drop-in has")
                elif layout not in npcs and layout not in standing and not any(
                        other in npcs for other, root in pooled.items() if layout in pooled and root == pooled[layout]):
                    # (One of a pool counts as retail's when another of it is: retail drew one at random each time,
                    # Austerities of Wind's plumes.)
                    result.problems.append(f"spawns {layout} at {t:.1f} s, which retail never had")
            elif name == "despawn_bnpc" and args:
                alive.discard(int(args[0]))
            elif name == "spawn_eobj" and args:
                eobjs.add(int(args[0]))
            elif name == "play_scene" and args:
                scene = int(args[0])
                took = lead_in.get(str(scene)) if t < 0 else None
                scenes_due.append((t + (took or fixture.get("scenes", {}).get(str(scene), 1.0)), scene))
        seen_calls = len(calls)

    try:
        # Before the opening cutscene (onSetup): not compared, retail's sequence 1 comes then too.
        clock[0] = SETUP_CLOCK
        if g.onSetup is not None:
            g.onSetup(director)
        clock[0] = start
        g.onCommence(director)
        follow_calls()
        t = start
        player_fell = [False]
        while t <= end:
            clock[0] = t
            for due in [d for d in scenes_due if d[0] <= t]:
                scenes_due.remove(due)
                if g.onSceneFinished is not None:
                    g.onSceneFinished(director, due[1])
            follow_calls()
            # Casts, by retail's clock counted from when the script spawned the caster.
            for cast in fixture.get("casts", []):
                at, layout, action = cast
                if layout not in alive or (at, layout, action) in cast_heard:
                    continue
                info = npcs.get(layout) or {}
                if t >= at + spawned[layout] - (info.get("spawn") or 0.0):
                    cast_heard.add((at, layout, action))
                    if g.onActorCast is not None:
                        g.onActorCast(director, layout, action, 0)
                    follow_calls()
            # The player talking to an object the script brought out, and the event action it answered with.
            for talk in [d for d in talks_due if d[0] <= t and d[1] in eobjs]:
                talks_due.remove(talk)
                before = len(calls)
                if g.onObjectTalk is not None:
                    g.onObjectTalk(director, 1, 2, talk[1])
                if any(name == "event_action" for _, name, _ in calls[before:]):
                    bars_due.append(t + talk[2])
                follow_calls()
            for due in [d for d in bars_due if d <= t]:
                bars_due.remove(due)
                if g.onEventActionCast is not None:
                    g.onEventActionCast(director, 1, 2)
                follow_calls()
            # Deaths, by retail's clock counted from when the script spawned each one.
            for layout in sorted(alive):
                info = npcs.get(layout)
                if info is None or info.get("death") is None or layout in pooled:
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
                # `"npc_health": "held"`: an NPC's health only changes when it's hit, so it holds between samples
                # (a run where the player stopped attacking for a while); otherwise it's interpolated.
                at = _health_then if fixture.get("npc_health") == "held" else _health_at
                pct = at(spec["hp"], t - offset) if spec else 100.0
                health[layout] = max(pct, 0.1)
                x, y, z = _position_then(positions.get(str(layout)), t - offset)
                info_table[layout] = lua.table_from({"hp": pct, "max_hp": 100, "x": x, "y": y, "z": z})
            player_pct = _health_then(fixture["player"], t) if fixture.get("player") else 100.0
            # The player falling, as the server tells the director (a lost run: Like Mother, Like Daughter's kept
            # spawning after retail's run had ended.)
            if player_pct <= 0 and not player_fell[0]:
                player_fell[0] = True
                if g.onPlayerDeath is not None:
                    g.onPlayerDeath(director)
                follow_calls()
            x, y, z = _position_then(positions.get("player"), t)
            info_table["player"] = lua.table_from({"hp": player_pct, "max_hp": 100, "x": x, "y": y, "z": z})
            g.onUpdate(director, lua.table_from(health), player_pct, lua.table_from(info_table))
            not_yet.clear()
            follow_calls(outside_update=False)
            t = round(t + TICK, 2)
    except Exception as error:  # Lua errors
        result.error = f"script error at {clock[0]:.1f} s: {str(error).strip().splitlines()[0]}"
        return result

    result.problems += spawn_problems(fixture, calls)
    result.problems += line_up_problems(fixture, calls)
    result.problems += compare(fixture, calls, result)
    result.ok = not result.problems
    return result


def pool_of(fixture: dict) -> dict[int, int]:
    """Each pooled layout's pool, by its first layout (see `pools`.)"""
    pools = {}
    for pool in fixture.get("pools", []):
        for layout in pool:
            pools[int(layout)] = int(pool[0])
    return pools


RETAIL_SPAWNS = repo.ROOT / "tools" / "questkit" / "retail_spawns.json"
EVENT_NPC = 3


def retail_line_up(fixture: dict, registry: dict | None = None) -> tuple[dict[int, int], dict[int, int]]:
    """What every retail run of this battle's territory had that a fixture made from one run's battle NPCs doesn't
    say (`retail_spawns.json`, from every capture): the event NPCs standing there (layout -> ENpcBase), and the battle
    NPCs that never moved or acted (layout -> how many runs). Only those seen in as many runs as the fixture's own
    NPCs, so someone who is there in some runs only isn't asked for.

    Every Little Thing She Does Is Magitek (quest battle 64) passed with neither: its two event NPCs had no one
    standing for them, and its reaper, which retail never moved, walked up to the boss as a tank (Bob's mark of
    2026-11-14)."""
    if registry is None:
        registry = json.loads(RETAIL_SPAWNS.read_text()) if RETAIL_SPAWNS.exists() else {}
    mine = set(fixture.get("npcs", {}))
    zone = max(registry.values(), key=lambda spots: len(mine & set(spots)), default={})
    runs = max((zone[layout]["visits"] for layout in mine & set(zone)), default=0)
    if not runs:
        return {}, {}
    standing = {int(layout): entry["base"] for layout, entry in zone.items()
                if entry.get("kind") == EVENT_NPC and entry["visits"] >= runs}
    still = {int(layout): entry["visits"] for layout, entry in zone.items()
             if entry.get("kind") != EVENT_NPC and entry["visits"] >= runs
             and entry.get("moved") is False and entry.get("acted") is False}
    return standing, still


def line_up_problems(fixture: dict, calls: list[tuple], registry: dict | None = None) -> list[str]:
    """The script against `retail_line_up`: every event NPC brought out, every battle NPC that never moved or acted
    held (`director:hold_bnpc`, which a role's `still = true` does) and not let go."""
    standing, still = retail_line_up(fixture, registry)
    out = {int(args[0]) for _, name, args in calls if name == "spawn_bnpc" and args}
    held: set[int] = set()
    for _, name, args in calls:
        if name == "hold_bnpc" and args and args[0] is not None:
            if len(args) > 1 and args[1] is False:
                held.discard(int(args[0]))
            else:
                held.add(int(args[0]))
    problems = []
    # Someone who only comes later (the registry's `late`: as a cutscene ends, say) isn't asked of a run that may not
    # have got there: one that was lost.
    registry = registry if registry is not None else (
        json.loads(RETAIL_SPAWNS.read_text()) if RETAIL_SPAWNS.exists() else {})
    late = {int(layout) for spots in registry.values() for layout, entry in spots.items() if entry.get("late")}
    won = any(event[1] == "won" for event in fixture.get("events", []))
    for layout, base in sorted(standing.items()):
        if layout not in out and (won or layout not in late):
            problems.append(f"retail had event NPC {layout} (ENpcBase {base}) standing there in every run: the "
                            "script never brings it out")
    for layout, runs in sorted(still.items()):
        if layout in out and layout not in held:
            problems.append(f"retail's {layout} never moved or acted ({runs} runs): the script doesn't hold it "
                            "(director:hold_bnpc, or `still = true` in its role)")
    return problems


def spawn_problems(fixture: dict, calls: list[tuple]) -> list[str]:
    """Who the script brings out against retail's line-up: everyone retail had from the start (the timed events only
    cover later arrivals), and nobody more often than retail (Like Mother, Like Daughter's first wave, written the wrong
    way round, brought the three conjurers out a second time as enemies and never the 1683s, and passed: Bob's bundle
    20261101-221536.)"""
    problems = []
    pooled = pool_of(fixture)  # a pool's spawns are counted together, under its first layout
    script_count: dict[int, int] = {}
    # onSetup's spawns count too: they're out before the opening cutscene, as retail's first line-up (The Rematch's 39,
    # quest battle 23, fighting in the background from the start.)
    for t, name, args in calls:
        if name == "spawn_bnpc" and args and t >= SETUP_CLOCK:
            layout = pooled.get(int(args[0]), int(args[0]))
            script_count[layout] = script_count.get(layout, 0) + 1
    retail_count: dict[int, int] = {}
    for layout, info in fixture["npcs"].items():
        if info.get("spawn") is None:
            retail_count[int(layout)] = 1
    for event in fixture["events"]:
        if event[1] == "spawn":
            layout = pooled.get(event[2], event[2])
            retail_count[layout] = retail_count.get(layout, 0) + 1
    for layout in sorted(retail_count):
        if fixture["npcs"].get(str(layout), {}).get("spawn", 0) is None and layout not in script_count:
            problems.append(f"retail had {layout} from the start: the script never brings it out")
    for layout, count in sorted(script_count.items()):
        if layout in retail_count and count > retail_count[layout]:
            problems.append(f"brings {layout} out {count} times, retail {retail_count[layout]}")
    return problems


def script_events(calls: list[tuple]) -> list[list]:
    events = []
    for t, name, args in calls:
        if t <= SETUP_CLOCK:
            continue
        if name == "set_sequence" and args and args[0] not in DONE_SEQUENCES:
            events.append([t, "sequence", int(args[0])])
        elif name == "battle_talk" and len(args) >= 3:
            events.append([t, "talk", int(args[2])])
        elif name == "spawn_bnpc" and args:
            events.append([t, "spawn", int(args[0])])
        elif name == "complete_quest_battle" and not (len(args) >= 6 and args[5] is False):
            # "won" is "Duty Complete" (scene 60001), which a battle won without the fanfare doesn't play.
            events.append([t, "won"])
    return events


def compare(fixture: dict, calls: list[tuple], result: SimResult) -> list[str]:
    """Matches retail's events to the script's: the same kind and value, in order, within the tolerance."""
    tolerance = fixture.get("tolerance", DEFAULT_TOLERANCE)
    pooled = pool_of(fixture)
    # A pool's spawns are one kind of event (see `pools`.)
    # (The event NPCs standing by aren't in a fixture's events: `line_up_problems` holds the script to them.)
    standing, _ = retail_line_up(fixture)
    mine = [[e[0], e[1], pooled.get(e[2], e[2])] if e[1] == "spawn" else e for e in script_events(calls)
            if not (e[1] == "spawn" and e[2] in standing)]
    initial = {int(k) for k, v in fixture["npcs"].items() if v.get("spawn") is None}
    mine = [e for e in mine if not (e[1] == "spawn" and e[2] in initial and e[0] <= 0.0)]
    used = [False] * len(mine)
    problems = []
    for event in fixture["events"]:
        t, kind = event[0], event[1]
        value = event[2] if len(event) > 2 else None
        if kind == "spawn":
            value = pooled.get(value, value)
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
