"""questkit transcribe: a quest script that does what retail did, from Chronofoil fixtures.

A Chronofoil fixture (`questkit fixture --chronofoil`) has every step of a quest as retail played it: who or what
started the event, every scene with its flags and parameters and how the client answered it, the progress bars
between them, the notices, the quest's sequence and variables after it, and what else the server did (zone changes,
unlocks). `transcribe` turns one or more fixtures of a quest into a script in the fork's pattern
(docs/src/fork/quests.md) that plays the same scenes with the same flags and changes the same things:

* each step is one branch, chosen by what started it (talking to someone, a key item, an emote, an event range,
  entering the zone, a kill, a duty cleared) and the sequence it happened in; steps that start the same way in the
  same sequence are told apart by the variables retail had before them;
* a scene answered with a yield of 1 (an offer, a trade, a reward, a quest battle's yes/no) goes on only on 1; a
  reward window closed sets REWARD_CLOSED, as retail does;
* a progress bar between two scenes is the step's event action; a step whose bar retail broke off is the same step;
* the scenes after a reward that carry the chosen item get it as their parameter;
* the step before a quest battle won joins it (QUESTBATTLE0 in the TERRITORYTYPE retail went to).

What it can't know is left as a TODO: things done in any order (it keeps retail's order and literal variables, so
counters and BitFlag8s are written by hand, see Sweet Somethings), zone changes within a zone (a pop range), and how
the client's script gates each step (IsAcceptEvent): read the listing.
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from pathlib import Path

from . import fixtures, repo

FLAG_ORDER = [
    "NO_DEFAULT_CAMERA", "FADE_OUT", "INVIS_ENPC", "INVIS_EOBJ", "INVIS_BNPC", "INVIS_OTHER_PC", "INVIS_PARTY_PC",
    "INVIS_GATHERING_POINT", "INVIS_AETHERYTE", "INVIS_TREASURE", "CONDITION_CUTSCENE", "HIDE_UI", "AUTO_LOC_CAMERA",
    "HIDE_HOTBAR", "INVINCIBLE", "DISABLE_SKIP",
]
SET_BASE = 0xF8400EFB


def flags_lua(value: int) -> str:
    """Retail's scene flags as the names Global.lua has (SET_BASE first: the cutscenes' set)."""
    names = repo.scene_flags()
    parts = []
    rest = value
    if value & SET_BASE == SET_BASE:
        parts.append("SET_BASE")
        rest &= ~SET_BASE
    for name in FLAG_ORDER:
        bit = names.get(name)
        if bit and rest & bit == bit:
            parts.append(name)
            rest &= ~bit
    if rest or not parts:
        return f"0x{value:X}"
    return " | ".join(parts)


@dataclass
class SceneStep:
    scene: int
    flags: int
    params: list
    results: list
    yielded: bool
    bar: int | None = None
    error: int = 0


@dataclass
class Handler:
    """One step of the quest: what starts it, in which sequence, the scenes, what changes."""
    number: int
    trigger: tuple
    sequence: int | None
    before: bytes
    scenes: list[SceneStep]
    after_sequence: int | None
    after: bytes | None
    completed: bool
    notices: list
    effects: list
    spawns: list
    joins_battle: bool = False
    accepted: bool = False
    sources: list = field(default_factory=list)
    pop_ranges: list = field(default_factory=list)
    guard: list = field(default_factory=list)
    territory: int | None = None  # where the step happens, when retail moved the player within it
    arrivals: list = field(default_factory=list)  # where retail's moves put the player: x, y, z, rotation


def _trigger(step: dict) -> tuple | None:
    if "talk" in step:
        return ("talk", step["talk"])
    if "event_item" in step:
        return ("item", step["event_item"][0], step["event_item"][1])
    if "emote" in step:
        return ("emote", step["emote"][0], step["emote"][1])
    if "range" in step:
        return ("range", step["range"])
    if "enter_territory" in step:
        return ("territory",)
    if "kill" in step:
        return ("kill", step["kill"][0], step["kill"][1])
    if "duty_cleared" in step:
        return ("duty", step["duty_cleared"])
    return None


def _scenes(step: dict) -> list[SceneStep]:
    out = []
    for entry in step.get("scenes", []):
        flags = entry["flags"]
        out.append(SceneStep(entry["scene"], int(flags, 16) if isinstance(flags, str) else int(flags),
                             list(entry.get("params", [])), list(entry.get("results", [])),
                             bool(entry.get("yield")), entry.get("bar"), int(entry.get("error", 0))))
    return out


def lua_list(values) -> str:
    return "{ " + ", ".join(map(str, values)) + " }" if values else "{}"


class Transcript:
    def __init__(self, dump: dict, fixture_list: list[tuple[str, dict]]):
        self.dump = dump
        self.quest = dump["quest_id"]
        self.params = fixtures.numeric_params(dump)
        self.labels = {p["name"]: p.get("label", "") for p in dump.get("params", []) if isinstance(p.get("name"), str)}
        self.todo: list[str] = []
        self.handlers: list[Handler] = []
        self.broken: set = set()  # (trigger, sequence, scene): scenes retail saw broken off, which change nothing
        for name, fixture in fixture_list:
            self._read(name, fixture)
        self._guards()
        self.any_order = self._any_order()
        self.member = {id(m["handler"]): (g, m) for g in self.any_order for m in g["members"]}

    # Names --------------------------------------------------------------------------------------------------------
    def name(self, value, prefixes=("ACTOR", "EOBJECT", "ENEMY")) -> str:
        for name, v in self.params.items():
            if v == value and re.match(rf"^({'|'.join(prefixes)})\d+$", name):
                return name
        return str(value)

    def param(self, value, prefix: str) -> str | None:
        # EVENT_ACTION_SEARCH, EVENT_ACTION_LOOKOUT_LONG...: the event actions are named for what they show.
        pattern = rf"^{prefix}(_[A-Z0-9_]+)?$" if prefix == "EVENT_ACTION" else rf"^{prefix}\d*$"
        for name, v in self.params.items():
            if v == value and re.match(pattern, name):
                return name
        return None

    def label(self, name: str) -> str:
        label = self.labels.get(name)
        return f"{name} ({label})" if label else name

    # Reading the fixtures -----------------------------------------------------------------------------------------
    def _read(self, source: str, fixture: dict):
        steps = fixture["steps"]
        start = fixture.get("from")
        sequence = start["sequence"] if start else None
        variables = bytes.fromhex(start.get("vars", "00" * 6)) if start else bytes(6)
        for index, step in enumerate(steps):
            expect = step.get("expect") or {}
            if "battle_won" in step:
                sequence = expect.get("sequence", step["battle_won"])
                variables = bytes.fromhex(expect["vars"]) if "vars" in expect else variables
                continue
            trigger = _trigger(step)
            if trigger is None:
                self.todo.append(f"{source} step {index + 1} isn't filled in")
                continue
            completed = bool(expect.get("completed"))
            declined = expect.get("accepted") is False
            after_sequence = None if completed or declined else expect.get("sequence")
            after = None if completed or declined or "vars" not in expect else bytes.fromhex(expect["vars"])
            joins = index + 1 < len(steps) and "battle_won" in steps[index + 1]
            handler = Handler(0, trigger, sequence, variables, _scenes(step), after_sequence, after, completed,
                              [list(n) for n in expect.get("notices", [])], list(step.get("effects", [])),
                              list(step.get("spawns", [])), joins, sequence is None and after_sequence is not None,
                              [f"{source} step {index + 1}"], list(step.get("pop_ranges", [])), [],
                              step.get("territory"), [list(a) for a in step.get("arrivals", [])])
            # A scene the client broke off (an error code, nothing changed): the step didn't happen, and the script
            # mustn't move on when it comes back that way.
            broken = bool(handler.scenes) and handler.scenes[-1].error != 0 and not handler.notices and \
                (handler.after is None or (handler.after == handler.before and handler.after_sequence == sequence))
            if broken:
                self.broken.add((trigger, sequence, handler.scenes[-1].scene))
            if step.get("interrupted") or declined or broken:
                handler.after_sequence, handler.after = None, None
            self._add(handler, partial=bool(step.get("interrupted")) or declined or broken)
            if after_sequence is not None:
                sequence = after_sequence
            if after is not None:
                variables = after
            if completed:
                sequence = None

    @staticmethod
    def _union(kept: Handler, other: Handler):
        """What the server did on either run of the same step (a quest battle joined on one of them, say.)"""
        kept.joins_battle = kept.joins_battle or other.joins_battle
        for key in ("effects", "spawns", "pop_ranges", "arrivals"):
            for value in getattr(other, key):
                if value not in getattr(kept, key):
                    getattr(kept, key).append(value)

    def _add(self, new: Handler, partial: bool):
        merged = self._merge(new, partial)
        if merged is not None and merged is not new:
            self._union(merged, new)

    def _merge(self, new: Handler, partial: bool) -> Handler | None:
        for old in self.handlers:
            if old.trigger != new.trigger or old.sequence != new.sequence or old.before != new.before:
                continue
            numbers_old = [s.scene for s in old.scenes]
            numbers_new = [s.scene for s in new.scenes]
            unchanged_old = old.after is None or (old.after == old.before and old.after_sequence == old.sequence
                                                  and not old.notices)
            unchanged_new = new.after is None or (new.after == new.before and new.after_sequence == new.sequence
                                                  and not new.notices)
            if partial and numbers_new == numbers_old[:len(numbers_new)]:
                old.sources += new.sources
                return old
            if numbers_old == numbers_new and unchanged_old and not unchanged_new:
                # A talk that changed nothing at first (a quest battle lost, say) and later did.
                new.number = old.number
                new.sources = old.sources + new.sources
                self._union(new, old)
                self.handlers[self.handlers.index(old)] = new
                return new
            if numbers_old == numbers_new and unchanged_new:
                old.sources += new.sources
                return old
            if numbers_old[:len(numbers_new)] == numbers_new and len(numbers_new) < len(numbers_old):
                old.sources += new.sources
                return old
            if numbers_new[:len(numbers_old)] == numbers_old and unchanged_old and not old.completed:
                new.number = old.number
                new.sources = old.sources + new.sources
                self._union(new, old)
                self.handlers[self.handlers.index(old)] = new
                return new
            if numbers_old == numbers_new and old.after == new.after and old.after_sequence == new.after_sequence \
                    and old.notices == new.notices and old.completed == new.completed:
                old.sources += new.sources
                return old
            self.todo.append(f"{', '.join(new.sources)} played {numbers_new} and changed other things than "
                             f"{', '.join(old.sources)} ({numbers_old}) from the same state: kept the first")
            return old
        new.number = len(self.handlers) + 1
        self.handlers.append(new)
        return new

    def _guards(self):
        """Steps starting the same way in the same sequence: the variable bytes that tell them apart."""
        for handler in self.handlers:
            same = [h for h in self.handlers if h.trigger == handler.trigger and h.sequence == handler.sequence
                    and h is not handler]
            handler.guard = sorted({b for h in same for b in range(6) if h.before[b] != handler.before[b]})

    def _any_order(self) -> list[dict]:
        """Talks, emotes or key items in one sequence that retail counted in a nibble, each setting a BitFlag8, the
        last moving the quest on (Sweet Somethings' three to ask): one group the player can do in any order."""
        groups = []
        sequences = sorted({h.sequence for h in self.handlers if h.sequence is not None})
        for sequence in sequences:
            for kind in ("talk", "emote", "item"):
                members = [h for h in self.handlers if h.trigger[0] == kind and h.sequence == sequence
                           and h.after is not None and not h.accepted and (h.after != h.before or h.notices)]
                if len(members) < 2:
                    continue
                final = [h for h in members if h.after_sequence != sequence]
                counting = [h for h in members if h.after_sequence == sequence]
                if len(final) > 1 or not counting:
                    continue
                counter = None
                for byte, high in [(b, hi) for b in range(5) for hi in (True, False)]:
                    def nib(v, byte=byte, high=high):
                        return v[byte] >> 4 if high else v[byte] & 0x0F
                    if all(nib(h.after) == nib(h.before) + 1 for h in counting) and \
                            sorted(nib(h.before) for h in members) == list(range(len(members))):
                        counter = (byte, high)
                        break
                if counter is None:
                    continue
                flags = {}
                for h in counting:
                    added = h.after[5] & ~h.before[5]
                    if added and bin(added).count("1") == 1:
                        flags[id(h)] = 8 - added.bit_length() + 1
                if len(flags) != len(counting):
                    continue
                for h in final:
                    flags[id(h)] = next(i for i in range(1, 9) if i not in flags.values())
                byte, high = counter
                group = {"sequence": sequence, "kind": kind, "counter": counter, "needed": len(members),
                         "final": final[0] if final else None, "members": []}
                templates = []
                for h in members:
                    count = (h.before[byte] >> 4 if high else h.before[byte] & 0x0F) + 1
                    extras = []
                    if h.after_sequence == sequence:
                        for b in range(5):
                            for hi in (True, False):
                                if (b, hi) == counter:
                                    continue
                                old_value = h.before[b] >> 4 if hi else h.before[b] & 0x0F
                                new_value = h.after[b] >> 4 if hi else h.after[b] & 0x0F
                                if new_value != old_value:
                                    extras.append((b, hi, new_value))
                        templates.append([[n, ["count" if a == count else "needed" if a == len(members) else str(a)
                                               for a in args]] for n, args in h.notices])
                    group["members"].append({"handler": h, "flag": flags[id(h)], "extras": extras,
                                             "guessed": h in final})
                group["template"] = templates[0] if templates and all(t == templates[0] for t in templates) \
                    and any("count" in a for _, args in templates[0] for a in args) else None
                groups.append(group)
        return groups

    def _group_outcome(self, handler: Handler, indent: str) -> list[str]:
        group, member = self.member[id(handler)]
        byte, high = group["counter"]
        h = "true" if high else "false"
        lines = [f"{indent}local count = getVar(player, {byte}, {h}) + 1"]
        if group["template"] is not None:
            for notice, args in group["template"]:
                lua_args = ", ".join("count" if a == "count" else str(group["needed"]) if a == "needed" else a
                                     for a in args)
                lines.append(f"{indent}player:event_notice(EVENT_ID, {notice}, {{ {lua_args} }})")
        else:
            for notice, args in handler.notices:
                lines.append(f"{indent}player:event_notice(EVENT_ID, {notice}, {lua_list(args)})")
        lines.append(f"{indent}if count >= {group['needed']} then")
        final = group["final"]
        if final is not None:
            for b in range(6):
                lines.append(f"{indent}    player:set_quest_var(EVENT_ID, {b}, 0x{final.after[b]:02X})")
            lines.append(f"{indent}    player:quest_sequence(EVENT_ID, {final.after_sequence})")
        lines.append(f"{indent}else")
        lines.append(f"{indent}    setVar(player, {byte}, {h}, count)")
        for b, hi, value in member["extras"]:
            lines.append(f"{indent}    setVar(player, {b}, {'true' if hi else 'false'}, {value}) "
                         f"-- UI8{'ABCDEF'[b]}{'H' if hi else 'L'}")
        lines.append(f"{indent}    setFlag(player, {member['flag']})"
                     + ("  -- not seen on retail (its step was the last): the next free one" if member["guessed"]
                        else ""))
        lines.append(f"{indent}end")
        return lines

    # Writing ------------------------------------------------------------------------------------------------------
    def condition(self, handler: Handler) -> str:
        sequence = "nil" if handler.sequence is None else str(handler.sequence)
        parts = [f"sequence == {sequence}"]
        kind = handler.trigger[0]
        if kind == "talk":
            parts.append(f"BASE_ID == {self.name(handler.trigger[1])}")
        elif kind in ("item", "emote"):
            what = self.param(handler.trigger[1], "ITEM") if kind == "item" else None
            parts.append(f"{'item' if kind == 'item' else 'emote'} == {what or handler.trigger[1]}")
            parts.append(f"BASE_ID == {self.name(handler.trigger[2])}")
        elif kind == "range":
            parts.append(f"arg == {self.param(handler.trigger[1], 'EVENTRANGE') or handler.trigger[1]}")
        elif kind == "kill":
            # A quest's own enemy by its layout (ENEMY* is one), an open-world one by its BNpcBase.
            layout = self.name(handler.trigger[2], ("ENEMY",))
            parts.append(f"layout_id == {layout}" if layout.startswith("ENEMY")
                         else f"base_id == {self.name(handler.trigger[1], ('ENEMY',))}")
        elif kind == "duty":
            parts.append(f"content_id == {self.param(handler.trigger[1], 'INSTANCEDUNGEON') or handler.trigger[1]}")
        if id(handler) in getattr(self, "member", {}):
            parts.append(f"not hasFlag(player, {self.member[id(handler)][1]['flag']})")
        else:
            for byte in handler.guard:
                parts.append(f"player:get_quest_var(EVENT_ID, {byte}) == 0x{handler.before[byte]:02X}")
        return " and ".join(parts)

    def describe(self, handler: Handler) -> str:
        kind = handler.trigger[0]
        if kind == "talk":
            what = self.label(self.name(handler.trigger[1]))
        elif kind == "item":
            what = f"key item {handler.trigger[1]} on {self.label(self.name(handler.trigger[2]))}"
        elif kind == "emote":
            what = f"emote {handler.trigger[1]} on {self.label(self.name(handler.trigger[2]))}"
        elif kind == "range":
            what = f"walking into {self.param(handler.trigger[1], 'EVENTRANGE') or handler.trigger[1]}"
        elif kind == "territory":
            what = "entering the zone"
        elif kind == "kill":
            what = f"defeating {self.name(handler.trigger[1], ('ENEMY',))}"
        else:
            what = f"clearing {self.param(handler.trigger[1], 'INSTANCEDUNGEON') or handler.trigger[1]}"
        scenes = []
        for s in handler.scenes:
            text = f"{s.scene} ({flags_lua(s.flags) if s.flags != 0x2000 else 'HIDE_HOTBAR'}"
            if s.yielded:
                text += ", a yield"
            text += ")"
            if s.bar is not None:
                text += f", the bar ({self.param(s.bar, 'EVENT_ACTION') or s.bar})"
            scenes.append(text)
        out = what + (": scene " + ", ".join(scenes) if scenes else "")
        if handler.completed:
            out += " -> completed"
        elif handler.accepted:
            out += f" -> accepted, sequence {handler.after_sequence}"
        elif handler.after_sequence is not None and handler.after_sequence != handler.sequence:
            out += f" -> sequence {handler.after_sequence}"
        if handler.after is not None and handler.after != handler.before:
            out += f", vars {handler.after.hex(' ')}"
        if handler.notices:
            out += f", notices {handler.notices}"
        if handler.joins_battle:
            out += ", into the quest battle"
        if handler.effects:
            out += ", " + ", ".join(f"{k} {v}" for e in handler.effects for k, v in e.items())
        return out

    def write(self, source: str) -> str:
        out = [f"-- Quest: {self.dump.get('name', '')}", "--",
               f"-- Transcribed by questkit from the client's quest script (kawari-questdump) and {source}:"]
        current = object()
        for handler in sorted(self.handlers, key=lambda h: (-1 if h.sequence is None else h.sequence, h.number)):
            label = 0 if handler.sequence is None else handler.sequence
            if label != current:
                current = label
                out.append(f"-- Sequence {label}:")
            out.append(f"--   {self.describe(handler)}")
        for g in self.any_order:
            what = {"talk": "talks", "emote": "emotes", "item": "key item uses"}[g["kind"]]
            out.append(f"-- The {g['needed']} {what} of sequence {g['sequence']} work in any order: "
                       f"UI8{'ABCDEF'[g['counter'][0]]}{'H' if g['counter'][1] else 'L'} counts them and each sets its "
                       f"BitFlag8 (retail's), the last moves the quest on.")
        for g in self.kill_groups():
            out.append(f"-- The {g['needed']} enemies of sequence {g['sequence']} count in any order "
                       f"(UI8{'ABCDEF'[g['counter'][0]]}{'H' if g['counter'][1] else 'L'}).")
        for text in self.todo:
            out.append(f"-- TODO: {text}")
        out.append("")
        out.append("-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855, 960.)")
        out.append("REWARD_CLOSED = 0x11")
        out.append("")
        out.extend(HELPERS)
        out.append("")
        if self.kill_groups() or self.any_order:
            out.extend(NIBBLES)
            out.append("")
        if self.any_order:
            out.extend(FLAGS)
            out.append("")
        out.extend(self._starts())
        out.extend(self._others())
        out.extend(self._finished())
        out.extend(FOOTER)
        return "\n".join(out) + "\n"

    def _first(self, handler: Handler, indent: str) -> list[str]:
        lines = [f"{indent}step = {handler.number}"]
        if handler.scenes:
            lines.append(f"{indent}{self._play(handler, 0)}")
        else:
            lines.extend(self._outcome(handler, indent))
            lines.append(f"{indent}player:finish_event()")
        lines.append(f"{indent}return")
        return lines

    def _play(self, handler: Handler, index: int) -> str:
        scene = handler.scenes[index]
        params = ""
        if scene.params:
            chosen = self._chosen_index(handler)
            if chosen is not None and chosen < index and scene.params == [handler.scenes[chosen].results[1]]:
                params = ", { chosen }"
            else:
                params = ", { " + ", ".join(map(str, scene.params)) + " }"
        return f"play(player, {scene.scene}, {flags_lua(scene.flags)}{params})"

    def _chosen_index(self, handler: Handler) -> int | None:
        """The reward scene: the last yield of 1 with the item picked, in a step that completes the quest."""
        if not handler.completed:
            return None
        for index in range(len(handler.scenes) - 1, -1, -1):
            s = handler.scenes[index]
            if s.yielded and len(s.results) >= 2 and s.results[0] == 1:
                return index
        return None

    def _starts(self) -> list[str]:
        lines = []
        for kind, signature in (("talk", "onTalk(target, player)"), ("item", "onEventItem(item, target, player)"),
                                ("emote", "onEmote(emote, target, player)"),
                                ("range", "onEnterTrigger(player, arg)"),
                                ("territory", "onEnterTerritory(player)")):
            handlers = [h for h in self.handlers if h.trigger[0] == kind]
            if not handlers and kind != "talk":
                continue
            lines.append(f"function {signature}")
            lines.append("    local sequence = player:get_quest_sequence(EVENT_ID)")
            lines.append("    step = nil")
            for i, handler in enumerate(handlers):
                lines.append(f"    {'if' if i == 0 else 'elseif'} {self.condition(handler)} then")
                lines.extend(self._first(handler, "        "))
            if handlers:
                lines.append("    end")
            lines.append("    player:finish_event()")
            lines.append("end")
            lines.append("")
        return lines

    def kill_groups(self) -> list[dict]:
        """Kills in one sequence that count up one nibble, the last moving the quest on: retail's counter. Each is
        {sequence, handlers, counter: (byte, high), needed, final, keys}; kills that don't fit stay single."""
        groups = []
        sequences = sorted({h.sequence for h in self.handlers if h.trigger[0] == "kill" and h.sequence is not None})
        for sequence in sequences:
            kills = [h for h in self.handlers if h.trigger[0] == "kill" and h.sequence == sequence]
            if len(kills) < 2:
                continue
            final = [h for h in kills if h.after_sequence != sequence]
            counting = [h for h in kills if h.after_sequence == sequence]
            if len(final) > 1 or not counting:
                continue
            for byte, high in [(b, hi) for b in range(5) for hi in (True, False)]:
                def nib(v):
                    return v[byte] >> 4 if high else v[byte] & 0x0F
                if all(h.after is not None and nib(h.after) == nib(h.before) + 1
                       and all(h.after[b] == h.before[b] for b in range(6) if b != byte) for h in counting) \
                        and sorted(nib(h.before) for h in kills) == list(range(len(kills))):
                    enemies = {v for k, v in self.params.items() if k.startswith("ENEMY")}
                    keys = []
                    for h in kills:
                        base, layout = h.trigger[1], h.trigger[2]
                        key = self.name(layout, ("ENEMY",)) if layout in enemies else self.name(base, ("ENEMY",))
                        if key not in keys:
                            keys.append(key)
                    groups.append({"sequence": sequence, "handlers": kills, "counter": (byte, high),
                                   "needed": len(kills), "final": final[0] if final else None, "keys": keys,
                                   "by_layout": all(h.trigger[2] in enemies for h in kills)})
                    break
        return groups

    @staticmethod
    def _count_template(group: dict) -> list | None:
        """The notices of the kills on the way, as Lua: each argument the count so far, how many are needed, or a
        fixed number. None when they don't follow one pattern."""
        byte, high = group["counter"]
        template = None
        for h in group["handlers"]:
            if h is group["final"]:
                continue
            count = (h.before[byte] >> 4 if high else h.before[byte] & 0x0F) + 1
            mine = [[notice, ["count" if a == count else str(group["needed"]) if a == group["needed"] else str(a)
                              for a in args]] for notice, args in h.notices]
            if template is None:
                template = mine
            elif template != mine:
                return None
        return template or []

    def _others(self) -> list[str]:
        lines = []
        groups = self.kill_groups()
        grouped = {id(h) for g in groups for h in g["handlers"]}
        for g in groups:
            lines.append(f"-- The enemies of sequence {g['sequence']}, in any order: "
                         f"UI8{'ABCDEF'[g['counter'][0]]}{'H' if g['counter'][1] else 'L'} counts them.")
            lines.append(f"KILLS_{g['sequence']} = {{ {', '.join(f'[{k}] = true' for k in g['keys'])} }}")
            lines.append("")
        for kind, signature in (("kill", "onBNpcKill(base_id, layout_id, player)"),
                                ("duty", "onDutyCleared(content_id, player)")):
            handlers = [h for h in self.handlers if h.trigger[0] == kind and id(h) not in grouped]
            kind_groups = groups if kind == "kill" else []
            if not handlers and not kind_groups:
                continue
            lines.append(f"function {signature}")
            lines.append("    local sequence = player:get_quest_sequence(EVENT_ID)")
            branch = "if"
            for g in kind_groups:
                byte, high = g["counter"]
                key = "layout_id" if g["by_layout"] else "base_id"
                lines.append(f"    {branch} sequence == {g['sequence']} and KILLS_{g['sequence']}[{key}] then")
                branch = "elseif"
                lines.append(f"        local count = getVar(player, {byte}, {str(high).lower()}) + 1")
                if g["final"] is None:
                    # None of them moves the quest on: retail counted all of them (the next step checks the count.)
                    lines.append(f"        if count <= {g['needed']} then")
                else:
                    lines.append(f"        if count >= {g['needed']} then")
                    lines.extend(self._outcome(g["final"], "            "))
                    lines.append("        else")
                lines.append(f"            setVar(player, {byte}, {str(high).lower()}, count)")
                template = self._count_template(g)
                if template is None:
                    lines.append(f"            -- TODO: retail's notices on the way: "
                                 f"{'; '.join(str(h.notices) for h in g['handlers'] if h is not g['final'])}")
                for notice, args in template or []:
                    lines.append(f"            player:event_notice(EVENT_ID, {notice}, {{ {', '.join(args)} }})")
                lines.append("        end")
            for handler in handlers:
                lines.append(f"    {branch} {self.condition(handler)} then")
                branch = "elseif"
                lines.extend(self._outcome(handler, "        "))
            lines.append("    end")
            lines.append("end")
            lines.append("")
        if any(s.bar is not None for h in self.handlers for s in h.scenes):
            lines.append("function onEventActionCast(player)")
            lines.append("    if after_bar ~= nil then")
            lines.append("        local scene = after_bar")
            lines.append("        after_bar = nil")
            lines.append("        play(player, scene.scene, scene.flags)")
            lines.append("        return")
            lines.append("    end")
            lines.append("    player:finish_event()")
            lines.append("end")
            lines.append("")
        return lines

    def _outcome(self, handler: Handler, indent: str) -> list[str]:
        lines = []
        for layout in handler.spawns:
            lines.append(f"{indent}player:spawn_quest_bnpc({self.name(layout, ('ENEMY',))})")
        if handler.completed:
            return lines
        if id(handler) in self.member:
            return lines + self._group_outcome(handler, indent)
        if handler.accepted:
            lines.append(f"{indent}player:accept_quest(EVENT_ID)")
        for effect in handler.effects:
            if "content" in effect:
                lines.append(f"{indent}player:unlock_content("
                             f"{self.param(effect['content'], 'INSTANCEDUNGEON') or effect['content']})")
            elif "aetheryte" in effect:
                lines.append(f"{indent}player:unlock_aetheryte(1, "
                             f"{self.param(effect['aetheryte'], 'AETHERYTE') or effect['aetheryte']})")
            elif "unlock" in effect:
                name = next((n for n, v in self.params.items() if v == effect["unlock"] and "UNLOCK" in n), None)
                lines.append(f"{indent}player:unlock({name or effect['unlock']})")
        for notice, args in handler.notices:
            lines.append(f"{indent}player:event_notice(EVENT_ID, {notice}, {lua_list(args)})")
        if handler.after is not None:
            for byte in range(6):
                if handler.after[byte] != handler.before[byte] or (handler.accepted and handler.after[byte]):
                    lines.append(f"{indent}player:set_quest_var(EVENT_ID, {byte}, 0x{handler.after[byte]:02X}) "
                                 f"-- UI8{'ABCDEF'[byte]}")
        if handler.after_sequence is not None and (handler.after_sequence != handler.sequence or handler.accepted):
            lines.append(f"{indent}player:quest_sequence(EVENT_ID, {handler.after_sequence})")
        return lines

    def _zone_lines(self, handler: Handler, indent: str) -> list[str]:
        """Where retail sent the player when the step ended: into the quest battle, or to a pop range (PrepareZoning,
        then the quest's update, then EventFinish on retail: `finish_event()` then the move here.)"""
        zones = [e["zone"] for e in handler.effects if "zone" in e]
        if handler.joins_battle:
            territory = zones[0] if zones else None
            name = self.param(territory, "TERRITORYTYPE") if territory is not None else None
            return [f"{indent}player:finish_event()",
                    f"{indent}player:join_quest_battle(QUESTBATTLE0, {name or territory or 'TERRITORYTYPE0'})",
                    f"{indent}return"]
        if not zones:
            return []
        zone = zones[0]
        pop = next((self.param(p, "POPRANGE") for p in handler.pop_ranges if self.param(p, "POPRANGE")), None)
        territory = self.param(zone, "TERRITORYTYPE") or zone
        lines = [f"{indent}player:finish_event()"]
        if pop and zone == handler.territory:
            lines.append(f"{indent}player:move_to_pop_range({pop}, true)")
        elif pop:
            lines.append(f"{indent}player:change_territory_pop_range({territory}, {pop})")
        elif handler.arrivals:
            x, y, z, rotation = handler.arrivals[0]
            lines.append(f"{indent}-- Where retail put the player ({'ActorSetPos' if zone == handler.territory else 'ZoneInit'}; "
                         f"no pop range of the quest's.)")
            lines.append(f"{indent}player:change_territory({territory}, {{ x = {x}, y = {y}, z = {z} }}, {rotation})")
        else:
            lines.append(f"{indent}-- TODO: where in {territory} retail put the player (not in the capture)")
            lines.append(f"{indent}player:change_territory({territory})")
        lines.append(f"{indent}return")
        return lines

    def _finished(self) -> list[str]:
        lines = ["function sceneFinished(scene, results, player)", "    if scene ~= current_scene then",
                 "        return", "    end", "    current_scene = nil", ""]
        first = True
        for handler in self.handlers:
            chosen = self._chosen_index(handler)
            for index, scene in enumerate(handler.scenes):
                lines.append(f"    {'if' if first else 'elseif'} step == {handler.number} and scene == {scene.scene} then")
                first = False
                last = index == len(handler.scenes) - 1
                gate = scene.yielded and scene.results[:1] == [1] and (not last or handler.completed
                                                                        or handler.accepted or handler.joins_battle
                                                                        or handler.after is not None)
                if gate:
                    lines.append("        if results[1] ~= 1 then")
                    if chosen == index:
                        lines.append("            -- The reward window closed: the quest stays open.")
                        lines.append("            if player:get_quest_var(EVENT_ID, 0) ~= REWARD_CLOSED then")
                        lines.append("                player:set_quest_var(EVENT_ID, 0, REWARD_CLOSED)")
                        lines.append("            end")
                    lines.append("            player:finish_event()")
                    lines.append("            return")
                    lines.append("        end")
                if chosen == index:
                    lines.append("        chosen = results[2] or 0")
                if not last:
                    if scene.bar is not None:
                        bar = self.param(scene.bar, "EVENT_ACTION") or scene.bar
                        following = handler.scenes[index + 1]
                        lines.append(f"        after_bar = {{ scene = {following.scene}, "
                                     f"flags = {flags_lua(following.flags)} }}")
                        lines.append(f"        player:event_action({bar})")
                    else:
                        lines.append(f"        {self._play(handler, index + 1)}")
                    lines.append("        return")
                    continue
                if scene.bar is not None:
                    # Retail broke the bar off every time: nothing more is known.
                    bar = self.param(scene.bar, "EVENT_ACTION") or scene.bar
                    lines.append(f"        -- TODO: retail's bar ({bar}) was always broken off: what comes after it")
                    lines.append(f"        player:event_action({bar})")
                    lines.append("        return")
                    continue
                if (handler.trigger, handler.sequence, scene.scene) in self.broken:
                    lines.append("        if player.scene_error_code ~= 0 then")
                    lines.append("            -- Broken off (retail's error code): nothing happens.")
                    lines.append("            player:finish_event()")
                    lines.append("            return")
                    lines.append("        end")
                if handler.completed:
                    lines.append("        player:finish_event()")
                    lines.append("        player:finish_quest(EVENT_ID, chosen)")
                    lines.extend(line for line in self._zone_lines(handler, "        ")
                                 if line.strip() not in ("player:finish_event()", "return"))
                    lines.append("        return")
                    continue
                lines.extend(self._outcome(handler, "        "))
                lines.extend(self._zone_lines(handler, "        "))
        if not first:
            lines.append("    end")
        lines.append("")
        lines.append("    player:finish_event()")
        lines.append("end")
        lines.append("")
        return lines


HELPERS = """-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- Which step of the quest this event is (the sequence notes above), the scene after a progress bar, and the reward
-- the player picked (the scenes after it show it.)
step = nil
after_bar = nil
chosen = 0

function play(player, scene, flags, params)
    current_scene = scene
    player:play_scene(scene, flags, params or {})
end""".splitlines()

NIBBLES = """-- Quest variables: UI8A..F are bytes 0..5, H is the high nibble and L the low one.
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
end""".splitlines()

FLAGS = """-- BitFlag8: flag 1 is the top bit of UI8F, as retail sets it.
function hasFlag(player, flag)
    return (player:get_quest_var(EVENT_ID, 5) & (0x80 >> (flag - 1))) ~= 0
end

function setFlag(player, flag)
    player:set_quest_var(EVENT_ID, 5, player:get_quest_var(EVENT_ID, 5) | (0x80 >> (flag - 1)))
end""".splitlines()

FOOTER = """function onYield(scene, id, results, player)
    sceneFinished(scene, results, player)
end

function onReturn(scene, results, player)
    sceneFinished(scene, results, player)
end""".splitlines()


def transcribe(quest: int, dumps: Path, fixture_paths: list[Path], source: str) -> tuple[str, Transcript]:
    dump = fixtures.load_dump(dumps, quest)
    if dump is None:
        raise ValueError(f"no dump of quest {quest} in {dumps} (kawari-questdump dump {quest})")
    loaded = [(Path(p).stem, fixtures.load(Path(p))) for p in fixture_paths]
    draft = Transcript(dump, loaded)
    return draft.write(source), draft
