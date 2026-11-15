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

Where it lives: `servers/world/src/server/keybound.rs` (the game), `server/keybound/play.rs` (the casters, big
enemies and summons at play), `server/keybound_mechanics.rs` (what they do, read from
`resources/data/keybound_mechanics.json`), `resources/data/keybound_waves.json` (the waves and
enemies, retail's), `resources/scripts/content/w1en.lua` (a director with nothing in it), `server/action.rs` (a typed
word is the game's), `events/gold_saucer.rs` (`keybound_rewards`). One to four players matched by the Duty Finder
(four in 220 of the 223 runs, three in three).

**How it's played on the wire.** The client does the typing. A word that matches an enemy's is an ActionRequest for
action 49516 or 49517 (it alternates) on that enemy (51027 and 51028 under Impassioned Inputs; 49518 and 49519 on
the barrier's words); a word nobody carries is 49520 on the player. Everything else is
the director's (InstanceContent 57001), as commands in ActorControlSelf 109 `[director, command, …]`
(`DirectorEvent::Raw`, new: up to seven numbers, for commands without names):

| Command | Numbers | When |
|---|---|---|
| 0x10000015 | a content ID's high half, its low half | with ZoneInit: one for each player in the game, the player's own first |
| 0x10000009 | | the answer to a loading client (below) |
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
| 0x10000011 | notice, argument | a state's notice: on, and the next number as it ends (17835/17836 Impassioned Inputs, 17837/17838 a big enemy's, 17839/17840 a player's lock, 17841/17842 the barrier's shield) |
| 0x10000017 | switch, 1 or 0 | a state on or off (3078 Impassioned Inputs, 3071 19594's, 3072 the last boss's) |
| 0x10000005 | 12782296, 12782297 | 8.05 s after a win, with the result |
| 0x10000018 | | 3.05 s after a loss |

0x10000001 and 0x10000002 are also a player's typing on and off: retail sent them to one player alone around a lock
(below).

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
| 19590 | 12406191 | 300 / 200 | stands | (casts 49526, 49527 or 49528) | 1,500 |
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
with each wave. The gauge on screen is drawn from its own packet, **`KeyboundGauge`** (opcode 330 in 7.57, 436 in
7.58: `[held u32, 0, gauge u32, 0, server time in ms u64]`): held as a hit lands, draining 0.535 s later, twice at
10,000 as it's set full, 0 at a wave's end. The barrier's own words (group 3) are worth 400 (574 first-level words
scored 400; the rest 400 × 1.2 to 2.0).

