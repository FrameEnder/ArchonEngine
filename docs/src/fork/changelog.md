# Changelog

Every change to the fork and its companion projects, newest first. **Add an entry with every change**, in the same
commit, under today's date. A commit can't contain its own hash, so write `—` for it; the next change fills it in.

## Documentation

| Date | Commit | Change |
|---|---|---|
| 2026-10-16 | — | Portraits: the window's disconnect was a server panic (Grand Company rank).
| 2026-10-16 | `048fde1` | Combat: recorded amounts and reading NPC-on-NPC damage from a duty capture; the two quest battles' targeting and healers; known issues.
| 2026-10-16 | `4416558` | Achievements, Portraits and Recommendations page; quest battles 12 and 14, the battle talk window; quests table (7 more); opcodes, saving, tools, known issues. |
| 2026-10-16 | `d69ba31` | questkit and CI pages; quests, overview, Kawari Web, diagnostics and known issues updated for them. |
| 2026-10-16 | `805fdbe` | This documentation section (`docs/src/fork/`), `CLAUDE.md`, and an updated `FORK_ROADMAP.md`. |

## Kawari Web (`FrameEnder/Kawari-Web`)

| Date | Commit | Change |
|---|---|---|
| 2026-10-16 | `ad44611` | 2.13.0: Quest Lab's *Test this quest* (setup, capture on, a bundle tagged with the quest). |
| 2026-10-16 | `67b2f69` | README and CLAUDE.md point to this documentation. |
| 2026-10-16 | `c49ff92` | 2.12.0: the Shop (829 listings, gil and play coins, moogle mail delivery, admin editing). |
| 2026-10-15 | `5b8d9c4` | 2.11.0: quest progress tracker. |
| 2026-10-15 | `b3be94f` | 2.10.0. |

## kawari-diagnostics

| Date | Commit | Change |
|---|---|---|
| 2026-10-16 | `2ef6163` | CLAUDE.md: capture notes aren't written now; findings go in the fork's docs. |
| 2026-10-15 | `df8cec1`, `37d0eb7` | Palace of the Dead notes (recordings and published rules). |
| 2026-10-15 | `1c652c9` | Recorder 2.3: duty packets keep up to 1 KB. |
| 2026-10-15 | `9e37abb` | Blunderville notes. |
| 2026-10-15 | `0967111` | Recorder 2.2: every server packet inside duties. |
| 2026-10-14 | `6b1e4b5` | Recorder 2.1: the server packets that drive quests and duties. |
| 2026-10-14 | `f1cb42f` … `03a9b32` | Kawari Recorder 1.0–2.0 (replaces Bestiary). |
| 2026-10-14 | `f9618ac` … `2a50ef7` | Kawari Bestiary. |
| 2026-10-13 | `469e4fb` … `c1d4d56` | Repository, Kawari Probe 0.1–0.6.5. |

## Kawari (fork)

Commits on `Latest-Quest` since the fork point (`8f65e4b`, upstream v7.56.1). `master` is at `955b0ce`.

### 2026-10-16

