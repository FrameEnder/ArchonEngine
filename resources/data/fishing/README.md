# Fishing data

`fishing.json` is made by `python3 tools/questkit fishing <AutoHook checkout>` (questkit's `fishing_data.py`) from
AutoHook's `AutoHook/Data/FishData/` (github.com/PunishXIV/AutoHook, commit 467cc6a, 2026-11-08). Numbers only: item,
fishing spot and weather IDs, counts, seconds and Eorzea minutes. The world server reads it (`servers/world/src/fishing.rs`).
See `docs/src/fork/gathering.md` (Fishing).

What it's made from, and their licences:

* AutoHook, BSD 3-Clause License, Copyright (c) 2024, Puni.sh: its fish list (each fish's tug, hookset, bait, mooch,
  predators, weathers, hours, least Gathering).
* Its `spot_pools.json` and bite times are FFXIV Teamcraft's catch logs (gubal.ffxivteamcraft.com: what players
  caught at each fishing spot on each bait, and how long each bite took). FFXIV Teamcraft is MIT-licensed
  (github.com/ffxiv-teamcraft/ffxiv-teamcraft).
* Its `fish_tracker.json` is FF14 Fish Tracker's, by Carbuncle Plushy (github.com/icykoneko/ff14-fish-tracker-app),
  MIT License: hours where the fish list has none, Fish Eyes, intuition seconds, big fish.

AutoHook's licence asks that redistributions keep its notice:

> Copyright (c) 2024, Puni.sh. Redistribution and use in source and binary forms, with or without modification, are
> permitted provided that the following conditions are met: redistributions of source code must retain the above
> copyright notice, this list of conditions and the following disclaimer; redistributions in binary form must
> reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation
> and/or other materials provided with the distribution; neither the name of the copyright holder nor the names of its
> contributors may be used to endorse or promote products derived from this software without specific prior written
> permission. THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED
> WARRANTIES ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DAMAGES ARISING
> IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
