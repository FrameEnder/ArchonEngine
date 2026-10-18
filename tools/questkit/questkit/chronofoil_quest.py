"""What a quest did on retail, read from Chronofoil captures: every talk, scene (flags, parameters, how the client
answered it), quest update, notice, and what else the server did while the quest's event was open (zone changes,
aetheryte and unlock packets).

Chronofoil keeps every packet, so it sees what Kawari Recorder's quest log can't: talks that change nothing (agreeing
to a quest battle), the scene numbers and flags, whether the client answered a scene as a yield or a return, and the
packets around them. The packet layouts are Kawari's (core/src/ipc/zone):

* StartTalkEvent (client): actor u64, handler u32 @8. For event NPCs and objects the client spawned itself the actor
  is their layout ID.
* EventStart: actor u64, handler u32 @8, event type u8 @12 (1 talk, 15 entering a zone), flags u8 @13, argument u32
  @16.
* EventScene2/4/8/16/32/64/128/255: actor u64, handler u32 @8, scene u16 @12, flags u32 @16, u32 @20, parameter
  count u8 @24, parameters u32 from @28.
* EventReturnHandler* / EventYieldHandler* (client): handler u32, scene u16 @4, error code (or yield ID) u8 @6,
  result count u8 @7, results i32 from @8.
* EventNotice*: handler u32, notice u8 @4, argument count u8 @5, arguments u32 from @8.
* AcceptQuest / FinishQuest: quest u16. UpdateQuest: quest u16 @4, sequence u8 @6, variables @14..20.
* PrepareZoning: territory u16 @4. ActorControlSelf: category u32, then u32 parameters (509 LearnTeleport: aetheryte;
  41 ToggleUnlock: UnlockLink).

Captures carry other players and chat; only these numbers come out of them.
"""

from __future__ import annotations

import struct
from dataclasses import dataclass, field

from .chronofoil import Names

QUEST_HANDLER_BASE = 0x10000
EVENT_TALK = 1
EVENT_ENTER_TERRITORY = 15
LEARN_TELEPORT = 509
TOGGLE_UNLOCK = 41
# A zone change this soon after the quest's event finished still belongs to it (joining a quest battle: EventFinish,
# then PrepareZoning 0.1 s later.)
AFTER_FINISH = 1.0


@dataclass
class Event:
    t: float
    zone: int | None
    kind: str  # talk, start, scene, answer, accept, update, notice, finish_quest, finish, zone, aetheryte, unlock
    data: dict = field(default_factory=dict)


