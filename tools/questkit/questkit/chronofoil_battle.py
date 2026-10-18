"""A Chronofoil capture's duty, as a battle run `questkit battle` reads (the shape of Kawari Recorder's battle files).

Decoded from the packets themselves, with the layouts of Kawari's structs (core/src/ipc/zone/server/):

* actors from SpawnNpc / SpawnPlayer (CommonSpawn: BNpcBase and BNpcName, layout ID, HP, battalion, level, class,
  position), gone with DeleteActor; players other than you are only "player" (their names are never read);
* samples: positions (ActorMove, ActorSetPos), HP (UpdateHpMpTp, EffectResult, EffectResultBasic), targets
  (ActorControlTarget SetTarget, which retail sends for NPCs too) and deaths (ActorControl Defeated);
* actions from ActionEffect1 and AoeEffect8–32: source, action, every target with its effects
  ([kind, value, p0..p4] as the Recorder writes them) and its HP just before;
* events: casts (ActorCast, in the Recorder's text), the director's sequence and variables (DirectorVars), battle talk
  (DirectorPopupMessage), NPC yells, map effects; every ActorControl as a control; HaterList and EnmityList as enmity.

What raw packets don't have and a Recorder file does: names (no game data here; NPCs show as "base N [layout]"), the
Action sheet (shapes, omens, class kits), layout layers, the exact HP between packets, and positions of NPCs that
don't move by ActorMove. Positions are as precise as the packets (packed to about 3 cm in ActorMove).
"""

from __future__ import annotations

import struct

from .chronofoil import EFFECTS, Capture, Names, Visit

DEFEATED = 6
SET_TARGET = 50
SPAWN = 16  # SpawnNpc's own fields before CommonSpawn
OBJECT_KINDS = {1: "player", 2: "battle npc", 3: "event npc", 4: "treasure", 5: "aetheryte", 6: "gathering point",
                7: "event object", 8: "mount", 9: "companion", 10: "retainer", 11: "area object", 12: "housing",
                13: "cutscene object", 14: "card stand", 15: "ornament"}


def _packed(value: int) -> float:
    return round(value / 65535 * 2000 - 1000, 2)


def _rotation(value: int) -> float:
    return round(value / 65535 * 2 * 3.141592653589793 - 3.141592653589793, 3)


def _effects(raw: bytes) -> list[list[int]]:
    out = []
    for k in range(0, len(raw) - 7, 8):
        kind, p0, p1, p2, p3, p4, value = struct.unpack_from("<BBBBBBH", raw, k)
        if kind:
            if p4 & 0x40:  # large amounts: the high bits are in p3 (as the client reads them)
                value += p3 << 16
            out.append([kind, value, p0, p1, p2, p3, p4])
    return out


