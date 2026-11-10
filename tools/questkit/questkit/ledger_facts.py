"""Fact tables over the ledgers (docs/src/fork/capture-pipeline.md, stages A and C): each a query with its sample
counts, as a dict (for `--out`, JSON) and as text.

* `coverage`: how much of the captures the opcode tables name, by game version.
* `uses`: per job and level band, each action's uses a minute in combat and what follows what.
* `statuses`: the statuses each action gives (to its target or its user), with the durations seen.
* `gauges`: the capturing player's ActorGauge bytes by job: which bytes move, between what values, after which actions.
* `actions`: per action, from the capturing player's own use: cast time, animation lock, the shortest gap between uses
  (its recast), request-to-effect time; from everyone's: how many it hit at most. (Not its MP cost: retail sends MP
  only with the three-second tick, the client taking an action's cost off itself, so a use's cost isn't in a packet.)
* `areas`: enemies' cones as Boss Mod has them against who they hit.
* `registry`: each capture's version, date and visits, with the capturing player's job and level in each.
* `controls`: every ActorControl and ClientTrigger category seen, with counts and an example, and whether Kawari's
  enums know it.
"""

from __future__ import annotations

import re
import statistics
from collections import Counter, defaultdict
from pathlib import Path

AUTO_ATTACKS = (7, 8)
COMBAT_GAP = 8.0  # seconds between a player's actions beyond which they weren't fighting
THIN = 5  # fewer samples than this and a fact is thin

JOBS = {1: "GLA", 2: "PGL", 3: "MRD", 4: "LNC", 5: "ARC", 6: "CNJ", 7: "THM", 19: "PLD", 20: "MNK", 21: "WAR",
        22: "DRG", 23: "BRD", 24: "WHM", 25: "BLM", 26: "ACN", 27: "SMN", 28: "SCH", 29: "ROG", 30: "NIN", 31: "MCH",
        32: "DRK", 33: "AST", 34: "SAM", 35: "RDM", 36: "BLU", 37: "GNB", 38: "DNC", 39: "RPR", 40: "SGE", 41: "VPR",
        42: "PCT"}


def band(level: int) -> int:
    return 100 if level >= 100 else level // 10 * 10


class Cast:
    """Who is who in one capture as its events go by: each actor's last `spawn`, and the capturing player's class."""

    def __init__(self):
        self.who: dict[int, dict] = {}
        self.own = {"job": 0, "lv": 0}

    def see(self, event: dict) -> None:
        kind = event["e"]
        if kind == "spawn":
            self.who[event["a"]] = event
        elif kind == "class":
            self.own = {"job": event["job"], "lv": event["synced"] or event["lv"]}
        elif kind == "zone":
            self.who = {a: s for a, s in self.who.items() if a == 0}

    def player(self, actor) -> tuple[int, int] | None:
        """A player's (job, level), None for anyone else."""
        if actor == 0 and self.own["job"]:
            return self.own["job"], self.own["lv"]
        spawn = self.who.get(actor)
        if spawn and spawn.get("k") in ("self", "player") and spawn["job"]:
            return spawn["job"], spawn["lv"]
        return None


def coverage(ledgers) -> dict:
    versions: dict[str, dict] = {}
    for capture, events in ledgers:
        for event in events:
            if event["e"] != "coverage":
                continue
            v = versions.setdefault(event["version"], {"captures": 0, "table": event["table"], "named": {
                "out": Counter(), "in": Counter()}, "unnamed": {"out": Counter(), "in": Counter()}, "wrong": Counter()})
            v["captures"] += 1
            for direction in ("out", "in"):
                v["named"][direction].update(event["named"][direction])
                v["unnamed"][direction].update(event["unnamed"][direction])
            v["wrong"].update(event["wrong"])
    out = {}
    for version, v in sorted(versions.items()):
        entry = {"captures": v["captures"], "table": v["table"], "wrong_sizes": dict(v["wrong"].most_common())}
        for direction in ("out", "in"):
            named, unnamed = sum(v["named"][direction].values()), sum(v["unnamed"][direction].values())
            entry[direction] = {"packets": named + unnamed, "named": named, "kinds": len(v["named"][direction]),
                                "unnamed_opcodes": len({key.split("/")[0] for key in v["unnamed"][direction]}),
                                "top_unnamed": dict(v["unnamed"][direction].most_common(15)),
                                "by_name": dict(v["named"][direction].most_common())}
        out[version] = entry
    return out


