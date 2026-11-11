# World Bots

Bots shown as other players about the world: standing round the big aetherytes and the marketboards, walking between
a zone's places, through its exits into the next zone, into an inn and out again, and teleporting between aetherytes,
as retail's players did in Bob's captures. Off until the server's owner turns them on (ArchonEngine ControlPanel's
Dashboard → World bots). Asked for by Bob on 2026-11-11; delivered in stages.

| Stage | What | Status |
|---|---|---|
| 1 | The world's population: idlers at every big aetheryte (30 in Limsa Lominsa, 10 in Ul'dah, 20 in Gridania by default), 5-10 at each marketboard, walking between places, exits, inns, Teleport; the ControlPanel page | done (2026-11-11), not played yet |
| 2 | Questing bots: routes from Questionable's quest paths (levels 1-100, at least 10 bots per ten levels, no more than 3 on one quest and never two on the same step), fighting quest enemies, quest-reward and dungeon gear, a new name and look after a route | to do |
| 3 | More like players: minions, mounts, crafting in place, sitting in chairs, chat, search info | to do |

Code: `servers/world/src/server/world_bots.rs`; the settings page `kawari_web/worldbots.py` in ArchonEngine
ControlPanel; retail's standing spots `resources/data/world_bot_spots.json`.

## How it works

**Every bot is kept by the world server, wherever it is** (`WorldBots`, beside the instances), whether anyone is in
its zone or not: who it is (name, look, class, level, gear and glamour, online status, title, idle pose), its home
(a territory and the place it keeps coming back to: a big aetheryte or a marketboard), where it is, what it's doing
(standing, walking, casting Teleport, between zones, in an inn room) and until when.

