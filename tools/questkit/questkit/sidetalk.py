"""Side talk: what a quest's NPCs say when talked to outside the quest's steps (`resources/data/quest_side_talks.json`,
written by `questkit sidetalk`; the server's `events/quest/SideTalk.lua` plays them when the quest's own script is
silent).

A quest's NPCs can be talked to in every sequence the client offers them (`IsAcceptEvent`), and most have a line for
it: the soldiers of Operation Aegis's camp before they're saluted, an NPC who has already said their piece, the ones
standing by while another is the objective. A recording has the steps a player took, so a transcribed script answers
those and ends every other talk without a word (Bob, 2026-11-14, of Dressed to Deceive's imperials: "I faintly
recalled NPCs giving dialogue when you talked to them without an emote"; "it may not break the flow, but that's no
excuse to ignore retail behavior if we can implement it").

Which scene is whose isn't written in the client's script, so it's worked out from what is:

* the Quest sheet's listeners (`quest.json`'s `listeners`, kawari-questdump): one row for each NPC in each sequence
  (two when it's talked to and emoted at), and the client's scenes are numbered in the rows' order, sequence by
  sequence;
* the fixtures: a recorded step pins its scenes to its NPC and sequence, and the scenes between two pins belong to
  the sequences between them;
* a scene's speaker, the name in its text constants (`TEXT_MANFST407_00521_IMPERIALSOLDIER_A_000_050`): the NPC a
  recorded scene shows to have it, or the one whose name it is (the Quest sheet's label for the parameter, spelling
  aside: HOUMEI is Homei), or, for a name the developers made up (NPCA), the one NPC of the sequences in question
  that no other name is.

A talk is given a scene when exactly one fits: a plain talk (turning, an animation, lines; no question, cutscene or
reward) by that NPC, in that sequence's place, that the script doesn't play itself. An NPC with nothing to say has an
empty scene there, and stays silent as on retail. What can't be told apart is reported, to be written by hand in the
quest's script (as 521's soldiers were, whose lines change once they're saluted).
"""

from __future__ import annotations

import difflib
import json
import re
from pathlib import Path

from . import repo, travel

OUT = repo.ROOT / "resources" / "data" / "quest_side_talks.json"
# The talks no line could be given to yet, each to be written by hand in its quest's script (`--update-known`).
KNOWN = Path(__file__).resolve().parent.parent / "sidetalk_known.json"
# Retail played this kind of talk with HIDE_HOTBAR (quest 466's, Bob's Chronofoil captures; `quests.md`).
FLAGS = 0x2000
# What a plain talk is made of.
PLAIN = {"Talk", "TurnTo", "WaitForTurn", "PlayActionTimeline", "WaitForActionTimeline", "CancelActionTimeline",
         "LookAt", "WaitForLookAt", "Wait", "Idle", "GetEquippedItem", "GetQuestSequence", "GetQuestId"}
# What a talk that's a step adds to those: a question, a trade, the quest's offer and reward. Still the NPC talked
# to speaking, where a cutscene's first line can be anyone's.
DEALINGS = {"YesNo", "CancelEventScene", "Menu", "SystemTalk", "NpcTrade", "CancelNpcTrade", "SetNpcTradeItem",
            "GetNpcTradeItemInfo", "getNpcTradeItemInfo", "GetBaseId", "Inventory", "QuestOffer", "QuestAccepted",
            "QuestReward", "QuestCompleted"}
TALK, EMOTE = 1, 2
# How alike a speaker's name and an NPC's must be to be the same (HOUMEI/Homei, KIKYOU/Kikyo: 0.91).
ALIKE = 0.85
# A scene numbered this far after the one before it is one of those numbered apart.
APART = 10
PLAYED = re.compile(r"play(?:_scene)?\(\s*(?:player\s*,\s*)?(\d+)\s*,")


def normal(text: str) -> str:
    return re.sub(r"[^A-Z0-9]", "", (text or "").upper())


def quest_sheets(roots) -> dict[str, Path]:
    """Every dump's quest.json, by script name."""
    out: dict[str, Path] = {}
    for root in roots:
        for path in Path(root).rglob("quest.json"):
            try:
                name = json.loads(path.read_text()).get("script") or path.parent.name
            except ValueError:
                continue
            out.setdefault(name, path)
    return out


def blocks_of(rows: list[dict]) -> list[dict]:
    """The listeners' rows as blocks: an NPC in a sequence (its rows there are one block: a talk and an emote, or
    the same row twice), in the rows' order."""
    blocks: list[dict] = []
    seen: dict[tuple[int, int], dict] = {}
    for row in rows:
        key = (row["spawn_sequence"], row["listener"])
        if key in seen:
            seen[key]["kinds"].add(row["quest_uint8a"])
        else:
            seen[key] = {"base": row["listener"], "sequence": row["spawn_sequence"], "kinds": {row["quest_uint8a"]}}
            blocks.append(seen[key])
    return blocks


