# Events

Seasonal and special events: how one is turned on, what the fork has built, and how each was played on retail.

Bob, 2026-11-15: "recreate the entirety of the Keybound event … an Event tab in the ControlPanel to select any
implemented Events to be turned on or scheduled."

## Turning an event on (2026-11-15)

Where it lives: `servers/world/src/server/game_events.rs`, `resources/events/fork_events.json`, and ArchonEngine
ControlPanel's Dashboard → Events ([ArchonEngine ControlPanel](kawari-web.md#events-dashboard--events-2331)).

**An event is a festival to the client.** ZoneInit's game festivals (eight IDs, each with a phase) decide which of a
zone's layers are out: an event's NPCs, decorations and objects sit on layers with its festival ID. The UI festivals
(the same list in ZoneInit and PlayerSetup) decide what the menus offer. Retail ran Keybound Brawler as festival
**257** (`festivals.yml`): every ZoneInit in Connor's Chronofoil captures of 2026-11-03 to 11-09 and Bob's of
2026-10-29 had the game festivals `[257, 256, 120]` with phases `[0, 2, 0]`, and the UI festivals `[257, 0, 256, 120]`
with phase 2 on 256. In the duty the game festivals were all 0 and the UI ones `[257, 256, 120]`, and three
ActorControlSelfs right after its ZoneInit set them one at a time: `[257]`, `[257, 131328]`, `[257, 131328, 120]`
(131328 = 256 with phase 2 in its upper half; category 924 in that version, SetFestival.)

* **`resources/events/fork_events.json`** lists the events the fork has built: `key`, `festival`, `phase`, `quests`,
  `label` (the festival's name in upstream's `festivals.yml`, for the ControlPanel's page), `duties`, `territories`, `content_entries` (the ContentEntry events that put a duty in the Duty Finder), and when
  retail ran it (`retail_start`, `retail_end`: Unix seconds, null when not over). The quests' and duties' names the
  ControlPanel reads from the game. **It's in `resources/events/`, not `resources/data/`**: a build's files leave
  `resources/data` out (the ControlPanel's source build and CI's artifact: `rsync --exclude /data/`), and the page
  reads the list from the build. It was in `resources/data` for three commits (`399192f` to `f9e4b0f`), and the
  page listed nothing (Bob, 2026-11-15). **A new event gets an entry here** and
  nothing else to be listed and switched.
* **`events.json`** in the server's data folder (`$KAWARI_DATA`, beside `drop-rates.json`) is the owner's:
  `{"format": 1, "events": {"keybound_brawler": {"mode": "on"}}}`. `mode` is `off` (the same as not there), `on`, or
  `scheduled` with `start` and `end` in Unix seconds (UTC); without a start it's on until its end, without an end from
  its start for good (`is_on_in`).
* **The festivals sent** (`festivals_in`): `world.active_festivals` (upstream's eight, set from the admin page or
  `!festival`), then each event that's on, with its phase, as far as the eight places go; an event whose festival is
  configured already keeps its place. They go in ZoneInit's game and UI festivals (with their phases, which Kawari
  left at 0 before) and PlayerSetup's.
