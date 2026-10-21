"""Tests for questkit, on made-up data only (no game files): python3 -m unittest discover tools/questkit/tests"""

import gzip
import json
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from questkit import battle, battle_sim, capture, chronofoil, chronofoil_battle, chronofoil_quest, compare, fixtures, lint, recording, replay, scaffold, triage  # noqa: E402
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


class InZoneMoveTest(unittest.TestCase):
    """Moving the player to a pop range in the zone they're in is retail's PrepareZoning to that same territory: replay
    counts it as a zone change there when the step says where it happens (Coming to Limsa Lominsa's lift.)"""

    def test_move_to_pop_range(self):
        from questkit.harness import Task, _effect
        task = Task("call", "move_to_pop_range", 4127803, True)
        self.assertEqual(_effect(task, 181), {"zone": 181})
        self.assertIsNone(_effect(task, None))


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
        self.assertIn("opcode 0 (72 bytes, the size of EventReturnHandler16)", report)
        self.assertIn("Lua errors", report)

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


class LintTest(unittest.TestCase):
    def test_timeline_actions(self):
        timeline = {"autoattack_action_id": 870, "timepoints": [{"data": {"type": "action", "action_id": 739}},
                                                                {"data": {"type": "invulnerability"}}],
                    "rotation": {"actions": [{"action": 9}]}, "recorded_amounts": {"21": {"npcs": 1}}}
        self.assertEqual(lint.timeline_actions(timeline), {870, 739, 9, 21})

    def test_repository_is_clean(self):
        report = lint.run()
        self.assertTrue(report.ok, lint.format_report(report))
        self.assertIn(870, lint.scripted_actions())