def coverage_text(table: dict) -> str:
    lines = ["| Version | Captures | From the server: named | kinds | unnamed opcodes | From the client: named | kinds | "
             "unnamed opcodes |", "|---|---|---|---|---|---|---|---|"]
    for version, v in table.items():
        cells = [version + ("" if v["table"] else " (no table)"), str(v["captures"])]
        for direction in ("out", "in"):
            d = v[direction]
            share = f"{100 * d['named'] / d['packets']:.1f} %" if d["packets"] else "-"
            cells += [f"{d['named']:,} of {d['packets']:,} ({share})", str(d["kinds"]), str(d["unnamed_opcodes"])]
        lines.append("| " + " | ".join(cells) + " |")
    for version, v in table.items():
        lines += ["", f"{version}: the unnamed seen most (opcode/size: count), from the server "
                      f"{_pairs(v['out']['top_unnamed'])}; from the client {_pairs(v['in']['top_unnamed'])}."]
        if v["wrong_sizes"]:
            lines.append(f"  Not the table's size: {_pairs(v['wrong_sizes'])}.")
    return "\n".join(lines) + "\n"


def _pairs(counts: dict, limit: int = 12) -> str:
    return ", ".join(f"{k}: {n}" for k, n in list(counts.items())[:limit]) or "none"


def uses(ledgers) -> dict:
    table: dict = defaultdict(lambda: {"uses": Counter(), "seconds": 0.0, "follows": Counter(), "actors": 0,
                                       "captures": set()})
    for capture, events in ledgers:
        cast = Cast()
        acts: dict[tuple, list] = defaultdict(list)  # (zone, actor, job, band) -> [(t, action)]
        zone = 0
        for event in events:
            cast.see(event)
            if event["e"] == "zone":
                zone += 1
            elif event["e"] == "effect" and event["type"] == 1 and event["action"] not in AUTO_ATTACKS:
                player = cast.player(event["a"])
                if player:
                    acts[(zone, event["a"], player[0], band(player[1]))].append((event["t"], event["action"]))
        for (_, _, job, level), used in acts.items():
            entry = table[(job, level)]
            entry["actors"] += 1
            entry["captures"].add(capture.id[:8])
            for _, action in used:
                entry["uses"][action] += 1
            for (t0, a0), (t1, a1) in zip(used, used[1:]):
                if t1 - t0 < COMBAT_GAP:
                    entry["seconds"] += t1 - t0
                    entry["follows"][(a0, a1)] += 1
    out = {}
    for (job, level), entry in sorted(table.items()):
        minutes = entry["seconds"] / 60
        out[f"{JOBS.get(job, job)}:{level}"] = {
            "job": job, "band": level, "actors": entry["actors"], "captures": sorted(entry["captures"]),
            "combat_seconds": round(entry["seconds"]),
            "uses": {str(a): n for a, n in entry["uses"].most_common()},
            "per_minute": {str(a): round(n / minutes, 2) for a, n in entry["uses"].most_common()} if minutes >= 1 else {},
            "follows": [[a, b, n] for (a, b), n in entry["follows"].most_common(60)],
            "thin": minutes < 2,
        }
    return out


def uses_text(table: dict, only_job: str | None = None) -> str:
    lines = []
    for key, entry in table.items():
        if only_job and not key.startswith(only_job.upper() + ":"):
            continue
        lines.append(f"## {key}: {entry['actors']} players, {len(entry['captures'])} captures, "
                     f"{entry['combat_seconds']} s in combat" + (" (thin)" if entry["thin"] else ""))
        rate = entry["per_minute"]
        lines.append("  " + ", ".join(f"{a} ×{n}" + (f" ({rate[a]:g}/min)" if a in rate else "")
                                      for a, n in list(entry["uses"].items())[:40]))
    return "\n".join(lines) + "\n"


STATUS_TO_TARGET, STATUS_TO_SOURCE = 14, 15


