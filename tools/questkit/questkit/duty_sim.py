"""Dungeons and trials played headless: a duty's director script run against what the same duty did on retail, as
`battle_sim.py` does quest battles (docs/src/fork/testing.md).

A **duty fixture** (`tools/questkit/duties/<ContentFinderCondition>-<capture>.json`, numbers only) is one retail run
of the duty from a Chronofoil capture (`questkit chronofoil --battle N.V`), in seconds from the duty commencing
(the director's 0x40000001, Kawari's `onCommence`):

* `npcs`: by layout ID, each of its lives (one, or one per attempt when the party wiped): when it appeared (`spawn`,
  null: there from the start), when it first had a target (`pulled`), when its health reached 0 (`death`), its health
  in percent over time (`hp`), its maximum HP and where it stood (`pos`, when it moved);
* `player` and `party`: where the capturing player and the rest of the party (players, Duty Support members) stood
  over time, and the player's health; `players`: which of the party were players (`info` has those as `player2`...
  too, as the server's does);
* `casts`: the NPCs' casts by layout ID;
* `wipes`: when the party wiped and when the duty recommenced (the director's 0x40000005 and 0x40000006);
* `ranges` (written by hand): when the party walked into one of the script's event ranges (`onGimmickRect`), which a
  capture doesn't say: `[[seconds, range ID]]` (Sastasha's cove, as its boss first appeared);
* `bars`: the progress bars of the party's event actions (a door being opened: ActorControlTarget 3 with 1 and the
  EventAction row as it began, 0 as it ended): when, the row, how long;
* `events`, what the script must do as retail did: the boss music (`bgm`, the director's 0x80000001), objectives done
  (`objective`, 0x80000016), the director's sequences, and `complete` (0x40000003).

The simulator runs the real script in Lua 5.4 as the server does: `onSetup`, `onCommence` at 0, `onUpdate` every half
second with every living NPC's health and details (`info[layout] = { hp, max_hp, x, y, z, target, id }`, `player1`,
`party1`...) from retail's run, `onActorDeath` and `onActorCast` when retail's NPCs fell and cast, `onPartyWipe` and
`onDutyRecommence` at retail's wipes, and at each of retail's progress bars `onGimmickAccessor` for the script's event
objects (its `EOBJ_*` globals) until one answers with that event action, whose `onEventActionCast` follows when the
bar ended. A director call after `director:delay(n)` counts n seconds later, as on the
server. An NPC retail brought out during the run is there once the script brings it out (`spawn_bnpc`, `spawn_boss`),
its life counted from then.

Checked: no Lua error; the script's music, objectives, sequences and completion against retail's, in order, each
within the tolerance. (Not that what it spawns exists: a duty's NPCs are in the game's own layout as well as the
drop-ins, and the game's isn't here.)

`rehearse_wipes` is the check no capture is needed for: in each boss fight, a wipe halfway through the fight and the
fight again must make the script do the second time what it did the first (a script that forgets to reset a flag in
`onPartyWipe` does less, or nothing: the boss that never casts again after a wipe.)

What it doesn't simulate: the server's own part (enmity, who hits whom, the party's NPCs, the wipe's return, packets):
`servers/world/src/server/bundle_tests.rs` has that side, without the game's data.
"""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from pathlib import Path

from . import repo
from .battle_sim import _health_then, _number_after, _position_then

DUTIES = repo.ROOT / "tools" / "questkit" / "duties"
CONTENT_SCRIPTS = repo.SCRIPTS / "content"
TICK = 0.5
DEFAULT_TOLERANCE = 3.0
AFTER_LAST_EVENT = 20.0
SETUP_CLOCK = -1000.0
DONE_SEQUENCES = (0xFE, 0xFF)

# The director's events (ActorControlSelf 109: the director, the event, its arguments.)
DIRECTOR_EVENT = 109
DUTY_COMMENCE, DUTY_COMPLETE, PARTY_WIPE, DUTY_RECOMMENCE = 0x40000001, 0x40000003, 0x40000005, 0x40000006
BGM, OBJECTIVE_DONE = 0x80000001, 0x80000016
EVENT_ACTION_BAR = 3  # ActorControlTarget: 1 and the EventAction row as a progress bar begins, 0 as it ends


# --- making a fixture -----------------------------------------------------------------------------------------------

def _sparse(points: list, step: float = 0.5) -> list:
    """Positions thinned: one kept when it's this far from the last kept."""
    kept: list = []
    for point in points:
        if not kept or sum((a - b) ** 2 for a, b in zip(point[1:], kept[-1][1:])) >= step * step:
            kept.append(point)
    return kept


