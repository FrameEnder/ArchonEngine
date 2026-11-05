"""questkit triage: the first read of a diagnostics bundle, as one Markdown report.

* what Bob was doing (the note, the build, a Quest Lab test if there was one);
* the server's view of each character's quests against the client's (Kawari Probe snapshots);
* the log's warnings and errors, grouped, with Lua errors and dropped connections called out;
* per player in the captures: unknown packets, events left open, client replies the server never answered, slow
  responses, late casts, and a timeline of the quest/event packets leading up to the bundle.
"""

from __future__ import annotations

import json

import re
from collections import Counter, defaultdict
from datetime import datetime, timezone

from . import capture, repo
from .bundle import Bundle

# Packets that say nothing about what the player did (movement, enmity lists, spawns), skipped when looking for the
# server's reaction to something.
NOISE = {
    "ActorMove", "HaterList", "EnmityList", "UpdateHpMpTp", "PingSyncReply", "SpawnNpc", "DeleteActor", "SpawnObject",
    "DeleteObject", "KeepAliveRequest", "KeepAliveResponse", "UpdatePositionHandler", "PingSync", "StatusEffectList",
    "ActorGauge", "UnkFate",
}
# Client packets that expect the server to do something about them.
NEEDS_REPLY = ("StartTalkEvent", "StartWalkInEvent", "ActionRequest", "EnterTerritoryEvent")
SLOW_REPLY = 1.0  # seconds
LATE_CAST = 0.3  # seconds past the cast time

LOG_PATTERNS = [
    ("Lua errors", re.compile(r"(Error while calling|Syntax error|runtime error|attempt to (call|index|compare|perform))",
                              re.I)),
    ("Panics", re.compile(r"panicked")),
    ("Unknown packets", re.compile(r"(unknown|Unknown) (packet|opcode)|probably (is|be) ")),
    ("Dropped connections", re.compile(r"(Dropping zone connection|was killed|Forcefully logging out|Removing client|"
                                       r"queue is full|isn't in the instance)")),
]


def clock(t: float | None) -> str:
    if t is None:
        return "?"
    return datetime.fromtimestamp(t, timezone.utc).strftime("%H:%M:%S.%f")[:-3]


def normalize(message: str) -> str:
    """Messages with their numbers taken out, so the same warning about different IDs groups together."""
    first = message.split("\n", 1)[0]
    return re.sub(r"\d+(\.\d+)?", "#", first)[:200]


def summary_of(packet: capture.Packet) -> str:
    """One short line about an event/quest packet."""
    text = packet.decoded
    handler = capture.handler_of(packet)
    where = f"{handler[0].lower()} {handler[1]}" if handler else ""
    if packet.name == "UpdateQuest":
        match = capture._ACTIVE_QUEST.search(text)
        if match:
            values = bytes(int(v) for v in match.group(5).split(",") if v.strip())
            return f"quest {match.group(1)} seq {match.group(2)} vars {values.hex(' ')}"
    if packet.name.startswith("EventNotice"):
        parsed = capture.parse_notice(packet.data)
        if parsed:
            return f"{where} notice {parsed[1]} {parsed[2]}"
    if packet.name.startswith("EventScene"):
        return f"{where} scene {capture.scene_of(packet)}"
    if packet.name.startswith(("EventReturn", "EventYield")):
        params = re.search(r"params: \[([^\]]*)\]", text)
        return f"{where} scene {capture.scene_of(packet)} -> [{params.group(1) if params else ''}]"
    if packet.name in ("AcceptQuest", "FinishQuest"):
        match = re.search(r"quest_id: (\d+)", text)
        return f"quest {match.group(1) if match else '?'}"
    if packet.name in ("StartTalkEvent", "EventStart"):
        target = re.search(r"ObjectId \((\d+)\)", text)
        return f"{where} with {target.group(1) if target else '?'}"
    if packet.name == "StartWalkInEvent":
        arg = re.search(r"event_arg: (\d+)", text)
        return f"{where} range {arg.group(1) if arg else '?'}"
    if packet.name == "EventFinish":
        return where
    if packet.name == "Unknown" and packet.hex:
        raw = bytes.fromhex(packet.hex)
        return f"opcode {int.from_bytes(raw[2:4], 'little')}, {len(raw) - capture.IPC_HEADER} bytes"
    return text[:100]


