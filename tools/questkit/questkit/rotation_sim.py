"""A party bot's rotation against a training dummy, by running the real scripts (docs/src/fork/capture-pipeline.md,
stage A.3): the job's rotation (`resources/scripts/rotations/`) chooses, as the server asks it every half second, and
each action it uses is its own script (`resources/scripts/actions/`) run against a mock EffectsBuilder, whose calls
make the statuses, gauges and MP the next choice sees.

What it has of the server: the 2.5 s global cooldown and cast times, recasts, statuses on oneself and on the target
with their durations and stacks, the gauge counters with their ranges (read from `effects_builder.rs`), combos by
combo steps (`RotationState::last_combo`), the instant casts (`action.rs`'s INSTANT_CASTS), the gauge's tick (Life of
the Dragon, Polyglot, Lilies), MP's tick. What it hasn't: damage (every hit is 1), enemies that die, movement, anything
the target does. So it answers "does the rotation use what it should, in what order, without a script failing", not
"how much damage".

`check()` is the CI test: every rotation at every level band, no script error and the bot never idle.
"""

from __future__ import annotations

import json
import random
import re
from collections import Counter
from pathlib import Path

from . import repo

try:
    from lupa import lua54
except ImportError:  # pragma: no cover
    lua54 = None

JOB_ACTIONS = Path(__file__).parent / "data" / "public" / "job_actions.json"
# ClassJob rows by Boss Mod's file, and each class's job.
JOBS = {"PLD": 19, "MNK": 20, "WAR": 21, "DRG": 22, "BRD": 23, "WHM": 24, "BLM": 25, "SMN": 27, "SCH": 28, "NIN": 30,
        "MCH": 31, "DRK": 32, "AST": 33, "SAM": 34, "RDM": 35, "GNB": 37, "DNC": 38, "RPR": 39, "SGE": 40, "VPR": 41,
        "PCT": 42}
CLASS_JOB = {1: 19, 2: 20, 3: 21, 4: 22, 5: 23, 6: 24, 7: 25, 26: 27, 29: 30}
# 7.x actions Boss Mod's list hasn't got: (job, id, level, on the global cooldown, recast).
EXTRA = [(38, 36983, 92, True, 2.5), (38, 36984, 96, True, 30.0), (38, 36985, 100, True, 2.5)]
# action.rs's INSTANT_CASTS: a status that makes casts instant, for these actions (none: any).
INSTANT = {1249: (), 1238: (7505, 7507, 7509, 16526, 25855, 25856), 2845: (24386,), 3679: (34688,), 841: (), 167: ()}
# MP costs the sheet has and the scripts don't set (the job guide): the Dark Knight's 3000s, the Black Mage's spells.
DRK_COSTS = {16466: 3000, 16467: 3000, 16469: 3000, 16470: 3000, 7393: 3000}
BLM_COSTS = {142: 400, 154: 800, 159: 1000, 25793: 800, 144: 200, 153: 200, 7447: 200, 7420: 200, 25797: 1600,
             3576: 800}
BLM, DRG, MNK, WHM = (7, 25), (4, 22), (2, 20), (6, 24)

MOCK = """
EffectsBuilder = function()
    local e = { ops = {} }
    return setmetatable(e, { __index = function(_, name)
        return function(self, ...) table.insert(self.ops, { name, ... }) end
    end })
end
function magicalDamage() return 1 end
function physicalDamage() return 1 end
function healAmount() return 1 end
function whmHeal() return 1 end
"""

PLAYER = """
__p.gauge = function(self, n) return __g[n] or 0 end
__p.get_effect = function(self, id) if __st[id] ~= nil then return { param = __st[id] } end return nil end
__p.effect_time_left = function(self, id) if __st[id] ~= nil then return 10 end return nil end
__p.has_trait = function() return true end
setmetatable(__p, { __index = function(_, k) return function() return 1 end end })
setmetatable(__p.parameters, { __index = function(_, k) return function() return 1 end end })
"""


def action_scripts() -> dict[int, Path]:
    found: dict[int, Path] = {}
    for path in sorted((repo.SCRIPTS / "actions").rglob("*.lua")):
        match = re.search(r"_(\d+)$", path.stem)
        if match:
            found.setdefault(int(match.group(1)), path)
    return found


def gauge_ranges() -> dict[str, tuple[int, int]]:
    """Each gauge counter's range, as `effects_builder.rs` clamps it."""
    text = (repo.ROOT / "servers" / "world" / "src" / "lua" / "effects_builder.rs").read_text(encoding="utf-8")
    names = dict(re.findall(r'GaugeField::(\w+) => "(\w+)"', text))
    ranges = {names[field]: (int(low), int(high))
              for field, low, high in re.findall(r"GaugeField::(\w+) => \((-?\d+), (-?\d+)\)", text) if field in names}
    ranges.update(aetherflow=(0, 3), lilies=(0, 3), blood_lily=(0, 3))
    return ranges


