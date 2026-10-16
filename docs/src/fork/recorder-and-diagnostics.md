# Recorder and diagnostics

Two kinds of evidence drive the fork: **recordings of retail** (what should happen) and **diagnostics bundles from
Bob's server** (what does happen). Both live in the private `FrameEnder/kawari-diagnostics` repository, which also holds
the Dalamud plugins' source. GitHub Actions builds each plugin on every change (Actions → the plugin's workflow → the
run → Artifacts).

## Kawari Recorder (retail, `recorder/`, version 2.3)

A Dalamud plugin used on the **live game**. It replaced Kawari Bestiary (and imports its data on first run). `/krec`
opens it; **Export (zip)** or `/krec export` writes `kawari-recorder-<date>.zip`; `/krec pause|resume`. Nothing is sent
anywhere; it saves every 30 s.

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
| Quest packets | Server packets addressed to a quest or director: opcode, payload (up to 1 KB inside duties since 2.3; 48 bytes before), quest steps, game version | Notices, EventNotice opcodes, director packets |

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

Reading order: `latest.txt` → the bundle's `README.md` and `meta.json` → compare server and client quest state →
the capture around the time of the note → the log.

## Research notes (`notes/`)

Analyses of recordings for content not implemented yet; see [Research notes](research.md).

No game files (dumps, `.luab`, sheets, the executable) ever go in this repository either.