def statuses(ledgers) -> dict:
    """Per action: the statuses it gave (`target` or `self`), how often against how often the action was seen, and the
    durations the results carried."""
    seen: dict[tuple, Counter] = defaultdict(Counter)  # (side, action) -> uses
    given: dict[tuple, dict] = defaultdict(lambda: {"n": 0, "durations": Counter(), "params": Counter()})
    for capture, events in ledgers:
        cast = Cast()
        pending: dict[tuple, tuple] = {}  # (actor it lands on, status) -> (t, key)
        for event in events:
            cast.see(event)
            if event["e"] == "effect" and event["type"] == 1:
                side = "player" if cast.player(event["a"]) else "npc"
                seen[(side, event["action"])]["uses"] += 1
                done = set()
                for hit in event["hits"]:
                    for kind, value, *_ in hit["fx"]:
                        if kind not in (STATUS_TO_TARGET, STATUS_TO_SOURCE):
                            continue
                        to = "target" if kind == STATUS_TO_TARGET else "self"
                        key = (side, event["action"], value, to)
                        if key not in done:
                            done.add(key)
                            given[key]["n"] += 1
                        pending[(hit["on"] if to == "target" else event["a"], value)] = (event["t"], key)
            elif event["e"] == "result" and event.get("st"):
                for status, param, duration, _ in event["st"]:
                    waiting = pending.pop((event["a"], status), None)
                    if waiting and event["t"] - waiting[0] < 3:
                        given[waiting[1]]["durations"][duration] += 1
                        given[waiting[1]]["params"][param] += 1
    out: dict = {"player": {}, "npc": {}}
    for (side, action, status, to), entry in sorted(given.items()):
        uses_seen = seen[(side, action)]["uses"]
        out[side].setdefault(str(action), {"uses": uses_seen, "gives": []})["gives"].append({
            "status": status, "to": to, "n": entry["n"], "share": round(entry["n"] / max(1, uses_seen), 3),
            "durations": {f"{d:g}": n for d, n in entry["durations"].most_common(4)},
            "params": {str(p): n for p, n in entry["params"].most_common(4)},
            "thin": entry["n"] < THIN,
        })
    return out


def statuses_text(table: dict, side: str = "player") -> str:
    lines = []
    for action, entry in sorted(table[side].items(), key=lambda kv: int(kv[0])):
        parts = []
        for g in entry["gives"]:
            duration = next(iter(g["durations"]), "?")
            parts.append(f"{g['status']} on {g['to']} for {duration} s ({g['n']} of {entry['uses']}"
                         + (", thin" if g["thin"] else "") + ")")
        lines.append(f"{action}: " + "; ".join(parts))
    return "\n".join(lines) + "\n"


def gauges(ledgers) -> dict:
    """The capturing player's gauge bytes by job: for each byte that ever changed, the values seen, and which of
    their own actions (the last within 1.5 s) came before each change, with the change."""
    table: dict = defaultdict(lambda: {"packets": 0, "captures": set(), "values": defaultdict(Counter),
                                       "changes": defaultdict(Counter)})
    for capture, events in ledgers:
        last: dict[int, bytes] = {}
        action = (0.0, 0)
        for event in events:
            if event["e"] == "effect" and event["a"] == 0 and event["type"] == 1:
                action = (event["t"], event["action"])
            elif event["e"] == "zone":
                last.clear()
            elif event["e"] == "gauge" and event["a"] == 0:
                job, data = event["job"], bytes.fromhex(event["b"])
                entry = table[job]
                entry["packets"] += 1
                entry["captures"].add(capture.id[:8])
                before = last.get(job)
                for k, value in enumerate(data):
                    entry["values"][k][value] += 1
                    if before is not None and before[k] != value:
                        cause = action[1] if event["t"] - action[0] < 1.5 else 0
                        entry["changes"][cause][(k, before[k], value)] += 1
                last[job] = data
    out = {}
    for job, entry in sorted(table.items()):
        moving = {k: v for k, v in entry["values"].items() if len(v) > 1}
        out[JOBS.get(job, str(job))] = {
            "job": job, "packets": entry["packets"], "captures": sorted(entry["captures"]),
            "bytes": {str(k): {"values": len(v), "min": min(v), "max": max(v)} for k, v in sorted(moving.items())},
            "after": {str(cause): [[k, a, b, n] for (k, a, b), n in changes.most_common(8)]
                      for cause, changes in sorted(entry["changes"].items(), key=lambda kv: -sum(kv[1].values()))[:40]},
            "thin": entry["packets"] < 30,
        }
    return out


