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
4. It's a duty (2026-10-27): its ContentFinderCondition is the row whose Content is the battle and whose territory
   is where it's fought, so ZoneInit names it and the player is synced to it ([Level Sync](level-sync.md)). ZoneInit
   also carries where the player starts, as retail's does (every zone change; Kawari sent the origin).
5. Its NPCs come from the territory's drop-in (`resources/dropins/fst_f1_planmap.json` layers named
   `QB_<quest script>_001`), with positions, levels and HP from Sapphire or a retail recording.

## Director script

Globals the script sets:

| Global | Meaning |
|---|---|
| `START_POSITION = {x, y, z}`, `START_ROTATION` | Where the player appears. |
| `EXIT_AFTER_WIN = {territory, x, y, z, rotation}` | Where a won battle sends the player when they leave it, instead of back where they entered it from (also where a logout after the win puts them): Lord of the Inferno's sends them on to Southern Thanalan. A lost one always goes back. |
| QuestBattle sheet variables | Injected like quest variables (`INIT_POP_01`, `BNPC_NAME_01`, `DEF_ACTION_*`…). Scripts use `X or <number>` so they still work if a name is missing. |

Callbacks:

| Callback | When |
|---|---|
| `onSetup(director)` | The instance was created. |
| `onCommence(director)` | The player is in: spawn NPCs, set the first sequence, start the chatter. A dungeon's script that defines it hears it as the duty commences (DutyCommence, since 2026-11-03): Brayflox's Longstop's dragon flies off 3.7 s later. |
| `onUpdate(director, health, player_health, info)` | Every logic tick (500 ms). `health` maps each living NPC's layout ID to its health %, `player_health` the player's; `info` has everyone's real numbers and position (`info[layout] = { hp, max_hp, x, y, z }`, `info.player` the same, and the party's NPCs, which have no layout ID, as `info.party1`, `info.party2`...), for highest-HP and nearest choices. Waves, thresholds and timed steps live here. Since 2026-10-30 every player is in `info` too, as `player1`, `player2`... (`player` is one of them), and an NPC targeting a party NPC (no layout ID) has `target` `"party"` (it had none, so a boss pulled by a Duty Support tank looked unpulled.) Since 2026-11-06 each entry has its actor ID (`id`) and its target's (`target_id`, when it has one): whom the Navel's Weight of the Land isn't under, and where a gaoled player stands. |
| `onActorCast(director, layout_id, action_id, actor_id, x, y, z, target_id)` | An NPC began an action from its timeline or rotation (its cast bar, or an instant one). Optional. For scripts that answer what a boss does: the Copperbell Mines' Kottos' 28543 sets off its markers and leaps. Since 2026-10-30 also where its target stood as it began (`x, y, z`, nil without one): the Thousand Maws of Toto-Rak's last boss's eggs land where its 698 was aimed. Since 2026-11-06 the target's actor ID after them (`target_id`, nil without one): the Navel's Rock Throw gaols that one. |
| `onActorDeath(director, layout_id, position)` | An NPC died. Optional (the server skips it when a script, or a FATE's, doesn't define it). |
| `onPlayerDeath(director)` | The player fell. |
| `onPartyWipe(director)` | A duty's party wipe (every player down, `server/wipe.rs`), as everyone is sent back to the start: reset the fight (the Bowl of Embers takes Ifrit away and brings him back). Whatever the script had `delay`ed is dropped first. Optional. |
| `onDutyRecommence(director)` | The duty begins again after a wipe (DutyRecommence). Optional. |
| `onSceneFinished(director, scene)` | A player finished one of the director's own scenes (`play_scene`), e.g. a cutscene halfway through. Optional. |
| `onObjectTalk(director, actor_id, object_id, base_id)` | A player talked to an event object of a quest battle's (one it brought out with `spawn_eobj`: a quest battle's object with no event of its own gets the director as its handler, as retail's rosary in In the Eyes of Gods and Men did). The script answers with no scene, as retail: `event_action(bar, actor_id, object_id)`, then `onEventActionCast` as the bar ends, which finishes the event (`finish_gimmick`). Since 2026-11-07 the server ends a director's bar as retail does, ActorControlTarget EventAction `[0, id]` on its target (a quest's bar always had it; the dungeons' keys and doors didn't). Without it, or for anything else in the battle (an event NPC), the server plays EVENT_QIB_NOT_TALK (scene 10001) as before (`ToServer::QuestBattleTalk`, `FromServer::QuestBattleNotTalk`; since 2026-11-06). |
| `onContentNpc(director, actor_id, id, npc, results)` | A player's talk with a duty's ContentNpc (row `id`, event NPC actor `npc`) returned: the server played its scene (4, flags 0x2000, retail's for ContentNpc 1) and the script finishes the event (`finish_gimmick(actor_id)`); without it the event just finishes. Brayflox's Longstop's key (2026-11-03). |

Director methods (`servers/world/src/server/director.rs`), the ones the fork added:

| Method | Does |
|---|---|
| `spawn_bnpc(layout_id, x, y, z, rotation)` | (upstream) Spawns a drop-in NPC, and returns its actor ID (fork): several spawned from one layout ID (retail's invisible helpers share one) can then be told apart by the methods below that take an actor ID. With a position (fork, 2026-10-30) it spawns there instead of its layout's spot, and faces `rotation` if given: the Thousand Maws of Toto-Rak's adds and eggs, one layout ID each, wherever retail put them. |
| `delay(seconds)` | The tasks queued after it wait that long (to the tick, 500 ms), until the next `delay` (`delay(0)` is at once again): a boss's choreography timed to the tenth of a second from one moment (`LuaDirectorTask::Later`, `Instance::later_director_tasks`). |
| `warp_bnpc(npc, x, y, z, rotation)` | Puts an NPC (actor ID or layout ID) somewhere at once: an ActorSetPos from it, as retail moves its helpers before they cast. |
| `jump_bnpc(npc, x, y, z, rotation)` | An NPC (actor ID or layout ID) leaps somewhere: ActorControl `SnapNextMove` (239) then an ActorMove there (speed byte 90), as retail's Kottos landed each leap; it carries on from where it lands. Without `rotation` it faces the way it leapt (retail's Kottos did). |
| `busy_bnpc(npc, seconds)` | Keeps an NPC (actor ID or layout ID) from moving, turning or acting on its own for that long, its timeline still counting (`Instance::busy_npcs`): Kottos from its 28543 to its 28546, through its leaps. |
| `root_bnpc(npc, seconds)` | Keeps an NPC (actor ID or layout ID) where it stands for that long: it doesn't move, but acts as ever, its timeline and attacks included (`Instance::rooted_npcs`; `busy_bnpc` would hold its timeline's actions back until it's over): Lady Amandine standing 4.9 s beside her eye where she leapt, using 28643 there. |
| `restart_timeline(npc, anchor)` | Starts a `from_pull` timeline's clock again from now for its points `after` `anchor` (an action's ID, as a point with `restarts` does, or any number the timeline uses for a phase), so the director begins a boss's phase: Ifrit's at 80 % and 50 % (his lines' text IDs, 2601 and 2602) and after Hellfire (458). |
| `move_bnpc(npc, x, y, z, rotation, speed)` | One ActorMove there (no snap), which the client walks the NPC along: the last Copperbell boss going to the middle for 28760. `speed` is the ActorMove's speed byte (90 unless given; retail's walk is 20, its run 60): Haukke Manor's eye drifting through the last room's door at a walk. |
| `sweep_bnpc(npc, x, y, z, speed, delay, helper, action, every, cast_after)` | Sends an NPC (actor ID or layout ID) down a straight line to (x, y, z) at `speed` yalms a second, `delay` seconds from now: an ActorMove each tick to where it will be by the next (byte `speed` × 10), as retail's every 0.31 s, and its position along the line on the server (a single ActorMove far off, as `move_bnpc` sends, snaps the NPC there on the client). Without `action`, just a run (the Stone Vigil's first boss to its Swinge spots). `helper` (an NPC, or the walker itself if nil) uses `action` (instant) where the walker is every `every` seconds, from `cast_after` seconds on (`delay` if nil) until it arrives, put there first with an ActorSetPos. The way ahead of it is a danger the party gets out of while it moves (`server/sweep.rs`). Retail's Koshchei's whirlwinds and their helpers in the Stone Vigil. A new sweep of the same NPC replaces its old one; a wipe ends them all. |
| `party_spots(action, x, z, spots, react_low, react_high, inside)` | Where the party goes as an enemy casts `action` from near (x, z) (within 3 yalms; anywhere if nil): `spots = { tank = { x, z }, healer = { x, z }, dps = { x, z } }` (also `melee`, `ranged`, `dps2`), each member to its role's spot, caught or not, setting off `react_low`-`react_high` s after the cast begins (0.0-0.3 if nil), unless the spot is in an attack being cast (then by the shapes). With `inside` true the spots stand inside `action`'s own area by design (one that covers the arena and hurts less farther out: Titan's Geocrush) and only the other attacks being cast send a member to the shapes (2026-11-09). Set in `onSetup`: retail's Duty Support members' spots for a duty's big casts ([Party Bots](party-bots.md#how-it-fights)). The same action and place set again takes the new spots. |
| `party_positions(spots, seconds)` | Sends the party to `spots` (as `party_spots`') for `seconds`, whatever is being cast; nil calls it off (a wipe does too). The Stone Vigil's flight before the Sheets of Ice, and the whirlwinds' lanes. |
| `hold_bnpc(npc, held)` | Holds an NPC (actor ID or layout ID; `held` defaults to true): `npc_behavior` leaves it alone (no noticing, chasing, timeline or enmity on players), so it does only what the director has it do, as retail's boss helpers (`Instance::held_npcs`). |
| `fade_out_bnpc(npc)` | Fades an NPC (actor ID or layout ID) out, as retail does before taking away one that leaves without dying: ActorControl `FadeOut` (607) `[itself, 1, 0, 100]` from it; despawn it a moment later (retail's DeleteActor came 0.6-1.4 s after). Brotherly Love's beast and Claxio as "Duty Complete" played. |
| `retreat_bnpc(npc, x, y, z)` | The NPC gives up the fight and runs back to where it appeared, paying no attention to anyone (an open-world leash's return: enmity cleared every tick, its target dropped, no health back in content). Whoever was after it can chase it. Brotherly Love's beast as its friends fell. With `x, y, z` (since 2026-11-03) it runs there instead, along the navmesh (its home from then on): Haukke Manor's eye to the last room's door. A cast it was in the middle of is cancelled (CancelCast 537 `[537, 1, action, 0]`, retail's for the eye's 514). |
| `release_bnpc(npc)` | ActorControl 417 with no one (0xE0000000) from the NPC, as retail let each boss helper go the moment its boss died. |
| `set_model_scale(npc, row)` | Draws an NPC at another ModelScale row (ActorControl 48, `SetModelScale`), as retail grew the second Copperbell boss (16, 17, 18). |
| `actor_use_action(actor_id, action, target_actor_id)` | `use_action` by actor IDs; with no target the NPC casts on itself, as retail's helpers do (an area attack cast on oneself lands on everyone inside it the caster's battalion is at war with, `action.rs`). |
| `set_sequence(n)` | Sets the director's sequence and sends DirectorVars (its first byte is the sequence). The client's own battle script reacts to it (e.g. `OnSequence001` sets the duty objective). `0xFE` = lost, shows "Duty Failed". |
| `complete_quest_battle(quest_sequence, stay, finish_scene, finish_flags, variables)` | Wins: the quest moves to `quest_sequence` (with `variables`, a table of quest variable index 0–5 to value, in the same journal update: Lurkers in the Grotto's UI8BH, `{ [1] = 0x10 }`, as retail sets it; replay reads the battle script's `QUEST_VARS_AFTER_WIN` for its `battle_won` step), "Duty Complete" and the victory fanfare (60001) play (unless `fanfare`, the sixth argument, is `false`: Lord of the Inferno's battle goes straight into its cutscene, as retail's), then `finish_scene` (default 6, `QUEST_BATTLE_SCENE_FINISH`; retail plays 6 in Chasing Shadows and Leia's Legacy, 11 in To Guard a Guardian, 9 in Spirithold Broken) with `finish_flags` (default 0x2001, HIDE_HOTBAR and NO_DEFAULT_CAMERA, as retail plays all of them but Spirithold Broken's, a cutscene with 0xF8402EFB), and the players go back when it ends. With `stay = true` the players remain (for closing lines) until `leave_quest_battle()`. |
| `leave_quest_battle()` | Sends the players back where they entered. |
| `fail_quest_battle()` | Loses: players go back where they entered, the quest stays on the same step to retry. Retail sets sequence `0xFE` first and sends them out about 6 s later. |
| `set_hate(npc, target, amount)` | Gives NPC `npc` (a layout ID, or since 2026-10-27 an actor ID from `spawn_bnpc` for one of several sharing a layout ID, as Trial by Wind's two 4248510 wind sprites) enmity on `target` (another NPC's layout ID, or the player when `nil`). Used to make allies and enemies fight each other. |
| `use_action(npc, action, target)` | Makes an NPC use an action (with its cast bar and area aimed where the target stands). |
| `end_fight()` | The fight is over: every NPC still standing forgets whom it hated, sheathes, leaves battle and drops its target, the players leave battle (retail's packets the moment Victory in Peril's boss fell: ActorControl ToggleWeapon 0 and SetBattle 0, ActorControlTarget SetTarget 0), and none takes anyone on again (`Instance::fight_over`: not hits, heals or damage over time, nor anyone coming close). `complete_quest_battle` does it too, so no one fights on through "Duty Complete" and the closing scene. |
| `battle_talk(npc, name_id, text_id, icon, duration_ms, kind)` | The **battle talk window** (DirectorPopupMessage, opcode 162 on 7.57), as retail shows it: `text_id` is the content's text by ID (retail's IDs from a recording; one outside the content's range crashes the client), `name_id` the speaker's BNpcName, `icon` the portrait, 4000 ms unless given. It comes from `npc` if it's there. `kind` is the speaker's object kind, 2 (BattleNpc) unless given; 3 (EventNpc) makes `name_id` an ENpcResident, for a speaker who isn't in the fight (Victory in Peril's watcher). A name that isn't in the sheet `kind` points to crashes the client. A `talk` step passes its speaker's `kind`. |
| `play_scene(scene, flags)` | Plays one of the director's scenes for every player (event type GameProgress), e.g. To Guard a Guardian's cutscene, scene 6 with flags `0xF8402EFB`. The script hears `onSceneFinished` when it's over. |
| `despawn_bnpc(layout_id)` | Removes an NPC (layout ID, or an actor ID from `spawn_bnpc`) without it dying (no `onActorDeath`). |
| `spawn_tethered_bnpc(layout_id, to, channeling)` | Spawns a drop-in NPC tethered to NPC `to` (layout ID): its spawn's tether target and Channeling row, the line the client draws between them until the NPC dies. The Tam-Tara Deepcroft's shields to Galvanth (Channeling 1). |
| `warp_players(x, y, z, rotation)` | Moves every player in the instance through a loading screen to a position in it (PrepareZoning to the same territory, then ActorSetPos). |
| `set_notice(npc, notices)` | With `false`, an aggressive NPC no longer notices whoever comes close (players or allied NPCs, `AGGRO_RANGE`); it fights only when hit or sent after someone (`set_hate`). `true` restores it. Kept per layout ID for the instance (`Instance::unaware_npcs`). |
| `set_invulnerable(npc, invulnerable)` | Whether NPC `npc` takes damage (its `currently_invulnerable`, as a timeline's invulnerability timepoint sets it). The Tam-Tara Deepcroft's last boss while a shield stands. |
| `boss_room(boss, wall, line, place, triggers, arena)` | A boss arena whose boss isn't spawned with it (`spawn_boss(boss, wall, line, place, arena)` spawns and seals on the boss's own pull): sealed 15 s after the boss or any of `triggers` (layout IDs) goes for someone (logs 2012, 2013, the wall shown), opened when `boss` dies (both hidden, log 2014). The Tam-Tara Deepcroft's rooms, sealed by their pack's pull.Called for several bosses with the same wall, they're one room (since 2026-11-02): sealed by any one's pull, opened as the **last** of them dies (Haukke Manor's pair). `arena` (both, optional): the layout's range (an EventRange's or MapRange's layout ID: a cylinder, sphere or unturned box) that is the room's floor; from the pull until the boss is gone, party bots and party NPCs don't go off it to dodge (`dodge::arena`; since 2026-11-04, Brayflox's Longstop's last boss's terrace). A layout ID the zone hasn't got is logged and keeps nobody in. |
| `leave_boss_room(boss)` | A boss that leaves its room without dying (despawned, not killed): it no longer holds the room, which opens (wall and line hidden, log 2014) if none of the room's other bosses is left. Brayflox's Longstop's dragon (429), which holds the third room with its boss (`boss_room` for both with one wall) and flies off (2026-11-03). |
| `map_effect(index, timeline, state)` | DirectorMapEffect: a map effect to `timeline` and `state` (1 unless given; Brayflox's Longstop's puddles go to 4 with timeline 8 as they burst). |
| `map_effect_at(layout_id, timeline, state, otherwise)` | The same for the map effect on a layout object (a SharedGroup of the duty's LGB): its index is where the ID stands in the InstanceContent's ContentDirectorManagedSG (`GameData::get_map_effects`, the list the director's DirectorSetupMapEffects follows), or `otherwise` if it isn't there (Brayflox's Longstop's puddles.) |
| `spawn_eobj(eobj, position, args2)` | Spawns an event object of the zone's layout (at `position`, a table with x, y, z, if given), or, when the layout has none, one made from its EObj row alone (layout ID 0, the director's handler, radius 1: Haukke Manor's pair's key, as retail spawned it). `args2` is SpawnObject's, as retail sent it (EObj-specific: 1282 and 66818 for Haukke Manor's two dropped keys). |
| `warp_to_pop_range(actor_id, pop_range)` | Sends that player to a pop range of the duty, as a shortcut does (PrepareZoning to the same territory, then ActorSetPos): a gimmick of the script's own that warps. |
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
| `bnpc_control(npc, category, params)` | The same for an NPC (actor ID or layout ID): the Thousand Maws of Toto-Rak's last boss's ActorControl 31 and 30 as its tail comes off and 20, 30, 31 as the tail dies (what they do isn't known). |
| `set_targetable(npc, targetable)` | ActorControl `SetTargetable` (54) from the NPC (actor ID or layout ID) to everyone around: the Thousand Maws of Toto-Rak's room bosses, spawned untargetable (retail's display flags 0x8, kept in the drop-in), become targetable as their web burns. Since 2026-11-06 the party's bots leave an NPC made untargetable alone until it's targetable again (or brought into view, `appear_bnpc`): retail's Duty Support members let Titan go through his leaps and while his Heart stood (`Instance::untargetable_npcs`). |
| `attach_bnpc(npc, to, distance)` | Keeps an NPC (actor ID or layout ID) `distance` yalms in front of another, facing as it faces, wherever that one goes: an ActorMove whenever it has moved or turned (`director::keep_attached`), until either is gone or dead. The Navel's Heart of the Mountain, 2.3 yalms before Titan (retail moved it with him: Bob's Chronofoil captures of 2026-11-06). |
| `spawn_bnpc_part(layout_id, to, distance)` | Spawns a drop-in NPC as a part of another (`to`: actor ID or layout ID): BattleNpcSubKind Part with `to`'s actor ID in the spawn's `pet_id`, `distance` yalms in front of it facing its way, kept there as `attach_bnpc` keeps one; returns its actor ID. The client draws and targets a part where the other's model has it, not where the part stands: the Navel's Heart of the Mountain in Titan's chest, as retail spawned it (Bob's Chronofoil captures 650dbfd8, 0fd8bce0, 48aaf6fe). Spawned as an ordinary enemy and attached, its target ring stood on the ground before him (Bob's bundle 20261109-201914). Bots and actions treat a part as an enemy. |
| `lose_effect(actor_id, status)` | The status ends now on that actor (player or NPC): its expiry dropped, ActorControl LoseEffect from 0xE0000000 and the status list to everyone around (`effect::lose_effect_instance`). The Navel's Granite Gaol (292) as the gaol breaks. `gain_effect` (actor, status, param, seconds; 0 never ends) is shown to everyone since 2026-11-06 (ActorControl GainEffect and the status list: Ifrit's 775 wasn't). |
| `dismount_bnpc(npc)` | The NPC (actor ID or layout ID) gets off its mount where it is: ActorControl 930 (PlayDismountAnimation) with where it stands and faces packed as for a player's (x, y, then rotation and z), to everyone around, and its spawn on foot from then on. One that rides in has the drop-in's `mount` (CommonSpawn's current_mount): In the Eyes of Gods and Men's Haurchefant (mount 10), off as he speaks (since 2026-11-10). |
| `kill_bnpc(npc)` | The NPC (actor ID or layout ID) dies where it stands, its health at 0 (`actor::kill_actor`, so `onActorDeath` follows): the Thousand Maws of Toto-Rak's eggs, gone 2.1 s after their burst. A death a task causes has its own tasks run the next tick (the director's task list used to be cleared after each tick, losing them.) |
| `objective(done, next)` | The duty's objectives (InstanceContent's ObjectiveStart to ObjectiveEnd, as the Duty Information shows them): ContentDirector event 0x80000016 `[done, 1]`, then 0x80000015 `[next, 0]` if `next` is given (`ContentDirectorEvent::ObjectiveDone`, `SetObjective`; upstream's `UnknownDirector16`, `15` with one parameter). Retail's in the Thousand Maws of Toto-Rak, which counts its progress with them instead of DirectorVars. |
| `set_weather(id, fade, forced)` | The instance's weather (WeatherId to everyone, fading in over `fade` seconds; `forced` is the second byte, Kawari's `disables_override`): the Thousand Maws of Toto-Rak's last room, 19 (forced, 20 s) from its cutscene until the boss's death, then 2 (4 s). |

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
| `ally(layout, role)` | `focus`: the enemy groups it goes for, in order (`"any"` is everyone); `order` within a group: `oldest`, `newest`, `highest_hp`, `lowest_hp`, `nearest`; `range` (yalms, `set_attack_range`); `keep_near` (`{ anchor, within }`, `keep_near`); `heals` (another healer doesn't pick a target one has just started healing for `HEAL_PENDING_UPDATES`, 1.5 s: three healers on one timer all Cured the same ally at once, QB 38); `protect = false` for a tank that doesn't pull enemies off others (`Battle:provoke(director, tank, enemy, action)` has it take one at a set moment, above everyone's enmity, `action` used on it for the show); the rules are tried in order, and after any heal the NPC heals again only after its `heal_gap` (`HEAL_GAP_SECONDS`, 2.5 s, a cast's global cooldown; since 2026-11-10: each rule kept its own clock, so QB 60's enemy conjurer Cured and Cure II'd 1.5 s apart). |
| `enemy(layout, role)` | `group` (a name or a list, e.g. `{ "lostwings", "adds" }`); `attacks`: whom it goes for first (an ally, or the player when `nil`) with `enmity` (300); `passive = true`: no one at first, it waits to be hit (To Guard a Guardian's masked mage); it can have `heals` too (the sincallers heal Quzal). |
| heals | `{ action, targets (layout IDs, Battle.PLAYER, or "allies" for its whole side), below (%), every (s), say (a function) }`: heals whichever target is lowest under `below`. |
| `spawn(director, layouts, at)` / `remove(director, layouts)` | Brings NPCs in as their roles say (placement, first target) / takes them out without dying. `at` (`{ x, y, z }`, since 2026-11-07) spawns them there instead of their layout's spot: what quest battle 61's boss turns into, where he stood. A heal rule keeps its own wait, so two NPCs need a table each (QB 60's `enemyHeals()`). |
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
`QUEST_BATTLE_OPENING_FLAGS` in `main.rs`), 0xF8412FF7 for 14, 0xF8412EFB for 24, 0xF841AF0B for 29, 0xF841AFFF for 16 (two openings, 2026-10-26), and 54's is
no cutscene at all (0x2001, over in 0.1 s); a script with
other flags sets `OPENING_SCENE_FLAGS`. They hide the UI (0x800) and the other characters; Kawari used to send the
duties' opening flags (0x0C53A401, right for InstanceContent), and the UI stayed up with no skip button. Retail's
EventStart also has its flags byte set to 1 for the content directors' entering (0x0f) and progress (0x0d) events and
a quest's entering event, 0 for every other event (228 events across the captures); Kawari sends the same
(`event_start_flags`). Which of the two the skip button needs isn't known.

`!directorscene <scene> [flags]` plays any scene of the current director, which is how the victory fanfare scene
(60001, from the client's generic director script) was found.

## The battles

### Quest battle 19 — Ul'dah's Most Wanted (`content/questbattle/19.lua`)

* Quest 261 (ClsGla100_00261, the Gladiators' level 10 quest) enters it (scene 5, YesNoQuestBattle; territory 253,
  duty 314). Territory 253's TerritoryType has Central Thanalan's level (w1f2), so its NPCs are layer
  `QB_ClsGla100_002` of `wil_w1_planmap.json`, with the QuestBattle sheet's layout IDs and retail's levels, HP,
  positions, gear, display flags (0x40003) and character data flags (2 for the ally, 3 for the enemies) from Bob's
  Chronofoil capture a763f0a0 (2026-11-05, `--battle 0.5`, one run, won; a level 14 gladiator): the gladiator on your
  side (3977801, base 625, BNpcName 924, level 10, 211 HP), three thugs (3977803-3977805, base 626), then two more and a
  tougher one (3977807-3977809, 626 and 627), then their leader (3983981, base 628, 672 HP). The start spot (27.35, 13.0,
  51.79, facing -2.571) and the opening's flags (0x2001: no cutscene) are the run's.
* The flow (seconds from the fight starting; the director acted on a death **2 s after the health reached 0**, its
  death animation, all four times, as Lord of the Inferno's: `DEATH_NOTICED_AFTER`): sequence 2, the thugs on the
  gladiator; battle talk 107 (the gladiator, no portrait) at 5.9 s. **The second of the three down** (18.4 s): sequence 3,
  the next three 0.6 s later, on the gladiator, 108 10.1 s after. **All six down** (47.3 s): sequence 4 and 103 at once;
  5.0 s later sequence 5 and their leader, on the gladiator, who holds back (no target, `holdBack`) until **sequence 6**,
  5.1 s after 5; 104 2.0 s after that. **The leader down** (77.6 s): "Duty Complete", sequence 7, the battle's scene 7
  (0x2001) 3.6 s later; the quest moves to 5. `questkit battle-sim 19` matches all 15 events within 0.5 s (the win 1.1 s
  early: 2.1 s after the death that time).
* The gladiator fights up close (Fast Blade, Rampart and Fight or Flight on himself) and Cures himself at 25-35 %
  (retail's at 52 and 74 of 211), never you. He's on your side by his battalion (0), not his character flags (2, an
  enemy's bit on Kawari's reading): until 2026-11-05 your area actions hit him, he hated you for it and, the thugs
  down, turned on you (Bob's bundle 20261105-163414, [Combat](combat.md): sides). The thugs only auto-attack, the tougher one with a True Thrust; the leader
  uses Heartstopper (866) 8 s after he comes, then every 8.9 s. Timelines `QB19*` (bases 625-628).
* The client's battle script (`ClsGla100Btl_00261`, Bob's extracted scripts of 2026-11-05) has scenes 1 and 7 (both
  just END_EVENT_SCENE_RESULT), OnSequence001 (the objective) and OnSequence002 (its message); nothing reacts to
  sequences 3-7. The director plays only 1 and 7.
* The quest's two thugs (layer `QB_ClsGla100_001`, base 1245, level 10, 192 HP; timeline `ClsGla100Thug_1245`) were
  spawned on retail for the player by the gutless gladiator's event (their SpawnNpc names him), which is why
  `questkit chronofoil --dropin` left them out ("owned by someone").

### Quest battle 20 — That Old Familiar Feeling (`content/questbattle/20.lua`)

* Quest 262 (ClsGla150_00262, the Gladiators' level 15 quest) enters it (scene 40, Aldis' YesNoQuestBattle; territory
  254, duty 315). Territory 254 is a copy of Ul'dah - Steps of Nald: the run's positions stand on Steps of Nald's ground
  (the median height difference from the same capture's visits to 130 is 0.03 yalms; from Eastern and Central
  Thanalan's drop-in spawns nearby, 1.7 and 8.6), so its NPCs are layer `QB_ClsGla150_002` of `wil_w1t1_planmap.json`, with retail's layout IDs, levels, HP,
  positions, gear, display flags (0x40003) and character data flags (3) from Bob's Chronofoil capture 941895d2
  (2026-11-06, `--battle 0.7`, one run, won; a gladiator synced to 19): the ally (3993254, base 629, BNpcName 690,
  level 50, 5178 HP, the talk's speaker); five archers out from the start (3993614-3993618, base 669, level 17, 1845
  HP), on him; then four more on him (base 630, level 20, 1280 HP) and two on you (base 631, 223 HP); then three more
  of 630 on him and four on you (two of 632, two lancers of 670.) The start spot (28.64, 7.21, -100.73, facing 2.876)
  and the opening's flags (0xF841AEFB, Kawari's default) are the run's.
* The flow (seconds from the fight starting): sequence 2, talk 122 (the ally, no portrait) at 5.1 s. **The second of
  the first five down** (32.7 s; heard 2.0 s later, its death animation): sequence 3 and 123 at once. **30.1 s after
  sequence 3**, whatever the first five were doing (two were still up at 1 % and 12 %): sequence 4, the second wave
  0.45 s later; 10.1 s after that sequence 10 (sic) and 124, and 125 15.1 s later. **The fifth of the second wave
  down** (106.8 s): sequence 5 at once, not 2 s later (as Big Trouble in Little Ala Mhigo's second wave), the third wave
  0.44 s later, 126 5.1 s after 5. **Every enemy down** (162.0 s, heard 2.0 s later): "Duty Complete", sequence 6, the
  battle's scene 6 (a cutscene, 0xF8402EFB) 3.6 s later; the quest moves to 255. `questkit battle-sim 20` matches all
  25 events within 0.6 s.
* The ally fights up close, on whichever enemy last hit him (21 of his 23 target changes); auto-attacks of 229,
  Fast Blade, Savage Blade and Rage of Halone about every 32 s, Flash (14) every 9.9 s and Sentinel on himself. The
  archers on him only plink (12-42), so he holds the whole fight; the ones on you hit for 1-10. Timelines `QB20*`
  (bases 629-632, 669, 670) and `ClsGla150Trafficker_1244` for the quest's two.
* The client's battle script (`ClsGla150Btl_00262`, Bob's extracted scripts of 2026-11-06) has scenes 1 (CUT_SCENE_01)
  and 6 (the closing cutscene), OnSequence001 (the objective) and OnSequence002 (its message). The director plays
  only 1 and 6.
* Flash (14) had no script: `actions/000/Flash_00014.lua` gives status 15 at 236 to every enemy within 5 yalms (the
  Action sheet's area), as retail's did each time (how long isn't recorded: 12 s).

### Quest battle 21 — The Face of Thal (`content/questbattle/21.lua`)

* Quest 263 (ClsGla200_00263, the Gladiators' level 20 quest) enters it (scene 18, Mylla's YesNoQuestBattle in
  Western Thanalan; territory 255, duty 316). Territory 255 is a copy of Western Thanalan (the run's positions within
  0.02 yalms of the capture's visits to 140), so its NPCs are layer `QB_ClsGla200_001` of `wil_w1f1_planmap.json`, with
  retail's layout IDs, levels, HP, positions, gear and flags from the same capture (`--battle 0.14`, one run, won; a
  gladiator synced to 24). On your side: the one who speaks (4000991, base 638, BNpcName 689, level 50, 1941 HP) and
  four gladiators (4001010-4001013, bases 868-871, level 20-26). Against you: four lancers (633), three archers (634)
  and two gladiators (635); then three lancers, three archers and two conjurers (636); then three lancers, three
  thaumaturges (637) and their leader (4001144, base 872, level 25, 2554 HP.) Start spot (-239.7, 32.44, 395.6, facing
  2.354); the opening is no cutscene (0x2001, over in 0.2 s).
* The flow: sequence 2, talk 165 at 5.1 s. **The seventh of the first nine down** (42.5 s, heard 2.0 s later):
  sequence 3, the second wave 0.45 s later. **10.05 s after 3**: sequence 4, 166 and 167 5.1 and 10.1 s after it.
  **The sixth of the second wave down** (84.6 s): sequence 5 at once, the third wave 0.17 s later, 168 10.1 s after 5.
  **Their leader at half health** (124.7 s): sequence 6 and 169. **Their leader down**, the rest already (136.0 s): the
  win 0.3 s later, before his death animation (unlike 20's): "Duty Complete", sequence 7, the battle's scene 7 (a
  cutscene, 0xF8402EFB) 3.7 s later; the quest moves to 255. `questkit battle-sim 21` matches all 27 events within
  0.6 s.
* Who goes for whom, from the run: every lancer and their leader on the one who speaks, the gladiators on two of the
  four (then him), the archers on the four, him and you (Quick Nock's cone takes several), a conjurer and a
  thaumaturge each on you, the rest on the four; the director starts each on retail's first target. The one who speaks
  takes the lancers then their leader, the first two of the four the archers, the other two the gladiators. Heals
  (Cure, 120): the one who speaks on the four below 70 % (615 each; 3-66 % after the heal, every 6.3 s or so, never
  himself or you), the second of the four on him, the others and himself below 65 % (132), the conjurers on their own
  side below 60 % (125). Timelines `QB21*` (bases 633-638, 868-872).
* The client's battle script (`ClsGla200Btl_00263`) has OnSequence001 and 002 and scenes 1 and 7 only.
* The enemy gladiators' Shield Bash is the NPC one, 718 (`actions/007/ShieldBash_00718.lua`: a hit and stun, status 2
  at 0), and the conjurers' Fluid Aura 134 (`actions/001/FluidAura_00134.lua`: status 13 at 0, no damage), as retail's
  effects were; neither had a script. How long the stun and bind last isn't recorded (6 s).

### Quest battle 22 — On Holy Ground (`content/questbattle/22.lua`)

* Quest 264 (ClsGla250_00264, the Gladiators' level 25 quest) enters it (scene 6, Aldis' YesNoQuestBattle in Eastern
  Thanalan; territory 256, duty 317). Territory 256 is a copy of Eastern Thanalan (the run's positions within 0.02
  yalms of the same capture's visits to 145; nothing else in the capture comes near), so its one NPC is layer
  `QB_ClsGla250_002` of `wil_w1f3_planmap.json`, retail's from Bob's Chronofoil capture accc33a9 (2026-11-06, `--battle
  0.6`, one run, won; a gladiator synced to 29): a duel with a pugilist (4004491, base 640, BNpcName 698, level 25,
  2079 HP). The one watching (ENpcResident 1004411) isn't in the fight: its lines go as an EventNpc's (kind 3, layout
  0), as Victory in Peril's watcher's. Start spot (-518.39, -21.6, 120.48, facing 2.334); opening flags 0xF8412EFB.
* The flow: sequence 2, talk 124 (the watcher) at 5.1 s. **Him at 80 %** (12.5 s): sequence 3, 127 (him) 2.5 s and 126
  (the watcher) 10.1 s after. **Him at 40 %** (24.6 s): sequence 4, 128 (him) 3.1 s, 125 (him) and 129 (the watcher)
  8.0 s after. His Second Wind (57) took him from 33 % back to 47 % at 29.6 s, so each threshold goes once. **Him down**
  (48.7 s, heard 2.1 s later): "Duty Complete", sequence 5, the battle's scene 5 (a cutscene with its menu, 0xF8482EFB)
  3.5 s later; the quest moves to 255. `questkit battle-sim 22` matches all 11 events within 1.1 s (the 80 % crossing
  falls between two of retail's samples.) The talk runs on timers of the script's own, not `Battle:steps`, which
  queues each step after the last: a quick kill would have waited for the talk.
* His timeline (`QB22Pugilist_640`): auto-attacks of 25, Fists of Earth (60), 55, Internal Release (59) and Second
  Wind at 4.1, 10.7, 13.3 and 26.5 s from his pull, one 84 hit (865), Hundred Fists (829, a 5 s cast) at 40.1 s. 59,
  60 and 829 had no scripts (`actions/000/QuestBattleNpc_00059.lua`, `_00060.lua`, `actions/008/QuestBattleNpc_00829.lua`:
  statuses 100, 104 and 291 on himself, as retail's effects; how long each lasts isn't recorded).
* The client's battle script (`ClsGla250Btl_00264`) has scenes 1 (the opening), 5 (the closing cutscene with a menu)
  and 10001, OnSequence001 and 002. The director plays 1 and 5.

### Quest battle 23 — The Rematch (`content/questbattle/23.lua`)

* Quest 265 (ClsGla300_00265, the Gladiators' level 30 quest) enters it (scene 11, after ACTOR2's NpcTrade in Eastern
  Thanalan; territory 257, duty 318). Territory 257 is a copy of Eastern Thanalan too (the run's positions on its ground,
  median 0.00 yalms off over 424 points), so its NPCs are layer `QB_ClsGla300_003` of `wil_w1f3_planmap.json`, all 43
  of retail's (`--battle 0.15`, one run, won; a gladiator synced to 34). Out before the opening cutscene (the script's
  `onSetup`), as retail's 39:
  * along the way up from the start, pairs that take the player on as they come close (`passive`, aware from the fight
    starting): a lancer (658), two conjurers (659), a pugilist (660), two archers (661), two thaumaturges (662);
  * at the camp above, waiting (`unaware`) for sequence 5: four pugilists (664), a lancer (663) and a conjurer (687)
    against the three on your side (4011016, base 665, BNpcName 689, who speaks; 666; 667), held back till then;
  * two champions, yours (4011120, base 657, BNpcName 690) and theirs (4011122, 1324), level 50 and 258900 HP, fighting
    each other only, from sequence 5;
  * twenty soldiers in ten pairs (yours 1335 and 1336, theirs 1338-1340; level 15, 191400 HP, chipping 15-40 a hit),
    each on its own one of the other side's from 1-2 s into the fight, the whole fight: none falls, and none counts.
  Then the last wave: three more lancers (663) on the one who speaks and a thaumaturge (656) on you.
* The flow: sequence 2, 145 (your champion) at 5.1 s. **The player at the third pair** (20.1 s, within 6 yalms of
  either; they came at the player there): sequence 3. **The player at the archers below the camp** (36.2 s, within 9
  yalms): sequence 5 and 146 (the one who speaks) at once; the camp and the champions fight. Both are where retail's
  player was (no kill or timer fits them; the director's sequence 4 never came); the battle-sim fixture holds the run's
  player trail (`positions`) to check them. **The conjurer down** (94.0 s): its 152 at once, as its health reached 0;
  sequence 6 2.4 s later, the last wave 0.35 s after that, 147 10.05 s after 6. **Every one that counts down** (177.6 s,
  0.05 s later: no death animation waited for): "Duty Complete", sequence 8, the battle's scene 8 (a cutscene,
  0xF8402EFB) 3.1 s later; the quest moves to 255 with UI8B 0 (`QUEST_VARS_AFTER_WIN`: its two key items gone, as
  retail's UpdateQuest). `questkit battle-sim 23` matches all 14 events within 1.1 s.
* Heals (Cure, 120), from the run: the one who speaks on himself and the other two below 40 % (6-36 % before the heal,
  every 6.9 s), the third on him and the second below 45 % (31-44 %, every 7.6 s), the conjurer its side below 30 %
  (0-29 %, every 5.9 s). Timelines `QB23*` (bases 656-667, 687, 1324, 1335-1340). The second gladiator's Awareness
  (29167) had no script (`actions/291/QuestBattleNpc_29167.lua`: status 3285 on himself); their champion's one Provoke
  (18) is left out of its timeline (no script; the director's targeting does what it would).
* The client's battle script (`ClsGla300Btl_00265`) has scenes 1 and 8 and OnSequence001 and 002 only.

### Quest battle 86 — History Repeating (`content/questbattle/86.lua`)

* Quest 967 (GaiUsc208_00967) enters it from the campfire (EOBJECT1: scene 8, EVENT_ACTION_WAITING2_MIDDLE's 5 s bar,
  scene 9's YesNoQuestBattle; territory 330, duty 379). Territory 330 is taken to be a copy of Western La Noscea: the
  run's spots stand on its ground (-42 under foot, 13 yalms from one of that zone's open-world enemies in `sea_s1f4`),
  so its NPCs are layer `QB_GaiUsc208_001` of `sea_s1f4_planmap.json` (21 NPCs from `questkit chronofoil --dropin
  330`, all `nonpop`), with retail's layout IDs, levels, HP, positions, display flags, character data flags (3, and 1
  for the two below) and looks from Bob's Chronofoil captures d7993cf0, 195176f8 and 0d10bf45 (2026-11-08,
  `--battle 0.1` of each: three runs, all won, level 44 players). Start spot (-305.5, -41.82, 697.34, facing 0) and
  the opening's flags (0xF8402EF3) are the runs'.
* No ally fights beside you. **The one who talks** (4294351, base 1632, BNpcName 1417, level 50, 258900 HP,
  battalion 4, display flags 0x40008: untargetable) stands on the rise behind the enemies the whole fight; all eight
  battle talks are its (portrait 73188) and so are the four NpcYells. **The director's helper** (4317701, base 2201,
  0x40208: unseen) stands beside you; the one who talks throws 1329 at it once (a miss) and faces it from sequence 2
  to 7. Both are out from `onSetup` (retail's spawned before the opening) and held.
* **A lead-in before the fight**, as The Lominsan Way's: the director's scene 2 (0x2001, 10.1 s; its client script
  waits 300 frames) with sequence 2, talk 81 5.1 s in and the throw 9.1 s in; scene 3 (0x2001, 270 frames) with
  sequence 3 and talk 82 5.1 s in; scene 60000 1.0-1.1 s after scene 3 ended. Kawari starts the fight as scene 3
  ends (`onSceneFinished`; the second's wait is left out, see below.)
* **Five waves**, each brought in when the one before is down, the same in all three runs (seconds from the last
  one's health reaching 0, which is the next sequence): sequence 4 and the first (a lancer, two marauders) 1.4-1.5 s
  after the fight starts, NpcYell 1321 at 4.1; sequence 5, the second (lancer, thaumaturge, marauder) at 2.2-2.3,
  talk 83 at 5.1, NpcYell 1342 at 10.0; sequence 6, talk 84 at 5.1, NpcYell 1343 at 10.1, the third (lancer,
  marauder, two base 2200) at 15.3-15.5; sequence 7, talks 85 and 86 at 5.0 and 10.0, the bombs from 10.2-10.5, the
  fourth (lancer, thaumaturge, marauder) at 15.4; sequence 8, talk 87 at 5.1, NpcYell 1344 at 10.0, the last (2202,
  2683 HP, and a lancer and a thaumaturge who run in from behind) at 15.3-15.5; sequence 9 when it's down, talk 88
  2.1 s later, "Duty Complete" and sequence 10 7.1 s after sequence 9, the battle's scene 10 (0x2001) 3.6 s after
  that; the quest moves to sequence 4 with ITEM0 gone (`QUEST_VARS_AFTER_WIN`), the director to 254. The waves' timers
  are 15.0 s where retail's waves came at 15.3-15.5: a timer set at a death fires up to half a second late on the
  tick, and battle-sim drifted 2.0 s by the end of the first run with 15.5 (`questkit battle-sim 86`: 139 events in
  three runs, within 0.8 s.)
* **The one who talks heals its side** (1070: 5.7 s, an area aimed at one of them, 654 a heal, 630-978, and Regen
  158 for about 18 s, 38-58 a tick) every 14-38 s while sequences 4-6 last, never after: `Battle`'s heal rule on its
  side below 80 %, every 14 s, dropped at sequence 7. One cast healed three of its side (195176f8): an NPC's area heal
  aimed at one of its own side now heals everyone of that side inside it ([Combat](combat.md)).
* **The bombs** (sequence 7 on: one every 10 s from 10.2-10.5 s after it; from sequence 8 the clock starts again, one
  8.3-8.5 s after it and every 8 s, the last wave's arrival not counting; none after sequence 9). Which of three
  layouts comes is random (11, 11 and 16 bombs in the three runs, each layout often). Nobody can target them (display
  flags 0x40008, 0x4000B). The two of base 1633 (4317589 and 4317648) wait 5 s, then run a loop each round the fight
  at 9 yalms a second (ActorMove speed byte 90, movement type 2), the same loop every time (a patrol path of the
  layout's that the client never sees; Kawari has retail's corners, from its positions every 0.3 s: 13 s and 22 s
  around) and blow (1069, 200-243 on the player) when they pass within 3.2-3.8 yalms of you; 1634 (4317650) waits
  5 s and comes for you at 1 yalm a second (speed byte 10), blowing within 3.9-5.3 yalms. Each falls 1.0 s after its
  blast (ActorControl Kill, by itself; Defeated 3.1 s after the blast). Kawari: the director spawns and holds them,
  moves them with `sweep_bnpc` (one line to each next corner; the creeper towards wherever you are, every tick),
  tracks where each is itself (one layout can be out several times at once, and `info` has one entry a layout), and
  has each blow (`actor_use_action` on you) and fall (`kill_bnpc`.)
* Their fighting (timelines `QB86*`, from their pull): lancers (1647) auto-attack 871 and use Heartstopper (866) every
  10.1 s, Life Surge and 477 on themselves early; marauders (1649) 870 with Heavy Swing, Fracture (33: a damage over
  time, status 244, 18 s), 35, 34 on themselves and 47 (heals itself 4); thaumaturges (1648, 227 HP, who never
  auto-attacked) Fire III, Fire, Scathe, Thunder II, Fire II, Manaward, Blizzard and 1304 (round itself: Bind, 13);
  the third wave's pair (2200) 870 and 1330 (389) every 20.4 s; the last wave's leader (2202) 870 and Hell Slash (341)
  every 8.3 s. Recorded amounts are the median damage to the player. New action scripts: `QB86Bomb_01069`,
  `QB86Talker_01070`, `QB86Talker_01329`, `QB86Enemy_01330`, `QB86Thaumaturge_01304`, `QuestBattleNpc_00033`,
  `QuestBattleNpc_00047`.
* The client's battle script (`GaiUsc208Btl_00967`, Bob's extracted scripts of 2026-11-08) has scenes 1 (the opening),
  2 and 3 (the lead-in's waits) and 10 (empty), OnSequence001 (no music), OnSequence004 (a message) and its objective
  (TODO001 before sequence 4, TODO002 from it).

### Quest battle 62 — Notorious Biggs (`content/questbattle/62.lua`)

* Quest 983 (GaiUsc408_00983) enters it from ACTOR2 (Cid, scene 3, YesNoQuestBattle; territory 304, duty 351).
  Territory 304 is taken to be a copy of Coerthas Central Highlands: retail's start spot is where it let the player out
  there afterwards (-499.6, 287.1, 166.9), so its NPCs are layer `QB_GaiUsc408_001` of `roc_r1f1_planmap.json` (17 NPCs
  from `questkit chronofoil --dropin 304`, `nonpop`, character data flags 3), from three runs: Connor's Chronofoil
  capture e834e436 (`--battle 9.53`, won, level 47) and Bob's 689b15da (`0.2` lost, `0.4` won, level 45). Start
  (-498.89, 286.89, 167.44, facing -2.512) and opening flags (0xF8412FFF) the same in all three.
* **Allies**: one to keep alive (4264070, base 1656, 5109 HP), a level 50 fighter (4264069, base 1650: holds the
  boss, heals itself at 40-47 %) and a level 50 conjurer (4264062, base 1651: Cure at 40-57 %, 129 on someone every
  20 s). **Enemies**: the boss (4264130, base 1652, 12038 HP, heals itself at 43-49 %) and three archers from the
  start; a second wave of six and a third of four. Each enemy's first target is retail's (the same in every run).
* **Flow** (seconds from the fight starting): sequence 2, talks 97-100 (the four sides' speakers) at 5.1, 10.1, 15.1
  and 20.1; the three archers down, or the boss under 50 % first (one run): sequence 3 and the second wave, talks 101
  and 102 5.1 and 15.1 s after; **five** of the boss and the later waves down (which five differed between runs, so
  it's a count): sequence 4 and the third wave, talks 103 and 104 5.1 and 10.1 s after; **seven** down: sequence 5 and
  talk 105; everyone down: "Duty Complete" and sequence 6 2.1 s after the last one's health reached 0, the battle's
  scene 6 (0xF8402EFB); the quest moves to 255. The one to keep alive under 20 %: NpcYell 1530; its health at 0: talk
  108 and sequence 0xFE (the lost run, 90.8 s in). `questkit battle-sim 62`: 65 events in the three runs within 1.4 s
  (fixtures `62.json`, `62-689b15da.json`, `62-lost.json`).
* Timelines `QB62*`; new action scripts `QuestBattleNpc_00064`, `QuestBattleNpc_00129` (status 151, 30 s: its end not
  seen), `QB62Boss_01068`, `QB62Enemy_00527` (status 17, 10 s: likewise). The battle's client script
  (`GaiUsc408Btl_00983`) has scenes 1 and 6 and OnSequence001-002, as played.

### Quest battle 64 — Every Little Thing She Does Is Magitek (`content/questbattle/64.lua`)

* Quest 1005 (GaiUsc605_01005) enters it from ACTOR6 (scene 44, YesNoQuestBattle; territory 309, duty 358).
  Territory 309 is taken to be Mor Dhona's copy as 63's (its spots are within 50 yalms of 63's), so its NPCs are layer
  `QB_GaiUsc605_001` of `lak_l1f1_planmap.json` (6 NPCs, `nonpop`; character data flags 3, 1 for the ally the enemies
  wear down), from one run: Connor's Chronofoil capture e834e436 (`--battle 9.202`, won, level 48). Start (-78.43,
  1.89, -627.73, facing -1.159), opening flags 0xF841AEFB.
* **Allies**: one who fights (4277164, base 1876: 870, and 1170 every 6.6 s) and one the enemies wear down (4277157,
  base 1877, 11193 HP, never seen acting). **Enemies**, all out from the start: the boss (4277127, base 1958: 526 and
  525, an area that caught both allies, every 20 s) and three others (4277134-4277136; 4277136 cures itself at 6 %).
  Two event NPCs talk (1007075 and 1007076: kind 3, no actor, as Victory in Peril's watcher).
* **Flow**: sequence 2, talks 122 (4277136) and 123 (the one who fights) at 5.1 and 10.1; the worn-down ally under 90 %:
  talk 124; 4277136's health at 0: sequence 3; the boss's: sequence 4, talks 125 and 128 1.2 and 5.0 s later;
  everyone down: "Duty Complete" and sequence 5 2.1 s later, the battle's scene 5 (0xF8402EFB); the quest moves to 255.
  No lost run: only your health at 0 loses it. `questkit battle-sim 64`: 10 events within 0.3 s (`64.json`).
* Timelines `QB64*`; new action scripts `QuestBattleNpc_00091` (status 119, 18 s: its end not seen) and
  `QB64Ally_01171`. The battle's client script (`GaiUsc605Btl_01005`) also has scenes 10001 and 10002, never played.

### Quest battle 63 — Fool Me Twice (`content/questbattle/63.lua`)

* Quest 1004 (GaiUsc604_01004) enters it from EOBJECT0 (ITEM0 used on it, scene 12, YesNoQuestBattle; territory 308,
  duty 357). Territory 308 is taken to be a copy of Mor Dhona: its spots are a few yalms from where the item is used
  in Mor Dhona and where retail let the player out (-129.6, -2.7, -629.2), so its NPCs are layer `QB_GaiUsc604_001` of
  `lak_l1f1_planmap.json` (10 NPCs from `questkit chronofoil --dropin 308`, all `nonpop`, character data flags 3 as
  retail's SpawnNpc), with retail's layout IDs, levels, HP, positions, display flags and looks from Connor's Chronofoil
  capture e834e436 (2026-11-07, `--battle 9.195` lost and `9.197` won; a level 47 player). Start spot (-127.58, -2.66,
  -630.68, facing -0.719) and the opening's flags (0xF8412EF3) are both runs'.
* **The ally to keep alive** (4270049, base 1987, 10560 HP) goes for whoever is after you, the adds before the boss,
  and heals you or itself at 15-48 % (964, 395-412, every 5.7-7.2 s): a tank (`protect`) with that heal. Its weapon
  skill 1170 (76-95) every 6.3 s. The boss (4270036, base 1988, 6540 HP) swings 1172 every 2 s instead of an
  auto-attack and casts 1204 (2.7 s, where its target stood) every 15.7 s; under 40 % it casts 1286 on itself (status
  61). The first wave is out from the start beside him (a gladiator, two lancers, two archers, who took the ally on at
  once); the second (an archer, a lancer, a conjurer who cured the boss at 39 %) comes as the first is down.
* **Flow** (seconds from the fight starting): sequence 2, talks 101 and 102 (the ally) at 5.1 and 10.1; the first
  wave's last one's health at 0: sequence 3 4.3 s later with the second wave, talks 103 (the second wave's lancer) and
  104 5.1 and 10.1 s after; the boss under 40 %: sequence 4, sequence 5 5.5 s later, talks 105 and 106 6.1 and 11.9 s
  after it; the boss's health at 0: "Duty Complete" and sequence 6 2.1 s later, the battle's scene 6 (0xF8402EFB); the
  quest moves to 255 with ITEM0 (UI8BH) gone (`QUEST_VARS_AFTER_WIN`). Lost: the ally's health at 0 (talk 107 and
  sequence 0xFE at once, 69.6 s in, the lost run) or yours. `questkit battle-sim 63`: 19 events in both runs within
  0.7 s (fixtures `63.json`, `63-lost.json`).
* Timelines `QB63*` (median damage to the player and the other side); new action scripts `QB63Ally_00964`,
  `QB63Ally_01170`, `QB63Boss_01172`, `QB63Boss_01204`, `QB63Boss_01286` (1286's status length not seen: 30 s). The
  battle's client script (`GaiUsc604Btl_01004`, Bob's extracted scripts of 2026-11-09) has scenes 1 (the opening) and
  6 (the finish) and OnSequence001-002 only, as played.

### Role statuses

As a quest battle begins (`onCommence`, with the director's scene 60000) retail gives the player a status for their
role, in StatusEffectList alone (no ActorControl GainEffect), from no one, with no end: tanks 1264, healers 1041,
others 1265 (seen in all 34 quest battles of Bob's captures, levels 6-45). Kawari does too since 2026-11-10
(`director::give_quest_battle_role_status`); the tanks' and healers' add 30 % to their damage
([Combat](combat.md)). It goes with the instance (statuses start empty in each zone).

### Quest battle 60 — In the Eyes of Gods and Men (`content/questbattle/60.lua`)

* Quest 912 (GaiUsb803_00912) enters it (ACTOR2's YesNoQuestBattle, scene 4, or EVENTRANGE0's, scene 18; territory
  301, duty 343). Territory 301's TerritoryType has Coerthas Central Highlands' level (r1f1), so its NPCs are layer
  `QB_GaiUsb803_001` of `roc_r1f1`, with retail's layout IDs, levels, HP, positions, flags, looks and the start spot
  (457.38, 303.03, -264.67, facing 2.673) from Bob's Chronofoil captures 130088ec and 3c56e09f (2026-11-06,
  `--battle 0.9` and `0.7`) and 21f967d2 (2026-11-07, `--battle 0.9`): one run on each of his three characters, all
  won; level 40. The third run's NPCs spawned at the same spots with the same levels and HP. On your side a conjurer (4261257,
  base 1739, 3787 HP) and, from sequence 3, a gladiator (4261300, base 1738, 4257 HP); against you the one you fight
  (4261262, base 1735, 9150 HP), an archer (1733) and a conjurer (1732), then a lancer (1736) and a pugilist (1734),
  then a marauder (1731), another conjurer (1732, 4287237) and a **wyvern** (4261305, base 1737, 15150 HP, battalion
  5: at war with both sides, and invulnerable: every hit on it was effect 7, `set_invulnerable`). An onlooker (event
  NPC 1007071, 4261248) talks and yells; talking to him plays scene 10001 (retail's, three times).
* The flow (seconds from scene 60000), the same in all three runs: sequence 2 and the onlooker's yell 1030; talks 85 (the
  conjurer, name 2081, portrait 73194) and 86 (the boss, 1454, 73195) at 5.0 and 10.0. The boss under **80 %** (78.7,
  78.4): sequence 3; the gladiator 4.2-4.4 s later; sequence 4 and talk 87 5.0 s after it; the second wave 6.4-6.5 s
  after sequence 4; sequence 5 10.0 s after sequence 4 with talk 88 (the gladiator, 2082, 73124), then 89 (the
  onlooker, ENpcResident 1007071, 73193, kind 3) 10.1 s later and yells 1031 (the conjurer) and 1032 (the onlooker)
  15.1 and 20.1 s later (they came even when sequence 6 had come first: 7.4 s after sequence 5 in the third run). The
  boss under **50 %** (48.3, 49.4, 47.7): sequence 6, talk 95 (the boss) 1.0 s and yell 1033 (the
  boss) 6.0 s later, the third wave 8.3-8.4 s later, talks 90 and 91 18.0 and 23.0 s later. Under **20 %**: the
  gladiator's yell 1034. Health at 0: sequence 7 and the boss's talk 92 at once. An enemy still up then fights on (the
  third run's marauder, killed 8.5 s later); nothing waits for it.
* **The rosary.** 0.2-0.5 s after sequence 7 an event object appears (EObj 2002279, layout 4285063, at (458.88,
  304.81, -279.59) both times, not where the boss fell), its handler the director (SpawnObject). The director's log
  3206 5.0 s after sequence 7; the wyvern runs off at once (0.2 s after sequence 7, a straight line at 5.9 yalms a
  second from wherever it was to (442.25, 302.42, -324.64), the same spot within half a yalm in all three runs), fades
  (FadeOut 607) as it gets there 6.4-7.0 s after and goes 1.35 s later (since 2026-11-10 `hold_bnpc` and `sweep_bnpc`
  at 6; Kawari faded it where it stood, Bob's bundle 20261110-102239); log 3209 10.05 s after
  (both runs), and if the rosary is still there then sequences 8 and 9 at once and the gladiator's yell 1371 12.1 s
  later. Talking to it (4.5, 24.8 and 17.2 s after sequence 7): EventStart (Talk, the director's event), **no scene**,
  the director's event action 22 at once (ActorControlTarget 3 `[1, 22]`, 5 s); as it ends, EventFinish and the bar's
  end (ActorControlTarget 3 `[0, 22]` on the rosary, which Kawari sends for every director's event action since
  2026-11-07, as for a quest's), sequence 10, the rosary fades (607) and goes 1.1 s later, log 3208; 3.0 s later "Duty Complete" and
  sequence 11, then the battle's scene 11 (0xF8402FFF). Kawari: `onObjectTalk` (above) and `onEventActionCast`.
* The boss's health stood at 50.1 % for 12 s in one run (the player was on the adds), which an interpolated health
  curve crosses early: its fixtures hold health (`npc_health: held`). The opening scene 1 is a cutscene (0xF8412FFF).
* **The enemy conjurers' heals** (the three runs' battle exports, each Cure and Cure II with its target's health as
  the cast began): one heal every 5.6-8.3 s, never closer, Cure II on one of their side under about 23 % (6-22 %),
  else Cure under 50 % (31-50 %); Cure healed 444-468, Cure II 695-730. Kawari's healed as much but each rule had its
  own clock, so a Cure II came 1.5 s after a Cure (Bob's bundle 20261110-091329, "heals twice in quick succession");
  since 2026-11-10 they share one, `heal_gap` 5.6 s. In that run (the damage formula and role status in) the first
  conjurer fell 81 s after it came, where retail's went in 15-35 s.
* **Who fights whom, against Kawari's play of it** (Bob's bundle 20261110-095652, "far more difficult": the
  gladiator fell at 202 s, after 10,047 damage where retail's took 2,547-4,826 in a whole fight). Retail's first wave
  archer shot the conjurer ally from the start (5-12 shots), and the conjurer ally went for it first; the gladiator
  Provoked (18) the boss once, 2.0-9.6 s after he began fighting, and pulled nothing else: the second wave's archer
  and the wyvern stayed on the player (the wyvern's 12-18 auto-attacks all on the player in two runs). Kawari's archer
  shot the player; Battle's tank `protect` pulled an enemy off the player every 10 s, so the archer and the wyvern
  fought the gladiator (the wyvern's 13 auto-attacks on him); hits on the invulnerable wyvern still angered it; and
  their auto-attacks on him, recorded only against players, went unscaled (229 and 240 where the player took 96 and
  85). Since 2026-11-10: the archer `attacks` the conjurer ally, who focuses it first (`archer` group); the gladiator
  doesn't `protect` but Provokes the boss 2.5 s after he arrives (`Battle:provoke`); an invulnerable NPC isn't
  angered by hits (`server/action.rs`); and an NPC with no recorded amount against NPCs hits them as it hits players
  (`Timeline::recorded_amount_at`).
* **The wyvern's 1136 hits everyone** (Bob's bundle 20261110-102239): on retail it caught whoever stood in it, both
  sides: the boss 3-6 times (1308-2642 of its 9150 HP), the third wave's marauder and conjurer, the allies and the
  player, 7-10 casts while it was out. Kawari's caught only the player's side, so the boss lost nothing to it, the
  last phase ran longer (14 casts) and the gladiator took 12 of them (3854 of the 6001 he took; 4-5 on retail). An
  NPC's area attack now lands on everyone its battalion is at war with ([combat](combat.md)).
* **Haurchefant rides in** (all three runs, to a yalm): retail's SpawnNpc had him on mount 10, 4.3-4.4 s after
  sequence 3, where the drop-in puts him; he runs (6 yalms a second, speed byte 60) along his facing to (463.1, 304.3,
  -275.1) in 7.6-7.8 s, turns towards the fight and runs on to (459.5, 304.5, -307.0), getting off his mount on the way
  as he says talk 88 (ActorControl 930 with where he was, 10.8-11.0 s after he appeared), and fights from 13.2-13.4 s.
  Kawari brought him out on foot where the drop-in has him and let Battle walk him to the boss (Bob: "he rode in on a
  horse and walked through the battle field before dismounting"). Since 2026-11-10: the drop-in's `mount`, two
  `sweep_bnpc` legs while he's held, `dismount_bnpc` with talk 88, let go at 13 s.
* **Haurchefant** (the gladiator, 4261300) falling loses the battle, as yours does (Bob, from retail; no retail run
  lost): Kawari fought on in that run and the player fell 6 s later.
* Retail's boss Cures itself once under about 15 % (its first 0.7-8 s after crossing it; 3.6-7.6 s apart); the enemy
  conjurers Cure their side under 50 % (every 5.6-21.7 s) and Cure II under about 25 % (the third run's, which lived
  long enough: 6-23 %, every 13.7 s); yours Cures the gladiator, you and itself at 31-79 % (every 16.7-23.4 s) and
  once Cure II on the gladiator at 25 %. Retail's enemy conjurers' Cures healed 435-475 (the first wave's took itself
  from 2 % to 53 % in one) and Cure II 712-737, as Kawari's do. Your conjurer attacked every 7.6 s (the median gap of
  74 attacks over three runs; its timeline's pace since 2026-11-10, when Bob's bundle 20261110-081839 had it at Stone
  II's own 13.2 s, half as often). The third run's marauder used Rive (1135, two at a time) too. Both allies went for the
  newest enemy, and for the wyvern while it was out. Timelines `QB60*` (1731-1739).
* Fixtures: `60.json` (130088ec), `60-3c56e09f.json` and `60-21f967d2.json`, each with the onlooker added, held health and the rosary's
  `object_talks` (retail's StartTalkEvent on it, [questkit](questkit.md#battle-sim)).

### Quest battle 61 — The Heretic among Us (`content/questbattle/61.lua`)

* Quest 941 (GaiUsc004_00941) enters it (ACTOR1's YesNoQuestBattle, scene 2, or EVENTRANGE0's, scene 9; territory
  302, duty 344, also r1f1's level): layer `QB_GaiUsc004_001` of `roc_r1f1`, retail's layout IDs, levels, HP,
  positions, flags, looks and the start spot (-865.98, 226.1, 14.44, facing -1.832) from the same captures
  (`--battle 0.11`; `0.13`, `0.15`, `0.17`; 21f967d2's `0.13`: five runs, three won, one lost as the gladiator fell,
  one as the player did; level 41). On your side a **gladiator you must keep alive** (4256801, base 1750, 2401
  HP), a conjurer (4256802, 1751) and an archer (4256803, 1752); against you the one you fight (4256791, base 1746,
  17030 HP), three adds (1748, 4256796-4256798), what he turns into (4291248, base 1747, 12772 HP), a wyvern (4256804,
  1749) and one more add (4292887).
* The flow (seconds from scene 60000), the same in all five runs: sequence 2; talks 75 (the gladiator, 2085, 73189) and
  76 (the boss, 2083, 73193) at 5.1 and 10.1; sequence 3 at 17.4-17.5, the three adds 0.2-0.5 s later. The boss under
  **50 %** (49.2-49.7): sequence 4, whether the adds are down or not (one was up in the fifth run), talks 77, 78 and 79 0.1, 5.1 and 10.1 s after. **Dragon's Blood**: 11.1-13.0 s
  after sequence 4 the boss casts 1162 (3.7 s) on himself, whatever his health (31-46 %: not a health mark), and
  sequence 5 comes 4.4-4.5 s after the cast began; he fades (607) 1.1 s later, what he turns into appears 1.3-1.5 s
  after sequence 5 (0.6 once) **where he stood** (0-5.5 yalms from its layout's spot, five runs: `Battle:spawn`'s `at`,
  his last position from `info`) and sequence 6 0.7 s after that as the boss is taken away (DeleteActor); talks 80 (it,
  2084, 73190) and 81 4.0 and 8.0 s after sequence 6. It under **75 %**: sequence 7, the wyvern 0.2-0.5 s later,
  talks 82 and 83 5.1 and 10.0 s after; under **40 %**: sequence 8, the last add 0.2-0.4 s later; under **20 %**: talk
  84. It, the wyvern and the last add down: "Duty Complete" and sequence 9 2.0-2.1 s after the last one's health
  reached 0 (with its death animation, not at once as in The Face of Thal), the battle's scene 9 (0xF8402EFB) after
  it. The archer's and the conjurer's health at 0: their last words, 87 and 86 (name 2086; the archer fell in every
  run). Lost: the gladiator's health at 0 (his talk 85 and sequence 0xFE at once) or yours (0xFE).
* The director casts Dragon's Blood (`use_action`, 12.0 s after sequence 4) and does the turning; the boss's timeline
  (`QB61Boss_1746`) has only his spells (Ruin, Ruin II, Bio, Miasma, Virus, Rotten Meat at 13.4 s); his Eye for an
  Eye and Rouse went on his own adds, which a timeline can't aim at, so they're left out. The adds' Stone Gaze (499)
  petrifies (status 1, 9 s) whoever faces it, and a petrified NPC does nothing (`modifiers::INCAPACITATED`).
* Fixtures: `61.json` (130088ec, won), `61-3c56e09f.json` (won), `61-21f967d2.json` (won), `61-lost-13.json` (the
  gladiator) and `61-lost-15.json` (you), all with held health.

### Quest battle 75 — Seer Folly (`content/questbattle/75.lua`)

* Quest 1079 (JobWhm300_01079, the White Mage's first quest) enters it (scene 2, YesNoQuestBattle; territory 319,
  duty 368). Territory 319's TerritoryType has the Central Shroud's level (f1f1), so its NPCs are layer
  `QB_JobWhm300_001` of `fst_f1_planmap.json`, with the QuestBattle sheet's layout IDs and retail's levels, HP,
  positions, display flags (0x40001 for the allies, 0x40003 for the enemies) and character data flags (3 for all) from
  Bob's Chronofoil capture 8d550e33 (2026-11-05, `--battle 0.4`, one run, won; a level 35 conjurer synced to 34):
  Raya-O-Senna and A-Ruhn-Senna on your side (4293115, base 1675, BNpcName 1422; 4293116, 1676, 1423; level 40 and 41,
  white mages), the boss (4293124, base 1674, 6209 HP) with a raptor (1678) and two bats (1680), then two waves of
  three. The sheet's other layout IDs (four more raptors, bats and slugs, two water sprites) never came in that run and
  aren't spawned. The start spot (-284.58, 19.84, 119.85, facing -0.149) and the opening's flags (0xF8412FFF) are the
  run's.
* The flow (seconds from the fight starting, the director's scene 60000): sequence 2, the boss on Raya-O-Senna and its
  adds on A-Ruhn-Senna; battle talk 86 (Raya-O-Senna, no portrait) at 5.1 s and 87 (A-Ruhn-Senna) at 10.1 s. **The
  boss's adds down** (20.8 s, as the last of them fell; the boss was at 85.6 % from 17.5 to 23.5 s, so not its health):
  sequence 3 and the second wave at once (a raptor on Raya-O-Senna, a bat on A-Ruhn-Senna, a slug on you). **The boss at
  40 %** (44.1 -> 38.3 %, 56.7 s): sequence 4 and the third wave at once (a raptor on A-Ruhn-Senna, a bat on you, a
  water sprite on Raya-O-Senna), her 88 3.1 s later. **Everything down** (113.0 s; the report's deaths are the Kill
  ActorControl, about 2 s after the health reached 0): "Duty Complete" and sequence 5, the battle's scene 5
  (0xF8402EFB) 3.1 s later, the quest to 255 with UI8BH 1 (`QUEST_VARS_AFTER_WIN`), the director to 254 with its last
  byte 0x80. `questkit battle-sim 75` matches all 14 events within 0.3 s.
* The allies stay by you and cast from 5 yalms (Stone, Stone II, Aero), and Cure each other below 60 % (retail's at
  40-80 %), never you. Their Regen, Stoneskin (129) and Divine Seal (138) aren't played (barriers don't absorb, and
  the others only show the status). **The boss** uses Realm Shaker (697, instant, round itself) 11.8 s in and every
  20.4 s until it's at 40 %, then only Silkscreen (701, 2.5 s, a line on whom it's fighting) every 6 s, which **knocks
  back** whoever it hits (Knockback row 1, as the Thousand Maws of Toto-Rak's boss's: its targets 9-10 yalms further
  off). The slug's Viscous Discharge (1166) silences everyone it catches for 10 s, and its Aqueous Discharge (381) hits
  round it every 7.2 s; the raptors breathe frost (318) every 20 s; the water sprite casts Water and Water II.
  Timelines `QB75*` (bases 1674-1680), action scripts `QB75Slug_00381`, `QB75Slug_01166`.
* The client's battle script (`JobWhm300Btl_01079`, Bob's extracted scripts of 2026-11-05) has scenes 1 (the opening)
  and 5 (the closing cutscene, CUT_END_NCUT: 346), OnSequence001 (the objective) and OnSequence002 (its message);
  nothing in it reacts to sequences 3-5. The director plays only 1 and 5.

### Quest battle 59 — The Lominsan Way (`content/questbattle/59.lua`)

* Quest 811 (GaiUsb004_00811) enters it from the Grey Fleet miller (scene 2; territory 307, duty 354). Territory 307's
  TerritoryType has Lower La Noscea's level (s1f2), so its NPCs are layer `QB_GaiUsb004_001` of `sea_s1f2`, with
  retail's layout IDs, levels, HP, positions, flags, looks and the start spot (704.42, 65.78, -290.68, facing -2.07)
  from Bob's Chronofoil captures 44df02fc (2026-11-03, `--battle 0.4` to `0.14`: five runs lost, the sixth won) and
  c9145fdc (2026-11-03, `--battle 0.5`, won); a level 34-35 player. The one who talks and fights (4295469, base 1769,
  BNpcName 2073, 677 HP, battalion 4), the enemy you defeat to win (4295470, base 1770, 3900 HP), the NPC you protect
  (4295472, base 1771, 2418 HP, battalion 5: the only one of its side), a bomb (4295473, base 1772, BNpcName 1385, 897
  HP) and an onlooker (event NPC 1007082, 4295474, ENpcResident name for its line: kind 3). All four battle NPCs had
  character data flags 1 on retail; battalions decide who fights whom.
* **A lead-in before the fight.** Retail's opening scene 1 (0x2001) is over at once; then the director's scene 2
  (0x2001, its client script only waits 480 frames: 16 s) as a GameProgress event with sequence 2 and three lines
  (1769's 92 and 93 at 3.1 and 8.1 s, the onlooker's 94 at 13.1 s); as scene 2 ends, scene 60000 and sequence 3. Kawari
  has no scene 60000: `onCommence` (the opening's end) plays scene 2 and `onSceneFinished(2)` starts the fight. battle-sim
  models it with the fixture's `lead_in` ([questkit](questkit.md#battle-sim)).
* The flow (seconds from the fight starting), the same in seven runs: 1769 fights 1771 the whole time and never turned
  on the player, even when hit (the director's enmity); 1770 and 1771 never moved or acted (held). 1769's NpcYells
  1274-1278 at 3.1, 8.1, 13.1, 18.1 and 23.1. 1770 at **80 %** (its first health seen under it 77.4-78.7): sequence 4
  and 1769's yell 1285; 1.7-3.6 s later 1769 casts Paralyze (1118, 3.7 s; status 17, param 0, for 30 s) on the player,
  and **sequence 5 5.0 s after that cast began** (4.95-5.06 in every run), whatever 1770's health (59.9-70.1 %: not a
  health mark), with 1769's 95 and 96 3.0 and 8.0 s after. A run that never brought 1770 to 80 % had no Paralyze.
  1770 at 50 %: sequence 6; at 40 %: sequence 7 and 1769's 97; 3.0-3.1 s later sequence 8 and 0.3-0.4 s after that
  the bomb, which stands where it came out: killed (three runs), sequence 9, 1769's yell 1279 3.0-3.1 s and its 98
  8.0-8.1 s after; otherwise its blast (1133, round itself, no cast) 32.0 s after it came out (31.8 in the other run):
  2290-2294 to 1771 and 1826 to the player beside it, both dead, and sequence 9; the bomb falls 2.0 s later (Kill).
  1770's health at 0: "Duty Complete" and sequence 10 at once, the battle's scene 10 (0xF8402EFB) 3.0 s later; the
  quest moves to sequence 255.
* Lost: sequence 0xFE as 1771's health reaches 0 (the bomb twice; 1769 wore it down in 217 s once), 1769's (1769's
  99 with it; twice) or yours; out about 5 s later.
* 1769 (QB59Attacker_1769): auto-attack 870 (25 on 1771), a 2.2 s line from itself (Rive, 1299) from 36 s and every
  15 s, three Heavy Swings (31, 37) after each, the first 2.4-2.5 s after Rive goes off and the others 2.6 s apart (timeline 41.0, 43.6, 46.2: they were at 38.5, 41.1, 43.7, measured from Rive's start, so the first began the tick Rive went off and Rive went 1771's way, Bob's bundle 20261104-190047); Paralyze is the director's (`use_action`), after sequence 4. **Rive is
  aimed at the player**, whoever 1769 is fighting: every cast's ActorCast faced the player within a degree (19 casts,
  seven runs; it hit them when they stood in it, 60 once, and 1771 once, a miss, when it stood in line with them), so
  its timeline's point has `facing: player` (Kawari had aimed it at 1771).
* **Rive knocks the player back** along its line: retail's ActionEffect on the player had a knockback after the
  damage (Knockback row 1, [Combat](combat.md#enemy-damage-to-players)) and the client took them about 8 yalms on
  the way the line faced; none when it missed. Kawari sent only the damage until 2026-11-04 (Bob's bundle
  20261104-184153); `QB59Enemy_01299` adds `effects:knockback(1)`.
* **1769 hides from the bomb.** At sequence 8 (as the bomb comes out) it runs off to (704.37, 66.09, -279.13), 2.2 s
  at a run from beside 1771, turns to -129° and does nothing while the bomb is out (no action in any run); 7.0-7.2 s
  after the bomb is gone (four runs that got that far) it runs back to (694.97, 66.45, -286.64) and fights 1771 again.
  Kawari: `retreat_bnpc` there, `hold_bnpc` and `move_bnpc` to face retail's way on arriving (`onUpdate`'s `info`),
  then `retreat_bnpc` back and the director's enmity on 1771 again as it gets there. Retail also used one Rive from
  where it hid about 3 s after the bomb was gone, before going back; Kawari's timeline stays where it was while 1769 is
  held, so its next Rive comes on its own count.
* **The bomb's countdown** is the director's log lines: LogMessage 3217, 3218 and 3219 ("15 seconds to
  self-destruct..." and 10, 5) 14.6, 19.6 and 24.7 s after the bomb came out, while it still stood (EventLogMessage
  from the director, every run; a bomb killed before one had none of the later). They don't line up with the blast
  (32 s), as on retail. battle-sim doesn't check log lines.
* Fixtures: `59.json` (44df02fc, won), `59-c9145fdc.json` (won), `59-lost-4.json` (the bomb), `59-lost-8.json` (1769
  killed), `59-lost-12.json` (1771 worn down), each with the onlooker added and `"npc_health": "held"` (interpolating put
  sequence 6 2.2 s early in the won run).

### Quest battle 17 — In Nature's Embrace (`content/questbattle/17.lua`)

* Quest 441 (ClsCnj006_00441) enters it (scene 26, YesNoQuestBattle; territory 235, duty 301). Territory 235's
  TerritoryType has the South Shroud's level, so its NPCs are layer `QB_ClsCnj006_002` of `fst_f1f3`, with retail's
  layout IDs, levels, HP, positions, display and character data flags and the start spot (318.21, 4.36, -82.8, facing
  1.933) from Bob's Chronofoil capture df6f5af9 (2026-11-01, `--battle 0.4` and `0.6` lost, `0.8` won; a level 33
  conjurer synced to 34): Sylphie on your side (3873175, base 558, BNpcName 628), the boss (3796294, base 301, BNpcName
  596, 2106 HP), two yodomi (4273807, 4273810, base 1710) and two imps (4273820, 4273821, base 1709). The script takes
  the layout IDs, the yells (QIB_YELL_01/02, 1003 and 1004) and the drain's ID (DEF_YODOMI_DORAIN, 1127) from the
  QuestBattle sheet.
* The flow, the same in all three runs (seconds from the fight starting): sequence 2; Sylphie's battle talk 48 and 49
  at 5 and 10 s (her BNpcName, no portrait), her yell 1003 at 15.1. The boss at 70 % (75.5 -> 66.2, 73.1 -> 66.5,
  71.4 -> 63.2): sequence 3, her yell 1004 and the first yodomi 0.2-0.5 s later. In sequence 3 the boss's first drain
  brings sequence 4 3.6 s after its cast begins and her 50 5 s after that (the first run only: in the others the boss
  reached 40 % first). The boss at 40 % (42.1 -> 39.4, 44.4 -> 36.5, 46.6 -> 38.8): sequence 5, the imps 0.3-0.5 s
  later, her 51 10 s after; 20 s after 5, sequence 6, the second yodomi and her 52 5 s after. The won run: the boss's
  health at 0 with every add already down, "Duty Complete" and sequence 7 at once, then the battle's scene 7
  (0xF8402FFF); the quest moves to 255. Both losses: sequence 0xFE at the player's health's 0. The opening scene's
  flags are 0xF841AFFF.
* **The yodomi** never act or move (`hold_bnpc`); they're there to be drained: about every 26 s the boss casts 1127 on
  one (its rotation's `on_ally`, see [Class rotations](combat.md#class-rotations-serverrotationrs)), which hurts the
  yodomi 61-97, heals the boss about 32 and gives it Regen for 21 s at about 90 a tick, some 30 % of its health
  (QuestBattleNpc_01127). Killing them stops it. **The boss's 513 is a gaze** (see [Gazes](combat.md)): it paralyses
  whoever faces it (status 17 for 16 s) and misses anyone looking away (QuestBattleNpc_00513). Bob, playing it: "this
  can be dodged with an 100% effectiveness by simply looking away from the Boss", and the fight doesn't end until every
  enemy is down; the script wins only once the boss and everything brought out are down.
* Sylphie fights the boss, then the imps when they come, and Cures you below 50 % (retail's at 7-50 %, never anyone
  else). Timelines `QB17*` (bases 301, 558, 1709, 1710). `questkit battle-sim 17` checks the three runs (`17.json`,
  `17-lost-1.json`, `17-lost-2.json`, their health held between hits, `npc_health: held`): every step within 0.8 s.
* The client's battle script (`ClsCnj006Btl_00441`, Bob's extracted scripts of 2026-11-01) has scenes 1 (the opening
  cutscene, CUT_SCENE_01: 19) and 7 (the closing one, CUT_SCENE_02: 20, which can be skipped), OnSequence001 (the
  objective) and OnSequence002 (its message); no 10001. Nothing in it reacts to sequences 3-7: they're the director's
  own. The director plays only 1 and 7.
* The quest's three tainted earth sprites (layer `QB_ClsCnj006_001`, base 294, level 20, 619 HP) are in the same
  drop-in.

### Quest battle 58 — Big Trouble in Little Ala Mhigo (`content/questbattle/58.lua`)

* Quest 782 (GaiUsa709_00782) enters it from Wilred (scene 13; territory 306, duty 353). Territory 306's TerritoryType
  has Southern Thanalan's level (w1f4), so its NPCs are layer `QB_GaiUsa709_001` of `wil_w1f4`, with retail's layout
  IDs, levels, HP, positions, display and character data flags, looks and the start spot (39.49, 3.84, -364.19, facing
  -0.14) from Bob's Chronofoil capture 20ef0e90 (2026-10-31, `--battle 0.1`, won; a level 31 player), the same in two
  more runs of other characters' (captures 9faa9bac and bd51ca23, `--battle 0.1`, both won; every spawn alike): on your side
  Wilred (4251863, base 1984, BNpcName 2071), two young men (4251864 and 4251867, bases 1985 and 1986, BNpcName 2072,
  891 HP), Gundobald (4251873, base 1982, BNpcName 2069, 7108 HP) and a conjurer (4251874, base 1983); against you the
  boss (4252035, base 1977, BNpcName 2067, 5712 HP) and three waves: three (4252772, 4252773, 4252775), three (4252781,
  4252783, 4252785) and four (4252786-4252789). The script takes the layout IDs from the QuestBattle sheet's
  INIT_P_POP_* and INIT_POP_1ST/2ST/3ST_* and the portraits from its FACE_GRAPHIC_*; the sheet names five more
  (4252774, 4252776, 4252782, 4252784, 4252790) that retail didn't bring out. Retail had everyone of the first wave
  and your side out from the moment you arrived (as in 57's runs); here they come out as the fight starts, like every
  other battle (the cutscenes make their own characters).
* The flow (seconds from the fight starting), the same in all three runs: sequence 2: the boss on Gundobald, the first
  wave's lancer on the first young man and its two pugilists on the conjurer; Wilred's battle talk 98 at 5 s,
  Gundobald's 99 at 10, the young men's yells 1127 and 1128 (the sheet's YELL_QIB_01, 02) at 15 and 20. The first wave
  down, as the last one's death animation ends (2.0 s after its health reached 0; 31.7, 35.2 and 28.4 s in): sequence
  3, the second wave 0.2-0.5 s later (on the first young man, Wilred, Wilred), the boss's 100 at + 5.1, Gundobald's
  101 with sequence 4 at + 10.1. Sequence 4 + 5.0 Gundobald's 102. **Sequence 5** comes with whichever happens first,
  and each run showed a different one: the whole second wave down (its last one's health at 0, at once: bd51ca23,
  28.9 s after 4), the boss at 40 % (40.9 -> 39.7 in 9faa9bac, 23.0 s after 4) or 50.0 s after sequence 4 (20ef0e90,
  one of the second wave still up and the boss at 53 %). With it the boss's 103 and the third wave (on Wilred, the
  conjurer, the first young man (Wilred in the run where he had already fallen), Gundobald); + 10.1 sequence 6. The
  third wave's third down (its health at 0, at once in all three: 148.9, 151.1 and 112.6 s in): sequence 7,
  Gundobald's 104 1.0-1.1 s later. A young man's health at 0: his last words (107 the first, 108 the second; both fell
  in every run); the conjurer fell in one run, saying nothing. The boss's health at 0: 2.0 s later "Duty Complete" and
  sequence 8, then the battle's scene 8 (0xF8402EFB); the
  quest moves to sequence 255 (retail's director went to 0xFE, its last byte 0x80, as the scene ended). The opening
  scene's flags are 0xF8412EFB.
* Gundobald fights the boss throughout and Cures the others below 64 % (retail's at 9-64 %; you once, never himself);
  the conjurer stands where it is and Cures its side and itself below 50 % (retail's at 2-50 %; never you); Wilred and
  the young men take the nearest add.
* Timelines `QB58*` (bases 1729, 1730, 1977-1986), their amounts and rates the three runs' medians; action scripts
  `QuestBattleNpc_00426` (status 204 for 30 s), `_00517`, `_00632` (status 197 for 20 s), `_00865`, `_01288` (status 15
  for 16 s on someone of the other side); `_00477`'s status 65 lasts 60 s (lost 60.5 s after, here; it was a 20 s
  guess). The third wave's pugilists' heal on themselves (57) is QuestBattleNpc_00057's fixed 52, not retail's 338:
  recorded amounts don't scale a heal on oneself. `questkit battle-sim 58` checks the three runs (`58.json`,
  `58-9faa9bac.json`, `58-bd51ca23.json`): every step within 0.6 s.
* The client's battle script (`GaiUsa709Btl_00782`, Bob's extracted scripts of 2026-10-31) has scenes 1 (the opening
  cutscene) and 8 (the closing one, which can be skipped), OnSequence001 (the objective, no music) and OnSequence002
  (its message); no 10001. Nothing in it reacts to sequences 3-8. The director plays only 1 and 8.

### Quest battle 13 — Nouveau Riche (`content/questbattle/13.lua`)

* Quest 3862 (XxaUsa308_03862; the old 740's battle too) enters it from Laurentius (scene 8; territory 232, duty 311).
  Territory 232's TerritoryType has the South Shroud's level, so its NPCs are layer `QB_XxaUsa308_002` of `fst_f1f3`,
  with retail's layout IDs, levels, HP, positions, display and character data flags, looks and the start spot (40.19,
  7.81, -9.13, facing -1.175) from Bob's Chronofoil capture caa5e908 (2026-10-30, `--battle 0.13`, won; a level 27
  player), checked against two more runs of other characters' (captures 1fac2416, `--battle 0.13`, and 2528a32d,
  `--battle 0.12`, both won): the boss (4293096, base 1639, BNpcName 2064, 4522 HP), three lancers with it (4293092, 4293093,
  4293095, base 1642), a lancer (4293098, base 1640) and a gladiator (4293097, base 1646, BNpcName 80) on your side, the
  second wave (archer 4293111 base 1644, thaumaturge 4293110 base 1643, lancer 4293109 base 1642, gladiator 4293099
  base 1645, BNpcName 57) and two archers who come to help (4293125, 4293126, base 1641, BNpcName 139).
* The flow (seconds from the fight starting): sequence 2 with the boss and its lancers (the boss and one on your lancer,
  one on your gladiator, one on you); battle talk 86 (the boss) at 5 s and 87 (your gladiator) at 10 s. The three
  lancers down, as the last one's death animation ends (29.9, 28.9 and 29.2 s in, 3.0, 1.1 and 2.0 s after the last
  one's health reached 0): sequence 3 and the second wave (archer and gladiator on your
  gladiator, thaumaturge and lancer on you), the enemy gladiator's 88 10 s later and the boss's 89 15 s later. 30.0 s
  after 3, sequence 4 and the archers, their 90 5.1 s later and the boss's 91 10.1 s later; 30.1 s after 4, sequence 5
  and the second wave's archer, thaumaturge and lancer again (two on the first archer). The enemy gladiator's last
  words (92) as its health reaches 0. The boss down: its 93, sequences 6 and 7 and "Duty Complete" at once, then scene 7
  (0xF8402EFB); the quest moves to sequence 255. The opening scene's flags are 0xF8412EFB.
* Your lancer takes the lancer on it, then the boss for good; your gladiator the one on it, then the newest arrivals,
  Curing your side below 63 % (retail's at 44-69 %); the boss Cures itself below 68 % (retail's at 63-68 %); the archers
  take the adds, then the boss. Sequences 4 and 5 came 30 s apart in all three runs (the second wave down by sequence 5 in
  each), so nothing else is known to hold them.
* Timelines `QB13*` (bases 1639-1646); action scripts `QuestBattleNpc_00077`, `QuestBattleNpc_00477`,
  `QuestBattleNpc_00403` (also Presence of the Enemy's fourth ambusher's) and `QuestBattleNpc_01003`, their statuses'
  durations guessed. `questkit battle-sim 13` checks the three runs (`13.json`, `13-1fac2416.json`, `13-2528a32d.json`): every step within 1 s.
* The client's battle script (`GaiUsa308Btl_00740.luab`, Bob's dump of 2026-10-30) has scenes 1 (the opening cutscene)
  and 7 (the closing one), OnSequence001 (the objective, no music) and OnSequence002 (its message); no 10001. The
  director plays only 1 and 7.

### Quest battle 57 — Brotherly Love (`content/questbattle/57.lua`)

* Quest 724 (GaiUsa201_00724) enters it from Claxio (scene 4; territory 303, duty 350). Territory 303 is taken to be a
  copy of the East Shroud (retail's spots in it are the East Shroud's, near Komuxio's quests), so its NPCs are layer
  `QB_GaiUsa201_001` of `fst_f1f2`, with retail's layout IDs, levels, HP, positions, display flags, character data
  flags (3), looks and the start spot (-288.22, 12.51, -52.01, facing -2.229) from Bob's Chronofoil capture 5fcc970c
  (2026-10-30, `--battle 0.7`, won; a level 26 player), checked against two more runs of other characters' (captures
  1fac2416, `--battle 0.4`, and 2528a32d, `--battle 0.3`, both won): three thaumaturges (4255869-4255871, base 1584) and
  their gladiator leader (4255868, base 1582, BNpcName 583), Yda (4255892, base 1586) and Papalymo (4255894, base 1587)
  on your side at level 50 with 25890 HP, Claxio watching (4256371, an event NPC, ENpcResident 1006684), and the second
  wave: a beast that isn't to be killed (4255903, base 1585, BNpcName 2063, 49300 HP), a gladiator (4255897, base 1658)
  and a conjurer (4255896, base 1583) that heals the gladiators.
* The flow (seconds from the fight starting): sequence 2; the leader's battle talk 78 at 5 s and Claxio's 79 at 10 s
  (kind 3, portrait 73208). Two thaumaturges go for Yda and Papalymo, the third and the leader for you (the leader on
  Papalymo, his every hit a 0 or a miss), and Yda and Papalymo each take the one on them, then the oldest left. The
  first thaumaturge down, as its death animation ends (2 s after its health reached 0; 26.4 s in, the first run), or 30 s
  in if none is (30.1, the second and third runs): sequence 3, the leader's 80 and the second wave (the beast on Yda, the other two
  on you). From sequence 3, the same in every run: Papalymo's 81 at 12.0 s, the beast's 604 at Yda and Papalymo at 13.6
  (14.1, 13.1 and 16.0), Claxio's 82 at 17.0, Yda's 85 and Papalymo's 86 together at 17.5 (18.2, 17.3, 17.1: 4.2 s after
  the 604 in two runs but 1.1 in the third, so they're timed from sequence 3), Yda's 83 at 22.0. Sequence 4 at 37 s
  from the start whatever came before (36.5, 37.0 and 38.6: 10.1, 6.9 and 8.5 s after sequence 3; no death, cast or
  position fits all three): Yda and Papalymo leave the rest to you and take on the beast. 10.1 s after 4, sequence 5: the leader
  turns on you. The leader at 40 % after that (43 -> 35 in the first run, 45 -> 39.9 in the second): sequence 6 and
  Papalymo's 84 (the third run's leader was already down, killed 35.9 s in: sequence 6 came 0.1 s after 5). Until
  2026-10-30 the director had sequence 3 wait for a death and timed the talks and sequence 4 from
  it; the second run (no thaumaturge down by 30 s) showed what's timed from what. Every enemy but the beast
  down: sequence 7 at once (the last one's health at 0, not its death animation's end); 2.5 s later the beast runs back
  the way it came (toward where it appeared, at speed bytes 40 then 32: Kawari's run is 60) with Yda and Papalymo after
  it (`retreat_bnpc`); 9.1 s after sequence 7 "Duty Complete" and sequence 8, ActorControl 607 from the beast, Claxio
  and the fallen conjurer, their DeleteActors 0.6-1.4 s later (`fade_out_bnpc`, then `despawn_bnpc`; until
  2026-10-30 the beast and Claxio just vanished, Bob's bundle 20261030-110334), then the battle's scene 8
  (0xF8402EFB); the quest moves to sequence 255. The opening scene's flags are 0xF8412EFB.
* The beast's first 604 left Yda and Papalymo at exactly 80 % (their EffectResults 1.2 s after its AoeEffect, whose
  effect on each was kind 48, with 63 and 52: a kind Kawari doesn't know); its second, 30.7 s later, took nothing. Here
  the director casts it once, a hit scaled to 20 % of their HP (`QB57Beast_00604`); Yda's and Papalymo's lines 85
  and 86 come as it lands. The beast fights only them; Papalymo Cures you below 62 % (retail's at 41-61 %).
* Timelines `QB57*` (bases 1582-1587, 1658); action scripts `QB57Beast_00604`, `QB57Beast_00606`,
  `QuestBattleNpc_00063` (Yda, on herself) and `QuestBattleNpc_00151` (Papalymo, on the beast), their statuses'
  durations guessed. `questkit battle-sim 57` checks the three runs (`57.json`, `57-1fac2416.json`, `57-2528a32d.json`,
  Claxio added by hand to each): every step within 1 s (the first run's talks 0.5-1.4 s early: its sequence 3 is simulated 0.4 s early).
* The client's battle script (`game_script/quest/007/GaiUsa201Btl_00724.luab`, Bob's `kawari-questdump search
  GaiUsa201` of 2026-10-30) has scene 1 (the opening cutscene), scene 8 (the closing cutscene, which can be skipped),
  scene 10001 (EVENT_QIB_NOT_TALK, for talking to someone in the battle: Kawari's generic answer), OnSequence001 (the
  objective, no music) and OnSequence002 (the objective as a message). Nothing in it reacts to sequences 3-8: they're
  the director's own. The director plays only 1 and 8.

### Quest battle 56 — Like Mother, Like Daughter (`content/questbattle/56.lua`)

* Quest 440 (ClsCnj005_00440) enters it from Sylphie (scene 11; territory 291, duty 356). Territory 291 is taken to be
  a copy of the South Shroud (retail's spots against the drop-ins'), so its NPCs are layer `QB_ClsCnj005_001` of
  `fst_f1f3`, with retail's layout IDs, levels, HP, positions, display flags and the start spot (33.69, 3.11, 34.99,
  facing 0.795) from Bob's Chronofoil capture b23b91ef (2026-10-26, `--battle 0.29` lost doing nothing, `0.31` won;
  a level 26 conjurer): the target (4250414, base 1682, 2138 HP, never acts: held), three conjurers on your side
  (Wulfiue 4249072, Nolanel 4249073, Joacin 4249074; bases 1688-1690, ClassJob 6, with retail's `appearance`),
  Sylphie standing by (4249177, an **event NPC** in retail's spawns: drop-in event NPCs can be `nonpop` now, and the
  director brings her out with `spawn_bnpc`), the waves (bases 1683-1687) and three elementals that join you
  (bases 1691-1693).
* The flow, by both runs (seconds from the fight starting): sequence 2, the target, the conjurers and three 1683s,
  each on its own conjurer; Sylphie's battle talk 103 and 104 at 5 and 10 s (ENpcResident 1006738, kind 3, no
  portrait), the conjurers' yells 996, 997, 998 at 15, 20, 25 s. Sequence 3 when the target is at 80 % (84 → 78, 37.9
  s in) or at 40 s if it isn't (the lost run, untouched, 40.1 s): a 1685 on Nolanel and a 1684 on Joacin; her 105 5 s
  later and 106 15 s later. Sequence 4 when it's at 65 % after that (69 → 62): the elementals and a second 1684 (on
  Wulfiue), her 107 10 s later; sequence 5 6 s after 4, sequence 6 15 s after 5, a 1687 5.3 s after that and her 108
  10 s after it. In the lost run sequence 4 never came, and a 1686 came 35 s after sequence 3 and again 35 s later.
  Each conjurer has last words as he falls (battle talk 110 Wulfiue, 109 Nolanel, 111 Joacin, under their BNpcNames);
  it goes on without them. The target at 0: sequence 7 and "Duty Complete" at once, then the battle's scene 7
  (0x2001); the quest moves to sequence 4. Your health at 0: sequence 0xFE.
* The conjurers stand where they are, heal whoever of the side (you included) is lowest below 40 % about every 6 s
  (retail's heals were at 11-74 %, median 21-32 %) and fight their own 1683, then the target, then the rest; the
  elementals heal too. The director keeps its own timers beside `Battle:steps` (whose steps run one after another).
* Timelines `QB56*` (bases 1682-1693); action scripts `QB56Add_00440`, `QB56Add_01119`, `QB56Sprite_01118`,
  `QB56Sprite_00313` (their statuses' durations guessed). `questkit battle-sim 56` checks both runs (`56.json`,
  `56-lost.json`): every step within 1 s, and the opening line-up.
* Until 2026-11-01 the first wave (`FIRST_WAVE`) was written the wrong way round (`{ [conjurer] = 1683 }` where
  `arrive` takes `{ [enemy] = conjurer }`): the three conjurers came out a second time, as enemies, and the 1683s never
  did, so six conjurers burnt the target down in 14 s and every wave came at once (Bob's bundle 20261101-221536: "it
  completes almost instantly", "the enemies end up changing sides").
* The client's battle script (`game_script/quest/004/ClsCnj005Btl_00440.luab`, Bob's dump of 2026-10-27) has scenes 1
  (a short wait) and 7 (nothing, the end), OnSequence001/002 (the objective and its message), and scene 10001: a log
  message, EVENT_QIB_NOT_TALK, for talking to someone in the battle. Sylphie's spawn carries the battle as its
  handler (retail's, 0x80060038), so clicking her starts the battle's event: Kawari's content events had no answer to
  a talk, so it would never have ended; a quest battle now plays 10001 for it (`QUEST_BATTLE_SCENE_NOT_TALK`, flags a
  message scene's, unseen on retail) and finishes.

### Quest battle 54 — Trial by Wind (`content/questbattle/54.lua`)

* Quest 91 (ClsCnj002_00091) enters it from the corrupted air (scene 8, YesNoQuestBattle; territory 289, duty 349).
  Territory 289 is a copy of the East Shroud: retail's spots in it are the East Shroud's, a few yalms from where the
  quest's own honeybee swarm comes out, so its NPCs are layer `QB_ClsCnj002_002` of the `fst_f1f2` drop-in (taken
  from the positions, not checked against the game's TerritoryType sheet): the thing you're there to destroy
  (4261180, BNpcBase 1662, BNpcName 1420, level 10, 768 HP), wind sprites (base 1659, level 8, 31 HP), the conjurer
  who comes to help (4248519, base 1661, BNpcName 1419, level 15, 638 HP) and a last arrival (4248516, base 1660,
  BNpcName 1910, level 10, 172 HP); layout IDs, positions, display flags and the start spot (-509.7, 6.58, 39.45,
  facing -3.021) from Bob's Chronofoil capture b23b91ef (2026-10-26, `questkit chronofoil --battle 0.11`, one run,
  won, a level 14 conjurer).
* The flow: sequence 1, the opening scene 1 with 0x2001 (no cutscene, over at once), then the fight (sequence 2) with
  the target and one wind sprite; the target never acts or moves (the script holds it, `hold_bnpc`), the wind sprites
  stay where they appear and cast Aero (969) every 3.1 s. The target's health brings the rest: at 75 % (78 → 71)
  sequence 3 and three more wind sprites, one of them **a second spawn of 4248510's layout ID** while the first
  still stands (`spawn_bnpc` again, its actor ID given enmity with `set_hate`, which takes actor IDs since); at 40 %
  (46 → 39) sequence 4 and the conjurer (Aero every 6 s, the nearest wind sprite, then the target); at 25 %
  (31 → 24) sequence 5 and 4248516 (416 on itself 5 s later: status 57, then 969); his battle talk 107 and 108
  (BNpcName 1419, no portrait) 3 and 8 s after that. The target at 0: sequence 6 and every enemy still standing gone
  with it; a second later "Duty Complete" (60001) and sequence 7, then the battle's scene 7 (0xF8402EFB); the quest
  moves to sequence 5.
* The client's battle script (`game_script/quest/000/ClsCnj002Btl_00091.luab`) has scene 1 (a short wait, no
  cutscene) and 7 (the closing cutscene), and OnSequence001/002 (the objective and its message); the script plays
  those two and nothing else.
* Bob's first run (bundle of 2026-10-27) won it, and the quest stayed at sequence 4, so the battle was offered again:
  QuestBattle row 54's quest isn't quest 91 (the server tried quest 786646), so the win moved nothing. A won battle
  now moves the quest that's being done and names it (QUESTBATTLE0..3 among its parameters) when the sheet's isn't
  (`ZoneConnection::quest_for_quest_battle`). The conjurer came out a Miqo'te in other clothes from his BNpcBase
  (1661); retail's SpawnNpc had a Hyur in Wulfiue's gear, as class 6, so the layer has his `appearance` from the
  capture (the drop-in's recorded ClassJob, weapons, gear and customize), and character data flags 3, as retail's.
* Timelines `WindSprite_1659`, `TrialByWindTarget_1662`, `TrialByWindConjurer_1661`, `TrialByWindAdd_1660`; action
  scripts `Aero_00969` and `TrialByWindAdd_00416`. `questkit battle-sim 54`: every step within 1.3 s.

### Quest battle 45 — Lord of the Inferno (`content/questbattle/45.lua`)

* Quest 343 (ManFst209_00343) enters it from the destination in Eastern Thanalan (EOBJECT0, scene 8; territory 275).
  Territory 275 is taken to be a copy of Eastern Thanalan (145; retail's spots stand on its ground), so its NPCs are
  layer `QB_ManFst209_001` of `wil_w1f3`, from `questkit chronofoil --dropin 275`: four Immortal Flames on your side
  (the lead gladiator 4098749, BNpcBase 963, level 26, 6730 HP; a second gladiator 4098743, 964; a pugilist 4098744,
  965; a thaumaturge 4098745, 966; with retail's `appearance`), the Amalj'aa leader (4098741, 967, 11360 HP), three
  waves of lancers (968), soldiers (969) and javeliniers (970), and the scout (4100860, an event NPC, ENpcResident
  1004668, there from the start). Start spot (-334.77, -12.09, -238.7, facing 2.229) and opening flags (0xF841AEFB,
  scene 1) from Bob's Chronofoil capture b8432613 (2026-10-27, `--battle 0.8`, `0.10`, `0.12` lost, `0.14` won; a
  level 24 conjurer).
* The flow (seconds from the fight starting): sequence 2 with the allies, the leader (on the lead) and the first wave
  (the lancers and two soldiers on the pugilist, two soldiers on the lead); battle talk 202 (the scout, kind 3,
  portrait 73225) at 5 s and 203 (the lead, BNpcName 993, 73226) at 10 s. **Sequence 3 when five of the first wave's
  nine are left** (four down in all four runs), 2 s after the fourth's health reached 0, as its death animation
  ended (the director counts deaths 4 updates late); sequence 4 30 s after it (three runs): the second wave (four
  soldiers on the thaumaturge, a javelinier on you), 204 at once, 205 10 s later. **Sequence 5 as the fourth of the
  second wave's five falls**, at once (all three won runs; until 2026-10-28 it was "when two adds are left", which
  an arcanist's run, Bob's Chronofoil capture cc62ad94, `--battle 0.29`, didn't fit): the third wave (three lancers and three soldiers on the second
  gladiator, a javelinier on you), 206 10 s later, 207 15 s. **55 s after sequence 5 it's won**, whatever is still
  standing (the leader at 84 %, six adds): sequence 7 and, with no "Duty Complete", the battle's scene 7 (0xF8402EFB);
  the quest moves to 3. As the cutscene ends retail sent the player
  on to Southern Thanalan (146, ZoneInit pop range 4104257, at (-56.7, -24.57, -565.88) facing -0.468), by the flame
  sergeant the quest goes to next, not back to Eastern Thanalan (`EXIT_AFTER_WIN`; Bob's bundle of 2026-10-28: he was
  sent back). After the three losses he went back where he had entered. Retail's NPCs fought on through the cutscene
  (the second gladiator fell 0.85 s into it); Kawari's stop (`end_fight`). An ally falling says its last words (the
  pugilist 209, BNpcName 1007, 73229; the thaumaturge 208, 1005, 73227; the second gladiator 210, 994, 73228, even
  once the battle is won, not once it's lost) and the battle goes on. Until 2026-10-28 the script said 210 at the win:
  in the first won run it came 0.85 s after sequence 7 as the second gladiator fell (at 0.6 %), and in Dags' won run
  (Bob's Chronofoil capture c99fe374 of 2026-10-28, a gladiator, `--battle 0.29`), where it lived, not at all. Your
  health at 0: sequence 0xFE (all three losses).
* The allies: the lead on the leader (3 yalms), the others on the adds (nearest), the thaumaturge from 25 yalms;
  Cure (120) on whoever of the side is lowest below 40 % about every 6 s from the lead, the second gladiator and the
  thaumaturge (retail's medians 31-41 %), the pugilist's Second Wind (57) on itself below 35 %.
* The client's battle script (ManFst209Btl_00343) has scenes 1 (the opening cutscene), 7 (the closing one) and 10001
  (the won't-talk message), and OnSequence001/002; the script plays 1 and 7.
* Timelines `QB45*` (bases 963-970); action script `QB45Amaljaa_00490` (the Amalj'aa's 490, potency 100). The
  Flames' 14 and 718 and the Amalj'aa's 426 (on themselves) aren't modelled. `questkit battle-sim 45` checks all five
  runs (`45.json` won, `45-lost-8/10/12.json`, and `45-dags.json`, Dags' won run as a gladiator: sequence 5 at 79.9 s,
  the win 55.1 s after it, the same exit to Southern Thanalan) and `45-arc.json` (the arcanist's, won, the win 55.1 s
  after sequence 5 again): every step within 1.6 s.

### The Company You Keep: battles 46, 47 and 48

The three Grand Companies' battles are one battle with each company's people (the same foes, BNpcBases 1016 and
1020-1023), and their scripts one script with a different head (layouts, lines, start spot), timed against all their
retail runs. Since 2026-10-28 (the Maelstrom's, Bob's Chronofoil capture cc62ad94):

* **Sequence 3**: their leader below half (49 %), **or the first wave all down**, 2 s after the last one's health
  reached 0 (the Maelstrom's run: the leader was still at 60 %). **Sequence 4** and the vanguard 11 s after it
  (11.08, 11.05, 11.06, 9.07 s) when the leader's health brought it, 8 s when the wave did.
* **The vanguard below 70 and 40 %**: sequences 5 and 6. Then **its Drill Cannons** (525) move it on: 2.9 s after it
  begins each one (2.85-2.9 s every time, `onActorCast`), sequence 7, and 8 too if it's below 30 % (27.1, 27.5 %: at
  once; 32.2 %: no, then 17.3 % at the next one). 15 s after 8 sequence 6 again (15.07 s twice), and the next Drill
  Cannons 7. Until then 7 and 8 were marks at 27 % and 6.3 s later, and the second 6-7 a mark at 14 and 8 %: they
  only happened to fit.
* **The win**: 1.5 s after the vanguard's health reached 0 (2.2, 2.05, 0.4 s).

### Quest battle 46 — The Company You Keep (Twin Adder) (`content/questbattle/46.lua`)

* Quest 680 (ManFst302_00680) enters it from the airship crewman in the East Shroud (scene 5) or EVENTRANGE0 (scene
  6), territory 277, ContentFinderCondition 340. Territory 277's Bg is the East Shroud's own (`fst_f1/fld/f1f2`,
  the same as 152: not a guess this time), so its NPCs are layer `QB_ManFst302_001` of `fst_f1f2`, from `questkit
  chronofoil --dropin 277`: three Twin Adders on your side (the lead gladiator 4103768, BNpcBase 1017, level 20, 4956
  HP; a conjurer 4103769, 1018, 1156; an archer 4103770, 1019, 1156; hostile set false), their leader (4103775, 1020,
  level 22, 3204 HP) with two lancers (1021), a thaumaturge (1022) and two archers (1023), then a magitek vanguard
  (4103788, 1016, level 22, 4683 HP) with two more archers. Start spot (51.32, 7.89, 450.57, facing 0) and opening
  flags (0xF841AEFB, scene 1) from Bob's Chronofoil capture b8432613 (2026-10-27, `--battle 0.30`, `0.32` lost,
  `0.34` won; a level 24 conjurer).
* The flow (seconds from the fight starting): sequence 2, every enemy on the lead gladiator (all three runs), battle
  talk 113 (the lead, BNpcName 1001, portrait 73216) at 5 s and 114 (their leader, 996, 73215) at 10 s. **Their leader
  below half** (his health went 50.1 -> 48.7, 50.3 -> 48.7, 50.8 -> 46.7 % as it came, 57.5-90.1 s in): sequence 3
  and 115, 116 5 s later; **11 s after it** sequence 4 and the magitek vanguard with two archers, all on the lead;
  117 7 s after that. The vanguard below 70 and 40 %: sequences 5 and 6, then its Drill Cannons bring 7 and 8, and 15 s
  after 8 sequence 6 again ([above](#the-company-you-keep-battles-46-47-and-48)). Nothing else came with them. **The
  vanguard down**: 2.2 s later (its death animation) sequence 9 and "Duty
  Complete", the battle's scene 9 (0xF8402EF3) 3.6 s after; the quest moves to 255 and the director to 0xFE with its
  last byte 0x80 (`complete_quest_battle(255, false, 9, 0xF8402EF3)`); out back to the East Shroud. The conjurer
  falling says its last words (119, BNpcName 1176, 73217; a lost run) and it goes on. Your health at 0: sequence
  0xFE (both losses).
* The allies: the lead on their leader up close, the conjurer and the archer on the adds (nearest) from about 10
  yalms; the lead and the conjurer heal the side with Cure (120) below 50 %, their leader his own below 35 % (retail's
  median 30 %).
* No client battle script was dumped for it (the scene numbers are the capture's).
* Timelines `QB46*` (bases 1016-1023, from the three runs); action scripts `QB46MagitekVanguard_00525` (Drill Cannons)
  and `_00526` (Cermet Drill), lines from the Action sheet. Flash (14), Rampart (10), Raging Strikes (101), Life Surge
  (83) and 983 aren't modelled. `questkit battle-sim 46` checks all three runs (`46.json` won, `46-lost-30/32.json`).

### Quest battle 47 — The Company You Keep (Immortal Flames) (`content/questbattle/47.lua`)

* Quest 682 (ManWil302_00682) enters it from the airship crewman in Western Thanalan (scene 5) or EVENTRANGE0 (scene
  13), territory 278, ContentFinderCondition 341. Territory 278's Bg is Western Thanalan's own (`wil_w1/fld/w1f1`, the
  same as 140), so its NPCs are layer `QB_ManWil302_001` of `wil_w1f1`, from `questkit chronofoil --dropin 278`: three
  Immortal Flames on your side (the lead gladiator 4105366, BNpcBase 1024, BNpcName 994, level 20, 4956 HP; a
  conjurer 4105365, 1025, 1006, 1156 HP; a thaumaturge 4105364, 1026, 1005, 1156 HP; hostile set false) and the Twin
  Adders' battle's foes, the same BNpcBases (their leader 4105369, 1020, level 22, 3204 HP; two lancers 1021, a
  thaumaturge 1022, two archers 1023; then the magitek vanguard 4105368, 1016, 4683 HP, with two more archers). Start
  spot (172.03, 52.36, -72.12, facing 0.506) and opening flags (0xF841AEFB, scene 1) from Bob's Chronofoil capture
  c99fe374 (2026-10-28, `--battle 0.47`, won; Dags, a level 24 gladiator).
* The flow is battle 46's, step for step (seconds from the fight starting): sequence 2, every enemy on the lead; battle
  talk 112 (the lead, BNpcName 994, portrait 73222) at 5 s and 113 (their leader, 996, 73215) at 10 s; their leader
  below half (50.8 -> 46.8 %, 47.3 s in): sequence 3 and 114, 115 5 s later; 11.05 s after it sequence 4 with the
  vanguard and two archers, 116 7.1 s after that; the vanguard below 70, 40 and 27 %: sequences 5, 6, 7 and 8; 15.2 s
  after 8 sequence 6 (the vanguard was down before the 7 that follows); the vanguard down: 2.0 s later sequence 9 and
  "Duty Complete", the battle's scene 9 (0xF8402EF3) 3.6 s after; the quest moves to 255; out back to Western
  Thanalan, where the crewman is.
* The allies: the lead on their leader up close, the conjurer and the thaumaturge on the adds (nearest) from 7-10
  yalms; the lead and the conjurer heal the side with Cure (120) below 50 % (retail's at 35-55 %), their leader his own
  below 35 % (retail's median 35 %).
* No client battle script was dumped for it (the scene numbers are the capture's). Timelines `QB47*` (bases 1024-1026)
  for the Flames; the foes use battle 46's `QB46*` timelines (the same bases). Flash (14) and Rampart (10) aren't
  modelled. `questkit battle-sim 47` checks the run (`47.json`): every step within 0.7 s.

### Quest battle 48 — The Company You Keep (Maelstrom) (`content/questbattle/48.lua`)

* Quest 681 (ManSea302_00681) enters it from the airship crewman in Lower La Noscea (scene 5) or EVENTRANGE0 (scene
  6, flags 0x2001), territory 279. Territory 279's Bg is Lower La Noscea's own (`sea_s1/fld/s1f2`, the same as 135),
  so its NPCs are layer `QB_ManSea302_001` of `sea_s1f2`, from `questkit chronofoil --dropin 279`: three of the
  Maelstrom on your side (the lead marauder 4105799, BNpcBase 1030, BNpcName 1003, 4956 HP; a conjurer 4105800, 1031;
  an archer 4105801, 1032; hostile set false) and the other two battles' foes (their leader 4105802, the first wave
  4105803-4105807, the vanguard 4105862 with archers 4105863 and 4105864). Start spot (368.26, 75.92, -326.54, facing
  -3.116) from Bob's Chronofoil capture cc62ad94 (2026-10-28, `--battle 0.54`, won; a level 24 arcanist).
* The flow is the other two's ([above](#the-company-you-keep-battles-46-47-and-48)); battle talk 112 from the lead
  (portrait 73219) and 113-116 from their leader. Here the first wave fell before their leader reached half, so
  sequence 3 came from the wave.
* Timelines `QB48LeadMarauder_1030`, `QB48Conjurer_1031` and `QB48Archer_1032`; the lead's Skull Sunder is
  `QuestBattleNpc_00035` (a slashing hit scaled by the timeline). The foes use `QB46*`. No client battle script was
  dumped for it. `questkit battle-sim 48` checks the run (`48.json`).

### Quest battle 16 — Trial by Water (`content/questbattle/16.lua`)

* Quest 92 (ClsCnj003_00092) enters it from the Gods' Quiver bow (scene 17) or EVENTRANGE0 (scene 34) (territory
  234, duty 300). Territory 234 is taken to be a copy of the East Shroud too (retail's spots against the drop-ins'),
  so its NPCs are layer `QB_ClsCnj003_002` of `fst_f1f2`: three Wood Wailer lancers (3905949, 4263914, 4263915;
  base 1423, level 15, 1244 HP) and an archer (3914511, base 1422, 995 HP) on your side; three waves of
  yellowjackets (base 298, level 15, 606 HP) and water sprites (base 1681, level 14, 146 HP). Layout IDs (Sapphire's
  INIT_POP_* too), positions, display flags and the start spot (-51.95, -9.75, 232.86, facing -3.142) from Bob's
  Chronofoil capture b23b91ef (2026-10-26, `--battle 0.18` lost doing nothing, `0.20` won; a level 19 conjurer).
* The flow, the same in both runs: sequence 1 and the opening cutscene (0xF841AFFF), sequence 2 as the fight starts
  with everyone there; 5.1 s in, battle talk 127 (BNpcName 628, no portrait, from no actor) and sequence 3. Each
  yellowjacket goes for its own lancer (the same pairs in both runs: 3905949 gets 3875540, 3905948 and 4263934;
  4263914 gets 3879109, 4263917, 4263935; 4263915 gets 3905947, 4263918, 4263936), and for you once he's down; the
  water sprites come for you (1120, then 1016 round itself). The lancers fight theirs and then help the nearest; the
  archer shoots 3905949's. **The waves come when one enemy is left standing**, not by time: 30.4 and 49.1 s in for
  the second (sequence 4, three yellowjackets and a sprite), 72.5 and 108.7 s for the third (sequence 5, three and two
  sprites), each with the archer's line (128, 129, BNpcName 621) 5 s later. Each lancer yells once as his health
  drops to 43 % (NpcYell 967 for 4263914, 966 for the others, half a second after the hit; 43.0 % yelled, 43.2 %
  didn't). All down: sequence 6, a second later "Duty Complete" and sequence 7, then scene 7 (0xF8402FFF); the
  quest moves to sequence 7. Lost when your health reaches 0 (sequence 0xFE); two lancers fell in the lost run and
  it went on.
* The client's battle script (`game_script/quest/000/ClsCnj003Btl_00092.luab`) has scene 1 (the opening cutscene,
  CUT_SCENE_01) and 7 (the closing one, CUT_SCENE_02), and OnSequence001/002; the script plays those two only.
* The lancers (ClassJob 4) and the archer (5) have their `appearance` from the capture too, and character data flags 2.
* Timelines `QB16Lancer_1423`, `QB16Archer_1422`, `QB16Yellowjacket_298`, `QB16WaterSprite_1681`; action scripts
  `QB16Yellowjacket_00321`, `QB16WaterSprite_01016`, `QB16WaterSprite_01120`. `questkit battle-sim 16` checks both
  runs (`16.json` won, `16-lost.json`): every step within 0.6 s.

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
  these NPCs use have NPC-only scripts (`QuestBattleNpc_000xx.lua`). `questkit battle-sim 44` (the won run, and
  `44-dags.json`, Dags' won run as a gladiator, Bob's capture c99fe374 visit 0.17): every step within 1.2 s. In Dags'
  run the pugilist stayed at 21.5 % from 52.5 s to 75.4 s (he was on the other two), and 208 came as it went below
  20 %, so that fixture holds an NPC's health between samples (`"npc_health": "held"`) rather than drawing it straight.

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
2. Dump the battle's client script. It sits beside its quest's: `game_script/quest/<NNN>/<class>Btl_<number>.luab`
   (Trial by Water's quest is `ClsCnj003_00092`, its battle `ClsCnj003Btl_00092.luab`; Bob's search of 2026-10-27). When
   that isn't it, `kawari-questdump search ClsCnj003` (ArchonEngine ControlPanel: Quest Lab → Scripts that mention)
   copies out every client script that mentions the quest, under its real name. A battle script has the director's
   `OnSequenceNNN` (the duty objective, a message) and its own scenes: check the director script plays only those.
3. Add the NPCs to the territory drop-in as a `QB_…` layer.
4. Write `content/questbattle/<id>.lua` with the callbacks above; keep a timing table in its header (seconds from the
   fight starting, the director's scene 60000, not from entering).
5. Make its battle fixture from a Chronofoil capture of the run (`questkit chronofoil --battle N.V --out run.json.gz`,
   then `questkit battle-sim --make <id> --run run.json.gz`) and run `questkit battle-sim <id> -v` until it matches
   ([questkit](questkit.md#battle-sim)); CI holds it to that from then on.
6. Add any new enemy skills as action scripts (`resources/scripts/actions/`), see [Combat](combat.md). Give allies and
   enemies that fight with a class's actions a `rotation` in their timeline (questkit's report writes one per NPC),
   rather than fixed timepoints.
