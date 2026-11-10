# Testing: what's checked without playing

How the fork is kept from breaking again what was fixed. Everything here runs from files, with no game install and no
client, so it runs in CI and in a session; what only a played run shows is said at the end. Since 2026-11-10 (Bob:
"is there any other way to increase the stability of what we currently have?"; a dungeon simulator and his bundles as
tests came first.)

## The layers

| Check | Runs | Catches |
|---|---|---|
| `questkit replay` | every push | A quest script that no longer does what retail did (scenes, flags, variables, notices) |
| `questkit battle-sim` | every push | A quest battle's director against retail's run: sequences, lines, waves, the win |
| `questkit duty-sim` (new) | every push | A dungeon's or trial's director against retail's run: boss music, objectives, completion; and a wipe rehearsed in every boss fight |
| `questkit rotation --check` (in questkit's tests) | every push | A bot rotation or action script that errors or leaves a bot idle, at every level band |
| `questkit lint`, `lua-syntax` | every push | Actions without scripts, timelines and fixtures missing, Lua that doesn't compile |
| questkit's own tests | every push | The tools themselves (made-up data) |
| `cargo test`: `server/bundle_tests.rs` (new) and the modules' own | PRs, pushes to master, by hand | The server's own behaviour in a situation a bundle showed, on a world made by hand (`server/test_world.rs`) |

## A fixed bundle becomes a test

When a mark in one of Bob's bundles is fixed, the situation that showed it is kept, so the fix stays:

* **The server's side** (who is sent what, an actor's state): a test in `servers/world/src/server/bundle_tests.rs`,
  named by what Bob wrote, with the bundle in its comment. `TestWorld` (`test_world.rs`) is an `Instance` with
  players (each with a client whose messages the test reads back: `sent`), enemies and a party's Player NPCs, all
  made by hand; `casting` queues a cast as `action.rs` does. Nothing in it reads the game's data, so what needs stats,
  sheets or the navmesh can't be tested there (say so in the fix's changelog entry instead.)
* **A script's side** (a quest, a quest battle, a duty): the retail run that shows the right behaviour as a fixture
  (`replay`, `battle-sim`, `duty-sim`), or a line in the fixture that failed.

The first two (bundle 20261110-141944, "stone vigil still broken"):

| Bob wrote | What it was | Test |
|---|---|---|
| "it all sees me as casting" | He fell 1.5 s into a Cure II. A dead actor's tasks are dropped, the cast's among them, and that task was what took the Casting condition off: it stayed set through the wipe's reset, and the client refused every action. Retail's Condition at a death mid-cast has no Casting and no CancelCast follows (four such deaths in his captures) | `a_player_who_dies_mid_cast_stops_casting` |
| "on reset the party list broke" | Most likely: at a wipe the Duty Finder's bots were taken away and spawned again, as retail does Duty Support's members. Retail never does that to a party's players (three wipes in parties of players, captures 1aac5be2, 84cdbfb1 and dd09ac61): each is moved with an ActorSetPos of the player's own warp type (25, 2), mode back to normal, ActorControls 60 and 1529, then ZoneIn | `a_player_npc_goes_back_at_a_wipe_without_leaving` |

## duty-sim

```shell
python3 tools/questkit duty-sim              # every duty fixture, against duties/known.json
python3 tools/questkit duty-sim 11 -v        # one duty, with every event's timing
python3 tools/questkit chronofoil chronofoil/ --battle 112.2 --out run.json.gz
python3 tools/questkit duty-sim --make r1d1_re.lua --run run.json.gz     # a fixture of that run
python3 tools/questkit duty-sim --update-known                          # after a fix, or a new fixture
```

A **duty fixture** (`tools/questkit/duties/<ContentFinderCondition>-<capture>.json`, numbers only) is one retail run
from the duty commencing (the director's 0x40000001): every NPC by layout ID with its lives (appeared, first had a
target, fell, health a percent at a time, where a boss went), where the player and the party stood, the NPCs' casts,
the party's wipes (0x40000005, 0x40000006), the progress bars of event actions (a door being opened), which of the
party were players (`players`: the script sees those as `player2`... too), event ranges the party walked into
(`ranges`, written by hand: a capture doesn't say; Sastasha's cove), and the events
to match: the boss music (0x80000001: 37 at a pull, 0 as the boss falls), objectives done (0x80000016), sequences and
the completion (0x40000003).

`duty_sim.py` runs the real script (with `Global.lua` and `content/lib/Battle.lua`) as the server does: `onSetup`,
`onCommence` at 0, `onUpdate` every half second with retail's health and details (`info[layout]` with `target` once
retail's NPC had one, `player1`, `party1`...), `onActorDeath` and `onActorCast` as retail's NPCs fell and cast,
`onPartyWipe` and `onDutyRecommence` at retail's wipes, and at each progress bar `onGimmickAccessor` on the script's
`EOBJ_*` objects until one answers with that event action (then its `onEventActionCast` when the bar ended). A call
after `director:delay(n)` counts n seconds later; `os.time()` is the run's clock; `math.random` is seeded. An NPC the
script itself brings out (`spawn_bnpc`, `spawn_boss`, `spawn_bnpc_part`: found by a first pass) is there from when the
script brings it out, and gone when the script takes it away. One retail gave no layout ID can be kept under the
script's ID for it (`fixture_from_run(…, unlaid={BNpcBase: id})`: the Navel's Heart of the Mountain, 1507 as
99001507). An NPC's `spawn` is when it was first seen at all (its first position), not its latest appearance to the
capturing player.