def _u32s(data: bytes, offset: int, count: int, signed=False) -> list[int]:
    count = max(0, min(count, (len(data) - offset) // 4))
    return list(struct.unpack_from(("<%di" if signed else "<%dI") % count, data, offset)) if count else []


def quest_events(captures, tables: dict, quests=None) -> dict[int, list[Event]]:
    """Each quest's events, in order, across the captures. `quests`: only these quest numbers."""
    wanted = None if quests is None else {QUEST_HANDLER_BASE + q for q in quests}
    flows: dict[int, list[Event]] = {}

    for capture in captures:
        names = Names(tables, capture.game_version)
        zone = None
        open_events: list[int] = []  # quest handlers with an open event, newest last
        finished: dict[int, float] = {}  # handler -> when its event last finished

        def add(handler: int, t: float, kind: str, **data):
            if handler < QUEST_HANDLER_BASE or handler >= QUEST_HANDLER_BASE + 0x10000:
                return
            if wanted is not None and handler not in wanted:
                return
            flows.setdefault(handler - QUEST_HANDLER_BASE, []).append(Event(t, zone, kind, data))

        def current(t: float) -> int | None:
            """The quest whose event is open (or just finished), for packets that don't name one."""
            if open_events:
                return open_events[-1]
            recent = [(when, h) for h, when in finished.items() if t - when <= AFTER_FINISH]
            return max(recent)[1] if recent else None

        for segment in capture.segments(protocols=("zone",)):
            name = names.name(segment)
            d = segment.data
            t = segment.t
            try:
                if segment.dir == "out" and name in ("ZoneInit", "InitZone"):
                    zone = struct.unpack_from("<H", d, 2)[0]
                    open_events.clear()
                elif name == "StartTalkEvent" and segment.dir == "in":
                    actor, handler = struct.unpack_from("<II", d, 0)[0], struct.unpack_from("<I", d, 8)[0]
                    add(handler, t, "talk", actor=actor)
                elif name == "EventStart":
                    handler = struct.unpack_from("<I", d, 8)[0]
                    if QUEST_HANDLER_BASE <= handler < QUEST_HANDLER_BASE + 0x10000:
                        if handler in open_events:
                            open_events.remove(handler)
                        open_events.append(handler)
                    add(handler, t, "start", actor=struct.unpack_from("<I", d, 0)[0], type=d[12], flags=d[13],
                        arg=struct.unpack_from("<I", d, 16)[0])
                elif name.startswith("EventScene") and len(d) >= 28:
                    handler = struct.unpack_from("<I", d, 8)[0]
                    add(handler, t, "scene", scene=struct.unpack_from("<H", d, 12)[0],
                        flags=struct.unpack_from("<I", d, 16)[0], params=_u32s(d, 28, d[24]))
                elif name.startswith(("EventReturnHandler", "EventYieldHandler")) and segment.dir == "in":
                    handler = struct.unpack_from("<I", d, 0)[0]
                    add(handler, t, "answer", scene=struct.unpack_from("<H", d, 4)[0],
                        yielded=name.startswith("EventYield"), code=d[6], results=_u32s(d, 8, d[7], signed=True))
                elif name.startswith("EventNotice"):
                    handler = struct.unpack_from("<I", d, 0)[0]
                    add(handler, t, "notice", notice=d[4], args=_u32s(d, 8, d[5]))
                elif name == "AcceptQuest":
                    add(QUEST_HANDLER_BASE + struct.unpack_from("<H", d, 0)[0], t, "accept")
                elif name == "FinishQuest":
                    add(QUEST_HANDLER_BASE + struct.unpack_from("<H", d, 0)[0], t, "finish_quest")
                elif name == "UpdateQuest":
                    add(QUEST_HANDLER_BASE + struct.unpack_from("<H", d, 4)[0], t, "update", sequence=d[6],
                        vars=d[14:20].hex())
                elif name == "EventFinish":
                    handler = struct.unpack_from("<I", d, 0)[0]
                    add(handler, t, "finish")
                    if handler in open_events:
                        open_events.remove(handler)
                        finished[handler] = t
                elif name == "PrepareZoning" and segment.dir == "out":
                    handler = current(t)
                    if handler is not None:
                        add(handler, t, "zone", territory=struct.unpack_from("<H", d, 4)[0] & 0x7FFF)
                elif name == "ActorControlSelf" and segment.dir == "out" and open_events:
                    category, first = struct.unpack_from("<II", d, 0)
                    if category == LEARN_TELEPORT:
                        add(open_events[-1], t, "aetheryte", aetheryte=first)
                    elif category == TOGGLE_UNLOCK:
                        add(open_events[-1], t, "unlock", unlock=first)
            except struct.error:
                continue
    return flows


def object_bases(captures, tables: dict) -> dict[int, int]:
    """Entity and layout IDs to BNpcBase / EObj base IDs, from the server's spawns in the captures (objects the server
    spawns itself; event NPCs the client spawns come from Kawari Recorder's world.json instead.)"""
    bases: dict[int, int] = {}
    for capture in captures:
        names = Names(tables, capture.game_version)
        for segment in capture.segments(protocols=("zone",)):
            if segment.dir != "out":
                continue
            name = names.name(segment)
            d = segment.data
            try:
                if name == "SpawnObject" and len(d) >= 24:
                    # SpawnObject: spawn index u8, kind u8, flags, u8, base ID u32 @4, entity ID u32 @8, layout ID @12
                    base, entity, layout = struct.unpack_from("<III", d, 4)
                    bases.setdefault(entity, base)
                    if layout:
                        bases.setdefault(layout, base)
            except struct.error:
                continue
    return bases


def describe(events: list[Event]) -> str:
    """A readable list of a quest's events (questkit chronofoil --quest)."""
    lines = []
    t0 = events[0].t if events else 0
    for e in events:
        d = dict(e.data)
        if "flags" in d and e.kind == "scene":
            d["flags"] = hex(d["flags"])
        detail = " ".join(f"{k} {v}" for k, v in d.items())
        lines.append(f"{e.t - t0:9.2f}  z{e.zone}  {e.kind:12} {detail}")
    return "\n".join(lines)
