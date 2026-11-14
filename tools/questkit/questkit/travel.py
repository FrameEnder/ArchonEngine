"""Quest travel: the NPCs a quest offers to take the player somewhere, in every sequence the client offers them
(`resources/data/quest_travel.json`, written by `questkit travel`; the server's `events/quest/Travel.lua` answers them).

A quest NPC who asks "go there?" (a Talk and a YesNo) often stands where the quest can be left from: Slafborn, outside
Cid's workshop in Every Little Thing She Does Is Magitek (1005), and the airship ticketers in Lady of the Vortex (519)
and Reclamation (520). The client's script offers him in several sequences, sequence 0 (before it's accepted)
included, each with its own scene, all the same scene (1005's 2, 4, 8 and 51). A recording catches one of them, so a
transcribed script answers only that one, and a player who leaves before the quest is accepted, or comes back later,
can't get back in (Bob's bundle 20261114-024122; the ticketers' 20261111-101422, fixed then quest by quest).

From the client's script (`quest-dumps/<script>/<script>.txt`, kawari-questdump's listing) and the quest's fixtures:

* every scene's methods and constants (`OnSceneNNNNN`): scenes with the same are copies of one another;
* which NPCs the client accepts a talk with in each sequence (`IsAcceptEvent`: its SEQ_n and ACTORn in order);
* a fixture step where talking to an NPC played one scene with a YesNo, the player said yes and was moved (`effects`
  with a `zone`, and `arrivals`) without the quest moving on: where retail took them.

Each copy of that scene is given to a sequence the NPC is accepted in where no recording shows another scene of
theirs, in order (scene numbers follow the sequences). The NPC then takes the player to the same place in each.
Questions with no recording of where they lead are reported, not guessed.
"""

from __future__ import annotations

import json
import re
from dataclasses import dataclass, field
from pathlib import Path

from . import repo

OUT = repo.ROOT / "resources" / "data" / "quest_travel.json"
DUMPS = repo.ROOT / "quest-dumps"
SCENE = re.compile(r"^== \w+:OnScene(\d+)\s")
FUNCTION = re.compile(r"^== \w+:(\w+)\s")
CONSTANT = re.compile(r'R0\["(SEQ_\w+|ACTOR\d+)"\]')
# A Talk's text constant names its speaker: TEXT_GAIUSC605_01005_SLAFBORN_000_005.
SPEAKER = re.compile(r"^TEXT_[A-Z0-9]+_\d{5}_([A-Z0-9]+)_\d+_\d+$")
NOT_SPEAKERS = re.compile(r"^(Q|A)\d+$|^SYSTEM$")


@dataclass
class ClientScript:
    scenes: dict[int, tuple] = field(default_factory=dict)  # scene -> (methods, constants)
    accepted: dict[int, list[str]] = field(default_factory=dict)  # sequence -> ACTORn accepted

    def copies(self, scene: int) -> list[int]:
        signature = self.scenes.get(scene)
        return sorted(n for n, s in self.scenes.items() if s == signature) if signature else []

    def speaker(self, scene: int) -> str | None:
        """Who says the scene's first line (its first text constant naming someone)."""
        for name in self.scenes.get(scene, ((), ()))[1]:
            match = SPEAKER.match(name)
            if match and not NOT_SPEAKERS.match(match.group(1)):
                return match.group(1)
        return None

    def asks(self, scene: int) -> bool:
        methods = self.scenes.get(scene, ((), ()))[0]
        return "YesNo" in methods and "Talk" in methods


def sequence_number(name: str) -> int | None:
    if name == "SEQ_FINISH":
        return 255
    match = re.fullmatch(r"SEQ_(\d+)", name)
    return int(match.group(1)) if match else None


def read_client_script(text: str) -> ClientScript:
    script = ClientScript()
    current, block, lines = None, None, []

    def close():
        if block is None:
            return
        if isinstance(block, int):
            methods = next((l.split(None, 1)[1] for l in lines if l.strip().startswith("methods")), "")
            fields = next((l.split(None, 1)[1] for l in lines if l.strip().startswith("fields")), "")
            script.scenes[block] = (tuple(m.strip() for m in methods.split(",") if m.strip()),
                                    tuple(f.strip() for f in fields.split(",") if f.strip()))
        elif block == "IsAcceptEvent":
            sequence = None
            for line in lines:
                for name in CONSTANT.findall(line):
                    if name.startswith("SEQ_"):
                        sequence = sequence_number(name)
                    elif sequence is not None:
                        actors = script.accepted.setdefault(sequence, [])
                        if name not in actors:
                            actors.append(name)

    for line in text.splitlines():
        if line.startswith("== "):
            close()
            scene = SCENE.match(line)
            function = FUNCTION.match(line)
            block = int(scene.group(1)) if scene else (function.group(1) if function else None)
            lines = []
        elif block is not None:
            lines.append(line)
    close()
    return script