def pins_of(fixtures: list[dict]) -> dict[int, tuple[int, int | None]]:
    """Each recorded scene's sequence and NPC (None when the step had none): the first step that played it."""
    pins: dict[int, tuple[int, int | None]] = {}
    for fixture in fixtures:
        before = (fixture.get("from") or {}).get("sequence")
        for step in fixture.get("steps", []):
            expect = step.get("expect") or {}
            after = expect.get("sequence", before) if expect.get("accepted", True) is not False else None
            base = step.get("talk") or (step.get("emote") or [None, None])[1]
            for scene in step.get("scenes") or []:
                pins.setdefault(scene["scene"], (before if before is not None else 0, base))
            before = after
    return pins


def is_plain(methods: tuple) -> bool:
    return "Talk" in methods and set(methods) <= PLAIN


def is_theirs(methods: tuple) -> bool:
    """A scene whose speaker is the NPC talked to."""
    return "Talk" in methods and set(methods) <= PLAIN | DEALINGS


def is_npc(base) -> bool:
    return base is not None and 1_000_000 <= base < 2_000_000


def has_scenes(base, objects: bool = True) -> bool:
    """An NPC or object: each of its blocks has a scene at least. Other listeners (enemies, ranges) may have none,
    and in a few quests an object has none either (1190's last row), so those are lined up with `objects` off."""
    return base is not None and 1_000_000 <= base < (3_000_000 if objects else 2_000_000)


def line_up(count: int, blocks: list[dict], allowed, objects: bool = True) -> list[set[int]] | None:
    """Where each of `count` scenes (in order) can be among the blocks (in order), every block that has scenes given
    one at least and `allowed(scene index, block index)` kept: the blocks each scene can be in over every way of
    lining them up, or None when there's no way."""
    width = len(blocks)
    must = [has_scenes(b["base"], objects) for b in blocks]

    def reach(forward: bool) -> list[list[bool]]:
        """table[i][j]: scene i can be in block j with the scenes before it (after it, backward) lined up."""
        table = [[False] * width for _ in range(count)]
        columns = range(width) if forward else range(width - 1, -1, -1)
        previous = None
        for i in (range(count) if forward else range(count - 1, -1, -1)):
            # passed: a block on the way here that the scene before is in (or the edge), with none between that must
            # have a scene.
            passed = previous is None
            for j in columns:
                table[i][j] = allowed(i, j) and (passed or (previous is not None and table[previous][j]))
                if previous is None:
                    passed = passed and not must[j]
                else:
                    passed = table[previous][j] or (passed and not must[j])
            previous = i
        return table

    if count == 0:
        return None if any(must) else []
    forward, backward = reach(True), reach(False)
    out = [{j for j in range(width) if forward[i][j] and backward[i][j]} for i in range(count)]
    return None if any(not options for options in out) else out


