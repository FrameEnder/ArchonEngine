"""Tests for questkit, on made-up data only (no game files): python3 -m unittest discover tools/questkit/tests"""

import gzip
import json
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from questkit import battle, capture, compare, fixtures, recording, replay, scaffold, triage  # noqa: E402
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
        heals = battle.ally_outline(b, 3, 0, 120)["role"]["heals"]
        self.assertEqual(heals[0]["targets"], ["player"])
        self.assertEqual(heals[0]["action"], 120)

    def test_outline_in_report(self):
        text = battle.report(self._run())
        self.assertIn("## The director's outline (draft)", text)
        self.assertIn("BATTLE:ally(3802271", text)
        self.assertIn('"change", "sequence 3"', text)


if __name__ == "__main__":
    unittest.main()
