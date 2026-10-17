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
| How does this fight go: phases, waves, who does what | A recording | Cactbot's timeline for the content, a Chronofoil capture |
| Which actions an enemy has | A recording | The Fandom enemy actions page (by name), Cactbot's triggers |
| An action's shape, size, cast time, telegraph | The Action sheet (`CastType`, `EffectRange`, `XAxisModifier`, `Cast100ms`, `Omen`) | The Fandom page, which lists the same columns by action |
| An enemy's level, HP and where it stands | A recording | Teamcraft's `monsters.json`, Sapphire's BNpc data |
| A packet's layout or opcode | A recording, then `kawari-opcodescan` | OverlayPlugin's packet structs, Machina, FFXIVOpcodes, Kawari upstream |
| What a sheet column means | EXDSchema | XIVAPI's documentation |
| A client structure (for the Recorder or Probe) | FFXIVClientStructs | |

## The sources

### Chronofoil

[ProjectChronofoil/Chronofoil.Plugin](https://github.com/ProjectChronofoil/Chronofoil.Plugin) is a Dalamud plugin that
records all game traffic. Its captures can be uploaded (censored of session IDs, chat and letters) to a central
preservation archive ([Chronofoil.Web](https://github.com/ProjectChronofoil/Chronofoil.Web)). Whether and how the
archive can be downloaded hasn't been checked. redstrate's
[PacketAnalyzer](https://zff.dev/redstrate/PacketAnalyzer) (live at analyze.xiv.zone) decodes Chronofoil captures with
the packet definitions Kawari uses.

* **Answers**: everything a recording does, for content Bob hasn't run, and how a fight went on an older patch.
* **Limits**: captures carry other players' characters and game text, so they stay out of every repository (the
  diagnostics one included); only numbers come out of them. A capture from another patch needs that patch's opcodes.
  Recording with Chronofoil next to Kawari Recorder would also give a raw copy of a run.

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

### Sapphire

[SapphireServer/Sapphire](https://github.com/SapphireServer/Sapphire): another server emulator. Its BNpc and spawn data
came from captures, and its scripts (quest battles included) were written against retail. Already used for Chasing
Shadows' layout IDs, health and waves. It's from an older patch, so check it against a recording.

### FFXIV Teamcraft

[ffxiv-teamcraft/ffxiv-teamcraft](https://github.com/ffxiv-teamcraft/ffxiv-teamcraft),
`libs/data/src/lib/json/monsters.json` (about 9.5 MB): for each BNpcName, its BNpcBase and every place it's been seen,
with **level, HP** and map coordinates, reported by Teamcraft's users from their packets. `gubal-bnpcs-index.json`
links the two IDs. Useful for open-world enemies Bob hasn't recorded. The coordinates are map coordinates, not world
positions.

### The game's sheets: XIVAPI v2 and EXDSchema

[XIVAPI v2](https://v2.xivapi.com/docs/welcome/) serves the game's sheets over HTTP, with older versions.
[xivdev/EXDSchema](https://github.com/xivdev/EXDSchema) names the columns (XIVAPI v2 uses it). The server already reads
the same sheets from the game install, so XIVAPI is for a quick look or an older patch. EXDSchema is the reference
whenever a column's meaning is in doubt.

### FFXIVClientStructs, Machina and FFXIVOpcodes

* [aers/FFXIVClientStructs](https://github.com/aers/FFXIVClientStructs): the client's memory layouts. Kawari Recorder
  and Probe use it.
* [ravahn/machina](https://github.com/ravahn/machina) and
  [karashiiro/FFXIVOpcodes](https://github.com/karashiiro/FFXIVOpcodes): packet structs and opcodes per patch, for
  checking ours (FFXIVOpcodes is updated irregularly; check its date).

### Wikis

* [Console Games Wiki](https://ffxiv.consolegameswiki.com): dungeon and trial guides that describe each boss's
  mechanics in order, and quest pages. Generally more accurate than Fandom for FFXIV.
* [Fandom: enemy actions](https://finalfantasy.fandom.com/wiki/Final_Fantasy_XIV_enemy_actions): every enemy action
  with a known ID, with effect, target and range, shape and size, cast and recast, and **which enemies use it**. That
  last column is the one thing the client's data doesn't have. By enemy name (several variants share a name), only for
  actions with a confirmed ID, and with no timing or conditions.
