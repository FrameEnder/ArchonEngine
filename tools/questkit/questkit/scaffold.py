"""questkit scaffold: a first draft of a quest script, from the quest's dump and what retail did.

From the dump (kawari-questdump):
* quest.json: the Quest sheet's parameters (ACTOR0, EOBJECT1, ENEMY0, ITEM0, SEQ_1_ACTOR2 = 2...), and
* the client script's listing: which scenes exist, and what each does (QuestOffer, QuestAccepted, NpcTrade,
  QuestReward, YesNoQuestBattle, ScenarioMessage, cutscenes, who talks in it).

From a Kawari Recorder export (through a replay fixture): each step the player took on retail, and the quest's
sequence, variables and notices after it.

It writes a script in the fork's pattern (docs/src/fork/quests.md) that does what retail did, step for step, plus the
fixture to check it with `questkit replay`. Steps of the same kind in one sequence (searching several objects,
defeating several enemies) become one handler that works in any order, counting with the nibble retail counted with
and marking each object with the BitFlag8 retail set. Scene numbers it had to guess are marked TODO.

It's a draft: read it against the listing, fix the TODOs, and replay it until it matches.
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from pathlib import Path

from . import fixtures, repo

NIBBLES = [(byte, high) for byte in range(6) for high in (True, False)]


def nibble_name(byte: int, high: bool) -> str:
    return f"UI8{'ABCDEF'[byte]}{'H' if high else 'L'}"


def nibble(variables: bytes, byte: int, high: bool) -> int:
    value = variables[byte]
    return value >> 4 if high else value & 0x0F


# --- The client script's scenes -----------------------------------------------------------------------------------

@dataclass
class Scene:
    number: int
    instructions: int
    methods: set[str] = field(default_factory=set)
    fields: list[str] = field(default_factory=list)

    @property
    def empty(self) -> bool:
        return self.instructions <= 1

    def kind(self) -> str:
        m = self.methods
        if "QuestOffer" in m and "QuestAccepted" in m:
            return "offer+accept"
        if "QuestOffer" in m:
            return "offer"
        if "QuestAccepted" in m:
            return "accepted"
        if "NpcTrade" in m:
            return "trade"
        if "QuestReward" in m or "QuestCompleted" in m:
            return "reward"
        if "YesNoQuestBattle" in m:
            return "battle"
        if "ScenarioMessage" in m or "IsBattleNpcTriggerOwner" in m:
            return "message"
        if {"PlayCutScene", "BeginCutScene"} & m:
            return "cutscene"
        if self.empty:
            return "empty"
        return "talk"

    def speakers(self, script: str) -> set[str]:
        """Who talks in it, from its text keys (TEXT_<SCRIPT>_<SPEAKER>_...)."""
        prefix = f"TEXT_{script.upper()}_"
        found = set()
        for name in self.fields:
            if name.startswith(prefix):
                rest = name[len(prefix):]
                rest = rest[5:] if rest.startswith("TALK_") else rest
                found.add(rest.split("_", 1)[0])
        return found


def read_listing(path: Path) -> dict[int, Scene]:
    scenes: dict[int, Scene] = {}
    current = None
    for line in Path(path).read_text(errors="replace").splitlines():
        match = re.match(r"== \S+:OnScene(\d+)\s+\[.*?(\d+) instructions\]", line)
        if match:
            current = Scene(int(match.group(1)), int(match.group(2)))
            scenes[current.number] = current
            continue
        if line.startswith("== "):
            current = None
            continue
        if current is None:
            continue
        stripped = line.strip()
        if stripped.startswith("methods "):
            current.methods.update(part.strip() for part in stripped[len("methods "):].split(","))
        elif stripped.startswith("fields "):
            current.fields.extend(part.strip() for part in stripped[len("fields "):].split(","))
    return scenes


def speaker_key(label: str) -> str:
    return re.sub(r"[^A-Z0-9]", "", label.upper())


# --- Steps --------------------------------------------------------------------------------------------------------

@dataclass
class Step:
    index: int
    raw: dict
    role: str  # accept, talk, search, kill, finish, battle, battle_won, range, enter_territory
    sequence: int | None  # before the step
    before: bytes
    after: bytes | None
    after_sequence: int | None
    completed: bool
    notices: list | None
    target: int | None = None
    target_name: str | None = None
    scene: int | None = None
    scene_guessed: bool = False
    extra_scene: int | None = None  # the accepted scene, or the reward after a trade
    group: "Group | None" = None


@dataclass
class Group:
    number: int
    role: str
    sequence: int
    steps: list[Step]
    counter: tuple | None = None  # (byte, high)
    increments: list[tuple] = field(default_factory=list)
    flags: dict[int, int] = field(default_factory=dict)  # target -> BitFlag8 index
    guessed_flags: set = field(default_factory=set)
    notice_template: list | None = None
    final_notices: list | None = None  # when only the last step sends these
    final_sequence: int | None = None
    final_vars: bytes | None = None


class Scaffold:
    def __init__(self, dump: dict, listing: Path, fixture: dict):
        self.dump = dump
        self.quest = dump["quest_id"]
        self.script = dump["script"]
        self.fixture = fixture
        self.params = fixtures.numeric_params(dump)
        self.labels = {p["name"]: p.get("label", "") for p in dump.get("params", []) if isinstance(p.get("name"), str)}
        self.scenes = read_listing(listing)
        self.used: set[int] = set()
        self.todo: list[str] = []
        self.by_value: dict[int, list[str]] = {}
        for name, value in self.params.items():
            if re.match(r"^(ACTOR|EOBJECT|ENEMY)\d+$", name):
                self.by_value.setdefault(value, []).append(name)
        self.event_action = next((n for n in self.params if n.startswith(("EVENT_ACTION", "EVENTACTION"))), None)
        self.steps = self._steps()
        self.groups = self._groups()
        self._choose_scenes()

    # Reading the fixture ------------------------------------------------------------------------------------------
    def name_of(self, value: int | None, prefixes=("ACTOR", "EOBJECT", "ENEMY")) -> str | None:
        for name in self.by_value.get(value, []):
            if name.startswith(prefixes):
                return name
        return None

    def _steps(self) -> list[Step]:
        steps = []
        sequence = None
        variables = bytes(6)
        raw_steps = self.fixture["steps"]
        for index, raw in enumerate(raw_steps):
            expect = raw.get("expect") or {}
            completed = bool(expect.get("completed"))
            after = bytes.fromhex(expect["vars"]) if "vars" in expect else (None if not completed else bytes(6))
            after_sequence = expect.get("sequence")
            notices = expect.get("notices")
            target = None
            if "talk" in raw:
                target = raw["talk"]
                name = self.name_of(target)
                if sequence is None:
                    role = "accept"
                elif completed:
                    role = "finish"
                elif not expect and index + 1 < len(raw_steps) and "battle_won" in raw_steps[index + 1]:
                    role = "battle"
                elif name and name.startswith("EOBJECT") and self.event_action:
                    role = "search"
                else:
                    role = "talk"
            elif "kill" in raw:
                role, target = "kill", raw["kill"][0]
                name = self.name_of(target, ("ENEMY",)) or self.name_of(raw["kill"][1], ("ENEMY",))
            elif "range" in raw:
                role, target, name = "range", raw["range"], next(
                    (n for n, v in self.params.items() if v == raw["range"] and n.startswith("EVENTRANGE")), None)
            elif "enter_territory" in raw:
                role, name = "enter_territory", None
            elif "battle_won" in raw:
                role, name = "battle_won", None
            else:
                raise ValueError(f"step {index + 1} isn't filled in: {raw.get('todo', raw)}")
            step = Step(index, raw, role, sequence, variables, after, after_sequence, completed, notices, target, name)
            steps.append(step)
            if after is not None:
                variables = after
            if after_sequence is not None:
                sequence = after_sequence
            if "battle_won" in raw:
                sequence = raw["battle_won"] if after_sequence is None else after_sequence
        return steps

    def _groups(self) -> list[Group]:
        groups: list[Group] = []
        index = 0
        while index < len(self.steps):
            step = self.steps[index]
            if step.role not in ("search", "kill") or step.after is None:
                index += 1
                continue
            run = [step]
            for later in self.steps[index + 1:]:
                if later.role == step.role and later.sequence == step.sequence and later.after is not None:
                    run.append(later)
                    if later.after_sequence != step.sequence:
                        break
                else:
                    break
            if len(run) >= 2 and (step.role == "kill" or len({s.target for s in run}) == len(run)):
                group = Group(len(groups) + 1, step.role, step.sequence, run)
                self._analyse(group)
                if group.counter is None:
                    # Different things count for different todos: keep the steps as retail did them, in order.
                    index += len(run)
                    continue
                for member in run:
                    member.group = group
                groups.append(group)
            index += len(run)
        return groups

    @staticmethod
    def _notice_template(steps: list, needed: int, position_offset: int = 0):
        template = None
        for position, step in enumerate(steps, start=1 + position_offset):
            if step.notices is None:
                return None
            options = [[notice, [{a} | ({"count"} if a == position else set()) | ({"needed"} if a == needed else set())
                                 for a in args]] for notice, args in step.notices]
            if template is None:
                template = options
                continue
            if len(options) != len(template) or any(len(o[1]) != len(t[1]) or o[0] != t[0]
                                                    for o, t in zip(options, template)):
                return None
            for o, t in zip(options, template):
                t[1] = [ts & os for ts, os in zip(t[1], o[1])]
        if template is None or not all(slot for _, args in template for slot in args):
            return None
        return [[notice, [("count" if "count" in slot else "needed" if "needed" in slot else next(iter(slot)))
                          for slot in args]] for notice, args in template]

    def _analyse(self, group: Group):
        """Which nibble counts, what else goes up with each step, and which BitFlag8 each object sets."""
        steps = group.steps
        last = steps[-1]
        counting = steps if last.after_sequence == group.sequence else steps[:-1]
        candidates = []
        for byte, high in NIBBLES:
            if byte == 5:
                continue
            values = [nibble(s.after, byte, high) for s in counting]
            start = nibble(steps[0].before, byte, high)
            if values and all(v == start + i + 1 for i, v in enumerate(values)):
                candidates.append((byte, high))
        if candidates:
            group.counter = candidates[0]
            group.increments = candidates[1:]
        for step in counting:
            added = step.after[5] & ~step.before[5]
            if added and bin(added).count("1") == 1:
                group.flags[step.target] = 8 - added.bit_length() + 1
        if group.role == "search" and group.flags:
            # The last object's flag isn't seen (its step clears the variables): give it the next free one.
            for step in steps:
                if step.target not in group.flags:
                    free = next(i for i in range(1, 9) if i not in group.flags.values())
                    group.flags[step.target] = free
                    group.guessed_flags.add(step.target)
        # Notices: each argument is the count so far, how many are needed, or a fixed number (an icon.) An argument
        # that's both (the last step's [4, 4]) fits either. Some quests only send one on the last step.
        group.notice_template = self._notice_template(steps, len(steps))
        if group.notice_template is None and len(steps) > 1:
            before = self._notice_template(steps[:-1], len(steps))
            final = self._notice_template(steps[-1:], len(steps), position_offset=len(steps) - 1)
            if before is not None and final is not None:
                group.notice_template = before
                group.final_notices = final
        group.final_sequence = last.after_sequence
        group.final_vars = last.after

    # Scenes -------------------------------------------------------------------------------------------------------
    def scenes_of(self, kind: str) -> list[Scene]:
        return [s for n, s in sorted(self.scenes.items()) if s.kind() == kind]

    def take(self, scene: Scene | int | None) -> int | None:
        if scene is None:
            return None
        number = scene if isinstance(scene, int) else scene.number
        self.used.add(number)
        return number

    def by_speaker(self, target_name: str | None, kinds: tuple) -> Scene | None:
        if not target_name:
            return None
        key = speaker_key(self.labels.get(target_name, ""))
        if not key:
            return None
        for scene in sorted(self.scenes.values(), key=lambda s: s.number):
            if scene.number in self.used or scene.kind() not in kinds:
                continue
            if key in scene.speakers(self.script):
                return scene
        return None

    def seq_param(self, sequence: int | None, name: str | None, suffix: str = "") -> int | None:
        if sequence is None or not name:
            return None
        return self.params.get(f"SEQ_{sequence}_{name}{suffix}")

    def _choose_scenes(self):
        empties = [s for s in self.scenes_of("empty")]
        for step in self.steps:
            role = step.role
            if role == "accept":
                offer = (self.scenes_of("offer") + self.scenes_of("offer+accept"))
                offer.sort(key=lambda s: s.number)
                step.scene = self.take(offer[0]) if offer else self.take(self.seq_param(0, step.target_name))
                if offer and offer[0].kind() == "offer":
                    accepted = self.scenes_of("accepted")
                    step.extra_scene = self.take(accepted[0]) if accepted else None
                if step.scene is None:
                    step.scene, step.scene_guessed = 0, True
            elif role == "finish":
                trade = self.by_speaker(step.target_name, ("trade",))
                reward = self.by_speaker(step.target_name, ("reward",))
                rewards = [s for s in self.scenes_of("reward") if s.number not in self.used]
                if reward is None and rewards:
                    reward = rewards[0]
                if trade is None:
                    trades = [s for s in self.scenes_of("trade") if s.number not in self.used]
                    if trades and reward is not None and any(t.number < reward.number for t in trades):
                        trade = [t for t in trades if t.number < reward.number][-1]
                if trade is not None:
                    step.scene = self.take(trade)
                    step.extra_scene = self.take(reward)
                else:
                    step.scene = self.take(reward)
                if step.scene is None:
                    step.scene, step.scene_guessed = self._guess()
            elif role in ("talk", "battle"):
                param = self.seq_param(step.sequence, step.target_name)
                kinds = ("battle",) if role == "battle" else ("talk", "trade", "cutscene")
                scene = param if param is not None else self.by_speaker(step.target_name, kinds)
                if scene is None and role == "battle":
                    battles = self.scenes_of("battle")
                    scene = battles[0] if battles else None
                if scene is None:
                    step.scene, step.scene_guessed = self._guess()
                else:
                    step.scene = self.take(scene)
                    if isinstance(scene, Scene) and scene.kind() == "trade":
                        ok = self.seq_param(step.sequence, step.target_name, "_NPCTRADEOK")
                        step.extra_scene = self.take(ok) if ok is not None else None
                if role == "talk" and step.scene is not None and self.scenes.get(step.scene) and \
                        self.scenes[step.scene].kind() == "trade" and step.extra_scene is None:
                    later = [s for s in sorted(self.scenes) if s > step.scene and s not in self.used
                             and self.scenes[s].kind() in ("talk", "reward")]
                    step.extra_scene = self.take(later[-1]) if later else None
            elif role == "search":
                param = self.seq_param(step.sequence, step.target_name)
                messages = [m for m in self.scenes_of("message") if m.number not in self.used]
                if param is not None:
                    step.scene = self.take(param)
                elif step.raw.get("spawns") and messages:
                    # Searching it brings enemies out: the scene with the "they appear" message.
                    step.scene = self.take(messages[0])
                else:
                    free = [s for s in empties if s.number not in self.used]
                    if free:
                        step.scene = self.take(free[0])
                        step.scene_guessed = True
                    else:
                        step.scene, step.scene_guessed = self._guess()
            elif role == "range":
                messages = [s for s in self.scenes_of("message") if s.number not in self.used]
                step.scene = self.take(messages[0]) if messages else None
            elif role == "enter_territory":
                cutscenes = [s for s in self.scenes_of("cutscene") if s.number not in self.used]
                if cutscenes:
                    step.scene = self.take(cutscenes[0])
                else:
                    step.scene, step.scene_guessed = self._guess()

    def _guess(self) -> tuple[int, bool]:
        for number, scene in sorted(self.scenes.items()):
            if number not in self.used and not scene.empty:
                self.used.add(number)
                return number, True
        return 0, True

    # Writing the script -------------------------------------------------------------------------------------------
    def label(self, name: str | None) -> str:
        if not name:
            return ""
        label = self.labels.get(name)
        return f"{name} ({label})" if label else name

    def scene_name(self, step: Step, extra: bool = False) -> str:
        if step.role == "accept":
            return "SCENE_ACCEPTED" if extra else "SCENE_OFFER"
        if step.role == "finish":
            if step.extra_scene is not None:
                return "SCENE_REWARD" if extra else "SCENE_TRADE"
            return "SCENE_REWARD"
        base = f"SCENE_{step.role.upper()}_{step.target_name or step.index + 1}"
        if step.group is not None and step.role == "search":
            base = f"SCENE_SEARCH_{step.target_name}"
        return base + ("_OK" if extra else "")

    def write(self, source: str = "") -> str:
        out: list[str] = []
        name = self.dump.get("name", "")
        out.append(f"-- Quest: {name}")
        out.append("--")
        out.append("-- DRAFT by questkit scaffold, from the client's quest script (kawari-questdump) and what retail did step by")
        out.append("-- step. Check every TODO against the listing, then `python3 tools/questkit replay "
                   f"{self.quest}` until it matches,")
        out.append("-- and say here what it was checked against (see docs/src/fork/quests.md).")
        out.extend(self._sequence_comments())
        out.append("")
        out.extend(self._constants())
        out.append("")
        out.extend(HELPERS)
        out.append("")
        out.extend(self._groups_lua())
        out.extend(self._on_talk())
        out.extend(self._other_callbacks())
        out.extend(self._scene_finished())
        out.extend(FOOTER)
        return "\n".join(out) + "\n"

    def _describe(self, step: Step) -> str:
        what = {
            "accept": f"{self.label(step.target_name)} offers the quest",
            "talk": f"talk to {self.label(step.target_name)}",
            "search": f"search {self.label(step.target_name)} ({self.event_action})",
            "kill": f"defeat {self.label(step.target_name)}",
            "finish": f"{self.label(step.target_name)}, the reward",
            "battle": f"{self.label(step.target_name)} offers the quest battle",
            "battle_won": "the quest battle is won",
            "range": f"walk into {step.target_name or step.target}",
            "enter_territory": "zone in",
        }[step.role]
        scenes = []
        if step.scene is not None:
            scenes.append(f"scene {step.scene}{' (TODO: guessed)' if step.scene_guessed else ''}")
        if step.extra_scene is not None:
            scenes.append(f"then {step.extra_scene}")
        result = ""
        if step.completed:
            result = " -> completed"
        elif step.after is not None:
            result = f" -> sequence {step.after_sequence}, vars {step.after.hex(' ')}"
        if step.notices:
            result += f", notices {step.notices}"
        return f"{what}{' (' + ', '.join(scenes) + ')' if scenes else ''}{result}"

    def _sequence_comments(self) -> list[str]:
        lines = []
        current = object()
        for step in self.steps:
            label = "accept" if step.sequence is None else step.sequence
            if label != current:
                current = label
                lines.append(f"-- Sequence {'0' if step.sequence is None else step.sequence}:")
            lines.append(f"--   {self._describe(step)}")
        for group in self.groups:
            counter = nibble_name(*group.counter) if group.counter else "TODO"
            lines.append(f"-- The {len(group.steps)} {'searches' if group.role == 'search' else 'kills'} in sequence "
                         f"{group.sequence} work in any order: {counter} counts them"
                         + (", each sets its BitFlag8" if group.flags else "") + ".")
        return lines

    def _constants(self) -> list[str]:
        lines = []
        seen = set()
        for step in self.steps:
            for extra, number in ((False, step.scene), (True, step.extra_scene)):
                if number is None or step.role in ("kill", "battle_won"):
                    continue
                const = self.scene_name(step, extra)
                if const in seen:
                    continue
                seen.add(const)
                note = " -- TODO: guessed, check the listing" if step.scene_guessed and not extra else ""
                lines.append(f"{const} = {number}{note}")
        if self.event_action and any(s.role == "search" for s in self.steps):
            lines.append("")
            lines.append("-- The empty scene after a search plays without hiding the hotbar, as Sapphire does.")
            lines.append("SEARCH_SCENE_FLAGS = 0")
        return lines

    def _spawns(self, step: Step, indent: str) -> list[str]:
        lines = []
        for layout in step.raw.get("spawns", []):
            name = next((n for n, v in self.params.items() if v == layout and n.startswith("ENEMY")), layout)
            lines.append(f"{indent}player:spawn_quest_bnpc({name})")
        return lines

    def _effects(self, step: Step, indent: str) -> list[str]:
        """What a step changes: variables, notices, then the sequence (all literal, as retail had them.)"""
        lines = self._spawns(step, indent)
        if step.completed:
            return lines
        if step.notices:
            for notice, args in step.notices:
                lines.append(f"{indent}player:event_notice(EVENT_ID, {notice}, {{ {', '.join(map(str, args))} }})")
        if step.after is not None:
            for byte in range(6):
                if step.after[byte] != step.before[byte] or (step.role == "accept" and step.after[byte]):
                    lines.append(f"{indent}player:set_quest_var(EVENT_ID, {byte}, 0x{step.after[byte]:02X}) "
                                 f"-- UI8{'ABCDEF'[byte]}")
        if step.after_sequence is not None and (step.after_sequence != step.sequence or step.role == "accept"):
            lines.append(f"{indent}player:quest_sequence(EVENT_ID, {step.after_sequence})")
        return lines

    def _groups_lua(self) -> list[str]:
        lines = []
        for group in self.groups:
            n = group.number
            lines.append(f"-- Group {n}: the {'objects searched' if group.role == 'search' else 'enemies defeated'} "
                         f"in sequence {group.sequence}, in any order.")
            if group.role == "search":
                lines.append(f"GROUP{n} = {{")
                for step in group.steps:
                    flag = group.flags.get(step.target)
                    guess = "  -- TODO: flag guessed (retail clears it on this step)" if step.target in group.guessed_flags else ""
                    lines.append(f"    [{step.target_name or step.target}] = {{ scene = {self.scene_name(step)}"
                                 f"{f', flag = {flag}' if flag else ''} }},{guess}")
                lines.append("}")
            else:
                targets = sorted({s.target_name or str(s.target) for s in group.steps})
                lines.append(f"GROUP{n} = {{ {', '.join(f'[{t}] = true' for t in targets)} }}")
            lines.append(f"GROUP{n}_NEEDED = {len(group.steps)}")
            lines.append("")
            lines.append(f"function group{n}Step(player, entry)")
            if group.counter:
                byte, high = group.counter
                lines.append(f"    local count = getVar(player, {byte}, {str(high).lower()}) + 1 "
                             f"-- {nibble_name(byte, high)} counts them")
            else:
                lines.append("    local count = 1 -- TODO: retail's counter wasn't found")
            lines.append(f"    local needed = GROUP{n}_NEEDED")
            if group.final_notices is not None:
                lines.append("    if count >= needed then")
                for notice, args in group.final_notices:
                    lines.append(f"        player:event_notice(EVENT_ID, {notice}, {{ {', '.join(map(str, args))} }})")
                if group.notice_template:
                    lines.append("    else")
                    for notice, args in group.notice_template:
                        lines.append(f"        player:event_notice(EVENT_ID, {notice}, "
                                     f"{{ {', '.join(map(str, args))} }})")
                lines.append("    end")
            elif group.notice_template is not None:
                for notice, args in group.notice_template:
                    lines.append(f"    player:event_notice(EVENT_ID, {notice}, {{ {', '.join(map(str, args))} }})")
            elif any(s.notices for s in group.steps):
                lines.append("    -- TODO: retail's notices differ from step to step: " +
                             "; ".join(str(s.notices) for s in group.steps))
            for byte, high in group.increments:
                lines.append(f"    setVar(player, {byte}, {str(high).lower()}, getVar(player, {byte}, "
                             f"{str(high).lower()}) + 1) -- {nibble_name(byte, high)}")
            lines.append("    if count >= needed then")
            if group.final_vars is not None:
                for byte in range(6):
                    lines.append(f"        player:set_quest_var(EVENT_ID, {byte}, 0x{group.final_vars[byte]:02X})")
            if group.final_sequence is not None and group.final_sequence != group.sequence:
                lines.append(f"        player:quest_sequence(EVENT_ID, {group.final_sequence})")
            lines.append("    else")
            if group.counter:
                byte, high = group.counter
                lines.append(f"        setVar(player, {byte}, {str(high).lower()}, count)")
            if group.flags:
                lines.append("        setFlag(player, entry.flag)")
            lines.append("    end")
            lines.append("end")
            lines.append("")
        return lines

    def _on_talk(self) -> list[str]:
        lines = ["function onTalk(target, player)", "    local sequence = player:get_quest_sequence(EVENT_ID)", ""]
        first = True
        done_groups = set()
        for step in self.steps:
            if step.role not in ("accept", "talk", "search", "finish", "battle"):
                continue
            keyword = "if" if first else "elseif"
            first = False
            seq = "nil" if step.sequence is None else step.sequence
            target = step.target_name or step.target
            if step.group is not None:
                if step.group.number in done_groups:
                    continue
                done_groups.add(step.group.number)
                n = step.group.number
                condition = f"sequence == {seq} and GROUP{n}[BASE_ID] ~= nil"
                if step.group.flags:
                    condition += f" and not hasFlag(player, GROUP{n}[BASE_ID].flag)"
                lines.append(f"    {keyword} {condition} then")
                lines.append(f"        searching = GROUP{n}[BASE_ID]")
                lines.append(f"        searching_group = {n}")
                lines.append(f"        player:event_action({self.event_action})")
                continue
            lines.append(f"    {keyword} sequence == {seq} and BASE_ID == {target} then")
            lines.append(f"        step = {step.index + 1}")
            if step.role == "search":
                lines.append(f"        player:event_action({self.event_action})")
            else:
                lines.append(f"        play(player, {self.scene_name(step)}, HIDE_HOTBAR)")
        lines.append("    else" if not first else "    if true then")
        lines.append("        player:finish_event()")
        lines.append("    end")
        lines.append("end")
        lines.append("")
        return lines

    def _other_callbacks(self) -> list[str]:
        lines = []
        if any(s.role == "search" for s in self.steps):
            lines.append("function onEventActionCast(player)")
            lines.append("    if searching ~= nil then")
            lines.append("        play(player, searching.scene, SEARCH_SCENE_FLAGS)")
            for step in self.steps:
                if step.role == "search" and step.group is None:
                    lines.append(f"    elseif step == {step.index + 1} then")
                    lines.append(f"        play(player, {self.scene_name(step)}, SEARCH_SCENE_FLAGS)")
            lines.append("    else")
            lines.append("        player:finish_event()")
            lines.append("    end")
            lines.append("end")
            lines.append("")
        kills = [s for s in self.steps if s.role == "kill"]
        if kills:
            lines.append("function onBNpcKill(base_id, layout_id, player)")
            lines.append("    local sequence = player:get_quest_sequence(EVENT_ID)")
            first = True
            done = set()
            for step in kills:
                keyword = "if" if first else "elseif"
                if step.group is not None:
                    if step.group.number in done:
                        continue
                    done.add(step.group.number)
                    n = step.group.number
                    lines.append(f"    {keyword} sequence == {step.sequence} and (GROUP{n}[base_id] or "
                                 f"GROUP{n}[layout_id]) then")
                    lines.append(f"        group{n}Step(player, nil)")
                else:
                    target = step.target_name or step.target
                    lines.append(f"    {keyword} sequence == {step.sequence} and (base_id == {target} or "
                                 f"layout_id == {target}) then")
                    lines.extend(self._effects(step, "        "))
                first = False
            lines.append("    end")
            lines.append("end")
            lines.append("")
        ranges = [s for s in self.steps if s.role == "range"]
        if ranges:
            lines.append("function onEnterTrigger(player, arg)")
            for step in ranges:
                lines.append(f"    if player:get_quest_sequence(EVENT_ID) == {step.sequence} then")
                if step.scene is None:
                    spawns = self._spawns(step, "        ")
                    lines.extend(spawns or ["        -- TODO: what walking in brings out (spawn_quest_bnpc(ENEMY0)?)"])
                if step.scene is not None:
                    lines.append(f"        step = {step.index + 1}")
                    lines.append(f"        play(player, {self.scene_name(step)}, NO_DEFAULT_CAMERA | HIDE_HOTBAR)")
                    lines.append("        return")
                lines.append("    end")
            lines.append("    player:finish_event()")
            lines.append("end")
            lines.append("")
        entering = [s for s in self.steps if s.role == "enter_territory"]
        if entering:
            lines.append("function onEnterTerritory(player)")
            for step in entering:
                lines.append(f"    if player:get_quest_sequence(EVENT_ID) == {step.sequence} then")
                lines.append(f"        step = {step.index + 1}")
                lines.append(f"        play(player, {self.scene_name(step)}, INITIAL_CUTSCENE_FLAGS)")
                lines.append("        return")
                lines.append("    end")
            lines.append("    player:finish_event()")
            lines.append("end")
            lines.append("")
        return lines

    def _scene_finished(self) -> list[str]:
        lines = [
            "function sceneFinished(scene, results, player)",
            "    if scene ~= current_scene then",
            "        return",
            "    end",
            "    current_scene = nil",
            "",
        ]
        first = True

        def branch(condition: str):
            nonlocal first
            lines.append(f"    {'if' if first else 'elseif'} {condition} then")
            first = False

        for group in self.groups:
            if group.role == "search":
                branch(f"searching ~= nil and searching_group == {group.number} and scene == searching.scene")
                lines.append("        local entry = searching")
                lines.append("        searching = nil")
                lines.append(f"        group{group.number}Step(player, entry)")
        for step in self.steps:
            number = step.index + 1
            if step.group is not None or step.role in ("kill", "battle_won"):
                continue
            first_scene = self.scene_name(step)
            if step.role == "accept":
                branch(f"step == {number} and scene == {first_scene}")
                lines.append("        if results[1] == 1 then")
                if step.extra_scene is not None:
                    lines.append(f"            play(player, {self.scene_name(step, True)}, HIDE_HOTBAR)")
                    lines.append("            return")
                else:
                    lines.append("            player:accept_quest(EVENT_ID)")
                    lines.extend(self._effects(step, "            "))
                lines.append("        end")
                if step.extra_scene is not None:
                    branch(f"step == {number} and scene == {self.scene_name(step, True)}")
                    lines.append("        player:accept_quest(EVENT_ID)")
                    lines.extend(self._effects(step, "        "))
            elif step.role == "finish":
                if step.extra_scene is not None:
                    branch(f"step == {number} and scene == {first_scene}")
                    lines.append("        -- 1 when the items were handed over, nothing when the window was closed.")
                    lines.append("        if results[1] == 1 then")
                    lines.append(f"            play(player, {self.scene_name(step, True)}, HIDE_HOTBAR)")
                    lines.append("            return")
                    lines.append("        end")
                    branch(f"step == {number} and scene == {self.scene_name(step, True)}")
                else:
                    branch(f"step == {number} and scene == {first_scene}")
                lines.append("        if results[1] == 1 then")
                lines.append("            player:finish_quest(EVENT_ID, results[2])")
                lines.append("        end")
            elif step.role == "battle":
                branch(f"step == {number} and scene == {first_scene}")
                lines.append("        if results[1] == 1 then")
                lines.append("            player:finish_event()")
                lines.append("            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)")
                lines.append("            return")
                lines.append("        end")
            else:
                if step.scene is None:
                    continue
                trade = self.scenes.get(step.scene) and self.scenes[step.scene].kind() == "trade"
                if trade and step.extra_scene is not None:
                    branch(f"step == {number} and scene == {first_scene}")
                    lines.append("        if results[1] == 1 then")
                    lines.append(f"            play(player, {self.scene_name(step, True)}, HIDE_HOTBAR)")
                    lines.append("            return")
                    lines.append("        end")
                    branch(f"step == {number} and scene == {self.scene_name(step, True)}")
                else:
                    branch(f"step == {number} and scene == {first_scene}")
                effects = self._effects(step, "        ")
                lines.extend(effects or ["        -- nothing changes"])
        if not first:
            lines.append("    end")
        lines.append("")
        lines.append("    player:finish_event()")
        lines.append("end")
        lines.append("")
        return lines


HELPERS = """-- Quest variables: UI8A..F are bytes 0..5, H is the high nibble and L the low one.
function getVar(player, byte, high)
    local value = player:get_quest_var(EVENT_ID, byte)
    if high then
        return value >> 4
    end
    return value & 0x0F
