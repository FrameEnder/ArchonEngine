# Quest battles and directors

Quest battles are the solo duties inside story quests (Chasing Shadows, Spirithold Broken, Leia's Legacy, To Guard a
Guardian). The fork added them on top of upstream's content directors.

## Entering one

1. The quest offers it. On the client, talking to the right NPC/object during the right sequence plays a scene that
   asks the client to show **YesNoQuestBattle**. The client first sends client trigger **9003** (Kawari's
   `ClientTriggerCommand::OpenUnk1`; `unk1` is the callback slot, `unk2` the command, 10 for this prompt) and waits.
   During a quest event the server answers with actor control **925**, `ServerRequestReply { callback_index, command,
   result: 0 }` (`main.rs`), which the client hands to `ServerRequestCallbackManager`; 0 means "show it". Only then
   does the prompt open. Outside quests, 9003 still gets upstream's retainer list. (Found with Kawari Probe in
   `f4e7ad8`; the first attempt used Sapphire's EventBattleDialog, actor control 924.) The server always allows entry;
   retail presumably checks something first.
2. The scene returns `1` if the player agreed; the quest script calls
   `player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)`.
3. The server creates a **solo instance** of that territory with a **QuestBattle director**, scripted by
   `resources/scripts/content/questbattle/<QuestBattle id>.lua`. The instance gets the server's Lua state (FATE code
   needs it; an empty state crashed the server in `0663bfc`).
4. Its NPCs come from the territory's drop-in (`resources/dropins/fst_f1_planmap.json` layers named
   `QB_<quest script>_001`), with positions, levels and HP from Sapphire or a retail recording.

## Director script

Globals the script sets:

| Global | Meaning |
|---|---|
| `START_POSITION = {x, y, z}`, `START_ROTATION` | Where the player appears. |
| QuestBattle sheet variables | Injected like quest variables (`INIT_POP_01`, `BNPC_NAME_01`, `DEF_ACTION_*`…). Scripts use `X or <number>` so they still work if a name is missing. |

Callbacks:

| Callback | When |
|---|---|
| `onSetup(director)` | The instance was created. |
| `onCommence(director)` | The player is in: spawn NPCs, set the first sequence, start the chatter. |
| `onUpdate(director, health, player_health)` | Every logic tick (500 ms). `health` maps each living NPC's layout ID to its health %, `player_health` the player's. Waves, thresholds and timed steps live here. |
| `onActorDeath(director, layout_id, position)` | An NPC died. Optional (the server skips it when a script, or a FATE's, doesn't define it). |
| `onPlayerDeath(director)` | The player fell. |
| `onSceneFinished(director, scene)` | A player finished one of the director's own scenes (`play_scene`), e.g. a cutscene halfway through. Optional. |

Director methods (`servers/world/src/server/director.rs`), the ones the fork added:

| Method | Does |
|---|---|
| `spawn_bnpc(layout_id)` | (upstream) Spawns a drop-in NPC. |
| `set_sequence(n)` | Sets the director's sequence and sends DirectorVars (its first byte is the sequence). The client's own battle script reacts to it (e.g. `OnSequence001` sets the duty objective). `0xFE` = lost, shows "Duty Failed". |
| `complete_quest_battle(quest_sequence, stay, finish_scene)` | Wins: the quest moves to `quest_sequence`, "Duty Complete" and the victory fanfare (60001) play, then `finish_scene` (default 6, `QUEST_BATTLE_SCENE_FINISH`; retail plays 6 in Leia's Legacy and 11 in To Guard a Guardian) and the players go back. With `stay = true` the players remain (for closing lines) until `leave_quest_battle()`. |
| `leave_quest_battle()` | Sends the players back where they entered. |
| `fail_quest_battle()` | Loses: players go back where they entered, the quest stays on the same step to retry. Retail sets sequence `0xFE` first and sends them out about 6 s later. |
| `set_hate(npc, target, amount)` | Gives NPC `npc` enmity on `target` (another NPC's layout ID, or the player when `nil`). Used to make allies and enemies fight each other. |
| `use_action(npc, action, target)` | Makes an NPC use an action (with its cast bar and area aimed where the target stands). |
| `battle_talk(npc, name_id, text_id, icon, duration_ms)` | The **battle talk window** (DirectorPopupMessage, opcode 162 on 7.57), as retail shows it: `text_id` is the content's text by ID (retail's IDs from a recording; one outside the content's range crashes the client), `name_id` the speaker's BNpcName, `icon` the portrait, 4000 ms unless given. It comes from `npc` if it's there. |
| `play_scene(scene, flags)` | Plays one of the director's scenes for every player (event type GameProgress), e.g. To Guard a Guardian's cutscene, scene 6 with flags `0xF8402EFB`. The script hears `onSceneFinished` when it's over. |
| `despawn_bnpc(layout_id)` | Removes an NPC without it dying (no `onActorDeath`). |
| `warp_players(x, y, z, rotation)` | Moves every player in the instance through a loading screen to a position in it (PrepareZoning to the same territory, then ActorSetPos). |
| `say(npc, key)` | NPC says a line from the quest's text sheet by key, in `/say` under its BNpcName. |
| `say_battle_talk(npc, number, name_id)` | Says the quest's n-th `*BATTLETALK*` line (key order) under BNpcName `name_id` (0 shows `???`), even if the speaker isn't present. The log names the key it used. |
| `yell(npc, npc_yell_id)` | NpcYell row (e.g. 118 "Curses! Not now!", 122/123 the golem's rumbles). |

**The battle talk window** is DirectorPopupMessage (162), found in the 2026-10-16 recording of Leia's Legacy and To Guard a
Guardian. Layout (40 bytes): speaker actor (ObjectTypeId, 8 bytes), director handler, BNpcName, text ID, duration ms
(4000), icon, then one byte each for the speaker's object kind (2, a BattleNpc), style (0) and parameter count (0), a
pad byte and two u32 parameters. Upstream's struct had the kind as `ObjectKind`, which writes a second byte for a
BattleNpc and makes the packet 41 bytes; it's a plain `u8` now. The text IDs are the content's own (101–106 in Leia's
Legacy, 161–181 in To Guard a Guardian), so they're copied from the recording. Chasing Shadows and Spirithold Broken
still use `say`/`say_battle_talk` (in chat), until their text IDs are recorded.

The retail flow around a quest battle, from the same recording: EventStart (type 0x0f) and the opening scene 1;
when it ends, EventStart 0x0d with scene 60000 and the fight starts (Kawari's `onCommence`); on winning scene 60001,
then the finish scene, UpdateQuest, and DirectorVars with sequence `0xFE` and the byte after it `0x80`, then the
player is sent out.

`!directorscene <scene> [flags]` plays any scene of the current director, which is how the victory fanfare scene
(60001, from the client's generic director script) was found.

## The two battles

### Quest battle 11 — Chasing Shadows (`content/questbattle/11.lua`)

* Yda (3802271) and Papalymo (3802272) fight beside you. Yda goes for the boss and stays on it; Papalymo takes the adds
  (oldest first) and casts at 10 yalms. Adds start with some enmity on their ally (`STARTING_ENMITY = 300`) that the
  player can pull away; each new ally order outweighs the last (`ALLY_ENMITY = 10000`).
* Waves arrive as the boss drops to 75 %, 50 % and 25 % (Sapphire's layout IDs).
* Yda: Pugilist rotation (Bootshine, True Strike, Snap Punch, 3 s apart). Papalymo: Thunder (refreshed as it runs out),
  Fire and Blizzard, and **Cure on the player below 50 %** at most every 4 s, with a line at most every 20 s.
* Director sequence as recorded on retail: 1 at the start, 2 at 8.5 s, 3–5 with each wave, 6 on winning. Papalymo
  yells NpcYell 118 as the last wave arrives, "Well done!" once it's cleared.
* Losing: sequence `0xFE`, out after 6 s. Winning moves the quest to sequence 2.
* Not yet: the enemies' own skills (auto-attacks only).

### Quest battle 15 — Spirithold Broken (`content/questbattle/15.lua`)

* The clay golem (layout 3869528, level 10, 844 HP) with auto-attacks, **True Grit** (120° cone, Blind) and
  **Plaincracker** (circle around it).
* Timed from a retail recording (seconds from entering): sequence 1 at 0.5 s, rumble and sequence 2 at 10.6 s, the
  masked mage's lines and sequences 3–8 through the fight, "Duty Complete" the moment the golem falls (sequence 9),
  two more lines, and out at about 143 s (`complete_quest_battle(4, true)`, then `leave_quest_battle()`). Health
  thresholds for lines and skills are Sapphire's (65 %, 60 %, 30 %, 15 %).
* Winning moves the quest to sequence 4; re-entering the Central Shroud plays the aftermath (scene 23).

### Quest battle 14 — Leia's Legacy (`content/questbattle/14.lua`)

* Layer `QB_SubFst055_001`: Janremi (2319770, level 15, 1212 HP), two hedge knights with swords (2319771, 2319773,
  level 14, 190 HP) and one who casts (2319772, level 15, 127 HP); Kuplu Kopo (4097283, level 15, 1116 HP) fights
  beside you. All from the recording (positions, levels, HP, the start position).
* Sequence 1 before the opening cutscene, 2 when the fight starts; Janremi's lines and sequences 3, 4, 5 at 70 %, 40 %
  and 25 % (Kuplu answers 10 s and 3 s later); below 25 % Janremi uses Heartstopper every 6 s. All down: sequence 6,
  finish scene 6, the quest moves to sequence 4.
* As on retail: Janremi goes after the player, the three hedge knights after Kuplu, Kuplu after the casting knight
  first, then the others. Kuplu Cures the player below 50 % and himself below 50 %; the casting knight Cures the
  others below 75 %.
* Every hit and heal is the recording's (`recorded_amounts`): the knights' auto-attack on Kuplu 15.4, the caster's
  Aero 6 and Cure 87, Kuplu's Stone 20.7 and Cure 95, Janremi on the player 17.5 (auto), 29 (True Thrust), 81
  (Heartstopper). On retail Kuplu went from 1116 to about 450 HP before healing himself.
* Timelines: `JanremiBlackheart_152`, `HedgeKnight_153`/`_154`, `KupluKopo_932`; skill rates fitted to the recorded
  use counts.

### Quest battle 12 — To Guard a Guardian (`content/questbattle/12.lua`)

* Layer `QB_ManFst007_001` (35 NPCs). First half: Galfrid (4089300), Miraudont (4089305) and two guards (3854581,
  3854582) with you against Quzal Huatotl (4089289, level 20, 5782 HP) and six Ixali; guards and Ixali with far too
  much health (191400 / 79750) fighting each other in the background. The first six down bring six more (sequence 3);
  Quzal at 33 % (sequence 4, Void Call) and 25 % (sequence 5, a blanga). Which Ixali come first is a guess from where
  they stand.
* Everyone down: sequence 6, everyone else is removed and the director's cutscene plays (scene 6). After it (sequence
  7) a lesser gargoyle and the masked mage; sequence 8 22 s later; the gargoyle at 25 % brings Yda and Papalymo
  (sequence 9, 10 s later 10); the mage at 40 % says one more line. The mage down: sequence 11, finish scene 11, the
  quest moves to sequence 3.
* As on retail: Quzal goes after Galfrid, the lostwings after Miraudont, the slowbeaks after the two guards, the
  sincallers after Galfrid and Miraudont; the blanga, the gargoyle and the mage after the player. Galfrid, Yda and
  Papalymo take the boss; Miraudont and the guards clear the adds first (oldest first) and only then turn on Quzal.
* Healing, by the Recorder's count of each one's Cures: the bow (9) and Miraudont (9) Cure the two guards for about 60
  and Galfrid (3) for about 200, never themselves or the player; the sincallers (12, about 88) heal Quzal; Papalymo
  heals the player once, big (660). The ~100 Cures on the guards and on the player in the recording were the player's
  own (Bob is a conjurer); an earlier version gave them to Miraudont and let everyone heal themselves, so allies and
  Quzal were healed about three times as often as on retail (111 NPC Cures in one run against 35).
* Every hit and heal between them is the recording's (`recorded_amounts`, 23 timelines): e.g. Galfrid's auto-attack
  76 and Quick Nock 94 on Quzal, the slowbeaks' 24 / True Thrust 42 on the guards (who dropped to about 10 % on
  retail before being healed back), Yda's 180 and Papalymo's Blizzard 770 on the mage.
* **After the cutscene** retail sends the player through a loading screen into the second half: PrepareZoning to the
  same territory (warp type 1, character hidden), ActorSetPos 1.5 s later (-299.54, 20.99, 161.21, facing -0.64),
  then ZoneIn when the client has finished zoning. The cutscene's flags leave the client waiting for it, so without
  it the second half played behind an endless loading screen (2026-10-16 bundle). The director's `warp_players(x, y,
  z, rotation)` does it now.
* Every battle talk line is retail's text ID, speaker and portrait, at the recorded times (the header of the script
  has the table).

## Adding a quest battle

1. Record it on retail (Duties tab: director sequence/variable changes, NPC chat, windows, timings; Monsters tab for
   the enemies' HP and skills).
2. Dump the battle's client script: `kawari-questdump extract <Name>Btl.luab` finds it by name in any game_script
   folder.
3. Add the NPCs to the territory drop-in as a `QB_…` layer.
4. Write `content/questbattle/<id>.lua` with the callbacks above; keep a timing table in its header.
5. Add any new enemy skills as action scripts (`resources/scripts/actions/`), see [Combat](combat.md).