def infer(sheet: dict, client: travel.ClientScript, fixtures: list[dict], script_text: str,
          answered: set[int], wanted: list[tuple[int, int]], names: bool = True) -> tuple[dict, list[str], dict]:
    """The scene for each wanted (sequence, NPC): {(sequence, base): scene, or None for an NPC with nothing to say},
    why the others have none, and each scene's block where it's sure ({scene: (sequence, base)}). `names` off leaves
    out the NPCs' names (a test of the rest.)"""
    # Block 0 is the quest's offer (its giver isn't a listener.)
    blocks = [{"base": None, "sequence": 0, "kinds": set()}] + blocks_of(sheet.get("listeners") or [])
    labels = {p["value"]: normal(p.get("label", "")) for p in sheet.get("params", []) if p.get("label")}
    pins = pins_of(fixtures)
    # The scenes numbered on from 0; the ones numbered apart (50, 90, 98 to 100: an offer turned down, an object
    # looked at) aren't in the rows' order.
    scenes = []
    for n in sorted(client.scenes):
        if scenes and n - scenes[-1] >= APART:
            break
        scenes.append(n)
    index = {scene: i for i, scene in enumerate(scenes)}
    plain = {scene: is_plain(client.scenes[scene][0]) for scene in scenes}
    speaker = {scene: client.speaker(scene) if is_theirs(client.scenes[scene][0]) else None
               for scene in scenes}
    played = set(pins) | answered | {int(n) for n in PLAYED.findall(script_text)}
    failed = [f"sequence {s}: {b}: the client's scenes and the Quest sheet's listeners don't line up" for s, b in wanted]

    pinned: dict[int, set[int]] = {}
    for scene, (sequence, base) in pins.items():
        if scene not in index:
            continue
        exact = {j for j, b in enumerate(blocks) if j and b["base"] == base and b["sequence"] == sequence}
        same = {j for j, b in enumerate(blocks) if b["sequence"] == sequence}
        if exact or same:
            pinned[index[scene]] = exact or same
    barred: set[tuple[int, int]] = set()
    only: dict[int, set[int]] = {}

    giver = sheet.get("issuer_start")
    giver = giver.get("value") if isinstance(giver, dict) else giver
    offers = {i for i, scene in enumerate(scenes) if {"QuestOffer", "QuestAccepted"} & set(client.scenes[scene][0])}

    def allowed(i: int, j: int) -> bool:
        if i in pinned and j not in pinned[i]:
            return False
        # The quest's offer is its giver's.
        if i in offers and j and blocks[j]["base"] != giver:
            return False
        if (i, j) in barred or (i in only and j not in only[i]):
            return False
        # A plain talk is an NPC's.
        return not plain[scenes[i]] or i in pinned or is_npc(blocks[j]["base"])

    objects = True
    options = line_up(len(scenes), blocks, allowed)
    if options is None:
        objects = False
        options = line_up(len(scenes), blocks, allowed, objects)
    if options is None:
        return {}, failed, {}
    for _ in range(8):
        # Who speaks under which name, from the scenes that are sure.
        voices: dict[int, set[str]] = {}
        for i, where in enumerate(options):
            scene = scenes[i]
            if len(where) == 1 and plain[scene] and speaker[scene]:
                voices.setdefault(blocks[next(iter(where))]["base"], set()).add(speaker[scene])
        bar = set(barred)
        for i, where in enumerate(options):
            scene = scenes[i]
            if len(where) < 2 or not plain[scene] or not speaker[scene]:
                continue
            for j in where:
                theirs = voices.get(blocks[j]["base"])
                if theirs and speaker[scene] not in theirs:
                    bar.add((i, j))
        if bar != barred:
            saved, barred = barred, bar
            tried = line_up(len(scenes), blocks, allowed, objects)
            if tried is None:
                barred = saved
            else:
                options = tried
                continue
        if not names:
            break
        # A speaker named as one NPC of the quest is (HOUMEI is Homei), where that still lines up.
        changed = False
        for i, where in enumerate(options):
            scene = scenes[i]
            if len(where) < 2 or not speaker[scene] or i in only:
                continue
            alike = {j for j in where if labels.get(blocks[j]["base"]) and difflib.SequenceMatcher(
                None, speaker[scene], labels[blocks[j]["base"]]).ratio() >= ALIKE}
            if alike and alike != where:
                only[i] = alike
                tried = line_up(len(scenes), blocks, allowed, objects)
                if tried is None:
                    del only[i]
                else:
                    options, changed = tried, True
        if not changed:
            break

    sure = {scenes[i]: (blocks[next(iter(w))]["sequence"], blocks[next(iter(w))]["base"])
            for i, w in enumerate(options) if len(w) == 1}
    out, why = {}, []
    for sequence, base in wanted:
        block = next((j for j, b in enumerate(blocks) if j and b["base"] == base and b["sequence"] == sequence
                      and TALK in b["kinds"]), None)
        tag = f"sequence {sequence}: {base}"
        if block is None:
            why.append(f"{tag}: the Quest sheet has no talk with them in this sequence")
            continue
        mine = [scenes[i] for i, w in enumerate(options) if w == {block}]
        maybe = [scenes[i] for i, w in enumerate(options) if block in w and len(w) > 1
                 and client.scenes[scenes[i]][0]]
        lines = [n for n in mine if plain[n] and n not in played]
        recorded = sorted(n for n in mine if n in pins)
        # A neighbour's scene that isn't a plain talk is no rival to the one line that is.
        if maybe and not (len(lines) == 1 and not any(plain[n] and n not in played for n in maybe)):
            why.append(f"{tag}: scenes {maybe} could be theirs or a neighbour's")
        elif recorded and EMOTE in blocks[block]["kinds"] and lines:
            # Lines around an emote's scene: the one before it until it's done, the one after it from then on, and
            # which it is is the script's to tell.
            why.append(f"{tag}: lines {lines} around their emote's scene {recorded}: the script must tell which "
                       "(by hand)")
        elif len(lines) == 1:
            out[(sequence, base)] = lines[0]
        elif lines:
            why.append(f"{tag}: several lines ({lines}): the script must tell which (by hand)")
        elif any(client.scenes[n][0] and n not in played for n in mine):
            other = [n for n in mine if client.scenes[n][0] and n not in played]
            why.append(f"{tag}: scenes {other} are theirs and more than a talk (by hand)")
        else:
            out[(sequence, base)] = None
    return out, why, sure