def recorded_travel(fixture: dict) -> list[dict]:
    """The fixture's steps where talking to an NPC played one scene, the player said yes and was moved to another
    zone, and nothing else happened (the sequence, variables and notices as before): {index, actor, base, sequence,
    scene, flags}."""
    out = []
    before, before_vars = None, None
    for index, step in enumerate(fixture.get("steps", [])):
        expect = step.get("expect") or {}
        accepted = expect.get("accepted", True) is not False
        after = expect.get("sequence", before) if accepted else None
        after_vars = expect.get("vars", before_vars) if accepted else None
        scenes = step.get("scenes") or []
        zones = [e["zone"] for e in step.get("effects", []) if isinstance(e, dict) and "zone" in e]
        what = step.get("what") or ""
        if ("talk" in step and len(scenes) == 1 and zones and (scenes[0].get("results") or [0])[0] == 1
                and after == before and after_vars == before_vars and not expect.get("notices")
                and what.startswith("ACTOR")):
            out.append({"index": index, "actor": what, "base": step["talk"],
                        "sequence": before if before is not None else 0, "scene": scenes[0]["scene"],
                        "flags": int(str(scenes[0].get("flags", 0)), 0)})
        before, before_vars = after, after_vars
    return out


def fixtures_for(quest: int) -> list[dict]:
    out = []
    for path in sorted(repo.FIXTURES.glob(f"{quest}*.json")):
        if re.fullmatch(rf"{quest}(-[\w-]+)?", path.stem):
            fixture = json.loads(path.read_text())
            # A fixture of a ride this module gave (`"travel": true`) isn't a recording of one.
            if not fixture.get("travel"):
                out.append(fixture)
    return out


def talked_scenes(fixtures: list[dict]) -> dict[str, set[int]]:
    """For each ACTORn, the sequences a recording shows them talked to in (any scene)."""
    out: dict[str, set[int]] = {}
    for fixture in fixtures:
        before = None
        for step in fixture.get("steps", []):
            expect = step.get("expect") or {}
            after = expect.get("sequence", before) if expect.get("accepted", True) is not False else None
            what = step.get("what") or ""
            if "talk" in step and what.startswith("ACTOR"):
                out.setdefault(what, set()).add(before if before is not None else 0)
            before = after
    return out


def assign(copies: list[int], free: list[int]) -> tuple[dict[int, int], list[int]]:
    """Copies of a question to the sequences that have none (both sorted): one each in order when they're as many,
    else each sequence the first copy numbered after those given to the sequences before it (scene numbers follow the
    sequences). The sequences left without one."""
    if len(copies) == len(free):
        return dict(zip(free, copies)), []
    scenes, left = {}, []
    for sequence in free:
        floor = max([n for s, n in scenes.items() if s < sequence] or [-1])
        candidates = [c for c in copies if c > floor and c not in scenes.values()]
        if candidates:
            scenes[sequence] = candidates[0]
        else:
            left.append(sequence)
    return scenes, left


def recorded_scenes(fixtures: list[dict]) -> set[int]:
    """Every scene a recording shows played."""
    return {scene["scene"] for f in fixtures for step in f.get("steps", []) for scene in step.get("scenes") or []}


def scripted_quests() -> dict[int, Path]:
    out = {}
    for path in repo.QUEST_SCRIPTS.rglob("*.lua"):
        match = re.search(r"_(\d{5})$", path.stem)
        if match:
            out[int(match.group(1))] = path
    return out


def listings(roots) -> dict[str, Path]:
    """Every client script listing (`<script>.txt`) under the dump folders, by script name (the first root that has
    one wins.)"""
    out: dict[str, Path] = {}
    for root in roots:
        for path in Path(root).rglob("*_0*.txt"):
            if re.fullmatch(r"[A-Za-z]+\d+_\d{5}", path.stem):
                out.setdefault(path.stem, path)
    return out


def answers(script: Path, quest: int, params: dict, sequence: int, base: int) -> list[int] | None:
    """The scenes the quest's own script plays when the player talks to `base` in `sequence` (0: not accepted), the
    variables at zero and every question answered no (Travel.lua left out). None when the probe can't tell (a Lua
    error, a talk that needs more than that)."""
    from . import harness as harness_module

    try:
        probe = harness_module.Harness(script, quest, params)
        if sequence:
            probe.world.quests[quest] = harness_module.Quest(sequence, [0] * 6)
        saved, harness_module.QUEST_TRAVEL = harness_module.QUEST_TRAVEL, Path("/nonexistent")
        try:
            outcome = probe.talk(base, {}, [])
        finally:
            harness_module.QUEST_TRAVEL = saved
    except Exception:
        return None
    return [played["scene"] for played in outcome.get("played", [])]


