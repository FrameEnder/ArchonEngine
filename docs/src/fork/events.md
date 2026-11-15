# Events

Seasonal and special events: how one is turned on, what the fork has built, and how each was played on retail.

Bob, 2026-11-15: "recreate the entirety of the Keybound event … an Event tab in the ControlPanel to select any
implemented Events to be turned on or scheduled."

## Turning an event on (2026-11-15)

Where it lives: `servers/world/src/server/game_events.rs`, `resources/data/fork_events.json`, and ArchonEngine
ControlPanel's Dashboard → Events ([ArchonEngine ControlPanel](kawari-web.md#events-dashboard--events-2330)).

**An event is a festival to the client.** ZoneInit's game festivals (eight IDs, each with a phase) decide which of a
zone's layers are out: an event's NPCs, decorations and objects sit on layers with its festival ID. The UI festivals
(the same list in ZoneInit and PlayerSetup) decide what the menus offer. Retail ran Keybound Brawler as festival
**257** (`festivals.yml`): every ZoneInit in Connor's Chronofoil captures of 2026-11-03 to 11-09 and Bob's of
2026-10-29 had the game festivals `[257, 256, 120]` with phases `[0, 2, 0]`, and the UI festivals `[257, 0, 256, 120]`
with phase 2 on 256. In the duty the game festivals were all 0 and the UI ones `[257, 256, 120]`, and three
ActorControlSelfs right after its ZoneInit set them one at a time: `[257]`, `[257, 131328]`, `[257, 131328, 120]`
(131328 = 256 with phase 2 in its upper half; category 924 in that version, SetFestival.)

* **`resources/data/fork_events.json`** lists the events the fork has built: `key`, `festival`, `phase`, `quests`,
  `duties`, `territories`, `content_entries` (the ContentEntry events that put a duty in the Duty Finder), and when
  retail ran it (`retail_start`, `retail_end`: Unix seconds, null when not over). No names: the ControlPanel reads the
  festival's from `festivals.yml` and the quests' and duties' from the game. **A new event gets an entry here** and
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

One to four players matched by the Duty Finder (four in 220 of the 223 runs); three waves of enemies marching on a
barrier, each enemy carrying words; a typed word is an attack. See `notes/keybound-brawler.md` for the director's
commands as retail sent them. **Not built yet** ([Known Issues](known-issues.md)): entering it from the registrar
loads the arena with nothing in it.