def fixture_from_run(run: dict, script: str, source: str = "", unlaid: dict[int, int] | None = None) -> dict:
    """A duty fixture from a battle run of a duty visit (`questkit chronofoil --battle N.V`). `unlaid`: the script's
    IDs for NPCs retail gave no layout ID, by BNpcBase."""
    actors = run.get("Actors", {})
    you = next((k for k, a in actors.items() if a.get("Kind") == "you"), None)
    director = [(c["S"], c["Params"]) for c in run.get("Controls", [])
                if c.get("Category") == DIRECTOR_EVENT and str(c.get("Id")) == you]
    commence = next((t for t, p in director if p[1] == DUTY_COMMENCE), None)
    if commence is None:
        raise ValueError("the run has no DutyCommence (the director's 0x40000001): not a duty's visit from its start")

    def rel(t: float) -> float:
        return round(t - commence, 2)

    positions: dict[str, list] = {}
    health: dict[str, list] = {}
    targets: dict[str, list] = {}
    for sample in run.get("Samples", []):
        key = str(sample.get("Id"))
        if "P" in sample:
            positions.setdefault(key, []).append([rel(sample["S"]), *[round(v, 1) for v in sample["P"]]])
        if "Hp" in sample:
            health.setdefault(key, []).append([rel(sample["S"]), sample["Hp"]])
        if "T" in sample:
            targets.setdefault(key, []).append([rel(sample["S"]), sample["T"]])
    for action in run.get("Actions", []):  # each hit says its target's HP too
        for target in action.get("Targets", []):
            if "Hp" in target:
                health.setdefault(str(target["Id"]), []).append([rel(action["S"]), target["Hp"]])

    def percent(hp: int, max_hp: int) -> float:
        return max(round(100 * hp / max_hp, 1), 0.1) if hp > 0 else 0.0

    npcs: dict[str, list] = {}
    events: list = []
    for key, actor in sorted(actors.items(), key=lambda item: item[1].get("Seen", 0)):
        # One with no layout ID (a boss's part the script spawns under an ID of its own) is kept under the script's ID
        # for it, when `unlaid` says so by its BNpcBase (the Navel's Heart of the Mountain.)
        layout = actor.get("LayoutId") or (unlaid or {}).get(actor.get("BaseId"))
        if actor.get("Kind") != "battle npc" or not layout or not actor.get("MaxHp"):
            continue
        curve: list = []
        for t, hp in sorted(health.get(key, [])):
            pct = percent(hp, actor["MaxHp"])
            # Kept small: a point when the health has moved a percent (and as it reaches 0.)
            if not curve or abs(curve[-1][1] - pct) >= 1.0 or (pct <= 0 < curve[-1][1]):
                curve.append([t, pct])
        spawn = rel(actor["Seen"])
        death = rel(actor["Died"]) if "Died" in actor else None
        zero = next((p[0] for p in curve if p[1] <= 0), None)
        if zero is not None and (death is None or zero < death):
            death = zero
        pulled = next((t for t, target in sorted(targets.get(key, [])) if target), None)
        path = sorted(positions.get(key, [])) or [[spawn, *[round(v, 1) for v in actor["Position"]]]]
        # `Seen` is its latest appearance to the capturing player (one that left their sight and came back into it
        # has a new one): it was there from its first position.
        spawn = min(spawn, path[0][0])
        # Where it went, for one that fought a while (a boss a script may place things by); where it stood, for the rest.
        long_fight = pulled is not None and (death if death is not None else rel(actor.get("LastSeen", 0))) - pulled >= 20
        life = {"spawn": spawn if spawn > 1.0 else None, "pulled": pulled, "death": death, "hp": curve,
                "max_hp": actor["MaxHp"], "base": actor.get("BaseId", 0),
                "pos": _sparse(path, 2.0) if long_fight else path[:1],
                "gone": rel(actor["LastSeen"]) if "LastSeen" in actor else None}
        npcs.setdefault(str(layout), []).append(life)

    party = {}
    others = [k for k, a in sorted(actors.items(), key=lambda item: item[1].get("Seen", 0))
              if k != you and (a.get("Kind") == "player"
                               or (a.get("Kind") == "battle npc" and not a.get("LayoutId") and a.get("Battalion") == 0
                                   and a.get("MaxHp", 0) > 1 and len(positions.get(k, [])) > 20))]
    players = []
    for index, key in enumerate(others[:7], 1):
        party[f"party{index}"] = _sparse(sorted(positions.get(key, [])), 2.0)
        if actors[key].get("Kind") == "player":
            players.append(f"party{index}")

    player_health: list = []
    if you is not None and actors[you].get("MaxHp"):
        for t, hp in sorted(health.get(you, []), key=lambda p: p[0]):
            pct = round(100 * hp / actors[you]["MaxHp"], 1)
            if not player_health or player_health[-1][1] != pct:
                player_health.append([t, pct])

    wipes = []
    for t, params in director:
        if params[1] == PARTY_WIPE:
            wipes.append([rel(t), None])
        elif params[1] == DUTY_RECOMMENCE and wipes and wipes[-1][1] is None:
            wipes[-1][1] = rel(t)
        elif params[1] == BGM and t >= commence:
            events.append([rel(t), "bgm", params[2]])
        elif params[1] == OBJECTIVE_DONE:
            events.append([rel(t), "objective", params[2]])
        elif params[1] == DUTY_COMPLETE:
            events.append([rel(t), "complete"])
    last = None
    for event in run.get("Events", []):
        if event.get("Kind") == "sequence":
            sequence = _number_after(event["Text"], "sequence")
            if event["S"] > commence and sequence not in DONE_SEQUENCES and sequence != last:
                events.append([rel(event["S"]), "sequence", sequence])
            last = sequence

    casts = []
    for event in run.get("Events", []):
        if event.get("Kind") != "cast":
            continue
        actor = actors.get(str(event.get("Id")))
        action = _number_after(event.get("Text", ""), "action")
        if actor and actor.get("LayoutId") and action is not None and rel(event["S"]) >= 0:
            casts.append([rel(event["S"]), actor["LayoutId"], action])

    # The party's event actions' progress bars (anyone's: whoever opened the door.)
    bars: list = []
    open_bars: dict = {}
    for control in run.get("Controls", []):
        if control.get("Category") != EVENT_ACTION_BAR or control.get("Packet") != "ActorControlTarget":
            continue
        began, action = control["Params"][0], control["Params"][1]
        key = (control["Id"], action)
        if began == 1 and control["S"] >= commence:
            open_bars[key] = control["S"]
        elif began == 0 and key in open_bars:
            start = open_bars.pop(key)
            bars.append([rel(start), action, round(control["S"] - start, 2)])

    events.sort(key=lambda e: (e[0], e[1]))
    return {"duty": run.get("ContentFinderCondition", 0), "script": script, "source": source or run.get("Source", ""),
            "tolerance": DEFAULT_TOLERANCE, "npcs": npcs,
            "player": {"pos": _sparse(sorted(positions.get(you, [])), 1.5), "hp": player_health},
            "party": party, "players": players, "casts": casts, "wipes": wipes, "bars": sorted(bars), "events": events}


