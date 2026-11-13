# Outside Sources

Square Enix's server scripts (the directors, the BNpcs' AI, enemy action lists) never reach the client, so most of what
the fork does in combat is inferred. This page lists the outside sources worth checking when a recording leaves
something unclear: what each one can answer, how far to trust it, and how to reach it. When two sources disagree, the
higher one in **Order of trust** wins, and the page that uses the result says where it came from.

None of these go in git. Code and data are read or fetched when needed, and only numbers and IDs make it into scripts,
with a comment naming the source (see `CLAUDE.md`).

## Order of trust

1. **Bob's retail recordings** (Kawari Recorder, [Recorder and diagnostics](recorder-and-diagnostics.md)). The right
   patch and content, every actor, target, cast, director variable and battle talk line with timings.
2. **Raw retail captures from elsewhere**: Chronofoil. Ground truth, but someone else's run, often on another patch.
3. **Curated timelines from many runs**: Cactbot (and FFLogs). Hand-checked against many logs, but they record
   what was seen, not why.
4. **Data gathered from captures**: Sapphire's and Teamcraft's enemy data. Good numbers, but their coverage and age vary.
5. **The client's own data and structures**: the game's sheets (XIVAPI, EXDSchema), FFXIVClientStructs. Exact for
   what the client knows, and silent about server-side behaviour.
6. **Written guides**: Console Games Wiki, the Fandom wiki. They say what a fight is meant to be (useful for intent),
   but they're edited by players and don't give timings.

## What to use for what