def actions_of(classjob: int, level: int, scripts: dict[int, Path], jobs: dict | None = None) -> dict[int, dict]:
    """The job's actions at this level that have a script."""
    jobs = jobs or json.loads(JOB_ACTIONS.read_text(encoding="utf-8"))["jobs"]
    job = CLASS_JOB.get(classjob, classjob)
    name = next((n for n, row in JOBS.items() if row == job), None)
    out = {a["id"]: a for a in jobs.get(name, []) if a["lv"] <= level and a["id"] in scripts}
    for row, action, learnt, gcd, recast in EXTRA:
        if row == job and learnt <= level and action in scripts:
            out[action] = {"id": action, "lv": learnt, "gcd": gcd, "recast": recast, "cast": 0.0, "friendly": False}
    if job == 28:  # the Scholar keeps the Arcanist's
        for action, entry in actions_of(26, min(level, 30), scripts, jobs).items():
            out.setdefault(action, entry)
    return out


def rotations() -> dict[int, Path]:
    """Each ClassJob's rotation file (`classjobs = { ... }` in it)."""
    found = {}
    for path in sorted((repo.SCRIPTS / "rotations").glob("*.lua")):
        listed = re.search(r"classjobs\s*=\s*\{([^}]*)\}", path.read_text(encoding="utf-8"))
        for number in re.findall(r"\d+", listed.group(1) if listed else ""):
            found[int(number)] = path
    return found