def save(fixture: dict, path: Path) -> Path:
    """One life, one position path a line: readable diffs, small files."""
    path.parent.mkdir(parents=True, exist_ok=True)
    lines = ["{"]
    for key in ("duty", "script", "source", "tolerance", "tolerances", "once", "ranges"):
        if key in fixture:
            lines.append(f' {json.dumps(key)}: {json.dumps(fixture[key])},')
    lines.append(' "npcs": {')
    rows = [f'  {json.dumps(layout)}: [' + ", ".join(json.dumps(life, separators=(",", ":")) for life in lives) + "]"
            for layout, lives in fixture["npcs"].items()]
    lines.append(",\n".join(rows))
    lines.append(" },")
    for key in ("player", "party", "players", "casts", "wipes", "bars"):
        if key not in fixture:
            continue
        lines.append(f' {json.dumps(key)}: {json.dumps(fixture[key], separators=(",", ":"))},')
    lines.append(' "events": [')
    lines.append(",\n".join("  " + json.dumps(event) for event in fixture["events"]))
    lines += [" ]", "}"]
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return path


# --- running the script ---------------------------------------------------------------------------------------------

@dataclass
class SimResult:
    name: str
    script: Path | None
    ok: bool = False
    problems: list[str] = field(default_factory=list)
    matched: list[tuple] = field(default_factory=list)  # (kind, value, retail time, script time)
    calls: list[tuple] = field(default_factory=list)  # (time, name, args)
    error: str | None = None