| Question | First look at | Then |
|---|---|---|
| How does this fight go: phases, waves, who does what | A recording | Bob's Chronofoil captures (`questkit chronofoil --duties`), Cactbot's timeline for the content |
| Which actions an enemy has | A recording | Boss Mod's module (`questkit sources --duty`, `--action`), the Fandom enemy actions page (by name), Cactbot's triggers |
| An action's shape, size, cast time, telegraph | The Action sheet (`CastType`, `EffectRange`, `XAxisModifier`, `Cast100ms`, `Omen`) | Boss Mod's shape for it (a cone's angle when the omen has none), the Fandom page, which lists the same columns by action |
| An enemy's level, HP and where it stands | A recording | Teamcraft's `monsters.json`, Sapphire's BNpc data |
| A packet's layout or opcode | A recording, then `kawari-opcodescan` | Bob's Chronofoil captures (sizes, every version since June), OverlayPlugin's packet structs, Machina, FFXIVOpcodes, Kawari upstream |
| What a sheet column means | EXDSchema | XIVAPI's documentation |
| A client structure (for the Recorder or Probe) | FFXIVClientStructs | |
| Crafting rules (formulas, actions, conditions) | raphael-rs, then Teamcraft's simulator and Craftimizer (where they agree) | The client's craft event handler for packets and CP rules |
| Gathering points, timed windows, boon and yield | A recording (`--dropin --gathering`) | GatherBuddy / GatherBuddy Reborn, Teamcraft's node data |
| A quest's steps: whom to talk to, what to use, which sequence, its variables | A recording (`questkit replay`) | Questionable's path (`questkit questionable`: checks every quest script, `--hints` for a new one) |
| A duty's doors, switches, keys, coffers and boss spots | A recording | AutoDuty's path (`questkit autoduty`: checks the duty scripts, `--draft` for a new duty) |
| What bites where, its tug and hookset, its hours and weather | A recording of a fisher (none yet) | AutoHook's fish data with Teamcraft's catch logs (`resources/data/fishing/`) |
| Triple Triad's rules | The game's TripleTriadRule sheet (names) | FFTriadBuddy's game simulation (`servers/world/src/triple_triad.rs`) |

## The sources

### Chronofoil

[ProjectChronofoil/Chronofoil.Plugin](https://github.com/ProjectChronofoil/Chronofoil.Plugin) is a Dalamud plugin that
records all game traffic. Its captures can be uploaded (censored of session IDs, chat and letters) to a central
preservation archive ([Chronofoil.Web](https://github.com/ProjectChronofoil/Chronofoil.Web)). Whether and how the
archive can be downloaded hasn't been checked. redstrate's
[PacketAnalyzer](https://zff.dev/redstrate/PacketAnalyzer) (live at analyze.xiv.zone) decodes Chronofoil captures with
the packet definitions Kawari uses.

* **Answers**: everything a recording does, for content Bob hasn't run, and how a fight went on an older patch.
* **Limits**: captures carry other players' characters and game text, so they stay out of every repository's history
  (the diagnostics one included); only numbers come out of them. A capture from another patch needs that patch's
  opcodes. Recording with Chronofoil next to Kawari Recorder also gives a raw copy of a run.

**Bob's own captures.** Bob has run Chronofoil since June 2026. His captures are attached to release `1` of
`kawari-diagnostics` (`chronofoil.zip`, never committed): 42 sessions from 2026-06-24 to 2026-10-16 over game versions
2026.06.18, 09.01, 09.15 and 10.06, most of the June–September ones also as censored copies, three unfinished.
`questkit chronofoil` ([questkit](questkit.md#chronofoil)) indexes and exports them. What's in them (2026-10-17):

* A Realm Reborn: Haukke Manor, Castrum Meridianum, The Navel, the Binding Coil (turns 3 and 5), the Second Coil
  (turn 4), the Final Coil (turns 1–4), Syrcus Tower, Akh Afah and The Striking Tree (Extreme), and the four quest
  battles of 2026-10-14 and 16 (duties 296–299, the same runs as the Recorder exports of those days);
* Heavensward's Alexander (all eight Father, Son and Creator turns); Shadowbringers dungeons, trials and Eden;
* The Palace of the Dead (floors 1–10, twice, 2026-10-15) and Blunderville (six matches) for the
  [research notes](research.md).

Where Chronofoil keeps them under XIVLauncher.Core: `~/.xlcore/wineprefix/drive_c/users/<user>/AppData/Local/chronofoil/`
(the plugin's `StorageDirectory`, set in `~/.xlcore/pluginConfigs/Chronofoil.json`). `<id>.cfcap` is a finished capture,
`<id>.ccfcap` a censored copy (`cfcli cf censor --capture <file>`), a `<id>/` folder an unfinished one.

### Cactbot

[OverlayPlugin/cactbot](https://github.com/OverlayPlugin/cactbot) (Apache-2.0), `ui/raidboss/data/`, by expansion
(`02-arr` is A Realm Reborn). Each content has:

* a **timeline** (`.txt`): the boss's actions by ID, with the time each lands, from many logged runs. `jump` marks
  where the fight loops, `window` how far off a real run may be, and `--sync--` lines tie a phase's start to an
  event: combat starting, a cast, or a **battle talk line** (BattleTalk2, by `instanceContentTextId`). Auto-attacks
  are left out (the `-ii` line at the top lists the ignored IDs);
* **triggers** (`.ts`): which cast, status or map effect means which mechanic, by ID.

Read the files from git (`git clone --filter=blob:none --sparse`, then `git sparse-checkout set
ui/raidboss/data/02-arr`); the workspace can reach GitHub.

**What it shows about directors.** The ARR trials' timelines are a loop of boss actions per phase, each phase starting
on a battle talk line the director sends at a health threshold (in Ifrit's, at 70% and 50%), with adds at fixed points
in a phase. That's the same shape as the battle framework's `onHealth` phases and the rotation engine's loops
([Combat](combat.md), [Quest battles](quest-battles.md)), so a Cactbot timeline converts to a phase list and a
rotation per phase.

**Coverage for ARR** (checked 2026-10-17, cactbot `e79ea7f`): 8 trials, 12 Binding Coil turns, 23 dungeon files and
The World of Darkness. Only trials and raids have timelines; dungeons have triggers alone.

| Kind | Covered |
|---|---|
| Trials (timelines) | Ifrit (normal), Titan (normal, hard, extreme), Leviathan (extreme), Shiva (hard, extreme), Ultima (extreme) |
| Raids | The Binding Coil turns 1, 2 and 4–13 (timelines from turn 4) |
| Dungeons, normal (triggers) | Brayflox's Longstop, Haukke Manor, The Sunken Temple of Qarn, Cutter's Cry, The Stone Vigil, Aurum Vale, Amdapor Keep, Pharos Sirius, The Lost City of Amdapor, Snowcloak, Hullbreaker Isle |
| Dungeons, hard (triggers) | Sastasha, The Tam-Tara Deepcroft, Haukke Manor, The Sunken Temple of Qarn, The Stone Vigil, The Wanderer's Palace, Amdapor Keep, Halatali |
| Not covered | **Quest battles and solo duties** (none, in any expansion); Sastasha, The Tam-Tara Deepcroft, Copperbell Mines, Halatali, Dzemael Darkhold and The Wanderer's Palace (normal); Garuda, Leviathan (normal, hard), Good King Moggle Mog |

So it says nothing about Chasing Shadows, To Guard a Guardian, Leia's Legacy or Spirithold Broken, nor about the
early Gridania main scenario. It's most useful from Brayflox's Longstop and Ifrit onwards.

### OverlayPlugin

[OverlayPlugin/OverlayPlugin](https://github.com/OverlayPlugin/OverlayPlugin): ACT's overlay host, which reads some
packets itself. `OverlayPlugin.Core/NetworkProcessors/` has their layouts and `resources/opcodes.jsonc` their opcodes
per region. Its `LineBattleTalk2.cs` is the battle talk window: opcode 162, 40 bytes, the same as Kawari's
DirectorPopupMessage, field for field (its `param1` is our icon and its `param2`, "2 = flags", is our speaker kind byte).
That independently confirms the layout in [Quest battles](quest-battles.md#director-script).

### FFLogs

[fflogs.com](https://www.fflogs.com): logs uploaded from ACT, with every action's timing, per fight. Almost all of it is
raids and trials at the level cap. It needs an API key (v2 GraphQL). Worth it for party content that Cactbot doesn't
time.

### Boss Mod

[awgil/ffxiv_bossmod](https://github.com/awgil/ffxiv_bossmod) (BSD-3-Clause): a Dalamud plugin that draws fights'
mechanics. `BossMod.Modules/` has a module per boss, by expansion and kind, written from its own replays of retail:
`OID` (the actors' BNpcBase), `AID` (every action, with a comment saying who uses it on whom, its cast time and
range), a component per mechanic (the shape with its sizes, and whether it's a raidwide, a tankbuster, a stack...),
and `ModuleInfo` with the duty's ContentFinderCondition and the boss's BNpcName. It covers nearly every dungeon and
trial from A Realm Reborn to Dawntrail (284 duties), where cactbot has ARR dungeons as triggers only. It has no
timing (its state machines say the order, read by hand), no damage and no HP.

The fork keeps a numbers-only extract (`tools/questkit/questkit/data/public/bossmod.json`, with cactbot's timelines
beside it) and uses it three ways ([questkit sources](questkit.md#sources)): the audit of its own duties, the draft
of a new duty, and the server's cone angles and ring holes where the game's files and the captures have none
(`resources/data/public_action_areas.json`). Facts only: none of its code is copied. Shapes it marks `TODO: verify`
are its guesses and aren't used by the server. Its job rotations (`BossMod/Autorotation`) were the action lists and
priorities for the bots' rotations ([Party bots](party-bots.md)).

### Sapphire

[SapphireServer/Sapphire](https://github.com/SapphireServer/Sapphire): another server emulator. Its BNpc and spawn data
came from captures, and its scripts (quest battles included) were written against retail. Already used for Chasing
Shadows' layout IDs, health and waves. It's from an older patch, so check it against a recording.

Its `src/common/Network/CommonActorControl.h` names the ActorControl categories (`ActorControlType`) and the client's
triggers (`PacketCommand`); `questkit chronofoil --controls --sapphire <that file>` puts its name beside each category
the captures have and Kawari's enums don't (from an older patch, so a name to start from: 30 of the 155 unnamed
ActorControl categories and 12 of the 31 ClientTriggers have one). The header is read where it's checked out, never
copied here.

### Public captures

There is no public archive of Chronofoil captures (searched 2026-11-10): captures hold their player's character and
everyone near them, so they aren't shared. What's public is the tooling around them (redstrate's
[PacketAnalyzer](https://zff.dev/redstrate/PacketAnalyzer) and
[XIVPacketTools](https://codeberg.org/redstrate/XIVPacketTools), [ffxivmon](https://github.com/SapphireServer/ffxivmon),
[deucalion](https://lib.rs/crates/deucalion)). Bob's own captures stay the only retail packets the fork has; the
public projects above (Boss Mod, cactbot, Sapphire) are what others distilled from theirs.

### FFXIV Teamcraft

[ffxiv-teamcraft/ffxiv-teamcraft](https://github.com/ffxiv-teamcraft/ffxiv-teamcraft),
`libs/data/src/lib/json/monsters.json` (about 9.5 MB): for each BNpcName, its BNpcBase and every place it's been seen,
with **level, HP** and map coordinates, reported by Teamcraft's users from their packets. `gubal-bnpcs-index.json`
links the two IDs. Useful for open-world enemies Bob hasn't recorded. The coordinates are map coordinates, not world
positions.

### Crafting simulators: raphael-rs, Teamcraft's simulator, Craftimizer

[KonaeAkira/raphael-rs](https://github.com/KonaeAkira/raphael-rs) (a Rust solver kept current with the game),
[ffxiv-teamcraft/simulator](https://github.com/ffxiv-teamcraft/simulator) and
[WorkingRobot/Craftimizer](https://github.com/WorkingRobot/Craftimizer) (a Dalamud plugin, `Simulator/`): the crafting
rules as players' tools model them, which [Crafting](crafting.md) follows where they agree. They work in floats
(Kawari uses exact integers) and differ on condition chances and a few action details. Teamcraft's main repository
also reads retail craft packets (`crafting-replay.service.ts`).

### GatherBuddy and GatherBuddy Reborn

[Ottermandias/GatherBuddy](https://github.com/Ottermandias/GatherBuddy) and
[FFXIV-CombatReborn/GatherBuddyReborn](https://github.com/FFXIV-CombatReborn/GatherBuddyReborn) (Apache-2.0): node
timers from the sheets, the gatherer's statuses, the boon and Bountiful Yield formulas, and GatherBuddy Reborn's
crowd-collected `world_locations.json` (world positions of 2,116 points, no layout IDs). Used by
[Gathering](gathering.md). Its `IlvConvertTable.csv` (each gathering item level's base Gathering and Perception, which
retail's node window matched) ships in `resources/data/gatherbuddy/` with its licence notice.

### The game's sheets: XIVAPI v2 and EXDSchema

[XIVAPI v2](https://v2.xivapi.com/docs/welcome/) serves the game's sheets over HTTP, with older versions.
[xivdev/EXDSchema](https://github.com/xivdev/EXDSchema) names the columns (XIVAPI v2 uses it). The server already reads
the same sheets from the game install, so XIVAPI is for a quick look or an older patch. EXDSchema is the reference
whenever a column's meaning is in doubt.

The workspace's shell reaches XIVAPI v2 (`curl "https://v2.xivapi.com/api/sheet/<Sheet>/<row>?fields=A,B.C"`), which is
how a session without the game install looks up a sheet: the Bowl of Embers' ShortCode (`w1fa_re`) and territory, the
Action rows of Ifrit's attacks (cast times, shapes, ranges), and item 3303's EquipRestriction (11, Miqo'te women) in
Bob's glamour bundle, all 2026-10-27. Numbers only go in the code, with a comment saying where they came from.

### FFXIVClientStructs, Machina and FFXIVOpcodes

* [aers/FFXIVClientStructs](https://github.com/aers/FFXIVClientStructs): the client's memory layouts. Kawari Recorder
  and Probe use it.
* [ravahn/machina](https://github.com/ravahn/machina) and
  [karashiiro/FFXIVOpcodes](https://github.com/karashiiro/FFXIVOpcodes): packet structs and opcodes per patch, for
  checking ours (FFXIVOpcodes is updated irregularly; check its date).

### vnavmesh

[awgil/ffxiv_navmesh](https://github.com/awgil/ffxiv_navmesh) is a Dalamud plugin that builds a navmesh from the
zone's collision as the client has it loaded (its `SceneDefinition.cs` and `SceneExtractor.cs`). The repository has no
licence, so it's read for how the game's collision is put together, never copied.

* **Answers**: how a BgPart's analytic collider (a box, cylinder, sphere or plane) is placed: half its bounds' extent
  and their middle, then its own transform, then the BgPart's; a cylinder is as deep as it is wide. How the game's
  material ID and mask are 64 bits (two 32-bit halves). Used for `kawari-navimesh`'s simple shapes (2026-11-02).
* **Limits**: it works on the client's loaded scene, not on the files; Kawari's builder reads the LGB, so the two can
  differ (a layer the client hasn't loaded, a shape's unknown fields).

### Questionable

[PunishXIV/Questionable](https://github.com/PunishXIV/Questionable) (AGPL-3.0, as Kawari is): a Dalamud plugin that
does quests. `QuestPaths/` has a path for 4,375 quests (2026-11-06): each sequence's steps, with the ENpc or EObj
talked to or used, where, the key item, the enemies, and the quest's variables after a step as its authors saw them
on retail (`CompletionQuestVariablesFlags`, comments starting "QW:"). Its files hold the game's text (names), so it's
read from a checkout: the world bots' routes (`server/questing.rs`) and `questkit questionable`.

* **Answers**: which NPCs and objects a quest uses in which sequence, for every quest, recorded or not; the variables
  a step leaves. `questkit questionable <checkout>` checks every quest script against it (each target a constant the
  script uses, each key item, each sequence spoken of in the header); `--hints QUEST` prints a quest's steps, numbers
  only, before scripting it. On 2026-11-13 it checked 305 scripts (303 with their constants' values from fixtures and
  dumps) and flagged three, all accounted for: the openings 39 and 594 talk to their NPCs by layout ID and are
  accepted on arrival, and Color Your World (699) has no sequence 1 on retail any more (Bob's capture afe32479).
* **Limits**: it takes the quickest way, so a script may rightly do more; it doesn't say what a scene plays or what
  the server sends; some steps are out of date (Color Your World's sequence 1, which retail no longer has).

### AutoDuty

[ffxivcode/AutoDuty](https://github.com/ffxivcode/AutoDuty) (unlicensed: its README says default copyright): a Dalamud
plugin that runs duties. `AutoDuty/Paths/(<TerritoryType>) <name>.json` has 254 routes (185 duties): where the player
walked, the EObjs used in order (switches, keys, doors), the key items a door waits for (`ConditionAction`
"ItemCount;<item>"), where each boss was fought and the treasure coffers on the way. Unlicensed, so nothing of it
goes in git; `questkit autoduty <checkout>` reads it and writes outside the repository.

* **Answers**: a duty's objects and keys before it's recorded (`--draft TERRITORY`), and whether the fork's duty
  scripts name every object and key AutoDuty uses (the check, against `dungeons.md`'s table).
* **Limits**: one player's route: their positions, not the enemies'; no timings, no packets.

### AutoHook, with Teamcraft's catch logs and FF14 Fish Tracker

[PunishXIV/AutoHook](https://github.com/PunishXIV/AutoHook) (BSD-3-Clause, © Puni.sh): a Dalamud fishing plugin.
Its `AutoHook/Data/FishData/` gathers FFXIV Teamcraft's catch logs (gubal.ffxivteamcraft.com: for each fishing spot
and bait, what thousands of players caught and after how many seconds each bit), its own fish list (each fish's tug,
hookset, bait, mooch, intuition, hours and weather, least Gathering) and
[FF14 Fish Tracker](https://github.com/icykoneko/ff14-fish-tracker-app)'s (MIT). None of this is in the game's sheets.
`questkit fishing <checkout>` makes the numbers-only `resources/data/fishing/fishing.json` (its README carries the
licences), which the world server reads (`fishing.rs`).

* **Answers**: what bites at a spot on a bait and how often, the bite times, the tug and hookset, the windows.
* **Limits**: catch logs, not the game's tables (rare fish are logged less); nothing about fishing's packets.

### FFTriadBuddy

[MgAl2O4/FFTriadBuddy](https://github.com/MgAl2O4/FFTriadBuddy) (MIT, © 2019 MgAl2O4): a Triple Triad solver.
`sources/gamelogic/` simulates a match (`TriadGameSimulation.cs`, `TriadGameModifier.cs`: captures, Same, Plus, the
combo, Reverse, Fallen Ace, Ascension, Descension, Order, Sudden Death); `assets/data/` has the cards, NPCs' decks and
rules and the tournaments as of 2025-06, which the game's sheets also have (TripleTriadCardResident, TripleTriad).
`servers/world/src/triple_triad.rs` follows its rules, with its notice.

* **Answers**: the order the rules apply in (Same and Plus on the card placed only, the combo with Reverse in force,
  Fallen Ace on the placed card, the count with the unplayed card).
* **Limits**: how retail's NPCs choose a move isn't in it (its agents are solvers for the player); no packets.

### Wikis

* [Console Games Wiki](https://ffxiv.consolegameswiki.com): dungeon and trial guides that describe each boss's
  mechanics in order, and quest pages. Generally more accurate than Fandom for FFXIV.
* [Fandom: enemy actions](https://finalfantasy.fandom.com/wiki/Final_Fantasy_XIV_enemy_actions): every enemy action
  with a known ID, with effect, target and range, shape and size, cast and recast, and **which enemies use it**. That
  last column is the one thing the client's data doesn't have. By enemy name (several variants share a name), only for
  actions with a confirmed ID, and with no timing or conditions.