def gauges_text(table: dict) -> str:
    lines = []
    for job, entry in table.items():
        lines.append(f"## {job}: {entry['packets']} packets, {len(entry['captures'])} captures"
                     + (" (thin)" if entry["thin"] else ""))
        lines.append("  bytes that move (of the 15 after the job): " + (", ".join(
            f"{k}: {v['min']}-{v['max']} ({v['values']} values)" for k, v in entry["bytes"].items()) or "none"))
        for cause, changes in list(entry["after"].items())[:14]:
            lines.append(f"  after {'no action of their own' if cause == '0' else cause}: " + ", ".join(
                f"byte {k} {a}→{b} ×{n}" for k, a, b, n in changes[:5]))
    return "\n".join(lines) + "\n"


def _mode(counter: Counter):
    return counter.most_common(1)[0][0] if counter else None


def actions(ledgers) -> dict:
    table: dict = defaultdict(lambda: {"own": 0, "all": 0, "cast": Counter(), "lock": Counter(), "gaps": [],
                                       "latency": [], "targets": Counter(), "levels": Counter()})
    for capture, events in ledgers:
        cast = Cast()
        requests: dict[int, tuple] = {}  # sequence -> (t, action)
        last_use: dict[int, float] = {}
        for event in events:
            cast.see(event)
            kind = event["e"]
            if kind == "zone":
                last_use.clear()
            elif kind == "request" and event["type"] == 1:
                requests[event["seq"]] = (event["t"], event["action"])
            elif kind == "cast" and event["type"] == 1:
                table[event["action"]]["cast"][event["time"]] += 1
            elif kind == "effect" and event["type"] == 1:
                entry = table[event["action"]]
                entry["all"] += 1
                entry["targets"][len(event["hits"])] += 1
                if event["a"] != 0:
                    continue
                entry["own"] += 1
                entry["lock"][event["lock"]] += 1
                entry["levels"][cast.own["lv"]] += 1
                if event["action"] in last_use:
                    entry["gaps"].append(round(event["t"] - last_use[event["action"]], 2))
                last_use[event["action"]] = event["t"]
                asked = requests.pop(event["seq"], None)
                if asked and asked[1] == event["action"]:
                    entry["latency"].append(round(event["t"] - asked[0], 3))
    out = {}
    for action, entry in sorted(table.items()):
        fact: dict = {"seen": entry["all"], "own": entry["own"], "most_hit": max(entry["targets"], default=0)}
        if entry["cast"]:
            fact["cast"] = _mode(entry["cast"])
        if entry["own"]:
            fact["lock"] = _mode(entry["lock"])
            fact["levels"] = [min(entry["levels"]), max(entry["levels"])]
            if entry["latency"]:
                fact["request_to_effect"] = round(statistics.median(entry["latency"]), 3)
            if len(entry["gaps"]) >= THIN:
                # The shortest gaps between uses are its recast (the fifth percentile, against one-off oddities.)
                gaps = sorted(entry["gaps"])
                fact["shortest_gap"] = gaps[len(gaps) // 20]
            fact["thin"] = entry["own"] < THIN
        out[str(action)] = fact
    return out


def actions_text(table: dict, own_only: bool = True) -> str:
    lines = ["| Action | Own uses | Levels | Cast | Lock | Shortest gap | Request to effect | Most hit |",
             "|---|---|---|---|---|---|---|---|"]
    for action, fact in table.items():
        if own_only and not fact["own"]:
            continue
        levels = fact.get("levels")
        lines.append("| " + " | ".join(str(x) for x in (
            action, f"{fact['own']}" + (" (thin)" if fact.get("thin") else ""),
            f"{levels[0]}-{levels[1]}" if levels else "", fact.get("cast", ""), fact.get("lock", ""),
            fact.get("shortest_gap", ""), fact.get("request_to_effect", ""), fact["most_hit"])) + " |")
    return "\n".join(lines) + "\n"


def known_categories(repo: Path) -> dict[str, set[int]]:
    """The categories Kawari's enums have (`#[brw(magic = Nu16)]` in actor_control.rs, `Nu32` in client_trigger.rs)."""
    found = {}
    for key, path in (("control", "core/src/ipc/zone/server/actor_control.rs"),
                      ("trigger", "core/src/ipc/zone/client/client_trigger.rs")):
        file = repo / path
        text = file.read_text(encoding="utf-8") if file.exists() else ""
        found[key] = {int(n, 0) for n in re.findall(r"magic\s*=\s*(0x[0-9A-Fa-f]+|\d+)u(?:16|32)", text)}
    return found


def sapphire_names(header: str | Path | None) -> dict[str, dict[int, str]]:
    """What Sapphire calls each category (its `src/common/Network/CommonActorControl.h`, given as a file: its
    `ActorControlType` and `PacketCommand` enums). A name to start from, from an older patch: not committed here."""
    names: dict[str, dict[int, str]] = {"control": {}, "trigger": {}}
    if not header:
        return names
    text = Path(header).read_text(encoding="utf-8", errors="replace")
    for key, enum in (("control", "ActorControlType"), ("trigger", "PacketCommand")):
        body = re.search(rf"enum {enum}\b[^{{]*\{{(.*?)\n  \}};", text, re.S)
        for name, value in re.findall(r"^\s*(\w+)\s*=\s*(0x[0-9A-Fa-f]+|\d+)\s*,", body.group(1) if body else "", re.M):
            names[key].setdefault(int(value, 0), name)
    return names


def controls(ledgers, repo: Path | None = None, sapphire: str | Path | None = None) -> dict:
    known = known_categories(repo) if repo else {"control": set(), "trigger": set()}
    called = sapphire_names(sapphire)
    seen: dict[tuple, dict] = {}
    for capture, events in ledgers:
        for event in events:
            if event["e"] == "control":
                key = ("control", event["pk"], event["cat"])
            elif event["e"] == "trigger":
                key = ("trigger", "Client", event["cat"])
            else:
                continue
            entry = seen.setdefault(key, {"n": 0, "captures": set(), "example": event["args"], "args": [
                Counter() for _ in event["args"]]})
            entry["n"] += 1
            entry["captures"].add(capture.id[:8])
            for counter, value in zip(entry["args"], event["args"]):
                if len(counter) < 50:
                    counter[value] += 1
    out: dict = {"control": [], "trigger": []}
    for (kind, packet, category), entry in sorted(seen.items(), key=lambda kv: -kv[1]["n"]):
        out[kind].append({"packet": packet, "category": category, "n": entry["n"], "captures": len(entry["captures"]),
                          "known": category in known[kind], "sapphire": called[kind].get(category, ""),
                          "example": entry["example"],
                          "distinct": [len(counter) for counter in entry["args"]]})
    return out


def controls_text(table: dict, unknown_only: bool = True) -> str:
    lines = []
    for kind, title in (("control", "ActorControl categories"), ("trigger", "ClientTrigger categories")):
        rows = [r for r in table[kind] if not (unknown_only and r["known"])]
        lines += [f"## {title}" + (" Kawari has no name for" if unknown_only else "")
                  + f": {len(rows)} of {len(table[kind])}", "",
                  "| Packet | Category | Seen | Captures | An example's arguments | Distinct values of each | "
                  "Sapphire's name |", "|---|---|---|---|---|---|---|"]
        for r in rows:
            lines.append(f"| {r['packet']} | {r['category']} | {r['n']} | {r['captures']} | "
                         f"{' '.join(map(str, r['example']))} | {' '.join(map(str, r['distinct']))} | "
                         f"{r['sapphire']} |")
        lines.append("")
    return "\n".join(lines)


def areas(ledgers, public=None) -> dict:
    """Enemies' cones against who they hit: for each NPC action Boss Mod has a cone for, how far off the caster's
    facing (degrees) the people hit stood, beside half the cone's angle. Someone hit well outside it says the angle (or
    this reading of the packet) is wrong; nobody near the edge says nothing. Positions are each actor's last sent one,
    so a moving player's is up to a third of a second old."""
    import math

    if public is None:
        from .public_sources import Public

        public = Public()
    cones = public.cone_angles(verified_only=False)
    checked = public.cone_angles()
    seen: dict[int, dict] = defaultdict(lambda: {"uses": 0, "hits": 0, "off": [], "captures": set()})
    for capture, events in ledgers:
        cast = Cast()
        for event in events:
            cast.see(event)
            if event["e"] != "effect" or event["action"] not in cones or cast.player(event["a"]) or not event["p"]:
                continue
            entry = seen[event["action"]]
            entry["uses"] += 1
            entry["captures"].add(capture.id[:8])
            for hit in event["hits"]:
                if not hit["p"] or not any(fx[0] == 3 for fx in hit["fx"]):
                    continue
                dx, dz = hit["p"][0] - event["p"][0], hit["p"][2] - event["p"][2]
                if math.hypot(dx, dz) < 1.0:
                    continue  # on top of the caster: any direction
                off = math.degrees(math.atan2(dx, dz) - event["r"])
                entry["hits"] += 1
                entry["off"].append(abs((off + 180) % 360 - 180))
    out = {}
    for action, entry in sorted(seen.items()):
        if not entry["off"]:
            continue
        off = sorted(entry["off"])
        half = cones[action] / 2
        out[str(action)] = {
            "angle": cones[action], "boss_mods_guess": action not in checked, "uses": entry["uses"],
            "hits": entry["hits"], "captures": sorted(entry["captures"]), "furthest_off": round(off[-1], 1),
            "p90_off": round(off[int(len(off) * 0.9) - (len(off) >= 10)], 1),
            # Past half the angle and a hitbox's worth (10 degrees): a hit the cone as recorded wouldn't make.
            "outside": sum(1 for o in off if o > half + 10), "thin": entry["hits"] < THIN,
        }
    return out


def areas_text(table: dict) -> str:
    lines = ["| Action | Cone (Boss Mod) | Uses | Hits | Furthest off its facing | 90 % within | Hit outside half + 10 |",
             "|---|---|---|---|---|---|---|"]
    for action, fact in sorted(table.items(), key=lambda kv: -kv[1]["outside"] / max(1, kv[1]["hits"])):
        lines.append(f"| {action} | {fact['angle']:g}" + (" (its guess)" if fact["boss_mods_guess"] else "")
                     + f" | {fact['uses']} | {fact['hits']}" + (" (thin)" if fact["thin"] else "")
                     + f" | {fact['furthest_off']:g} | {fact['p90_off']:g} | {fact['outside']} |")
    return "\n".join(lines) + "\n"


def registry(ledgers) -> dict:
    """Each capture: its game version, when it began, and its visits (territory, duty, seconds, the capturing
    player's job and level there, how many actions of their own). What a session looks up before opening a capture."""
    out = {}
    for capture, events in ledgers:
        visits: list[dict] = []
        own = {"job": 0, "lv": 0}
        for event in events:
            kind = event["e"]
            if kind == "zone":
                if visits:
                    visits[-1]["seconds"] = round(event["t"] - visits[-1].pop("t"))
                visits.append({"t": event["t"], "territory": event["territory"], "content": event["content"],
                               "job": own["job"], "level": own["lv"], "own_actions": 0, "gauge_packets": 0})
            elif kind == "class":
                own = {"job": event["job"], "lv": event["synced"] or event["lv"]}
                if visits:
                    visits[-1].update(job=own["job"], level=own["lv"])
            elif not visits:
                continue
            elif kind == "effect" and event["a"] == 0 and event["type"] == 1 and event["action"] not in AUTO_ATTACKS:
                visits[-1]["own_actions"] += 1
            elif kind == "gauge" and event["a"] == 0:
                visits[-1]["gauge_packets"] += 1
            if "t" in event and visits:
                visits[-1]["end"] = event["t"]
        for visit in visits:
            end = visit.pop("end", None)
            if "t" in visit:
                visit["seconds"] = round((end or visit["t"]) - visit.pop("t"))
        out[capture.id[:8]] = {
            "version": capture.game_version, "start": f"{capture.start:%Y-%m-%d %H:%M}" if capture.start else "",
            "minutes": round(capture.duration / 60) if capture.duration else None, "visits": visits}
    return out


def registry_text(table: dict) -> str:
    lines = ["| Capture | Began | Version | Minutes | Visits | Duties (ContentFinderCondition) | Jobs and levels | "
             "Own actions |", "|---|---|---|---|---|---|---|---|"]
    for capture, entry in sorted(table.items(), key=lambda kv: kv[1]["start"]):
        visits = entry["visits"]
        duties = sorted({v["content"] for v in visits if v["content"]})
        jobs = sorted({(v["job"], v["level"]) for v in visits if v["job"] and v["own_actions"]})
        lines.append(f"| {capture} | {entry['start']} | {entry['version'][:10]} | {entry['minutes'] or ''} | "
                     f"{len(visits)} | {', '.join(map(str, duties))} | "
                     f"{', '.join(f'{JOBS.get(j, j)} {lv}' for j, lv in jobs)} | "
                     f"{sum(v['own_actions'] for v in visits)} |")
    return "\n".join(lines) + "\n"


TABLES = {
    "registry": (registry, registry_text),
    "areas": (areas, areas_text),
    "coverage": (coverage, coverage_text),
    "uses": (uses, uses_text),
    "statuses": (statuses, statuses_text),
    "gauges": (gauges, gauges_text),
    "actions": (actions, actions_text),
    "controls": (controls, controls_text),
}