* **No restart**: the file is read again when its modification time changes, and the world looks at the festivals on
  every turn (`changed`): when they differ from the turn before (the file changed, or a schedule's start or end
  passed), everyone online gets SetFestival with the first four, each with its phase in its upper half
  (`send_festivals`, the same as upstream's ReloadFestivals now sends), and the log says "The festivals are now …".
  A zone's layers follow at a player's next zone change.

Tests (`game_events.rs`): `the_built_events_read`, `off_on_and_scheduled`,
`an_event_s_festival_joins_the_configured_ones`, `a_phase_rides_in_the_upper_half`. Not played.

## Keybound Brawler

The FFVII Remake collaboration's typing game (patch 7.57; retail from 2026-10-28 08:00 UTC). From **Connor's
Chronofoil captures** (`Connor.zip`: 933b9471, 17338003, 786a99de, 437f16e1, e834e436, 137de9a1; 2026-11-03 to
11-09: the quest twice, about 120 visits to the square, **223 runs of the duty**), **Bob's Kawari Recorder export of
2026-11-12** (four runs of the duty on 2026-11-03, and the quest) and Bob's Chronofoil captures deaf7e28, 2c98f9e8
and 5fcc970c (the Gold Saucer with the event on). The working notes, numbers only, are
`notes/keybound-brawler.md` in kawari-diagnostics.

| What | ID |
|---|---|
| Festival | 257 |
| Quest (Keyward Bound) | 5466, `events/quest/054/SubGsc801_05466.lua` ([Quests](quests.md)) |
| Keybound Square | territory 1360 |
| The duty | ContentFinderCondition 1095 (content type 19), territory 1359, InstanceContent 57001 |
| The registrar | CustomTalk 986, then ContentEntry 120 |
| The MGC trader | SpecialShop 1549 |
| The ways in and out | Warps 561 and 563 (the Gold Saucer to the square), 562 (back) |
| MGC, the Keybound Coffer | items 52322 (at most 10,000 held), 52450 |

### The Gold Saucer and the square

* The event's NPCs are on the festival's layers, so they're out only while the event is on. In the Gold Saucer (144):
  the quest giver (layout 12479849), the guide (12479854: the quest's scene 2, and afterwards Warp 561 to the square)
  and a second way in (12479859, Warp 563). Each warp plays its ConditionSuccessEvent first (DefaultTalks 601297 and
  601302, nested, type 7), then its own scene 0 (0x2000), which returns `[1]` to go: PrepareZoning, ActorControlSelf
  147 with the warp's event ID, the event finished (`events/generic/Warp.lua` does this for every warp.)
* In the square (1360), arriving at pop range 12479949, (-0.04, 0, -5.91) facing 3.101: the magitype master (12479942)
  ends the quest and afterwards explains the game (PreHandler 135: DefaultTalk 601295 nested with argument 8, then
  Description 30's scene 0, flags 0x2001); 12479896 goes back (Warp 562, DefaultTalk 601298); the MGC trader
  (12479946) is SpecialShop 1549 (ActorControlSelf 138 `[1771021]` before its scene 0, flags 0x2001.)
* **The registrar** (12479940) is CustomTalk 986: scene 0 (flags 0x2000) returns `[1, 0x2C0078]` to play (closing it
  cancels the scene, error 50.) The ContentEntry it names (120) starts, nested (type 7), and its scene 0 (0x2000)
  returns `[2, 0]`; the duty goes into the Duty Finder (ContentFinderUpdate), both events finish, and
  ContentFinderFound, ContentFinderCommencing and the zone change to 1359 follow (282 talks, 275 entries.)
  `events/custom/009/Capture_00986.lua` and `ContentEntryEventHandler` play it; ContentEntry 120's duty comes from
  `fork_events.json`'s `content_entries` (`GameData::content_entry_duty` knew only the GATEs' sheets.)
* **A script by its number**: CustomTalk 986's client script hasn't been dumped, so its file is named for its number
  alone. `event::script_by_number` finds a custom talk's script as it already found a quest's: the sheet's name first,
  then any file ending `_00986.lua` in the same folder.

### The duty

Where it lives: `servers/world/src/server/keybound.rs` (the game), `resources/data/keybound_waves.json` (the waves and
enemies, retail's), `resources/scripts/content/w1en.lua` (a director with nothing in it), `server/action.rs` (a typed
word is the game's), `events/gold_saucer.rs` (`keybound_rewards`). One to four players matched by the Duty Finder
(four in 220 of the 223 runs, three in three).

**How it's played on the wire.** The client does the typing. A word that matches an enemy's is an ActionRequest for
action 49516 or 49517 (it alternates) on that enemy; a word nobody carries is 49520 on the player. Everything else is
the director's (InstanceContent 57001), as commands in ActorControlSelf 109 `[director, command, …]`
(`DirectorEvent::Raw`, new: up to seven numbers, for commands without names):

| Command | Numbers | When |
|---|---|---|
| 0x10000009 | | as the duty loads (0.8 s) |
| 0x10000007 | player, 12609, 12610 | once per player 2-3 s after the opening; `player, 708` after each wave |
| 0x10000006, 0x10000001 | | a wave's enemies appear; 5.0 s later the wave is on |
| 0x10000013 | 0, 1, 2, 4 | the phase: between waves, enemies appearing, wave on, over |
| 0x10000008 | enemy, group, 2000010 + group, word, next word, 100 | an enemy's words (the client shows both) |
| 0x1000000A | score | to the player who scored |
| 0x10000012, 0x1000000B | enemy | to the player whose word felled it; 0x1000000B to all 2.0 s after a fall |
| 0x10000016 | enemy, 3068 | an enemy fell |
| 0x1000000D | kills | the wave's count: every kill in wave 1, the big enemy's in the others |
| 0x10000004, 0x10000003 | HP | the barrier's HP now; set whole (40000) between waves |
| 0x10000014 | wave | a wave cleared (not the last) |
| 0x10000002, 0x1000000C | | a wave over; twice between waves (3.0 and 8.0 s after) |

**The run** (159 of 223 runs had exactly this order). The opening (the duty's usual scene; EventFinish 20.0-21.8 s
in), DutyCommence with 3600 s. Wave 1's enemies appear 7.5 s later; a wave ended, the next one's appear 12.6 s after
(30.5 s before the third, with a cutscene between: EventScene 3, flags 0x08502401, `[4024, 3, 20301]`, and the music
set to 1 and then 20301 around it). At a wave's end each player's score goes out as it stands and again with the
bonus: 10,000 × the barrier's HP / 40,000. Waves lasted 85 s (81.5-109), 135 s (62-242) and 147 s (50-265).

**The waves** (`keybound_waves.json`; times from the enemies appearing):

* **Wave 1** is one of three line-ups (77, 73 and 73 runs: picked evenly) of 29, 33 and 31 enemies in four or five
  groups. Every SpawnNpc and every start came at the same tenth of a second in every run of a line-up: the first group
  spawns 3.3-4.2 s before and sets off as the wave begins (4.9 s), the rest spawn in groups (5.2, 30-33, 40-58 and
  82-85 s) and set off one or a few at a time, 8-28 s after spawning. **It ends at 25 kills**, whoever is left (they
  were deleted within a second).
* **Wave 2** is one of three line-ups (85, 71, 67 runs) and one of three big enemies (BNpcBase 19593, 19594 or 19595),
  which gets its words 78 s in (70.6-93.5) and whose fall ends the wave. Its line-ups' three 19590s stood at three of
  twelve spots picked at random; Kawari has the commonest three.
* **Wave 3** is the last boss (19596, BNpcName 14933), spawned 20.2 s before the wave, during the cutscene, and given
  its words 5.0 s in. Its fall ends the game.

**The enemies** (all level 100, BNpcName 108 but the last boss; HP with four players / three):

| BNpcBase | Layout | HP | Walks (y/s) | Hits the barrier with | For |
|---|---|---|---|---|---|
| 19586 | 12406186 | 500 / 400 | 2.22 | 49521 | 400 |
| 19587 | 12406187 | 300 / 200 | 2.95 | 49522 | 400 |
| 19588 | 12406188 | 800 / 600 | 1.47 | 49523 | 800 |
| 19589 | 12406190 | 500 | 2.98 | 49525 | 400 |
| 19590 | 12406191 | 300 / 200 | stands | (spells 49526, 49527) | |
| 19591 | 12406192 | 800 / 600 | 1.5 | 49530 | 800 |
| 19592 | 12406193 | 400 / 300 | 1.5 | (49531) | |
| 19593 | 12406194 | 4800 | stands at (0, 0) | 49540 | 550 |
| 19594 | 12406195 | 4500 | stands at (0, 0) | 49549 | 780 |
| 19595 | 12406196 | 5300 / 4100 | 0.99, to z 0.8 | 49556 | 400 |
| 19596 | 12406197 | 8400 / 6600 | stands at (0, 0) | 49715 every 4.0 s | 400 |

An enemy spawns standing in its column (x -9, -4.5, 0, 4.5 or 9; z -30 to -12). As it gets its words (`0x10000008`
and ActorControl 54 `[1]`; ActorControl 407 `[7750]` 2.0 s before, for one that waited) it walks straight at the
barrier, stops at z 11.5 (the first blows came 35.5 yalms' walk after the words: 12.0 s for 19587, 16.0 s for
19586) and hits the nearest of the barrier's pieces at once and every 3.0 s. The barrier is nine NPCs (19585, layout
12406198, 188,300 HP) along z 17.5 and a tenth (20146) at (0, 0, 18), out all run; its 40,000 HP is the director's.

**A hit**: ActionEffect for 100 (every one of 48,173), the score, the enemy's new words (its next word moves up). At 0
HP: ActorControls 14 and 2 `[2]`, the commands above, ActorControl 607 `[enemy, 1, 0, 100]` 3.04 s on, DeleteActor
4.32 s on. A word at an enemy that fell meanwhile lands for nothing.

**Words** are numbers into the client's lists: list 2000010 (group 0, the common enemies': words 0-427 seen; 19592
only 0-20), 2000011 (the wave 2 big enemies': 0-969), 2000012 (the last boss's: 0-282), 2000013 (the words over the
barrier: 0-45). No word was on the field twice at once. Patch 7.58 took three words out, so Kawari gives 0-424, 0-966
and 0-279.

**Score.** A word is worth 100, 200 (wave 2's big enemy's) or 300 (the last boss's), times 1.0 to 2.0 by the player's
bonus level. The bonus gauge (0-10,000) gains 15 × the word's worth a hit (1,500; 4,500 for the boss's), holds
0.535 s, then drains about 620 a second (586-702 in clean stretches). Filled, it goes round; 1.2 s later it's set
full again, and going round after that is a level (a fifth more a word, five at most). Gauge and level begin again
with each wave. The gauge's own packet (opcode 330 in 7.57: `[held u32, 0, gauge u32, 0, server time in ms u64]`,
held as a hit lands and draining 0.535 s later) **isn't sent**: its opcode in 7.58 isn't known.

