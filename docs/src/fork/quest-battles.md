# Quest battles and directors

Quest battles are the solo duties inside story quests (Chasing Shadows, Spirithold Broken, Leia's Legacy, To Guard a
Guardian, Underneath the Sultantree). The fork added them on top of upstream's content directors.

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
| `onUpdate(director, health, player_health, info)` | Every logic tick (500 ms). `health` maps each living NPC's layout ID to its health %, `player_health` the player's; `info` has everyone's real numbers and position (`info[layout] = { hp, max_hp, x, y, z }`, `info.player` the same), for highest-HP and nearest choices. Waves, thresholds and timed steps live here. |
| `onActorCast(director, layout_id, action_id, actor_id)` | An NPC began an action from its timeline or rotation (its cast bar, or an instant one). Optional. For scripts that answer what a boss does: the Copperbell Mines' Kottos' 28543 sets off its markers and leaps. |
| `onActorDeath(director, layout_id, position)` | An NPC died. Optional (the server skips it when a script, or a FATE's, doesn't define it). |
| `onPlayerDeath(director)` | The player fell. |
| `onSceneFinished(director, scene)` | A player finished one of the director's own scenes (`play_scene`), e.g. a cutscene halfway through. Optional. |

Director methods (`servers/world/src/server/director.rs`), the ones the fork added:

