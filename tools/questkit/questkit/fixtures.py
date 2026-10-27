"""Replay fixtures: what a quest did on retail, step by step, as numbers only.

A fixture (tools/questkit/fixtures/<quest>.json) is made from a Kawari Recorder export and the quest's dump
(kawari-questdump), and holds no game text, so it can be committed:

    {
      "quest": 387, "script": "SubFst073_00387",
      "source": "...where the recording came from...",
      "params": {"ACTOR0": 1000503, ...},          # the Quest sheet's Lua parameters (numbers)
      "steps": [
        {"talk": 1000503, "expect": {"sequence": 1, "vars": "000000000000", "notices": []}},
        {"kill": [12, 3838969], "expect": {...}},
        {"range": 3842570},                         # walking into an event range
        {"emote": [12, 1002640], "layout": 3929260},  # an emote (its Emote row) on an NPC (base ID)
        {"action": [120, 1003001], "layout": 3854494},  # an action (its Action row) used on an NPC (base ID)
        {"enter_territory": 148, "expect": {...}},  # zoning in (quests that play a scene on arrival)
        {"battle_won": 4, "expect": {...}},         # a quest battle's director winning it
        {"duty_cleared": 4, "expect": {...}},       # leaving a cleared duty (its InstanceContent ID)
        {"talk": 1000470, "results": {"24": [1, 0]}, "expect": {"completed": true}},
        {"talk": 1003987, "layout": 3969639, "expect": {...}}  # its layout ID too: the script's target.object_id
      ]
    }

`expect` is the quest's state after the step: its sequence and its six variable bytes (UI8A..UI8F) as hex, or
"completed". `notices` is the notices the step sends, as [notice, [args]]; it's left out (not checked) where the
recording didn't capture packets. A step without `expect` only has to run without errors.

`results` sets what the client answers to a scene (every scene answers [1, 0] otherwise): a list, or
{"results": [...], "error": n} for a scene whose answer is its error code (50 is "No" in The Gridanian Envoy's
boarding yes/nos; retail's "Yes" there is error 0 with results [0, 0]). `"yield": true` in it sends the scene to the
script's `onYield` rather than `onReturn`, for scripts that handle the two apart (the cities' opening quests): which
one the client sends for each scene is in a Chronofoil capture (EventYieldHandler or EventReturnHandler).

Recorder 2.0 and later record who the player last targeted, which says what each step was. Older recordings don't,
so their steps come out as {"todo": ...} placeholders to fill in by hand.

**From Chronofoil** (`questkit fixture --chronofoil`, `build_from_chronofoil`) a fixture holds more, and replay checks
all of it:

    {"talk": 1003988, "layout": 3969632, "what": "ACTOR0",
     "scenes": [{"scene": 0, "flags": "0x2000", "results": [1, 0, 0], "yield": true},
                {"scene": 50, "flags": "0xf8402efb", "results": [0, 0]}],
     "effects": [{"zone": 130}],
     "expect": {"sequence": 1, "vars": "000010000000", "notices": []}}

`scenes` is every scene retail played in the step, in order, with its flags (and parameters when it had any) and the
client's own answer (its results, "yield" when it came back as EventYieldHandler, "error" for a return's error code);
the script has to play the same scenes with the same flags, and each gets retail's answer. `effects` is what the
server did outside the quest while the event was open: zone changes (`change_territory`, a pop range, a quest
battle), aetheryte attunements (`unlock_aetheryte`) and unlocks (`unlock`); a step without it must do none.
`"expect": {"accepted": false}` is an offer that was turned down.
"""

from __future__ import annotations

import json
from pathlib import Path

from . import repo
from .recording import Recordings

MERGE_SECONDS = 3.0
EVENT_ENTER_TERRITORY = 15
EVENT_WITHIN_RANGE = 10
EVENT_EMOTE = 2
AFTER_FINISH = 1.0  # seconds after its event finished that a quest update still belongs to the step
ACTIONS = ("talk", "emote", "kill", "range", "enter_territory", "battle_won", "duty_cleared")
NOTICE_LEAD = 0.5  # notices arrive just before the quest change they go with


def load_dump(dumps: Path, quest: int) -> dict | None:
    """A quest's quest.json from a kawari-questdump output folder (quest-dumps/<script>/quest.json)."""
    dumps = Path(dumps)
    for path in sorted(dumps.glob("*/quest.json")):
        try:
            data = json.loads(path.read_text())
        except json.JSONDecodeError:
            continue
        if data.get("quest_id") == quest:
            return data
    return None