* **In a zone a player is in** (a public instance of a town or open-world zone with a player in it), it's an actor
  of that instance: an NPC shown as a player, as party bots are (`party_bot::player_spawn`, a SpawnPlayer made from
  its NPC spawn), with a made-up content ID (`0x00B2…`, party bots' are `0x00B0…`), the server's world, its own online
  status, title and pose (`ShownAsPlayer` carries them now; party bots keep Party Member, no title, pose 0). It walks
  on the navmesh and the zone's collision as a party bot walks (`party_bot::run_along`), at the NPCs' run (6 yalms a
  second; retail's players ran 6.0-6.9). Clients get it as any actor in range.
* **Elsewhere its time just runs on**: a walk takes as long as the run would (the straight line, a third longer), a
  Teleport its cast and a loading screen, an inn its stay. When a player arrives it's there, where it stood or got to.
* **The NPC tick leaves world bots alone** (`Instance::world_bots`: they're skipped as a director's held helpers are),
  and **enemies never go after them** (`Instance::find_possible_enemies` leaves them out.)

**Its places** are read from each zone's layout the first time an instance of it is seen (`Places::of`), and kept:

| Place | From | |
|---|---|---|
| Big aetherytes | the zone's Aetheryte layout entries whose Aetheryte row has IsAetheryte; where Teleport lands is the row's first Level that's one of the zone's pop ranges | in a town or the open world (TerritoryIntendedUse 0 or 1) |
| Aethernet shards | the other Aetheryte entries (a city's guilds, markets, gates) | |
| Marketboards | event objects whose EObj Data is a CustomTalk (handler type 11) named `CmnDefMarketBoard…` | |
| Inns | ENpcs with a Warp handler (type 2) into a territory meant for an inn room (TerritoryIntendedUse 2): the innkeepers | |
| Exits | ExitRange entries into another territory, with their pop range there | |

The server logs what it found per zone ("World bots: territory 129: … big aetherytes, … shards, … marketboards, …
innkeepers, … exits"), and `!worldbots` says it for the zone you're in. None of it has been checked in game.

## Who's where, and what they do

From retail's players in Bob's Chronofoil captures: 124 captures, 404 player-hours of other players in the six city
zones (128-133), numbers only. The scripts are in the diagnostics repository's `notes/world-bots/` (`cityfolk.py`
reads every capture's other players: when they appeared and left, their moves, emotes, poses, ActorControls, their
spawn's class, level, mount, minion, online status, title and mode; `cityagg.py` sums them up and writes the standing
spots; `cityemotes.py` counts emotes, which come as ActorControlTarget 290.)

| What | Retail | World bots |
|---|---|---|
| How many in view | median 57 in Limsa Lominsa's Lower Decks (51-62), 13 in the Steps of Nald (7-21), 14 in New Gridania (6-36, up to 44) | Bob's numbers: 30 round Limsa's big aetheryte (129), 10 round Ul'dah's (130), 20 round Gridania's (132), by default; 1 at every other big aetheryte; 5-10 at each marketboard |
| Where they stood | the busiest 3-yalm cells where players stood still for 8 s or more (each city zone's are its aetheryte plaza and markets) | a recorded spot near its place (within 30 yalms of the crystal, 10 of a board), weighted by how long players stood there, a yalm or so apart; elsewhere a ring round it (4-12 yalms from a crystal, 2-6 from a board), on the navmesh |
| How long they stood | median 34-59 s by zone, a tenth 3-10 minutes (10th to 90th percentiles kept per zone) | drawn from the same percentiles |
| Idle pose changes | 29.6 per player-hour (ActorControl 295: 0, then the pose 0-6, cycled about evenly) | the same rate, the next pose each time |
| Emotes | 2.9 per player-hour; the commonest 112, 146, 211, 216, 105, 205 | the same rate, weighted as retail's, those that play once (no EmoteMode) |
| Weapon drawn or sheathed | 18 per player-hour (ActorControl 0) | not done |
| How they left | one in eight by Teleport (612 of 4,872 that left view): a 5.0 s cast (all 1,526), gone 5.9 s after it began | a trip somewhere else is a Teleport a quarter of the time: the cast shown (ActorCast, action 5), gone 5.9 s after |
| How they arrived | ZoneIn (ActorControl 200) about a second after they appeared (median of 6,062: 0.9-1.3 s) | ZoneIn a second after it appears, after a Teleport |
| Jobs | all of them, White Mage, Warrior, Paladin, Sage, Black Mage the commonest; crafters and gatherers 1% each | drawn from the same shares |
| Levels | 55% at 100, 3-7% in each ten below | the same |
| Online status | none 32%, New Adventurer 23%, Away 16%, Mentor 6%, Waiting for the Duty Finder 6%, Trade Mentor, Returner, PvE Mentor, Viewing a Cutscene… | the same shares, less a party's and Disconnected; the sprout only under 60, a mentor's crown only from 80 |
| Titles | two in three | two in three, any title of the Title sheet |
| Minions | more than half had one out (4,062 of 7,495) | not yet (stage 3) |
| Mounted when they came into view | none | none |
| Crafting in place, sitting | mode 5 (crafting, 334 times), mode 11 (sitting in a chair, 4% as they appeared), mode 3 (a looped emote such as sitting on the ground, 3%) | not yet (stage 3) |

**What a bot does.** At home, when a stay is over: most often another spot by its place (a short walk); three times
in ten somewhere else, if travel is on: a Teleport to another big aetheryte (a city's three times in four), or a walk
to another of the zone's places (a shard, a marketboard, another big aetheryte), an inn (in for 2-20 minutes, then out
where it went in) or an exit (gone through it, a loading screen of 4-10 s, then in at the next zone's pop range).
Away from home, after a stay it Teleports home six times in ten, or stays on. The shares of trips, the inn's stay and
the loading screen are guesses; retail's captures only see one side of a trip.

**Its look and gear.** As a party bot's (`party_bot::random_look`, `random_name`, `GameData::best_gear` for its class
and level, a glamour from the server's list for its sex and class, `wear_gear`, `wear_glamour`), with a player's HP
for its class, level and gear (`bot_parameters`). Stage 2 gives questing bots the gear their quests and dungeons
would have.

## Settings

`world-bots.json` in the server's data folder (`$KAWARI_DATA`, the container's `/kawari/data`, beside `world.db` and
`glamours.txt`), read again within 3 s of a change; a change makes a new population (those shown go):

```json
{"format": 1, "enabled": true, "cities": {"129": 30, "130": 10, "132": 20}, "per_aetheryte": 1,
 "per_marketboard": [5, 10], "travel": true}
```

* `cities`: by TerritoryType, how many gather round that town's big aetheryte (at most 200 each).
* `per_aetheryte`: every other big aetheryte in a town or the open world (at most 20).
* `per_marketboard`: between the two, drawn for each board, as its zone is first seen (at most 50).
* `travel`: off, they stay by their place.

Without the file (or with `enabled` false) there are none. ArchonEngine ControlPanel's
[Dashboard → World bots](kawari-web.md#world-bots-dashboard--world-bots-2280) writes it.

`!worldbots` in Say: whether they're on, how many and in which territories, how many shown, the places found in your
zone, and the five nearest with their home, class, level and what they're doing.

## Stage 2: questing bots (the plan)

[Questionable](https://github.com/PunishXIV/Questionable) (AGPL-3.0, as Kawari is) has a path for 4,389 quests: each
sequence's steps with their territory, position, the NPC or object (DataId), what's done there (AcceptQuest,
Interact, CompleteQuest, WalkTo, Combat with KillEnemyDataIds, UseItem, Duty, SinglePlayerDuty, Emote…: 12,439
Interact, 4,548 AcceptQuest, 4,307 CompleteQuest, 2,907 WalkTo, 1,571 Combat, 279 solo duties, 268 duties among
30,042 steps), and the aetheryte and aethernet shortcuts it takes, by its own names. Its file names and comments are
the game's text (quest and NPC names), so nothing of it goes in git: the server reads a checkout of it at runtime
(ArchonEngine ControlPanel will download it), as it reads the game's files. Its quest files are named by the quest's
ID (`551_….json` is Quest row 65536 + 551.)

What's planned: a route per bot through quests in level order (the Quest sheet's level, class and the quests before
it), at least 10 bots for each ten levels 1-100, no more than 3 on one quest and never two on the same step; walking
the steps, fighting the steps' enemies with their kit, talking to the quest's NPCs, Teleporting where Questionable
takes a shortcut, duties and solo duties as time away; their level and actions those of the quest they're on; gear
from the quests' rewards they'd have had by then and the dungeons of their level, a whole set more often than not;
after a route, a Teleport out and back as someone new.

## Not done yet

* Nothing has been seen in game: the places found, the spots, the bots' looks and walks, the Teleport cast and
  arrival, the inns and exits.
* Weapons drawn and sheathed, minions, mounts, crafting in place, sitting in chairs or on the ground (stage 3).
* Inviting, examining or searching for a world bot finds no character (nothing happens.)
* Bots don't turn to face their place after a walk (an ActorMove to turn on the spot makes the client play a step).
* Questing bots (stage 2).