class Stage:
    """The script in a Lua runtime with a director that records what it's told."""

    def __init__(self, script: Path):
        from lupa.lua54 import LuaRuntime

        self.lua = LuaRuntime(unpack_returned_tuples=True)
        self.lua.execute("math.randomseed(1)")  # a script's random choices the same every run
        self.clock = SETUP_CLOCK
        self.delay = 0.0
        self.calls: list[tuple] = []
        self.next_actor = 900000
        self.lua.globals().LOG = self._log
        # The script's clock is the run's (a script that times things by `os.time()`: the Bowl of Embers'.)
        self.lua.globals().NOW = lambda: self.clock
        self.lua.execute("os.time = function() return math.floor(NOW() + 100000) end "
                         "os.clock = function() return NOW() + 100000 end")
        # Global.lua's helpers, as the server gives every script (it reads constants the server defines, ALL_CAPS: 0
        # while it loads.)
        self.lua.execute('setmetatable(_G, {__index = function(_, k) if type(k) == "string" '
                         'and k:match("^[A-Z][A-Z_0-9]*$") then return 0 end end})')
        self.lua.execute((repo.SCRIPTS / "Global.lua").read_text(encoding="utf-8"))
        self.lua.execute("setmetatable(_G, nil)")
        self.lua.execute((repo.SCRIPTS / "content" / "lib" / "Battle.lua").read_text(encoding="utf-8"))
        self.director = self.lua.eval("""setmetatable({}, { __index = function(_, name)
            return function(_, ...) return LOG(name, ...) end
        end })""")
        self.lua.execute(Path(script).read_text(encoding="utf-8"))
        self.g = self.lua.globals()

    def _log(self, name, *args):
        if name == "delay":
            self.delay = float(args[0] or 0) if args else 0.0
            return None
        self.calls.append((round(self.clock + self.delay, 2), name,
                           [dict(a.items()) if hasattr(a, "items") else a for a in args]))
        if name in ("spawn_bnpc", "spawn_boss", "spawn_eobj", "spawn_tethered_bnpc", "spawn_bnpc_part"):
            # The server gives back the new actor's ID.
            self.next_actor += 1
            return self.next_actor
        return None

    def call(self, hook: str, *args) -> bool:
        """A hook of the script's, if it has it; each starts with no delay, as the server's director does."""
        function = self.g[hook]
        if function is None:
            return False
        self.delay = 0.0
        function(self.director, *args)
        self.delay = 0.0
        return True


SPAWNS = ("spawn_bnpc", "spawn_boss", "spawn_bnpc_part")