def numeric_params(dump: dict) -> dict[str, int]:
    params = {}
    for param in dump.get("params", []):
        value = param.get("value")
        if isinstance(value, int) and isinstance(param.get("name"), str):
            params[param["name"]] = value
    return params


def _group(events):
    """Recorded changes that came from one action: the same target, close together."""
    groups = []
    for event in events:
        if groups:
            last = groups[-1][-1]
            same_target = event.target_layout and event.target_layout == last.target_layout
            if same_target and event.time - last.time <= MERGE_SECONDS and event.kind != "completed":
                groups[-1].append(event)
                continue
        groups.append([event])
    return groups


def build(quest: int, recordings: Recordings, dump: dict, source: str = "") -> dict:
    events = recordings.quest_events(quest)
    if not events:
        raise ValueError(f"quest {quest} isn't in the recording")
    notices, covered = recordings.notices(quest)
    params = numeric_params(dump)
    groups = _group(events)

    # Each notice goes with the first change that comes after it (or with it.)
    assigned: list[list] = [[] for _ in groups]
    for notice in notices:
        for index, group in enumerate(groups):
            if group[-1].time >= notice.time - NOTICE_LEAD:
                assigned[index].append(notice.as_list())
                break

    battle = "QUESTBATTLE0" in params
    by_value = {}
    for name, value in params.items():
        by_value.setdefault(value, []).append(name)

    steps = []
    for index, group in enumerate(groups):
        last = group[-1]
        first = group[0]
        step: dict = {}
        kind = first.target_kind
        fresh = first.target_seconds_ago is not None and first.target_seconds_ago <= 10 and first.target_base
        territory = last.raw.get("TerritoryId")
        previous = groups[index - 1][-1].raw.get("TerritoryId") if index > 0 else None
        if battle and territory is not None and territory == params.get("TERRITORYTYPE0"):
            # Changed inside the quest battle: its director won it (whoever the player was hitting.)
            step["battle_won"] = last.sequence
        elif kind in ("EventNpc", "EventObj", "Aetheryte") and fresh:
            step["talk"] = first.target_base
            if first.target_layout:
                step["layout"] = first.target_layout
        elif kind == "BattleNpc" and fresh:
            step["kill"] = [first.target_base, first.target_layout]
        elif last.kind == "sequence" and previous is not None and territory != previous:
            # Changed on arriving somewhere else (e.g. back from a quest battle, a scene on entering the zone.)
            step["enter_territory"] = territory
        else:
            step["todo"] = (f"{last.kind} in territory {last.raw.get('TerritoryId')}: the recording doesn't say "
                            f"what the player did (talk, kill, range, enter_territory or battle_won)")
        names = by_value.get(step.get("talk")) or by_value.get((step.get("kill") or [None])[0])
        if names:
            step["what"] = "/".join(sorted(names))
        expect: dict = {}
        if last.kind == "completed":
            expect["completed"] = True
        else:
            expect["sequence"] = last.sequence
            expect["vars"] = last.variables.hex()
        if covered:
            expect["notices"] = assigned[index]
        step["expect"] = expect
        steps.append(step)

    return {
        "quest": quest,
        "script": dump.get("script", ""),
        "source": source,
        "params": params,
        "steps": steps,
    }


def _scene_entry(scene: dict, answer: dict | None) -> dict:
    """One scene as retail played it and the client answered it (a fixture step's "scenes")."""
    entry = {"scene": scene["scene"], "flags": f"0x{scene['flags']:x}"}
    if scene.get("params"):
        entry["params"] = scene["params"]
    if answer is not None:
        entry["results"] = answer["results"]
        if answer["yielded"]:
            entry["yield"] = True
        elif answer["code"]:
            entry["error"] = answer["code"]
    return entry


def _state(step: dict) -> dict:
    return {k: v for k, v in step.get("expect", {}).items() if k != "notices"}