def simulate(classjob: int, level: int, seconds: float = 180, targets: int = 1, seed: int = 1) -> dict:
    """`uses` (action: count), `sequence` ((time, action)...), `errors` ((action, message): count), `actions` (the
    job's at this level), `seconds`."""
    if lua54 is None:
        raise RuntimeError("needs lupa: pip install lupa")
    rng = random.Random(seed)
    scripts = action_scripts()
    ranges = gauge_ranges()
    lua = lua54.LuaRuntime(unpack_returned_tuples=True)
    # Global.lua reads constants the server defines (ALL_CAPS): 0 while it loads.
    lua.execute('setmetatable(_G, {__index = function(_, k) if type(k) == "string" and k:match("^[A-Z][A-Z_0-9]*$") '
                "then return 0 end end})")
    lua.execute((repo.SCRIPTS / "Global.lua").read_text(encoding="utf-8"))
    lua.execute("setmetatable(_G, nil)")
    lua.execute(MOCK)
    lua.execute((repo.SCRIPTS / "rotations" / "lib" / "Rotation.lua").read_text(encoding="utf-8"))
    path = rotations().get(classjob)
    if path is None:
        raise ValueError(f"no rotation for ClassJob {classjob}")
    source = path.read_text(encoding="utf-8")
    script = lua.execute(source)
    acts = actions_of(classjob, level, scripts)
    # The combo steps (the Action sheet's ActionCombo on the server): those the rotation follows on, and those whose
    # scripts read `in_combo`. Other actions on the global cooldown leave the combo be.
    constants = {}
    for names, values in re.findall(r"^local ([A-Z_0-9, ]+) = ([\d, ]+)$", source, re.M):
        for name, value in zip(names.split(","), values.split(",")):
            constants[name.strip()] = int(value)
    combo_steps = {constants[name] for name in re.findall(r"b\.combo == ([A-Z_0-9]+)", source) if name in constants}
    load = lua.eval('function(src) local env = setmetatable({}, {__index = _G}); load(src, "s", "t", env)(); '
                    "return env.doAction end")
    functions = {}
    for action in acts:
        text = scripts[action].read_text(encoding="utf-8")
        functions[action] = load(text)
        body = text.split("function doAction", 1)[-1].split(")", 1)[-1]
        if "in_combo" in body:
            combo_steps.add(action)

    statuses: dict[int, float] = {}  # on oneself: when each ends
    params: dict[int, int] = {}
    on_target: dict[int, float] = {}
    used: dict[int, float] = {}
    gauge: Counter = Counter()
    memory = lua.table()
    last_gcd = last_combo = None
    weaves, casting_until, mp, t = 0, 0.0, 10000, 0.0
    uses: Counter = Counter()
    sequence: list = []
    errors: Counter = Counter()
    blm, drk = classjob in BLM, classjob == 32

    def player():
        p = lua.table_from({"is_player": True, "level": level, "classjob": classjob, "mp": mp, "max_mp": 10000,
                            "parameters": lua.table()})
        g = lua.globals()
        g["__g"], g["__st"], g["__p"] = lua.table_from(dict(gauge)), lua.table_from(
            {k: params.get(k, 0) for k in statuses}), p
        lua.execute(PLAYER)
        return p

    while t < seconds:
        for held in (statuses, on_target):
            for status in [s for s, ends in held.items() if ends <= t]:
                del held[status]
        # The server's gauge tick.
        if classjob in DRG and gauge["lotd"] > 0:
            gauge["lotd"] = max(0, gauge["lotd"] - 5)
        if blm and level >= 70:
            if gauge["element"] == 0:
                gauge["enochian_timer"] = 0
            else:
                gauge["enochian_timer"] += 5
                if gauge["enochian_timer"] >= 300:
                    gauge["enochian_timer"] -= 300
                    gauge["polyglot"] = min(3 if level >= 98 else 2 if level >= 80 else 1, gauge["polyglot"] + 1)
        if int(t * 2) % 6 == 0 and not (blm and gauge["element"] > 0):
            mp = min(10000, mp + 200)
        if t >= casting_until:
            since = t - last_gcd[1] if last_gcd else 99
            gcd_left = max(0, max(2.5, last_gcd[2]) - since) if last_gcd else 0
            known = {a: (max(0, v["recast"] - (t - used[a])) if a in used and (not v["gcd"] or v["recast"] > 2.5) else 0)
                     for a, v in acts.items()}
            if drk:
                known = {a: (left if mp >= DRK_COSTS.get(a, 0) or gauge["dark_arts"] else 99) for a, left in known.items()}
            view = lua.globals().BotView(lua.table_from({
                "id": 1, "classjob": classjob, "level": level, "mp": mp, "max_mp": 10000, "hp": 100,
                "target_hp": 100 - 60 * t / seconds, "gcd_left": gcd_left, "since_gcd": since, "weaves": weaves,
                "combo": last_combo[0] if last_combo and t - last_combo[1] < 30 else 0,
                "near_target": targets, "near_self": targets, "distance": 3, "in_combat": t,
                "known": lua.table_from(known), "gcd_actions": lua.table_from({a: v["gcd"] for a, v in acts.items()}),
                "costs": lua.table_from({a: 0 for a in acts}),
                "statuses": lua.table_from({k: v - t for k, v in statuses.items()}), "params": lua.table_from(params),
                "target_mine": lua.table_from({k: v - t for k, v in on_target.items()}),
                "target_any": lua.table_from({k: v - t for k, v in on_target.items()}),
                "gauge": lua.table_from(dict(gauge)), "mem": memory}))
            try:
                chosen = script["choose"](view)
            except Exception as error:  # a rotation script's own failure
                errors[(0, str(error)[:160])] += 1
                chosen = None
            action, on = chosen if isinstance(chosen, tuple) else (chosen, None)
            if action is not None and action in acts:
                entry = acts[action]
                if gcd_left <= 0.25 if entry["gcd"] else known[action] <= 0.25:
                    try:
                        effects = functions[action](player(), True)
                    except Exception as error:
                        errors[(action, str(error)[:160])] += 1
                        effects = None
                    uses[action] += 1
                    sequence.append((round(t, 1), action))
                    used[action] = t
                    before = set(statuses)
                    if effects is not None:
                        ops = [list(op.values()) for op in effects["ops"].values()]
                        critical = any(op[0] == "guarantee_critical" for op in ops) or rng.random() < 0.2
                        to_self = on == "self" or entry["friendly"]
                        for op in ops:
                            name = op[0]
                            if name == "gain_effect_self" or (name == "gain_effect" and to_self):
                                statuses[int(op[1])] = t + (op[3] if len(op) > 3 and op[3] and op[3] > 0 else 999)
                                params[int(op[1])] = int(op[2] or 0)
                            elif name == "gain_effect":
                                on_target[int(op[1])] = t + (op[3] if len(op) > 3 and op[3] and op[3] > 0 else 999)
                            elif name == "damage_over_time":
                                on_target[int(op[1])] = t + op[2]
                            elif name in ("consume_effect", "lose_effect"):
                                statuses.pop(int(op[1]), None)
                            elif name == "consume_effect_stack":
                                status = int(op[1])
                                if status in statuses:
                                    params[status] = params.get(status, 1) - 1
                                    if params[status] <= 0:
                                        statuses.pop(status, None)
                            elif name in ("gauge_add", "gauge_set") and op[1] in ranges:
                                low, high = ranges[op[1]]
                                gauge[op[1]] = max(low, min(high, (gauge[op[1]] if name == "gauge_add" else 0) + int(op[2])))
                            elif name == "gain_aetherflow":
                                gauge["aetherflow"] = int(op[1]) if len(op) > 1 and op[1] else 2
                            elif name == "spend_aetherflow":
                                gauge["aetherflow"] = max(0, gauge["aetherflow"] - 1)
                            elif name == "spend_lily":
                                gauge["lilies"] = max(0, gauge["lilies"] - 1)
                                gauge["blood_lily"] = min(3, gauge["blood_lily"] + 1)
                            elif name == "spend_blood_lily":
                                gauge["blood_lily"] = 0
                            elif name == "restore_mp":
                                mp = min(10000, mp + int(op[1]))
                        set_costs = [int(op[1]) for op in ops if op[0] == "set_mp_cost"]
                        sheet = BLM_COSTS if blm else DRK_COSTS
                        mp = max(0, mp - (set_costs[0] if set_costs else sheet.get(action, 0)))
                        if classjob in MNK and entry["gcd"] and critical and level >= 38 and any(
                                op[0] == "damage" for op in ops):
                            gauge["chakra"] = min(10, gauge["chakra"] + 1)
                    if entry["gcd"]:
                        cast = entry["cast"]
                        if any(status in before and (not only or action in only) for status, only in INSTANT.items()):
                            cast = 0
                        last_gcd = (action, t, cast)
                        weaves = 0
                        casting_until = t + cast
                        if action in combo_steps:
                            last_combo = (action, t)
                    else:
                        weaves += 1
        if classjob in WHM and level >= 52 and int(t * 2) % 40 == 0 and t > 0:
            gauge["lilies"] = min(3, gauge["lilies"] + 1)
        t += 0.5
    return {"uses": uses, "sequence": sequence, "errors": errors, "actions": acts, "seconds": seconds}