class World:
    """Retail's run, as the script is shown it."""

    def __init__(self, fixture: dict, stage: Stage, scripted: set[int] | None = None):
        self.fixture = fixture
        self.stage = stage
        # The layouts the script itself brings out (`scripted_layouts`): there once it has, not when retail had them.
        self.scripted = scripted or set()
        self.lives = {int(layout): lives for layout, lives in fixture["npcs"].items()}
        # Each layout's life in play and how far the script's clock is from retail's for it (it spawned it later.)
        self.life: dict[int, dict] = {}
        self.offset: dict[int, float] = {}
        self.dead: set[tuple] = set()
        self.casts_heard: set[tuple] = set()
        self.seen_calls = 0
        self.problems: list[str] = []
        self.shift = 0.0  # a rehearsal's: retail's clock is this far behind the script's
        self.spawned: set[int] = set()  # the layouts the script has brought out
        self.rehearsing: tuple | None = None  # a rehearsal's second go: the boss's layout and its life

    def follow_calls(self) -> None:
        for t, name, args in self.stage.calls[self.seen_calls:]:
            if name == "despawn_bnpc" and args and isinstance(args[0], (int, float)) and int(args[0]) in self.life:
                # One the script brought out and takes away again is gone: it doesn't fall later.
                del self.life[int(args[0])]
                del self.offset[int(args[0])]
            if name in SPAWNS and args and isinstance(args[0], (int, float)) and int(args[0]) in self.lives:
                layout = int(args[0])
                self.spawned.add(layout)
                later = [life for life in self.lives[layout] if life["spawn"] is not None]
                if not later or layout not in self.scripted:
                    continue  # there from the start on retail: its lives follow retail's clock
                if self.rehearsing is not None and layout == self.rehearsing[0]:
                    # A rehearsal's second go: the boss the script brings back is retail's of the first go, its fight
                    # beginning where the first did.
                    self.life[layout], self.offset[layout] = self.rehearsing[1], self.shift
                    continue
                # The life nearest in time to when the script brings it out.
                life = min(later, key=lambda l: abs(l["spawn"] + self.shift - t))
                self.life[layout] = life
                self.offset[layout] = max(t, 0.0) - life["spawn"]
        self.seen_calls = len(self.stage.calls)

    def _life_now(self, layout: int, then: float) -> dict:
        """Of an NPC the script brought out: the life it's in at retail's `then`. Retail brought some back itself
        (a boss gone back to its room when the player fell, with no wipe: the Stone Vigil's last, Duty Support),
        which the script's one spawn stands for."""
        lives = self.lives[layout]
        first = lives.index(self.life[layout])
        later = [life for life in lives[first:] if life["spawn"] is not None and life["spawn"] <= then]
        return later[-1] if later else self.life[layout]

    def present(self, t: float) -> dict[int, tuple[dict, float]]:
        """The NPCs alive at t (the script's clock): each layout's life and retail's time for it."""
        out = {}
        for layout, lives in self.lives.items():
            if layout in self.life:
                then = t - self.offset[layout]
                life = self._life_now(layout, then)
                if then < life["spawn"]:
                    continue
            elif layout in self.scripted:
                continue  # the script hasn't brought it out
            else:
                # Retail's own: there from when the capturing player first saw it (or the start.)
                then = t - self.shift
                started = [l for l in lives if l["spawn"] is None or l["spawn"] <= then]
                if not started:
                    continue
                life = started[-1]
            if life["death"] is not None and then >= life["death"]:
                continue
            if life["death"] is None and life.get("gone") is not None and then > life["gone"] + 1.0:
                continue
            out[layout] = (life, then)
        return out

    def tick(self, t: float) -> None:
        """One half second: deaths and casts retail had by now, then `onUpdate`."""
        stage, lua = self.stage, self.stage.lua
        stage.clock = t
        for layout, lives in self.lives.items():
            candidates = lives[lives.index(self.life[layout]):] if layout in self.life else lives
            for life in candidates:
                key = (layout, id(life))
                if life["death"] is None or key in self.dead:
                    continue
                then = t - (self.offset[layout] if layout in self.life else self.shift)
                if layout not in self.life and layout in self.scripted:
                    continue  # the script hasn't brought it out
                if then >= life["death"]:
                    self.dead.add(key)
                    x, y, z = _position_then(life["pos"], life["death"])
                    stage.call("onActorDeath", layout, lua.table_from({"x": x, "y": y, "z": z}))
                    self.follow_calls()
        present = self.present(t)
        for cast in self.fixture.get("casts", []):
            at, layout, action = cast
            if layout not in present or (at, layout, action) in self.casts_heard:
                continue
            life, then = present[layout]
            if then >= at and at >= (life["spawn"] or -1e9):
                self.casts_heard.add((at, layout, action))
                x, y, z = _position_then(life["pos"], at)
                stage.call("onActorCast", layout, action, layout, x, y, z)
                self.follow_calls()
        health, info = {}, {}
        for layout, (life, then) in self.present(t).items():
            # Held between its points (one a percent): an NPC's health moves only when it's hit, and a boss not yet
            # pulled is whole (a script may wait for that: the Howling Eye's new Garuda after a wipe.)
            pct = _health_then(life["hp"], then) if life["hp"] else 100.0
            health[layout] = max(pct, 0.1)
            x, y, z = _position_then(life["pos"], then)
            entry = {"hp": round(pct * life["max_hp"] / 100), "max_hp": life["max_hp"], "x": x, "y": y, "z": z,
                     "id": layout}
            if life["pulled"] is not None and then >= life["pulled"]:
                entry["target"] = "player"
                entry["target_id"] = 1
            info[layout] = lua.table_from(entry)
        retail_t = t - self.shift
        player = self.fixture["player"]
        player_pct = _health_then(player["hp"], retail_t) if player.get("hp") else 100.0
        x, y, z = _position_then(player["pos"], retail_t)
        me = {"hp": player_pct, "max_hp": 100, "x": x, "y": y, "z": z, "id": 1}
        info["player"] = lua.table_from(me)
        info["player1"] = lua.table_from(me)
        for index, (key, path) in enumerate(sorted(self.fixture.get("party", {}).items()), 2):
            x, y, z = _position_then(path, retail_t)
            info[key] = lua.table_from({"hp": 100, "max_hp": 100, "x": x, "y": y, "z": z, "id": index})
            # A member who was a player (a Duty Finder run) is one to the script too, as on the server.
            if key in self.fixture.get("players", ()):
                info[f"player{index}"] = info[key]
        stage.call("onUpdate", lua.table_from(health), player_pct, lua.table_from(info))
        self.follow_calls()


