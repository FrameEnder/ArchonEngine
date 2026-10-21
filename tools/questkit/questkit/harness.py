"""Runs a fork quest script outside the server, the way the world server runs it, to test it against what retail does.

What it copies from the server (servers/world/src/events/lua.rs, lua/player.rs, zone_connection/quest.rs):
* every event gets a new Lua state: the scene flags, Global.lua, the Quest sheet's parameters, the script, then
  EVENT_ID and BASE_ID;
* the script works on a copy of the player's quests: sequence and variable changes show up in the copy straight
  away, but accepting a quest doesn't until the event's tasks have run;
* the tasks a callback queued (scenes, sequence and variable changes, notices...) run in order once it returns, and
  accepting a quest starts it at sequence 255 until the script sets one;
* the client's side is played automatically: every scene returns [1, 0] (accepted / handed over / the first
  reward) with error code 0 unless the step gives other results (a list, or {"results": [...], "error": n} for a
  scene that answers through its error code, like The Gridanian Envoy's gangplanks; "yield": true sends it to
  onYield, as the client does for some scenes, where the script handles the two apart), and an event action's progress
  bar always completes. The error code is the script's `player.scene_error_code`, as on the server. A fixture made
  from Chronofoil gives every scene retail's own answer instead (its "scenes" list);
* every scene the script plays (number, flags, parameters) and what it does outside the quest (zone changes, quest
  battles, aetherytes, unlocks: "effects") is kept, for replay to compare with retail.

Needs lupa (pip install lupa) for Lua 5.4, which the server uses too.
"""

from __future__ import annotations

import copy
from pathlib import Path
from dataclasses import dataclass, field

from . import repo

try:
    from lupa.lua54 import LuaRuntime
except ImportError:  # pragma: no cover - reported by the commands that need it
    LuaRuntime = None

DEFAULT_RESULTS = [1, 0]
MAX_CALLBACKS = 64


class HarnessError(Exception):
    pass


@dataclass
class Quest:
    """A quest as the server keeps it."""

    sequence: int
    variables: list[int] = field(default_factory=lambda: [0] * 6)

    def hex(self) -> str:
        return bytes(self.variables).hex()


@dataclass
class World:
    """Everything the harness keeps between events for one player."""

    quests: dict[int, Quest] = field(default_factory=dict)
    completed: set[int] = field(default_factory=set)
    spawned: list[int] = field(default_factory=list)
    aetherytes: set[int] = field(default_factory=set)
    battle: tuple | None = None  # (quest battle, territory) while one is joined
    log: list[str] = field(default_factory=list)


class Task:
    def __init__(self, kind: str, *args):
        self.kind = kind
        self.args = args

    def __repr__(self):
        return f"{self.kind}{self.args}"


def _values(table) -> list:
    if table is None:
        return []
    try:
        return [table[i] for i in range(1, len(table) + 1)]
    except TypeError:
        return list(table.values())


def _effect(task: Task, territory: int | None = None) -> dict | None:
    """What a task does outside the quest, in a fixture's "effects" terms: a zone change (to a quest battle too), an
    aetheryte attuned, an unlock. `territory` is where the step happens (its fixture's `territory`): moving the player
    to a pop range there (`move_to_pop_range`) is retail's PrepareZoning to the same territory."""
    if task.kind == "call" and task.args and task.args[0] == "move_to_pop_range" and territory is not None:
        return {"zone": int(territory)}
    if task.kind == "join_battle":
        return {"zone": int(task.args[1])}
    if task.kind == "aetheryte":
        return {"aetheryte": int(task.args[0])}
    if task.kind == "call" and len(task.args) >= 2:
        name, first = task.args[0], task.args[1]
        if name in ("change_territory", "change_territory_pop_range"):
            return {"zone": int(first)}
        if name == "unlock":
            return {"unlock": int(first)}
    return None