def _fill_from(steps: list[dict], previous: list[dict]):
    """Steps the capture can't name (an object whose base ID it lacks, a kill, a quest battle won elsewhere) take
    their action from the existing fixture's step that left the quest in the same state, in order. A previous step
    without an expectation (agreeing to a quest battle, searching something that spawns enemies) matches a talk by
    its place in the order."""
    def state_matches(match, step, loose):
        return ("expect" not in match and loose) or ("expect" in match and _state(match) == _state(step))

    used = -1
    for step in steps:
        actor = step.get("_actor")
        for i in range(used + 1, len(previous)):
            match = previous[i]
            if "todo" not in step:
                # Already named: keep the alignment, and what retail spawned (not read from the capture.)
                ok = all(match.get(k) == step.get(k) for k in ACTIONS if k in step) and \
                    any(k in match for k in ACTIONS if k in step) and state_matches(match, step, True)
            elif actor is not None:
                ok = "talk" in match and state_matches(match, step, True)
            else:
                ok = any(k in match for k in ("kill", "battle_won", "enter_territory", "range")) and \
                    state_matches(match, step, False)
            if not ok:
                continue
            used = i
            if "todo" in step:
                del step["todo"]
                for key in ACTIONS + ("layout",):
                    if key in match:
                        step[key] = match[key]
                if actor is not None:
                    step["layout"] = actor
            if "spawns" in match:
                step["spawns"] = match["spawns"]
            break
    # The same object talked to again (a progress bar broken off and tried again) has the same base.
    known = {s["layout"]: s["talk"] for s in steps if "talk" in s and s.get("layout")}
    for step in steps:
        if "todo" in step and step.get("_actor") in known:
            del step["todo"]
            step["talk"], step["layout"] = known[step["_actor"]], step["_actor"]


def build_from_chronofoil(quest: int, events: list, dump: dict, bases: dict[int, int], source: str = "",
                          previous: dict | None = None) -> dict:
    """A fixture from a quest's events in Chronofoil captures (chronofoil_quest.quest_events): every talk (including
    the ones that change nothing, like agreeing to a quest battle), each with the scenes retail played (number, flags,
    parameters, and how the client answered: results, yield or return, error code) and what else the server did while
    the event was open (`effects`: zone changes, aetherytes, unlocks). `bases` maps the talked-to actor (a layout ID
    for the event NPCs and objects the client spawns itself) to its base ID; `previous`, the quest's existing fixture,
    fills in what the capture can't say (base IDs it lacks, kills.)"""
    params = numeric_params(dump)
    battle_zone = params.get("TERRITORYTYPE0") if "QUESTBATTLE0" in params else None
    previous_steps = (previous or {}).get("steps", [])
    known = dict(bases)
    for step in previous_steps:
        if "talk" in step and step.get("layout"):
            known.setdefault(step["layout"], step["talk"])

    steps: list[dict] = []
    state: dict = {}
    step: dict | None = None
    pending: dict | None = None  # the scene waiting for the client's answer
    loose: list = []  # notices sent outside an event

    def close():
        nonlocal step, pending
        if step is None:
            return
        if pending is not None:
            # The client never answered: the player broke off (moved during a progress bar). Nothing happened, so
            # there's nothing to replay.
            pending = None
            if not step["notices"] and not step.get("_changed"):
                step = None
                return
        if state.get("completed"):
            # Completing a quest unlocks what its Quest sheet row says (the server's finish_quest does that, not the
            # script), so those aren't the script's to send.
            step["effects"] = [e for e in step["effects"] if "unlock" not in e]
        for key in ("scenes", "effects"):
            if not step[key]:
                del step[key]
        step["expect"] = dict(state or {"accepted": False}, notices=step.pop("notices"))
        steps.append(step)
        step = None

    def update(event):
        if event.kind == "update":
            state.clear()
            state.update({"sequence": event.data["sequence"], "vars": event.data["vars"]})
        elif event.kind == "finish_quest":
            state.clear()
            state["completed"] = True

    for event in events:
        kind, data = event.kind, event.data
        if kind == "talk":
            close()
            actor = data["actor"]
            step = {"scenes": [], "effects": [], "notices": [], "_zone": event.zone}
            if actor in known:
                step["talk"] = known[actor]
                step["layout"] = actor
            else:
                step["todo"] = f"talked to actor {actor} in territory {event.zone}: its base ID isn't known"
                step["_actor"] = actor
            step["_open"] = True
        elif kind == "start":
            if step is not None and step.get("_open") and data["type"] == 1:
                continue  # the talk's own EventStart
            close()
            step = {"scenes": [], "effects": [], "notices": [], "_open": True, "_zone": event.zone}
            if data["type"] == EVENT_ENTER_TERRITORY:
                step["enter_territory"] = event.zone
            elif data["type"] == EVENT_WITHIN_RANGE:
                step["range"] = data["arg"]
            elif data["type"] == EVENT_EMOTE:
                # An emote on an NPC: the event's argument is the emote, its actor the NPC.
                actor = data["actor"]
                step["layout"] = actor
                if actor in known:
                    step["emote"] = [data["arg"], known[actor]]
                else:
                    step["todo"] = f"emote {data['arg']} on actor {actor} in territory {event.zone}: its base ID isn't known"
                    step["_actor"] = actor
            else:
                step["todo"] = f"event type {data['type']} (argument {data['arg']}) in territory {event.zone}"
        elif kind == "scene" and step is not None:
            if pending is not None:
                step["scenes"].append(_scene_entry(pending, None))
            pending = data
        elif kind == "answer" and step is not None:
            if pending is not None and pending["scene"] == data["scene"]:
                step["scenes"].append(_scene_entry(pending, data))
                pending = None
        elif kind == "notice":
            if step is not None and step.get("_open"):
                step["notices"].append([data["notice"], data["args"]])
            else:
                loose.append([data["notice"], data["args"]])  # e.g. a kill's, before its update
        elif kind in ("zone", "aetheryte", "unlock", "content") and step is not None:
            value = data.get("territory", data.get("aetheryte", data.get("unlock", data.get("content"))))
            step["effects"].append({kind: value})
            if kind == "zone" and value == step.get("_zone"):
                # Moved within the territory the step happens in (PrepareZoning to the same one): replay needs to
                # know where that is (a script's move_to_pop_range.)
                step["territory"] = value
        elif kind in ("update", "finish_quest"):
            # Part of the step while its event is open, or just after it finished (a zone change, then the update.)
            if step is not None and (step.get("_open") or event.t - step.get("_finished", -1e9) <= AFTER_FINISH):
                update(event)
                step["_changed"] = True
            else:
                # Changed outside any event: a quest battle won, or a kill.
                close()
                update(event)
                new = {"expect": dict(state, notices=list(loose))}
                loose.clear()
                if battle_zone is not None and event.zone == battle_zone:
                    new = {"battle_won": state.get("sequence"), **new}
                else:
                    new = {"todo": f"changed in territory {event.zone} outside an event (a kill?)", **new}
                steps.append(new)
        elif kind == "finish" and step is not None:
            step["_open"] = False
            step["_finished"] = event.t
            # Zone changes right after the event belong to it; the step closes at the next event.
    close()
    _fill_from(steps, previous_steps)
    for s in steps:
        for key in ("_open", "_changed", "_actor", "_finished", "_zone"):
            s.pop(key, None)
        what = None
        for name, value in params.items():
            if value == s.get("talk") and (name.startswith(("ACTOR", "EOBJECT", "AETHERYTE"))):
                what = name if what is None else f"{what}/{name}"
        if what:
            s["what"] = what
    # The order a step's keys read in.
    order = ["talk", "event_item", "emote", "action", "layout", "territory", "kill", "range", "enter_territory", "battle_won", "duty_cleared", "todo", "what", "scenes", "effects",
             "spawns", "expect"]
    steps = [{k: s[k] for k in order if k in s} | {k: v for k, v in s.items() if k not in order} for s in steps]
    return {
        "quest": quest,
        "script": dump.get("script", ""),
        "source": source,
        "params": params,
        "steps": steps,
    }