class Driver:
    """A run played to the script: its wipes, its progress bars and its half seconds, in order."""

    def __init__(self, fixture: dict, script: Path, scripted: set[int] | None = None):
        self.fixture = fixture
        self.stage = Stage(script)
        self.world = World(fixture, self.stage, scripted)
        self.wipes = [list(w) for w in fixture.get("wipes", [])]
        self.bars = [list(b) for b in fixture.get("bars", [])]
        self.bars_due: list[tuple[float, int]] = []
        self.ranges = [list(r) for r in fixture.get("ranges", [])]
        self.t = 0.0
        self.stage.clock = SETUP_CLOCK
        self.stage.call("onSetup")
        self.stage.clock = 0.0
        self.stage.call("onCommence")
        self.world.follow_calls()

    def event_objects(self) -> list[int]:
        """The script's event objects: its `EOBJ_*` globals."""
        return sorted({int(value) for name, value in self.stage.g.items()
                       if isinstance(name, str) and name.startswith("EOBJ") and isinstance(value, (int, float))})

    def step(self, t: float) -> None:
        stage, world = self.stage, self.world
        retail_t = t - world.shift
        for wipe in self.wipes:
            # The script's own reset comes as everyone is sent back, 3.1 s after the wipe (`wipe.rs`.)
            if wipe[0] is not None and retail_t >= wipe[0] + 3.1:
                wipe[0] = None
                stage.clock = t
                stage.call("onPartyWipe")
                world.follow_calls()
            if wipe[0] is None and wipe[1] is not None and retail_t >= wipe[1]:
                wipe[1] = None
                stage.clock = t
                stage.call("onDutyRecommence")
                world.follow_calls()
        for bar in [b for b in self.bars if b[0] <= retail_t]:
            self.bars.remove(bar)
            stage.clock = t
            for eobj in self.event_objects():
                functional = stage.g["isGimmickFunctional"]
                if functional is not None and not functional(eobj):
                    continue
                before = len(stage.calls)
                stage.call("onGimmickAccessor", 1, eobj, None)
                if any(name == "event_action" and args and args[0] == bar[1] for _, name, args in stage.calls[before:]):
                    self.bars_due.append((t + bar[2], eobj))
                    break
            world.follow_calls()
        for entered in [r for r in self.ranges if r[0] <= retail_t]:
            self.ranges.remove(entered)
            stage.clock = t
            stage.call("onGimmickRect", entered[1])
            world.follow_calls()
        for due in [d for d in self.bars_due if d[0] <= t]:
            self.bars_due.remove(due)
            stage.clock = t
            stage.call("onEventActionCast", 1, due[1])
            world.follow_calls()
        world.tick(t)
        self.t = t

    def run(self, start: float, until: float) -> None:
        t = start
        while t < until:
            self.step(t)
            t = round(t + TICK, 2)


def script_of(fixture: dict) -> Path:
    """A fixture's script: a file of `resources/scripts/content/` (or a path, for a test's own.)"""
    return CONTENT_SCRIPTS / fixture["script"]


def script_events(calls: list[tuple]) -> list[list]:
    events = []
    for t, name, args in calls:
        if t <= SETUP_CLOCK / 2:
            continue
        if name == "set_bgm" and args:
            events.append([t, "bgm", int(args[0])])
        elif name == "objective" and args:
            events.append([t, "objective", int(args[0])])
        elif name == "set_sequence" and args and args[0] not in DONE_SEQUENCES:
            events.append([t, "sequence", int(args[0])])
        elif name == "complete_duty":
            events.append([t, "complete"])
    return events


def compare(fixture: dict, calls: list[tuple], result: SimResult) -> list[str]:
    """Retail's events against the script's: the same kind and value, in order, each within the tolerance."""
    tolerance = fixture.get("tolerance", DEFAULT_TOLERANCE)
    mine = script_events(calls)
    used = [False] * len(mine)
    problems = []
    for event in fixture["events"]:
        t, kind = event[0], event[1]
        value = event[2] if len(event) > 2 else None
        label = f"{kind} {value}" if value is not None else kind
        found = next((i for i, other in enumerate(mine) if not used[i] and other[1] == kind
                      and (other[2] if len(other) > 2 else None) == value), None)
        if found is None:
            problems.append(f"{label} at {t:.1f} s on retail: the script never did it")
            continue
        used[found] = True
        result.matched.append((kind, value, t, mine[found][0]))
        if abs(mine[found][0] - t) > fixture.get("tolerances", {}).get(label, tolerance):
            problems.append(f"{label}: {mine[found][0]:.1f} s, retail {t:.1f} s")
    for i, other in enumerate(mine):
        if not used[i]:
            label = f"{other[1]} {other[2]}" if len(other) > 2 else other[1]
            problems.append(f"{label} at {other[0]:.1f} s: retail didn't")
    return problems


def simulate(fixture: dict, name: str = "") -> SimResult:
    script = script_of(fixture)
    result = SimResult(name or f"duty {fixture['duty']}", script)
    try:
        import lupa.lua54  # noqa: F401
    except ImportError:
        result.error = "duty-sim needs lupa: pip install lupa"
        return result
    if not script.exists():
        result.error = f"no script {script}"
        return result
    end = max([e[0] for e in fixture["events"]] or [0]) + AFTER_LAST_EVENT
    driver = None
    try:
        driver = Driver(fixture, script, scripted_layouts(fixture, script, end))
        result.calls = driver.stage.calls
        driver.run(0.0, end + TICK)
    except Exception as error:  # Lua errors
        at = f" at {driver.stage.clock:.1f} s" if driver else ""
        result.error = f"script error{at}: {str(error).strip().splitlines()[0]}"
        return result
    result.problems = driver.world.problems + compare(fixture, driver.stage.calls, result)
    result.ok = not result.problems
    return result


