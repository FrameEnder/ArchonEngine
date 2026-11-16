# FATEs

Upstream's FATEs (`servers/world/src/server/fate.rs`): each public zone runs up to five of the FATEs its layout has
ranges for, picked at random, each for 15 minutes; a FATE's script (`resources/scripts/fates/<id / 100>/…_<id>.lua`,
found by the number at the end of its name) says in `onSetup` what it brings out. A FATE without a script runs with
nothing in it. `!fate <id>` starts one where the GM stands. Rewards are in [Level Sync](level-sync.md#fate-rewards).

Scripted: 120 and 603 (upstream's test scripts) and, from retail, **586**.

## A boss FATE (`fate:spawn_boss(layout ID)`)

FATE 586, Outer La Noscea, from Bob's Chronofoil capture 651981e7 (2026-11-16): the whole FATE, Bob alone on it (all
121 of his hits on FATE enemies in the capture were on its one enemy).

| When | Retail |
|---|---|
| start | FateInit (ActorControlSelf 2353) `[586, 1]`, UnkFate (start time, 900 s), 2356 `[586]`, 2353 `[586, 4]`, UnkFate again, 2353 `[586, 4]`, 2370 `[586]` |
| 0.3 s | the enemy's SpawnNpc: BNpcBase 2153, BNpcName 1524, level 44, 20240 HP, layout 4306103, at (82.6, 55.5, -467.7), display flags 0x40000, FATE 586 |
| every 1.33 s | FateUpdate (2364) `[586, progress]`, changed or not: 229 of them in the 306 s it lived, 44 at 0 before Bob's first hit |
| the enemy dies | its Defeated 2.0 s later; 0.1 s after that 2353 `[586, 5]`; 1.0 s later `[586, 7]` and `[586, 10]`; 6.5 s later `[586, 0]` |
| 24 minutes after its start | it began again (the next time it came up) |

**Progress is the health the enemy has lost**, in whole percent, rounded down: each FateUpdate against the enemy's
health at that moment is exact for 210 of 229 and one above for 19 (a hit's result a moment ahead of the health
seen).

Kawari: `fate:spawn_boss` spawns the enemy as `spawn_bnpc` does (from the zone's battle NPCs, here drop-in layer
`FATE_00586` in `sea_s1f6_planmap.json`, `nonpop`) and marks it the FATE's boss (`FateInstance::boss`).
`fate::boss_tick` tells its progress every `BOSS_PROGRESS_EVERY` (`boss_progress`), and at once when it falls: 100,
then `QueuedTaskData::FateState` with 5 after 2.1 s, upstream's `EndFate` (7, 10, the rewards, the next FATE) after
3.1 s and `FateState` 0 6.5 s later. A FATE that ends with its enemies still standing (out of time) takes them with
it (`EndFate`): before, a FATE's enemy stayed for good.

**The enemy** (timeline `Fate586Boss_2153`): auto-attacks (870) 3 s apart for 136; 720, a 2.2 s cast on an area,
every 10.6 s, 430 the three times of nineteen it landed on Bob; 429, a 1.2 s cast giving itself status 207 (100) for
30 s, five times, 34 to 63 s apart.

Not from the capture, so as upstream has it: the start's packets (Kawari's own order), what the client is sent
when the time runs out, and the rewards (gold's EXP and gil for whoever fought it). FATE 588 finished twice around
Bob in the same capture (its progress in steps of 8 or 9, the same four states at each end); he didn't fight in it,
so it has no script.

Test: `a_boss_fate_follows_its_boss_s_health`.
