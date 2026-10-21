# Achievements, Portraits and Recommendations

Three of the character windows Bob tried on retail (2026-10-16) and found broken or missing on Kawari. What the
client asks for and expects comes from reading `ffxiv_dx11.exe` (2026.10.06) beside FFXIVClientStructs; the
Recorder doesn't capture these packets (it only keeps packets addressed to quests and directors), so nothing here is
checked against a retail capture yet.

## Achievements (`servers/world/src/zone_connection/achievement.rs`)

**What the client does.** The Achievements window asks with client trigger **1001** (Achievement::
RequestCompletedAchievements, `ClientTriggerCommand::RequestAchievements`) and stays on "loading" until the reply, the
**Achievements** packet (server opcode **991**, 552 bytes, `fork_opcodes.yml`; FFXIVClientStructs' AchievementsPacket):

| Offset | Size | Contents |
|---|---|---|
| 0x000 | 510 | Completed achievements, Achievement sheet rows, **lowest bit first** (row 1 is bit 1 of byte 0). IDs above 4077 are ignored. |
| 0x1FE | 10 | The last five completed (u16), newest first. |
| 0x208 | 26 | Unknown (the client copies it beside its near-completion data); sent as zeroes. |

Other requests and replies:

| Client | Server | Notes |
|---|---|---|
| trigger **1000** `RequestAchievementProgress { id }` | actor control **514** `AchievementProgressInfo { id, current, max }` | The counter shown under an achievement. |
| trigger **1002** `RequestNearCompletionAchievements { slots }` (bit 0: the login notice, bit 1: the window's Near Completion page) | actor control **538** with the slot in its first parameter | 538 empties that list and marks it loaded; the server sends it for each requested slot, so both lists are empty for now. |
| | actor control **515** `AchievementCompleted { id }` | The client marks it complete, adds it to its history and shows the "Achievement unlocked" pop-up. |
| | actor control **600** | Marks the first parameter complete when the second is 1, and takes the third and fourth as its progress. Not used. |

The actor control cases were read from the client's actor control switch (`0xB3A2E0`, jump table at `0xB47964`):
514 calls ReceiveAchievementProgress, 515 SetAchievementCompleted and the pop-up queue, 538
ReceiveNearCompletionAchievements with no bitmap.

**What the server keeps.** `unlock.achievements` (510 bytes) and `unlock.achievement_history` (five little-endian
u16, newest first), migration `update-2026-10-16_achievements_portrait`, sizes `ACHIEVEMENT_BITMASK_SIZE` and
`ACHIEVEMENT_HISTORY_SIZE` in `constants.yml`.

**Completing them.** After finishing a quest, levelling up, discovering a map part, attuning a zone's aether
currents and defeating an enemy, the server completes (with the pop-up) every achievement whose requirements are
met, for the Achievement sheet Types it understands. What each Type means comes from what the sheet's Key and Data
hold for that Type's achievements and from when retail completed them (Bob's Chronofoil captures), not from the
client. **For the list Types the list starts in Key and goes on in Data**: A Life of Adventure I is Key 44, Data 55
(both achievements); Gone from Gridania is Key 66043 (The Gridanian Envoy) with Data empty; I Got the Magic Stick I is
Key 6 (conjurer), Data[0] 10 (the level). Until 2026-10-17 Kawari read only Data, so Types 3 and 6 never completed and
Types 2 and 9 missed their first ID.

| Type | Achievements | Needs | Progress shown |
|---|---|---|---|
| 1 | 2149 | Counter Key reaching Data[0] (only the counters below) | count / wanted |
| 2 | 67 | Every achievement in Key and Data | done / listed |
| 3 | 267 | The class or job Key at level Data[0] | level / wanted |
| 6 | 382 | Every quest in Key and Data | done / listed |
| 8 | 202 | Every part of map Key discovered ("Mapping the Realm"): Map's DiscoveryFlag, of every map that shares its discovery slot (DiscoveryIndex); A Realm Reborn's field zones are revealed whole on the first visit (below) | parts / all |
| 9 | 10 | Any quest in Key and Data | 0 or 1 of 1 |
| 20 | 30 | Every aether current of AetherCurrentCompFlgSet Key ("Freebird") | 0 or 1 of 1 |

**When retail completes them** (Bob's sessions of 2026-10-14 and 16, `questkit chronofoil`):

* Mapping the Realm: Central Shroud (657, Type 8): on entering the Central Shroud for the first time from Gridania. The
  client sends NewDiscovery as it arrives; the server answers with the achievement, LocationDiscovered with **part
  164**, which reveals the whole map, and the discovery EXP. Retail does that for **A Realm Reborn's field zones**
  (TerritoryType ExVersion 0, TerritoryIntendedUse Overworld): exactly those 17 zones' Mapping the Realm achievements
  say "Visit ... and unlock the area map", all 185 others "Discover every location" (XIVAPI). Kawari now sends 164
  for those zones (`WHOLE_MAP_DISCOVERY`, `GameData::reveals_whole_map`) and saves every part, which completes the
  achievement; elsewhere maps are revealed part by part as before.
* I Got the Magic Stick I (45, Type 3): turning in a quest that took conjurer to 10, right after LevelUpMessage.
* Storyteller I (789, Type 1, Key 214): turning in the tenth quest. Key 214 is unique quests completed; Kawari counts
  it from the completed quests, so it's right for existing characters too.
* Gone from Gridania (311, Type 6): turning in The Gridanian Envoy in Ul'dah.
* **Discover every location** (Bob's sessions, 2026-06 to 09: Mt. Gulg, Amaurot, The Grand Cosmos, eight Alexander
  floors): each with the last part discovered. Some duties' maps share a discovery slot and are done together: The
  Cuff of the Father is map 238 (parts 1-2) and map 255 (parts 3-4), both DiscoveryIndex 58, and Gordias II (Key 238)
  came with part 4; Midas II (Key 287) needs map 288's parts too. Comparing against Key's own DiscoveryFlag would have
  completed Midas II after one discovery.
* The 31 unlocks in Bob's June–September sessions, by Type: 11 Mapping the Realm, 8 quests (Type 6), 2 Freebird, 1 made of others (Frequent Flyer
  III, with the last Freebird it needs) are all completed as retail did now; 8 counters (Triple Triad cards,
  orchestrion rolls, Wondrous Tails, five duty clears) and 1 Type 21 (minions) aren't tracked.
* Each 515 `AchievementCompleted` is followed by an **ActorControl 518** (not Self) with the same ID, from the player.
  Kawari doesn't know 518 (presumably what others nearby see); it isn't sent yet. On every zone-in retail also sends 600
  `AchievementProgress` for the achievements being tracked, then 538.

**Counters** (Type 1, `unlock.achievement_counters`, migration `update-2026-10-16-3_achievement_counters`) are kept
per character by Key. There are 1128 Keys, which the sheet only describes in words ("Defeat 100 enemies" is Key 11,
"gil from defeating enemies" 12, levequests 13–22, synthesis by craft and level 23–62, gathering by job, level and
region from 63…, unique quests 214). The server counts **Key 11** (an enemy the player fought was defeated, as quests
count kills) and **Key 214** (unique quests: the completed quests, counted from the character's quest log, not kept).
Counters start at zero when the fork started counting them, where retail's count from the character's creation.

**It has to be quick.** The check runs on the player's connection while their packets wait. In `de2d795` it read the
Map sheet for every Mapping the Realm achievement (202, twice each, every pass), which took seconds after finishing a
quest or discovering a map part; the client's packets then stopped parsing ("desynced with the client's Oodle
state" in the log) and the player was left frozen, or revived into an endless loading screen (2026-10-16 bundles).
The Map sheet's discovery columns are now read once (`GameData::map_discovery`), and the server warns when a check
takes over 50 ms.

Everything else (the other counters, hunting log, dungeons, PvP, beast tribes, Grand Company…) isn't tracked; those
can be given by hand with `!achievement <id>`. `!achievement clear <id>` takes one away, `!achievement check` runs the check (for
characters that met requirements before this existed). The sheet is read once, the first time it's needed.

## Portraits

**What the client does.** Client trigger **3200** (Kawari's `OpenPortraitsWindow`) asks for the **adventurer plate's
portrait**; the client sends it whenever it doesn't have it yet, e.g. opening the Portraits window. The reply,
**PortraitsInformation** (182, 56 bytes), is copied into CharaCard's CurrentBannerData (FFXIVClientStructs'
BannerData, 52 bytes: a "has one" byte, expression, lighting colours and brightness, then u16s for the banner pose,
its progress, head and eye direction, camera position and target, image rotation, light angles, decoration,
background and frame, and a u32 checksum of the gear) and marks it received. Setting a portrait as the adventurer
plate's sends **SetAdventurerPlatePortrait** (client opcode **107**, 56 bytes, the same 52 bytes) — a fork opcode,
found by the function that sends it.

The portraits themselves (the list in the window) are the client's own: BannerModule keeps them in the character's
local UI save data, not on the server.

**What the server does now.** Saves the 52 bytes it's sent in `volatile.portrait` (same migration) and sends them back
for 3200; a character who never set one gets zeroes, as before.

**The "crash" was the server.** Opening the window (Character → Portraits) disconnected the client with error 90002
("Connection with the server was lost"), because the world server panicked: the window asks for the player's own
adventurer plate (RequestAdventurerPlate), and upstream's `lookup_adventurer_plate` (`database/character.rs`) read
the Grand Company rank as `company_ranks[active_company - 1]`, which underflows for a character in no Grand Company
(0). It now sends rank 0 then, as the other two rank lookups already did. (For reference, the window's addon is set
up at `0x14658B0`, naming "BannerList"; CharaCard's portrait getters are at `0xBC5540` and `0xBC5970`.)

## Adventurer Plate

**What the client does.** Opening a plate (your own, or another player's) sends **RequestAdventurerPlate** with the
actor; the reply is **AdventurerPlate** (server, 480 bytes; FFXIVClientStructs' CharaCardPacket). Checked against
Bob's Chronofoil capture of retail (2026-10-20: his plate opened and saved eight times, one section at a time):

| Offset | Field | Retail |
|---|---|---|
| 0x00 | FC crest (CrestData, u64) | 0 without one |
| 0x08 | account ID (u64) | the character's account, nonzero |
| 0x10 | content ID (u64) | |
| 0x18 | actor | |
| 0x1C | a state number (u32) | 2 |
| 0x20 | world (u16) | |
| 0x22 | level of the class/job they're on (u16) | |
| 0x24 | that class/job (u8) | |
| 0x25 | **sex** (u8) | 1 for Bob's female character, matching customize byte 1 |
| 0x26 | Grand Company, rank | |
| 0x28 | **CharaCardData** (188 bytes, `core/src/ipc/zone/server/chara_card.rs`) | |
| 0xE4 | **timestamp** (u32, Unix seconds) | 0 for a plate never saved; afterwards the time of the last save |
| 0xE8 | search comment (193 bytes) | |
| 0x1A9 | name (32 bytes) | |
| 0x1C9 | free company name (22 bytes), one empty byte | |

CharaCardData is the plate: version, the portrait's expression, zoom and lighting, class/job, the character's
customize data, dyes, gear visibility, borders, preferred job, active hours, play styles, flags (2: visible to no
one), byte 0x43 (1 on retail, meaning unknown), privacy flags (1: friends only), more dyes, byte 0x51 (`unk14`), the
pose and camera (u16s), decoration, background, frame, title, **base plate**, five decorations, glasses, a u32 (0 on
retail; ClientStructs calls it a timestamp) and the twelve gear item IDs.

Saving the plate in the window sends **UpdateAdventurerPlate** (client opcode **795** on 10.06, 350 on 7.57h1;
200 bytes, a fork opcode): a u32 naming **the window section saved** (1 portrait, 2 profile: title, active hours,
play styles; 3 design: base plate and decorations; 4 settings: who can see it), the whole 188-byte CharaCardData, the
timestamp of the plate it last received, and 4 bytes of leftover memory; the client then asks for the plate again.
Its `unk14` is leftover memory too (80, 25, 1, 149, 137, 9 in the capture): retail stores and sends back 0. The
version the client saved (4) is kept, even when retail's had been 3.

**What the server does.** Saves the 188 bytes in `volatile.adventurer_plate` with `unk14` set to 0, and the save time
in `volatile.adventurer_plate_saved` (migrations `update-2026-10-16-2_adventurer_plate`,
`update-2026-10-20_adventurer_plate_saved`), committed straight away so the re-request and other players see it,
and logs the section ("Adventurer plate saved (section N)"). AdventurerPlate sends them back with the character's
account ID and sex and the saved time (0 if never). Before, Kawari sent account 0, sex 1 (so male characters' plates
said female) and the current time as the timestamp, and echoed the client's `unk14`. Kawari has no free companies,
so the crest and FC name are empty. The level is the class/job's own (upstream sent 100).

**A plate never saved** is sent as 188 zero bytes (version 0), as retail sent a brand-new character's (Bob's capture of
the Limsa opening, 2026-10-20: CharaCardData all zero, timestamp 0, no comment); the client then makes its own
starting design from the character when the window opens, and the first save (section 1) sends it: version 4, zoom
160, white directional and grey ambient light, banner timeline 1, decoration 1, background 8, frame 1, the
character's customize data and gear. Kawari used to send upstream's made-up design instead (expression 22, base plate
78, active hours, play styles, preferred job 38). `CharaCardData::never_saved` (test `never_saved_is_zeroes`).
`adventurer_plate_layout` (`core/src/ipc/zone/server/mod.rs`) checks the offsets.

## Recommendations

The list itself is worked out by the **client** (AgentRecommendList reads RecommendContents against its own level
and quest state); no server packet feeds it. But the window has to be **unlocked**: AgentRecommendList's
IsActivatable (`0xC9F340`) checks **unlock link 9** (UIState IsUnlockLinkUnlocked) and that main command 31 is
enabled. Unlock link 9 is Spirithold Broken's `REWARD0` quest parameter, which the server grants when the quest is
finished (see [Quests](quests.md#quest-rewards-and-exp)); until the fork did that, the main menu's entry was greyed
and the window answered "That command is unavailable at this time" (LogMessage 488).

## How the client decides a main menu command is available

UIModule's ExecuteMainCommand (vf206, `0x78BEE0`) prints LogMessage 488 whenever IsMainCommandUnlocked (vf207,
`0x78C5B0`) says no. For commands above 26 that looks the command up in a table at `0x28A8270` (16-byte entries:
main command, addon, agent ID, condition flags), asks the agent's IsActivatable (vf8), then refuses it under some
conditions when the flags say so. Entries: 31 Recommendations → agent 109 (flags 1), 92 Portraits → 407 (flags 1),
93 Adventurer Plate → 413 (flags 0). AgentCharaCard's IsActivatable (`0xDC1220`) is only "not condition 56"
(BoundByDuty56, Kawari's wire bit 28: the networked conditions are numbered in Condition sheet order, `IsNetworked`
rows only, so wire bit n is the n-th of those), so the Adventurer Plate is only refused while bound by duty. (The
"unavailable" messages in Bob's 2026-10-16 screenshot were the Recommendations window's: the bundle shows the
plate opening and being saved at the same time.)

## Hair highlights

The client marks hair highlights as on with **0x80** in CustomizeData's byte 7 (retail's SpawnPlayer: 786 with 0x80,
348 with 0, never 1, in Bob's Chronofoil captures; Sapphire: "negative to enable"), and character creation sends it as
"128". Kawari read only "1" as on, so a new character's highlights were dropped, and physis writes the flag as 1, so
nobody saw them in game (only in character creation). Now any non-zero value counts, the lobby's list writes "128",
and packets write 0x80 (`write_customize`, used by CommonSpawn and the adventurer plate). Characters made before this
have the flag saved off and need their highlights set again.

Checked on Bob's 2026-10-18 bundle: a character made after the fix (`77d5315`) gets 0x80 in its own SpawnPlayer, the
same 26 bytes character creation sent, and retail's own-character SpawnPlayer and lobby list carry exactly the
bytes of character creation too (Bob's Chronofoil captures: the lobby's `"128"` and the spawn's 0x80 for the same
character), so Kawari now matches retail on both. The flag can be lost before it reaches Kawari:

* characters saved before `77d5315` with highlights have them saved off;
* **Auracite** dumps never have them: its Dalamud plugin reads `Customize[HasHighlights] == 1`
  (`Auracite/Steps/AppearanceStep.cs`), and the client holds 128, so `has_highlights` is always false. Importing a
  dump can't recover them.

The aesthetician (`CmnDefBeautySalon_00148`) had the same bug: its yield 27 carries the new appearance with the flag
as 128 (Bob's 2026-10-18 bundle) and the script only took 1 as on, so everything but the highlights was saved. Fixed
2026-10-18; the aesthetician is the in-game way to turn them back on.

Or give the character a Fantasia (Kawari Web's character page, "Fantasia (remake mode)", while
they're logged out) and edit the appearance at the lobby: the client sends the new appearance with `"128"`, which is
now kept.
