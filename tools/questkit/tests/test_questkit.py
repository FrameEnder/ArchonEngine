"""Tests for questkit, on made-up data only (no game files): python3 -m unittest discover tools/questkit/tests"""

import gzip
import json
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from questkit import battle, battle_sim, capture, chronofoil, chronofoil_battle, chronofoil_quest, compare, duty_sim, duty_sources, fixtures, ledger, ledger_facts, lint, public_sources, recording, replay, rotation_sim, scaffold, transcribe, triage  # noqa: E402
from questkit import autoduty, fishing_data, questionable  # noqa: E402
from questkit.bundle import Bundle  # noqa: E402

QUEST = 9999
HANDLER = 0x10000 + QUEST

# A small quest in the fork's pattern: ACTOR0 offers it, ACTOR1 moves it on with a notice, two objects are searched in
# any order (UI8AL counts, BitFlag8 each, notice [count, 2]), then ACTOR0 gives the reward.
SCRIPT = """
SCENE_OFFER = 0
SCENE_ACTOR1 = 1
SCENE_REWARD = 5
OBJECTS = { [EOBJECT0] = { scene = 2, flag = 1 }, [EOBJECT1] = { scene = 3, flag = 2 } }
current_scene = nil
searching = nil
function play(player, scene, flags) current_scene = scene; player:play_scene(scene, flags, {}) end
function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_ACTOR1, HIDE_HOTBAR)
    elseif sequence == 2 and OBJECTS[BASE_ID] ~= nil
            and (player:get_quest_var(EVENT_ID, 5) & (0x80 >> (OBJECTS[BASE_ID].flag - 1))) == 0 then
        searching = OBJECTS[BASE_ID]
        player:event_action(EVENT_ACTION_SEARCH)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end
function onEventActionCast(player) play(player, searching.scene, 0) end
function sceneFinished(scene, results, player)
    if scene ~= current_scene then return end
    current_scene = nil
    if scene == SCENE_OFFER and results[1] == 1 then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_ACTOR1 then
        player:event_notice(EVENT_ID, 0)
        player:quest_sequence(EVENT_ID, 2)
    elseif searching ~= nil and scene == searching.scene then
        local count = (player:get_quest_var(EVENT_ID, 0) & 0x0F) + 1
        player:event_notice(EVENT_ID, 1, { count, 2 })
        if count >= 2 then
            for byte = 0, 5 do player:set_quest_var(EVENT_ID, byte, 0) end
            player:quest_sequence(EVENT_ID, 255)
        else
            player:set_quest_var(EVENT_ID, 0, count)
            player:set_quest_var(EVENT_ID, 5, player:get_quest_var(EVENT_ID, 5) | (0x80 >> (searching.flag - 1)))
        end
    elseif scene == SCENE_REWARD and results[1] == 1 then
        player:finish_quest(EVENT_ID, results[2])
    end
    player:finish_event()
end
function onYield(scene, id, results, player) sceneFinished(scene, results, player) end
function onReturn(scene, results, player) sceneFinished(scene, results, player) end
"""

PARAMS = {"ACTOR0": 1001, "ACTOR1": 1002, "EOBJECT0": 2001, "EOBJECT1": 2002, "EVENT_ACTION_SEARCH": 1}

# What "retail" did, as the Recorder writes it.
T0 = "2026-10-14T20:00:00.0000000Z"


def at(seconds: float) -> str:
    return f"2026-10-14T20:{int(seconds) // 60:02}:{seconds % 60:09.6f}Z".replace(" ", "0") + ""


def event(seconds, kind, sequence, variables, base=0, layout=0, kind_target="EventNpc"):
    return {"Time": at(seconds), "Kind": kind, "Sequence": sequence, "Variables": variables, "TerritoryId": 148,
            "Target": "someone", "TargetBaseId": base, "TargetLayoutId": layout, "TargetKind": kind_target,
            "TargetSecondsAgo": 0.2}


RETAIL_EVENTS = [
    event(1, "accepted", 1, "000000000000", 1001, 1),
    event(10, "sequence", 2, "000000000000", 1002, 2),
    event(20, "variables", 2, "010000000080", 2001, 3, "EventObj"),
    event(30, "sequence", 255, "000000000000", 2002, 4, "EventObj"),
    event(40, "completed", 255, "000000000000", 1001, 1),
]


def notice_hex(notice, args):
    data = HANDLER.to_bytes(4, "little") + bytes([notice, len(args), 0, 0])
    for value in args:
        data += value.to_bytes(4, "little")
    return data.ljust(48, b"\0").hex()


RETAIL_PACKETS = [
    {"Time": at(0.5), "Opcode": 769, "Handler": f"quest {QUEST}", "Hex": "00" * 48},
    {"Time": at(9.9), "Opcode": 386, "Handler": f"quest {QUEST}", "Hex": notice_hex(0, [])},
    {"Time": at(19.9), "Opcode": 386, "Handler": f"quest {QUEST}", "Hex": notice_hex(1, [1, 2])},
    {"Time": at(29.9), "Opcode": 386, "Handler": f"quest {QUEST}", "Hex": notice_hex(1, [2, 2])},
    {"Time": at(45), "Opcode": 769, "Handler": f"quest {QUEST}", "Hex": "00" * 48},
]

LISTING = """; scenes: X:OnScene00000
== X:OnScene00000  [function 0.0.0, 3 params, 0 upvalues, stack 7, 28 instructions]
   methods  TurnTo, QuestOffer, Talk, QuestAccepted
   fields   TEXT_SUBTST001_09999_ALPHA_000_1
== X:OnScene00001  [function 0.0.1, 3 params, 0 upvalues, stack 7, 20 instructions]
   methods  TurnTo, Talk
   fields   TEXT_SUBTST001_09999_BETA_000_1
== X:OnScene00002  [function 0.0.2, 3 params, 0 upvalues, stack 3, 1 instructions]
== X:OnScene00003  [function 0.0.3, 3 params, 0 upvalues, stack 3, 1 instructions]
== X:OnScene00004  [function 0.0.4, 3 params, 0 upvalues, stack 3, 1 instructions]
== X:OnScene00005  [function 0.0.5, 3 params, 0 upvalues, stack 9, 36 instructions]
   methods  Talk, QuestReward, QuestCompleted
   fields   TEXT_SUBTST001_09999_ALPHA_000_2
== X:GetEventItems  [function 0.0.6, 2 params, 0 upvalues, stack 9, 3 instructions]
"""