class Report:
    def __init__(self):
        self.lines: list[str] = []

    def add(self, line: str = ""):
        self.lines.append(line)

    def text(self) -> str:
        return "\n".join(self.lines).rstrip() + "\n"


def triage(bundle: Bundle, timeline_items: int = 60) -> str:
    report = Report()
    meta = bundle.meta
    build = meta.get("build", {})
    report.add(f"# Triage: {bundle.path.name}")
    report.add()
    if bundle.note:
        report.add(f"> {bundle.note}")
        report.add()
    report.add(f"* Bundle made {meta.get('created', '?')}, window {meta.get('window_minutes', '?')} minutes")
    report.add(f"* Build `{build.get('short', '?')}` ({build.get('message', '')}), game {build.get('game_version', '?')},"
               f" Kawari Web {meta.get('kawari_web', '?')}")
    test = bundle.quest_test
    if test:
        report.add(f"* **Quest test**: quest {test.get('quest')} ({test.get('name', '')}) on {test.get('character', '?')},"
                   f" from sequence {test.get('sequence', '?')}, started {test.get('started', '?')}")
    report.add()

    _quests(report, bundle)
    _log(report, bundle)
    _captures(report, bundle, timeline_items, test)
    return report.text()


# --- Quest state: server against client ---------------------------------------------------------------------------

def _quests(report: Report, bundle: Bundle):
    report.add("## Quests: server and client")
    report.add()
    snapshots = bundle.snapshots()
    latest_by_name = {}
    for snap in snapshots:
        name = snap.get("character")
        if name and (name not in latest_by_name or snap.get("client_time", "") > latest_by_name[name].get("client_time", "")):
            latest_by_name[name] = snap
    if not bundle.characters:
        report.add("No characters in the bundle.")
        report.add()
    for character in bundle.characters:
        name = character.get("character", {}).get("name", "?")
        volatile = character.get("volatile", {})
        summary = character.get("quest_summary") or {}
        server = {q["id"]: q for q in summary.get("active", [])}
        snap = latest_by_name.get(name)
        report.add(f"### {name}")
        position = volatile.get("position", {})
        report.add(f"Zone {volatile.get('zone_id')}, at ({position.get('x', 0):.1f}, {position.get('y', 0):.1f}, "
                   f"{position.get('z', 0):.1f}), {'online' if volatile.get('is_online') else 'offline'}")
        client = {}
        if snap:
            for quest in snap.get("quests", []):
                client[quest["id"]] = quest
            report.add(f"Client snapshot \"{snap.get('label', '')}\" at {snap.get('client_time', '?')}, territory "
                       f"{snap.get('territory')}, conditions {', '.join(snap.get('conditions', []))}")
        else:
            report.add("No Kawari Probe snapshot for this character, so only the server's side.")
        report.add()
        ids = sorted(set(server) | set(client))
        if not ids:
            report.add("No active quests.")
            report.add()
            continue
        report.add("| Quest | Server seq | Server vars | Client seq | Client vars | |")
        report.add("|---|---|---|---|---|---|")
        for quest_id in ids:
            s = server.get(quest_id)
            c = client.get(quest_id)
            s_vars = ""
            if s and "variables" in s:
                s_vars = bytes(s["variables"]).hex(" ")
            elif s:
                s_vars = s.get("variables_hex", "")
            c_vars = ""
            if c:
                c_vars = " ".join(c.get("variables_hex", "")[i:i + 2] for i in range(0, len(c.get("variables_hex", "")), 2))
            verdict = ""
            if s and c:
                same = s.get("sequence") == c.get("sequence") and s_vars.replace(" ", "") == c_vars.replace(" ", "")
                verdict = "same" if same else "**differ**"
            elif s and snap:
                verdict = "**client doesn't have it**"
            elif c:
                verdict = "**server doesn't have it**"
            report.add(f"| {quest_id} | {s.get('sequence') if s else ''} | `{s_vars}` | {c.get('sequence') if c else ''} |"
                       f" `{c_vars}` | {verdict} |")
        if snap:
            report.add()
            report.add("The snapshot may be older than the server's save; check its time against the log.")
        report.add()
    _stuck(report, snapshots)


