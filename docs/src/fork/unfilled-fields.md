# What Retail Sends That Kawari Doesn't

The client builds much of what you see from data the server sends and never shows: who stands in a room, which
dialogue a scene plays (by class, race, starting city, story so far), which seasonal decorations are up. A field Kawari
leaves at zero can change all of that without anything looking broken in the logs: the Waking Sands showed every visit's
people at once because ZoneInit's layer-filter key was 0 ([Travel](travel.md#layer-filters-the-waking-sands)). So the
fields retail fills in that Kawari doesn't are tracked here.

## Running it

```sh
python3 tools/questkit fields --retail <Chronofoil captures or folders> --kawari <Kawari captures or folders> \
    [--packets ZoneInit,UpdateQuest,...] [--share 0.1] [--min-count 3] [--out report.md]
```

Retail's side is Chronofoil captures of the game version Kawari's opcodes are for (`--all-versions` takes older ones,
whose layouts can differ); Kawari's side is its own packet captures (diagnostics bundles' `captures/`). It reports:

* packets retail sends that Kawari never sent;
* bytes of the packets both send that retail sets (in at least `--share` of its packets) and Kawari always sent as
  zero, with retail's commonest values for each, read at their natural width;
* packets whose sizes differ.

Packets carrying other players (names, content IDs, social and party lists, examine, market listings) are left out,
since only numbers of one's own may come out of captures. The full report stays outside the repository; what it
teaches goes in the table below. A low `--share` (0.005) with `--packets` finds the rare ones: the Waking Sands' key
was set in one ZoneInit of 72.

## Findings

First run, 2026-10-26: Bob's twelve Chronofoil captures of 2026.10.16 against the 344 Kawari captures in the
diagnostics bundles. Some of what it lists is only that the characters differ (a longer crafting log, more mounts);
those aren't listed here.

| Packet | Bytes | Retail | Kawari | What it is | Status |
|---|---|---|---|---|---|
| ZoneInit, ActorSetPos | 8–11, 4–7 | 90338, 90339 in the Waking Sands; other keys in six later zones | 0 | the layout's layer filter: which stage of the story a place shows | sent from `layer_filters.json` (the Waking Sands); the rest need their quests matched |
| QuestActiveList, UpdateQuest | each quest's byte 4 | the class/job it was accepted as (24, 32, 39…), every quest | 0 | `accept_class_job`; the client's quest scripts can branch on it | **sent** (2026-10-26): kept with the quest from when it's accepted, and from Auracite (Kawari) backups |
| UpdateClassInfo | 12–15 | 604800 (a full rested bar) and others | 0 | `rested_exp` (also in UpdateRestedExp, which Kawari sent) | **sent** (2026-10-26) |
| UpdateClassInfo | 1 | 58, 72, 17… every time | 0 | not read by the client (Kawari's struct) | left |
| AcceptQuest | 4–5 | 3764 seven times, 678 | 0 | unknown (Kawari has it as padding) | open |
| ZoneInit | 0 | the world server's ID | 0 | informational (Kawari's struct) | left |
| ZoneInit | 12–15 | a layout instance in the zone (a quest's POPRANGE*, a duty's exit); small numbers after a teleport (9 arriving in Ul'dah, 24 in Mor Dhona, 19, 11, 15: apparently the aetheryte's row; Connor's Chronofoil capture af05916c, 2026-11-09) | 0 | `pop_range_id`: where one comes in | open |
| ZoneInit | 32, 36 | 5; 1 sometimes | 0 | the data centre for ranked Crystalline Conflict; limited-time bonus | left |
| ZoneInit | 38–101 | game festival 120 (phase 2 in 72 %), UI festivals 148, 176, 256, 120 | config's `active_festivals` (none) | the seasonal events running (October 2026's) | Bob's choice: `world.active_festivals` |
| ZoneInit | 104–115 | a position | 0 | where level streaming preloads | open |
| ZoneInit | 116–126 | 1s, 2s and 4s | 0 | content roulette bonuses | open |
| PlayerSetup | 426–446 | 148, 176, 256, 120; 2 | 0 | the UI festivals again | with the festivals |
| SpawnNpc | 594–607 (inside the name, which battle NPCs don't use) | set in 77 % | 0 | unknown | open |
| EventScene64 | most of 9–262 | set in up to 100 % | 0 in 135 | the scene's arguments: Kawari's scripts pass fewer | per quest, with `questkit compare` |
| QuestTracker | 1, 10–19 | set with several quests tracked | 0 | the other tracked quests | open |
| EventNotice2, EventLogMessage2 | 3, 10–13; 1 | some notices; 117–122 | 0 | unknown arguments | open |

Packets retail sends that Kawari never sent, among others: PartyMemberPositions (485: the party's places on the map),
GatheringPointPhase (420), UpdateMapMarkers 4/8/16/32, StatusEffectList3, UpdateRecastTimes, Mount, Unknown533 (53, 32
bytes: an event NPC's instance and a quest's event ID, as a quest became available) and over forty unknown opcodes.
