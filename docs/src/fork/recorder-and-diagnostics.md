# Recorder and diagnostics

Two kinds of evidence drive the fork: **recordings of retail** (what should happen) and **diagnostics bundles from
Bob's server** (what does happen). Both live in the private `FrameEnder/kawari-diagnostics` repository, which also holds
the Dalamud plugins' source. GitHub Actions builds each plugin on every change (Actions → the plugin's workflow → the
run → Artifacts).

## Kawari Recorder (retail, `recorder/`, version 2.7)

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
| Battles (2.4) | Inside duties and quest battles, **each battle in a file of its own**, kept until deleted (2.5; see below): everyone present (layout ID, BNpcBase/Name, battalion, level, max HP); every 250 ms what changed about each of them (position, rotation, HP, MP, **who they're targeting**, what they're casting and at whom, statuses with their source, in combat, weapon drawn, dead, targetable); every action with its results already decoded (source, target, amounts, the target's HP at the time); every ActorControl; casts, map effects and cutscenes (EventPlay) from the game's handlers; the director's sequence, variables and battle talk; marks; and the client's **layout layers** the NPCs came from, every object in them with all its settings | Quest battles and duties: `questkit battle` turns it into waves, targeting rules, heal triggers |
| Quest packets | Server packets addressed to a quest or director: opcode, payload (up to 1 KB inside duties since 2.3; 48 bytes before), quest steps, game version | Notices, EventNotice opcodes, director packets |
| Duty packets | Every server packet inside duties, raw (`content_packets.json`). Since 2.4 with the actor each is about, the packets from zoning in, and NPC/object spawns | Content logic not addressed to a director |

### Saved battles (2.5)

Every duty or quest battle is saved to `battles/<start>-<territory>-<content>.json.gz` in the plugin's folder
(gzipped JSON, one battle each; a 10-minute fight is a few hundred KB), written every 30 s while it runs and once more
when it ends. Nothing is thrown away: the folder keeps every battle until it's deleted from the Battles tab (Ctrl+click)
or the Overview's Clear. `battles/index.json` lists them with a **label** of your own ("stood still", "boss first"),
editable in the tab. **Export (zip)** takes the battles not exported before (or all of them, with the tab's checkbox)
plus the index; a battle's own **Export** button copies just its file to the export folder. Files dropped into the
folder by hand are picked up. 2.4's `battles.json` (the last 4 battles in one file) is split into files on first load.

### What each battle file holds (2.6)

On top of everyone's samples, actions, actor controls, events and layout layers (2.4):

| Field | What | For |
|---|---|---|
| `Hooks` | Each of the Recorder's hooks: working and how much it had seen, or `FAILED: <why>` | Knowing what a recording can't have. The Battles tab shows the same live ("Hooks: all 10 working"); a hook whose signature a patch broke shows there, not only in the Dalamud log |
| `Setup` | Your job, level, level sync, item level (approximate, the character window's way), gear (item, item level, HQ), statuses at the start (the Echo), party (job and level, no names), the Duty Finder settings (unrestricted, minimum item level, level sync, explorer mode) | Comparing numbers between runs |
| `Packets` | Every server packet during the battle, raw (as the Duty packets tab keeps them), from zoning in | One file has everything, even after the Duty packets tab has rolled over |
| `Enmity` | Your enmity list when it changes (at most once a second): your enmity % with each enemy and the enemies on you in order | How aggro builds and moves |
| Samples' `Tethers` | Tethers from each character: Channeling row and target | Tether mechanics |
| Samples' `Visible`, `Targetable`, `State` on event objects | Barriers, crystals, doors appearing, becoming targetable, changing state (also as `object` events) | Arena gimmicks the director drives |
| Samples' `P` | Positions every frame, written each half yalm (or a tenth of a second after a smaller move); `fast move` events for knockbacks and jumps | Exact paths: walking or running, stuck pathing, knockbacks |
| `cast` events | Each cast's target, time, where it's aimed (the packet's position and rotation) and omen delay | AoE placement |
| `ActionInfo` | The client's Action sheet for every action seen: name, cast type, ranges, X-axis modifier, omen, cast and recast, and (2.7) the class mechanics: `ClassJob`, `ClassJobLevel`, `ActionCombo`, `CooldownGroup` (58 = GCD), `StatusGainSelf`, `ActionProcStatus`, `MaxCharges` | Shapes and sizes of AoEs; names in the report; class rotations |
| `ClassKits` (2.7) | For every class or job whose actions an NPC used: its whole kit from the client's sheets (every non-PvP action with its level, GCD group, recast, cast, combo predecessor, self-buff, proc status, charges, aspect...; its base class's too for a job) and its traits by level | Reading an NPC's rotation against its class; what it could have used at its level |
| `auto mark` events | You or a party member dying or getting back up, a wipe, bosses (three times your max HP or more) going below 75/50/25/10 % | Signposts without typing |
| `Field` | Set for fights out in the world (the tab's *Also record fights out in the world*: from entering combat until 15 s after) | Open-world aggro, leash and AI per enemy |

The Battles tab also has **Compare** (a battle against the previous run of the same one: each NPC's death time,
action count and whom it went after, in order) and **Send** / *Send each battle when it ends* (to Kawari Web, below).

### Sending battles to Kawari Web (2.6, Kawari Web 2.14)

In the Battles tab under *Send to Kawari Web*: the site's address and the account's character-sync token (as Kawari
Probe's), then *Test*. Each battle goes to `POST /api/diag/battle` as its `.json.gz`; Kawari Web keeps it in
`<web>/diag/battles` and files it in kawari-diagnostics under **`battles/`** at once, with `battles/index.json`
(every battle with its label, job, counts and failed hooks) and `battles/latest.txt`. So retail battles reach the
diagnostics repository without an export: `python3 tools/questkit battle ../kawari-diagnostics/battles --list`.

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