def _stuck(report: Report, snapshots: list[dict]):
    """What each snapshot's client said was stopping the player (Kawari Probe 0.7.0's `stuck`, /kprobe stuck.)"""
    with_stuck = [snap for snap in snapshots if isinstance(snap.get("stuck"), dict)]
    if not with_stuck:
        return
    report.add("## What the client said was stopping the player")
    report.add()
    for snap in sorted(with_stuck, key=lambda s: s.get("client_time", "")):
        stuck = snap["stuck"]
        report.add(f"### \"{snap.get('label', '')}\" at {snap.get('client_time', '?')} ({snap.get('character', '?')})")
        report.add()
        for label, key in (("Conditions", "conditions"), ("Character mode", "character_mode"),
                           ("Animation lock (s)", "animation_lock"), ("Casting", "casting"), ("Queued", "queued"),
                           ("Action sequence (used, handled)", "last_action_sequence"),
                           ("Pending item moves", "pending_item_moves"), ("Statuses", "statuses")):
            value = stuck.get(key)
            if isinstance(value, list):
                value = ", ".join(json.dumps(v) if isinstance(v, dict) else str(v) for v in value) or "none"
            elif isinstance(value, dict):
                value = json.dumps(value)
            if key == "character_mode" and stuck.get("character_mode_param") is not None:
                value = f"{value} ({stuck['character_mode_param']})"
            report.add(f"* {label}: {value if value is not None else '-'}")
        events = stuck.get("events") or []
        if events:
            report.add(f"* Refused actions and log messages (the last {min(len(events), 20)}):")
            for event in events[-20:]:
                report.add(f"  * `{event}`")
        failed = {k: v for k, v in (stuck.get("hooks") or {}).items() if v != "ok"}
        if failed:
            report.add(f"* Hooks that failed (that part is missing): {failed}")
        report.add()


# --- Log ------------------------------------------------------------------------------------------------------------

def _log(report: Report, bundle: Bundle):
    lines = bundle.log()
    report.add("## Server log")
    report.add()
    if not lines:
        report.add("No server.log.")
        report.add()
        return
    levels = Counter(line.level for line in lines)
    report.add(f"{len(lines)} lines from {clock(lines[0].time)} to {clock(lines[-1].time)} UTC: "
               + ", ".join(f"{count} {level}" for level, count in levels.most_common()))
    report.add()
    for title, pattern in LOG_PATTERNS:
        hits = [line for line in lines if pattern.search(line.message)]
        if not hits:
            continue
        report.add(f"### {title} ({len(hits)})")
        report.add()
        shown = set()
        for line in hits:
            key = normalize(line.message)
            if key in shown:
                continue
            shown.add(key)
            report.add("```text")
            report.add(f"{clock(line.time)} {line.level} {line.target}: {line.message[:1500]}")
            report.add("```")
            if len(shown) >= 8:
                break
        report.add()
    grouped: dict[str, list] = defaultdict(list)
    for line in lines:
        if line.level in ("WARN", "ERROR"):
            grouped[f"{line.level} {line.target}: {normalize(line.message)}"].append(line)
    if grouped:
        report.add("### Warnings and errors, grouped")
        report.add()
        report.add("| Count | First | Last | Message |")
        report.add("|---|---|---|---|")
        for key, group in sorted(grouped.items(), key=lambda kv: -len(kv[1]))[:25]:
            message = key.replace("|", "\\|")
            report.add(f"| {len(group)} | {clock(group[0].time)} | {clock(group[-1].time)} | `{message}` |")
        report.add()


# --- Captures ---------------------------------------------------------------------------------------------------

