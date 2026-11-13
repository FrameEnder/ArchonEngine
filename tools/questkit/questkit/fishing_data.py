"""Fishing's numbers from AutoHook's data (github.com/PunishXIV/AutoHook, BSD-3-Clause, © Puni.sh), for the world
server: `resources/data/fishing/fishing.json` (see its README for what each source is and its licence).

AutoHook's `AutoHook/Data/FishData/` gathers the community's catch logs: `spot_pools.json` (Teamcraft's,
gubal.ffxivteamcraft.com: for each fishing spot and bait, how many of each fish were caught, and after how many
seconds each bit), `fish_list.json` (each fish: its bite's tug, the hookset that lands it, the bait it takes and what
it can be mooched from, the fish to catch first for it, its weather and hours, the Gathering it needs) and
`fish_tracker.json` (FF14 Fish Tracker's, MIT: hours, weather and the Fish Eyes and intuition details). Only numbers
come out: item, spot and weather IDs, counts, seconds, hours.

The extract (one spot, one fish a line, so a new checkout's changes show in a diff):

* `spots`: spot -> bait -> [[fish, catches, bite seconds: minimum, tenth, median, ninetieth, maximum], ...], from the
  catches with no lure, caught more than once (fish -1: nothing on the line);
* `fish`: fish -> {"tug": 36 light / 37 medium / 38 heavy (the bite's ActionTimeline row as AutoHook names it),
  "hookset": 4103 Powerful / 4179 Precision (the action), "bait", "mooch" (fish it's mooched from), "predators"
  ([[fish, how many], ...] for intuition), "weathers", "weathers_from", "minutes" [start, end) in Eorzea minutes of the day (some windows begin
  on the half hour),
  "gathering" (the least Gathering), "snagging", "fish_eyes", "intuition" (seconds), "big"}: only what's set.
"""

from __future__ import annotations

import json
from pathlib import Path

from . import repo

OUT = repo.ROOT / "resources" / "data" / "fishing" / "fishing.json"


def _percentiles(pairs: list[int]) -> list[float] | None:
    """Bite seconds from a flat seconds/count list: minimum, tenth, median, ninetieth, maximum."""
    counts = sorted(zip(pairs[0::2], pairs[1::2]))
    total = sum(c for _, c in counts)
    if not total:
        return None

    def at(share: float) -> float:
        running = 0
        for seconds, count in counts:
            running += count
            if running >= share * total:
                return float(seconds)
        return float(counts[-1][0])

    return [float(counts[0][0]), at(0.1), at(0.5), at(0.9), float(counts[-1][0])]


def read(checkout: str | Path) -> dict:
    root = Path(checkout)
    data = root / "AutoHook" / "Data" / "FishData"
    if not data.is_dir():
        data = root
    pools = json.loads((data / "spot_pools.json").read_text(encoding="utf-8"))
    fish_list = json.loads((data / "fish_list.json").read_text(encoding="utf-8"))
    tracker = json.loads((data / "fish_tracker.json").read_text(encoding="utf-8")).get("Fish", {})

    spots: dict[str, dict[str, list]] = {}
    for spot, pool in pools.get("Spots", {}).items():
        bites = {(b[0], b[1]): _percentiles(b[2:]) for b in pool.get("BiteTimes", [])}
        by_bait: dict[str, list] = {}
        for bait, fish, a_lure, m_lure, count in pool.get("Catches", []):
            # One catch alone is more likely a slip in someone's log than a fish that bites there.
            if a_lure or m_lure or count <= 1:
                continue
            by_bait.setdefault(str(bait), []).append([fish, count, bites.get((bait, fish))])
        for catches in by_bait.values():
            catches.sort(key=lambda c: (-c[1], c[0]))
        if by_bait:
            spots[spot] = dict(sorted(by_bait.items(), key=lambda kv: int(kv[0])))

    fish: dict[str, dict] = {}
    for entry in fish_list:
        if entry.get("IsSpearFish"):
            continue
        item = entry["ItemId"]
        row: dict = {}
        if entry.get("BiteType") in (36, 37, 38):
            row["tug"] = entry["BiteType"]
        if entry.get("HookType") in (4103, 4179):
            row["hookset"] = entry["HookType"]
        if entry.get("InitialBait"):
            row["bait"] = entry["InitialBait"]
        if entry.get("Mooches"):
            row["mooch"] = entry["Mooches"]
        if entry.get("Predators"):
            row["predators"] = [[p["ItemId"], p["Quantity"]] for p in entry["Predators"]]
        if entry.get("Weathers"):
            row["weathers"] = entry["Weathers"]
        if entry.get("WeathersFrom"):
            row["weathers_from"] = entry["WeathersFrom"]
        if entry.get("Spawn") is not None and entry.get("Duration"):
            row["minutes"] = [round(entry["Spawn"] * 60), round((entry["Spawn"] + entry["Duration"]) % 24 * 60)]
        if entry.get("MinGathering"):
            row["gathering"] = entry["MinGathering"]
        if entry.get("Snagging"):
            row["snagging"] = True
        known = tracker.get(str(item), {})
        if "minutes" not in row and known.get("StartHour") is not None and (known.get("StartHour"), known.get(
                "EndHour")) != (0, 24):
            row["minutes"] = [round(known["StartHour"] * 60), round(known["EndHour"] % 24 * 60)]
        if known.get("FishEyes"):
            row["fish_eyes"] = True
        if known.get("IntuitionSeconds"):
            row["intuition"] = known["IntuitionSeconds"]
        if known.get("BigFish"):
            row["big"] = True
        if row:
            fish[str(item)] = row
    return {"spots": spots, "fish": dict(sorted(fish.items(), key=lambda kv: int(kv[0])))}


def build(checkout: str | Path, out: Path = OUT) -> dict:
    data = read(checkout)
    out.parent.mkdir(parents=True, exist_ok=True)
    lines = ["{", ' "spots": {']
    lines.append(",\n".join(f"  {json.dumps(spot)}: " + json.dumps(baits, separators=(",", ":"))
                            for spot, baits in data["spots"].items()))
    lines += [" },", ' "fish": {']
    lines.append(",\n".join(f"  {json.dumps(item)}: " + json.dumps(row, separators=(",", ":"))
                            for item, row in data["fish"].items()))
    lines += [" }", "}"]
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return {"spots": len(data["spots"]), "fish": len(data["fish"])}