**Rewards** (223 results, to the unit): MGP = score × 0.046 (2,800 at most), MGC = score × 0.0058 (350 at most), and
a Keybound Coffer for a win; LogMessages 11598 `[MGP, 0]`, 11601 `[52322, MGC]`, 11595 `[52450, 1]` from the
director. A lost game (the barrier at 0) pays the MGP and MGC without the coffer.

**With fewer than four players** an enemy has less HP. Three (three runs) had 100 less on the common enemies and
1,200-1,800 less on the big ones; one and two weren't seen, and Kawari takes the same off again for each player
fewer, never under one hit (`EnemyKind::hp_for`: alone, 19587 falls to one word and the last boss has 3,000).

Tests (`keybound.rs`): `the_waves_read`, `an_enemy_s_hp_by_the_players`, `a_line_up_has_its_wave_s_big_enemy`,
`the_bonus_gauge_and_its_levels`, `a_score_s_rewards`, `a_command_goes_out_as_retail_s`, `a_word_nobody_carries`, and `a_typed_word_hits_and_the_third_fells` (a
player's three words at a 19587 in the test world: the blow for 100 with the request's sequence, the score, the next
words; at the third its fall, `DEFEATED`, `WORDS_GONE`, `ENEMY_DOWN`, `KILLS [1]`; a word at it afterwards and a miss
score nothing; another action stays the battle's). **Not played**, and nothing of it has
run against a client: what isn't built is in [Known Issues](known-issues.md).