def _captures(report: Report, bundle: Bundle, timeline_items: int, test: dict | None):
    files = bundle.capture_files()
    report.add("## Packet captures")
    report.add()
    if not files:
        report.add("No packet captures (turn them on in Kawari Web's Diagnostics page).")
        report.add()
        return
    packets = bundle.packets()
    names = {}
    for character in bundle.characters:
        actor = character.get("character", {}).get("actor_id")
        if actor:
            names[actor] = character.get("character", {}).get("name", str(actor))
    report.add(f"{len(files)} file(s), {len(packets)} packets from {clock(packets[0].t)} to {clock(packets[-1].t)} UTC.")
    report.add()
    # The moments Bob marked in game (`!mark`, the world server's note lines starting with MARK.)
    marks = [p for p in packets if p.dir == "note" and p.text.startswith("MARK")]
    if marks:
        report.add("**Marks** (`!mark` in game):")
        report.add()
        for mark in marks:
            report.add(f"* {clock(mark.t)} UTC: {mark.text[len('MARK'):].strip()}")
        report.add()

    by_actor: dict[int, list[capture.Packet]] = defaultdict(list)
    for packet in packets:
        by_actor[packet.src].append(packet)
    test_quest = int(test["quest"]) if test and str(test.get("quest", "")).isdigit() else None

    players = set(names) | set(capture.actors(packets))
    for actor, own in by_actor.items():
        if actor not in players:
            continue
        report.add(f"### {names.get(actor, actor)}")
        report.add()
        _unknown_packets(report, own)
        _open_events(report, own)
        _unanswered(report, own)
        _late_casts(report, own)
        _timeline(report, own, bundle.created, timeline_items, test_quest)


def _unknown_packets(report: Report, packets: list[capture.Packet]):
    unknown = Counter()
    for packet in packets:
        if packet.name == "Unknown" and packet.hex:
            raw = bytes.fromhex(packet.hex)
            opcode = int.from_bytes(raw[2:4], "little") if len(raw) >= 4 else -1
            unknown[(packet.dir, opcode, len(raw) - capture.IPC_HEADER)] += 1
    if not unknown:
        return
    outstanding = {}
    for entry in repo.fork_opcodes():
        if "size" in entry:
            outstanding.setdefault(entry["size"], []).append(entry["name"])
    report.add("**Unknown packets**")
    report.add()
    for (direction, opcode, size), count in unknown.most_common():
        guess = outstanding.get(size)
        hint = f", the size of {', '.join(guess)}" if guess else ""
        report.add(f"* {count}x {direction} opcode {opcode} ({size} bytes{hint})")
    report.add()


def _open_events(report: Report, packets: list[capture.Packet]):
    """Events the server started but never finished: the client is left waiting in them."""
    open_events: dict[tuple, capture.Packet] = {}
    for packet in packets:
        if packet.dir != "out":
            continue
        if packet.name == "EventStart":
            handler = capture.handler_of(packet)
            if handler:
                open_events[handler] = packet
        elif packet.name == "EventFinish":
            handler = capture.handler_of(packet)
            if handler:
                open_events.pop(handler, None)
        elif packet.name in ("PrepareZoning", "LogOutComplete"):
            open_events.clear()
    if open_events:
        report.add("**Events still open at the end of the capture** (the client waits in them until finished):")
        report.add()
        for handler, packet in open_events.items():
            report.add(f"* {handler[0].lower()} {handler[1]}, started {clock(packet.t)}")
        report.add()