Then two checks:

* **Against retail**: the script's `set_bgm`, `objective`, `set_sequence` and `complete_duty` against retail's, in
  order, each within 3 s (`tolerance`, `tolerances` by label).
* **A wipe rehearsed** in every boss fight retail got through in one go (the bosses: the NPCs whose fall the music or
  an objective followed): the fight to its middle, `onPartyWipe`, `onDutyRecommence`, and the same stretch again.
  Every *kind* of thing the script told the director the first time must come again (not each call: a random choice
  or helpers kept across attempts differ in the particulars). A flag `onPartyWipe` doesn't reset shows as a kind that
  never comes again. Lines and shortcuts said once a duty are left out (`ONCE`; a fixture's `once` adds its own.)

**What's written down** (`duties/known.json`, by fixture): the problems a run has now. `duty-sim` passes when each
run has exactly those: a new one fails, and so does one that no longer happens until it's taken off the list
(`--update-known`), so the list stays what's true. 21 fixtures, two a duty (three for the Bowl of Embers), 28 s.
Bundle tests so far (`bundle_tests.rs`): `a_player_who_dies_mid_cast_stops_casting`,
`a_player_npc_goes_back_at_a_wipe_without_leaving`, `a_cast_is_aimed_where_its_target_stands`,
`a_whirlwind_tosses_whoever_it_hits`.

**Since 2026-11-10 (second pass): 21 of 21 clean, nothing written down.** What was done for each is in the table's
last column; the scripts' changes are in [Dungeons](dungeons.md). What the simulator had wrong this time: party
members who were players weren't players to the script (Toto-Rak's rooms count a player crossing in: objectives
3-12 s late), an NPC that left the capturing player's sight and came back "appeared" when it came back (Brayflox's
third boss 66 s late), and one run's capture missed the last boss's fall (Toto-Rak 2528a32d, replaced by caa5e908,
a Duty Support run). Brayflox 44df02fc allows its `bgm 0` 5 s (`tolerances`): the dragon came 2.4 s late in the
simulator, by health sampled at hits.

**First run, 2026-11-10** (7 of 21 clean: the Stone Vigil, the Bowl of Embers, the Howling Eye; 68 problems
written down, all in [Known issues](known-issues.md#duty-scripts-against-duty-sim-2026-11-10)):

| Duty | Script | What it found | Since (second pass) |
|---|---|---|---|
| The Thousand Maws of Toto-Rak (1) | `f1r1_re` | No `onPartyWipe`: after a wipe the boss music and the room's objective don't come again, nor the last boss's adds; objectives 3-12 s late in one run, and that run never completes | `onPartyWipe`; the lateness was the simulator's (players), the run that never completed a capture's gap |
| The Tam-Tara Deepcroft (2) | `f1d1_re` | No `onPartyWipe`: the rooms' adds don't come again; the music stops 3-7 s before retail's | `onPartyWipe`; the room's boss comes as the last of its pack falls, not its leader |
| Copperbell Mines (3) | `w1d1_re` | No `onPartyWipe`: the first two bosses' music doesn't start again | `onPartyWipe` |
| Sastasha (4) | `s1d1_re` | Never starts the boss music (retail: 37 at each of three pulls), and stops it once, at the end | It did start it, as each captain appeared; retail's is at his pull, and stops as he runs off: the captains run off now, Denn comes at the cove, `onPartyWipe` |
| Haukke Manor (6) | `f1d2_re` | No `onPartyWipe`: music not again, the last boss's adds not again | `onPartyWipe` |
| Brayflox's Longstop (8) | `s1d2_re` | No `onPartyWipe`: music and the second boss's adds not again; completion 6.8 s late in one run | `onPartyWipe`; the timings were the simulator's (an NPC's first sight) |
| The Navel (57) | `s1fa_re` | Never changes the music (retail: 158, 160, 156 as the phases go) | 158 and 160 with the Geocrushes, 156 with Earthen Fury |

What the simulator had wrong on the way, and has right now: the Bowl of Embers' Nail (taken away by the script at a
wipe, it "fell" afterwards) and the Howling Eye's new Garuda (waited for at full health, which an interpolated
health never showed.)

**Adding a duty**: write its fixtures from two runs (`--make`), run `duty-sim <n> -v`, fix the script until the
events match, then `--update-known` for what's left and say it in Known issues.

## What none of this shows

* Anything the client decides: what it draws, whether a packet it was sent makes it do the right thing (the party
  list above is tested as "sent what retail sent", not as "the list looks right").
* Anything that needs the game's data on the server: stats and damage, the navmesh and collision, sheets. The
  `TestWorld` has none; `GameData` can't be made without an install.
* Enmity, who hits whom and the party's NPCs inside a duty (`duty-sim` shows the script retail's fight, it doesn't
  fight one), and the timing between the server's tasks.

Those still need Bob's run and a bundle; the bundle's fix then goes in as above.