| Method | Does |
|---|---|
| `spawn_bnpc(layout_id)` | (upstream) Spawns a drop-in NPC, and returns its actor ID (fork): several spawned from one layout ID (retail's invisible helpers share one) can then be told apart by the methods below that take an actor ID. |
| `delay(seconds)` | The tasks queued after it wait that long (to the tick, 500 ms), until the next `delay` (`delay(0)` is at once again): a boss's choreography timed to the tenth of a second from one moment (`LuaDirectorTask::Later`, `Instance::later_director_tasks`). |
| `warp_bnpc(npc, x, y, z, rotation)` | Puts an NPC (actor ID or layout ID) somewhere at once: an ActorSetPos from it, as retail moves its helpers before they cast. |
| `jump_bnpc(npc, x, y, z, rotation)` | An NPC (actor ID or layout ID) leaps somewhere: ActorControl `SnapNextMove` (239) then an ActorMove there (speed byte 90), as retail's Kottos landed each leap; it carries on from where it lands. |
| `move_bnpc(npc, x, y, z, rotation)` | One ActorMove there (no snap), which the client walks the NPC along: the last Copperbell boss going to the middle for 28760. |
| `hold_bnpc(npc, held)` | Holds an NPC (actor ID or layout ID; `held` defaults to true): `npc_behavior` leaves it alone (no noticing, chasing, timeline or enmity on players), so it does only what the director has it do, as retail's boss helpers (`Instance::held_npcs`). |
| `release_bnpc(npc)` | ActorControl 417 with no one (0xE0000000) from the NPC, as retail let each boss helper go the moment its boss died. |
| `set_model_scale(npc, row)` | Draws an NPC at another ModelScale row (ActorControl 48, `SetModelScale`), as retail grew the second Copperbell boss (16, 17, 18). |
| `actor_use_action(actor_id, action, target_actor_id)` | `use_action` by actor IDs; with no target the NPC casts on itself, as retail's helpers do (an area attack cast on oneself lands on everyone inside it the caster's battalion is at war with, `action.rs`). |
| `set_sequence(n)` | Sets the director's sequence and sends DirectorVars (its first byte is the sequence). The client's own battle script reacts to it (e.g. `OnSequence001` sets the duty objective). `0xFE` = lost, shows "Duty Failed". |
| `complete_quest_battle(quest_sequence, stay, finish_scene, finish_flags, variables)` | Wins: the quest moves to `quest_sequence` (with `variables`, a table of quest variable index 0–5 to value, in the same journal update: Lurkers in the Grotto's UI8BH, `{ [1] = 0x10 }`, as retail sets it; replay reads the battle script's `QUEST_VARS_AFTER_WIN` for its `battle_won` step), "Duty Complete" and the victory fanfare (60001) play, then `finish_scene` (default 6, `QUEST_BATTLE_SCENE_FINISH`; retail plays 6 in Chasing Shadows and Leia's Legacy, 11 in To Guard a Guardian, 9 in Spirithold Broken) with `finish_flags` (default 0x2001, HIDE_HOTBAR and NO_DEFAULT_CAMERA, as retail plays all of them but Spirithold Broken's, a cutscene with 0xF8402EFB), and the players go back when it ends. With `stay = true` the players remain (for closing lines) until `leave_quest_battle()`. |
| `leave_quest_battle()` | Sends the players back where they entered. |
| `fail_quest_battle()` | Loses: players go back where they entered, the quest stays on the same step to retry. Retail sets sequence `0xFE` first and sends them out about 6 s later. |
| `set_hate(npc, target, amount)` | Gives NPC `npc` enmity on `target` (another NPC's layout ID, or the player when `nil`). Used to make allies and enemies fight each other. |
| `use_action(npc, action, target)` | Makes an NPC use an action (with its cast bar and area aimed where the target stands). |
| `end_fight()` | The fight is over: every NPC still standing forgets whom it hated, sheathes, leaves battle and drops its target, the players leave battle (retail's packets the moment Victory in Peril's boss fell: ActorControl ToggleWeapon 0 and SetBattle 0, ActorControlTarget SetTarget 0), and none takes anyone on again (`Instance::fight_over`: not hits, heals or damage over time, nor anyone coming close). `complete_quest_battle` does it too, so no one fights on through "Duty Complete" and the closing scene. |
| `battle_talk(npc, name_id, text_id, icon, duration_ms, kind)` | The **battle talk window** (DirectorPopupMessage, opcode 162 on 7.57), as retail shows it: `text_id` is the content's text by ID (retail's IDs from a recording; one outside the content's range crashes the client), `name_id` the speaker's BNpcName, `icon` the portrait, 4000 ms unless given. It comes from `npc` if it's there. `kind` is the speaker's object kind, 2 (BattleNpc) unless given; 3 (EventNpc) makes `name_id` an ENpcResident, for a speaker who isn't in the fight (Victory in Peril's watcher). A name that isn't in the sheet `kind` points to crashes the client. A `talk` step passes its speaker's `kind`. |
| `play_scene(scene, flags)` | Plays one of the director's scenes for every player (event type GameProgress), e.g. To Guard a Guardian's cutscene, scene 6 with flags `0xF8402EFB`. The script hears `onSceneFinished` when it's over. |
| `despawn_bnpc(layout_id)` | Removes an NPC (layout ID, or an actor ID from `spawn_bnpc`) without it dying (no `onActorDeath`). |
| `spawn_tethered_bnpc(layout_id, to, channeling)` | Spawns a drop-in NPC tethered to NPC `to` (layout ID): its spawn's tether target and Channeling row, the line the client draws between them until the NPC dies. The Tam-Tara Deepcroft's shields to Galvanth (Channeling 1). |
| `warp_players(x, y, z, rotation)` | Moves every player in the instance through a loading screen to a position in it (PrepareZoning to the same territory, then ActorSetPos). |
| `set_notice(npc, notices)` | With `false`, an aggressive NPC no longer notices whoever comes close (players or allied NPCs, `AGGRO_RANGE`); it fights only when hit or sent after someone (`set_hate`). `true` restores it. Kept per layout ID for the instance (`Instance::unaware_npcs`). |
| `set_invulnerable(npc, invulnerable)` | Whether NPC `npc` takes damage (its `currently_invulnerable`, as a timeline's invulnerability timepoint sets it). The Tam-Tara Deepcroft's last boss while a shield stands. |
| `boss_room(boss, wall, line, place, triggers)` | A boss arena whose boss isn't spawned with it (`spawn_boss` spawns and seals on the boss's own pull): sealed 15 s after the boss or any of `triggers` (layout IDs) goes for someone (logs 2012, 2013, the wall shown), opened when `boss` dies (both hidden, log 2014). The Tam-Tara Deepcroft's rooms, sealed by their pack's pull. |
| `set_attack_range(npc, yalms)` | How close the NPC gets to its target before attacking: melee (3) or a caster's distance. |
| `keep_near(npc, anchors, yalms)` | Keeps the NPC within `yalms` of every anchor (one, or a list: NPC layout IDs, `"player"` or `nil` for the player): it heads back to the middle of them when it's further from any, whatever it's fighting (Papalymo near Yda and you). `yalms` `nil` stops it. |
| `stand_behind(npc, tank, yalms)` | Has the NPC stand `yalms` behind the enemy `tank` (a layout ID) is fighting, out of its frontal cones and cleaves, as long as that keeps it near its anchors and its own target in reach. `yalms` `nil` stops it. |
| `say(npc, key)` | NPC says a line from the quest's text sheet by key, in `/say` under its BNpcName. |
| `say_battle_talk(npc, number, name_id)` | Says the quest's n-th `*BATTLETALK*` line (key order) under BNpcName `name_id` (0 shows `???`), even if the speaker isn't present. The log names the key it used. |
| `play_action_timeline(npc, timeline_id)` | Plays an animation (ActionTimeline row) on the NPC (ActorControl 407), as Sastasha's second captain does (980) as he calls his men. |
| `yell(npc, npc_yell_id)` | A speech bubble over the NPC: NpcYell row (e.g. 117/118 Yda's and Papalymo's in Chasing Shadows, 122/123 the golem's rumbles). The packet (NpcYell) is the actor, 4 bytes, the speaker's BNpcName (for a battle NPC; upstream's comment says ENpcResident), the NpcYell row, then four u32 parameters, 0 in every quest battle bubble on retail. |
| `party_npc_yell(role, npc_yell_id, delay)` | The party NPC (Duty Support member) playing `role` (`"tank"`, `"healer"`, `"melee"`, `"ranged"`) yells an NpcYell line `delay` seconds from now (0 if left out), under its BNpcName; nobody of that role, nothing. For a duty's story lines: see [Party NPCs](party-npcs.md#talking). |
| `play_cutscene(id, params)` | (upstream; `params` the fork's) Plays a duty's cutscene: the director's scene 3 with the cutscene's ID, then `params` (default 1, 38, 1, 0, retail's for Denn's arrival in Sastasha). Retail's differ per cutscene: Denn's death was 176 with 1, 18, 0, 0, and with the default ones the client played his arrival's dialogue again after it (Bob's bundle and Chronofoil capture of 2026-10-23). Take them from a capture. |
| `appear_bnpc(layout, timeline, delay)` | Brings a BNpc spawned hidden (display flags with INVISIBLE, kept from retail's spawn in the drop-in) into view `delay` seconds later (default 0.75, `DEFAULT_APPEAR_DELAY`): ActorControl `Appear` (36: a one-second fade-in, and ActionTimeline `timeline` played on it, 0 for none) and `SetTargetable` (54, 1) to everyone around, and INVISIBLE dropped from its spawn for whoever sees it later (`director::update_appearances`). As retail brought in the Copperbell Mines' ambushing gigas (158, 0.58-0.94 s after the spawn, all six ambushes). |
| `eobj_control(eobj, category, params)` | Sends an EObj (by EObj ID or layout ID) an ActorControl by category with up to four parameters, for the ones Kawari has no name for: the Copperbell Mines' blasts (413 with 8, 2) and last hall (409 with 64 and the director). `hide_eobj`, `show_eobj` and `delete_eobj` take a layout ID too, for EObjs sharing an ID (the EObj ID is tried first). |

## The battle framework (`content/lib/Battle.lua`)

On retail a battle's director runs the fight by an outline the developers wrote for it: who is on which side, what each
ally is there to do, and how that changes as the fight does. In Chasing Shadows Yda (a pugilist, close-range damage) is
there for the boss and then finishes whatever's left, toughest first; Papalymo (a thaumaturge, ranged damage) stands in
as support, since neither knows your class: he takes the adds from range, stays near everyone and heals you when you're
low. When new adds arrive the outline already says who deals with them.

`content/lib/Battle.lua` is that outline as code, shared by every battle. The server loads it before the battle's own
script; the script describes its fight and calls `BATTLE:update` / `BATTLE:died` from its callbacks:

```lua
BATTLE = Battle.new()
BATTLE:ally(BNPC_IDA, { focus = { "boss", "adds" }, order = "highest_hp" })
BATTLE:ally(BNPC_PAPALYMO, { focus = { "adds", "boss" }, order = "oldest", range = 10,
    keep_near = { within = 15 }, heals = { { action = 120, targets = { Battle.PLAYER }, below = 50, every = 4 } } })
BATTLE:enemy(BNPC_BOSS, { group = "boss", attacks = BNPC_IDA })
BATTLE:onHealth(BNPC_BOSS, 75, function(director) BATTLE:spawn(director, WAVE_1) end)
```

| Part | |
|---|---|
| `ally(layout, role)` | `focus`: the enemy groups it goes for, in order (`"any"` is everyone); `order` within a group: `oldest`, `newest`, `highest_hp`, `lowest_hp`, `nearest`; `range` (yalms, `set_attack_range`); `keep_near` (`{ anchor, within }`, `keep_near`); `heals` (another healer doesn't pick a target one has just started healing for `HEAL_PENDING_UPDATES`, 1.5 s: three healers on one timer all Cured the same ally at once, QB 38). |
| `enemy(layout, role)` | `group` (a name or a list, e.g. `{ "lostwings", "adds" }`); `attacks`: whom it goes for first (an ally, or the player when `nil`) with `enmity` (300); `passive = true`: no one at first, it waits to be hit (To Guard a Guardian's masked mage); it can have `heals` too (the sincallers heal Quzal). |
| heals | `{ action, targets (layout IDs, Battle.PLAYER, or "allies" for its whole side), below (%), every (s), say (a function) }`: heals whichever target is lowest under `below`. |
| `spawn(director, layouts)` / `remove(director, layouts)` | Brings NPCs in as their roles say (placement, first target) / takes them out without dying. |
| `enemy(…, { unaware = true })` / `engage(director, layouts, target, enmity)` | An enemy that doesn't notice anyone on its own (`set_notice(npc, false)` when it spawns) / sends enemies after someone (`nil`, the player) and lets unaware ones notice people again. |
| `holdBack(layouts)` / `release(director, layouts)` | Keeps allies out of the fight, spawned or not, until released (the tank then goes at once, the others `FOLLOW_DELAY` later): Victory in Peril's allies appear during a cutscene and wait for it to end. |
| `change(director, name, changes)` | A new context: `changes = { [layout] = { focus = … } }` from now on (the boss gone, someone joining). |
| `steps(list)` | Timed steps, one after another: `{ updates after the one before, kind, … }` with kinds `sequence`, `talk`, `say`, `yell`, `spawn`, `remove`, `action`, `change`, `call`. A step runs the update after its count is up. |
| `onHealth(layout, percent, fn)`, `onCleared(group or list, fn)` | Phases: once an NPC's health reaches a mark (or it falls before it), once a group (or a list) that arrived has all fallen. |

**Roles.** Every class belongs to one of three roles (the ClassJob sheet's `Role`: 1 tank, 2 melee and 3 ranged DPS,
4 healer), and the role says most of what an NPC is there to do; enemies follow it too, even unique ones with no class.
`role = "tank"`, `"dps"` or `"support"` starts an NPC from `Battle.ROLES`, and anything else in its table overrides it.
`role` is the part it plays in this fight; `class_role` the one its class has, when they differ:

| Role | Default |
|---|---|
| Tank | Leads the charge: the others hold back `FOLLOW_DELAY` (2 s) when a tank is there. Keeps the enemies off everyone else (`protect`): every `provoke_every` (10) seconds it pulls onto itself an enemy that's hitting someone else, one on a support before one on a DPS before one on the player (`PROTECT_ORDER`), with more enmity each time (`TANK_ENMITY`). Up close, the toughest enemy first. |
| DPS | The most versatile role: a DPS can carry heals and buffs and still be a DPS. The enemies with the most health (`highest_hp`). Two or more DPS **split** them: the first takes the toughest, the second the next, across their focus groups (the boss, then the adds by health), so between them they cover the high-HP enemies and clean up the adds. Can heal itself (a heal on `"self"`). Up close unless given a `range`. |
| Support | The backline: from 15 yalms, within 20 of the tank and the player, and **not straight behind the tank**: with the least health it has to keep out of cones and area attacks, so it stands behind the enemy the tank is fighting (`stand_behind`, 6 yalms beyond it), unless that takes it too far from the others or out of reach of its own target. `buffs` (`{ action, every }`) go on the DPS first, then the tank, in turn. Heals on `"party"` look after the tank first (below `tank_below`, 70 %: it takes the hits) and then whoever else is lowest below `below` (50 %). For enemies, `"party"` puts their boss first. |

The tank's pull uses who each enemy is targeting (`info[layout].target`). In Chasing Shadows both are DPS by class: Yda
(a pugilist) plays the **tank**, taking the toughest enemy (the boss) on up close, without a tank's pull
(`protect = false`); Papalymo (a thaumaturge) plays a ranged **DPS** with some support (his Cure on you), taking the adds
and staying within 15 yalms of both Yda and you. That's why the pair is put together as it is.

Allies are re-pointed whenever an enemy arrives or falls, roles change, and every 2 s (for highest-HP and nearest);
only an ally whose target changed gets new enmity, each time more than the last (`ALLY_ENMITY`), so it switches and
stays. Chasing Shadows and To Guard a Guardian run on it; simulated against their previous scripts (lupa), every
director call and every ally's sequence of targets came out the same. `questkit battle` drafts the outline from a
retail recording (see [questkit](questkit.md#battle)).

**The battle talk window** is DirectorPopupMessage (162), found in the 2026-10-16 recording of Leia's Legacy and To Guard a
Guardian. Layout (40 bytes): speaker actor (ObjectTypeId, 8 bytes), director handler, BNpcName, text ID, duration ms
(4000), icon, then one byte each for the speaker's object kind (2, a BattleNpc), style (0) and parameter count (0), a
pad byte and two u32 parameters. Upstream's struct had the kind as `ObjectKind`, which writes a second byte for a
BattleNpc and makes the packet 41 bytes; it's a plain `u8` now. OverlayPlugin reads the same packet (its BattleTalk2:
opcode 162, 40 bytes, the same fields; see [Outside sources](sources.md#overlayplugin)). The text IDs are the content's own (101–106 in Leia's
Legacy, 161–181 in To Guard a Guardian, 126–131 in Chasing Shadows, 152–154 in Spirithold Broken), so they're copied
from a recording or capture. A speaker who isn't in the battle (Spirithold Broken's masked mage) is sent with no actor
and no portrait, only the BNpcName. Victory in Peril's watcher, who isn't a BNpc at all, came with kind 3, an
EventNpc's actor ID (0x4000818f) and an ENpcResident name (1002702), the only talk of that battle not of kind 2; sent
as a BattleNpc with that name, the client crashed (Bob's bundle of 2026-10-22). Kawari sends it with kind 3 and no
actor.

The retail flow around a quest battle, from the same recording: EventStart (type 0x0f) and the opening scene 1;
when it ends, EventStart 0x0d with scene 60000 and the fight starts (Kawari's `onCommence`); on winning scene 60001,
then the finish scene, UpdateQuest, and DirectorVars with sequence `0xFE` and the byte after it `0x80`, then the
player is sent out.

The opening scene's flags differ from one quest battle to the next, always the same for one battle (Bob's Chronofoil
captures, 14 openings of 9 battles): 0xF841AEFB for 11, 12, 15, 35 and 36 (Kawari's default,
`QUEST_BATTLE_OPENING_FLAGS` in `main.rs`), 0xF8412FF7 for 14, 0xF8412EFB for 24, 0xF841AF0B for 29; a script with
other flags sets `OPENING_SCENE_FLAGS`. They hide the UI (0x800) and the other characters; Kawari used to send the
duties' opening flags (0x0C53A401, right for InstanceContent), and the UI stayed up with no skip button. Retail's
EventStart also has its flags byte set to 1 for the content directors' entering (0x0f) and progress (0x0d) events and
a quest's entering event, 0 for every other event (228 events across the captures); Kawari sends the same
(`event_start_flags`). Which of the two the skip button needs isn't known.

`!directorscene <scene> [flags]` plays any scene of the current director, which is how the victory fanfare scene
(60001, from the client's generic director script) was found.

## The battles

### Quest battle 44 — Into a Copper Hell (`content/questbattle/44.lua`)

* Quest 660 (ManFst205_00660) starts it with `player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)` (44, territory
  274; ContentFinderCondition 338). Territory 274 is taken to be a copy of Ul'dah - Steps of Nald (130): the player's
  spot (37.01, 4.0, -137.54, facing 2.879) and the three adventurers stand on its ground (y 4.0, by the Quicksand,
  where the quest's Momodi is), so they're layer `QB_ManFst205_001` of a new `wil_w1t1` drop-in appending
  `bg/ffxiv/wil_w1/twn/w1t1/level/planmap.lgb` (a guess at Steps of Nald's LGB; not checked against the game's
  files): the pugilist (4094987, BNpcBase 960, level 11, 1844 HP), the gladiator (4094990, 961, level 10, 288 HP) and
  the thaumaturge (4094992, 962, level 10, 134 HP), layout IDs, positions and display flags from the capture (Sapphire
  has the same layout IDs and no script).
* Bob's Chronofoil capture of 2026-10-25 (`4777fd0f`, `questkit chronofoil --battle 0.1/0.3/0.5/0.7`, duty 338): three
  lost, one won, as a level 21 conjurer. Sequence 1 and the opening cutscene (scene 1, 0xF841AEFB), all three there
  from the start; sequence 2 as the fight starts, the pugilist coming for you; its lines (battle talk under BNpcName
  991, portrait 73230): 204 5.1 s in, 205 below 65 %, 206 below 50 %; below 35 % sequence 3 and 207, 5 s later
  sequence 4 and the other two come (before that they fought only once hit: AC 106 state 0 and SetBattle on them);
  208 below 20 %. The marks held in all four runs (65: 67.0 -> 61.5, 68.8 -> 59.4, 65.2 -> 59.8, 70.0 -> 64.5; 35:
  38.0 -> 32.6, 35.0 -> 29.6, 38.0 -> 32.8, 40.1 -> 33.8). The pugilist heals itself (57) at 33-46 %.
* Won (all three down): 2 s after the last one's health reached 0, sequence 5 and "Duty Complete" (60001), then the
  battle's scene 5 (a cutscene, 0xF8402EFB, 76 s); as it ended, UpdateQuest put quest 660 at **sequence 5** and the
  director's sequence went to 0xFE with its last data byte (9) at 0x80. The script does it with
  `complete_quest_battle(5, false, 5, 0xF8402EFB)`, as the other battles move their quest; the quest's script
  doesn't set the sequence itself. Lost (your health at 0): sequence 0xFE at once and out 5 s later.
* Timelines `IntoACopperHell*_<base>.json` (rotations from the four runs' medians); the old kits' 10, 55 and 57 that
  these NPCs use have NPC-only scripts (`QuestBattleNpc_000xx.lua`). `questkit battle-sim 44` (the won run): every
  step within 1.2 s.

### Quest battle 38 — Duty, Honor, Country (`content/questbattle/38.lua`)

* Territory 271, a copy of Central Thanalan (layer `QB_ManWil007_001` of the `wil_w1` drop-in, 39 NPCs): Papashan
  (4087433, level 26, 5128 HP), Owyne (4087434, 17, 3690) and two Sultansworn elites (4087459/60, 15, 765) with you
  against Garibald (4087437, 20, 3799), three bowmen (271 HP) and three fistfighters (190); nine background pairs (26400
  HP each, display flags 0x40208) fight each other; the player starts at (247.74, -17.58, -131.33) facing 1.351. From
  Bob's Chronofoil capture of 2026-10-18, three runs (`questkit chronofoil --battle 0.36`, `0.38` lost as Papashan and
  Owyne fell, `0.40` won).
* Sequence 2 at the start, Papashan 149 at 5 s and Owyne 151 at 10 s. Garibald at 70 %: sequence 3, six more half a
  second later, Papashan 155 8 s after the sequence. Sequence 4 and Garibald's 157 at 30 %, or at 40 % once at most one
  other enemy stands (a guess that fits the three runs: 29.9 % with six standing, 36.5 % as the last-but-one fell, never
  at 50 %). At about 27 % he casts Void Call (867) on himself (retail's has no damage; it goes by his health: 0.6–0.7 s
  after he passed 27 % in both runs, 5.3 and 9.3 s after sequence 4); a second after it lands, sequence 5, his 158, and
  the blanga it called (after you); Owyne 156 5 s later; Papashan 150 at Garibald's 10 %. The blanga's Terror Eye (740,
  a 6-yalm circle where you stand) comes 16.3–16.9 s after it appears and then every 14.4 s (the script casts it),
  hitting you for about 33 and the allies by you for about 60.
* Bob's 2026-10-20 runs had sequence 4, Void Call and the blanga 10 s in: the script took Garibald's missing health on
  the first update (he's spawned after it) for 0 %; it now waits for his health, and battle-sim leaves NPCs spawned
  outside `onUpdate` out of the next update's health, as the server does, so it catches that.
* Papashan or Owyne falling loses it (at their 0 HP: sequence 0xFE and their line, 160 or 161); the elite 4087459
  falling only says 162.
* Everyone down (the blanga last): sequence 6, everyone else removed, the director's scene 6 (0xF8402EFB), then a
  loading screen to (258.39, -19.72, -129.58, facing 1.288). Sequence 7: a lesser gargoyle (after you) and the masked
  mage (passive until hit), the mage 165 at 5 s; sequence 8 at 22 s, 166 5 s later; the gargoyle at 50 % 168, at 30 %
  sequence 9 and Thancred (4087577, level 50), sequence 10 and his 169 10 s later, the mage 167 5 s after; the mage at
  40 % 170, Thancred 171 5 s later; the mage down: 2 s later sequence 11, scene 11 (0x2001), the quest moves to
  sequence 3. `questkit battle-sim 38`: every step within two seconds of retail.
* Who goes after whom, the same in every run: Garibald after Papashan, the bowmen after Owyne, the fistfighters
  4087449/452 after the elite 4087459 and 4087451 after 4087460, the second wave's fistfighters 504 → 460 and 502 → 459,
  its bowmen after Owyne, its two casters after Papashan; Papashan stays on Garibald, Owyne takes the bowmen, the elites
  the fistfighters. Papashan, Owyne and the elite 4087459 Cure whichever of the four is below 70 % (themselves too, never
  you), each about every 6 s (191 Cures in three runs); the spellweaver Cures its side. The gargoyle's Corrupted Tail
  comes 20.6 s after it appears, a second before sequence 8. Damage and heals are the capture's (`recorded_amounts`, 20 new timelines; hitboxes from Bob's Recorder
  export of the same runs).

### Quest battle 39 — Oh Captain, My Captain (`content/questbattle/39.lua`)

* Territory 273, a copy of Western Thanalan (the same Bg as 140, so its NPCs are layer `QB_SubWil088_001` of the
  `wil_w1f1` drop-in, `nonpop`): Baldewyn beside you (4093947, level 15, 701 HP, a gladiator), Fufulupa (4093955,
  level 15, 1403 HP, a lancer), a thaumaturge (4093957, level 15, 382 HP) and two Brass Blades (4093961, 4093962,
  level 14, 175 HP). Layout IDs, levels, HP, positions, display flags and the start spot (-238.43, 15.2, -270.35,
  facing 3.098) come from Bob's Chronofoil capture of 2026-10-18 (`questkit chronofoil --battle 0.23`, `0.25`, `0.27`,
  duty 337: lost, lost, won).
* Sequence 1 before the opening cutscene, 2 when the fight starts; Fufulupa's battle talk 102 (BNpcName 1381, portrait
  0x11E27) at 5 s, Baldewyn's 103 (1378, 0x11E26) at 10 s; Fufulupa at 70 %: sequence 3 and 104; below 45 % (45.5 %
  and 45.7 % didn't trigger it): sequence 4 and 105; 10.2 s later (a timer: his health then was 31 %, 30 % and 25 % in
  the three runs) sequence 5, half a second after he uses Gut Rip (477, Critical Skill) on himself; Baldewyn's 106
  3.1 s later; Heartstopper (866) at you 6.1 s after sequence 5 and every 6 s after (crits: about 58 on you, 110 on
  Baldewyn). Everyone but Baldewyn down: 2 s later sequence 6, "Duty Complete", scene 6 (0xF8402EFB); the quest moves
  to 255. `questkit battle-sim 39`: every step within a second of retail in all three runs.
* **Losing:** Baldewyn falling is a loss (both lost runs): his talk 101 and sequence 0xFE at once, out about 6 s later.
  The battle's fixtures include the two lost runs (`battles/39-lost-*.json`, which `battle-sim` checks too).
* As on retail: Fufulupa and the thaumaturge go after you, the Brass Blades after Baldewyn; Baldewyn takes the
  thaumaturge, then the Brass Blades, then Fufulupa, and casts Cure (about 65) on you (47–70 %) and himself (9–50 %).
  The hits are the capture's (`recorded_amounts`, timelines `BaldewynFourYalms_953`, `FufulupaTheUnbending_954`,
  `SerBaronVonQuiveronIVEsquire_955`, `BrassBlade_956`).

### Quest battle 35 — Lurkers in the Grotto (`content/questbattle/35.lua`)

* Territory 252, a copy of Middle La Noscea (retail's positions in it are Middle La Noscea's, where the quest's stone
  monument stands, so its NPCs are layer `QB_ManSea005_001` of the `sea_s1f1` drop-in, `nonpop`; that 252 loads that
  drop-in, i.e. has Middle La Noscea's Bg, is assumed until a run shows it): the ally beside you (4083622, level 50,
  1372 HP, a conjurer), the boss (4083604, level 12, 6196 HP) and three waves of two (4083606/07 and 4083610/11, level
  5, 56 HP; 4083608/09, level 6, 75 HP). Layout IDs, levels, HP, positions, display flags (0x40003) and the start spot
  (-58.21, 25.26, -135.32, facing 0.122) come from Bob's Chronofoil capture of 2026-10-21 (`questkit chronofoil
  --battle 0.7` and `0.9`, duty 332: lost, won).
* Sequence 1 before the opening cutscene, 2 when the fight starts; the ally's battle talk 82 (BNpcName 1374, portrait
  73187) at 10 s; the boss at 65 % (it went 73.5 % → 63.0 % around then; Kawari's first guess, 75 %, came 6 s early in
  the simulation): sequence 3, the first wave and 83 seven seconds later; at 41 % (45.5 % → 37.2 %): sequence 4 and
  the second wave; 10 s later (a timer, both runs): sequence 5, the third wave and 85. The ally's 84, 86 and 87 come
  as the player's health drops below 80 %, 40 % and 15 % (86 came 0.5 s and 4.1 s after the player fell below 40 % in
  the two runs, so its fixture allows 4.5 s). Everyone down: 2 s later sequence 6, "Duty Complete", scene 6 (0x2001);
  the quest moves to 2 with UI8BH = 1 (ITEM0). Losing (the player down): 0xFE, out 6 s later.
* As on retail: the boss fights the ally the whole time and the waves go after you; the ally goes to the boss (which
  hardly moves: at most 4.1 yalms on retail) and stays on it, then takes the adds, casting Stone, Aero, Stone II and Aero II from about 5 yalms and Cure on you
  (17–48 %, every 12–20 s). The hits are the capture's (`recorded_amounts`, timelines `QB35Ally_904`, `QB35Boss_902`,
  `QB35Add_903`, `QB35Add_905`). `questkit battle-sim 35`: every step within 1 s of the won run but 86.
* First run in Kawari (Bob's 2026-10-21 bundle): with the ally told to stay within 3 yalms of you (retail's figure,
  because Bob stood by the boss there), it stayed by Bob casting from the start, the boss walked over to it and
  everyone piled onto him. The ally now goes to the boss.

### Quest battle 36 — Feint and Strike (`content/questbattle/36.lua`)

* Territory 280, taken to be a copy of Western La Noscea (retail's positions are Western La Noscea's), so its 31 NPCs
  are layer `QB_ManSea007_001` of the `sea_s1f4` drop-in, with retail's layout IDs, levels, HP, positions and display
  flags (the five background pairs are 0x4020B). The player starts at (667.83, 8.27, 524.17), facing -2.74. Bob's
  Chronofoil capture of 2026-10-21, four runs (`questkit chronofoil --battle 0.27`, `0.29`, `0.31`, the third lost as
  he fell, and `0.33`, won; territory 280, duty 334). It's Duty, Honor, Country with Limsa Lominsa's numbers.
* First half: the leader (4102995) holds the boss (4102991) and Cures his side below 70 %; the marauder takes the
  archers, the two elites the pugilists (an elite falling only says its line, 152 or 153); five pairs fight each other
  further off. Talks 144 and 145 at 5 and 10 s; the boss at 70 %: sequence 3 and six more (two archers, two
  pugilists, a conjurer that Cures its side, a thaumaturge), talk 147 8 s later; at 30 % (or 40 % with two others
  left) sequence 4 and 148; at 35 % after that Void Call (867) on itself, 4 s later sequence 5, 149 and the blanga
  (Terror Eye, 740, 16.2 s after it appears, then every 15 s); 146 5 s after; at 10 % the marauder's 154.
* The boss's health at 0: 7.8 s later (its death 2 s after, then 5.8 s) sequence 6, everyone leaves and the director's
  cutscene plays (scene 6, 0xF8402EFB); then retail sends you through a loading screen to (658.91, 9.56, 489.72)
  (`warp_players`), sequence 7, the gargoyle and the masked mage (passive until hit), talk 156 at 5 s, sequence 8 at
  22 s and 157 5 s later; the gargoyle at 50 % 159, at 25 % sequence 9 and the level-50 conjurer ally (4103188), 10 s
  later sequence 10 and 160, 158 5 s after; the mage at 40 % 161, 162 3 s later; the mage down: at once sequence 11,
  "Duty Complete", scene 11 (0x2001), the quest moves to 3. Your falling loses it.
* Timelines `QB36*_<base>.json`, with retail's recorded amounts. `questkit battle-sim 36` (the won run, NPC health held
  between hits): every step within 2 s.

### Quest battle 24 — Victory in Peril (`content/questbattle/24.lua`)

* Territory 249, taken to be a copy of Lower La Noscea, so its NPCs are layer `QB_SubSea117_001` of the `sea_s1f2`
  drop-in (15 NPCs, display flags 0x40000); the player starts at (165.01, 14.1, 676.28), facing -2.727. Bob's
  Chronofoil capture of 2026-10-21, two runs (`--battle 0.20`, lost as he fell, and `0.22`, won; territory 249, duty
  313).
* The boss (3970651) and four corsairs stand by. Each of the first three parts starts when you come close, at
  retail's distances (Bob's capture, both runs: the player's positions are in the capture, the NPCs' in the runs,
  on one clock): the first pair when you're within 26 yalms of the nearer (or hurt one): sequence 3, they come at you
  (the won run: sequence 3 as you came within 25.6, the first arrow 1.3 s later; the lost run: the first arrow as you
  came within 22–27, sequence 3 4 s after it). The second pair, once the first is down, when you're within 26.7 yalms
  of the nearer (its conjurer both times): sequence 4 came 0.1–0.2 s after you were 26.6 and 25.4 yalms from it and
  not at 26.9–28.2, though 4.2 and 5.3 s after the first pair fell, so it's the distance; they don't notice you on
  their own until then. The cutscene, once those two are down, when you're within 15 yalms of the boss: sequence 5
  came 0.1–0.2 s after you were 13.8 and 14.4 yalms from it and not at 15.7–16.4 (Bob's X 26.1, Y 33.7, about 15 from
  it); the client then stops you where you stand, 13.1–13.3 off. (Two earlier readings of these had the player's
  clock 7.3 s off the NPCs' and gave a 9 s wait, which Bob's third run of 2026-10-22 showed wrong.) Sequence 5 brings
  the cutscene (scene 5, 0xF8402EFB) and the allied marauder and conjurer (Cure below 55 %), held back (`holdBack`)
  until it ends. Nothing fights during it (retail: the boss's first blow is 1 s after sequence 6): the boss is
  unaware until then, as it would otherwise take on the allies appearing 13 yalms from it (Bob's bundle of
  2026-10-22: it fought the marauder through the cutscene, reached 55 % and turned on him). After it sequence 6, the allies released, the boss on the marauder, three more corsairs and talks 105–108 every 5 s; the boss
  at 55 %: sequence 7, five more (after the marauder), the boss on you, 8 at 4.5 s and 109–111; at 25 %: 9 and 112,
  10 5.5 s later, 115, 113, 114. Its health at 0: the fight ends at once (`end_fight`; retail: the allies sheathed
  and left battle, and so did you, in the same packets as its death; the won run had no adds left, Bob's run of
  2026-10-22 had two archers who went on to kill the allied conjurer and shoot him through the closing scene); 2 s
  later sequence 11, "Duty Complete" (scene 11, 0xF8402FFF) and
  114 if not said yet; the quest moves to 4. The corsair conjurers Cure their side: on retail 81–86 HP each, 5.5–7.7 s
  apart, below about 50 % (34 Cures in the two runs). The one in the second pair (4167379) only ever Cured itself and
  the archer beside it, never the first pair, so only those two are its targets; the third (3970673) Cures the boss
  and the adds.
* What everyone does to whom is retail's (`recorded_amounts`): the archers' 11–12, the boss's 12 and Heavy Swing's 19
  on you, the conjurers' Stone 18 and Aero 7 with its damage over time 2–7 a tick (the tick is scaled as the hit is:
  it was 26–28, Bob's "ghost damage"), the third wave's Fire 32 every 3 s (the thaumaturges never cast it before:
  [Class rotations](combat.md#class-rotations-serverrotationrs)), Fast Blade 39 and True Thrust 38 on the allied
  marauder. Not done: the boss's Fracture, Brutal Swing, Bloodbath, Foresight, Overpower and Barbaric Surge, the
  thaumaturges' Groundburst and Spinning Kick, the marauder's Shield Bash (1–12 uses a run each; no action scripts).
* Timelines `QB24*_<base>.json`. `questkit battle-sim 24` (the won run, NPC health held, retail's positions): every
  step within 0.7 s, sequences 3–5 within 0.5 s. Against the lost run (not a fixture) sequences 4 and 5 are within
  0.3 s and sequence 3 5.5 s early (see above).

### Quest battle 29 — Just Deserts (`content/questbattle/29.lua`)

* Territory 272, taken to be a copy of Middle La Noscea as 252 is (retail's positions are Middle La Noscea's), so the
  clay golem is layer `QB_SubSea057_001` of the `sea_s1f1` drop-in: layout 4082418, base 549, level 10, 844 HP, display
  flags 0x40000, at retail's spot; the player starts at (-25.52, 10.87, -439.72), facing 2.49. Bob's Chronofoil capture
  of 2026-10-21 (`questkit chronofoil --battle 0.11`, territory 272, duty 333, won; he was an arcanist, so his
  carbuncle, BNpcBase 13498, is in it too). The golem's timeline is Spirithold Broken's (`ClayGolem_549`).
* Way Down in the Hole's fight with Limsa Lominsa's lines (battle talk 82, 83, 84 under BNpcName 750, no portrait):
  sequence 2 at the start, 3 five seconds later with NpcYell 122; at 70 % talk 82 and sequence 4, Sand Breath 3.2 s
  later, sequence 5 8 s after 4, talk 83 and sequence 6 5 s after that; below 45 % sequence 7, then a second later
  NpcYell 123 and Plaincracker, every 16.3 s from then on (Ul'dah's started at 30 %); below 30 % sequence 8, Sand
  Breath 2 s later and Plaincracker 6.4 s after it, then every 16.3 s again; below 15 % talk 84 and sequence 9; its
  health at 0: 2 s later sequence 10, "Duty Complete", scene 10 (0xF8402EFB) and out; the quest moves to 5.
* `questkit battle-sim 29`: every step within 2 s. Bob stopped attacking twice for about 50 s, so the golem sat at
  30.1 % for 55 s; interpolating its health between hits put sequence 8 53 s early, so this fixture holds the health
  between hits (`"npc_health": "held"`, see [questkit](questkit.md#battle-sim)).

### Quest battle 18 — Way Down in the Hole (`content/questbattle/18.lua`)

* Territory 248, a copy of Central Thanalan (layer `QB_SubWil066_001` of the `wil_w1` drop-in): the clay golem
  (3872242, level 10, 844 HP, at retail's spot), the player starting at (-224.76, -31.75, 111.42), facing -0.537.
  Bob's Chronofoil capture of 2026-10-18 (`questkit chronofoil --battle 0.11`, territory 248, duty 312).
* Spirithold Broken's fight (quest battle 15) with Ul'dah's numbers: sequence 1 and the opening cutscene; 2 at the
  start, 3 three seconds later with the golem's NpcYell 122; at 70% the masked mage's battle talk 95 (BNpcName 750, no
  portrait) and sequence 4, Sand Breath (445) 2.4 s later, sequence 5, then talk 96 and sequence 6; below 45% (it went
  45.5% -> 41.4%) sequence 7, NpcYell 123 and Plaincracker (482); at 30% sequence 8, Sand Breath, then Plaincracker
  every 16.3 s; at 15% talk 97 and sequence 9; dead: sequence 10, "Duty Complete", scene 10 (0xF8402EFB) and out; the
  quest moves to sequence 5. `questkit battle-sim 18`: every step within two seconds of retail.

### Quest battle 37 — Underneath the Sultantree (`content/questbattle/37.lua`)

* Territory 270, a copy of Central Thanalan (the same LGBs as 141, so its NPCs are a layer of the `wil_w1` drop-in:
  `QB_ManWil005_001`, `nonpop`). The blanga (4077147, level 12, 13608 HP), the handsome stranger beside you (4077150,
  level 50, 2589 HP, a gladiator) and three pairs of lesser blangas (level 4, 44 HP): layout IDs, levels, HP,
  positions, links and display flags from Bob's Chronofoil session of 2026-10-17 (`questkit chronofoil --battle 4.4`,
  the battle in territory 270, duty 335), and the start spot (198.3, 14.24, 538.25, facing -1.24) from its
  SpawnPlayer. Sapphire's script has the same layout IDs, two waves (70 % and 40 %) and guessed HP.
* Sequence 1 before the opening cutscene, 2 the moment the fight starts (0.1 s after scene 60000). The stranger's
  battle talk (BNpcName 1377, portrait 0x11DCD): 114 five seconds in; at the blanga's 70 % sequence 3 and two lesser
  blangas, 115 seven seconds later, 116 five after and 117 thirteen after that (25 s after sequence 3); at 40 %
  sequence 4 and two more, 118 ten seconds later; at about 23 % sequence 5 and the last two; 119 at 10 %. Everything
  down: sequence 6, scene 6 (0x2001), the quest moves to sequence 2. `questkit battle-sim 37`: every step within two
  seconds of retail.
* As on retail: the stranger stays on the blanga the whole time and it on him; the lesser blangas go after you; once
  the blanga is down he kills the lesser blangas one after another, and he casts Cure (120) on you when you're low
  (retail: once, at 42 %; Kawari below 50 %, at most every 15 s).
* A second run (Bob's Chronofoil capture of 2026-10-18, `questkit chronofoil --battle 0.9`, a level 6 gladiator)
  agreed on the layout IDs, levels, HP and timings, and settled two things: 117 is a timer (+25.1 s after sequence 3
  in both runs; the first pair was still alive in the second, so it isn't "the pair went down"), and the pairs come
  at the blanga's health (the hit before sequence 5 took it from 24.8 % to 22.7 %; both runs were also 15 s after the
  second pair, which follows from the stranger's steady damage).
* Damage from the capture's hits (`recorded_amounts`): the stranger's auto-attack 212.8, Fast Blade 241.8, Savage
  Blade 300.2, Rage of Halone 165, Spirits Within 425; the lesser blangas' 3.9 on the player. The blanga missed the
  stranger 25 times in 46 (its Bloody Claw 3 in 6), against Kawari's fixed 8 %, so its amounts are per swing,
  misses included (15.3 and 21.8), to wear him down as much as retail did (to about 74 %). On retail the stranger
  dealt about 13000 of the blanga's 13608.
* Rotations from the capture: the stranger's (pace 5.6 s; Savage Blade only after Fast Blade, as all four times;
  Rage of Halone on its own, which needed the combo rule in [Combat](combat.md#class-rotations-serverrotationrs)),
  the blanga's Bloody Claw every 7.9 s.
* The quest (`ManWil005_00550`): ACTOR1 offers it with scene 2; retail plays the aftermath
  (scene 5, flags 0xF8402FFF) on EventStart type 15 when the player is back in Central Thanalan.

### Quest battle 11 — Chasing Shadows (`content/questbattle/11.lua`)

* On the battle framework. Yda (DPS by class) plays the tank: the boss, then what's left, highest HP first; she goes in
  first and Papalymo follows 2 s later. Papalymo (DPS, with some support): the adds (oldest first) from 10 yalms,
  staying within 15 yalms of Yda and you (a guess until a recording shows it), Cure on you below 50 %.
* Yda (3802271) and Papalymo (3802272) fight beside you. Yda goes for the boss and stays on it; Papalymo takes the adds
  (oldest first) and casts at 10 yalms. Adds start with some enmity on their ally (`STARTING_ENMITY = 300`) that the
  player can pull away; each new ally order outweighs the last (`ALLY_ENMITY = 10000`).
* Waves arrive as the boss drops to 75 %, 45 % and 25 % (Sapphire's layout IDs; Sapphire has 50 % for the second,
  but the capture shows it at 45 %: the boss went from 45.8 % to 43.7 % as it came).
* Yda: Pugilist rotation (Bootshine, True Strike, Snap Punch, 3 s apart). Papalymo: Thunder (refreshed as it runs out),
  Fire and Blizzard, and **Cure on the player below 50 %** at most every 4 s, with a line at most every 20 s.
* Director sequence as on retail: 1 before the opening cutscene, 2 the moment the fight starts (scene 60000; the
  Recorder's "8.5 s" was from entering the zone), 3–5 with each wave, 6 on winning.
* Battle talk with retail's text IDs (Bob's Chronofoil capture, 2026-10-14): Papalymo 126 and Yda 130 5 s after the
  fight starts, Yda 131 and Papalymo 127 10 s after the first wave, each pair 5 s apart, Papalymo 128 3 s after the
  last wave's health reaches 0; Papalymo's BNpcName here is 1376 (portrait 0x11DCC), Yda's 1375 (0x11DC4). Speech bubbles about
  7 s after the last wave: Papalymo's NpcYell 118, then Yda's 117 1.5 s later. Closing scene 6 (0x2001).
* Losing: sequence `0xFE`, out after 6 s. Winning moves the quest to sequence 2.
* Not yet: the enemies' own skills (auto-attacks only).

### Quest battle 15 — Spirithold Broken (`content/questbattle/15.lua`)

* The clay golem (layout 3869528, level 10, 844 HP) with auto-attacks, **True Grit** (120° cone, Blind) and
  **Plaincracker** (circle around it).
* Timed from a retail recording and Bob's Chronofoil capture (2026-10-14): sequence 1 before the opening cutscene,
  the rumble and sequence 2 the moment the fight starts (it was 10 s later, timed from entering the zone), the masked
  mage's lines (battle talk 152, 153 and 154, BNpcName 750, no speaker actor or portrait) and sequences 3–8 through the
  fight, "Duty Complete" the moment the golem falls (sequence 9), then the battle's scene 9, a cutscene (0xF8402EFB)
  with two more lines the client plays itself, and out when it ends (`complete_quest_battle(4, false, 9,
  0xF8402EFB)`). The phases come at the golem's 70 %, 45 % and 15 %, as the capture shows (`questkit battle-sim 15`);
  Sapphire's 65 % and 30 % came 7 and 16 s late against it.
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
  much health (191400 / 79750) fighting each other in the background: five fights, as the layer groups them
  (`INIT_NOTAR_P_POP_<n>ST_..` for the guard, `INIT_NOTAR_POP_<n>ST_..` for its Ixali). The two sincallers fight a guard
  each, the six melee Ixali the other three guards in pairs. The Recorder bears it out: each of three guards took 112
  hits on retail (two Ixali), each of the other two 26 (one sincaller casting), and every guard and Ixal kept fighting
  the whole time. Pairing them by hand had left an Ixal 47 yalms from its guard, with two Ixali and a guard standing
  idle. The first six down bring six more (sequence 3); Quzal at 34 % (sequence 4), and at about 27 % he casts Void
  Call (2.7 s): a second after it lands, sequence 5 and the blanga it called. Which Ixali come first and which second
  is confirmed by the capture (below).
* Everyone down: sequence 6, everyone else is removed and the director's cutscene plays (scene 6). After it (sequence
  7) a lesser gargoyle and the masked mage; sequence 8 22 s later; the gargoyle at 25 % brings Yda and Papalymo
  (sequence 9, 10 s later 10); the mage below 42 % says one more line (the capture, `questkit battle-sim 12`). The
  mage down: 2 s later (its death animation, as retail waits) sequence 11, finish scene 11, the quest moves to
  sequence 3.
* As on retail: Quzal goes after Galfrid, the lostwings after Miraudont, the slowbeaks after the two guards (3803267
  the bow, 3803268 the lance), both sincallers after Galfrid at first; the blanga and the gargoyle after the player.
  The masked mage stands by, spawned not hostile, until someone hits it (34 s after it appeared, on retail), then
  goes after them, and turns on Yda when she arrives; Bob remembers it also coming once the gargoyle is down, so that
  starts it too. The gargoyle stands by the mage at the tree until the player has loaded in after the cutscene (it set
  off half a second after ZoneIn, 4.8 s after appearing), then comes straight for them. Galfrid and Yda take the boss;
  Miraudont and the guards clear the adds first and only then turn on Quzal; Papalymo takes the gargoyle first and then
  the mage.
* The sincallers stay back by the tree behind Quzal, unlike a player healer: from where they appear (18 yalms behind
  him) they settle about 15 yalms behind him, 1.6 yalms apart, and cast from 12-26 yalms. Their range alone (19) keeps
  them there; keeping them near Quzal had sent both to the same spot, inside each other.
* The player starts facing -0.636 (retail's SpawnPlayer); it was 1.88, nearly the opposite way. Leia's Legacy (-1.288,
  was -0.36), Chasing Shadows and Spirithold Broken (start spots about 1 and 3 yalms off) are retail's now too.
* The background guards and Ixali are scenery: retail spawns them with display flags 0x208 (no health bar, can't be
  targeted) where the fighters have 0x001, and without CharacterData flag 0x1. The drop-in's battle NPCs take a
  `display_flags` now, and those 13 have retail's.
* **Checked against Bob's Chronofoil capture of the same run** (2026-10-16, `questkit chronofoil --battle 41.6`, see
  [questkit](questkit.md#chronofoil)). The raw packets say what the Recorder couldn't: which layout IDs arrived when
  (SpawnNpc), every NPC's target as it changed (retail sends ActorControlTarget SetTarget for NPCs too), the enemy
  list, and the HP at each moment. All 20 timed steps and 20 battle talk lines were within half a second of the
  script. What changed: sequence 4 at 34 % (Quzal went 34.8 → 33.4 % as it came; a third would never have fired), Void
  Call at about 27 % instead of with sequence 4, and sequence 5 when Void Call lands rather than at 25 % (Quzal was
  already at 23 %); the second wave's two slowbeaks swapped; the second sincaller starts on Galfrid; the mage passive
  until hit; Papalymo on the adds first and his Cure on you below 70 % (retail's came at 67 %). One run can't tell
  whether Void Call goes by Quzal's health (26.2 % when it began) or by time (11.8 s after sequence 4).
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

1. Record it on retail with Recorder 2.4 or later, ideally two or three times played differently, marking moments
   with `/krec mark`. Then `python3 tools/questkit battle <export.zip>`: who's in it (with their layout layer, instance
   name and AI settings), the waves and what preceded each, every NPC's targets with which explanations fit each change
   (counted), its actions and intervals, heals with how hurt the target was, and who never acted. The Duties tab has
   the director's sequence/variable changes; the Monsters tab the enemies' HP and skills.
2. Dump the battle's client script: `kawari-questdump extract <Name>Btl.luab` finds it by name in any game_script
   folder.
3. Add the NPCs to the territory drop-in as a `QB_…` layer.
4. Write `content/questbattle/<id>.lua` with the callbacks above; keep a timing table in its header (seconds from the
   fight starting, the director's scene 60000, not from entering).
5. Make its battle fixture from a Chronofoil capture of the run (`questkit chronofoil --battle N.V --out run.json.gz`,
   then `questkit battle-sim --make <id> --run run.json.gz`) and run `questkit battle-sim <id> -v` until it matches
   ([questkit](questkit.md#battle-sim)); CI holds it to that from then on.
6. Add any new enemy skills as action scripts (`resources/scripts/actions/`), see [Combat](combat.md). Give allies and
   enemies that fight with a class's actions a `rotation` in their timeline (questkit's report writes one per NPC),
   rather than fixed timepoints.