def _unanswered(report: Report, packets: list[capture.Packet]):
    """Client packets the server took long to react to, or never did: a scene's return with nothing after it is a
    script that didn't handle the scene (Pauline standing there.)"""
    rows = []
    for index, packet in enumerate(packets):
        if packet.dir != "in":
            continue
        if not (packet.name in NEEDS_REPLY or packet.name.startswith(("EventReturn", "EventYield"))):
            continue
        reply = None
        for later in packets[index + 1:]:
            if later.t - packet.t > 15:
                break
            if later.dir == "out" and later.name not in NOISE:
                reply = later
                break
        if reply is None:
            rows.append((packet, None))
        elif reply.t - packet.t > SLOW_REPLY:
            rows.append((packet, reply))
    if not rows:
        return
    report.add("**Slow or missing replies**")
    report.add()
    report.add("| Time | Client sent | Server's next packet | After |")
    report.add("|---|---|---|---|")
    for packet, reply in rows[-20:]:
        what = f"{packet.name} ({summary_of(packet)})"
        if reply is None:
            report.add(f"| {clock(packet.t)} | {what} | **nothing within 15 s** | |")
        else:
            report.add(f"| {clock(packet.t)} | {what} | {reply.name} | {reply.t - packet.t:.1f} s |")
    report.add()


_CAST_TIME = re.compile(r"cast_time: ([\d.]+)")
_ACTION_ID = re.compile(r"action_id: (\d+)")


def _late_casts(report: Report, packets: list[capture.Packet]):
    """The player's casts: how long after the cast bar ended the action landed, and warps after a cast (Teleport.)"""
    rows = []
    for index, packet in enumerate(packets):
        if packet.dir != "out" or packet.name != "ActorCast":
            continue
        cast = _CAST_TIME.search(packet.decoded)
        action = _ACTION_ID.search(packet.decoded)
        if not cast or not action:
            continue
        cast_time = float(cast.group(1))
        landed = warp = None
        for later in packets[index + 1:]:
            if later.t - packet.t > cast_time + 20:
                break
            if landed is None and later.dir == "out" and later.name.startswith("ActionEffect") and \
                    f"action_id: {action.group(1)}," in later.decoded:
                landed = later
                continue
            if landed is not None and later.name in ("PrepareZoning", "ActorSetPos"):
                warp = later
                break
            if later.dir == "out" and later.name == "ActorCast":
                break
        if landed is None:
            continue
        late = landed.t - packet.t - cast_time
        warp_delay = (warp.t - landed.t) if warp else None
        if late > LATE_CAST or (warp_delay is not None and warp_delay > 1.5):
            rows.append((packet, action.group(1), cast_time, late, warp_delay))
    if not rows:
        return
    report.add("**Casts that landed late, or warped late**")
    report.add()
    report.add("| Cast at | Action | Cast time | Landed after the bar | Warp after landing |")
    report.add("|---|---|---|---|---|")
    for packet, action, cast_time, late, warp_delay in rows[-20:]:
        report.add(f"| {clock(packet.t)} | {action} | {cast_time:.2f} s | {late:+.2f} s | "
                   f"{'' if warp_delay is None else f'{warp_delay:.1f} s'} |")
    report.add()


def _timeline(report: Report, packets: list[capture.Packet], created: float | None, items: int,
              quest: int | None):
    # Unknown packets go in too: an unknown reply to a scene is a script that never hears back.
    events = [p for p in packets if (capture.is_event_packet(p) and p.name != "QuestActiveList") or p.name == "Unknown"]
    if quest is not None:
        handler_text = f"id: {quest} "
        events = [p for p in events if handler_text in p.decoded or f"quest_id: {quest}," in p.decoded
                  or f"quest_id: {quest} " in p.decoded or (p.name == "UpdateQuest" and f"id: {quest}," in p.decoded)
                  or (p.name.startswith("EventNotice") and (capture.parse_notice(p.data) or (0,))[0] == quest + 0x10000)]
    if not events:
        return
    shown = events[-items:]
    title = f"quest {quest}" if quest is not None else "quests and events"
    report.add(f"**Timeline of {title}** (last {len(shown)} of {len(events)}; time before the bundle in brackets)")
    report.add()
    report.add("```text")
    for packet in shown:
        before = f"[-{created - packet.t:5.0f}s]" if created else ""
        arrow = "->" if packet.dir == "out" else "<-"
        report.add(f"{clock(packet.t)} {before} {arrow} {packet.name:22} {summary_of(packet)}")
    report.add("```")
    report.add()