def rides(quests: dict[int, tuple]) -> tuple[dict[int, list], dict[int, dict]]:
    """Every recorded ride, replayed through its quest's script for the move it makes: by quest, [(travel, move)];
    and by NPC (ENpc base), the first ride seen ({move, flags, quest})."""
    from . import replay

    by_quest: dict[int, list] = {}
    by_npc: dict[int, dict] = {}
    for quest, (script, client, fixtures) in sorted(quests.items()):
        for fixture in fixtures:
            travels = [t for t in recorded_travel(fixture) if client.asks(t["scene"])]
            if not travels:
                continue
            result = replay.run(fixture, script)
            steps = {step.index: step for step in result.steps}
            for travel in travels:
                step = steps.get(travel["index"])
                moves = step.outcome.get("moves", []) if step is not None and step.ok else []
                if not moves:
                    continue
                by_quest.setdefault(quest, []).append((travel, moves[0]))
                by_npc.setdefault(travel["base"], {"move": moves[0], "flags": travel["flags"], "quest": quest,
                                                   "speaker": client.speaker(travel["scene"])})
    return by_quest, by_npc


def build(roots=(DUMPS,)) -> dict:
    found = listings(roots)
    quests = {}
    for quest, script in sorted(scripted_quests().items()):
        listing = found.get(script.stem)
        if listing is not None:
            quests[quest] = (script, read_client_script(listing.read_text(errors="replace")), fixtures_for(quest))
    by_quest, by_npc = rides(quests)

    data, unknown, questions, inferred, silent = {}, [], [], [], []
    for quest, (script, client, fixtures) in sorted(quests.items()):
        params = next((f.get("params") for f in fixtures if f.get("params")), {})
        # What the script answers: each NPC the client accepts a talk with, in each sequence.
        played: dict[tuple[str, int], list[int] | None] = {}
        for sequence, actors in sorted(client.accepted.items()):
            for actor in actors:
                if actor in params:
                    played[(actor, sequence)] = answers(script, quest, params, sequence, params[actor])
        answered_scenes = {n for scenes in played.values() if scenes for n in scenes}

        def unanswered(actor: str) -> list[int]:
            return sorted(s for (a, s), scenes in played.items() if a == actor and scenes == [])

        entries, given = [], set()
        # The NPCs whose ride this quest's recordings show: their question's copies where the script is silent.
        for travel, move in by_quest.get(quest, []):
            actor = travel["actor"]
            if actor in given:
                continue
            given.add(actor)
            copies = [c for c in client.copies(travel["scene"]) if c not in answered_scenes]
            scenes, left = assign(copies, unanswered(actor))
            unknown += [f"quest {quest}: {actor} ({travel['base']}) is offered in sequence {s} with no copy of its "
                        f"question (scene {travel['scene']}) left" for s in left]
            if scenes:
                entries.append({"actor": travel["base"], "scenes": {str(k): v for k, v in sorted(scenes.items())},
                                "flags": travel["flags"], "move": move})
        # Questions the script never asks: the NPC's ride in another quest, when it's a known one.
        groups: dict[tuple, list[int]] = {}
        for scene in sorted(client.scenes):
            if client.asks(scene):
                groups.setdefault(client.scenes[scene], []).append(scene)
        covered = answered_scenes | {n for e in entries for n in e["scenes"].values()}
        for copies in groups.values():
            free_copies = [c for c in copies if c not in covered]
            if not free_copies or (len(copies) < 2 and not answered_scenes.isdisjoint(copies)):
                continue
            speaker = client.speaker(copies[0])
            known = [(name, params[name]) for name in sorted({a for s in client.accepted.values() for a in s})
                     if name in params and params[name] in by_npc and name not in given and unanswered(name)
                     and speaker is not None and by_npc[params[name]]["speaker"] == speaker]
            if len(known) != 1:
                if len(copies) >= 2:
                    questions.append(f"quest {quest}: scenes {free_copies} of {copies} ({speaker or 'no speaker'}), "
                                     "one question (Talk, YesNo), are never played and no recording shows where "
                                     "they lead")
                continue
            actor, base = known[0]
            ride = by_npc[base]
            scenes, left = assign(free_copies, unanswered(actor))
            if scenes:
                given.add(actor)
                entries.append({"actor": base, "scenes": {str(k): v for k, v in sorted(scenes.items())},
                                "flags": ride["flags"], "move": ride["move"], "from": ride["quest"]})
                inferred.append(f"quest {quest}: {actor} ({base}) scenes {scenes} take the player where its ride in "
                                f"quest {ride['quest']} did")
        if entries:
            data[str(quest)] = entries
        # Talks the client offers that the script doesn't answer (and no ride does).
        for (actor, sequence), scenes in sorted(played.items(), key=lambda kv: (kv[0][1], kv[0][0])):
            if scenes == [] and not any(e["actor"] == params[actor] and str(sequence) in e["scenes"]
                                        for e in entries):
                silent.append(f"quest {quest}: sequence {sequence}: {actor} ({params[actor]})")
    return {"quests": data, "unknown": unknown, "questions": questions, "inferred": inferred, "silent": silent}


def save(result: dict, out: Path = OUT) -> Path:
    lines = ["{"]
    quests = list(result["quests"].items())
    for i, (quest, entries) in enumerate(quests):
        lines.append(f"  {json.dumps(quest)}: " + json.dumps(entries, separators=(",", ":"))
                     + ("," if i < len(quests) - 1 else ""))
    lines.append("}")
    out.write_text("\n".join(lines) + "\n")
    return out