def fixture_path(quest: int) -> Path:
    return repo.FIXTURES / f"{quest}.json"


def save(fixture: dict, path: Path | None = None) -> Path:
    path = path or fixture_path(fixture["quest"])
    path.parent.mkdir(parents=True, exist_ok=True)
    text = dumps(fixture)
    path.write_text(text + "\n")
    return path


def dumps(fixture: dict) -> str:
    """JSON with lists of numbers ([notice, [args]], kills) kept on one line, so a fixture reads step by step."""
    import re

    text = json.dumps(fixture, indent=2)
    tokens: list[str] = []

    def keep(match):
        tokens.append("[" + ", ".join(part.strip() for part in match.group(1).split(",") if part.strip()) + "]")
        return f"\x00{len(tokens) - 1}\x00"

    while True:
        new = re.sub(r"\[([^\[\]{}\x00]*(?:\x00\d+\x00[^\[\]{}\x00]*)*)\]", keep, text)
        if new == text:
            break
        text = new
    while "\x00" in text:
        text = re.sub(r"\x00(\d+)\x00", lambda m: tokens[int(m.group(1))], text)

    # Small objects (a scene, an effect, an expect) on one line too.
    def small(match):
        body = " ".join(part.strip() for part in match.group(1).splitlines() if part.strip())
        line = "{" + body + "}"
        return line if len(line) <= 110 else match.group(0)

    return re.sub(r"\{([^{}\[\]]*(?:\[[^\n]*?\][^{}\[\]]*)*)\}", small, text)


def load(path: Path) -> dict:
    return json.loads(Path(path).read_text())


def all_fixtures() -> list[Path]:
    return sorted(repo.FIXTURES.glob("*.json"), key=lambda p: int(p.stem) if p.stem.isdigit() else 0)
