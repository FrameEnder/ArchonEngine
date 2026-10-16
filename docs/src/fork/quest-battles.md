# Quest battles and directors

Quest battles are the solo duties inside story quests (Chasing Shadows, Spirithold Broken). The fork added them on top
of upstream's content directors.

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
| `onActorDeath(director, layout_id, position)` | An NPC died. |
| `onPlayerDeath(director)` | The player fell. |

Director methods (`servers/world/src/server/director.rs`), the ones the fork added:

| Method | Does |
|---|---|
| `spawn_bnpc(layout_id)` | (upstream) Spawns a drop-in NPC. |
| `set_sequence(n)` | Sets the director's sequence and sends DirectorVars (its first byte is the sequence). The client's own battle script reacts to it (e.g. `OnSequence001` sets the duty objective). `0xFE` = lost, shows "Duty Failed". |
| `complete_quest_battle(quest_sequence, stay)` | Wins: the quest moves to `quest_sequence`, "Duty Complete" and the victory fanfare play. With `stay = true` the players remain (for closing lines) until `leave_quest_battle()`. |
| `leave_quest_battle()` | Sends the players back where they entered. |
| `fail_quest_battle()` | Loses: players go back where they entered, the quest stays on the same step to retry. Retail sets sequence `0xFE` first and sends them out about 6 s later. |
| `set_hate(npc, target, amount)` | Gives NPC `npc` enmity on `target` (another NPC's layout ID, or the player when `nil`). Used to make allies and enemies fight each other. |
| `use_action(npc, action, target)` | Makes an NPC use an action (with its cast bar and area aimed where the target stands). |
| `say(npc, key)` | NPC says a line from the quest's text sheet by key, in `/say` under its BNpcName. |
| `say_battle_talk(npc, number, name_id)` | Says the quest's n-th `*BATTLETALK*` line (key order) under BNpcName `name_id` (0 shows `???`), even if the speaker isn't present. The log names the key it used. |
| `yell(npc, npc_yell_id)` | NpcYell row (e.g. 118 "Curses! Not now!", 122/123 the golem's rumbles). |

The battle talk window isn't known for this client (every candidate packet did nothing), so battle talk is said in
chat for now (see [Known issues](known-issues.md)).

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

## Adding a quest battle

1. Record it on retail (Duties tab: director sequence/variable changes, NPC chat, windows, timings; Monsters tab for
   the enemies' HP and skills).
2. Dump the battle's client script: `kawari-questdump extract <Name>Btl.luab` finds it by name in any game_script
   folder.
3. Add the NPCs to the territory drop-in as a `QB_…` layer.
4. Write `content/questbattle/<id>.lua` with the callbacks above; keep a timing table in its header.
5. Add any new enemy skills as action scripts (`resources/scripts/actions/`), see [Combat](combat.md).