def report(classjob: int, level: int, targets: int = 1, seconds: float = 180) -> str:
    run = simulate(classjob, level, seconds, targets)
    minutes = seconds / 60
    lines = [f"ClassJob {classjob} at level {level}, {targets} target(s), {seconds:g} s: uses a minute",
             "  " + ", ".join(f"{a} {n / minutes:.1f}" for a, n in run["uses"].most_common()),
             "  first 40: " + " ".join(f"{t:g}:{a}" for t, a in run["sequence"][:40])]
    unused = sorted(a for a, v in run["actions"].items() if v["gcd"] and not v["friendly"] and a not in run["uses"])
    if unused:
        lines.append(f"  on the global cooldown, for enemies, never used: {', '.join(map(str, unused))}")
    for (action, message), count in run["errors"].items():
        lines.append(f"  ERROR ×{count} in {action or 'the rotation'}: {message}")
    return "\n".join(lines)


BANDS = (1, 10, 15, 20, 30, 40, 50, 60, 70, 80, 90, 100)
# The level each job after A Realm Reborn's starts at.
STARTS = {31: 30, 32: 30, 33: 30, 34: 50, 35: 50, 37: 60, 38: 60, 39: 70, 40: 70, 41: 80, 42: 80}
# The fewest actions on the global cooldown a minute a bot that's never idle uses (2.5 s casts are 24; cast times above
# that and low levels' longer casts take some.)
BUSY = 15


def check(seconds: float = 120) -> list[str]:
    """Every rotation at every level band, alone and against three: script errors, and bots that stand idle."""
    problems = []
    by_file: dict[Path, list[int]] = {}
    for classjob, path in rotations().items():
        by_file.setdefault(path, []).append(classjob)
    for path, classjobs in sorted(by_file.items()):
        for classjob in sorted(classjobs):
            is_class = classjob in CLASS_JOB
            for level in BANDS:
                # A class is played to 30 before its job; the jobs after A Realm Reborn start at 30 or later.
                if (is_class and level > 30) or (not is_class and classjob in CLASS_JOB.values() and level < 30) \
                        or level < STARTS.get(classjob, 1):
                    continue
                for targets in (1, 3):
                    run = simulate(classjob, level, seconds, targets)
                    if not run["actions"]:
                        continue
                    for (action, message), count in run["errors"].items():
                        problems.append(f"{path.name}: ClassJob {classjob} level {level}, {targets} target(s): "
                                        f"{action or 'the rotation'} failed ×{count}: {message}")
                    on_gcd = sum(n for a, n in run["uses"].items() if run["actions"][a]["gcd"])
                    if on_gcd / (seconds / 60) < BUSY:
                        problems.append(f"{path.name}: ClassJob {classjob} level {level}, {targets} target(s): only "
                                        f"{on_gcd / (seconds / 60):.1f} actions on the global cooldown a minute")
    return problems