def run_from_visit(capture: Capture, tables: dict, visit: Visit) -> dict:
    names = Names(tables, capture.game_version)
    actors: dict[int, dict] = {}
    samples: list[dict] = []
    actions: list[dict] = []
    events: list[dict] = []
    controls: list[dict] = []
    enmity: list[dict] = []
    hp: dict[int, int] = {}
    you = None
    yours: list = []
    start = visit.start

    def at(t: float) -> float:
        return round(t - start, 3)

    def sample(t: float, actor: int, **fields):
        if actor:
            samples.append({"S": at(t), "Id": actor, **fields})

    def spawn(t: float, actor: int, data: bytes, offset: int, kind: str):
        base, name, layout = struct.unpack_from("<III", data, offset + 48)
        max_hp, current = struct.unpack_from("<II", data, offset + 76)
        object_kind = data[offset + 113]
        position = [round(v, 2) for v in struct.unpack_from("<fff", data, offset + 496)]
        entry = {
            "Kind": kind or OBJECT_KINDS.get(object_kind, f"kind {object_kind}"),
            "Name": "", "BaseId": base, "NameId": name, "LayoutId": layout, "Battalion": data[offset + 117],
            "Level": data[offset + 118], "ClassJob": data[offset + 119], "MaxHp": max_hp, "Seen": at(t),
            "LastSeen": at(visit.end), "Position": position,
            "Rotation": _rotation(struct.unpack_from("<H", data, offset + 98)[0]),
        }
        if entry["Kind"] in ("you", "player"):
            entry.update(BaseId=0, NameId=0, LayoutId=0)  # nothing that names a player
        actors[actor] = entry
        hp[actor] = current
        target, = struct.unpack_from("<I", data, offset)
        sample(t, actor, Hp=current, P=position, T=target if target != 0xE0000000 else 0)

    for position, segment in enumerate(capture.segments(protocols=("zone",))):
        if position < visit.first:
            continue
        if position > visit.last:
            break
        name = names.name(segment)
        data, t, src = segment.data, segment.t, segment.src
        if segment.dir != "out":
            # When the client finishes a director's scene (a cutscene halfway through, the closing scene.)
            if name.startswith(("EventReturnHandler", "EventYieldHandler")) and len(data) >= 6:
                handler, scene = struct.unpack_from("<IH", data, 0)
                if handler >= 0x80000000:
                    events.append({"S": at(t), "Kind": "scene end", "Id": 0,
                                   "Text": f"event 0x{handler:08X} scene {scene}"})
            continue
        if you is None and segment.dst:
            you = segment.dst
        try:
            if name == "SpawnNpc" and len(data) >= SPAWN + 508:
                spawn(t, src, data, SPAWN, "")
            elif name == "SpawnPlayer" and len(data) >= 32 + 508:
                # SpawnPlayer: 32 bytes of the player's own (account, worlds, status), then CommonSpawn
                spawn(t, src, data, 32, "you" if src == you else "player")
            elif name == "DeleteActor" and len(data) >= 8:
                gone, = struct.unpack_from("<I", data, 4)
                if gone in actors:
                    actors[gone]["LastSeen"] = at(t)
            elif name == "ActorMove" and len(data) >= 12:
                x, y, z = struct.unpack_from("<HHH", data, 6)
                sample(t, src, P=[_packed(x), _packed(y), _packed(z)])
            elif name == "ActorSetPos" and len(data) >= 20:
                sample(t, src, P=[round(v, 2) for v in struct.unpack_from("<fff", data, 8)])
            elif name == "UpdateHpMpTp" and len(data) >= 4:
                hp[src], = struct.unpack_from("<I", data, 0)
                sample(t, src, Hp=hp[src])
            elif name in ("EffectResultBasic", "EffectResult") and len(data) >= 16:
                # The HP is the packet's own actor's (its source), not the ID at 8 Kawari calls target_id: retail
                # sends each actor's result from that actor, and the ID at 8 is sometimes someone else's.
                hp[src], = struct.unpack_from("<I", data, 12)
                sample(t, src, Hp=hp[src])
            elif name in ("ActorControl", "ActorControlSelf", "ActorControlTarget") and len(data) >= 20:
                category, = struct.unpack_from("<H", data, 0)
                params = list(struct.unpack_from("<IIII", data, 4))
                control = {"S": at(t), "Id": src, "Category": category, "Params": params, "Packet": name}
                if name == "ActorControlTarget" and len(data) >= 28:
                    control["Target"], = struct.unpack_from("<I", data, 24)
                controls.append(control)
                if category == SET_TARGET and name == "ActorControlTarget" and len(data) >= 28:
                    target = control["Target"]
                    sample(t, src, T=0 if target == 0xE0000000 else target)
                elif category == DEFEATED:
                    sample(t, src, Hp=0, Dead=True)
                    if src in actors:
                        actors[src].setdefault("Died", at(t))
            elif name == "ActorCast" and len(data) >= 30:
                spell, kind, omen, action, cast_time, target, rot, interruptible = struct.unpack_from(
                    "<HBBIfIHH", data, 0)
                x, y, z = struct.unpack_from("<HHH", data, 24)
                events.append({"S": at(t), "Kind": "cast", "Id": src, "Text": (
                    f"action {action} spell {spell} type {kind} on {target} {cast_time:.2f}s at {_packed(x)}, "
                    f"{_packed(y)}, {_packed(z)} facing {_rotation(rot)} omen delay {omen} "
                    f"interruptible {bool(interruptible)}")})
            elif name in EFFECTS:
                action, = struct.unpack_from("<I", data, 8)
                kind = data[31]
                count = data[33]
                if name == "ActionEffect1":
                    effects_at, targets_at, slots = 42, 112, 1
                else:
                    slots = int(name.removeprefix("AoeEffect"))
                    effects_at, targets_at = 48, 48 + slots * 64
                targets = []
                for k in range(min(count, slots)):
                    target, = struct.unpack_from("<I", data, targets_at + k * 8)
                    effects = _effects(data[effects_at + k * 64:effects_at + (k + 1) * 64])
                    entry = {"Id": target, "Effects": effects}
                    if target in hp:
                        entry["Hp"] = hp[target]
                    if target in actors:
                        entry["MaxHp"] = actors[target]["MaxHp"]
                    targets.append(entry)
                actions.append({"S": at(t), "Source": src, "Action": action, "Type": kind,
                                "SourceHp": hp.get(src), "Targets": targets})
            elif name.startswith("EventScene") and len(data) >= 20:
                handler, scene = struct.unpack_from("<IH", data, 8)
                flags, = struct.unpack_from("<I", data, 16)
                if handler >= 0x80000000:  # the director's own (60000 starts the fight, 60001 is "Duty Complete")
                    events.append({"S": at(t), "Kind": "scene", "Id": 0,
                                   "Text": f"event 0x{handler:08X} scene {scene} flags 0x{flags:X}"})
            elif name == "DirectorVars" and len(data) >= 16:
                handler, sequence, branch = struct.unpack_from("<IBB", data, 0)
                variables = list(data[6:16])
                events.append({"S": at(t), "Kind": "sequence", "Id": 0, "Text": f"0x{handler:08X} sequence {sequence}"})
                events.append({"S": at(t), "Kind": "variables", "Id": 0,
                               "Text": f"0x{handler:08X} branch {branch} {' '.join(map(str, variables))}"})
            elif name == "DirectorPopupMessage" and len(data) >= 28:
                speaker, = struct.unpack_from("<I", data, 0)
                npc_name, text, duration, icon = struct.unpack_from("<IIII", data, 12)
                events.append({"S": at(t), "Kind": "battle talk", "Id": speaker,
                               "Text": f"text {text} name {npc_name} icon {icon} {duration} ms"})
            elif name == "NpcYell" and len(data) >= 16:
                speaker, = struct.unpack_from("<I", data, 0)
                npc_name, yell = struct.unpack_from("<II", data, 8)
                events.append({"S": at(t), "Kind": "npc chat", "Id": speaker, "Text": f"NpcYell {yell} name {npc_name}"})
            elif name == "DirectorMapEffect" and len(data) >= 9:
                handler, state, timeline, index = struct.unpack_from("<IHHB", data, 0)
                events.append({"S": at(t), "Kind": "map effect", "Id": 0,
                               "Text": f"0x{handler:08X} index {index} state {state} timeline {timeline}"})
            elif name == "EnmityList" and len(data) >= 8:
                count, = struct.unpack_from("<I", data, 0)
                # Entries straight after the count (retail; Kawari's struct pads 4 bytes there.)
                yours = [list(struct.unpack_from("<II", data, 4 + k * 8)) for k in range(min(count, 8))]
            elif name == "HaterList" and len(data) >= 8:
                count, = struct.unpack_from("<I", data, 0)
                haters = [list(struct.unpack_from("<II", data, 4 + k * 8)) for k in range(min(count, 32))]
                enmity.append({"S": at(t), "Yours": yours, "Haters": haters})
        except struct.error:
            continue

    for i, actor in actors.items():
        if "Died" not in actor and hp.get(i) == 0 and actor["Kind"] == "battle npc":
            actor["Died"] = actor["LastSeen"]
    return {
        "Zone": f"territory {visit.territory}", "Content": f"duty {visit.content}" if visit.content else "",
        "TerritoryId": visit.territory, "ContentFinderCondition": visit.content,
        "Start": f"{capture.start:%Y-%m-%d} +{int(visit.start - capture.start.timestamp())} s" if capture.start else "",
        "Source": f"chronofoil {capture.id} {capture.game_version}",
        "Actors": {str(k): v for k, v in actors.items()}, "Samples": sorted(samples, key=lambda s: s["S"]),
        "Actions": actions, "Events": events, "Controls": controls, "Enmity": enmity, "Layers": [],
    }
