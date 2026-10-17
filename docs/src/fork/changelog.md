# Changelog

Every change to the fork and its companion projects, newest first. **Add an entry with every change**, in the same
commit, under today's date. A commit can't contain its own hash, so write `—` for it; the next change fills it in.

## Documentation

| Date | Commit | Change |
|---|---|---|
| 2026-10-17 | — | Leaving a zone with FATEs (travel); FATE packet order (known issues).
| 2026-10-17 | `32ba317` | Area attacks' edge, NPC moves, no wandering in content, straight paths when none (combat); To Guard a Guardian's sincallers and start facing, the quest battles' start spots (quest battles); the blanga's Terror Eye (known issues).
| 2026-10-17 | `8dd3b56` | Mapping the Realm, the whole-map reveal and shared discovery slots, the 31 unlocks in Bob's June–September sessions (character windows); To Guard a Guardian's sincallers, gargoyle, mage and scenery (quest battles); display flags and the gargoyle's wait (known issues).
| 2026-10-17 | `661aa3e` | Achievements: the list Types start in Key, Mapping the Realm on the first discovery, unique quests counted, and when retail completes them (character windows); ActorControl 518 (known issues).
| 2026-10-17 | `db6faaa` | To Guard a Guardian checked against Bob's Chronofoil capture (quest battles); `questkit chronofoil --battle` and who's in the fight (questkit); what the Recorder's raw packets can't be trusted for (recorder and diagnostics); packet layouts from retail captures (known issues).
| 2026-10-17 | `95ce15b` | Bob's Chronofoil captures and what's in them (outside sources, recorder and diagnostics); `questkit chronofoil` (questkit); packet sizes that differ on retail (known issues).
| 2026-10-17 | `b435ef4` | Outside Sources (new page): what to check when a recording leaves something unclear, the order of trust, Cactbot's ARR coverage; OverlayPlugin confirms the battle talk window's layout (quest battles).
| 2026-10-17 | `82f65de` | Placement, battle roles vs class roles (quest battles, combat, questkit).
| 2026-10-17 | `21d1075` | Roles (quest battles, questkit).
| 2026-10-17 | `c603df3` | The battle framework, `set_attack_range`, `keep_near` and `onUpdate`'s `info` (quest battles); staying near someone (combat); the outline and telegraphs (questkit); known issues.
| 2026-10-16 | `dbedea9` | Class rotations (combat); Recorder 2.7's class kits (recorder and diagnostics); the class profile (questkit); adding a quest battle; known issues.
| 2026-10-16 | `29ecfc9` | Recorder 2.6's battle files and sending to Kawari Web (recorder and diagnostics, Kawari Web); the report's new sections (questkit).
| 2026-10-16 | `ee05899` | Recorder 2.5's saved battles (recorder and diagnostics); `questkit battle` across runs (questkit).
| 2026-10-16 | `c2697fd` | Recorder 2.4 and what a recording can and can't tell (recorder and diagnostics); `questkit battle` (questkit, quest battles).
| 2026-10-16 | `9ef685a` | Combat: partial paths, no leashing in content; To Guard a Guardian's background fights; known issues.
| 2026-10-16 | `fe36e6c` | Quest battles: `warp_players`, To Guard a Guardian's second half, targeting and healing; known issues.
| 2026-10-16 | `6a4203e` | Achievements must be quick (character windows); connections desync when busy (known issues).
| 2026-10-16 | `de2d795` | Items That Teach Things (new page); achievements (Types 8, 20, counters); saving; known issues.
| 2026-10-16 | `53316cd` | Combat: leashing on the target's distance, the InCombat condition; the Adventurer Plate (character windows, opcodes, saving); known issues (teleport animation, the plate).
| 2026-10-16 | `feef465` | Quest unlock links (quests); how the client gates main menu commands, Recommendations and the Adventurer Plate (character windows, known issues).
| 2026-10-16 | `177bebd` | Debug commands only work in zone chat (tools, known issues).
| 2026-10-16 | `6d54f81` | Portraits: the window's disconnect was a server panic (Grand Company rank).
| 2026-10-16 | `048fde1` | Combat: recorded amounts and reading NPC-on-NPC damage from a duty capture; the two quest battles' targeting and healers; known issues.
| 2026-10-16 | `4416558` | Achievements, Portraits and Recommendations page; quest battles 12 and 14, the battle talk window; quests table (7 more); opcodes, saving, tools, known issues. |
| 2026-10-16 | `d69ba31` | questkit and CI pages; quests, overview, Kawari Web, diagnostics and known issues updated for them. |
| 2026-10-16 | `805fdbe` | This documentation section (`docs/src/fork/`), `CLAUDE.md`, and an updated `FORK_ROADMAP.md`. |