**Impassioned Inputs.** A 19592's fall (67 of 67) gives everyone 20.0 s of it: notice `[17835, 5342]`, switch
`[3078, 1]`, status 5342 `[1180]` for 20 s on each player, every gauge held where it stands. The client then sends
its words as 51027 and 51028, each worth 1,000 whatever the level (533 of 548; the other 15 were 3,000, taken to be
the big enemies' words). At its end notice `[17836, 5342]`, switch `[3078, 0]`, and the gauges drain again. A 19592
has status 2056 `[1169]` from when it gets its words; one that reaches the barrier (3 of 70) does 49531 and is gone.

**The barrier's look** is the director's map effects (DirectorMapEffect `state, timeline, index`; 12,589 of them):
a blow on it `1, 2, 1` while more than 20,300 is left after it (down to 20,400 seen), `1, 4, 1` to 10,400 and
`1, 8, 1` from 10,280; its look changing `0x10, 0x20, 0` and `0x40, 0x80, 0` right before the first blow of each;
its fall `4, 0x400, 0`; whole again between waves `1, 2, 0` (only when it didn't look whole). Each map effect came
with the blow's `0x10000004`, before it.

**Casts** (`keybound_mechanics.json`'s `casters` and the rotations' `cast` steps). An ActorCast at one of the
barrier's pieces; the effect lands 0.3 s after the bar ends (a 2.7 s cast 3.0 s after it began, 11.7 s 12.0, 29.7 s
29.9); one that hurts the barrier is an ordinary blow with the map effect and `0x10000004`, the others an effect of
kind 27 with the action's own ID (kind 1 for 49547 and 49701). A caster that falls mid-cast: ActorControl 15
`[540, 1, the action, 1]`. A caster neither walks nor hits while it casts.

* **The counter**: a cast that can be broken begins with status 5354 with four stacks on the caster (ActorControl 20
  `[5354, 4]`, 0.09 s before the bar). Each word on it takes a stack off (`[5354, 3]`, `[5354, 2]`, `[5354, 1]`), and
  the fourth breaks the cast: ActorControl 21 `[5354, 1]`, then 1522 `[the action, n, the player]` (n was 2 to 10 in
  665 breaks, 6 in 230 of them: Kawari sends 6). 19589's 49524 (6.7 s; 2,000 to the barrier, landed 5 times in 383),
  19593's 49538 (11.7 s; 3,000), 19594's 49548 (11.7 s; 6,000), the last boss's 49703 (11.7 s; 3,000).
* **19589** casts as it gets its words and walks only once the cast is broken or has landed. **19590** (three a
  line-up) cast 49526 (5.7 s), 49527 (8.2 s) and 49528 (10.7 s), one each in any order (all six orders, 38 to 53
  times), 1,500 each to the barrier, and a 19590 falls by its own cast: ActorControl 2 `[2, 2]` and its words gone
  with the blow, ActorControls 14 and 2 `[2, 0]` 1.2 s later, 607 at 4.3 s. **19591** casts 49529 (4.7 s; the
  warning below) and then walks.
* **The warning and the lock**: 19591's 49529, 19593's 49532 and the last boss's 49702 (all 4.7 s) give every player
  a status for 2 s as they land (5346 for 49529, 5437 for the other two, 0.45 s after). A client whose player was
  caught by it says so itself: ClientTrigger 808 `[director, 0x10000002, 0, 0, 365]`, 4.4 to 5.8 s into the cast (8
  in 361 casts; what the player did to be caught is the client's to judge, and isn't known). Retail answered 1.4 s
  later (7 of 8; the eighth had 0 for 365 and got nothing): status 5347 for 7 s (parameter 25 for 49529's, 1170 for
  the others'), `0x10000011 [17839, 5347]` and `0x10000002` to that player alone (their typing off), then
  `[17840, 5347]` and `0x10000001` 7 s on. `Game::trigger`, from `ClientTriggerCommand::DirectorTrigger` (its
  `Unknown { id: 0x10000002, unk3: 365 }`.)

**The big enemies' rotations** (`scripts`; times from when one gets its words; every run of each the same to a few
tenths of a second):

* **19593**: 5.1 s in, 49533 (2.7 s), 50651 at +5.2, four bombs (19825) at +6.3, 49532 (4.7 s, the warning) at +13.3,
  50650 at +29.4; 36.5 s after it began the same again with 49538 (11.7 s, the counter, 3,000) for 49532, 46.5 s
  long; and so on in turn.
* **19594**: 49547 (2.7 s), its state 4.7 s in (`[17837, 5348]`, switch `[3071, 1]`, for 28.0 s), 49541 (2.7 s) at
  5.1, four 19827 at 9.8 on (±8, 0) and (±4, 5), 49548 (11.7 s, the counter, 6,000) at 16.3; again 36.4 s after it
  began, and every 46.3 s after that. A 19827 (200 HP, words of group 1 2.76 s after it comes) casts one of
  49542-49545 for 19.7 s (one each, in any order); the 25 of 600 that weren't felled first became a 19828 where they
  stood 0.7 s later, which walks to the barrier (2.75 y/s) and hits it with 49546 for 1,000 every 3.0 s.
* **19595**: 49550 (2.7 s), then a 19829 every 5.0 s from 4.2 s in, on a free one of eight spots, for as long as it
  lives. 49554 (11.7 s; 3,000) at 14.3, 37.4, 66.5 and 95.6 s, each with the barrier's words (19824) 0.3 s after it
  begins and the shield gone 14.0 s after; then 49555 (29.7 s) about 115 s in, which never landed in 75 runs (the
  enemy fell first), so what it does isn't known. A 19829 (200 HP) hits the barrier with 49551 for 670 0.76 s after it
  comes (ActorControl 36 with it), gets its words at 2.05 s, casts 49552 (6.7 s; 1,500, landed 33 times in 905) at
  2.8 s and is gone after it; felled, its own 49553 shows its fall.
* **The last boss (19596)**, 5.2 s in: 49699 (29.7 s) with the barrier's words (20079) 0.2 s after it begins; as it
  lands the helper (BNpcBase 9020, unseen at (0, 0)) hits the barrier three times with 49700 for 6,000 (1.9, 2.9 and
  3.8 s after), and the shield goes 35.0 s after the cast began. 42.1 s after that cast began comes one of three
  parts, then (8.2 s after the part's last action) the long cast again, then another part, then 49713 twice 8.1 s
  apart and 49711 (4.7 s; the helper's 49712 for 600 five times, 1.8 to 5.8 s after it lands) every 12.15 s until it
  falls. The parts (the first was *three* in 82 runs, *one again* in 64, *one* and *two* in 36 each; the second one
  of the other pair, *one* or *one again* after *two* or *three* and the other way round, 79 of 79):
  * *one* (56.6 s): 49710 (2.7 s), 50652 at +5.2, four bombs (19826) at +6.2 and one more in the front row at 11.4,
    16.6, 21.4, 26.6 and 31.4, 49702 (the warning) at 13.3, 49701 (2.7 s) at 22.4 and its state at 27.1
    (`[17837, 5472]`, switch `[3072, 1]`, for 30.0 s), 49713 at 56.6;
  * *one again* (56.6 s): 49710, 50652 at 5.2, four bombs at 6.3 and one more at 11.1 and 16.3, 49701 at 7.3 and its
    state at 11.9, 49703 (the counter, 3,000) at 34.4, 49713 at 56.6;
  * *two* (62.8 s): 50652, 49701 at 2.1 and its state at 6.8, 49704 (2.7 s) at 7.3, 17.4 and 34.6 with three, four
    and three 19923 5.2 s after each begins, 49703 at 46.7, 49713 at 62.8;
  * *three* (62.8 s): 50652, 49704 at 2.1 (three 19923 at 7.3), 49702 at 12.2, 49704 at 21.4 (four at 26.5), 49701
    at 26.5 and its state at 31.1, 49704 at 34.6 (three at 39.8), 49713 at 62.8.

  A 19923 (500 HP; any free one of sixteen spots) does 49705 0.76 s after it comes, gets its words at 1.25 s and
  casts 49706 (19.7 s; 3,000, landed 90 times in 1,321) at 2.84 s, and is gone after it.

**The bombs** (19825, 19826; 500 HP): four at a time, one in each column (x ±2.5, ±7.5), each in a row of its own
(z -18, -13, -8, -3); the last boss's later ones one at a time in the front row. 0.75 s after it comes ActorControl
36, its 49534 (an effect of kind 72) and ActorControl 54 `[1]`; its words and ActorControl 62 `[0, 1]` at 1.8 s, and
it walks at the barrier (1.44 y/s to z 14.2: its blast came 13.9, 17.4, 20.9 and 24.3 s after it did from the four
rows). There it does 49535 (49536 for 19826) for 3,000 and is gone (its words and ActorControl 54 `[0]` at once, 607
a second later). Felled, its own 49537 with ActorControl 2 `[2, 2]` shows it, and it dies a second later (so do its
words' end and ActorControl 607, each a second later than another enemy's).

**The barrier's words and its shield.** With the last boss's 49699 and 19595's 49554 an object comes at (0, 0, 18.5)
(20079 with 700 HP and words of group 3, list 2000013; 19824 with 500 HP and common words; 100 less with three
players) with **1 HP** of its own, and each player gets their own words for it. A word on it is ActionRequest 49518
or 49519: the action's effect is of kind 4 for 100 (1,603 of 1,603), the object's HP goes up by 100, the player
scores as for any word, and everyone gets map effect `1, 0x10, 1`. Filled: `0x10000011 [17841]` and map effect
`0x100, 0x200, 0`, and the object goes as a felled enemy does. The blows that follow then take nothing off the
barrier (998 of 1,001 of the helper's 6,000s came after a 17841; the other three took 18,000 off it): each is map
effect `1, 0x20, 1` without `0x10000004`. The shield's end is `[17842]` and the barrier's look (`1, 2, 0` whole).
Words nobody filled go as the cast lands.

**Loading in** (223 of 223 runs, in this order). With ZoneInit, `0x10000015` for each player in the game (their
content ID, the high half first: 375 different ones, each the content ID the result's packet has for a player). As
it loads the client sends the director trigger `0x10000001 [1, 0, a number of its own]` (the same number in every
run of one player's), and the server answers with **`KeyboundSetup`** (opcode 503 in 7.57, 413 in 7.58; 48 bytes:
the places no player has, i.e. 4 less the players; 6, 12743016 and 12743021 in every one; a number that differs
each run; the barrier's 40,000; sixteen zeros; the server's time in ms) and `0x10000009 [0]`. Only then does the
client send trigger 801 (for the director's variables) and its EnterTerritoryEvent, and the opening (scene 1, a
duty's own, the same bytes as Kawari's) plays. **Bob's infinite loading of 2026-11-15** (build `da8ad1c`, read
through the harness): Kawari sent `0x10000009` with ZoneInit and nothing for the trigger, so the client waited
45 s, sent its EnterTerritoryEvent without the 801, got no variables and never played the opening scene it was
sent. `keybound::player_trigger` → `loaded`, `Game::list_players`; test `a_loading_client_is_answered`.

**What a player has in the duty** (Bob's marks of 2026-11-15 on `4470363`, read through the harness: "typing
anything didn't send anything … I typed something slowly and correctly and got a can't use yet warning", and no
ActionRequest came in 52 s; then "leaving the event duty leaves my UI disabled I can't use esc, can only type in
chat"). Retail's runs against his:
* **status 5373 with 505**, from no one and with no end, in the first StatusEffectList of the duty (80 of 80 runs
  looked at); Kawari gave none. `keybound::PLAYER_STATUS`, given with the players' list.
* **two conditions**, bits 3 and 66 of Condition (`SufferingStatusAffliction` and `Unknown96` in Kawari's names),
  from ZoneInit to the way out (`08 02 00 10 | 00 00 00 00 | 04`, 223 of 223, and nowhere else in the captures of
  the square or of a quest battle); Kawari's Condition had neither. `KEYBOUND_CONDITIONS` (`zone_connection/zone.rs`).
* **the director's sequence 1** (DirectorVars' first byte) and **the map effects' states 1, 1**
  (DirectorSetupMapEffects64 `05 00 | 01 00 | 01 00`); Kawari's were 0 and 4, 4 (4 is the barrier's fall).
* `0x10000015` had gone out as `[0, 0]`: the player's spawn has no content ID until their client has loaded, so the
  list now waits for it.
* `0x10000013 [0]` as the duty commences (with its timer), and **the banners** (ActorControlSelf 123,
  ShowScreenImage `[image, 0xE0000000]`): 1620 2.0 s after the duty commences, 1624 with the first wave's
  `0x10000002`, 1621 8.05 s after, 1625 with the second's, 1628 8.05 s after, 1622 as the cutscene ends (26.0 s
  after), 1627 3.0 s into a loss (1,340 in 223 runs, always in that order).
* **The way out** (Bob, 2026-11-15, on `7c23f85`: "after finishing a game whether winning, or loosing I just stand
  there in the map"; his capture: scene 11 played and came back 17 s later, EventFinish, and nothing more). The end
  of one of the director's own scenes was only passed on for a quest battle's director
  (`ContentEventHandler::on_return`), so neither the result's end nor the way out's reached the game. Now Keybound
  Brawler's are too (`ToServer::DirectorSceneFinished` → `keybound::scene_finished`: the result's end plays the
  way out, the way out's end sends the player back). The same capture showed the first of the two conditions gone a
  tick after zoning in: the world's check for statuses that hold a player took it off (it wasn't one of those); it
  leaves a player with status 5373 alone now.
* **Leaving early** was in no retail run. Kawari turns the leaver's typing off first (`0x10000002`,
  `keybound::player_leaves`), a guess at what keeps the keyboard the game's after the zone change.

**The cutscene and the end.** 8.0 s after wave 2 the music is set to 1 (the content director's 0x80000001), at
10.1 s every player gets scene 3 (GameProgress with argument 1, flags 0x08502401, `[4024, 3, 20301, 0, 0]`), and at
26.0 s the music is 20301. **A win** (the last boss's fall): phase 4, the rewards' three lines, **`KeyboundResult`**
(opcode 173 in 7.57, 961 in 7.58; 240 bytes: `1, 52322, MGC, 52450, coffers, MGP`, then four places, four scores,
four content IDs and four account IDs in the order of the content IDs, a place being 1 + the number of higher
scores), scene 3 with flags 0x08502403 and `[4025, 1, 20302, 0, 0]`, `0x10000002` and three `0x1000000C`; 8.05 s
later `0x10000005` and the duty's completion. A player closing the result gets scene 11 (flags 0x0C002001, argument
7, as Air Force One's way out) and leaves as it ends. **A loss** (the barrier at 0): phase 5, the result's packet
with nothing won, `0x10000002`, the gauge held at 0, the music 1; 3.05 s later the music 20303, `0x10000018` and
`0x10000007 [player, 718, 3]` for each; at 6.1 s the duty's end (0x40000002, 0x40000007 `[0]` and `[1]`), the MGP
and MGC, and scene 11.

**Rewards** (223 results, to the unit): MGP = score × 0.046 (2,800 at most), MGC = score × 0.0058 (350 at most), and
a Keybound Coffer for a win; LogMessages 11598 `[MGP, 0]`, 11601 `[52322, MGC]`, 11595 `[52450, 1]` from the
director. A lost game (the barrier at 0) pays the MGP and MGC without the coffer.

**With fewer than four players** an enemy has less HP. Three (three runs) had 100 less on the common enemies and
1,200-1,800 less on the big ones; one and two weren't seen, and Kawari takes the same off again for each player
fewer, never under one hit (`EnemyKind::hp_for`: alone, 19587 falls to one word and the last boss has 3,000).

Tests (`keybound.rs`): `the_waves_read`, `an_enemy_s_hp_by_the_players`, `a_line_up_has_its_wave_s_big_enemy`,
`the_bonus_gauge_and_its_levels`, `a_score_s_rewards`, `a_command_goes_out_as_retail_s`, `a_word_nobody_carries`,
`the_result_s_places`, `a_word_under_impassioned_inputs`, `impassioned_inputs_begins_and_ends` (a 19592 felled in the
test world: the gauge held, the notice, switch and status; a word then worth 1,000 and answered as 51027; its end
20 s on) and `a_typed_word_hits_and_the_third_fells` (a player's three words at a 19587: the blow for 100 with the
request's sequence, the score, the next words; at the third its fall, `DEFEATED`, `WORDS_GONE`, `ENEMY_DOWN`,
`KILLS [1]`; a word at it afterwards and a miss score nothing; another action stays the battle's).
`keybound_mechanics.rs`: `the_mechanics_read` (every block a path names, every summon's spots and kind),
`a_rotation_s_steps_come_in_order` (the last boss's, through a long turn and round its last block),
`a_repeated_spawn`, `where_summons_stand`, `the_barrier_s_look`. `keybound/play.rs`, in the test world:
`four_words_break_a_caster_s_cast`, `a_cast_that_lands_hurts_the_barrier`, `the_barrier_s_map_effects`,
`the_barrier_s_words_raise_its_shield`, `a_client_asks_for_its_lock`, `a_state_ends_with_its_time_or_the_wave`.
**Nothing of it has run against a client**: what isn't known is in [Known Issues](known-issues.md).