class FakePlayer:
    """What a script's `player` is: reads come from the copy, writes become tasks."""

    def __init__(self, harness: "Harness"):
        self._h = harness
        self.position = harness.lua.table_from({"x": 0.0, "y": 0.0, "z": 0.0})
        self.rotation = 0.0
        self.scene_error_code = 0

    # Reads, from the copy -----------------------------------------------------------------------------------------
    def get_quest_sequence(self, quest):
        q = self._h.copy.quests.get(self._h.quest_number(quest))
        return None if q is None else q.sequence

    def get_quest_var(self, quest, index):
        q = self._h.copy.quests.get(self._h.quest_number(quest))
        if q is None or not 0 <= index < 6:
            return 0
        return q.variables[index]

    def has_quest(self, quest):
        number = self._h.quest_number(quest)
        return number in self._h.copy.quests or number in self._h.copy.completed

    def has_aetheryte(self, aetheryte):
        return aetheryte in self._h.copy.aetherytes

    # Writes, as tasks ---------------------------------------------------------------------------------------------
    def set_quest_var(self, quest, index, value):
        if not 0 <= value <= 255:
            raise HarnessError(f"set_quest_var({index}, {value}): a variable is one byte")
        q = self._h.copy.quests.get(self._h.quest_number(quest))
        if q is not None and 0 <= index < 6:
            q.variables[index] = value
        self._h.tasks.append(Task("var", self._h.quest_number(quest), index, value))

    def quest_sequence(self, quest, sequence):
        q = self._h.copy.quests.get(self._h.quest_number(quest))
        if q is not None:
            q.sequence = sequence
        self._h.tasks.append(Task("sequence", self._h.quest_number(quest), sequence))

    def accept_quest(self, quest, sequence=None):
        self._h.tasks.append(Task("accept", self._h.quest_number(quest)))

    def finish_quest(self, quest, item=None):
        self._h.tasks.append(Task("finish", self._h.quest_number(quest), item or 0))

    def cancel_quest(self, quest):
        self._h.tasks.append(Task("cancel", self._h.quest_number(quest)))

    def play_scene(self, scene, flags=0, params=None):
        self._h.tasks.append(Task("scene", int(scene), int(flags), _values(params)))

    def finish_event(self):
        self._h.tasks.append(Task("finish_event"))

    def event_action(self, action):
        self._h.tasks.append(Task("event_action", action))

    def event_notice(self, handler, notice, args=None):
        values = _values(args)
        if len(values) > 4:
            raise HarnessError(f"event_notice with {len(values)} arguments: the server refuses more than 4")
        if int(handler) != repo.QUEST_HANDLER_BASE + self._h.quest:
            # Another event's notice (the aetheryte's, while attuning inside a quest): not the quest's progress.
            self._h.tasks.append(Task("other_notice", int(handler), int(notice), [int(v) for v in values]))
            return
        self._h.tasks.append(Task("notice", int(notice), [int(v) for v in values]))

    def spawn_quest_bnpc(self, layout):
        self._h.tasks.append(Task("spawn", layout))

    def join_quest_battle(self, battle, territory):
        self._h.tasks.append(Task("join_battle", battle, territory))

    def unlock_aetheryte(self, on, aetheryte):
        self._h.tasks.append(Task("aetheryte", aetheryte))

    def __getattr__(self, name):
        if name.startswith("_"):
            raise AttributeError(name)

        # Anything else the server offers: recorded, does nothing here.
        def call(*args):
            if args and args[0] is self:
                args = args[1:]
            self._h.tasks.append(Task("call", name, *[_values(a) if hasattr(a, "values") else a for a in args]))
            return None

        return call


