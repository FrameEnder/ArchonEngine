"""Tests for questkit, on made-up data only (no game files): python3 -m unittest discover tools/questkit/tests"""

import gzip
import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from questkit import capture, compare, fixtures, recording, replay, scaffold, triage  # noqa: E402
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


if __name__ == "__main__":
    unittest.main()