## Kawari Web (`FrameEnder/Kawari-Web`)

| Date | Commit | Change |
|---|---|---|
| 2026-10-16 | `7133931` | 2.14.0: battles from Kawari Recorder (`POST /api/diag/battle`), kept and filed in the diagnostics repository's `battles/`; the Diagnostics page's Battles card; bundles name the battles from their window. |
| 2026-10-16 | `ad44611` | 2.13.0: Quest Lab's *Test this quest* (setup, capture on, a bundle tagged with the quest). |
| 2026-10-16 | `67b2f69` | README and CLAUDE.md point to this documentation. |
| 2026-10-16 | `c49ff92` | 2.12.0: the Shop (829 listings, gil and play coins, moogle mail delivery, admin editing). |
| 2026-10-15 | `5b8d9c4` | 2.11.0: quest progress tracker. |
| 2026-10-15 | `b3be94f` | 2.10.0. |

## kawari-diagnostics

| Date | Commit | Change |
|---|---|---|
| 2026-10-16 | `b699bca`, `b1d579e` | Recorder 2.7: the class mechanics in every action's info (combo predecessor, GCD group, level, self-buff, proc, charges) and each class's whole kit (actions and traits) in the battle. |
| 2026-10-16 | `25015ae`–`97e5538` | Recorder 2.6: hook health (in the tab and in every battle), your setup, each battle's own raw packets, enmity, tethers, event object states, positions every frame and fast moves, cast placement, the Action sheet's info per action, auto marks, Compare with the last run, field fights, and sending battles to Kawari Web. |
| 2026-10-16 | `b161d2b`, `b24206d` | Recorder 2.5: every battle saved to a file of its own (`battles/<start>-<territory>-<content>.json.gz`), kept until deleted, with labels, per-battle export and delete; exports take the battles not exported yet (or all); 2.4's `battles.json` is split into files. |
| 2026-10-16 | `ccedc94`–`efafdf2` | Recorder 2.4: the Battles tab (everyone's position, HP, target, casts and statuses every 250 ms, decoded actions, actor controls, casts, map effects, cutscenes, director events, marks, and the layout layers the NPCs came from with their AI settings); duty packets get their actor, the packets from zoning in and NPC/object spawns (up to 2.3 every packet read actor 0 and spawns weren't kept); `/krec mark`. |
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

### 2026-10-17