| Commit | Change |
|---|---|
| — | Fix the server panic (and the client's 90002 disconnect) when opening the Portraits window as a character in no Grand Company: the adventurer plate read the rank at index `0 - 1`. |
| `048fde1` | Quest battles as on retail: timelines' `recorded_amounts` (what each NPC's action did on retail to players and to other NPCs, damage and heals, read from the duty capture's ActionEffect packets), retail's targeting (hedge knights on Kuplu Kopo; Quzal, the Ixali and the guards on each other) and healers (Cure thresholds fitted to the recording) in Leia's Legacy and To Guard a Guardian; skill rates from the recorded use counts; scripts for the 24 actions those NPCs use that had none (enemy auto-attack 873 among them, so the bows, Galfrid and the hedge knights attack at all). |
| `4416558` | The rest of Gridania's story: Leia's Legacy (161), Dread Is in the Air (446), To Guard a Guardian (447), Festive Endeavors (448), Renewing the Covenant (449), The Gridanian Envoy (507) and Call of the Sea (674), with replay fixtures. Quest battles 14 (Leia's Legacy) and 12 (To Guard a Guardian) from the 2026-10-16 recording: drop-in layers `QB_SubFst055_001` and `QB_ManFst007_001`, 23 timelines. Directors: the battle talk window (`battle_talk`, DirectorPopupMessage, whose kind is now one byte), a finish scene per battle (`complete_quest_battle`'s third argument), `play_scene` with `onSceneFinished`, `despawn_bnpc`; `onActorDeath` is skipped for directors and FATEs that don't define it. Achievements: the window's list (server opcode 991), progress, near-completion replies, completion with the pop-up for quest/level/achievement Types, `!achievement`. Portraits: the adventurer plate's portrait is saved (client opcode 107) and sent back. Migration `update-2026-10-16_achievements_portrait`. |
| `d69ba31` | questkit (`tools/questkit`): bundle triage, quest replay tests with fixtures for all 16 scripted quests, the quest scaffolder, capture/retail comparison and a Lua syntax check; GitHub Actions checks (`.github/workflows/fork.yml`, `tools/ci/check-changes.sh`). Fixes the replay found: Close to Home counts its objectives in UI8AH and sets BitFlag8 1 when attuning, Chasing Shadows clears UI8AL as the battle starts, as retail does. Fixes upstream's `FesHlwTransform_00946.lua`, which didn't compile. |
| `805fdbe` | Document the fork: every feature and mechanic, in docs/src/fork |
| `a0aec12` | Save playtime with autosave; kawari-questdump items for Kawari Web's shop |

### 2026-10-15

| Commit | Change |
|---|---|
| `955b0ce` | Run an action's script tasks straight away, so Teleport warps when its cast ends |
| `2590f47` | Save progress every 30 seconds, and notice connections that dropped silently |
| `9e40e38` | Casts end with their cast bar, can be slidecast; enemy hits interrupt quest searches |
| `f92f557` | Script the Gridania quests up to Salvaging the Scene (379, 380, 381, 384, 387) |
| `5f27926` | Script Don't Look Down (quest 377) |
| `d28f9cf` | Script You Shall Not Trespass (quest 376), with the quest progress pop-up |
| `b073a3c` | Unlock Teleport once a second aetheryte is attuned |
| `ba3db92` | Merge opcode-scan (upstream 7.57 and the fork opcode scanner) into Latest-Quest |
| `8791c78` | questdump index: each quest's expansion, journal section and main scenario flag |
| `c320e5e` | questdump index: each quest's expansion, journal section and main scenario flag |
| `93bba11` | Read the fork's opcodes from the 7.57 executable: EventReturnHandler16 is 200 |
| `bc46eb3` | Read the fork's opcodes out of the game's executable at startup |
| `c08aeef` | Keep the fork's own opcodes in fork_opcodes.yml, with the version each was found on |

### 2026-10-14

| Commit | Change |
|---|---|
| `8e1a63e` | Merge upstream Kawari from Codeberg (through b1e7e2b, patch 7.57) |
| `74c308e` | Merge upstream (GitHub mirror, final commit before the move to Codeberg) |
| `da5ef28` | On to Bentbranch: accepting already starts at sequence 255 |
| `1b96208` | Script On to Bentbranch (quest 176) |
| `5040198` | Duty Complete as the clay golem falls; level-ups announce only the new levels |
| `6b304e0` | Spirithold Broken: talking to the Hearer at sequence 3 offers the quest battle |
| `58aae5e` | Define MAP_DISCOVERY_BITMASK_SIZE in constants.yml |
| `90b9858` | Save map discovery, and stop doubling rested EXP |
| `28f4635` | Script Spirithold Broken (quest 129) and its quest battle (15) |
| `8e46dbc` | Enemy area attacks hit every player inside them |
| `d9b9b68` | Enemy area attacks, aggro, leashing and the level gap |
| `cfb69d8` | Quest kill credit by BNpcBase, and event action casts finish on time |
| `24045b0` | Script A Soldier's Breakfast (quest 3855) |
| `2316f43` | Handle EventReturnHandler16 (client opcode 856), so Pauline takes the Surveying the Damage items |
| `2ab22c6` | Script Surveying the Damage (quest 175) |
| `888489f` | Attuning to aetherytes and aethernet shards plays the attunement animation and progress bar |
| `49c68f0` | Players deal and heal what retail does: main stats from the level's main modifier, retail damage formula |
| `2e21715` | Fill in what the live recording (first quest to Spirithold Broken) showed |
| `9517325` | Stop the world server dropping players whose message queue fills up |
| `acb93c2` | Enemies hit players for what they do on retail, with crits, and drains heal |
| `9bfdac6` | Central Shroud enemies from Kawari Bestiary's recording of retail |
| `731bf0f` | Give ordinary enemies Sapphire's per-level health |
| `92d04bc` | Scenario Guide: skip the other variants of a step once one is done |
| `bdbca42` | Scenario Guide: read ScenarioTree in a language it has, and log why quests are skipped |
| `c156537` | Log what the Scenario Guide is sent |
| `daf8d4a` | Fix striking dummies, sheathing during auto-attack, and the Scenario Guide |
| `95464be` | Give EXP and drops for defeated enemies |

### 2026-10-13

| Commit | Change |
|---|---|
| `7cac6c4` | Implement Eggs over Queasy, and let quests count defeated enemies |
| `38d4659` | Chasing Shadows: Yda and Papalymo's battle chatter |
| `9367259` | Chasing Shadows: Papalymo heals the player, and Yda's combo is a little slower |
| `a06e5ed` | Chasing Shadows: set the director's sequence, so losing can show "Duty Failed" |
| `a4a847e` | Add !directorscene, to look for a content director's generic scenes |
| `3e91c63` | questdump: extract game_script files by name alone |
| `6d62594` | questdump: skip game paths that don't exist when extracting |
| `0fd5256` | TEMPORARY: wait 60 seconds after falling in Chasing Shadows, to test the director's failure update |
| `2cb34d6` | Scale NPC damage against players down to retail-like amounts |
| `f5eb759` | Give Yda a Pugilist rotation and Papalymo a Thaumaturge one |
| `ff7926e` | Chasing Shadows: Yda takes the boss, Papalymo casts at the adds, and NPCs keep their stance |
| `8275c99` | Give NPCs enmity, a per-NPC attack timer, and stop them jumping when they change target |
| `71e37c9` | Fail Chasing Shadows when the player falls, and keep NPCs in their battle stance |
| `3414427` | Let Yda and Papalymo fight in Chasing Shadows |
| `f4e7ad8` | Answer the quest battle prompt's request so it opens by itself |
| `20538c9` | Let directors watch NPC health, and bring in Chasing Shadows' enemy waves |
| `e2e19a2` | Note what's known about the quest battle prompt request |
| `0663bfc` | Fix the world server crashing when a quest battle starts |
| `4895fa6` | Add !rawsweep debug command for finding unknown server opcodes |
| `6c9f669` | Answer the client's request to open the quest battle prompt |
| `5be9762` | End a quest's event action cast before continuing the event |
| `6506b09` | Implement Chasing Shadows |
| `11dc8b7` | Add quest battles and quest event actions |
| `76f6d79` | Put gear replaced from the main inventory into the Armoury Chest, like the client does |
| `3eb437d` | Give To the Bannock's reward the player picks |
| `ad4279e` | Implement Passing Muster, and give the optional quest reward the player picks |
| `905beaa` | Implement To the Bannock |
| `f5dc99f` | Add !rawpacket debug command for testing packet layouts |
| `3f2d23e` | Put quest variables where the client reads them |
| `7c139e3` | Implement Close to Home (Conjurer) |
| `9184958` | Don't remove instances while their navmesh is being generated |
| `3fb25fe` | Roll back the quest work to the White Mage state |
| `ff8090e` | Note packet capture and the diagnostics pipeline in the roadmap |
| `ed897d8` | Add optional packet capture to the world server |

### 2026-10-12

| Commit | Change |
|---|---|
| `bcf882f` | *(rolled back in `3fb25fe`)* Log why zone clients get dropped, add !questvar for testing |
| `02e22b5` | *(rolled back in `3fb25fe`)* Finish enter-territory events for scripts that don't handle them |
| `4070c7c` | *(rolled back in `3fb25fe`)* Only save the zone and quests on zone change |
| `4ed1233` | *(rolled back in `3fb25fe`)* Fix quest variables never reaching the client |
| `a8a7bf3` | *(rolled back in `3fb25fe`)* Log quest progress changes and implement //gm quest inspect |
| `a30ad51` | *(rolled back in `3fb25fe`)* Move players out of the opening area at login instead of mid-load |
| `d543ce9` | *(rolled back in `3fb25fe`)* Send players past Close to Home out of the Gridania opening area |
| `7486d2f` | *(rolled back in `3fb25fe`)* Stop the opening's city boundaries once the opening quest is done |
| `f29eede` | *(rolled back in `3fb25fe`)* Fix Gridania quests against the client's quest scripts, add the last four |
| `4556bf1` | *(kept by the rollback)* Add a quest index to kawari-questdump |
| `effe700` | *(kept by the rollback)* Add kawari-questdump for pulling quest data and client scripts from the game |

### 2026-10-11

| Commit | Change |
|---|---|
| `b27a8ce` | *(rolled back in `3fb25fe`)* Script 16 Gridania story quests after Close to Home |
| `164fa73` | *(rolled back in `3fb25fe`)* Add quest progression and finish Close to Home |
| `e164184` | Merge White Mage kit |
| `e36fa77` | Add a roadmap for the fork |
| `dc63634` | Add a full White Mage kit and the healer engine support it needs |