def scripted_layouts(fixture: dict, script: Path, end: float) -> set[int]:
    """The layouts the script brings out itself (`spawn_bnpc`, `spawn_boss`): found by a first run in which every NPC
    is there when retail had it. In the real run those are there only once the script has brought them out."""
    driver = Driver(fixture, script)
    driver.run(0.0, end + TICK)
    return {layout for layout in driver.world.spawned
            if any(life["spawn"] is not None for life in driver.world.lives.get(layout, []))}


# --- a wipe in every boss fight -------------------------------------------------------------------------------------

def _fights(fixture: dict) -> list[tuple[int, float, float]]:
    """The boss fights of a run without a wipe in them: (layout, pulled, fell), the bosses being the NPCs whose fall
    retail's music or an objective followed."""
    marks = [e[0] for e in fixture["events"] if e[1] in ("objective", "complete") or (e[1] == "bgm" and e[2] == 0)]
    fights = []
    for layout, lives in fixture["npcs"].items():
        for life in lives:
            if life["pulled"] is None or life["death"] is None or life["death"] - life["pulled"] < 20:
                continue
            if not any(abs(mark - life["death"]) <= 3.0 for mark in marks):
                continue
            if any(w[0] is not None and life["pulled"] <= w[0] <= life["death"] for w in fixture.get("wipes", [])):
                continue
            fights.append((int(layout), life["pulled"], life["death"]))
    return sorted(fights, key=lambda f: f[1])


# What a script rightly tells the director once a duty, wipe or no wipe: the lines said and the shortcut opened on
# first coming to a room.
ONCE = {"party_npc_yell", "yell", "log_message", "update_shortcut", "battle_talk"}


