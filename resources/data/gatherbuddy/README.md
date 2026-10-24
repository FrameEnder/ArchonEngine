# GatherBuddy Reborn data

`IlvConvertTable.csv` is GatherBuddy Reborn's `GatherBuddy/CustomInfo/IlvConvertTable.csv` (commit a204f2f), unchanged:
each gathering item level (GatheringItemLevelConvertTable row) with its base Gathering and Perception. GatherBuddy Reborn
is licensed under the Apache License 2.0 (https://www.apache.org/licenses/LICENSE-2.0); © its authors.

The world server reads it (`GameData::gathering_base_stats`) because retail's node window sends these numbers: Bob's
Chronofoil capture of 2026-10-24 matched it on every item (one Perception off by one, level 36: retail 195, here 194),
where the ItemLevel sheet doesn't. See `docs/src/fork/gathering.md`.
