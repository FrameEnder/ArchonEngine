# Recorder and diagnostics

Two kinds of evidence drive the fork: **recordings of retail** (what should happen) and **diagnostics bundles from
Bob's server** (what does happen). Both live in the private `FrameEnder/kawari-diagnostics` repository, which also holds
the Dalamud plugins' source. GitHub Actions builds each plugin on every change (Actions → the plugin's workflow → the
run → Artifacts).

## Kawari Recorder (retail, `recorder/`, version 2.4)

A Dalamud plugin used on the **live game**. It replaced Kawari Bestiary (and imports its data on first run). `/krec`
opens it; **Export (zip)** or `/krec export` writes `kawari-recorder-<date>.zip`; `/krec pause|resume`; `/krec mark
<note>` notes a moment inside a duty (put it on a macro). Nothing is sent anywhere; it saves every 30 s.

| Tab | Records | Used for |
|---|---|---|
| Monsters | Per kind per zone: level, HP/MP per level, spawn points by layout ID, hitbox, model, every action with damage to players (your level/job), statuses, misses, knockbacks, targets hit | Enemy HP, timelines, `damage_to_players`, drop-ins |
| You | Your stats per job and level, your actions' damage/healing/crits/statuses, cast/recast times, regen ticks | Player stat and damage formulas |
| Kills | EXP per kill (levels, rested), items | Kill EXP, chains, drop rates |
| Quests | Every quest change: accepted, each sequence/variable change (with position and target up to 30 s before), completion rewards, abandons | Quest scripts (the step-by-step variable bytes) |
| Duties | Directors (handler, Lua class), sequence/variable changes, objectives, time limit, NPC chat, windows, log messages, timed | Quest battles, duties |
| Messages | Log messages by ID with params, Talk window dialogue, windows opened | Text/log behaviour |
| Statuses | Durations, params, source, actual length | Status scripts |
| World | Event NPCs/objects (layout ID, position, quest markers), weather by Eorzea hour, aggro range, leash distance, wander, speeds, respawns | Aggro/leash tuning (still global constants), spawns |
| FATEs | Level, place, radius, time limit, progress, rewards | FATEs (future) |
| Battles (2.4) | Inside duties and quest battles (`battles.json`, the last 4): everyone present (layout ID, BNpcBase/Name, battalion, level, max HP); every 250 ms what changed about each of them (position, rotation, HP, MP, **who they're targeting**, what they're casting and at whom, statuses with their source, in combat, weapon drawn, dead, targetable); every action with its results already decoded (source, target, amounts, the target's HP at the time); every ActorControl; casts, map effects and cutscenes (EventPlay) from the game's handlers; the director's sequence, variables and battle talk; marks; and the client's **layout layers** the NPCs came from, every object in them with all its settings | Quest battles and duties: `questkit battle` turns it into waves, targeting rules, heal triggers |
| Quest packets | Server packets addressed to a quest or director: opcode, payload (up to 1 KB inside duties since 2.3; 48 bytes before), quest steps, game version | Notices, EventNotice opcodes, director packets |
| Duty packets | Every server packet inside duties, raw (`content_packets.json`). Since 2.4 with the actor each is about, the packets from zoning in, and NPC/object spawns | Content logic not addressed to a director |

### What a recording can and can't tell

The scripts that run a duty's director and its NPCs (when waves come, HP thresholds, targeting, who heals whom) are on
Square Enix's servers and never reach the client, so no plugin can read them; they're inferred from what everyone did.
What the client does have, and the Battles tab now keeps:

* **The layout** (`bg/<zone>/level/planmap.lgb` and friends): every NPC's layer and instance name (groups such as
  `INIT_NOTAR_POP_<n>ST_..` in To Guard a Guardian), and the settings the server's scripts read: `NormalAI`, `MoveAi`,
  `ServerPathId` (patrol routes, the ServerPath objects in the same layer), `LinkGroup`/`LinkFamily`/`LinkRange`,
  `ActiveType`, `PopEvent`, `Nonpop`, `BoundInstanceID`, `SenseRangeRate`. Read with Lumina, by reflection, so a newer
  Lumina with more fields records more.
* **Who targets whom, every moment** (each character's target, from the client's object table), which makes a rule like
  "the guards clear the adds before the boss" something the recording shows rather than a guess.
* **The client's battle script** (`<Name>Btl.luab`, via `kawari-questdump extract`): scenes, battle talk, and the
  director variables it reacts to.

Inference gets firmer with more runs: record a battle two or three times played differently (stand still once, go
straight for the boss once, let an ally die once); what changes with how you play is a reaction, what doesn't is
scripted. Mark the moments you notice (`/krec mark he summoned an add`).

Up to 2.3 the duty packets had no actor (every packet read actor 0 and size 0: the module read the actor from memory
before the packet, which isn't its segment header) and no spawns (NPC spawns don't pass through the dispatcher hook),
so which NPC did what had to be worked out from HP totals and hit counts (the 2026-10-16 To Guard a Guardian recording).

## Kawari Probe (on Kawari, `probe/`)

A Dalamud plugin used on **Bob's server**: it reports what the client itself believes. Setup: add the DLL as a dev
plugin, `/kprobe url http://<server>:7301`, `/kprobe token <character-sync token>`, `/kprobe test`. `/kprobe <note>`
sends a snapshot; `/kprobe zone on` sends one after every zone change. Snapshots go to Kawari Web
(`POST /api/diag/snapshot`) and into the next bundle. A snapshot's `quests` list has each quest's `raw_hex` (the
client's QuestWork in memory: an 8-byte header, then id, sequence, flags, six variables, accepting class/job), key
items, zone, position, conditions. Also used to find client internals (e.g. where ServerRequestCallbackManager takes
replies, actor control 925).

## Kawari Bestiary (`bestiary/`)

The Recorder's predecessor, monsters only. Its Central Shroud recording set the enemy HP, auto-attack rate and damage.

## Diagnostics bundles

Kawari Web → Dashboard → Diagnostics → *Send diagnostics* commits `bundles/<time>-<note>/`; `latest.txt` names the
newest.

| File | Contents | How to read it |
|---|---|---|
| `README.md` | Summary with the characters' active quests and variables | Start here |
| `meta.json` | Build running, the note (what Bob was doing) | |
| `server.log` | Container log, colour codes stripped | Errors, "Variables available", quest changes |
| `characters.json` | Saved quest rows and position from world.db (`quest_summary`) | The server's view |
| `snapshots/*.json` | Kawari Probe snapshots (`quests[].raw_hex`) | The client's view; compare with `characters.json` |
| `captures/*.jsonl.gz` | Packet captures (gunzip; see [Tools](tools.md#packet-capture)) | Timings, exact bytes sent |
| `config.json` | Server config, secrets removed | |

Reading order: `python3 tools/questkit triage <kawari-diagnostics>` first (it does the next steps in one report);
by hand: `latest.txt` → the bundle's `README.md` and `meta.json` → compare server and client quest state → the
capture around the time of the note → the log. A bundle sent by Quest Lab's *Test this quest* has `quest_test` in
`meta.json` (quest, name, character, content_id, where it started, when), and its note starts with "Quest test".

`questkit compare --capture <bundle> --recording <export>` checks the bundle's captures against a retail recording.

## Research notes (`notes/`)

Analyses of recordings for content not implemented yet; see [Research notes](research.md).

No game files (dumps, `.luab`, sheets, the executable) ever go in this repository either.