def _calls_by_name(calls: list[tuple]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for _, name, _ in calls:
        counts[name] = counts.get(name, 0) + 1
    return counts


def rehearse_wipes(fixture: dict, name: str = "") -> SimResult:
    """In each boss fight retail got through in one go: the fight to its middle, a wipe, and the same stretch again.
    Every kind of thing the script told the director the first time it must tell it again the second (a flag
    `onPartyWipe` forgets to reset shows as a kind of call that never comes again: the boss that stops casting after
    a wipe.) Kinds, not each call: a script that picks at random, or keeps its helpers out across attempts, differs
    in the particulars. A fixture's `once` lists the kinds a script rightly does once a duty (`{"<layout>":
    ["spawn_bnpc"]}`)."""
    script = script_of(fixture)
    result = SimResult(name or f"duty {fixture['duty']}", script)
    try:
        import lupa.lua54  # noqa: F401
    except ImportError:
        result.error = "duty-sim needs lupa: pip install lupa"
        return result
    try:
        scripted = scripted_layouts(fixture, script, max([e[0] for e in fixture["events"]] or [0]))
    except Exception as error:  # Lua errors
        result.error = f"script error: {str(error).strip().splitlines()[0]}"
        return result
    for layout, pulled, fell in _fights(fixture):
        if len(fixture["npcs"][str(layout)]) > 1 or any(
                e[1] == "bgm" and e[2] and pulled + 1.0 < e[0] < fell for e in fixture["events"]):
            continue  # pulled more than once on retail: its first go isn't a whole fight
        middle = round((pulled + (fell - pulled) / 2) * 2) / 2
        lead = max(0.0, round((pulled - 5.0) * 2) / 2)
        driver = None
        try:
            driver = Driver(fixture, script, scripted)
            stage, world = driver.stage, driver.world
            driver.run(0.0, lead)
            before = len(stage.calls)
            driver.run(lead, middle)
            first = _calls_by_name(stage.calls[before:])
            # The wipe: the script's reset as everyone goes back, the recommencement 7.2 s later (`wipe.rs`), and
            # the same stretch of retail's run again from 15 s after the wipe.
            # (A script without `onPartyWipe` keeps nothing of a fight, or this shows what it kept.)
            again = middle + 15.0
            world.shift = again - lead
            if layout in world.life:
                world.rehearsing = (layout, world.life[layout])
                world.offset[layout] = world.shift
            stage.clock = middle + 3.0
            stage.call("onPartyWipe")
            world.follow_calls()
            stage.clock = middle + 10.0
            stage.call("onDutyRecommence")
            world.follow_calls()
            world.casts_heard = {cast for cast in world.casts_heard if cast[0] < lead}
            driver.bars = [list(bar) for bar in fixture.get("bars", []) if lead <= bar[0] < middle]
            before = len(stage.calls)
            driver.run(again, again + (middle - lead))
            second = _calls_by_name(stage.calls[before:])
        except Exception as error:  # Lua errors
            at = f" at {driver.stage.clock:.1f} s" if driver else ""
            result.problems.append(f"boss {layout}: script error{at}: {str(error).strip().splitlines()[0]}")
            continue
        result.matched.append(("fight", layout, sum(first.values()), sum(second.values())))
        once = ONCE | set(fixture.get("once", {}).get(str(layout), []))
        for kind, count in sorted(first.items()):
            if kind in once or second.get(kind, 0) > 0:
                continue
            result.problems.append(f"boss {layout}: {kind} ({count} times to the middle of the fight) never comes "
                                   f"again after a wipe")
    result.ok = not result.problems and result.error is None
    return result


# --- all of them ----------------------------------------------------------------------------------------------------

def all_fixtures() -> list[Path]:
    return sorted(DUTIES.glob("*.json"), key=lambda p: (int(p.stem.split("-")[0]) if p.stem.split("-")[0].isdigit()
                                                         else 0, p.stem))


KNOWN = DUTIES / "known.json"


def run_all(duties: list[int] | None = None, folder: Path | None = None) -> list[tuple[SimResult, SimResult]]:
    """Each fixture's run against retail, and its wipes rehearsed."""
    results = []
    for path in (sorted(folder.glob("*.json")) if folder else all_fixtures()):
        if path.name == KNOWN.name:
            continue
        fixture = json.loads(path.read_text(encoding="utf-8"))
        if duties and fixture["duty"] not in duties:
            continue
        results.append((simulate(fixture, path.stem), rehearse_wipes(fixture, path.stem)))
    return results


def problems_of(run: SimResult, wipes: SimResult) -> list[str]:
    return ([run.error] if run.error else []) + run.problems + ([wipes.error] if wipes.error and not run.error else []) \
        + wipes.problems


def against_known(results: list[tuple[SimResult, SimResult]], known: dict[str, list[str]]) -> tuple[list[str], int]:
    """What changed against the problems already written down (`duties/known.json`, by fixture): the new ones, and
    the ones written down that no longer happen (to be taken off the list, so it stays what's true.) With how many
    known ones stand."""
    changes, standing = [], 0
    for run, wipes in results:
        found, listed = problems_of(run, wipes), known.get(run.name, [])
        standing += sum(1 for problem in found if problem in listed)
        changes += [f"{run.name}: new: {problem}" for problem in found if problem not in listed]
        changes += [f"{run.name}: no longer happens (take it off duties/known.json): {problem}"
                    for problem in listed if problem not in found]
    return changes, standing


def load_known() -> dict[str, list[str]]:
    return json.loads(KNOWN.read_text(encoding="utf-8")) if KNOWN.exists() else {}


def save_known(results: list[tuple[SimResult, SimResult]]) -> int:
    known = {run.name: problems_of(run, wipes) for run, wipes in results if problems_of(run, wipes)}
    KNOWN.write_text(json.dumps(known, indent=1, sort_keys=True) + "\n", encoding="utf-8")
    return sum(len(problems) for problems in known.values())


def format_results(results: list[tuple[SimResult, SimResult]], verbose: bool = False,
                   known: dict[str, list[str]] | None = None) -> str:
    known = known or {}
    lines = []
    clean = 0
    for run, wipes in results:
        found, listed = problems_of(run, wipes), known.get(run.name, [])
        clean += not found
        script = run.script.name if run.script else "?"
        if run.error:
            lines.append(f"{'known' if run.error in listed else 'FAIL '} {run.name} ({script}): {run.error}")
            continue
        status = "ok   " if not found else ("known" if all(problem in listed for problem in found) else "FAIL ")
        lines.append(f"{status} {run.name} ({script}): {len(run.matched)} events matched retail, "
                     f"{len(wipes.matched)} boss fights rehearsed with a wipe"
                     + ("" if not found else f", {len(found)} problems"))
        for problem in found:
            lines.append(f"       {'(known) ' if problem in listed else ''}{problem}")
        if verbose:
            for kind, value, retail, mine in run.matched:
                label = f"{kind} {value}" if value is not None else kind
                lines.append(f"       {label:20} retail {retail:7.1f} s   script {mine:7.1f} s   ({mine - retail:+.1f})")
            for _, layout, first, second in wipes.matched:
                lines.append(f"       boss {layout}: {first} director calls to the middle of the fight, {second} again "
                             f"after a wipe")
    changes, standing = against_known(results, known)
    lines.append(f"{clean} of {len(results)} duty runs match retail and reset at a wipe; {standing} known problems"
                 + (f"; {len(changes)} changes" if changes else ""))
    return "\n".join(lines)