def build(roots=(travel.DUMPS,), result: dict | None = None) -> dict:
    """Every scripted quest's side talks: {"quests": {quest: [{actor, scenes: {sequence: scene}}]}, "given": n,
    "nothing": NPCs with nothing to say, "left": [why], "undumped": [quests whose dump has no quest.json]}."""
    result = result if result is not None else travel.build(roots)
    found = travel.listings(roots)
    sheets = quest_sheets(roots)
    scripts = travel.scripted_quests()
    wanted: dict[int, list[tuple[int, int]]] = {}
    # A ride the shipped quest_travel.json gives is answered, whether or not these dumps have the quest it's from.
    rides = json.loads(travel.OUT.read_text()) if travel.OUT.exists() else {}
    for quest, sequence, _, base in result["silent_talks"]:
        if not any(e["actor"] == base and str(sequence) in e["scenes"] for e in rides.get(str(quest), [])):
            wanted.setdefault(quest, []).append((sequence, base))
    data, left, undumped, count, nothing = {}, [], [], 0, 0
    for quest, talks in sorted(wanted.items()):
        script = scripts[quest]
        sheet = sheets.get(script.stem)
        if sheet is None:
            undumped.append(quest)
            continue
        client = travel.read_client_script(found[script.stem].read_text(errors="replace"))
        scenes, why, _ = infer(json.loads(sheet.read_text()), client, travel.fixtures_for(quest), script.read_text(),
                               set(result["answered"].get(quest, ())), talks)
        left += [f"quest {quest}: {line}" for line in why]
        by_actor: dict[int, dict[str, int]] = {}
        for (sequence, base), scene in sorted(scenes.items()):
            if scene is None:
                nothing += 1
                continue
            by_actor.setdefault(base, {})[str(sequence)] = scene
            count += 1
        if by_actor:
            data[str(quest)] = [{"actor": base, "scenes": by_actor[base]} for base in sorted(by_actor)]
    return {"quests": data, "given": count, "nothing": nothing, "left": left, "undumped": undumped,
            "talks": result["silent_talks"]}


def left_order(line: str) -> tuple:
    return tuple(int(n) for n in re.findall(r"\d+", line)[:3])


def merged(result: dict, out: Path = OUT) -> dict:
    """The file's quests with this build's over them: a quest whose dump isn't at hand keeps what it had."""
    kept = json.loads(out.read_text()) if out.exists() else {}
    kept.update(result["quests"])
    return dict(sorted(kept.items(), key=lambda kv: int(kv[0])))


def save(quests: dict, out: Path = OUT) -> Path:
    lines = ["{"]
    items = list(quests.items())
    for i, (quest, entries) in enumerate(items):
        lines.append(f"  {json.dumps(quest)}: " + json.dumps(entries, separators=(",", ":"))
                     + ("," if i < len(items) - 1 else ""))
    lines.append("}")
    out.write_text("\n".join(lines) + "\n")
    return out


def played_problems(out: Path = OUT) -> list[str]:
    """Each line of the file played through its quest's script, as the server does: talking to the NPC in that
    sequence (the variables at zero) plays that scene with HIDE_HOTBAR and ends. What doesn't."""
    from . import harness

    problems = []
    scripts = travel.scripted_quests()
    for quest, entries in (json.loads(out.read_text()) if out.exists() else {}).items():
        quest = int(quest)
        if quest not in scripts:
            problems.append(f"quest {quest} has side talks and no script")
            continue
        params = {}
        for path in sorted(repo.FIXTURES.glob(f"{quest}*.json")):
            if re.fullmatch(rf"{quest}(-[\w-]+)?", path.stem):
                params = json.loads(path.read_text()).get("params") or params
        for entry in entries:
            for sequence, scene in entry["scenes"].items():
                tag = f"quest {quest}: sequence {sequence}: {entry['actor']}"
                try:
                    probe = harness.Harness(scripts[quest], quest, params)
                    if int(sequence):
                        probe.world.quests[quest] = harness.Quest(int(sequence), [0] * 6)
                    outcome = probe.talk(entry["actor"], {}, [])
                except Exception as error:
                    problems.append(f"{tag}: {error}")
                    continue
                played = [(p["scene"], p["flags"]) for p in outcome["played"]]
                if played != [(scene, FLAGS)] or not outcome["finished"]:
                    problems.append(f"{tag}: played {played}, not scene {scene}")
    return problems