end

function setVar(player, byte, high, nibble)
    local value = player:get_quest_var(EVENT_ID, byte)
    if high then
        value = (value & 0x0F) | (nibble << 4)
    else
        value = (value & 0xF0) | nibble
    end
    player:set_quest_var(EVENT_ID, byte, value)
end

-- BitFlag8: flag 1 is the top bit of UI8F, as retail sets it.
function hasFlag(player, flag)
    return (player:get_quest_var(EVENT_ID, 5) & (0x80 >> (flag - 1))) ~= 0
end

function setFlag(player, flag)
    player:set_quest_var(EVENT_ID, 5, player:get_quest_var(EVENT_ID, 5) | (0x80 >> (flag - 1)))
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- Which step of the quest this event is (see the sequence notes above), and the object being searched.
step = nil
searching = nil
searching_group = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end""".splitlines()

FOOTER = """function onYield(scene, id, results, player)
    sceneFinished(scene, results, player)
end

function onReturn(scene, results, player)
    sceneFinished(scene, results, player)
end""".splitlines()


def listing_for(dumps: Path, script: str) -> Path | None:
    folder = Path(dumps) / script
    path = folder / f"{script}.txt"
    return path if path.exists() else None


def scaffold(quest: int, dumps: Path, fixture: dict) -> tuple[str, Scaffold]:
    dump = fixtures.load_dump(dumps, quest)
    if dump is None:
        raise ValueError(f"no dump of quest {quest} in {dumps} (kawari-questdump dump {quest})")
    listing = listing_for(dumps, dump["script"])
    if listing is None:
        raise ValueError(f"no listing for {dump['script']} in {dumps}")
    draft = Scaffold(dump, listing, fixture)
    return draft.write(), draft


def default_path(dump: dict) -> Path:
    return repo.quest_script_path(dump["script"], dump["quest_id"])