| Commit | Change |
|---|---|
| — | Leaving a zone takes its FATEs down as retail does (2352 for each start NPC, the new `RemoveFateContext`, 2357, for every FATE, then the FATE director): only terminating the director crashed the client in its FATE update when teleporting from the Central Shroud to Gridania. |
| `32ba317` | Enemy area attacks count half a yalm past their edge, not a whole yalm (players who had left Terror Eye were still hit). NPCs are only sent an ActorMove when they move or turn (idle NPCs flickered between walking and standing, and flooded the open world); NPCs in content don't wander, and walk straight at a target they can't path to instead of dropping it (To Guard a Guardian's mage walking off, the second wave looping around the tree). To Guard a Guardian: sincallers kept back by range alone (they stood inside each other); all four quest battles start the player at retail's spot and facing (To Guard a Guardian's was nearly backwards). Test in `action.rs`. |
| `8dd3b56` | A Realm Reborn's field zones reveal their whole map on the first visit (LocationDiscovered part 164, as retail), which completes their Mapping the Realm; the rest need every part again, counted over every map sharing the discovery slot (Midas II and the like completed after one floor otherwise). Checked against the 31 unlocks in Bob's June–September sessions. To Guard a Guardian: the sincallers stay back behind Quzal, the gargoyle waits until you've loaded in, the mage also comes when the gargoyle falls, and the background fighters have retail's display flags (no health bar, not targetable; drop-in battle NPCs take `display_flags`). |
| `661aa3e` | Achievements read the sheet as retail's rows are laid out: Types 2, 6 and 9 list Key and then Data, Type 3 is class Key at level Data[0] (Types 3 and 6 never completed before); Mapping the Realm completes with the map's first discovered part, as on retail; Storyteller's counter (Key 214, unique quests) is counted from the completed quests. Checked against four unlocks in Bob's Chronofoil captures; tests in `achievement.rs`. |
| `db6faaa` | To Guard a Guardian from Bob's Chronofoil capture of the same run: sequence 4 at 34 %, Void Call at 27 % with sequence 5 and the blanga when it lands, the second wave's slowbeaks swapped, both sincallers start on Galfrid, the masked mage passive until hit (not hostile in the drop-in; Battle.lua's `passive`), Papalymo on the adds first and his Cure below 70 %. `questkit chronofoil --battle N.V` turns a capture's duty into a battle run (`chronofoil_battle.py`); `questkit battle` only counts NPCs in your fight and picks an engaged boss. |
| `95ce15b` | `questkit chronofoil`: reads Chronofoil captures (finished and unfinished), indexes each session's zones and duties with what happened there, and exports a visit as a Kawari packet capture; opcode tables for game versions 2026.06.18, 09.01 and 09.15 (`tools/questkit/questkit/data/chronofoil_opcodes.json`, derived in `chronofoil_opcodes.py`), checked against every packet's size in Bob's captures. CI installs zstandard. |
| `b435ef4` | Chasing Shadows' comment on battle talk brought up to date (the window is known; its text IDs aren't recorded yet). |
| `82f65de` | Placement: `keep_near` takes several anchors (Papalymo near Yda and you), `stand_behind` puts an NPC behind the enemy the tank is fighting (`server/placement.rs`); supports default to it. Roles: a battle role (`role`) apart from the class's (`class_role`): in Chasing Shadows Yda, a DPS, plays the tank, and Papalymo is a DPS with some support. questkit only calls an NPC a support when healing is a large part of what it does. |
| `21d1075` | Roles in the battle framework: tank (leads, pulls enemies off the others: supports first, then DPS, then the player), DPS (highest health first, two or more split the toughest enemies, can heal themselves), support (backline, near the tank, buffs DPS then tank, heals the tank first); enemies take roles too. `onUpdate`'s `info` gains each actor's target. `questkit battle`'s outline gives each ally's role, by class and by what it did. |
| `c603df3` | The battle framework (`content/lib/Battle.lua`, loaded before every quest battle's script): allies' roles (focus groups, order, range, staying near someone, heals), enemies' groups and first targets, context changes, timed steps and health/cleared phases. Chasing Shadows and To Guard a Guardian run on it (simulated: the same director calls and targets as before; Papalymo now casts from 10 yalms and stays within 15 of you). The director gets `set_attack_range` and `keep_near` (NPCs head back to their anchor), and `onUpdate` a fourth argument with everyone's HP, max HP and position. `questkit battle` drafts the director's outline per phase (focus, order, range, staying near, heals) and lists enemy telegraphs. |

### 2026-10-16

| Commit | Change |
|---|---|
| `dbedea9` | Class rotations for NPCs: a timeline's `rotation` (pace and priority-ordered actions with `every`, `refresh`, `then`, HP conditions), run every tick by `server/rotation.rs` with the Action sheet's GCD group, recast and combo predecessor (`GameData::get_action_mechanics`). `questkit battle` profiles each NPC by its class (pace, combos, abilities on cooldown, buff follow-ups, re-applied debuffs, unused kit) and writes its `rotation`. |
| `29ecfc9` | `questkit battle` reports Recorder 2.6's additions: failed hooks, your setup, action names, tethers, event objects, fast moves, auto marks, your enmity, and actor controls by category (named from `ActorControlCategory`). |
| `ee05899` | `questkit battle` reads any number of battles (exports, the Recorder's battles folder, single `.json.gz` files, 2.4's `battles.json`; labels from the index) and, for several runs of the same battle, adds up each NPC's evidence across them. |
| `c2697fd` | `questkit battle`: a report on a recorded duty or quest battle (Recorder 2.4's `battles.json`): who's in it with their layout layer and AI settings, waves and what preceded them, each NPC's targets with the explanations that fit every change (counted), actions, intervals and heals, and who never acted. Tested on made-up data. |
| `9ef685a` | To Guard a Guardian could never end: after Quzal fell, the last two Ixali were on a background guard 9 yalms past where the navmesh stops, and every ally stood at the edge (2026-10-16 bundle). NPCs given a partial path now walk straight on to their target (up to 15 yalms out in the world, any distance in content; Detour's success-but-partial status is no longer logged as an error), and enemies in content don't leash (those lostwings had leashed chasing Miraudont the long way round, then picked up the guard). The background fights are the layer's five groups (each sincaller on a guard, the melee Ixali in pairs), as the Recorder shows, so no Ixal or guard stands idle. |
| `fe36e6c` | To Guard a Guardian: the cutscene now ends with retail's loading screen into the second half (director `warp_players`: PrepareZoning to the same territory, then ActorSetPos), which the client waited for; Miraudont and the guards clear the adds before turning on Quzal; healing follows the Recorder's count of each NPC's Cures (the guards healed by the bow, Miraudont and Galfrid, Quzal by the sincallers, the player once by Papalymo), where before everyone also healed themselves and Miraudont was given the player's own Cures. |
| `6a4203e` | Fix players freezing (or reviving into endless loading) after finishing a quest or discovering a map part: the achievement check read the Map sheet for every Mapping the Realm achievement, taking long enough to desync the client's connection (2026-10-16 bundles). The Map sheet's discovery columns are read once now; a check over 50 ms is logged. |
| `de2d795` | Items that teach things work: mounts, unlock links (emotes, hairstyles…), barding, Triple Triad cards, fashion accessories and facewear are learned, as well as minions and orchestrion rolls (which no longer un-learn on a second use), and the item is used up. Achievements: Mapping the Realm (Type 8, the whole map discovered), Freebird (Type 20) and "Defeat N enemies" (counter 11), with counters saved (migration `update-2026-10-16-3_achievement_counters`); checked on discovering a map part, attuning aether currents and defeating an enemy. |
| `53316cd` | The adventurer plate migration is `update-2026-10-16-2_adventurer_plate`: as `update-2026-10-16_adventurer_plate` it had the same version as the achievements one (Diesel's version is the part before `_`), so it wouldn't have run on a database that already had that one, and a new database skipped the achievements migration. |
| `70fa284` | Adventurer plates are saved: the client's UpdateAdventurerPlate (opcode 795) is stored and sent back in AdventurerPlate (CharaCardData, migration `update-2026-10-16-2_adventurer_plate`); the plate shows the class/job's real level, and upstream's misplaced glasses/base plate are fixed. Enemies give up once the player they're after is more than 40 yalms from their spawn (enemies with no navmesh path never moved, so never leashed, and kept players in combat: no regeneration, 2026-10-16 bundle). The server sets the player's InCombat condition while enemies are after them, as retail does, so the client refuses Teleport and Return in combat. |
| `feef465` | Quests grant their unlock links (`REWARD…` parameters) when finished, and at login for quests already finished: Spirithold Broken's unlocks the Recommendations window (unlock link 9). |
| `177bebd` | `!achievement check` says how many achievements it completed (it was silent when there were none). |
| `6d54f81` | Fix the server panic (and the client's 90002 disconnect) when opening the Portraits window as a character in no Grand Company: the adventurer plate read the rank at index `0 - 1`. |
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