class QuestionableTest(unittest.TestCase):
    """`questkit questionable`: a quest script against a Questionable path, made up here (Questionable's files hold
    game text and stay out of git)."""

    def path(self, folder: Path) -> Path:
        steps = {
            "QuestSequence": [
                {"Sequence": 0, "Steps": [{"DataId": 1000001, "InteractionType": "AcceptQuest", "TerritoryId": 1,
                                           "Position": {"X": 1, "Y": 2, "Z": 3}}]},
                {"Sequence": 1, "Steps": [
                    {"DataId": 1000009, "InteractionType": "AcceptQuest", "PickUpQuestId": 9},
                    {"DataId": 2000002, "InteractionType": "UseItem", "ItemId": 2000500,
                     "CompletionQuestVariablesFlags": [None, None, None, None, None, 128], "$": "QW: 16 0 0 0 0 128"},
                    {"InteractionType": "UseItem", "ItemId": 4868},
                    {"DataId": 2000003, "InteractionType": "Interact", "TargetTerritoryId": 2}]},
                {"Sequence": 3, "Steps": [{"DataId": 1000004, "InteractionType": "Interact"}]},
                {"Sequence": 255, "Steps": [{"DataId": 1000001, "InteractionType": "CompleteQuest"}]},
            ]
        }
        (folder / "QuestPaths" / "x").mkdir(parents=True)
        (folder / "QuestPaths" / "x" / "4000_Made Up.json").write_text("\ufeff" + json.dumps(steps), encoding="utf-8")
        return folder

    def test_finds_what_the_script_leaves_out(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder = self.path(Path(tmp))
            paths = questionable.read_paths(folder)
            self.assertEqual(paths[4000].sequences, [0, 1, 3, 255])
            script = folder / "Made_04000.lua"
            script.write_text("-- Sequence 0: ACTOR0 offers it.\n-- Sequences 1, 2: EOBJECT0 with ITEM0.\n"
                              "-- Sequence 255: ACTOR0.\n\nfunction onTalk() return ACTOR0, EOBJECT0, ITEM0 end\n")
            dumps = folder / "dumps" / "Made_04000"
            dumps.mkdir(parents=True)
            (dumps / "quest.json").write_text(json.dumps({"params": [
                {"name": "ACTOR0", "value": 1000001}, {"name": "ACTOR1", "value": 1000004},
                {"name": "EOBJECT0", "value": 2000002}, {"name": "ITEM0", "value": 2000500}]}))
            findings, targets = questionable.check_quest(4000, script, paths[4000], folder / "dumps")
        self.assertTrue(targets)
        kinds = sorted((f.kind, f.detail.split(":")[0]) for f in findings)
        # ACTOR1 in sequence 3 is never used, and sequence 3 isn't in the header; the other quest's NPC, the door,
        # and the item used on nothing aren't this quest's.
        self.assertEqual(kinds, [("sequence", "Questionable goes through sequence 3; the script's header doesn't "
                                              "speak of it (it does of [0, 1, 2, 255])"),
                                 ("target unused", "sequence 3")])

    def test_header_sequences(self):
        text = "-- Sequences 1, 3, 5: soil.\n-- Sequences 2-4: talk.\n-- Sequence 255: reward.\nx = 1\n-- Sequence 9\n"
        self.assertEqual(questionable.header_sequences(text), {1, 2, 3, 4, 5, 255})

    def test_hints_keep_numbers_only(self):
        with tempfile.TemporaryDirectory() as tmp:
            hints = questionable.retail_hints(self.path(Path(tmp)), 4000)
        self.assertIn("sequence 1: UseItem 2000002 (item 2000500; variables after [None, None, None, None, None, 128]; "
                      "QW: 16 0 0 0 0 128)", hints)
        self.assertIn("sequence 0: AcceptQuest 1000001 at (1.0, 2.0, 3.0) in 1", hints)


class AutoDutyTest(unittest.TestCase):
    """`questkit autoduty`: a made-up path in AutoDuty's format (AutoDuty is unlicensed and stays out of git)."""

    def write(self, folder: Path) -> Path:
        actions = [
            {"Tag": "None", "Name": "Interactable", "Position": {"X": 1.0, "Y": 2.0, "Z": 3.0},
             "Arguments": ["2000250"], "Note": ""},
            {"Tag": "None", "Name": "ConditionAction", "Position": {"X": 0.0, "Y": 0.0, "Z": 0.0},
             "Arguments": ["ItemCount;2000512;<;1&ModifyIndex;7"], "Note": ""},
            {"Tag": "None", "Name": "Interactable", "Position": {"X": 4.0, "Y": 5.0, "Z": 6.0},
             "Arguments": ["2000227"], "Note": ""},
            {"Tag": "None", "Name": "Boss", "Position": {"X": 7.0, "Y": 8.0, "Z": 9.0}, "Arguments": ["<-31,24,63>"]},
            {"Tag": "Treasure", "Name": "TreasureCoffer", "Position": {"X": 10.0, "Y": 0.0, "Z": 0.0},
             "Arguments": [""]},
        ]
        paths = folder / "AutoDuty" / "Paths"
        paths.mkdir(parents=True)
        (paths / "(4321) Made Up.json").write_text("\ufeff" + json.dumps({"Actions": actions, "Meta": {}}),
                                                  encoding="utf-8")
        (paths / "(4321) 「Tank W2W」 Made Up.json").write_text(json.dumps({"actions": [], "meta": {}}))
        return folder

    def test_reads_a_route(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder = self.write(Path(tmp))
            path = autoduty.read_paths(folder)[4321][0]
            self.assertEqual(path.file, "(4321) Made Up.json", "the main path before the variants")
            self.assertEqual(path.eobjs, [2000250, 2000227])
            self.assertEqual(path.key_items, [2000512])
            draft = autoduty.draft(folder, 4321)
        self.assertIn("* object 2000250 at (1.0, 2.0, 3.0)", draft)
        self.assertIn("* needs key item 2000512 here", draft)
        self.assertIn("* boss 1 fought at (7.0, 8.0, 9.0), arena <-31,24,63>", draft)
        self.assertIn("* treasure coffer at (10.0, 0.0, 0.0)", draft)

    def test_every_dungeon_in_the_table(self):
        """The check finds the fork's duty scripts in dungeons.md's table, each one that exists."""
        duties = autoduty.fork_duties()
        self.assertEqual(duties[1036].name, "s1d1_re.lua")
        self.assertIn(1041, duties)
        self.assertIn(1042, duties)
        for script in duties.values():
            self.assertTrue(script.exists(), script)


class FishingDataTest(unittest.TestCase):
    """`questkit fishing`: AutoHook's fish data, made up here, into fishing.json's shape."""

    def test_extract(self):
        with tempfile.TemporaryDirectory() as tmp:
            data = Path(tmp) / "AutoHook" / "Data" / "FishData"
            data.mkdir(parents=True)
            (data / "spot_pools.json").write_text(json.dumps({"Spots": {"7": {
                "Catches": [[2586, 4927, 0, 0, 10], [2586, -1, 0, 0, 3], [2586, 4925, 1, 0, 50], [2586, 9999, 0, 0, 1]],
                "BiteTimes": [[2586, 4927, 5, 1, 10, 8, 20, 1]]}}}))
            (data / "fish_list.json").write_text(json.dumps([
                {"ItemId": 4927, "BiteType": 36, "HookType": 4179, "InitialBait": 2586, "Mooches": [],
                 "Predators": [{"ItemId": 1, "Quantity": 2}], "Weathers": [2], "WeathersFrom": [], "Spawn": 18.5,
                 "Duration": 9, "MinGathering": None, "Snagging": False, "IsSpearFish": False},
                {"ItemId": 5000, "BiteType": 0, "HookType": 0, "IsSpearFish": True}]))
            (data / "fish_tracker.json").write_text(json.dumps({"Fish": {"4927": {"FishEyes": True,
                                                                                  "IntuitionSeconds": 300}}}))
            out = Path(tmp) / "fishing.json"
            made = fishing_data.build(tmp, out)
            result = json.loads(out.read_text())
        self.assertEqual(made, {"spots": 1, "fish": 1})
        # No lures, nothing caught once; the bite times' minimum, tenth, median, ninetieth and maximum.
        self.assertEqual(result["spots"]["7"]["2586"], [[4927, 10, [5.0, 5.0, 10.0, 10.0, 20.0]], [-1, 3, None]])
        self.assertEqual(result["fish"]["4927"], {"tug": 36, "hookset": 4179, "bait": 2586, "predators": [[1, 2]],
                                                  "weathers": [2], "minutes": [1110, 210], "fish_eyes": True,
                                                  "intuition": 300})


class Fixtures(unittest.TestCase):
    def setUp(self):
        self.dir = Path(tempfile.mkdtemp())
        rec = self.dir / "rec"
        rec.mkdir()
        (rec / "quests.json").write_text(json.dumps(
            {"Version": 1, "Quests": {str(QUEST): {"Name": "Test", "Events": RETAIL_EVENTS}}}))
        (rec / "packets.json").write_text(json.dumps(
            {"Version": 1, "GameVersion": "2026.10.06.0000.0000", "Timeline": RETAIL_PACKETS}))
        self.recordings = recording.Recordings.open([rec])
        dumps = self.dir / "dumps" / "SubTst001_09999"
        dumps.mkdir(parents=True)
        self.dump = {"quest_id": QUEST, "event_id": HANDLER, "script": "SubTst001_09999", "name": "Test",
                     "params": [{"name": k, "value": v, "label": {"ACTOR0": "Alpha", "ACTOR1": "Beta"}.get(k, "")}
                                for k, v in PARAMS.items()]}
        (dumps / "quest.json").write_text(json.dumps(self.dump))
        (dumps / "SubTst001_09999.txt").write_text(LISTING)
        self.dumps = self.dir / "dumps"
        self.script = self.dir / "SubTst001_09999.lua"
        self.script.write_text(SCRIPT)

    def test_time_and_notice(self):
        self.assertAlmostEqual(recording.parse_time("2026-10-14T20:00:00.1234567Z") % 60, 0.123456, places=5)
        self.assertEqual(recording.parse_notice(bytes.fromhex(notice_hex(1, [3, 4]))), (HANDLER, 1, [3, 4]))

    def test_notices_from_recording(self):
        notices, covered = self.recordings.notices(QUEST)
        self.assertTrue(covered)
        self.assertEqual([n.as_list() for n in notices], [[0, []], [1, [1, 2]], [1, [2, 2]]])

    def test_notices_under_a_stale_game_version(self):
        # A Recorder export whose packets.json kept an older GameVersion than its packets (Bob's of 2026-11-12 said
        # 2026.10.06 with 7.58's notices, 633): the version whose opcodes fit its quest packets wins.
        rec = self.dir / "stale"
        rec.mkdir()
        (rec / "quests.json").write_text(json.dumps(
            {"Version": 1, "Quests": {str(QUEST): {"Name": "Test", "Events": RETAIL_EVENTS}}}))
        packets = [dict(p, Opcode=633) if p["Opcode"] == 386 else p for p in RETAIL_PACKETS]
        (rec / "packets.json").write_text(json.dumps(
            {"Version": 1, "GameVersion": "2026.10.06.0000.0000", "Timeline": packets}))
        notices, covered = recording.Recordings.open([rec]).notices(QUEST)
        self.assertTrue(covered)
        self.assertEqual([n.as_list() for n in notices], [[0, []], [1, [1, 2]], [1, [2, 2]]])

    def test_build_and_replay(self):
        fixture = fixtures.build(QUEST, self.recordings, self.dump)
        self.assertEqual([("talk" in s, s["expect"].get("sequence")) for s in fixture["steps"]],
                         [(True, 1), (True, 2), (True, 2), (True, 255), (True, None)])
        self.assertEqual(fixture["steps"][1]["expect"]["notices"], [[0, []]])
        result = replay.run(fixture, self.script)
        self.assertTrue(result.ok, replay.format_results([result]))

    def test_battle_steps(self):
        # A quest battle: the director wins it in TERRITORYTYPE0 (while the player hits something), then coming back
        # to the zone plays a scene that moves the quest on.
        events = [
            event(1, "accepted", 1, "000000000000", 1001, 1),
            dict(event(20, "sequence", 2, "000000000000", 933, 4077147, "BattleNpc"), TerritoryId=270),
            dict(event(40, "sequence", 255, "000000000000"), TerritoryId=141, TargetSecondsAgo=None),
            event(60, "completed", 255, "000000000000", 1001, 1),
        ]
        rec = self.dir / "battle"
        rec.mkdir()
        (rec / "quests.json").write_text(json.dumps(
            {"Version": 1, "Quests": {str(QUEST): {"Name": "Test", "Events": events}}}))
        dump = dict(self.dump, params=self.dump["params"] + [{"name": "QUESTBATTLE0", "value": 37},
                                                             {"name": "TERRITORYTYPE0", "value": 270}])
        fixture = fixtures.build(QUEST, recording.Recordings.open([rec]), dump)
        self.assertEqual([s.get("battle_won") or s.get("enter_territory") or s.get("talk") for s in fixture["steps"]],
                         [1001, 2, 141, 1001])

    def test_layout_and_yield(self):
        # The script reads the talked-to object's layout ID, and the scene comes back as a yield (onYield, not
        # onReturn, even though the script has both.)
        script = self.dir / "layout.lua"
        script.write_text(SCRIPT.replace(
            "function onYield(scene, id, results, player) sceneFinished(scene, results, player) end",
            "function onYield(scene, id, results, player) if target_layout == 77 then "
            "sceneFinished(scene, results, player) end end").replace(
            "function onTalk(target, player)", "function onTalk(target, player)\n    target_layout = target.object_id"
        ).replace(
            "function onReturn(scene, results, player) sceneFinished(scene, results, player) end",
            "function onReturn(scene, results, player) player:finish_event() end"))
        fixture = fixtures.build(QUEST, self.recordings, self.dump)
        fixture["steps"] = fixture["steps"][:1]
        fixture["steps"][0]["layout"] = 77
        fixture["steps"][0]["results"] = {"0": {"results": [1, 0], "yield": True}}
        result = replay.run(fixture, script)
        self.assertTrue(result.ok, replay.format_results([result]))
        fixture["steps"][0]["layout"] = 78
        self.assertFalse(replay.run(fixture, script).ok)

    def test_replay_catches_differences(self):
        fixture = fixtures.build(QUEST, self.recordings, self.dump)
        fixture["steps"][2]["expect"]["vars"] = "020000000080"
        result = replay.run(fixture, self.script)
        self.assertFalse(result.ok)
        self.assertIn("vars 01 00 00 00 00 80, retail 02 00 00 00 00 80", result.steps[-1].problems[0])

    def test_interrupted_search(self):
        # A search broken off (moving, a hit, paralysis): the bar starts, the server ends the event, nothing else.
        fixture = fixtures.build(QUEST, self.recordings, self.dump)
        search = fixture["steps"][2]
        broken = {"talk": search["talk"], "layout": search.get("layout"), "scenes": [], "interrupted": True,
                  "expect": dict(fixture["steps"][1]["expect"], notices=[])}
        fixture["steps"].insert(2, broken)
        result = replay.run(fixture, self.script)
        self.assertTrue(result.ok, replay.format_results([result]))
        del broken["interrupted"]
        self.assertFalse(replay.run(fixture, self.script).ok)

    def test_replay_catches_unfinished_events(self):
        broken = self.dir / "broken.lua"
        broken.write_text(SCRIPT.replace("    player:finish_event()\nend\nfunction onYield",
                                         "end\nfunction onYield"))
        fixture = fixtures.build(QUEST, self.recordings, self.dump)
        result = replay.run(fixture, broken)
        self.assertFalse(result.ok)
        self.assertIn("never finished", result.steps[-1].problems[0])

    def test_dumps_round_trip(self):
        fixture = fixtures.build(QUEST, self.recordings, self.dump)
        text = fixtures.dumps(fixture)
        self.assertEqual(json.loads(text), fixture)
        self.assertIn('"notices": [[1, [1, 2]]]', text)

    def test_scaffold_draft_replays(self):
        fixture = fixtures.build(QUEST, self.recordings, self.dump)
        text, draft = scaffold.scaffold(QUEST, self.dumps, fixture)
        self.assertEqual(len(draft.groups), 1)
        self.assertIn("UI8AL counts them", text)
        out = self.dir / "draft.lua"
        out.write_text(text)
        result = replay.run(fixture, out)
        self.assertTrue(result.ok, replay.format_results([result]) + "\n" + text)


def packet(t, direction, name, decoded="", data=b"", src=5):
    header = bytes(16)
    return {"t": int(t * 1000), "dir": direction, "src": src, "dst": src, "kind": "Ipc", "name": name,
            "size": 16 + len(data), "hex": (header + data).hex(), "decoded": decoded}


def update(t, sequence, variables):
    values = ", ".join(str(b) for b in bytes.fromhex(variables))
    return packet(t, "out", "UpdateQuest", f"UpdateQuest {{ index: 0, quest: ActiveQuest {{ id: {QUEST}, sequence: "
                  f"{sequence}, flags: 0, accept_class_job: 0, bitflags: [{values}] }} }}")


def handler(name):
    return f"handler_id: HandlerId {{ type: QUEST, id: {QUEST} }}"


class DumpLayoutBasesTest(unittest.TestCase):
    def test_todo_locations_are_their_listeners(self):
        # Sylphic Studies' todo 0 at 2339560 is Ysabel Hawthorne (1000615), listening until todo 0; todo 1 has two
        # locations and one listener, so it's left alone.
        with tempfile.TemporaryDirectory() as tmp:
            quest = Path(tmp) / "GaiUsa003_00709"
            quest.mkdir()
            (quest / "quest.json").write_text(json.dumps({
                "todos": [{"index": 0, "locations": [2339560]}, {"index": 1, "locations": [1, 2]}],
                "listeners": [{"listener": 1000615, "despawn_sequence": 0},
                              {"listener": 1000616, "despawn_sequence": 1}],
            }))
            self.assertEqual(fixtures.dump_layout_bases(Path(tmp)), {2339560: 1000615})


class QuestEnemiesInDropInsTest(unittest.TestCase):
    """Every enemy a quest's fixture saw retail bring out (`spawns`) is in a drop-in, or the server can't spawn it
    (Monumental Hopes' three revenants, Bob's bundle 20261111-190723: "isn't in this zone's drop-ins")."""

    # Quests whose enemies aren't added yet (each listed in known-issues.md); none since 2026-11-11.
    NOT_YET: set[int] = set()

    def test_every_spawned_enemy_has_a_drop_in(self):
        root = Path(__file__).resolve().parents[3]
        laid_out = set()
        for path in (root / "resources" / "dropins").glob("*.json"):
            for layer in json.loads(path.read_text())["layers"]:
                laid_out.update(obj["layout_id"] for obj in layer["objects"])
        missing = {}
        for path in sorted((root / "tools" / "questkit" / "fixtures").glob("*.json")):
            fixture = json.loads(path.read_text())
            if not isinstance(fixture, dict) or fixture.get("quest") in self.NOT_YET:
                continue
            for step in fixture.get("steps", []):
                gone = [layout for layout in step.get("spawns") or [] if layout not in laid_out]
                if gone:
                    missing.setdefault(path.name, []).extend(gone)
        self.assertEqual(missing, {})


class InZoneMoveTest(unittest.TestCase):
    """Moving the player to a pop range in the zone they're in is retail's PrepareZoning to that same territory: replay
    counts it as a zone change there when the step says where it happens (Coming to Limsa Lominsa's lift.)"""

    def test_move_to_pop_range(self):
        from questkit.harness import Task, _effect
        task = Task("call", "move_to_pop_range", 4127803, True)
        self.assertEqual(_effect(task, 181), {"zone": 181})
        self.assertIsNone(_effect(task, None))


class BattleSimRecorderTalkTest(unittest.TestCase):
    """A Kawari Recorder battle file (2.7) logs no "battle talk" events: battle-sim reads its DirectorPopupMessage
    packets (236 on 7.58) instead, as Tactical Planning's K'lyhia's talks (Bob's run of 2026-11-12)."""

    def test_talks_from_packets(self):
        data = bytes.fromhex("7d7601400000000032000680d505000056000000a00f0000") + bytes(16)
        run = {"GameVersion": "2026.11.03.0000.0000",
               "Packets": [{"S": 7.693, "Op": 236, "Actor": 1, "Hex": data.hex()},
                           {"S": 8.0, "Op": 735, "Actor": 1, "Hex": data.hex()}]}
        talks = battle_sim._talk_events(run)
        self.assertEqual(len(talks), 1)
        self.assertEqual(battle_sim._number_after(talks[0][1], "text"), 86)
        self.assertEqual(battle_sim._number_after(talks[0][1], "name"), 1493)


class BattleSimSpawnTest(unittest.TestCase):
    """battle-sim checks the opening line-up too: everyone retail had from the start must be brought out, and nobody
    more often than retail (Like Mother, Like Daughter's first wave, written the wrong way round.)"""

    def test_line_up(self):
        fixture = {"npcs": {"1": {"spawn": None}, "2": {"spawn": None}, "3": {"spawn": 40.0}},
                   "events": [[40.0, "spawn", 3], [75.0, "spawn", 3]]}
        right = [(0.0, "spawn_bnpc", (1,)), (0.0, "spawn_bnpc", (2,)), (40.0, "spawn_bnpc", (3,)),
                 (75.0, "spawn_bnpc", (3,))]
        self.assertEqual(battle_sim.spawn_problems(fixture, right), [])
        wrong = [(0.0, "spawn_bnpc", (1,)), (0.0, "spawn_bnpc", (1,))]
        self.assertEqual(battle_sim.spawn_problems(fixture, wrong),
                         ["retail had 2 from the start: the script never brings it out", "brings 1 out 2 times, retail 1"])

    def test_on_setup_spawns_count(self):
        # Spawned in onSetup, before the opening cutscene (The Rematch's line-up): there from the start too.
        fixture = {"npcs": {"1": {"spawn": None}}, "events": []}
        self.assertEqual(battle_sim.spawn_problems(fixture, [(battle_sim.SETUP_CLOCK, "spawn_bnpc", (1,))]), [])

    def test_pools(self):
        # Quest battle 86's bombs: one of three layouts at random each time; any of a pool matches any other.
        fixture = {"npcs": {"7": {"spawn": 10.0}, "8": {"spawn": 20.0}}, "pools": [[7, 8, 9]],
                   "events": [[10.0, "spawn", 7], [20.0, "spawn", 8], [30.0, "spawn", 7]]}
        calls = [(10.0, "spawn_bnpc", (9,)), (20.0, "spawn_bnpc", (9,)), (30.5, "spawn_bnpc", (8,))]
        self.assertEqual(battle_sim.spawn_problems(fixture, calls), [])
        self.assertEqual(battle_sim.compare(fixture, calls, battle_sim.SimResult(86, None)), [])
        too_many = calls + [(40.0, "spawn_bnpc", (7,))]
        self.assertEqual(battle_sim.spawn_problems(fixture, too_many), ["brings 7 out 4 times, retail 3"])


class BattleSimPlayerTest(unittest.TestCase):
    """battle-sim gives a script the player's health as retail's last seen value (it jumps, so no interpolating), and
    a fixture can give one event more slack than the rest."""

    def test_player_health_and_tolerances(self):
        curve = [[0.0, 100.0], [93.89, 13.2], [94.15, 100.0]]
        self.assertEqual(battle_sim._health_then(curve, 94.0), 13.2)
        self.assertEqual(battle_sim._health_then(curve, 94.2), 100.0)
        fixture = {"tolerance": 2.0, "tolerances": {"talk 86": 4.5}, "npcs": {},
                   "events": [[80.9, "talk", 86], [10.0, "talk", 82]]}
        calls = [(77.0, "battle_talk", [1, 2, 86, 3]), (13.0, "battle_talk", [1, 2, 82, 3])]
        problems = battle_sim.compare(fixture, calls, battle_sim.SimResult(1, None))
        self.assertEqual(problems, ["talk 82: 13.0 s, retail 10.0 s"])

    def test_positions(self):
        """Positions are the last one recorded at or before the time, the origin before any (or without any.)"""
        path = [[1.0, 10.0, 2.0, 30.0], [3.0, 11.0, 2.0, 31.0]]
        self.assertEqual(battle_sim._position_then(path, 0.5), (0.0, 0.0, 0.0))
        self.assertEqual(battle_sim._position_then(path, 2.9), (10.0, 2.0, 30.0))
        self.assertEqual(battle_sim._position_then(path, 3.0), (11.0, 2.0, 31.0))
        self.assertEqual(battle_sim._position_then(None, 5.0), (0.0, 0.0, 0.0))


class CaptureTools(unittest.TestCase):
    def setUp(self):
        self.dir = Path(tempfile.mkdtemp())
        base = recording.parse_time(at(0))
        lines = [
            packet(base + 0.0, "out", "PlayerSetup"),
            packet(base + 1.0, "out", "AcceptQuest", f"AcceptQuest {{ quest_id: {QUEST} }}"),
            update(base + 1.0, 255, "000000000000"),
            update(base + 1.05, 1, "000000000000"),
            packet(base + 10, "out", "EventNotice2", "", bytes.fromhex(notice_hex(0, []))[:12]),
            update(base + 10, 2, "000000000000"),
            update(base + 20, 2, "010000000000"),
            update(base + 20, 2, "010000000040"),  # flag 2 first: another order than retail
            packet(base + 20, "out", "EventNotice2", "", bytes.fromhex(notice_hex(1, [1, 2]))[:16]),
            packet(base + 50, "in", "StartTalkEvent", f"StartTalkEvent {{ actor_id: ObjectTypeId {{ object_id: "
                   f"ObjectId (77), object_type: EObjOrNpc }}, {handler('')} }}"),
            packet(base + 50, "out", "EventStart", f"EventStart(EventStart {{ {handler('')}, event_type: Talk }})"),
            packet(base + 50, "out", "EventScene2", f"EventScene2(EventScene {{ {handler('')}, scene: 5 }})"),
            packet(base + 60, "in", "Unknown", "Unknown { unk: [] }", bytes(72)),
        ]
        lines.append({"t": int((base + 61) * 1000), "dir": "note",
                      "text": "MARK the boss went the other way | zone 148 at (1.00, 2.00, 3.00) facing 0.000 | quests none"})
        captures = self.dir / "bundle" / "captures"
        captures.mkdir(parents=True)
        with gzip.open(captures / "zone-1-ClientId1.jsonl.gz", "wt") as out:
            for line in lines:
                out.write(json.dumps(line) + "\n")
        bundle = self.dir / "bundle"
        (bundle / "meta.json").write_text(json.dumps({"created": "2026-10-14T20:02:00+0000", "note": "stuck",
                                                      "build": {"short": "abc"}, "files": {}}))
        (bundle / "characters.json").write_text(json.dumps([{
            "character": {"name": "Tester", "actor_id": 5}, "volatile": {"zone_id": 148, "position": {}},
            "quest_summary": {"active": [{"id": QUEST, "sequence": 2, "variables": [1, 0, 0, 0, 0, 0x40]}]}}]))
        snapshots = bundle / "snapshots"
        snapshots.mkdir()
        (snapshots / "a.json").write_text(json.dumps({"character": "Tester", "client_time": "x", "conditions": [],
                                                      "quests": [{"id": QUEST, "sequence": 2,
                                                                  "variables_hex": "010000000000"}]}))
        (snapshots / "b.json").write_text(json.dumps({"character": "Tester", "client_time": "w", "label": "stuck: frozen",
                                                      "quests": [], "stuck": {
                                                          "conditions": ["Casting"], "animation_lock": 0.5,
                                                          "pending_item_moves": [],
                                                          "events": ["12:00:00.000 LogMessage 572"],
                                                          "hooks": {"actions refused": "ok", "log messages": "failed: x"}}}))
        (bundle / "server.log").write_text(
            "2026-10-14T20:01:00.000000Z  WARN kawari_world::server: Unknown Zone packet Unknown(856) recieved\n"
            "2026-10-14T20:01:01.000000Z ERROR kawari_world: Error while calling onTalk: attempt to index nil\n")
        self.bundle = Bundle(bundle)
        rec = self.dir / "rec"
        rec.mkdir()
        (rec / "quests.json").write_text(json.dumps(
            {"Version": 1, "Quests": {str(QUEST): {"Name": "Test", "Events": RETAIL_EVENTS}}}))
        (rec / "packets.json").write_text(json.dumps(
            {"Version": 1, "GameVersion": "2026.10.06.0000.0000", "Timeline": RETAIL_PACKETS}))
        self.recordings = recording.Recordings.open([rec])

    def test_triage(self):
        report = triage.triage(self.bundle)
        self.assertIn("> stuck", report)
        self.assertIn("**differ**", report)  # server UI8F 0x40, client 0x00
        self.assertIn("quest 9999, started", report)  # the event left open
        self.assertIn("opcode 0 (72 bytes, the size of EventReturnHandler16", report)
        self.assertIn("Lua errors", report)
        self.assertIn("**Marks**", report)
        self.assertIn("the boss went the other way | zone 148", report)
        self.assertIn("What the client said was stopping the player", report)
        self.assertIn("`12:00:00.000 LogMessage 572`", report)
        self.assertIn("Hooks that failed", report)

    def test_compare(self):
        packets = self.bundle.packets()
        same, text = compare.compare(packets, self.recordings)
        self.assertIn("another order", text)
        self.assertIn("the capture ends before this", text)
        self.assertTrue(same, text)  # only an order difference, and the capture stops early

    def test_quest_states(self):
        states = [s for s in capture.quest_states(self.bundle.packets()) if s.kind == "update"]
        self.assertEqual(states[-1].variables.hex(), "010000000040")


def _battle_run():
    """A made-up quest battle: you and a guard (battalion 0) against a boss; two adds join at 10 s; the guard turns
    on the newest add, then the next once it dies; a healer heals the hurt guard; a statue never acts."""
    you, guard, healer, boss, add1, add2, statue = 1, 2, 3, 10, 11, 12, 13
    def actor(kind, name, layout, battalion, seen, hp=1000, died=None, pos=(0, 0, 0)):
        a = {"Kind": kind, "Name": name, "LayoutId": layout, "Battalion": battalion, "Level": 20, "MaxHp": hp,
             "Seen": seen, "LastSeen": 60.0, "Position": list(pos)}
        if died is not None:
            a["Died"] = died
        return a
    actors = {
        you: actor("you", "", 0, 0, 0.0),
        guard: actor("battle npc", "Guard", 100, 0, 0.0),
        healer: actor("battle npc", "Healer", 101, 0, 0.0),
        boss: actor("battle npc", "Boss", 200, 4, 0.0, pos=(10, 0, 0)),
        add1: actor("battle npc", "Add", 201, 4, 10.0, hp=300, died=20.0, pos=(5, 0, 0)),
        add2: actor("battle npc", "Add", 202, 4, 10.5, hp=300, pos=(20, 0, 0)),
        statue: actor("battle npc", "Statue", 300, 4, 0.0, pos=(50, 0, 0)),
    }
    samples = [{"S": 0.0, "Id": i, "Hp": a["MaxHp"], "P": a["Position"], "T": 0} for i, a in actors.items()]
    samples += [
        {"S": 1.0, "Id": guard, "T": boss},
        {"S": 11.0, "Id": guard, "T": add2},       # the newest add (add2 appeared at 10.5)
        {"S": 15.0, "Id": guard, "Hp": 400},
        {"S": 19.0, "Id": add1, "Hp": 0},
        {"S": 20.0, "Id": add1, "Dead": True},
        {"S": 30.0, "Id": add2, "Hp": 0, "Dead": True},
        {"S": 30.5, "Id": guard, "T": add1},       # add1 is already dead here: picked anyway (tests the "dead" path)
        {"S": 31.0, "Id": guard, "T": boss},
    ]
    actions = [
        {"S": 16.0, "Source": healer, "Action": 120, "Type": 1, "SourceHp": 1000,
         "Targets": [{"Id": guard, "Hp": 400, "MaxHp": 1000, "Effects": [[4, 300, 0, 0, 0, 0, 0]]}]},
        {"S": 12.0, "Source": guard, "Action": 7, "Type": 1, "SourceHp": 400,
         "Targets": [{"Id": add2, "Hp": 300, "MaxHp": 300, "Effects": [[3, 50, 0, 0, 0, 0, 0]]}]},
        {"S": 15.0, "Source": guard, "Action": 7, "Type": 1, "SourceHp": 400,
         "Targets": [{"Id": add2, "Hp": 250, "MaxHp": 300, "Effects": [[3, 50, 0, 0, 0, 0, 0]]}]},
    ]
    events = [{"S": 9.5, "Kind": "sequence", "Id": 0, "Text": "0x80030001 sequence 2"},
              {"S": 25.0, "Kind": "mark", "Id": 0, "Text": "guard stuck"}]
    layers = [{"File": "planmap.lgb", "LayerId": 7, "Name": "QB_TEST_001", "Objects": [
        {"Type": "BattleNPC", "Id": 201, "Name": "INIT_POP_2ND_01", "Transform": {},
         "Data": {"ParentData": {"MoveAi": 3}, "NormalAI": 42, "ServerPathId": 0}}]}]
    return {"Zone": "Test Zone", "Content": "Test Battle", "TerritoryId": 1, "ContentFinderCondition": 2,
            "Actors": {str(k): v for k, v in actors.items()}, "Samples": samples, "Actions": actions,
            "Events": events, "Controls": [], "Layers": layers}


class BattleReportTest(unittest.TestCase):
    def setUp(self):
        self.run = _battle_run()
        self.battle = battle.Battle(self.run)

    def test_waves_and_layout(self):
        waves = self.battle.waves()
        self.assertEqual([sorted(w) for _t, w in waves], [[2, 3, 10, 13], [11, 12]])
        info = self.battle.layout_info(201)
        self.assertEqual(info["layer"], "QB_TEST_001")
        self.assertEqual(info["NormalAI"], 42)
        self.assertEqual(info["MoveAi"], 3)

    def test_switch_reasons(self):
        reasons = self.battle.switch_reasons(2, 11.0, 10, 12)
        self.assertIn("newest enemy", reasons)
        self.assertIn("new one had just appeared", reasons)
        self.assertNotIn("old target dead or gone", reasons)
        reasons = self.battle.switch_reasons(2, 31.0, 11, 10)
        self.assertIn("old target dead or gone", reasons)

    def test_report(self):
        text = battle.report(self.run)
        self.assertIn("**2 appear**", text)
        self.assertIn("mark: guard stuck", text)
        self.assertIn("newest enemy: 1/", text)
        self.assertIn("healed targets were at 40–40% HP", text)
        self.assertIn("the most hurt of its side 1/1 times", text)
        self.assertIn("## Never acted", text)
        self.assertIn("Statue [300]", text.split("## Never acted")[1])
        self.assertIn("NormalAI 42", text)


class BattleFilesTest(unittest.TestCase):
    """Recorder 2.5 saves each battle to battles/<start>-<territory>-<what>.json.gz, with labels in index.json."""

    def _write(self, folder: Path):
        folder.mkdir(parents=True, exist_ok=True)
        first, second = _battle_run(), _battle_run()
        first["Start"], second["Start"] = "2026-10-17T01:00:00Z", "2026-10-17T02:00:00Z"
        # In the second run the guard goes for the boss instead of the newest add.
        second["Samples"] = [x if not (x["Id"] == 2 and x["S"] == 11.0) else {"S": 11.0, "Id": 2, "T": 11}
                             for x in second["Samples"]]
        names = ["20261017-010000-1-test-battle.json.gz", "20261017-020000-1-test-battle.json.gz"]
        for name, run in zip(names, (first, second)):
            (folder / name).write_bytes(gzip.compress(json.dumps(run).encode()))
        (folder / "index.json").write_text(json.dumps({"Battles": [{"File": names[1], "Label": "boss first"}]}))
        return names

    def test_folder_zip_and_labels(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder = Path(tmp) / "battles"
            names = self._write(folder)
            runs = battle.load_runs([folder])
            self.assertEqual(len(runs), 2)
            self.assertEqual(runs[1]["_label"], "boss first")
            # An unpacked export (battles/ inside) and a zip of it read the same.
            self.assertEqual(len(battle.load_runs([Path(tmp)])), 2)
            archive = Path(tmp) / "export.zip"
            with zipfile.ZipFile(archive, "w") as z:
                for name in names + ["index.json"]:
                    z.write(folder / name, f"battles/{name}")
            runs = battle.load_runs([archive])
            self.assertEqual([r.get("_label", "") for r in runs], ["", "boss first"])
            # A single file, and 2.4's battles.json.
            self.assertEqual(len(battle.load_runs([folder / names[0]])), 1)
            legacy = Path(tmp) / "battles.json"
            legacy.write_text(json.dumps({"Runs": [_battle_run(), _battle_run()]}))
            self.assertEqual(len(battle.load_runs([legacy])), 2)

    def test_across_runs(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder = Path(tmp)
            self._write(folder)
            text = battle.across(battle.load_runs([folder]))
            self.assertIn("# Across 2 runs: Test Zone Test Battle", text)
            guard = text.split("## Guard [100]")[1].split("## ")[0]
            self.assertIn("newest enemy: 1/", guard)
            self.assertIn("In 2/2 runs", guard)
            statue = text.split("## Statue [300]")[1].split("## ")[0]
            self.assertIn("never acted in 2", statue)
            self.assertIn("(boss first)", text)


class BattleRecorder26Test(unittest.TestCase):
    """What Recorder 2.6 adds: setup, hook health, action names, enmity, tethers, event objects, actor controls."""

    def test_new_sections(self):
        run = _battle_run()
        run["Setup"] = {"JobName": "CNJ", "Level": 16, "SyncedLevel": 0, "LevelSynced": False, "ItemLevel": 18,
                        "Hp": 900, "MaxHp": 900, "Statuses": [[42, 0, 0]], "Party": [[6, 16, 1]], "ExplorerMode": False}
        run["Hooks"] = {"Casts": "ok, seen 4", "Map effects": "FAILED: its signature wasn't found in the game"}
        run["ActionInfo"] = {"120": {"Name": "Cure", "CastType": 1}}
        run["Enmity"] = [{"S": 1.0, "Yours": [], "Haters": [[10, 100]]}, {"S": 12.0, "Yours": [], "Haters": [[12, 100], [10, 80]]}]
        run["Samples"] = run["Samples"] + [{"S": 5.0, "Id": 10, "Tethers": [[17, 2]]}, {"S": 8.0, "Id": 10, "Tethers": []}]
        run["Events"] = run["Events"] + [{"S": 21.0, "Kind": "auto mark", "Id": 10, "Text": "Boss below 50%"},
                                         {"S": 22.0, "Kind": "object", "Id": 0, "Text": "Barrier shown, not targetable, state 1"}]
        run["Controls"] = [{"S": 1.0, "Id": 10, "Category": 0, "Args": [1, 1, 0, 0, 0, 0, 0, 0], "Target": 0}]
        text = battle.report(run)
        self.assertIn("Missing from this recording", text)
        self.assertIn("Map effects", text)
        self.assertIn("CNJ level 16", text)
        self.assertIn("action 120 (Cure)", text)
        self.assertIn("tether: Boss [200] → Guard [100] (Channeling 17)", text)
        self.assertIn("tether gone: Boss [200]", text)
        self.assertIn("auto mark: Boss below 50%", text)
        self.assertIn("object: Barrier shown", text)
        self.assertIn("## Your enmity", text)
        self.assertIn("Add [202], Boss [200]", text)
        self.assertIn("0 ToggleWeapon: 1", text)

    def test_telegraphs(self):
        run = _battle_run()
        run["ActionInfo"] = {"740": {"Name": "Sweep", "CastType": 3, "EffectRange": 8, "Omen": 2, "Cast100ms": 30}}
        run["Events"] = run["Events"] + [
            {"S": 12.0, "Kind": "cast", "Id": 10, "Text": "action 740 spell 740 type 1 on 1 3s at 1, 2, 3 facing 0.5 omen delay 0 interruptible False"},
            {"S": 32.0, "Kind": "cast", "Id": 10, "Text": "action 740 spell 740 type 1 on 1 3s at 1, 2, 3 facing 0.5 omen delay 0 interruptible False"}]
        text = battle.report(run)
        self.assertIn("## Telegraphs", text)
        self.assertIn("Boss [200]: 740 (Sweep), cone 8 y, omen 2, 3.0 s cast, 2× every 20.0 s", text)


class ClassProfileTest(unittest.TestCase):
    """An NPC lancer and an NPC thaumaturge, read against their class mechanics (Recorder 2.7's ActionInfo)."""

    INFO = {
        "75": {"Name": "True Thrust", "CooldownGroup": 58, "Recast100ms": 25, "ActionCombo": 0, "ClassJob": 4, "ClassJobLevel": 1},
        "78": {"Name": "Vorpal Thrust", "CooldownGroup": 58, "Recast100ms": 25, "ActionCombo": 75, "ClassJob": 4, "ClassJobLevel": 4},
        "83": {"Name": "Life Surge", "CooldownGroup": 3, "Recast100ms": 450, "ClassJob": 4, "ClassJobLevel": 6, "StatusGainSelf": 116},
        "82": {"Name": "Leg Sweep", "CooldownGroup": 4, "Recast100ms": 200, "ClassJob": 4, "ClassJobLevel": 10},
        "871": {"Name": "attack", "ActionCategory": 1},
        "144": {"Name": "Thunder", "CooldownGroup": 58, "Recast100ms": 25, "ClassJob": 7, "ClassJobLevel": 6},
        "142": {"Name": "Blizzard", "CooldownGroup": 58, "Recast100ms": 25, "ClassJob": 7, "ClassJobLevel": 1},
    }

    def _run(self):
        run = _battle_run()
        run["ActionInfo"] = self.INFO
        run["ClassKits"] = {"4": {"Abbreviation": "LNC", "Actions": {
            "75": self.INFO["75"], "78": self.INFO["78"], "83": self.INFO["83"], "82": self.INFO["82"],
            "79": {"Name": "Piercing Talon", "CooldownGroup": 58, "ClassJob": 4, "ClassJobLevel": 15},
            "84": {"Name": "Full Thrust", "CooldownGroup": 58, "ActionCombo": 78, "ClassJob": 4, "ClassJobLevel": 26}}}}
        lancer, thaum, boss = 2, 3, 10
        def act(t, who, action, target=boss):
            return {"S": t, "Source": who, "Action": action, "Type": 1, "SourceHp": 1000,
                    "Targets": [{"Id": target, "Hp": 900, "MaxHp": 1000, "Effects": [[3, 50, 0, 0, 0, 0, 0]]}]}
        actions = []
        # The lancer: True Thrust, Life Surge woven, Vorpal Thrust, every 4 s; Leg Sweep every 20 s; autos.
        t = 0.0
        for n in range(15):
            actions.append(act(t, lancer, 75 if n % 2 == 0 else 78))
            if n % 2 == 0 and n % 12 == 0:
                actions.append(act(t + 1, lancer, 83))
            if n % 5 == 1:
                actions.append(act(t + 1.5, lancer, 82))
            actions.append(act(t + 0.5, lancer, 871))
            t += 4.0
        # The thaumaturge: Thunder on the boss every 18 s, Blizzard between, every 3 s.
        for n in range(20):
            t = n * 3.0
            actions.append(act(t, thaum, 144 if n % 6 == 0 else 142))
        run["Actions"] = sorted(actions, key=lambda x: x["S"])
        run["Actors"]["2"]["Level"] = 20
        return run

    def test_lancer(self):
        b = battle.Battle(self._run())
        profile = battle.class_profile(b, 2)
        self.assertEqual(profile["class"], "LNC")
        self.assertEqual(profile["auto_attacks"], [871])
        text = "\n".join(profile["facts"])
        self.assertIn("every 4.0 s", text)
        self.assertIn("Combo: 78 (Vorpal Thrust) follows 75 (True Thrust) in 7/7 uses.", text)
        self.assertIn("82 (Leg Sweep): on cooldown (every 20.0 s", text)
        self.assertIn("Piercing Talon", text)  # in its kit at level 20, never used
        self.assertNotIn("Full Thrust", text)  # level 26: not yet
        rotation = profile["rotation"]
        self.assertEqual(rotation["pace"], 4.0)
        order = [e["action"] for e in rotation["actions"]]
        self.assertLess(order.index(78), order.index(75))  # the finisher before the starter
        surge = next(e for e in rotation["actions"] if e["action"] == 83)
        self.assertEqual(surge.get("then"), 78)

    def test_thaumaturge_refresh(self):
        b = battle.Battle(self._run())
        profile = battle.class_profile(b, 3)
        thunder = next(e for e in profile["rotation"]["actions"] if e["action"] == 144)
        self.assertEqual(thunder.get("refresh"), 18.0)
        self.assertEqual(profile["rotation"]["actions"][0]["action"], 144)

    def test_report_has_rotation_block(self):
        text = battle.report(self._run())
        self.assertIn("fights as **LNC**", text)
        self.assertIn('"rotation"', text)


class OutlineTest(unittest.TestCase):
    """A made-up Chasing Shadows: Yda on the boss up close, Papalymo on the adds (oldest first) from range, near you,
    healing you. The outline should find exactly that."""

    def _run(self):
        you, yda, papa, boss, add1, add2, add3, add4 = 1, 2, 3, 10, 11, 12, 13, 14
        def actor(kind, name, layout, battalion, seen, hp, pos, died=None):
            a = {"Kind": kind, "Name": name, "LayoutId": layout, "Battalion": battalion, "Level": 20, "MaxHp": hp,
                 "Seen": seen, "LastSeen": 120.0, "Position": list(pos)}
            if died is not None:
                a["Died"] = died
            return a
        actors = {
            you: actor("you", "", 0, 0, 0, 900, (0, 0, 0)),
            yda: actor("battle npc", "Yda", 3802271, 0, 0, 5000, (8, 0, 0)),
            papa: actor("battle npc", "Papalymo", 3802272, 0, 0, 4000, (-2, 0, 4)),
            boss: actor("battle npc", "Boss", 3802315, 4, 0, 20000, (10, 0, 0)),
            add1: actor("battle npc", "Add", 4094218, 4, 0, 800, (-14, 0, 2), died=40.0),
            add2: actor("battle npc", "Add", 4094238, 4, 30, 800, (-16, 0, 6), died=70.0),
            add3: actor("battle npc", "Add", 4094239, 4, 31, 800, (-12, 0, 9), died=90.0),
            add4: actor("battle npc", "Add", 4094240, 4, 32, 800, (-18, 0, 1), died=100.0),
        }
        samples = [{"S": a["Seen"], "Id": i, "Hp": a["MaxHp"], "P": a["Position"], "T": 0} for i, a in actors.items()]
        samples += [{"S": 0.5, "Id": yda, "T": boss}, {"S": 0.5, "Id": papa, "T": add1},
                    {"S": 40.5, "Id": papa, "T": add2}, {"S": 70.5, "Id": papa, "T": add3},
                    {"S": 90.5, "Id": papa, "T": add4}, {"S": 100.5, "Id": papa, "T": boss}]
        for i in (add1, add2, add3, add4):
            samples.append({"S": actors[i]["Died"], "Id": i, "Hp": 0, "Dead": True})
        samples.append({"S": 60.0, "Id": boss, "Hp": 14000})  # Yda wearing it down
        actions = []
        def act(t, who, action, target, kind=3, hp=500, max_hp=800):
            actions.append({"S": t, "Source": who, "Action": action, "Type": 1, "SourceHp": 1,
                            "Targets": [{"Id": target, "Hp": hp, "MaxHp": max_hp, "Effects": [[kind, 50, 0, 0, 0, 0, 0]]}]})
        for k in range(30):
            act(1 + k * 3, yda, 53, boss)
            t = 1.5 + k * 3
            target = add1 if t < 40 else add2 if t < 70 else add3 if t < 90 else add4 if t < 100 else boss
            act(t, papa, 142, target)
        for t in (20.0, 50.0):
            act(t, papa, 120, you, kind=4, hp=360, max_hp=900)
        events = [{"S": 8.5, "Kind": "sequence", "Id": 0, "Text": "0x80030001 sequence 2"},
                  {"S": 30.0, "Kind": "sequence", "Id": 0, "Text": "0x80030001 sequence 3"}]
        return {"Zone": "Central Shroud", "Content": "", "TerritoryId": 1, "Actors": {str(k): v for k, v in actors.items()},
                "Samples": samples, "Actions": actions, "Events": events, "Controls": [], "Layers": []}

    def test_roles(self):
        b = battle.Battle(self._run())
        start, end, name = battle.phases(b)[-1]
        self.assertEqual(name, "sequence 3")
        yda = battle.ally_outline(b, 2, start, end)["role"]
        self.assertEqual(yda.get("focus"), ["boss", "adds"])
        self.assertEqual(yda.get("range"), 3)
        papa = battle.ally_outline(b, 3, start, end)
        self.assertEqual(papa["role"].get("focus"), ["adds", "boss"])
        self.assertEqual(papa["role"].get("order"), "oldest")
        self.assertGreaterEqual(papa["role"].get("range"), 8)
        self.assertIn("keep_near", papa["role"])
        # Papalymo heals now and then but mostly attacks: still a DPS, with some support.
        outlined = battle.ally_outline(b, 3, 0, 120)
        self.assertEqual(outlined["role"]["role"], "dps")
        self.assertIn("a DPS with some support (it heals, but mostly attacks)", outlined["facts"])
        self.assertEqual(yda.get("role"), "dps")
        heals = battle.ally_outline(b, 3, 0, 120)["role"]["heals"]
        self.assertEqual(heals[0]["targets"], ["player"])
        self.assertEqual(heals[0]["action"], 120)

    def test_outline_in_report(self):
        text = battle.report(self._run())
        self.assertIn("## The director's outline (draft)", text)
        self.assertIn("BATTLE:ally(3802271", text)
        self.assertIn('"change", "sequence 3"', text)


class BattleFrameworkTest(unittest.TestCase):
    """content/lib/Battle.lua's roles, run in Lua: a tank, two DPS and a support against a boss and two adds."""

    TANK, DPS1, DPS2, SUPPORT, BOSS, ADD1, ADD2 = 101, 102, 103, 104, 201, 202, 203

    def setUp(self):
        try:
            from lupa import LuaRuntime
        except ImportError:
            self.skipTest("lupa isn't installed")
        from questkit import repo
        self.lua = LuaRuntime(unpack_returned_tuples=True)
        self.lua.execute((repo.SCRIPTS / "content" / "lib" / "Battle.lua").read_text())
        self.calls = []

        def log(name, *args):
            self.calls.append((name, [dict(a.items()) if hasattr(a, "items") else a for a in args]))
        self.lua.globals().LOG = log
        # The director lives in Lua and logs through one long-lived Python function (lupa mixes up short-lived
        # Python objects passed back and forth.)
        self.director = self.lua.eval("""setmetatable({}, { __index = function(_, name)
            return function(_, ...) LOG(name, ...) end
        end })""")
        self.lua.execute("""
            B = Battle.new()
            B:ally(101, { role = "tank" })
            B:ally(102, { role = "dps" })
            B:ally(103, { role = "dps", range = 15 })
            B:ally(104, { role = "support", heals = { { action = 120, targets = "party", below = 50, tank_below = 70, every = 2 } },
                buffs = { { action = 7, every = 30 } } })
            B:enemy(201, { group = "boss", attacks = 104 })
            B:enemy(202, { group = "adds" })
            B:enemy(203, { group = "adds" })
        """)
        self.battle = self.lua.globals().B

    def update(self, health, info, n=1):
        for _ in range(n):
            self.battle.update(self.battle, self.director, self.lua.table_from(health), 100,
                               self.lua.table_from({k: self.lua.table_from(v) for k, v in info.items()}))

    def hates(self, who):
        return [args[1] for name, args in self.calls if name == "set_hate" and args[0] == who]

    def test_passive_enemy(self):
        b, d = self.battle, self.director
        self.lua.execute("B:enemy(204, { group = 'boss', passive = true })")
        b.spawn(b, d, self.lua.table_from([204, self.ADD1]))
        # A passive enemy is sent after no one (it waits to be hit); the others go after the player as usual.
        self.assertEqual(self.hates(204), [])
        self.assertEqual(len(self.hates(self.ADD1)), 1)

    def test_roles(self):
        b, d = self.battle, self.director
        b.spawn(b, d, self.lua.table_from([self.TANK, self.DPS1, self.DPS2, self.SUPPORT]))
        b.spawn(b, d, self.lua.table_from([self.BOSS, self.ADD1, self.ADD2]))
        # The tank charges in at once; the others hold back.
        self.assertEqual(self.hates(self.TANK), [self.BOSS])
        self.assertEqual(self.hates(self.DPS1), [])
        health = {self.TANK: 100, self.DPS1: 100, self.DPS2: 100, self.SUPPORT: 100, self.BOSS: 100, self.ADD1: 100, self.ADD2: 100}
        info = {self.BOSS: {"hp": 5000, "target": self.SUPPORT}, self.ADD1: {"hp": 900}, self.ADD2: {"hp": 1200},
                self.TANK: {"hp": 3000}}
        self.update(health, info, 5)
        # Then the DPS split the toughest between them: the boss, then the add with more health.
        self.assertEqual(self.hates(self.DPS1)[-1], self.BOSS)
        self.assertEqual(self.hates(self.DPS2)[-1], self.ADD2)
        # The support takes the adds and stays near the tank.
        # ... near the tank and the player, and behind the enemy the tank is fighting (out of its cones.)
        self.assertIn(("keep_near", [self.SUPPORT, {1: self.TANK, 2: "player"}, 20]), self.calls)
        self.assertIn(("stand_behind", [self.SUPPORT, self.TANK, 6]), self.calls)
        # The tank pulls the boss off the support it was hitting.
        self.assertIn([self.BOSS, self.TANK], [args[:2] for name, args in self.calls if name == "set_hate"])
        # Buffs go to a DPS first.
        buffs = [args for name, args in self.calls if name == "use_action" and args[1] == 7]
        self.assertIn(buffs[0][2], (self.DPS1, self.DPS2))
        # Heals: the tank at 65% before a DPS at 40% (the tank's threshold is higher: it takes the hits.)
        self.calls.clear()
        health.update({self.TANK: 65, self.DPS1: 40})
        self.update(health, info, 1)
        heals = [args for name, args in self.calls if name == "use_action" and args[1] == 120]
        self.assertEqual(heals[0][2], self.TANK)
        self.calls.clear()
        health.update({self.TANK: 90})
        self.update(health, info, 5)
        heals = [args for name, args in self.calls if name == "use_action" and args[1] == 120]
        self.assertEqual(heals[0][2], self.DPS1)

    def test_allies_meet_whoever_comes_for_them(self):
        """Bob's bundle 20261112-195353 (Notorious Biggs): retail's escort and conjurer ran out to the wave's enemies
        that came for them the moment they appeared, the weakest first (four runs); Kawari's stayed on the boss and
        the wave walked 45-54 yalms to them. And the wave stood 4.9 s where it appeared first."""
        b, d = self.battle, self.director
        self.lua.execute("""
            B:ally(105, { focus = { "mine", "boss", "adds" }, order = "lowest_hp" })
            B:enemy(205, { group = "adds", attacks = 105 })
            B:enemy(206, { group = "adds", attacks = 105 })
            B:enemy(207, { group = "adds", attacks = 101 })
        """)
        b.spawn(b, d, self.lua.table_from([105, self.BOSS]))
        self.assertEqual(self.hates(105), [self.BOSS])
        # Spawned before the fight: nobody stands.
        self.assertFalse([args for name, args in self.calls if name == "root_bnpc"])
        health = {105: 100, self.BOSS: 100}
        self.update(health, {self.BOSS: {"hp": 9000}}, 1)
        # The wave: 105 goes for one of the two after it at once (whom they were sent after), not 207 (after the
        # tank), and settles on the one with less health once their health is known.
        self.calls.clear()
        b.spawn(b, d, self.lua.table_from([205, 206, 207]))
        self.assertIn(self.hates(105)[-1], (205, 206))
        self.update(health, {self.BOSS: {"hp": 9000}, 205: {"hp": 1019, "target": 105},
                             206: {"hp": 561, "target": 105}, 207: {"hp": 561, "target": 101}}, 1)
        self.assertEqual(self.hates(105)[-1], 206, "the next update, not RETARGET_EVERY later")
        # ... each of them standing a moment first.
        stood = {args[0]: args[1] for name, args in self.calls if name == "root_bnpc"}
        self.assertEqual(set(stood), {205, 206, 207})
        self.assertAlmostEqual(stood[205], 4.9)
        # Once they're down it's back to the boss.
        b.died(b, d, 206)
        self.assertEqual(self.hates(105)[-1], 205)
        b.died(b, d, 205)
        self.assertEqual(self.hates(105)[-1], self.BOSS)


# --- Chronofoil captures, built by hand (protobuf as Chronofoil writes it) ---

def _pb_varint(value: int) -> bytes:
    out = bytearray()
    while True:
        byte = value & 0x7F
        value >>= 7
        out.append(byte | (0x80 if value else 0))
        if not value:
            return bytes(out)


def _pb(fields) -> bytes:
    out = b""
    for number, value in fields:
        if isinstance(value, int):
            out += _pb_varint(number << 3) + _pb_varint(value)
        else:
            out += _pb_varint(number << 3 | 2) + _pb_varint(len(value)) + value
    return out


def _delimited(message: bytes) -> bytes:
    return _pb_varint(len(message)) + message


def _frame(time_ms: int, direction: int, packets) -> bytes:
    """A game frame: 40-byte header, then IPC packets (opcode, source actor, data)."""
    import struct

    body = b""
    for opcode, src, data in packets:
        ipc = struct.pack("<HHHHII", 0x14, opcode, 0, 0, time_ms // 1000, 0) + data
        body += struct.pack("<IIIHH", 16 + len(ipc), src, 0, 3, 0) + ipc
    header = bytes(16) + struct.pack("<QIHHBBHI", time_ms, 40 + len(body), 1, len(packets), 1, 0, 0, 0)
    frame = header + body
    return _delimited(_pb([(1, _pb([(1, 1), (2, direction)])), (2, frame)]))


def _zone_init(territory: int, content: int) -> bytes:
    import struct

    return struct.pack("<HHHHIIBBH", 1, territory, 0, content, 0, 0, 0, 0, 2 if content else 0) + bytes(136 - 20)


def _capture_parts(capture_id: str, version: str):
    version_info = _delimited(_pb([(1, 1), (6, version.encode()), (7, b"Chronofoil"), (8, b"1.0.4.17")]))
    capture_info = _delimited(_pb([(1, capture_id.encode()), (2, _pb([(1, 1_760_000_000)])),
                                   (3, _pb([(1, 1_760_000_600)])), (4, 0)]))
    t = 1_760_000_000_000
    data = (_frame(t, 1, [(716, 0, _zone_init(132, 0))])
            + _frame(t + 1000, 1, [(769, 7, bytes(32)), (716, 0, _zone_init(1043, 15))])
            + _frame(t + 2000, 1, [(225, 0x8003_0001 & 0xFFFF, (0x8003_0001).to_bytes(4, "little") + bytes(20))])
            + _frame(t + 3000, 2, [(5, 1, bytes(8))]))
    return version_info, capture_info, data


class ChronofoilTest(unittest.TestCase):
    VERSION = "2026.10.06.0000.0000"
    TABLES = {VERSION: {"ServerZoneIpcType": {716: ("ZoneInit", 136, "test"), 769: ("ActorCast", 32, "test"),
                                              225: ("DirectorVars", 24, "test")}}}

    def test_unfinished_capture_folder(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder = Path(tmp) / "abcd"
            folder.mkdir()
            version_info, capture_info, data = _capture_parts("abcd", self.VERSION)
            (folder / "VersionInfo").write_bytes(version_info)
            (folder / "CaptureInfo").write_bytes(capture_info)
            (folder / "Data").write_bytes(data + b"\x05\x01")  # a frame cut short, as when the game closes
            captures = chronofoil.find_captures([tmp])
            self.assertEqual(len(captures), 1)
            capture = captures[0]
            self.assertEqual((capture.id, capture.game_version, capture.finished), ("abcd", self.VERSION, False))
            self.assertEqual(capture.duration, 600)
            segments = list(capture.segments())
            self.assertEqual([s.opcode for s in segments], [716, 769, 716, 225, 5])
            self.assertEqual(segments[-1].dir, "in")
            summary = chronofoil.summarize(capture, self.TABLES)
            self.assertEqual([(v.territory, v.content, v.instanced) for v in summary.visits],
                             [(132, 0, False), (1043, 15, True)])
            self.assertEqual(summary.visits[0].counts["ActorCast"], 1)
            self.assertEqual(summary.visits[1].directors, {0x8003_0001})
            report = chronofoil.index_report([summary], {"territory": {1043: "Somewhere"}, "content": {}},
                                             duties_only=True)
            self.assertIn("territory 1043 Somewhere duty 15", report)
            self.assertNotIn("territory 132", report)
            out = Path(tmp) / "visit.jsonl"
            self.assertEqual(chronofoil.export(capture, self.TABLES, summary.visits[1], out), 3)
            packets = capture_module_load(out)
            self.assertEqual([p.name for p in packets], ["ZoneInit", "DirectorVars", "Unknown5"])

    def test_finished_capture_needs_zstd(self):
        try:
            import zstandard
        except ImportError:
            self.skipTest("zstandard not installed")
        with tempfile.TemporaryDirectory() as tmp:
            version_info, capture_info, data = _capture_parts("efgh", self.VERSION)
            path = Path(tmp) / "efgh.cfcap"
            with zipfile.ZipFile(path, "w") as archive:
                archive.writestr("VersionInfo", version_info)
                archive.writestr("CaptureInfo", capture_info)
                archive.writestr("Data", zstandard.ZstdCompressor().compress(data))
            capture, = chronofoil.find_captures([path])
            self.assertTrue(capture.finished)
            self.assertEqual(len(chronofoil.summarize(capture, self.TABLES).visits), 2)

    def test_battle_from_capture(self):
        import struct

        you, ally, boss, scenery = 0x1000_0001, 0x4000_0001, 0x4000_0002, 0x4000_0003
        def spawn_npc(layout, hp, battalion):
            common = bytearray(560)
            struct.pack_into("<III", common, 48, 900, 500, layout)
            struct.pack_into("<II", common, 76, hp, hp)
            common[113], common[117], common[118] = 2, battalion, 15
            struct.pack_into("<fff", common, 496, 1.0, 2.0, 3.0)
            return bytes(16) + bytes(common) + bytes(640 - 16 - 560)
        def set_target(target):
            return struct.pack("<HHIIII", 50, 0, 0, 0, 0, 0) + bytes(4) + struct.pack("<II", target, 0)
        def effect(action, target, amount):
            data = bytearray(124)
            struct.pack_into("<I", data, 8, action)
            data[31], data[33] = 1, 1
            struct.pack_into("<BBBBBBH", data, 42, 3, 0, 0, 0, 0, 0, amount)
            struct.pack_into("<I", data, 112, target)
            return bytes(data)
        def hp(current):
            return struct.pack("<IIIIII", 1, 1, 0, current, 0, 0)
        t = 1_760_000_000_000
        def frame(ms, src, opcode, data):
            return _frame(t + ms, 1, [(opcode, src, data)])
        tables = {self.VERSION: {"ServerZoneIpcType": {716: ("ZoneInit", 136, "t"), 1: ("SpawnNpc", 640, "t"),
                                                       2: ("ActorControlTarget", 32, "t"), 3: ("ActionEffect1", 124, "t"),
                                                       4: ("EffectResultBasic", 24, "t"), 225: ("DirectorVars", 24, "t")}}}
        data = (frame(0, you, 716, _zone_init(1043, 15))
                + frame(100, ally, 1, spawn_npc(11, 1000, 0)) + frame(100, boss, 1, spawn_npc(22, 5000, 4))
                + frame(100, scenery, 1, spawn_npc(33, 90000, 4))
                + frame(2000, 0, 225, (0x8003_0001).to_bytes(4, "little") + bytes([2]) + bytes(19))
                + frame(2100, ally, 2, set_target(boss)) + frame(2500, ally, 3, effect(873, boss, 300))
                + frame(2600, boss, 4, hp(4700)))
        with tempfile.TemporaryDirectory() as tmp:
            folder = Path(tmp) / "cap"
            folder.mkdir()
            version_info, capture_info, _ = _capture_parts("cap", self.VERSION)
            (folder / "VersionInfo").write_bytes(version_info)
            (folder / "CaptureInfo").write_bytes(capture_info)
            (folder / "Data").write_bytes(data)
            capture, = chronofoil.find_captures([folder])
            visit, = chronofoil.summarize(capture, tables).visits
            run = chronofoil_battle.run_from_visit(capture, tables, visit)
        actors = {a["LayoutId"]: a for a in run["Actors"].values()}
        self.assertEqual((actors[22]["MaxHp"], actors[22]["Battalion"], actors[22]["BaseId"]), (5000, 4, 900))
        self.assertEqual(actors[11]["Position"], [1.0, 2.0, 3.0])
        self.assertEqual(run["Actions"][0]["Targets"][0]["Effects"][0][:2], [3, 300])
        self.assertIn({"S": 2.1, "Id": ally, "T": boss}, run["Samples"])
        self.assertIn({"S": 2.6, "Id": boss, "Hp": 4700}, run["Samples"])
        self.assertEqual(run["Events"][0]["Text"], "0x80030001 sequence 2")
        b = battle.Battle(run)
        # The ally and the boss it fights are linked to... nobody here is "you", so everyone is in the fight; the
        # boss is engaged (it lost HP) and the scenery isn't.
        self.assertTrue(b.engaged(boss))
        self.assertFalse(b.engaged(scenery))
        self.assertEqual(battle._boss_at(b, ally, 3.0)[0], boss)

    def test_saved_tables_cover_every_version(self):
        tables = chronofoil.load_opcodes()
        for version in ("2026.06.18.0000.0000", "2026.09.01.0000.0000", "2026.09.15.0000.0000",
                        "2026.10.06.0000.0000"):
            names = {name for entries in tables[version].values() for name, _, _ in entries.values()}
            for needed in ("ZoneInit", "ActorCast", "ActionEffect1", "SpawnNpc", "DirectorVars"):
                self.assertIn(needed, names, f"{needed} in {version}")


def capture_module_load(path):
    return capture.load(path)



class DropinTest(unittest.TestCase):
    """questkit chronofoil --dropin: which sightings become spawn points, and where."""

    @staticmethod
    def sighting(layout, base=100, x=0.0, **extra):
        spawn = {"layout": layout, "base": base, "name": 7, "owner": 0xE0000000, "max_hp": 126, "display": 0,
                 "fate": 0, "rotation": 1.5, "kind": 2, "level": 7, "position": [x, 1.0, 0.0], "event_npc": 0,
                 "flags": 1, "max_links": 4, "link_family": 0, "link_range": 20, "behavior": 30246,
                 "seconds": 0.0, "moved": False, "busy": False}
        spawn.update(extra)
        return spawn

    def test_stationary_and_guards(self):
        from questkit import chronofoil_dropin as D
        s = self.sighting
        seen = {
            20: [s(20, seconds=90)],                              # idle a minute and a half, never moved
            21: [s(21, seconds=90), s(21, seconds=5, moved=True)],  # seen walking once: wanders
            22: [s(22, seconds=120, busy=True)],                  # only seen fighting: can't tell
            # A kind that never moved in two long sightings: its briefly seen spot stands still too.
            30: [s(30, base=200, seconds=70)], 31: [s(31, base=200, seconds=80)], 32: [s(32, base=200, seconds=3)],
            40: [s(40, base=791, flags=4, behavior=30053)],      # a guard
        }
        dropin, _ = D.build(seen, "bg/x/planmap.lgb")
        data = {o["layout_id"]: o["data"] for o in dropin["layers"][0]["objects"]}
        self.assertEqual(sorted(k for k, d in data.items() if d.get("stationary")), [20, 30, 31, 32])
        self.assertEqual((data[40]["character_data_flags"], data[40]["behavior"]), (4, 30053))
        self.assertNotIn("character_data_flags", data[20])

    def test_build(self):
        from questkit import chronofoil_dropin as D
        s = self.sighting
        seen = {
            # Seen three times as it wandered: placed where it was nearest the average (x = 2).
            10: [s(10, x=0.0), s(10, x=2.0), s(10, x=5.0)],
            11: [s(11, flags=2)],                    # hostile
            12: [s(12, fate=375)],                   # a FATE's
            13: [s(13, owner=0x10000001)],           # someone's chocobo
            14: [s(14, event_npc=4745262)],          # a quest NPC turned battle NPC
            15: [s(15, base=3180)], 16: [s(16, base=3180)],   # a hunt mark's two spots
            17: [s(17, base=999)],                   # excluded
        }
        dropin, skipped = D.build(seen, "bg/x/planmap.lgb", exclude_bases={999}, one_spot={3180})
        objects = {o["layout_id"]: o for o in dropin["layers"][0]["objects"]}
        self.assertEqual(sorted(objects), [10, 11, 15])
        self.assertEqual(objects[10]["position"]["x"], 2.0)
        self.assertEqual(objects[10]["data"]["hp"], 126)
        self.assertFalse(objects[10]["data"]["hostile"])
        self.assertTrue(objects[11]["data"]["hostile"])
        self.assertEqual(dropin["appends"], "bg/x/planmap.lgb")
        self.assertEqual(skipped, {"FATE": 1, "owned by someone": 1, "event NPC": 1,
                                   "another spot of a one-spot NPC": 1, "excluded": 1})

    def test_reads_spawn_npc(self):
        import struct
        from questkit import chronofoil_dropin as D
        data = bytearray(16 + 600)
        struct.pack_into("<I", data, 4, 0)
        data[8] = 2
        data[11:14] = bytes([4, 0, 20])
        c = 16
        struct.pack_into("<IIII", data, c + 48, 351, 262, 3747899, 0)
        struct.pack_into("<I", data, c + 68, 0xE0000000)
        struct.pack_into("<I", data, c + 76, 44)
        struct.pack_into("<H", data, c + 98, 65535)      # quantized: +pi
        data[c + 113] = 2
        data[c + 118] = 1
        struct.pack_into("<fff", data, c + 496, -213.5, -28.5, 128.0)
        spawn = D._spawn(bytes(data))
        self.assertEqual((spawn["base"], spawn["name"], spawn["layout"], spawn["max_hp"], spawn["level"]),
                         (351, 262, 3747899, 44, 1))
        self.assertAlmostEqual(spawn["rotation"], 3.14159, places=4)
        self.assertEqual(spawn["position"], [-213.5, -28.5, 128.0])
        self.assertEqual((spawn["flags"], spawn["max_links"], spawn["link_range"]), (2, 4, 20))

if __name__ == "__main__":
    unittest.main()


class ChronofoilQuestTest(unittest.TestCase):
    """A quest read out of a (made-up) Chronofoil capture, made into a fixture with scenes and effects, and replayed."""

    VERSION = "2026.10.06.0000.0000"
    TABLES = {VERSION: {
        "ServerZoneIpcType": {716: ("ZoneInit", 136, "t"), 10: ("EventStart", 24, "t"), 11: ("EventScene2", 40, "t"),
                              12: ("AcceptQuest", 8, "t"), 13: ("UpdateQuest", 24, "t"), 14: ("EventFinish", 16, "t"),
                              15: ("PrepareZoning", 16, "t"), 16: ("EventNotice2", 16, "t")},
        "ClientZoneIpcType": {20: ("StartTalkEvent", 16, "t"), 21: ("EventYieldHandler4", 24, "t"),
                              22: ("EventReturnHandler2", 16, "t")},
    }}
    SCRIPT = """
current_scene = nil
function play(player, scene, flags) current_scene = scene; player:play_scene(scene, flags, {}) end
function onTalk(target, player)
    if player:get_quest_sequence(EVENT_ID) == nil then play(player, 0, HIDE_HOTBAR) else player:finish_event() end
end
function sceneFinished(scene, results, player)
    if scene ~= current_scene then return end
    current_scene = nil
    if scene == 0 and results[1] == 1 then
        play(player, 1, CUTSCENE)
        return
    elseif scene == 1 then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
        player:event_notice(EVENT_ID, 0)
        player:change_territory(130, { x = 0, y = 0, z = 0 }, 0)
    end
    player:finish_event()
end
function onYield(scene, id, results, player) sceneFinished(scene, results, player) end
function onReturn(scene, results, player) sceneFinished(scene, results, player) end
"""

    def capture(self, tmp):
        import struct

        handler = 0x10000 + QUEST
        layout = 3969632
        t = 1_760_000_000_000

        def out(ms, opcode, data):
            return _frame(t + ms, 1, [(opcode, 0, data)])

        def into(ms, opcode, data):
            return _frame(t + ms, 2, [(opcode, 0, data)])

        def scene(number, flags):
            return struct.pack("<QIHHII", layout, handler, number, 0, flags, 0) + bytes(16)

        data = (out(0, 716, _zone_init(182, 0))
                + into(100, 20, struct.pack("<QII", layout, handler, 0))
                + out(150, 10, struct.pack("<QIBBHII", layout, handler, 1, 0, 0, 0, 0))
                + out(160, 11, scene(0, 0x2000))
                + into(2000, 21, struct.pack("<IHBBiii", handler, 0, 0, 3, 1, 0, 0) + bytes(4))
                + out(2100, 11, scene(1, 0xF8402EFB))
                + into(9000, 22, struct.pack("<IHBBii", handler, 1, 0, 2, 0, 0))
                + out(9100, 12, struct.pack("<H", QUEST) + bytes(6))
                + out(9100, 16, struct.pack("<IBBH", handler, 0, 0, 0) + bytes(8))
                + out(9100, 15, struct.pack("<IH", 0, 130) + bytes(10))
                + out(9100, 13, struct.pack("<HHHBB", 0, 0, QUEST, 1, 0) + bytes(6) + bytes([0] * 6) + bytes(4))
                + out(9200, 14, struct.pack("<I", handler) + bytes(12)))
        folder = Path(tmp) / "cap"
        folder.mkdir()
        version_info, capture_info, _ = _capture_parts("cap", self.VERSION)
        (folder / "VersionInfo").write_bytes(version_info)
        (folder / "CaptureInfo").write_bytes(capture_info)
        (folder / "Data").write_bytes(data)
        return chronofoil.find_captures([folder])

    def test_events_fixture_and_replay(self):
        with tempfile.TemporaryDirectory() as tmp:
            flows = chronofoil_quest.quest_events(self.capture(tmp), self.TABLES, [QUEST])
            kinds = [e.kind for e in flows[QUEST]]
            self.assertEqual(kinds, ["talk", "start", "scene", "answer", "scene", "answer", "accept", "notice", "zone",
                                     "update", "finish"])
            dump = {"script": "SubTst001_09999", "params": [{"name": "ACTOR0", "value": 1001}]}
            fixture = fixtures.build_from_chronofoil(QUEST, flows[QUEST], dump, {3969632: 1001})
            step, = fixture["steps"]
            self.assertEqual((step["talk"], step["layout"], step["what"]), (1001, 3969632, "ACTOR0"))
            self.assertEqual(step["scenes"], [
                {"scene": 0, "flags": "0x2000", "results": [1, 0, 0], "yield": True},
                {"scene": 1, "flags": "0xf8402efb", "results": [0, 0]}])
            self.assertEqual(step["effects"], [{"zone": 130}])
            self.assertEqual(step["expect"], {"sequence": 1, "vars": "000000000000", "notices": [[0, []]]})

            good = Path(tmp) / "good.lua"
            good.write_text(self.SCRIPT.replace("CUTSCENE", "SET_BASE | HIDE_HOTBAR"))
            self.assertTrue(replay.run(fixture, good).ok, replay.format_results([replay.run(fixture, good)]))
            # The wrong flags for the cutscene, or no zone change, fail.
            wrong = Path(tmp) / "wrong.lua"
            wrong.write_text(self.SCRIPT.replace("CUTSCENE", "HIDE_HOTBAR"))
            result = replay.run(fixture, wrong)
            self.assertIn("scenes 0 (0x2000), 1 (0x2000), retail 0 (0x2000), 1 (0xf8402efb)",
                          result.steps[-1].problems[0])
            stays = Path(tmp) / "stays.lua"
            stays.write_text(self.SCRIPT.replace("CUTSCENE", "SET_BASE | HIDE_HOTBAR").replace(
                "player:change_territory(130, { x = 0, y = 0, z = 0 }, 0)", ""))
            result = replay.run(fixture, stays)
            self.assertIn("did nothing outside the quest, retail zone 130", result.steps[-1].problems[0])

    def test_unnamed_steps_come_from_the_old_fixture(self):
        Event = chronofoil_quest.Event
        events = [
            Event(0, 148, "talk", {"actor": 5}), Event(0, 148, "start", {"type": 1, "actor": 5, "flags": 0, "arg": 0}),
            Event(1, 148, "scene", {"scene": 0, "flags": 0x2000, "params": []}),
            Event(2, 148, "answer", {"scene": 0, "yielded": True, "code": 0, "results": [0, 0, 0]}),
            Event(2, 148, "finish", {}),  # the offer declined
            Event(10, 148, "talk", {"actor": 5}), Event(10, 148, "start", {"type": 1, "actor": 5, "flags": 0, "arg": 0}),
            Event(11, 148, "scene", {"scene": 0, "flags": 0x2000, "params": []}),
            Event(12, 148, "answer", {"scene": 0, "yielded": True, "code": 0, "results": [1, 0, 0]}),
            Event(12, 148, "accept", {}), Event(12, 148, "update", {"sequence": 1, "vars": "000000000000"}),
            Event(12, 148, "finish", {}),
            Event(20, 148, "talk", {"actor": 7}), Event(20, 148, "start", {"type": 1, "actor": 7, "flags": 0, "arg": 0}),
            Event(21, 148, "scene", {"scene": 2, "flags": 0x2001, "params": []}),
            Event(23, 148, "finish", {}),  # broken off during the progress bar: nothing happened
            Event(30, 148, "notice", {"notice": 1, "args": [1, 2]}),
            Event(30, 148, "update", {"sequence": 255, "vars": "000000000000"}),  # a kill
        ]
        previous = {"steps": [
            {"talk": 1001, "expect": {"sequence": 1, "vars": "000000000000"}},
            {"kill": [12, 3838969], "expect": {"sequence": 255, "vars": "000000000000"}}]}
        dump = {"script": "X", "params": [{"name": "ACTOR0", "value": 1001}]}
        fixture = fixtures.build_from_chronofoil(QUEST, events, dump, {}, previous=previous)
        steps = fixture["steps"]
        self.assertEqual([s.get("talk") or s.get("kill") for s in steps], [1001, 1001, [12, 3838969]])
        self.assertEqual(steps[0]["expect"], {"accepted": False, "notices": []})
        self.assertEqual(steps[2]["expect"]["notices"], [[1, [1, 2]]])
        self.assertNotIn("todo", json.dumps(steps))

    def test_items_and_log_lines_a_step_gives(self):
        # Following in His Footsteps (Bob's Chronofoil captures d1c46dff and 898dd0b9): each prayer gives an armour
        # piece (EventLogMessage 750 [item] with handler 0 while the quest's event is open) and the quest's
        # LOGMESSAGE0; the transcription missed both (Bob's bundle 20261113-092701).
        import struct
        handler = 0x10000 + QUEST
        tables = {self.VERSION: {"ServerZoneIpcType": {10: ("EventStart", 24, "t"), 17: ("EventLogMessage2", 24, "t"),
                                                       14: ("EventFinish", 16, "t")}, "ClientZoneIpcType": {}}}
        t = 1_760_000_000_000

        def out(ms, opcode, data):
            return _frame(t + ms, 1, [(opcode, 0, data)])

        def log(h, message, params):
            return struct.pack("<IIB3x", h, message, len(params)) + struct.pack(f"<{len(params)}I", *params) \
                + bytes(4 * (2 - len(params)) + 4)

        data = (out(0, 10, struct.pack("<QIBBHII", 7, handler, 2, 0, 0, 58, 0))
                + out(100, 17, log(0, 750, [3687])) + out(100, 17, log(handler, 3657, []))
                + out(200, 14, struct.pack("<I", handler) + bytes(12))
                + out(300, 17, log(0, 750, [5])))  # after the event: not the quest's
        with tempfile.TemporaryDirectory() as tmp:
            folder = Path(tmp) / "cap"
            folder.mkdir()
            version_info, capture_info, _ = _capture_parts("cap", self.VERSION)
            (folder / "VersionInfo").write_bytes(version_info)
            (folder / "CaptureInfo").write_bytes(capture_info)
            (folder / "Data").write_bytes(data)
            flows = chronofoil_quest.quest_events(chronofoil.find_captures([folder]), tables, [QUEST])
        self.assertEqual([(e.kind, e.data) for e in flows[QUEST] if e.kind in ("item", "log")],
                         [("item", {"item": 3687, "quantity": 1}), ("log", {"message": 3657, "params": []})])

        Event = chronofoil_quest.Event
        events = [Event(0, 138, "start", {"type": 2, "actor": 7, "flags": 0, "arg": 58}),
                  Event(1, 138, "scene", {"scene": 3, "flags": 0x2001, "params": []}),
                  Event(2, 138, "answer", {"scene": 3, "yielded": False, "code": 0, "results": [0, 0]}),
                  Event(2, 138, "item", {"item": 3687, "quantity": 1}),
                  Event(2, 138, "log", {"message": 3657, "params": []}),
                  Event(2, 138, "finish", {})]
        fixture = fixtures.build_from_chronofoil(QUEST, events, {"script": "X", "params": []}, {},
                                                 previous={"steps": [{"emote": [58, 2002396], "layout": 7}]})
        expect = fixture["steps"][0]["expect"]
        self.assertEqual((expect["items"], expect["logs"]), ([[3687, 1]], [[3657, []]]))

        script = """
function onEmote(target, emote, player) player:accept_quest(EVENT_ID); player:play_scene(3, 0x2001, {}) end
function onReturn(scene, results, player)
    GIVE
    player:finish_event()
end
"""
        step = {"emote": [58, 2002396], "layout": 7, "scenes": [{"scene": 3, "flags": "0x2001", "results": [0, 0]}],
                "expect": {"items": [[3687, 1]], "logs": [[3657, []]]}}
        quest = {"quest": QUEST, "script": "X", "params": {}, "steps": [step]}
        with tempfile.TemporaryDirectory() as tmp:
            good, bare = Path(tmp) / "good.lua", Path(tmp) / "bare.lua"
            good.write_text(script.replace("GIVE", "player:obtain_item(3687, 1)\n"
                                           "    player:event_log_message(EVENT_ID, 3657, {})"))
            bare.write_text(script.replace("GIVE", ""))
            self.assertTrue(replay.run(quest, good).ok, replay.format_results([replay.run(quest, good)]))
            result = replay.run(quest, bare)
            self.assertIn("gave items [], retail [[3687, 1]]", result.steps[-1].problems[0])

    def test_an_item_used_on_an_object_and_an_emote(self):
        # Drowning Out the Voices (Bob's Chronofoil capture da424662, 2026-11-12): an item used on an object is
        # EventStart type 8 (the object, the item); the old fixture names the object. Escape from Castrum Centri: an
        # emote on an NPC only the old fixture names stays an emote (it took a talk's base before.)
        Event = chronofoil_quest.Event
        events = [Event(0, 156, "start", {"type": 8, "actor": 4300105, "flags": 0, "arg": 2000766}),
                  Event(1, 156, "scene", {"scene": 9, "flags": 0x2001, "params": []}),
                  Event(2, 156, "answer", {"scene": 9, "yielded": False, "code": 0, "results": [0, 0]}),
                  Event(2, 156, "finish", {}),
                  Event(10, 156, "start", {"type": 2, "actor": 4317487, "flags": 0, "arg": 59}),
                  Event(11, 156, "scene", {"scene": 24, "flags": 0x2001, "params": []}),
                  Event(12, 156, "answer", {"scene": 24, "yielded": False, "code": 0, "results": [0, 0]}),
                  Event(12, 156, "finish", {})]
        previous = {"steps": [{"event_item": [2000766, 2002232], "layout": 4300105, "expect": {}},
                              {"talk": 1006562, "layout": 4273822, "expect": {}},
                              {"emote": [59, 1007611], "layout": 4317487, "expect": {}}]}
        dump = {"script": "X", "params": [{"name": "EOBJECT3", "value": 2002232}]}
        steps = fixtures.build_from_chronofoil(QUEST, events, dump, {}, previous=previous)["steps"]
        self.assertEqual(steps[0]["event_item"], [2000766, 2002232])
        self.assertEqual(steps[0]["what"], "EOBJECT3")
        self.assertEqual(steps[1]["emote"], [59, 1007611])
        self.assertNotIn("talk", steps[1])

        # Without the old fixture, a todo naming the item.
        steps = fixtures.build_from_chronofoil(QUEST, events[:4], dump, {})["steps"]
        self.assertIn("item 2000766 used on actor 4300105", steps[0]["todo"])

    def test_the_old_fixtures_bases_win_over_the_dumps(self):
        # Following the Evidence (911): the dumps' todo paired layout 4293142 with 2002141, the old fixture (checked by
        # hand) says 2002140.
        Event = chronofoil_quest.Event
        events = [Event(0, 155, "talk", {"actor": 4293142}),
                  Event(0, 155, "start", {"type": 1, "actor": 4293142, "flags": 0, "arg": 0}),
                  Event(1, 155, "scene", {"scene": 13, "flags": 0x2001, "params": []}),
                  Event(2, 155, "answer", {"scene": 13, "yielded": False, "code": 0, "results": [0, 0]}),
                  Event(2, 155, "finish", {})]
        previous = {"steps": [{"talk": 2002140, "layout": 4293142, "expect": {}}]}
        dump = {"script": "X", "params": []}
        fixture = fixtures.build_from_chronofoil(QUEST, events, dump, {4293142: 2002141}, previous=previous)
        self.assertEqual(fixture["steps"][0]["talk"], 2002140)


class TranscribeTest(unittest.TestCase):
    """Bars, spawns and moves read from a capture's events, and a script transcribed from the fixture that replays."""

    def events(self):
        Event = chronofoil_quest.Event

        def talk(t, actor):
            return [Event(t, 148, "talk", {"actor": actor}),
                    Event(t, 148, "start", {"type": 1, "actor": actor, "flags": 0, "arg": 0})]

        def scene(t, number, flags, results, yielded=False, code=0):
            return [Event(t, 148, "scene", {"scene": number, "flags": flags, "params": []}),
                    Event(t + 1, 148, "answer", {"scene": number, "yielded": yielded, "code": code,
                                                 "results": results})]

        return (talk(0, 5) + scene(1, 0, 0x2000, [1, 0, 0], True) + scene(3, 1, 0x2000, [0, 0])
                + [Event(4, 148, "accept", {}), Event(4, 148, "update", {"sequence": 1, "vars": "000000000000"}),
                   Event(4, 148, "finish", {})]
                # Talked to in combat: no event at all.
                + [Event(8, 148, "talk", {"actor": 7})]
                # The object: its scene, the bar broken off; then again, the bar done and enemies out.
                + talk(10, 7) + scene(11, 2, 0x2001, [0, 0]) + [Event(13, 148, "bar", {"action": 1}),
                                                                Event(14, 148, "finish", {})]
                + talk(20, 7) + scene(21, 2, 0x2001, [0, 0]) + [Event(23, 148, "bar", {"action": 1})]
                + scene(26, 3, 0x2001, [0, 0])
                + [Event(28, 148, "spawn", {"base": 12, "layout": 4000001}),
                   Event(28, 148, "update", {"sequence": 2, "vars": "000000000000"}), Event(28, 148, "finish", {})]
                + [Event(30, 148, "update", {"sequence": 255, "vars": "000000000000"})]
                # The reward, then a move to a spot of no pop range of the quest's.
                + talk(40, 5) + scene(41, 4, 0xF8402EFB, [1, 5826, 0, 0], True)
                + [Event(43, 148, "finish", {}),
                   Event(43, 148, "zone", {"territory": 148, "arrival": [1.0, 2.0, 3.0, 1.57]}),
                   Event(43, 148, "finish_quest", {})])

    def test_fixture_and_script(self):
        dump = {"quest_id": QUEST, "script": "SubTst001_09999", "name": "Test",
                "params": [{"name": "ACTOR0", "value": 1001}, {"name": "EOBJECT0", "value": 2001},
                           {"name": "ENEMY0", "value": 4000001}, {"name": "EVENT_ACTION_SEARCH", "value": 1}]}
        previous = {"steps": [{"kill": [12, 4000001], "expect": {"sequence": 255, "vars": "000000000000"}}]}
        fixture = fixtures.build_from_chronofoil(QUEST, self.events(), dump, {5: 1001, 7: 2001}, previous=previous)
        steps = fixture["steps"]
        self.assertEqual([s.get("talk") or s.get("kill") for s in steps], [1001, 2001, 2001, [12, 4000001], 1001])
        self.assertTrue(steps[1]["interrupted"])
        self.assertEqual(steps[1]["scenes"][0]["bar"], 1)
        self.assertEqual(steps[2]["spawns"], [4000001])
        self.assertEqual(steps[4]["arrivals"], [[1.0, 2.0, 3.0, 1.57]])
        self.assertEqual(steps[4]["territory"], 148)

        text, draft = transcribe.Transcript(dump, [("test", fixture)]), None
        script = text.write("a made-up capture")
        self.assertIn("player:event_action(EVENT_ACTION_SEARCH)", script)
        self.assertIn("player:spawn_quest_bnpc(ENEMY0)", script)
        self.assertIn("player:change_territory(148, { x = 1.0, y = 2.0, z = 3.0 }, 1.57)", script)
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "script.lua"
            path.write_text(script)
            result = replay.run(fixture, path)
            self.assertTrue(result.ok, replay.format_results([result]))

    def test_flags(self):
        self.assertEqual(transcribe.flags_lua(0x2001), "NO_DEFAULT_CAMERA | HIDE_HOTBAR")
        self.assertEqual(transcribe.flags_lua(0xF8402EFB), "SET_BASE | HIDE_HOTBAR")
        self.assertEqual(transcribe.flags_lua(0xF8402FFF), "SET_BASE | INVIS_ENPC | INVIS_AETHERYTE | HIDE_HOTBAR")


class BattleSimObjectTalkTest(unittest.TestCase):
    """battle-sim plays the player talking to an object the script brought out (`object_talks`), and the event action
    the script answers with ending `bar` seconds later (In the Eyes of Gods and Men's rosary.)"""

    SCRIPT = """
function onCommence(director)
    director:set_sequence(2)
    director:spawn_eobj(5, { x = 0, y = 0, z = 0 })
end
function onUpdate(director, health, player_health, info) end
function onObjectTalk(director, actor_id, object_id, base_id)
    if base_id == 5 then director:event_action(22, actor_id, object_id) end
end
function onEventActionCast(director, actor_id, target)
    director:finish_gimmick(actor_id)
    director:set_sequence(3)
end
"""

    def test_talk_and_bar(self):
        try:
            import lupa  # noqa: F401
        except ImportError:
            self.skipTest("lupa isn't installed")
        fixture = {"quest_battle": 9998, "tolerance": 1.0, "scenes": {}, "npcs": {},
                   "events": [[0.0, "sequence", 2], [15.0, "sequence", 3]], "object_talks": [[10.0, 5, 5.0]]}
        with tempfile.TemporaryDirectory() as tmp:
            script = Path(tmp) / "9998.lua"
            script.write_text(self.SCRIPT)
            result = battle_sim.simulate(fixture, script)
            self.assertTrue(result.ok, battle_sim.format_results([result], verbose=True))
            # Without the talk, nothing comes of the object.
            result = battle_sim.simulate({**fixture, "object_talks": []}, script)
            self.assertIn("sequence 3 at 15.0 s on retail: the script never did it", result.problems)

    def test_a_few_hp_is_not_dead(self):
        run = {"Events": [{"S": 1.0, "Kind": "scene", "Text": "event 0x8006003C scene 60000"}],
               "Actors": {"7": {"Kind": "battle npc", "LayoutId": 70, "MaxHp": 9150, "Seen": 0.5}},
               "Samples": [{"S": 5.0, "Id": 7, "Hp": 1}, {"S": 8.0, "Id": 7, "Hp": 0}], "Actions": []}
        fixture = battle_sim.fixture_from_run(run, 60)
        self.assertEqual(fixture["npcs"]["70"]["hp"], [[4.0, 0.1], [7.0, 0.0]])
        self.assertEqual(fixture["npcs"]["70"]["death"], 7.0)


class BattleSimTest(unittest.TestCase):
    """A quest battle script against a made-up retail run: a boss, a wave at half health, a line, the win."""

    SCRIPT = """
BATTLE = Battle.new()
BATTLE:ally(11, { role = "tank" })
BATTLE:enemy(22, { group = "boss", attacks = 11 })
BATTLE:enemy(33, { group = "adds" })
BATTLE:onHealth(22, HALF, function(director)
    director:set_sequence(3)
    BATTLE:spawn(director, { 33 })
end)
function onSetup(director) director:set_sequence(1) end
function onCommence(director)
    director:set_sequence(2)
    BATTLE:spawn(director, { 11, 22 })
    BATTLE:steps({ { 10, "talk", { layout = 11, name = 1, icon = 2 }, 100 } })
end
function onUpdate(director, health, player_health, info) BATTLE:update(director, health, player_health, info) end
function onActorDeath(director, layout)
    BATTLE:died(director, layout)
    if BATTLE:enemiesLeft() == 0 and BATTLE.alive[22] == nil then
        director:set_sequence(4)
        director:complete_quest_battle(2)
    end
end
"""
    FIXTURE = {
        "quest_battle": 9999, "tolerance": 1.0, "scenes": {},
        "npcs": {"11": {"spawn": None, "death": None, "hp": [[0, 100]]},
                 "22": {"spawn": None, "death": 60.0, "hp": [[0, 100], [30, 50], [60, 0]]},
                 "33": {"spawn": 30.5, "death": 40.0, "hp": [[30.5, 100], [40, 0]]}},
        "events": [[0.0, "sequence", 2], [5.0, "talk", 100], [30.0, "sequence", 3], [30.5, "spawn", 33],
                   [60.0, "sequence", 4], [60.0, "won"]],
    }

    def run_script(self, half):
        try:
            import lupa  # noqa: F401
        except ImportError:
            self.skipTest("lupa isn't installed")
        with tempfile.TemporaryDirectory() as tmp:
            script = Path(tmp) / "9999.lua"
            script.write_text(self.SCRIPT.replace("HALF", str(half)))
            original = battle_sim.dropin_layouts
            battle_sim.dropin_layouts = lambda: {11, 22, 33}
            try:
                return battle_sim.simulate(self.FIXTURE, script)
            finally:
                battle_sim.dropin_layouts = original

    def test_matches(self):
        result = self.run_script(50)
        self.assertTrue(result.ok, battle_sim.format_results([result], verbose=True))
        self.assertEqual(len(result.matched), 6)

    def test_late_wave(self):
        # At 25% the wave comes 15 s late; the add then falls 15 s late too, but the boss still falls at 60 s.
        result = self.run_script(25)
        self.assertFalse(result.ok)
        self.assertIn("sequence 3: 45.0 s, retail 30.0 s", result.problems)

    def test_spawned_npcs_have_no_health_on_the_next_update(self):
        # The server spawns what onCommence asked for after the first update, so that update has no health for it:
        # a script treating a missing health as 0 (Duty, Honor, Country's Garibald, 2026-10-20) jumps ahead.
        self.SCRIPT = self.SCRIPT.replace(
            "function onUpdate(director, health, player_health, info) BATTLE:update(director, health, player_health, info) end",
            "function onUpdate(director, health, player_health, info)\n"
            "    if not EARLY and (health[22] or 0) <= 50 then EARLY = true; director:set_sequence(3) end\n"
            "    BATTLE:update(director, health, player_health, info)\nend")
        result = self.run_script(50)
        self.assertFalse(result.ok)
        self.assertIn("sequence 3: 0.0 s, retail 30.0 s", result.problems)

    def test_fixture_from_run(self):
        run = {
            "Actors": {"1": {"Kind": "battle npc", "LayoutId": 22, "MaxHp": 1000, "Seen": 1.0, "Died": 62.0},
                       "2": {"Kind": "battle npc", "LayoutId": 33, "MaxHp": 100, "Seen": 40.0, "Died": 52.0}},
            "Samples": [{"S": 10.0, "Id": 1, "Hp": 1000}, {"S": 40.0, "Id": 1, "Hp": 500},
                        {"S": 60.0, "Id": 1, "Hp": 0}, {"S": 50.0, "Id": 2, "Hp": 0}],
            "Actions": [],
            "Events": [{"S": 2.0, "Kind": "sequence", "Text": "0x80060025 sequence 1"},
                       {"S": 10.0, "Kind": "scene", "Text": "event 0x80060025 scene 60000 flags 0x40000"},
                       {"S": 10.0, "Kind": "sequence", "Text": "0x80060025 sequence 2"},
                       {"S": 15.0, "Kind": "battle talk", "Text": "text 100 name 1 icon 2 4000 ms"},
                       {"S": 62.0, "Kind": "scene", "Text": "event 0x80060025 scene 60001 flags 0x40000"},
                       {"S": 63.0, "Kind": "sequence", "Text": "0x80060025 sequence 254"}],
        }
        fixture = battle_sim.fixture_from_run(run, 9999)
        self.assertEqual(fixture["events"], [[0.0, "sequence", 2], [5.0, "talk", 100], [30.0, "spawn", 33],
                                             [52.0, "won"]])
        self.assertEqual(fixture["npcs"]["22"], {"spawn": None, "death": 50.0, "hp": [[0.0, 100.0], [30.0, 50.0],
                                                                                    [50.0, 0.0]]})
        self.assertEqual(fixture["npcs"]["33"]["death"], 40.0)  # its health reached 0 before its Defeated

    def test_a_death_from_its_status(self):
        # Notorious Biggs's boss (Bob's Chronofoil capture da424662, 2026-11-12): the killing blow's HP never came, only
        # his status (ActorControl 2 with 2: dead) and Defeated 2.1 s after it. He fell with the status.
        run = {
            "Actors": {"1": {"Kind": "battle npc", "LayoutId": 22, "MaxHp": 1000, "Seen": 1.0, "Died": 62.1}},
            "Samples": [{"S": 10.0, "Id": 1, "Hp": 1000}, {"S": 59.9, "Id": 1, "Hp": 2},
                        {"S": 62.1, "Id": 1, "Hp": 0, "Dead": True}],
            "Actions": [],
            "Controls": [{"S": 60.0, "Id": 1, "Category": 2, "Params": [2, 0, 0, 0], "Packet": "ActorControl"},
                         {"S": 61.0, "Id": 1, "Category": 2, "Params": [1, 0, 0, 0], "Packet": "ActorControl"}],
            "Events": [{"S": 10.0, "Kind": "scene", "Text": "event 0x80060025 scene 60000 flags 0x40000"},
                       {"S": 10.0, "Kind": "sequence", "Text": "0x80060025 sequence 2"}],
        }
        fixture = battle_sim.fixture_from_run(run, 9999)
        self.assertEqual(fixture["npcs"]["22"]["death"], 50.0)

    def test_fixture_onsetup_sequence_and_repeated_layout(self):
        # Trial by Wind: sequence 1 0.4 s before the fight starts (the script's onSetup), and a layout ID spawned a
        # second time while the first is still standing.
        run = {
            "Actors": {"2": {"Kind": "battle npc", "LayoutId": 22, "MaxHp": 30, "Seen": 15.0},
                       "1": {"Kind": "battle npc", "LayoutId": 22, "MaxHp": 30, "Seen": 0.5}},
            "Samples": [],
            "Actions": [],
            "Events": [{"S": 0.7, "Kind": "sequence", "Text": "0x80060036 sequence 1"},
                       {"S": 1.1, "Kind": "scene", "Text": "event 0x80060036 scene 60000 flags 0x40000"},
                       {"S": 1.1, "Kind": "sequence", "Text": "0x80060036 sequence 2"}],
        }
        fixture = battle_sim.fixture_from_run(run, 9999)
        self.assertEqual(fixture["events"], [[0.0, "sequence", 2], [13.9, "spawn", 22]])
        self.assertIsNone(fixture["npcs"]["22"]["spawn"])


class LintTest(unittest.TestCase):
    def test_timeline_actions(self):
        timeline = {"autoattack_action_id": 870, "timepoints": [{"data": {"type": "action", "action_id": 739}},
                                                                {"data": {"type": "invulnerability"}}],
                    "rotation": {"actions": [{"action": 9}]}, "recorded_amounts": {"21": {"npcs": 1}}}
        self.assertEqual(lint.timeline_actions(timeline), {870, 739, 9, 21})

    def test_quest_battle_npcs_against_retail_spawns(self):
        # Fool Me Twice (quest battle 63): retail's boss rode a Magitek Reaper (Mount 14, Bob's Chronofoil capture
        # da424662); the layer had no mount and Kawari's stood on foot (Bob's bundle 20261114-015635).
        boss = {"base": 1988, "name": 2092, "level": 46, "hp": 6540, "flags": 3, "display": 262146, "mount": 14,
                "model_chara": 0, "visits": 3}
        lancer = {"base": 1990, "name": 1811, "level": 42, "hp": 572, "flags": 3, "display": 262146, "mount": 0,
                  "model_chara": 0, "visits": 3}
        registry = {"308": {"4270036": boss, "4270060": lancer}}
        data = {"base_id": 1988, "name_id": 2092, "level": 46, "hp": 6540, "hostile": True, "type": "battle_npc",
                "character_data_flags": 3, "display_flags": 262146}
        dropins = {"lak_l1f1_planmap.json": {"layers": [{"name": "QB_GaiUsc604_001", "objects": [
            {"layout_id": 4270036, "data": data}]}]}}
        problems = lint.spawn_differences(registry, dropins)
        self.assertEqual(len(problems), 2, problems)
        self.assertIn("4270036 has mount 0, retail spawned it with 14", problems[0])
        self.assertIn("retail spawned battle NPC 4270060", problems[1])

        data["mount"] = 14
        dropins["lak_l1f1_planmap.json"]["layers"][0]["objects"].append(
            {"layout_id": 4270060, "data": dict(data, base_id=1990, name_id=1811, level=42, hp=572, mount=None)})
        self.assertEqual(lint.spawn_differences(registry, dropins), [])
        # Not hostile in the layer, aggressive on retail.
        data["character_data_flags"], data["hostile"] = 0, False
        self.assertIn("has flags 0, retail spawned it with 3", lint.spawn_differences(registry, dropins)[0])

    def test_spawn_fields_read_retail_offsets(self):
        from questkit import chronofoil_spawns
        import struct
        data = bytearray(16 + 620)
        data[8] = 3
        struct.pack_into("<III", data, 16 + 48, 1988, 2092, 4270036)
        struct.pack_into("<I", data, 16 + 76, 6540)
        struct.pack_into("<I", data, 16 + 84, 262146)
        struct.pack_into("<H", data, 16 + 100, 14)
        data[16 + 118] = 46
        self.assertEqual(chronofoil_spawns.spawn_fields(bytes(data)),
                         {"base": 1988, "name": 2092, "level": 46, "hp": 6540, "flags": 3, "display": 262146,
                          "mount": 14, "model_chara": 0})

    def test_repository_is_clean(self):
        report = lint.run()
        self.assertTrue(report.ok, lint.format_report(report))
        self.assertIn(870, lint.scripted_actions())


class ClientCheckTest(unittest.TestCase):
    """questkit clientcheck against a made-up client listing laid out as kawari-questdump writes them."""

    LISTING = """; scenes: Npc:OnScene00000, Npc:OnScene00001, Npc:OnScene00002
== Npc:OnScene00000  [function 0.0.1]
== Npc:OnScene00001  [function 0.0.2]
== Npc:OnScene00002  [function 0.0.3]
== <main>/<anonymous 1>  [function 0.1]
      4  SETTABLE   R0["MAIN_MENU_RESULT_MENU"] = 1
      6  SETTABLE   R0["MAIN_MENU_RESULT_SPECIALSHOP"] = 2
"""

    def _setup(self, root: Path, lua: str) -> Path:
        from questkit import clientcheck  # noqa: F401

        scripts = root / "scripts"
        (scripts / "events" / "custom" / "001").mkdir(parents=True)
        (scripts / "events" / "custom" / "001" / "Npc_00107.lua").write_text(lua)
        dumps = root / "dumps" / "custom"
        (dumps / "001").mkdir(parents=True)
        (dumps / "001" / "Npc_00107.txt").write_text(self.LISTING)
        (dumps / "001" / "Npc_00107.json").write_text(json.dumps({"params": [{"name": "ACTOR0", "value": 10}]}))
        (dumps / "index.json").write_text(json.dumps([{
            "name": "Npc_00107", "handler_id": 720896 + 107, "path": "001/Npc_00107.luab",
            "server_script": "events/custom/001/Npc_00107.lua", "on_server": True, "found": True}]))
        return scripts

    def test_agrees(self):
        from questkit import clientcheck

        lua = ("SCENE_GREETING = 00000\nSCENE_MENU = 00001\nSCENE_GIVE = 00002\nSCENE_UNUSED = 00009\n"
               "MAIN_MENU_RESULT_SPECIALSHOP = 2\n"
               "function onTalk(target, player) player:play_scene(SCENE_MENU, HIDE_HOTBAR, {0}) end\n"
               "function more(player) play(player, SCENE_GREETING) play(player, SCENE_GIVE) end\n"
               "-- 255 is Kawari's, not a scene\nSCENE_RETURNED_FROM_NEST = 255\nlocal x = ACTOR0\n")
        with tempfile.TemporaryDirectory() as tmp:
            scripts = self._setup(Path(tmp), lua)
            checks, missing = clientcheck.collect(Path(tmp) / "dumps", scripts)
        self.assertEqual(missing, [])
        self.assertEqual(len(checks), 1)
        self.assertFalse(checks[0].issues, checks[0])
        self.assertEqual(checks[0].played, [0, 1, 2])

    def test_talks_table(self):
        # A script that plays its scenes from a TALKS table (the Arcanist and Summoner quests of 2026-11-12.)
        from questkit import clientcheck

        lua = ("TALKS = {\n    { seq = nil, who = ACTOR0, scenes = { 0, 1 }, go_on = offered },\n"
               "    { seq = 1, who = ACTOR0, scenes = { 2 } },\n}\n")
        with tempfile.TemporaryDirectory() as tmp:
            scripts = self._setup(Path(tmp), lua)
            checks, _ = clientcheck.collect(Path(tmp) / "dumps", scripts)
        self.assertEqual(checks[0].played, [0, 1, 2])
        self.assertEqual(checks[0].unplayed, [])

    def test_finds_what_disagrees(self):
        from questkit import clientcheck

        lua = ("SCENE_GREETING = 00000\nSCENE_SHOP = 00010\nMAIN_MENU_RESULT_SPECIALSHOP = 3\n"
               "function onTalk(target, player) play(player, SCENE_GREETING) play(player, SCENE_SHOP) end\n"
               "-- TODO: how is his shop menu brought up?\nlocal x = ACTOR1\n")
        with tempfile.TemporaryDirectory() as tmp:
            scripts = self._setup(Path(tmp), lua)
            checks, _ = clientcheck.collect(Path(tmp) / "dumps", scripts)
            text = clientcheck.report(checks, [])
        check = checks[0]
        self.assertEqual(check.unknown_scenes, [10])
        self.assertEqual(check.unplayed, [1, 2])
        self.assertEqual(check.constant_mismatches, [("MAIN_MENU_RESULT_SPECIALSHOP", 3.0, 2.0)])
        self.assertEqual(check.unknown_params, ["ACTOR1"])
        self.assertEqual(len(check.notes), 1)
        self.assertIn("needs a look", text)


class PublicSourcesTest(unittest.TestCase):
    """Boss Mod's modules and cactbot's timelines, read into numbers (made-up module and timeline)."""

    MODULE = """namespace BossMod.Made.Up;

public enum OID : uint
{
    Boss = 0x1234,
    Helper = 0x233C,
    SomeAdd = 0x1235,
}

public enum AID : uint
{
    AutoAttack = 870, // Boss->player, no cast, single-target
    BigCone = 5001, // Boss->self, 4.0s cast, range 12 90-degree cone
    Ring = 5002, // Helper->self, 5.0s cast, range 5-20 donut
    Guess = 5003, // Boss->self, 3.0s cast, range 8 ?-degree cone
    AddHit = 5004, // SomeAdd->player, no cast, single-target
}

class BigCone(BossModule module) : Components.StandardAOEs(module, AID.BigCone, new AOEShapeCone(12, 45.Degrees()));
class Ring(BossModule module) : Components.StandardAOEs(module, AID.Ring, new AOEShapeDonut(5, 20));
class Guess(BossModule module) : Components.StandardAOEs(module, AID.Guess, new AOEShapeCone(8, 30.Degrees())); // TODO: verify angle

[ModuleInfo(BossModuleInfo.Maturity.Verified, GroupType = BossModuleInfo.GroupType.CFC, GroupID = 77, NameID = 99)]
public class MadeUp(ModuleInit init) : BossModule(init, new(10, -20), new ArenaBoundsCircle(20));
"""

    def public(self, folder: Path) -> public_sources.Public:
        checkout = folder / "bossmod" / "BossMod.Modules" / "Made"
        checkout.mkdir(parents=True)
        (checkout / "Up.cs").write_text(self.MODULE, encoding="utf-8")
        cactbot = folder / "cactbot"
        (cactbot / "resources").mkdir(parents=True)
        (cactbot / "resources" / "zone_id.ts").write_text("const data = {\n  'MadeUp': 4242,\n} as const;\n")
        data = cactbot / "ui" / "raidboss" / "data" / "00-made"
        data.mkdir(parents=True)
        (data / "up.ts").write_text("export default { zoneId: ZoneId.MadeUp };\n")
        (data / "up.txt").write_text('hideall "--sync--"\n'
                                     '10.0 "A Name" Ability { id: "1389", source: "A Boss" }\n'
                                     '25.0 "Other Name" Ability { id: "138A", source: "A Boss" }\n'
                                     '40.0 "A Name" Ability { id: "1389", source: "A Boss" }\n'
                                     '70.0 "A Name" Ability { id: "1389", source: "A Boss" } jump 10.0\n')
        out = folder / "out"
        made = public_sources.build(folder / "bossmod", cactbot, out)
        self.assertEqual(made, {"bossmod": 1, "cactbot": 1})
        return public_sources.Public(out)

    def test_module_is_read_as_numbers(self):
        with tempfile.TemporaryDirectory() as folder:
            public = self.public(Path(folder))
            text = (Path(folder) / "out" / "bossmod.json").read_text() + (Path(folder) / "out" / "cactbot.json").read_text()
        for name in ("BigCone", "SomeAdd", "A Name", "Other Name"):
            self.assertNotIn(name, text)
        module, = public.duty(77)
        self.assertEqual(module["oids"], {"Boss": 0x1234, "Helper": 0x233C, "actor4661": 0x1235})
        self.assertEqual(module["arena"], {"x": 10.0, "z": -20.0, "bounds": "circle", "size": [20.0]})
        # Boss Mod's half angle is the server's whole one; a TODO line is its guess.
        self.assertEqual(public.area(5001), {"shape": "cone", "radius": 12.0, "angle": 90.0})
        self.assertEqual(public.cone_angles(), {5001: 90.0})
        self.assertEqual(public.cone_angles(verified_only=False), {5001: 90.0, 5003: 60.0})
        self.assertEqual(public.timing(4242)[0x1389]["every"], 30.0)

    def test_server_data_and_draft(self):
        with tempfile.TemporaryDirectory() as folder:
            public = self.public(Path(folder))
        data = duty_sources.server_data(public)
        self.assertEqual(data["cone_angles"], {"5001": 90.0})
        self.assertEqual(data["donut_holes"], {"5002": 0.25})
        files = duty_sources.draft(77, 4242, public)
        timeline = json.loads(files["Up_4660.json"])
        self.assertEqual(timeline["autoattack_action_id"], 870)
        # The boss's own actions: not the helper's ring, not the add's hit.
        self.assertEqual([entry["action"] for entry in timeline["rotation"]["actions"]], [5001, 5003])
        self.assertIn("Boss Mod's guess", files["README.md"])

    def test_draft_takes_what_the_captures_hold(self):
        class Made:
            id = "madeupcapture"

        def spawn(actor, base, hp):
            return {"e": "spawn", "t": 1.0, "a": actor, "k": "npc", "kind": 2, "base": base, "name": 99, "lv": 50,
                    "hp": hp, "max": hp, "job": 0, "p": [0.0, 0.0, 0.0]}

        def hit(t, actor, action, amount):
            return {"e": "effect", "t": t, "a": actor, "action": action, "type": 1, "seq": 0, "lock": 0.6, "p": None,
                    "r": 0.0, "hits": [{"on": 0, "p": None, "fx": [[3, amount, 0, 0, 0, 0, 0]]}]}

        events = [{"e": "class", "t": 0.0, "a": 0, "job": 19, "lv": 50, "class_lv": 50, "synced": 0},
                  {"e": "zone", "t": 0.5, "territory": 4242, "content": 77},
                  spawn(1, 0x1234, 60000), spawn(2, 0x233C, 1), spawn(3, 555, 3000),
                  {"e": "cast", "t": 9.0, "a": 2, "action": 5002, "type": 1, "time": 5.0, "on": None,
                   "p": [10.0, 0.0, -20.0], "r": 0.0}]
        events += [hit(10.0 + 20 * k, 1, 5001, 900 + k) for k in range(4)]
        events += [hit(11.0, 2, 5002, 700), hit(12.0, 3, 870, 40), hit(15.0, 3, 870, 44), hit(18.0, 3, 870, 48),
                   hit(13.0, 1, 5999, 100), {"e": "zone", "t": 200.0, "territory": 1, "content": 0}, hit(201.0, 1, 5001, 5)]
        with tempfile.TemporaryDirectory() as folder:
            public = self.public(Path(folder))
        captured = duty_sources.capture_facts([(Made, events)], 77)
        self.assertEqual((captured["territory"], captured["runs"]), (4242, ["madeupca"]))
        boss = captured["bases"][0x1234]
        self.assertEqual((boss["level"], boss["hp"]), (50, 60000))
        self.assertEqual(boss["actions"][5001], {"uses": 4, "most_hit": 1, "every": 20.0, "players": 902})
        files = duty_sources.draft(77, None, public, captured)
        timeline = json.loads(files["Up_4660.json"])
        self.assertEqual(timeline["recorded_amounts"], {"5001": {"players": 902}, "5999": {"players": 100}})
        self.assertEqual(timeline["rotation"]["actions"][0]["every"], 20.0)
        self.assertIn("60000 HP", timeline["comment"])
        readme = files["README.md"]
        self.assertIn("Helper: 5002", readme)
        self.assertIn("cast from 1 spots (x, z): (10, -20)", readme)
        self.assertIn("not in Boss Mod's module: 5999", readme)
        # The add fought: its own timeline, from the captures alone (the helper has none.)
        add = json.loads(files["Duty77Enemy_555.json"])
        self.assertEqual((add["autoattack_action_id"], add["recorded_amounts"]), (870, {"870": {"players": 44}}))
        self.assertNotIn("Duty77Enemy_9020.json", files)

    def test_audit_lists_what_a_timeline_never_mentions(self):
        with tempfile.TemporaryDirectory() as folder:
            public = self.public(Path(folder))
            repo = Path(folder) / "repo"
            (repo / "resources" / "timelines").mkdir(parents=True)
            (repo / "resources" / "scripts" / "content").mkdir(parents=True)
            (repo / "resources" / "data").mkdir(parents=True)
            (repo / "resources" / "timelines" / "MadeUp_4660.json").write_text(json.dumps(
                {"autoattack_action_id": 870, "rotation": {"actions": [{"action": 5001}]}, "comment": ""}))
            (repo / "resources" / "data" / "action_areas.json").write_text(json.dumps({"cone_angles": {"5001": 60}}))
            report = duty_sources.audit(repo, public)
        self.assertIn("never mentioned: 5003", report)
        self.assertNotIn("5004", report)
        self.assertIn("5001: recorded 60, Boss Mod 90", report)


class LedgerTest(unittest.TestCase):
    """A capture read into typed events with nobody named, and the fact tables over it (a made-up capture)."""

    VERSION = "2026.10.06.0000.0000"
    TABLES = {VERSION: {
        "ServerZoneIpcType": {716: ("ZoneInit", 136, "t"), 1: ("SpawnNpc", 640, "t"), 2: ("SpawnPlayer", 640, "t"),
                              3: ("ActionEffect1", 124, "t"), 4: ("EffectResult", 96, "t"), 5: ("ActorGauge", 16, "t"),
                              6: ("ActorControl", 24, "t"), 7: ("UpdateClassInfo", 16, "t")},
        "ClientZoneIpcType": {10: ("ActionRequest", 32, "t"), 11: ("ClientTrigger", 32, "t")}}}
    YOU, OTHER, ENEMY = 0x1000_0001, 0x1000_0002, 0x4000_0001

    def frames(self) -> bytes:
        import struct

        t = 1_760_000_000_000

        def frame(ms, src, opcode, data, direction=1):
            ipc = struct.pack("<HHHHII", 0x14, opcode, 0, 0, (t + ms) // 1000, 0) + data
            body = struct.pack("<IIIHH", 16 + len(ipc), src, self.YOU, 3, 0) + ipc
            header = bytes(16) + struct.pack("<QIHHBBHI", t + ms, 40 + len(body), 1, 1, 1, 0, 0, 0)
            return _delimited(_pb([(1, _pb([(1, 1), (2, direction)])), (2, header + body)]))

        def spawn(lead, job, level, base=0):
            common = bytearray(560)
            struct.pack_into("<II", common, 48, base, 0)
            struct.pack_into("<II", common, 76, 1000, 1000)
            common[113], common[118], common[119] = (2 if base else 1), level, job
            return bytes(lead) + bytes(common) + bytes(640 - lead - 560)

        def effect(action, sequence, target, kind, value):
            data = bytearray(124)
            struct.pack_into("<I", data, 8, action)
            struct.pack_into("<f", data, 16, 0.6)
            struct.pack_into("<H", data, 24, sequence)
            data[31], data[33] = 1, 1
            struct.pack_into("<BBBBBBH", data, 42, kind, 0, 0, 0, 0, 0, value)
            struct.pack_into("<I", data, 112, target)
            return bytes(data)

        def result(status, seconds, source):
            data = bytearray(96)
            struct.pack_into("<IIH", data, 12, 900, 1000, 10000)
            data[25] = 1
            struct.pack_into("<BBHHHfI", data, 28, 0, 0, status, 0, 0, seconds, source)
            return bytes(data)

        def request(action, sequence, target):
            return struct.pack("<IBBH", action, 0, 1, sequence) + bytes(8) + struct.pack("<II", target, 0) + bytes(8)

        out = frame(0, self.YOU, 716, _zone_init(132, 0))
        out += frame(10, self.YOU, 7, struct.pack("<BBHHH", 26, 0, 30, 30, 0) + bytes(8))
        out += frame(20, self.OTHER, 2, spawn(32, 21, 50)) + frame(20, self.ENEMY, 1, spawn(16, 0, 30, base=77))
        out += frame(30, self.YOU, 5, bytes([26]) + bytes(15))
        for k in range(8):  # eight uses 2.5 s apart, each asked for 0.1 s before it lands
            ms = 1000 + k * 2500
            out += frame(ms - 100, self.YOU, 10, request(163, k + 1, self.ENEMY), direction=2)
            out += frame(ms, self.YOU, 3, effect(163, k + 1, self.ENEMY, 3, 100))
        out += frame(21000, self.YOU, 3, effect(164, 9, self.ENEMY, 14, 179))
        out += frame(21100, self.ENEMY, 4, result(179, 30.0, self.YOU))
        out += frame(21200, self.YOU, 5, bytes([26, 0, 0, 0, 0, 0, 0, 0, 2]) + bytes(7))
        out += frame(21300, self.OTHER, 3, effect(31, 0, self.ENEMY, 3, 50))
        out += frame(21400, self.ENEMY, 6, struct.pack("<HHIIII", 9999, 0, self.OTHER, 5, 0, 0) + bytes(4))
        out += frame(21500, self.YOU, 11, struct.pack("<IIIIII", 8888, 1, 0, 0, 0, 0) + struct.pack("<II", 0, 0), 2)
        out += frame(21600, self.YOU, 999, bytes(8))
        return out

    def capture(self, folder: Path):
        folder.mkdir()
        version_info, capture_info, _ = _capture_parts("ledgertest", self.VERSION)
        (folder / "VersionInfo").write_bytes(version_info)
        (folder / "CaptureInfo").write_bytes(capture_info)
        (folder / "Data").write_bytes(self.frames())
        return chronofoil.find_captures([folder])[0]

    def test_events_name_nobody(self):
        with tempfile.TemporaryDirectory() as tmp:
            capture = self.capture(Path(tmp) / "cap")
            events = list(ledger.events(capture, self.TABLES))
        text = json.dumps(events)
        for entity in (self.YOU, self.OTHER, self.ENEMY):
            self.assertNotIn(str(entity), text)
        kinds = [event["e"] for event in events]
        self.assertEqual(kinds[:5], ["zone", "class", "spawn", "spawn", "gauge"])
        spawns = [event for event in events if event["e"] == "spawn"]
        self.assertEqual([(s["a"], s["k"], s["job"], s["lv"]) for s in spawns], [(1, "player", 21, 50), (2, "npc", 0, 30)])
        self.assertEqual(spawns[1]["base"], 77)
        hit = next(event for event in events if event["e"] == "effect")
        self.assertEqual((hit["a"], hit["action"], hit["seq"], hit["lock"], hit["hits"][0]["on"]), (0, 163, 1, 0.6, 2))
        given = next(event for event in events if event["e"] == "result")
        self.assertEqual(given["st"], [[179, 0, 30.0, 0]])
        control = next(event for event in events if event["e"] == "control")
        self.assertEqual((control["cat"], control["args"][:2]), (9999, ["a1", 5]))
        self.assertEqual(events[-1]["e"], "coverage")
        self.assertEqual(events[-1]["unnamed"]["out"], {"999/8": 1})

    def test_fact_tables(self):
        with tempfile.TemporaryDirectory() as tmp:
            capture = self.capture(Path(tmp) / "cap")
            cache = Path(tmp) / "ledgers"

            def ledgers():
                return [(capture, list(ledger.events(capture, self.TABLES)))]

            uses = ledger_facts.uses(ledgers())
            self.assertEqual(uses["ACN:30"]["uses"], {"163": 8, "164": 1})
            self.assertEqual(uses["WAR:50"]["uses"], {"31": 1})
            statuses = ledger_facts.statuses(ledgers())
            self.assertEqual(statuses["player"]["164"]["gives"][0]["status"], 179)
            self.assertEqual(statuses["player"]["164"]["gives"][0]["durations"], {"30": 1})
            gauges = ledger_facts.gauges(ledgers())
            self.assertEqual(gauges["ACN"]["after"], {"164": [[7, 0, 2, 1]]})
            actions = ledger_facts.actions(ledgers())
            self.assertEqual((actions["163"]["own"], actions["163"]["lock"], actions["163"]["shortest_gap"],
                              actions["163"]["request_to_effect"]), (8, 0.6, 2.5, 0.1))
            controls = ledger_facts.controls(ledgers())
            self.assertEqual([(r["category"], r["known"]) for r in controls["control"]], [(9999, False)])
            self.assertEqual(controls["trigger"][0]["category"], 8888)
            coverage = ledger_facts.coverage(ledgers())
            self.assertEqual(coverage[self.VERSION]["out"]["packets"], coverage[self.VERSION]["out"]["named"] + 1)
            self.assertIn("999/8: 1", ledger_facts.coverage_text(coverage))
            # An unfinished capture (a folder) isn't cached.
            self.assertEqual(len(list(ledger.cached(capture, self.TABLES, cache))), len(ledgers()[0][1]))
            self.assertFalse(cache.exists())


class RotationSimTest(unittest.TestCase):
    """Every bot rotation, run with its real action scripts: no script error, no bot standing idle."""

    def test_every_rotation_runs(self):
        if rotation_sim.lua54 is None:
            self.skipTest("lupa not installed")
        self.assertEqual(rotation_sim.check(seconds=60), [])

    def test_summoner_summons(self):
        if rotation_sim.lua54 is None:
            self.skipTest("lupa not installed")
        run = rotation_sim.simulate(27, 80, 120)
        # Bahamut, then the three arcana and their gem spells, as retail's Summoners (rotations/Summoner.lua).
        for action in (7427, 25820, 25805, 25806, 25807, 25823, 25824, 25825):
            self.assertIn(action, run["uses"])
        self.assertFalse(run["errors"])


class DutySimTest(unittest.TestCase):
    """A duty's director script against a retail run, and a wipe rehearsed in its boss fight (a made-up script and
    run.)"""

    SCRIPT = """
BOSS = 5000001
ADD = 5000002
EOBJ_DOOR = 2000001
pulled = false
add_came = false
door_open = false
function onSetup(director) director:set_sequence(1) end
function onCommence(director) end
function onUpdate(director, health, player_health, info)
    if not pulled and info[BOSS] ~= nil and info[BOSS].target ~= nil then
        pulled = true
        director:set_bgm(37)
    end
    if pulled and not add_came and health[BOSS] ~= nil and health[BOSS] < 80 then
        add_came = true
        director:delay(2)
        director:spawn_bnpc(ADD)
        director:delay(0)
    end
end
function onGimmickAccessor(director, actor, id) if id == EOBJ_DOOR then director:event_action(24, actor, id) end end
function onEventActionCast(director, actor, target) door_open = true director:hide_eobj(target) end
function onActorDeath(director, layout)
    if layout == BOSS and door_open then
        director:set_bgm(0)
        director:objective(0)
        director:complete_duty()
    end
end
"""
    RESET = "function onPartyWipe(director) pulled = false add_came = false director:despawn_bnpc(ADD) end\n"

    def fixture(self, script: Path) -> dict:
        life = {"spawn": None, "pulled": 10.0, "death": 70.0, "max_hp": 1000, "base": 1, "gone": 80.0,
                "hp": [[10.0, 99.0], [20.0, 79.0], [45.0, 40.0], [70.0, 0.0]], "pos": [[0.0, 1.0, 0.0, 2.0]]}
        add = {"spawn": 22.0, "pulled": 22.0, "death": 30.0, "max_hp": 100, "base": 2, "gone": 40.0,
               "hp": [[22.0, 100.0], [30.0, 0.0]], "pos": [[22.0, 3.0, 0.0, 2.0]]}
        return {"duty": 999, "script": str(script), "source": "made up", "tolerance": 3.0,
                "npcs": {"5000001": [life], "5000002": [add]},
                "player": {"pos": [[0.0, 0.0, 0.0, 0.0]], "hp": []}, "party": {}, "casts": [], "wipes": [],
                "bars": [[5.0, 24, 2.0]],
                "events": [[10.0, "bgm", 37], [70.0, "bgm", 0], [70.0, "objective", 0], [70.0, "complete"]]}

    def test_run_matches_and_a_wipe_is_rehearsed(self):
        with tempfile.TemporaryDirectory() as tmp:
            script = Path(tmp) / "made.lua"
            script.write_text(self.SCRIPT + self.RESET)
            fixture = self.fixture(script)
            run = duty_sim.simulate(fixture, "made")
            self.assertIsNone(run.error)
            self.assertEqual(run.problems, [])
            self.assertEqual([(kind, value) for kind, value, _, _ in run.matched],
                             [("bgm", 37), ("bgm", 0), ("objective", 0), ("complete", None)])
            # The door's bar (5 s, 2 s long) opened it, and the add came 2 s after the boss went below 80 % (the delay.)
            calls = {name: t for t, name, _ in run.calls}
            self.assertEqual((calls["hide_eobj"], calls["spawn_bnpc"]), (7.0, 22.0))
            wipes = duty_sim.rehearse_wipes(fixture, "made")
            self.assertEqual((wipes.error, wipes.problems), (None, []))
            self.assertEqual(len(wipes.matched), 1)
            # Against what's written down: nothing new, nothing gone.
            self.assertEqual(duty_sim.against_known([(run, wipes)], {}), ([], 0))
            self.assertEqual(duty_sim.against_known([(run, wipes)], {"made": ["an old one"]})[0],
                             ["made: no longer happens (take it off duties/known.json): an old one"])

    def test_a_script_that_forgets_a_fight_at_a_wipe(self):
        with tempfile.TemporaryDirectory() as tmp:
            script = Path(tmp) / "made.lua"
            script.write_text(self.SCRIPT)  # no onPartyWipe: `pulled` and `add_came` stay set
            fixture = self.fixture(script)
            self.assertEqual(duty_sim.simulate(fixture, "made").problems, [])
            wipes = duty_sim.rehearse_wipes(fixture, "made")
            self.assertEqual(sorted(problem.split(" (")[0] for problem in wipes.problems),
                             ["boss 5000001: set_bgm", "boss 5000001: spawn_bnpc"])
            changes, standing = duty_sim.against_known([(duty_sim.simulate(fixture, "made"), wipes)],
                                                       {"made": wipes.problems})
            self.assertEqual((changes, standing), ([], 2))

    def test_late_and_missing(self):
        with tempfile.TemporaryDirectory() as tmp:
            script = Path(tmp) / "made.lua"
            script.write_text(self.SCRIPT + self.RESET)
            fixture = self.fixture(script)
            fixture["bars"] = []  # nobody opens the door: the script never completes the duty
            fixture["events"][0][0] = 2.0  # and retail's music began 8 s before the script's
            problems = duty_sim.simulate(fixture, "made").problems
            self.assertIn("bgm 37: 10.0 s, retail 2.0 s", problems)
            self.assertIn("complete at 70.0 s on retail: the script never did it", problems)

    def test_fixture_from_a_run(self):
        you, boss, mate = 0x1000_0001, 0x4000_0001, 0x1000_0002
        def director(t, event, arg=0):
            return {"S": t, "Id": you, "Category": 109, "Params": [0x8003_0001, event, arg, 0], "Packet": "ActorControlSelf"}
        run = {
            "ContentFinderCondition": 999, "Source": "chronofoil abcdef0123 2026.10.16",
            "Actors": {str(you): {"Kind": "you", "MaxHp": 1000, "Seen": 0.0, "Position": [0, 0, 0]},
                       str(mate): {"Kind": "player", "MaxHp": 1000, "Seen": 0.0, "Position": [0, 0, 0]},
                       str(boss): {"Kind": "battle npc", "LayoutId": 5000001, "BaseId": 77, "MaxHp": 2000, "Seen": 12.0,
                                   "LastSeen": 90.0, "Died": 80.0, "Position": [5.0, 0.0, 5.0], "Battalion": 4}},
            "Samples": [{"S": 12.0, "Id": boss, "Hp": 2000, "P": [5.0, 0.0, 5.0]}, {"S": 30.0, "Id": boss, "T": you},
                        {"S": 40.0, "Id": boss, "Hp": 1000}, {"S": 80.0, "Id": boss, "Hp": 0},
                        {"S": 11.0, "Id": you, "P": [1.0, 0.0, 1.0]}, {"S": 20.0, "Id": mate, "P": [2.0, 0.0, 2.0]}],
            "Actions": [],
            "Events": [{"S": 31.0, "Kind": "cast", "Id": boss, "Text": "action 903 spell 903"},
                       {"S": 32.0, "Kind": "battle talk", "Id": boss, "Text": "text 3700 name 1644 icon 0 5000 ms"}],
            "Controls": [{"S": 33.0, "Id": boss, "Category": 407, "Params": [1415, 0, 0, 0], "Packet": "ActorControl"},
                         director(10.0, 0x40000001, 5400), director(30.0, 0x80000001, 37), director(80.0, 0x80000001),
                         director(80.0, 0x80000016), director(80.0, 0x40000003), director(50.0, 0x40000005),
                         director(60.0, 0x40000006),
                         {"S": 15.0, "Id": mate, "Category": 3, "Params": [1, 24, 0, 0], "Packet": "ActorControlTarget"},
                         {"S": 17.0, "Id": mate, "Category": 3, "Params": [0, 24, 0, 0], "Packet": "ActorControlTarget"}],
        }
        fixture = duty_sim.fixture_from_run(run, "made.lua")
        life, = fixture["npcs"]["5000001"]
        self.assertEqual((life["spawn"], life["pulled"], life["death"], life["max_hp"]), (2.0, 20.0, 70.0, 2000))
        self.assertEqual(life["hp"], [[2.0, 100.0], [30.0, 50.0], [70.0, 0.0]])
        # The fight's own timeline too (since 2026-11-11): the boss coming out, its battle talk and ActionTimelines.
        self.assertEqual(fixture["events"], [[2.0, "appear", 5000001], [20.0, "bgm", 37], [22.0, "talk", 3700],
                                             [23.0, "timeline", [5000001, 1415]], [70.0, "bgm", 0],
                                             [70.0, "complete"], [70.0, "objective", 0]])
        self.assertEqual((fixture["wipes"], fixture["bars"], fixture["casts"]),
                         ([[40.0, 50.0]], [[5.0, 24, 2.0]], [[21.0, 5000001, 903]]))
        self.assertEqual(list(fixture["party"]), ["party1"])
        with tempfile.TemporaryDirectory() as tmp:
            path = duty_sim.save(fixture, Path(tmp) / "999-made.json")
            self.assertEqual(json.loads(path.read_text()), fixture)


class DutySimFightKindsTest(unittest.TestCase):
    """A fixture's talk, timeline and appear events are compared only where it has them (older fixtures don't.)"""

    def test_only_the_fixtures_kinds(self):
        calls = [(5.0, "battle_talk", [4246177, 1644, 3700, 0, 5000]), (6.0, "complete_duty", [])]
        result = duty_sim.SimResult("x", Path("x.lua"))
        old = {"events": [[6.0, "complete"]]}
        self.assertEqual(duty_sim.compare(old, calls, result), [])
        new = {"events": [[5.5, "talk", 3700], [6.0, "complete"]]}
        self.assertEqual(duty_sim.compare(new, calls, result), [])
        late = {"events": [[1.0, "talk", 3700], [6.0, "complete"]]}
        self.assertEqual(duty_sim.compare(late, calls, result), ["talk 3700: 5.0 s, retail 1.0 s"])


class LibZstdTest(unittest.TestCase):
    """Finished Chronofoil captures read through the system's libzstd when neither `zstandard` nor Python 3.14's
    module is there."""

    def test_round_trip(self):
        import ctypes
        import ctypes.util
        import io

        name = ctypes.util.find_library("zstd")
        if not name:
            self.skipTest("no libzstd")
        lib = ctypes.CDLL(name)
        lib.ZSTD_compressBound.restype = ctypes.c_size_t
        lib.ZSTD_compressBound.argtypes = [ctypes.c_size_t]
        lib.ZSTD_compress.restype = ctypes.c_size_t
        lib.ZSTD_compress.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t, ctypes.c_int]
        data = bytes(range(256)) * 5000
        out = ctypes.create_string_buffer(lib.ZSTD_compressBound(len(data)))
        size = lib.ZSTD_compress(out, len(out), data, len(data), 3)
        reader = io.BufferedReader(chronofoil._LibZstdReader.open(io.BytesIO(out.raw[:size])))
        self.assertEqual(reader.read(1), b"\x00")
        self.assertEqual(reader.read(), data[1:])