class Harness:
    """One player and one quest script."""

    def __init__(self, script_path, quest: int, params: dict[str, int], world: World | None = None):
        if LuaRuntime is None:
            raise HarnessError("questkit's replay needs lupa: pip install lupa")
        self.script_path = script_path
        self.quest = quest
        self.params = params
        self.world = world or World()
        self._flags = repo.scene_flags()
        self._global = (repo.SCRIPTS / "Global.lua").read_text()
        self._source = Path(script_path).read_text()
        self.lua = None
        self.copy = World()
        self.tasks: list[Task] = []
        self.territory: int | None = None  # where the current step happens, when its fixture says

    def quest_number(self, value) -> int:
        value = int(value)
        return value - repo.QUEST_HANDLER_BASE if value >= repo.QUEST_HANDLER_BASE else value

    # One event ------------------------------------------------------------------------------------------------------
    def _new_state(self, base_id: int | None):
        lua = LuaRuntime(unpack_returned_tuples=True)
        globals_ = lua.globals()
        for name, value in self._flags.items():
            globals_[name] = value
        lua.execute(self._global)
        for name, value in self.params.items():
            globals_[name] = value
        lua.execute(self._source)
        globals_.EVENT_ID = repo.QUEST_HANDLER_BASE + self.quest
        if base_id is not None:
            globals_.BASE_ID = base_id
        self.lua = lua
        self.player = FakePlayer(self)
        return globals_

    def _call(self, name: str, *args) -> list[Task]:
        """Calls one of the script's callbacks with a fresh copy of the player, then runs what it queued."""
        function = self.lua.globals()[name]
        if function is None:
            raise HarnessError(f"the script has no {name}")
        self.copy = copy.deepcopy(self.world)
        self.tasks = []
        function(*args)
        tasks = self.tasks
        self._apply(tasks)
        return tasks

    def _apply(self, tasks: list[Task]):
        world = self.world
        for task in tasks:
            kind = task.kind
            if kind == "accept":
                world.quests[task.args[0]] = Quest(0xFF)
                world.log.append(f"accept {task.args[0]}")
            elif kind == "sequence":
                quest = world.quests.get(task.args[0])
                if quest is None:
                    raise HarnessError(f"quest_sequence({task.args[1]}) on a quest that isn't active")
                quest.sequence = task.args[1]
            elif kind == "var":
                quest = world.quests.get(task.args[0])
                if quest is None:
                    raise HarnessError(f"set_quest_var on a quest that isn't active")
                quest.variables[task.args[1]] = task.args[2]
            elif kind == "finish":
                world.quests.pop(task.args[0], None)
                world.completed.add(task.args[0])
                world.log.append(f"finish {task.args[0]} item {task.args[1]}")
            elif kind == "cancel":
                world.quests.pop(task.args[0], None)
            elif kind == "spawn":
                world.spawned.append(task.args[0])
            elif kind == "aetheryte":
                world.aetherytes.add(task.args[0])
            elif kind == "join_battle":
                world.battle = (task.args[0], task.args[1])

    def _drive(self, first: list[Task], results: dict, trace: list) -> dict:
        """Plays the client's side of an event until the script finishes it."""
        outcome = {"notices": [], "scenes": [], "played": [], "effects": [], "finished": False, "calls": [],
                   "spawned": []}
        pending = list(first)
        callbacks = 0
        # A fixture's "scenes" (what retail played, in order): each scene takes the client's answer from the first
        # unused entry with its number.
        unused = list(results) if isinstance(results, list) else None
        nested = False
        while True:
            scene = None
            action = False
            for task in pending:
                trace.append(repr(task))
                if task.kind == "notice":
                    outcome["notices"].append([task.args[0], task.args[1]])
                elif task.kind == "scene":
                    scene = task.args[0]
                    outcome["scenes"].append(scene)
                    # A scene after start_event is the nested event's (the cities' openings), not the quest's.
                    if not nested:
                        outcome["played"].append({"scene": scene, "flags": task.args[1], "params": task.args[2]})
                elif task.kind == "event_action":
                    action = True
                elif task.kind == "finish_event":
                    outcome["finished"] = True
                elif task.kind == "spawn":
                    outcome["spawned"].append(task.args[0])
                elif task.kind in ("call", "join_battle"):
                    outcome["calls"].append(task.kind if task.kind != "call" else task.args[0])
                    nested = nested or (task.kind == "call" and task.args[0] == "start_event")
                effect = _effect(task, self.territory)
                if effect is not None:
                    outcome["effects"].append(effect)
            if outcome["finished"] and (scene is not None or action):
                raise HarnessError("the script played something after finishing the event")
            callbacks += 1
            if callbacks > MAX_CALLBACKS:
                raise HarnessError("the event never ends (more than 64 callbacks)")
            if scene is not None:
                if unused is not None:
                    entry = next((e for e in unused if e.get("scene") == scene), None)
                    if entry is not None:
                        unused.remove(entry)
                    values = entry if entry is not None else DEFAULT_RESULTS
                else:
                    values = results.get(str(scene), results.get(scene, DEFAULT_RESULTS))
                error = 0
                yielded = False
                if isinstance(values, dict):
                    error = values.get("error", 0)
                    yielded = values.get("yield", False)
                    values = values.get("results", DEFAULT_RESULTS)
                self.player.scene_error_code = error
                table = self.lua.table(*values)
                if self.lua.globals().onReturn is not None and not yielded:
                    pending = self._call("onReturn", scene, table, self.player)
                else:
                    pending = self._call("onYield", scene, 0, table, self.player)
                continue
            if action:
                pending = self._call("onEventActionCast", self.player)
                continue
            return outcome

    # Steps ------------------------------------------------------------------------------------------------------------
    def talk(self, base_id: int, results: dict | None = None, trace: list | None = None,
             layout: int | None = None) -> dict:
        """Talking to an NPC or object. `layout` is its layout ID, the script's `target.object_id` (the cities'
        opening quests tell their NPCs apart by it.)"""
        self._new_state(base_id)
        trace = trace if trace is not None else []
        target = self.lua.table_from({"object_id": layout}) if layout is not None else None
        first = self._call("onTalk", target, self.player)
        return self._drive(first, results or {}, trace)

    def event_item(self, item_id: int, base_id: int, results: dict | None = None, trace: list | None = None,
                   layout: int | None = None) -> dict:
        """A key item used on an NPC or object (the server's EventItem event, `onEventItem(item, target, player)`)."""
        self._new_state(base_id)
        trace = trace if trace is not None else []
        if self.lua.globals().onEventItem is None:
            raise HarnessError("the script has no onEventItem for the key item's use")
        target = self.lua.table_from({"object_id": layout}) if layout is not None else None
        first = self._call("onEventItem", item_id, target, self.player)
        return self._drive(first, results or {}, trace)

    def enter_range(self, arg: int, results: dict | None = None, trace: list | None = None) -> dict:
        self._new_state(None)
        trace = trace if trace is not None else []
        first = self._call("onEnterTrigger", self.player, arg)
        return self._drive(first, results or {}, trace)

    def enter_territory(self, results: dict | None = None, trace: list | None = None) -> dict:
        self._new_state(None)
        trace = trace if trace is not None else []
        first = self._call("onEnterTerritory", self.player)
        return self._drive(first, results or {}, trace)

    def kill(self, base_id: int, layout_id: int, trace: list | None = None) -> dict:
        self._new_state(None)
        trace = trace if trace is not None else []
        if self.lua.globals().onBNpcKill is None:
            return {"notices": [], "scenes": [], "played": [], "effects": [], "finished": True, "calls": [],
                    "spawned": [], "no_handler": True}
        first = self._call("onBNpcKill", base_id, layout_id, self.player)
        outcome = self._drive(first, {}, trace)
        outcome["finished"] = True  # kills aren't events
        return outcome

    def battle_won(self, sequence: int) -> dict:
        """What a quest battle's director does when it's won: the quest moves to its sequence."""
        if self.world.battle is None:
            raise HarnessError("battle_won without joining a quest battle first")
        quest = self.world.quests.get(self.quest)
        if quest is None:
            raise HarnessError("battle_won for a quest that isn't active")
        quest.sequence = sequence
        self.world.battle = None
        return {"notices": [], "scenes": [], "played": [], "effects": [], "finished": True, "calls": [],
                "spawned": []}

    def state(self) -> dict:
        quest = self.world.quests.get(self.quest)
        if quest is None:
            return {"completed": self.quest in self.world.completed}
        return {"sequence": quest.sequence, "vars": quest.hex()}
