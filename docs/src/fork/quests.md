# Quests

How quests work in the fork: what the server keeps, what the client expects, the Lua API, the script pattern every
fork quest follows, how a quest is verified, and every quest scripted so far.

## What the server keeps for a quest

**In memory and in world.db** (`quest` table, one row per character):

| Column | Contents |
|---|---|
| `active` | JSON list of the active quests: id, sequence, flags, the six variable bytes (UI8A–F), the class/job it was accepted as. Up to 30. |
| `completed` | Kawari's completed-quest bitmask (most significant bit first). |

Quest progress is saved by the 30-second autosave and on logout (see [Saving](saving.md)).

**What the client gets** (`QuestActiveList`, `core/src/ipc/zone/server/quest_active_list.rs`): 30 entries of 16 bytes,
laid out like the client's `QuestWork` (mapped with marker bytes sent through `!rawpacket` on 2026-10-13):

| Byte | Field |
|---|---|
| 0–1 | quest number (u16, e.g. 387; the Quest sheet row and handler ID are 65536 + this) |
| 2 | sequence (0 = not started, 255 = the last step, "report back") |
| 3 | flags |
| 4 | accepting class/job |
| 5–9 | ignored by the client |
| 10–15 | UI8A … UI8F, the six variable bytes |

Single quest updates (`UpdateQuest`) go to the quest's own journal slot. A padding mistake here once made the client
read every variable one byte off (fixed in `4ed1233`, then `3f2d23e`): objectives never ticked and key items never
showed, so when objectives look wrong, compare a Kawari Probe snapshot's `raw_hex` with the server's state first.

### Quest variables

Each quest has six bytes, UI8A..UI8F (`get_quest_var(quest, 0..5)`). The client's scripts address them as:

* `UI8AH` / `UI8AL`: the **high** and **low nibble** of byte 0 (so two counters 0–15 in one byte);
* `UI8A`: the whole byte;
* **BitFlag8** n: a bit of **UI8F**, flag 1 being the top bit: `0x80 >> (n - 1)`. Used to mark "this object is done"
  so the client stops offering it.

What each variable means is **per quest**, and only the client's script says: it reads them to decide which todos are
ticked, which objects are interactable, and which **key items** the player holds. Key items are not inventory: the
client lists them itself from the variables (its `GetEventItems`), e.g. "ITEM0 count in UI8BH". So giving a key item
means setting the right nibble, and handing it over means clearing it. Quests often **move** item counts to different
nibbles at sequence 255 (Surveying the Damage, A Soldier's Breakfast); copy what the recording shows exactly.

### Where a quest's names come from

When a quest script loads, the server injects the Quest sheet's Lua parameters as globals
(`servers/world/src/events/lua.rs`, `inject_lua_parameters`): `ACTOR0`, `EOBJECT1`, `ENEMY0` (a BNpcBase ID, or a
layout ID for quest-spawned enemies), `ITEM0`, `EVENTRANGE0`, `QUESTBATTLE0`, `TERRITORYTYPE0`, `EVENT_ACTION_SEARCH`,
`QST_ACCEPT_CHECK` and so on, plus `EVENT_ID` (the quest's handler ID) and `BASE_ID` (the NPC/object being talked
to). The log lists them ("Variables available in event …"). A name the sheet doesn't have is silently `nil` in Lua;
Kawari Web's Quest Lab script check flags those.

## The Lua API for quests

Player methods (`servers/world/src/lua/player.rs`), the ones quests use:

| Method | Does |
|---|---|
| `play_scene(scene, flags, {params})` | Plays a scene of the current event. Flags from `event_scene.rs`: `HIDE_HOTBAR`, `NO_DEFAULT_CAMERA`, `HIDE_UI`, … (Global.lua has the names; `INITIAL_CUTSCENE_FLAGS` is the full cutscene set). |
| `finish_event()` | Ends the event. Every path through a script must end with one, or the client waits forever. |
| `player.scene_error_code` | The error code of the scene that just returned: 0 normally. Some yes/nos answer through it instead of their results: The Gridanian Envoy's boarding scenes return results `[0, 0]` either way, and error code 50 for "No" (Sapphire's scripts; retail's "Yes" was 0 in Bob's capture, and a cancelled scene came back with 50). |
| `accept_quest(id)` | Adds the quest at sequence 255 (`zone_connection/quest.rs`); scripts with more steps then call `quest_sequence(id, 1)` or whatever the recording shows. |
| `get_quest_sequence(id)` | The sequence, or `nil` if the quest isn't active. |
| `quest_sequence(id, seq)` | Sets the sequence and sends the update. |
| `get_quest_var(id, byte)` / `set_quest_var(id, byte, value)` | Reads/writes one variable byte (0–5) and sends the quest update. Changes aren't logged; a packet capture's `UpdateQuest` packets show them. |
| `finish_quest(id, chosen_item)` | Completes it: gil, EXP, the fixed items and crystals, the optional item the player picked (`results[2]` of the reward scene). EXP is `ExpFactor × ScaledQuestXP[L] × QuestExpModifier[L] / 100` with L the player's level but no more than the quest's (ClassJobLevel + QuestLevelOffset): exact on 30+ turn-ins in Bob's captures (a level 1 quest at level 2 gives 400, not 660). As retail, the log lines come first (EventLogMessage from no event: 1605 the gil, 1606 an item, 1607 items with a count), then the gil and items, then the EXP. |
| `cancel_quest(id)`, `incomplete_quest(id)`, `has_quest(id)` | As named. |
| `event_notice(id, notice, {args})` | Sends an EventNotice (see below). |
| `mark_quest_completed(id)` | Marks another quest completed with no rewards (its FinishQuest), as retail does for a quest one replaces: Lurkers in the Grotto (543) completes 464. |
| `start_nested_event(handler)` / `finish_nested_event(handler)` | Sends only the packets of a nested event (EventStart / EventFinish, type Nest, on the current event's target), as retail does with the aetheryte's event while attuning in Close to Home; no handler runs for it. If an event action is interrupted, the nested event is finished too (and OccupiedInEvent removed). |
| `event_action(id)` | Starts an event action (the progress bar: Searching, Aiding, Interacting…); the script gets `onEventActionCast(player)` when it completes, after the EventAction row's CastTime (read in the world server's language: the sheet has a Name column, and read without one it failed and every bar ended after a 2 s fallback, before the client's, until 2026-10-20). Directors' event actions (`begin_event_action`) take the same time. |
| `spawn_quest_bnpc(layout_id)` | Brings out a battle NPC from the zone's drop-ins for this quest. |
| `join_quest_battle(id, zone)` | Enters a quest battle (see [Quest battles](quest-battles.md)). |
| `take_handed_in_items()` | Takes the items handed in through the current event's trade window out of the bags (see [NPC trades](#npc-trades)). |
| `player.inventory:count_in_bags(item_id)` | How many of an item are in the four bags. |

Callbacks a quest script can define:

| Callback | When |
|---|---|
| `onTalk(target, player)` | The player talks to/examines `BASE_ID`. |
| `onYield(scene, id, results, player)` / `onReturn(scene, results, player)` | A scene finished. The client can report a scene through **both**, so scripts guard with `current_scene` (below). `results` is the scene's return values. Scenes with more than 8 results arrive as EventReturnHandler16 (opcode 200 on 7.57). Which one a scene comes back as is the client script's choice: in the Ul'dah quests the offers, the rewards and Close to Home's trade were yields (EventYieldHandler4), Nothing to See Here's trades and the cutscenes returns. |
| `onEventActionCast(player)` | An `event_action` progress bar completed. Not called if it was interrupted. |
| `onEnterTrigger(player, arg)` | The player walked into one of the quest's event ranges (e.g. `EVENTRANGE0`). |
| `onEnterTerritory(player)` | Entering a zone where the quest has something to do (e.g. post-battle cutscenes). A script the client starts this for must define it and finish the event; otherwise the error is logged and the client keeps waiting (a fix for that, `02e22b5`, went with the rollback). |
| `onEmote(emote, target, player)` | The player used an emote (its Emote row) on `BASE_ID`, an NPC or object the quest listens to for emotes in its current sequence (see [Emotes](#emotes-on-an-npc)). |
| `onDutyCleared(content_id, player)` | The player is leaving a duty they cleared (the director's `complete_duty`), with its **InstanceContent** ID, what `INSTANCEDUNGEON*` parameters are (Sastasha is 4). Retail moved It's Probably Pirates on as Bob left Sastasha by its exit after the clear, before the loading screen (Chronofoil capture of 2026-10-23: the exit's event, then the notice and UpdateQuest, then PrepareZoning); the client's `IsAcceptDirectorResult` checks the duty. Not an event: nothing to finish. Kawari: `FromServer::DutyCleared` from `director.rs`, kept in `ZoneConnection::duty_cleared` and called from `FromServer::LeaveContent` (`zone_connection/quest.rs`, `on_duty_cleared`). |
| `onBNpcKill(base_id, layout_id, player)` | A battle NPC the player (or their party in the same instance) had enmity with died. `base_id` is the **BNpcBase** ID, which is what `ENEMY*` parameters are (an anole is base 142, name 120). |

### Notices (the progress pop-up)

`player:event_notice(EVENT_ID, n, {args})`. From retail recordings, **notice n means todo n was completed or
advanced**, and its arguments are either:

* `[count, needed]` (e.g. `[2, 4]`: "2/4"), optionally followed by an icon (`[count, needed, icon]`), or
* a single key item icon (`[25919]`), or nothing.

0–2 arguments are sent as **EventNotice2** (opcode 386 on 7.57, 834 on 7.57h1), 3–4 as **EventNotice4** (210; 467);
more than 4 is refused
with a warning. Retail sends them at specific moments (some quests only on the last kill, some on every step), so take
them from the recording, not from guesswork. The number is the todo's, not the order things were done in: Close to
Home's three objectives send 0, 1 and 2 whichever is done first, the last one included. Some todos send none
(Spirithold Broken's battle and aftermath, Chasing Shadows' everything but the item).

Bob's Chronofoil captures have every notice of his sessions (`UpdateQuest`: index, 3 bytes, quest u16, sequence,
flags, accepting class, 5 bytes, then the 6 variables at offset 14). The early Gridania quests' fixtures took theirs
from the capture of 2026-10-14: each notice goes with the first quest change at most 0.5 s before it, matched to the
fixture's steps by state. The table of what each quest sends is in each script's header.

### NPC trades

A trade scene opens the hand-over window. It returns `[1, …]` when the items were handed over and nothing when the
window was closed. On `1`, play the "trade OK" scene, which returns `[1, chosen_item]` with the reward; then
`finish_quest`. Never advance on a closed window.

What the client sends for an inventory item (Color Your World, Bob's Chronofoil capture of 2026-10-25): an
ItemOperation Move (`0xF1`, BASE_INVENTORY_ACTION 234 + 7) from the bag slot into **HandIn** (container 2005) slot 0,
which the server acks (InventoryActionAck type 7). The bag slot stays as it was: retail only emptied it (an
UpdateInventorySlot with quantity 0) with the reward, alongside the reward item, the EXP and the unlock. Kawari keeps
such moves in `Inventory::handed_in` (bag, slot, item, count) without moving anything, forgets them when the event
finishes, and a script takes them with `player:take_handed_in_items()` just before `finish_quest` (a move back out of
HandIn drops the record). Before 2026-10-25 a move into HandIn emptied the source slot at once with nowhere to put
it, so the item vanished on the server while the client still showed it. Key items (NPC_TRADE_INVENTORY_MODE_EVENT
trades) live in the quest's variables, so their scripts clear the nibble instead. To pick between a trade scene and a
"you don't have it" line, `player.inventory:count_in_bags(item_id)` counts an item across the four bags (call it
through `pcall` and assume the item is there if it fails).

### Searching objects (event actions)

`player:event_action(EVENT_ACTION_SEARCH)` shows the 'Searching' bar (the EventAction row comes from the Quest sheet's
parameters). When it completes the server ends the bar and calls `onEventActionCast`. Retail plays a search in three
parts (Bob's Chronofoil captures of 2026-10-14, every search, aid and examination in Gridania's quests): the object's
scene (`SEQ_n_EOBJECTm`, which only waits), then the bar when that scene returns, then the object's "found it" scene
(`SEQ_n_EOBJECTm_EVENTACTIONOK`, e.g. 100, 98, 96…; `…NO`, one less, is what retail plays when the bar is broken off).
Both scenes use `HIDE_HOTBAR | NO_DEFAULT_CAMERA` (0x2001), and aiding someone in Spirithold Broken adds `INVINCIBLE`
to the second (0x6001). The fork's scripts started the bar straight away and played only one scene, with flags 0 (as
Sapphire does), until `questkit replay` began checking scenes. Enemy damage interrupts the bar (see
[Timing](timing.md)): the event ends and the player tries again.

The bar's end comes from the server (a timer), not from the client, so what `onEventActionCast` queues is sent at once
(the end of `process_server_msg` runs whatever a server message had a script queue). Before, it waited for the
client's next packet: objects answered several seconds after the bar (2.7 and 6 s for the gangplank, 2026-10-17
bundle), as did anything else a server message gives a script (kills, won quest battles).

### Key items used on something (event items)

Some steps want a key item used on an object or NPC (Out of House and Home: ITEM0 on the fissure). On retail (Bob's
Chronofoil capture of 2026-10-18):

1. the client sends an **ActionRequest** of type **EventItem** (3) with the item as the action and the object
   (its layout ID) as the target;
2. the server shows its cast bar: ActorCast with the EventItem row's `Action` as the spell, the item as the action and
   the row's `CastTime` (3 s for it); moving cancels it like any cast;
3. when it ends: ActionEffect (same spell and item, no effects, a 0.1 s animation lock), then the quest's event:
   EventStart on the object with **type 8 (EventItem) and the item as its argument**, the quest's scene, EventFinish
   (type 8, result 1, the item).

Kawari does this in `server/action.rs` (`start_event_item`, `use_event_item`, the cast as a `CastAction` task) and
`main.rs` (`FromServer::EventItemUsed`): the EventItem row's `Quest` must be underway, and its script gets
`onEventItem(item_id, target, player)` with `BASE_ID` the object's base ID. Before, any EventItem ActionRequest hit
an `unimplemented!()` (and any ActionRequest without an Action row an `unwrap`), panicking the world server's task;
an unknown action is now ignored with a warning. questkit's fixtures replay it as an `event_item: [item, base ID]`
step.

### Emotes on an NPC

Some quests want an emote used on an NPC (Double Dealing, 466: ACTOR1 won't talk until you do). On retail (Bob's
Chronofoil capture of 2026-10-21) the server sends the emote as usual (ActorControlTarget, category 290, the emote),
then starts the quest's event on the NPC: EventStart with **type 2 (Emote) and the emote as its argument** (12 there),
the quest's scene, EventFinish. The client sends nothing quest-specific; the server knows from the Quest sheet's
listeners, where event type 2 (`QuestUInt8A`) marks an NPC that takes an emote in that sequence (`ActorSpawnSeq`; the
listener says no emote, so any emote does).

Kawari: `server/mod.rs` sends the emoting player's connection `FromServer::EmotedAt(emote, target)`; `main.rs` looks
up the target's base ID and, if no event is running and an active quest listens for emotes on it in its current
sequence (`GameData::quest_listens_for_emote`), starts that quest's event (type Emote, the emote as its argument) and
calls `onEmote(emote, target, player)`, with `BASE_ID` the NPC's base ID. questkit's fixtures replay it as an
`emote: [emote, base ID]` step.

### Quest-spawned enemies

`player:spawn_quest_bnpc(layout_id)` (`ToServer::SpawnQuestBattleNpc`, handled in `server/mod.rs`):

1. looks up the layout ID in the zone's battle NPCs (from `resources/dropins/<zone>.json` layers such as
   `QB_SubFst060_001`);
2. does nothing if that NPC is already out (one of each at a time);
3. inserts it, records it in `instance.quest_spawned_npcs` so it **doesn't respawn** when defeated;
4. fills in what retail's SpawnNpc gives a quest's enemy (Bob's Chronofoil captures of 2026-10-18 and b23b91ef,
   Trial by Earth's tainted earth sprite): `event_npc_instance_id` = what brought it out (the current event's target:
   the object searched, or the range walked into), `handler_id` = the quest, and the player as `owner_id`,
   `target_id` and `combat_tagger_id`; the client's GetGimmickState asks IsBattleNpcTriggerOwner. Until 2026-10-27
   these were 0;
5. gives it enmity on the player, so it attacks at once.

New spawns need a drop-in layer with the retail position, level and HP (from a Kawari Recorder recording).
When editing a drop-in JSON, splice the new layer into the text: rewriting it through a JSON library changes how every
float is written.

### Cutscenes that end somewhere else

Some cutscenes end with the player in another zone: the client fades to black at the end (or when it's skipped) and
waits for the server's PrepareZoning. If the script only finishes the event, the screen stays black, with only chat
working. On retail the order is PrepareZoning, then the quest's UpdateQuest, then EventFinish; in a script that's
`quest_sequence`, `finish_event()`, then `change_territory_pop_range(territory, pop_range)`. A quest parameter named
`POPRANGE*` that no talk or door uses is the hint; the Level sheet gives its territory. Renewing the Covenant's ceremony
is one (to the Roost, 179).

## The script pattern

Every fork quest script follows the same shape (see `SubFst073_00387.lua`):

```lua
-- Quest: <name> (<city>)
--
-- Checked against the client's quest script (kawari-questdump), Sapphire's script and a live recording of the quest:
-- Sequence 0:   ... one paragraph per sequence: who, which scene, what the variables do, which notice is sent ...
-- Sequence 255: ...

SCENE_OFFER = 0            -- named scene constants
NOTICE_PROGRESS = 0        -- notice numbers and icons, with where they come from

function getVar(player, byte, high) ... end    -- nibble helpers (H = high, L = low)
function setVar(player, byte, high, nibble) ... end
function hasFlag(player, flag) ... end         -- BitFlag8 in UI8F

current_scene = nil        -- only the scene we last played may finish
function play(player, scene, flags) current_scene = scene; player:play_scene(scene, flags, {}) end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    if sequence == nil and BASE_ID == ACTOR0 then play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif ... 
    else player:finish_event() end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then return end
    current_scene = nil
    ... advance on the results ...
    player:finish_event()      -- unless another scene was just played (then `return`)
end

function onYield(scene, id, results, player) sceneFinished(scene, results, player) end
function onReturn(scene, results, player) sceneFinished(scene, results, player) end
```

Rules: the header's "Checked against…" line says what it was verified with (Kawari Web's progress page counts a quest
as *verified* when it mentions a recording); scene numbers come from the client's script (Sapphire's when the client's
doesn't name them); variables and notices come from the recording; scene flags and anything else the server does
(zone changes, unlocks) from a Chronofoil capture, which the fixture holds and replay checks; everything else (names,
text) from game data.

**Retail's scene flags** (every quest in Bob's 2026-10-14 to 17 captures): talking to someone `HIDE_HOTBAR` (0x2000);
cutscenes `SET_BASE | HIDE_HOTBAR` (0xF8402EFB, which hides other players, NPCs and objects: the fork's scripts had
`FADE_OUT | CONDITION_CUTSCENE | HIDE_UI` until replay checked flags), some with `DISABLE_SKIP` (Spirithold Broken's
reward, 0xF8482EFB) or `INVIS_ENPC | INVIS_AETHERYTE` too (Underneath the Sultantree's aftermath and reward,
0xF8402FFF); objects and searches `HIDE_HOTBAR | NO_DEFAULT_CAMERA` (0x2001). One-offs are in the scripts with where
they came from. Moving the player within a zone (The Gridanian Envoy's ticket counters and attendants) goes through a
loading screen on retail, PrepareZoning to the same territory, so the scripts use `change_territory` for it too.
The script goes in `resources/scripts/events/quest/<id / 100, 3 digits>/<ScriptId>.lua` (the Quest sheet's script
name, e.g. `SubFst073_00387`).

## Adding a quest, step by step

1. **Recording.** Bob plays it on retail with the Kawari Recorder (Quests and Quest packets tabs) and exports. The
   export has every sequence/variable change with what was targeted, the notices with their arguments, and NPC and
   object positions.
2. **Dump** it: `kawari-questdump <id>` (or Kawari Web → Quest Lab). Read `listing.txt`: which scenes exist, what each
   `OnScene`/`OnYield` returns, which variables tick which todo, `GetEventItems` for key items, `QST_ACCEPT_CHECK`.
3. **Compare** with Sapphire's script (`/home/claude/sapphire` when available) for scene meaning and battle data.
4. **Scaffold** it: `python3 tools/questkit scaffold <id> --dumps quest-dumps --recording <export>` writes a draft in
   the pattern above and its replay fixture, and replays it. Fix the scenes marked TODO against the listing (and
   Sapphire), and anything retail does that didn't change the quest (spawns: add `spawns` to the fixture step). New
   enemies → drop-in layer; new solo duty → quest battle script.
5. **Replay**: make the fixture from Bob's Chronofoil capture of the quest (`questkit fixture <id> --chronofoil …`,
   see [questkit](questkit.md#making-fixtures)), then `python3 tools/questkit replay <id> -v` until every step matches
   retail's scenes, flags, effects, variables and notices. The
   fixture is committed with the script, and CI replays it on every push from then on.
6. **Test in game**: Kawari Web → Quest Lab → *Test this quest* sets the character up, turns capture on, and sends a
   bundle tagged with the quest when you finish; `questkit triage` and `questkit compare` read it. (Or by hand:
   `//gm quest accept|sequence|incomplete …`.)
7. **Document**: add it to the table below and the [Changelog](changelog.md).

**Quests without a dump.** A capture shows everything retail did on one run: who you talked to, every scene and its
flags and answers, the variables, notices, items and rewards. What it can't show is what the client's script would do
on another run: other answers, the checks the client makes on its own (Prudence at This Junction's equipment check),
and the parameter names (`ACTOR0`…, injected from the sheet) with the script's file name. A script written from a
capture alone compares NPCs by their recorded ENpc IDs and can be named anything ending in `_<quest number, five
digits>.lua`: `quest_script_path` (`event.rs`) falls back to it when the sheet's script name has no file.

## Scripted quests

Gridania's story from Close to Home on, its side quests, the first quest after it, and Ul'dah's first nine. Every one
has a fixture from Bob's Chronofoil captures (2026-10-14 to 17), so replay checks its scenes, their flags and what it
does outside the quest as well as its variables and notices. "Recording" means it matched a retail recording's variables and notices.
Every quest here has a replay fixture in `tools/questkit/fixtures/` and passes `questkit replay`.

| ID | Quest | Script | Checked against | What it involves |
|---|---|---|---|---|
| 124 | Close to Home (Conjurer) | `001/ManFst004_00124` | client script, recording, Chronofoil capture (notices) | Moves the player from the opening's New Gridania (183) into the real one (132); three objectives in any order: attune (also teaches Return, sets BitFlag8 1), visit the guild, deliver ITEM0 (held in UI8CH); UI8AH counts the objectives done; notice 0, 1 or 2 for each (by objective). The Lancer (85) and Archer (123) versions are the same script with their own ACTOR2 and ITEM0. |
| 85 | Close to Home (Lancer) | `000/ManFst002_00085` | client script (2026-10-19; 124's apart from the guild's dialogue, ACTOR2 and ITEM0); fixture adapted from 124's, no retail recording | As 124 with its own guild NPC (ACTOR2) and ITEM0. |
| 123 | Close to Home (Archer) | `001/ManFst003_00123` | client script (2026-10-19; 124's apart from the guild's dialogue, ACTOR2 and ITEM0); fixture adapted from 124's, no retail recording | As 124 with its own guild NPC (ACTOR2) and ITEM0. |
| 28 | To the Bannock | `000/SubFst005_00028` | client script, Chronofoil capture | Offer, report at the Bannock, chosen reward. |
| 201 | Passing Muster | `002/SubFst045_00201` | client script, Chronofoil capture | The client checks item level 5 gear itself and returns 0 if not. |
| 445 | Chasing Shadows | `004/ManFst005_00445` | client script, recording (replay), Chronofoil capture (notices, quest battle) | Examine the blade (scene 2, Process event action; UI8AL remembers it until the battle starts), quest battle 11, aftermath on re-entering, second blade gives ITEM0 (UI8BH, notice 2 `[30426]`, the only notice), trade. |
| 3854 | Eggs over Queasy | `038/XxaFst034_03854` | client script, Chronofoil capture | Three chigoe kills each give an egg (UI8BH, notice 0 `[count, 3, 22623]`), trade. |
| 175 | Surveying the Damage | `001/SubFst038_00175` | client script, recording, Chronofoil capture (notices) | Four searches → three todos, BitFlag8s 1–4, notices 0 `[26151]`, 1 `[count, 2, 21002]`, 2 `[26103]`, item counts move at 255; the three-item trade returns 12 results (EventReturnHandler16). |
| 3855 | A Soldier's Breakfast | `038/XxaFst031_03855` | client script, recording, Chronofoil capture (notices) | Anole egg (Interacting bar) + three anole kills in any order (notices 0 `[25251]` and 1 `[count, 3]`), notice 2 on reporting, egg moves to UI8BH at sequence 2, trade. |
| 129 | Spirithold Broken | `001/SubFst035_00129` | client script, Sapphire, recording, Chronofoil capture (notices, quest battle) | Notice 0 at Spirithold; aid five (Aiding bar, BitFlag8 1–5, notice 1 `[count, 5]`), the Hearer or EOBJECT0 offers quest battle 15, post-battle cutscene, notice 4 on reporting back, reward. |
| 176 | On to Bentbranch | `001/SubFst027_00176` | client script, recording | Accepting goes straight to sequence 255. |
| 376 | You Shall Not Trespass | `003/SubFst049_00376` | client script, recording | Notice 0 on reaching Roseline; four qiqirn kills, notice 1 `[count, 4]` each. |
| 377 | Don't Look Down | `003/SubFst056_00377` | client script, Sapphire, recording | Four trumpets searched, BitFlag8 1–4, notice 0 `[count, 4]`. |
| 379 | In the Grim Darkness of the Forest | `003/SubFst058_00379` | client script, Sapphire, recording | Notice 0 at Roseline; the bag's key item (UI8BH) with notice 1 `[25919]`; trade (scene 5 → 94). The campfire can be searched in every sequence but does nothing. |
| 380 | Threat Level Elevated | `003/SubFst059_00380` | client script, Sapphire, recording | A letter traded along three NPCs; key items in UI8BH/BL/CH; notices 0 and 1 with `[26151]`. |
| 381 | Migrant Marauders | `003/SubFst060_00381` | client script, Sapphire, recording | Searching the egg spawns vultures 3838969/3838970; notice 0 only on the second kill; the egg can be searched again. |
| 384 | A Hearer Is Often Late | `003/SubFst068_00384` | client script, Sapphire, recording | Notice 0 at Leonnie. |
| 387 | Salvaging the Scene | `003/SubFst073_00387` | client script, Sapphire, recording | Two cargoes (BitFlag8 1–2), notice 0 `[count, 2, 21003]`; walking into EVENTRANGE0 spawns the lumber toad 3842567 (scene 22); trade 23 → 24. |
| 161 | Leia's Legacy | `001/SubFst055_00161` | client script, Sapphire, recording | Ask three witnesses in any order (UI8AL counts, BitFlag8 1–3, notice 0 `[count, 3]`), report (cutscene 4, notice 1), quest battle 14, the egg as a key item (UI8BH, notice 3 `[25252]`), trade 15 → 16. |
| 446 | Dread Is in the Air | `004/ManFst006_00446` | client script, Sapphire, recording | Accepting goes straight to 255 with the letter (UI8BH); trade 1 → 100 (99 when the window is closed). |
| 447 | To Guard a Guardian | `004/ManFst007_00447` | client script, Sapphire, recording, Chronofoil capture (quest battle, 2026-10-17) | Message (UI8BH) for Lewin through the Seat of the First Bow's door (scene 5, POPRANGE0); trade 2 → 3, notice 0 `[1, 1]`; quest battle 12 from the instructor (scene 7) or EVENTRANGE0 (scene 6); the aftermath on re-entering (scene 20). |
| 448 | Festive Endeavors | `004/ManFst008_00448` | client script, Sapphire, recording | Miounne (notice 0), Beatin gives the mask (UI8BH, notice 1 `[21002]`), trade 5 → 6. |
| 449 | Renewing the Covenant | `004/ManFst009_00449` | client script, Sapphire, recording, Chronofoil capture (the ceremony) | Estaine checks the player is wearing RITEM0 (scene 3 → 4, or 5 without it); the ceremony (cutscene 4) leaves you at POPRANGE0 in the Roost (179); Lotus Stand door (scene 8, POPRANGE1). |
| 507 | The Gridanian Envoy | `005/ManFst200_00507` | Sapphire, recording, Chronofoil capture (gangplanks) | Two letters (UI8BH/BL); airships to Limsa Lominsa and Ul'dah: each gangplank plays a check scene (6, 28), the Waiting bar (event action 11), then the yes/no (7, 29) whose answer is its error code (50 = No), then the flight (8, 30) and `change_territory`; notices 0, 3, 4, 7; positions are Sapphire's. Scenes not yet checked against the client's script (needs a dump). |
| 674 | Call of the Sea | `006/SubWil141_00674` | Sapphire, recording | Accepting goes straight to 255; Baderon gives the reward. Scenes not yet checked against the client's script. |
| 673 | Call of the Sea (from Ul'dah) | `006/SubFst102_00673` | client script (2026-10-20), Chronofoil capture (2026-10-18) | ACTOR0 (the serpent officer, Old Gridania) offers it (scene 0; straight to 255) and talks meanwhile (scene 2); ACTOR1 (Baderon, Limsa Lominsa) rewards it (scene 1, a yield). |
| 594 | Coming to Ul'dah | `005/ManWil001_00594` (upstream) | client script, recording, Chronofoil captures (2026-10-17, re-checked on 2026-10-18) | Upstream's script matches retail unchanged: the offer and two scenes come back as yields, the opening (scene 30) nested in the accept. Ends the event for the objects the client also allows in sequence 255 (EOBJECT0–3), and completes the quest only when the reward scene returns 1 (client script, 2026-10-19). |
| 568 | Close to Home (Ul'dah) | `005/ManWil002_00568` | client script, recording, Chronofoil captures (2026-10-17, re-checked on 2026-10-18) | Moves the player from the opening's Ul'dah into the real one (130) with ITEM0 (UI8CH); three objectives in any order: hand ITEM0 to the guild (trade scene 4 → 5; UI8BH, BitFlag8 3, notice 2), attune (scenes 1 → 2, then Return; UI8BL, BitFlag8 1, notice 0), talk to ACTOR1 (scene 3; UI8AL, BitFlag8 2, notice 1); UI8AH counts them. The Pugilist and Thaumaturge versions (569, 570) are the same script with their own ITEM0 and ACTOR1. The aetheryte step never started in game until aetherytes got a `BASE_ID`; its attunement (bar, nested aetheryte event) checked against Bob's Chronofoil capture of the quest (2026-10-18). |
| 569 | Close to Home (Ul'dah, Pugilist) | `005/ManWil003_00569` | client script (2026-10-19; identical to 568's apart from ITEM0, ACTOR1 and cutscene staging); replay fixture adapted from 568's, no retail capture of this version | 568's flow with this version's ITEM0 and ACTOR1: the guild hand-over (scenes 4 → 5, notice 2), attuning (nested aetheryte event, EventAction 19, scenes 1 → 2, notice 0), ACTOR1 (scene 3, notice 1), in any order; ACTOR3 rewards it (scene 8). |
| 570 | Close to Home (Ul'dah, Thaumaturge) | `005/ManWil004_00570` | client script (2026-10-19; identical to 568's apart from ITEM0, ACTOR1 and cutscene staging); replay fixture adapted from 568's, no retail capture of this version | As 569, with this version's ITEM0 and ACTOR1. |
| 595 | We Must Rebuild | `005/SubWil027_00595` | client script, recording, Chronofoil captures (2026-10-17, re-checked on 2026-10-18) | Accepting goes straight to 255; ACTOR1 gives the chosen reward. |
| 671 | Nothing to See Here | `006/SubWil025_00671` | client script, recording, Chronofoil captures (2026-10-17, re-checked on 2026-10-18) | Three ITEM0 (UI8CH counts down) handed to three NPCs in any order (trade 1/4/7 → 2/5/8; UI8AL/BH/BL and BitFlag8 1–3), UI8AH counts, notice 0 `[count, 3]` each; reward from ACTOR0 (scene 10); ACTOR1–3 have a single line each in sequence 255 (scenes 11–13, client script). |
| 550 | Underneath the Sultantree | `005/ManWil005_00550` | client script, Sapphire, recording, Chronofoil captures (2026-10-17, re-checked on 2026-10-18) (quest battle) | Offer and cutscene (0 → 1); ACTOR1 offers quest battle 37 (scene 2); the aftermath on re-entering Central Thanalan (scene 5) moves it to 255; reward (scene 6). No variables or notices at all. |
| 303 | Step Nine | `003/SubWil060_00303` | client script (2026-10-19), Sapphire, Chronofoil capture (2026-10-18); tested in game by Bob (2026-10-19) | ACTOR0 offers it (scene 0, a yield) and gives ITEM0 (UI8BH); ACTOR1 takes it (hand-over scene 1, then 2) and gives ITEM1 (UI8BH 0, UI8BL 1, notice 0 with its icon 25210), sequence 255; ACTOR2 takes ITEM1 (scene 4) and rewards (scene 5, a yield). Parameters from Sapphire's SubWil060 until a dump confirms them. |
| 3852 | Prudence at This Junction | `038/XxaWil063_03852` | client script, Chronofoil capture (2026-10-18); tested in game by Bob (2026-10-19) | ACTOR0 (Roger) offers it (scene 0, a yield, then scene 1; accepted after 1); ACTOR1 (Warin) moves it to 255 (scene 2, notice 0, no variables) and rewards it (scene 3, a yield with the chosen item). The client refuses the offer while 324, 305 or 306 is accepted, and the reward without item level 5 in head, body, hands, legs and feet. Not the same quest as 306 (SubWil063, same name, now a side quest). |
| 307 | Out of House and Home | `003/SubWil064_00307` | client script (2026-10-19), Sapphire, Chronofoil capture (2026-10-18) | Warin offers it (scene 0, a yield; ITEM0 in UI8BH); talking to the fissure (EOBJECT0) only plays scene 1; using ITEM0 on it (a 3 s cast, the EventItem event) plays scene 2 and brings out a coblyn (ENEMY0); its death counts UI8AL 1 and brings out two more (ENEMY1, ENEMY2); the third kill sends notice 0, clears the variables, sequence 255 (Sapphire counted to 4); Warin rewards it (scene 13). The coblyns are drop-in layer `QB_SubWil064_001` with retail's positions, levels (6, 4, 4) and HP. |
| 320 | Way Down in the Hole | `003/SubWil066_00320` | client script (2026-10-19), Chronofoil capture (2026-10-18), Sapphire's quest battle | Offer (0, then 1; the client refuses it while QST_ACCEPT_CHECK is accepted); the foreman (ACTOR1) gives four ITEM0 (scene 2, UI8CL 4, notice 0 `[24051]`); one to each of ACTOR2–5 in any order (yield hand-over scenes 3/6/9/12 then 4/7/10/13 with 0x6001; UI8AH counts, notice 1 `[n, 4]`; UI8AL/BH/BL/CH and BitFlag8 1–4 mark each one done, UI8CL the items left; the foreman's talk scene 15 meanwhile); the foreman's cutscene (16, notice 2); ACTOR6 sends you into QUESTBATTLE0 18 (17); winning moves to 5; back in TERRITORYTYPE1 (141) the aftermath (26, an enter-territory event) moves to 6; the foreman (27, notice 5) to 255; Momodi (ACTOR13) rewards it (28). ACTOR7–12 and EOBJECT0 have only empty scenes in sequence 4. |
| 623 | Takin' What They're Givin' | `006/SubWil026_00623` | client script (2026-10-19), Chronofoil capture (2026-10-18) | ACTOR0 (Momodi) offers it (scene 0, a yield; accepting goes straight to 255); ACTOR1 (Dadanen) in Western Thanalan: cutscene (scene 1, 0xF8402EFB), then the reward (scene 2, a yield). |
| 328 | Supply and Demands | `003/SubWil080_00328` | client script (2026-10-19), Chronofoil capture (2026-10-18) | ACTOR0 (Dadanen) offers it (scene 0) and gives ITEM0 (UI8BH 1), sequence 255; ACTOR1 (the Drunken Stag) takes it (hand-over scene 1), then the reward (scene 2, a yield; the client cancels the trade if it isn't 1). |
| 503 | Give It to Me Raw | `005/SubWil095_00503` | client script (2026-10-19), Chronofoil capture (2026-10-18) | ACTOR0 (the Drunken Stag) offers it (scene 0, a yield), sequence 1; three ore mounds in any order, each its scene (1/4/7), the gathering bar (EventAction 6, EVENT_ACTION_GATHER_SHORT), then its scene (2/5/8), all 0x2001, after which its marshlight (ENEMY0–2) comes out; each kill sets the mound's own nibble (UI8AL/BH/BL), its BitFlag8 (1–3), one more ore (ITEM0 in UI8CH) and UI8AH, with notice 0 `[n, 3, 20004]` (20004 probably ITEM0's icon); the third moves the ores to UI8BH (3) and goes to 255; trade 10 → reward 11 (yields). The marshlights are drop-in layer `QB_SubWil095_001` with retail's positions, level 10 and HP 192. |
| 329 | The Perfect Swarm | `003/SubWil081_00329` | client script (2026-10-19), Chronofoil capture (2026-10-18) | ACTOR0 (the Drunken Stag) offers it (scene 0), sequence 1; four ENEMY0 (tainted water sprites, BNpcBase 136, from the Western Thanalan drop-in) counted in UI8AL with notice 0 `[n, 4]`; the fourth clears it, sequence 255; ACTOR1 (Fufulupa) rewards it (scene 1). Needs Give It to Me Raw (503) first. The fixture's kills use stand-in layout IDs (the capture doesn't tie a death to a layout). |
| 330 | Last Letter to Lost Hope | `003/SubWil082_00330` | client script (2026-10-19), Chronofoil capture (2026-10-18) | ACTOR0 (Fufulupa) offers it (scene 0), sequence 1; in Central Thanalan ACTOR1 (Seseli) gives ITEM0 (scene 1: UI8BH 1, notice 0 `[25918]`), sequence 255; ACTOR2 only talks (scene 2); ACTOR3 (Leofric) takes it (hand-over scene 3), then the reward (scene 4, a yield; gil only). Scenes 6 and 7 (ACTOR1 and ACTOR2 in 255) are matched to them by order, unseen on retail. |
| 3853 | Heir Today, Gone Tomorrow | `038/XxaWil083_03853` | client script (2026-10-19), Chronofoil capture (2026-10-18) | ACTOR0 (Leofric) offers it (scene 0, a yield, then scene 1; accepted after 1; the client refuses it while QST_ACCEPT_CHECK, quest 331, is accepted); walking into EVENTRANGE0 plays scene 2 (0x2001), after which ENEMY0, a mildewed goobbue (layout 4142933), comes out; its death sends notice 0 (no arguments) and moves to 255, with no variables at all; ACTOR0 rewards it (scene 5, a yield). EOBJECT0 and ENEMY0 have only empty scenes (3, 4). The goobbue is drop-in layer `QB_XxaWil083_001` with retail's position, level 12, HP 243 and display flags; it casts 968 (0.7 s, 8–9 damage at level 12) about every 3.1 s instead of auto-attacking (timeline `MildewedGoobbue_355`). |
| 332 | Passing the Blade | `003/SubWil084_00332` | client script (2026-10-19), Chronofoil capture (2026-10-18) | ACTOR0 (Leofric) offers it (scene 0) and gives ITEM0 (UI8BH 1), sequence 255; ACTOR1 (Fufulupa) takes it (hand-over scene 1), then the reward (scene 2, a yield; gil only). Needs Heir Today, Gone Tomorrow (3853). |
| 333 | Following Footfalls | `003/SubWil085_00333` | client script (2026-10-19), Chronofoil capture (2026-10-18) | ACTOR0 (Fufulupa) offers it (scene 0; straight to 255); ACTOR1 (Totoruna) rewards it (scene 1, a yield). |
| 334 | Storms on the Horizon | `003/SubWil086_00334` | client script (2026-10-20), Chronofoil capture (2026-10-18) | ACTOR0 (Nunuzofu) offers it (scene 0; straight to 255); ACTOR1 (Raffe) rewards it (scene 1, a yield). |
| 336 | Oh Captain, My Captain | `003/SubWil088_00336` | client script (2026-10-20), Chronofoil capture (2026-10-18; two lost runs and a won one) | ACTOR0 (Merilda) offers it (scene 0, a yield; the client refuses it while QST_ACCEPT_CHECK is accepted), then cutscene 1 (0xF8402EFB); accepted after it, sequence 1; ACTOR1 (Baldewyn) sends you into QUESTBATTLE0 39 (scene 2, YesNoQuestBattle; offered while UI8AL and BitFlag8 1 are unset); losing leaves the quest as it was, winning moves it to 255 inside the battle; ACTOR2–6 and EOBJECT0 have only empty scenes (3–8); ACTOR7 (Fufulupa) rewards it (scene 9, a yield). No variables or notices. |
| 628 | Secrets and Lies | `006/ManWil006_00628` | client script (2026-10-20), Chronofoil capture (2026-10-18) | ACTOR0 (Fufulupa) offers it (scene 0) and gives ITEM0 (UI8BH 1), sequence 255; ACTOR1 (Momodi, Ul'dah) takes it (hand-over scene 1), then the reward in a cutscene (scene 2, 0xF8402EFB, a yield; gil only). Needs Oh Captain, My Captain (336). |
| 551 | Duty, Honor, Country | `005/ManWil007_00551` | client script (2026-10-20), Chronofoil capture (2026-10-18, quest battle: three runs) | ACTOR0 (Momodi) offers it (scene 0, a yield), cutscene 1, scene 2 (QuestAccepted): ITEM0 in UI8BH, sequence 1; ACTOR1 outside TERRITORYTYPE0 (210) takes you to POPRANGE0 there (scene 3, a yes/no yield; leaving is the exit's own warp); ACTOR2 (Owyne) takes ITEM0 (hand-over 4 → cutscene 5: notice 0 `[1, 1]`, UI8BH 0, sequence 2); EOBJECT0 in Central Thanalan: scene 7, the Waiting bar (EventAction 13, about 10 s), scene 8 (0x2001, YesNoQuestBattle) into QUESTBATTLE0 38 (TERRITORYTYPE1 271); winning moves to 3; back in 141 the aftermath (scene 10, an enter-territory event) moves to 255; ACTOR1 takes you in again (scene 11); ACTOR3 (Papashan) rewards it (scene 12, a yield), then the closing cutscenes (scene 13, given the chosen item; retail sets UI8AH and UI8AL to 1 as it starts). ACTOR2 has lines in 2 and 255 (scenes 9, 14). |
| 641 | A Matter of Tradition | `006/ManWil008_00641` | client script (2026-10-20), Chronofoil capture (2026-10-18) | ACTOR0 (Owyne, in 210) offers it (scene 0, a yield, then 1), sequence 1; ACTOR1 (the Sultansworn elite outside) asks whether to go in (scene 2, a yes/no) and takes you back to POPRANGE0 in 210 to accept it (Bob couldn't once he'd left the room before accepting, 2026-10-20); ACTOR2 (Momodi) (scene 3: notice 0), sequence 2, ACTOR0 talks meanwhile (4); ACTOR3 (Serendipity) gives ITEM0 (scene 5: notice 1 `[21002]`, UI8BH 1), sequence 255; ACTOR2 takes it (hand-over 6) and rewards it (7: gil and item 4196, which is what makes the client offer Serendipity in A Royal Reception). |
| 552 | A Royal Reception | `005/ManWil009_00552` | client script (2026-10-20), Chronofoil capture (2026-10-18) | ACTOR0 (Momodi) offers it (scene 0, a yield, then 1, where the client accepts), sequence 1; ACTOR1 (Bartholomew) talks (scenes 2, 3), then the cutscene (4, 0xF8402EFB), after which the player is at POPRANGE0 in the Hourglass (178) and the quest at 255; ACTOR2 (Serendipity), offered only while you have RITEM0, talks (scene 5, a guess between the client's 5 and 6); ACTOR3 (Raubahn) rewards it (scene 7, then 8, a yield; gil only). |
| 528 | The Ul'dahn Envoy | `005/ManWil200_00528` | client script (2026-10-20), Chronofoil capture (2026-10-18) | Raubahn offers it (scene 0, a yield; cutscene 1 accepts it) with two letters (UI8BH/BL); Momodi (cutscene 2, notice 0); Elyenora's yes/no (4) takes you to the landing (POPRANGE0); the gangplank: scene 6, the Waiting bar (event action 11), the yes/no 7 (error code 50 = No), flight 8 to Limsa Lominsa (POPRANGE1); storm honor guard (11, notice 3); attendant down (12, POPRANGE2); Zanthael takes ITEM0 (hand-over 16 → 17 → cutscene 18, notice 4); L'nophlo (24, POPRANGE3); gangplank 28 → bar → 29 → flight 30 to Gridania (POPRANGE4); serpent honor guard (35, notice 7); attendant down (37, POPRANGE5); the silent conjurer takes ITEM1 (42 → 43 → cutscene 44) and rewards it (45, a yield). The ticketers' later yes/no + airship cutscenes (14/15 … 51/52) fly you to the next city's arrival, from the client script only; EOBJECT0/2/4's scenes are empty. |
| 107 | Coming to Limsa Lominsa | `001/ManSea001_00107` (upstream) | client script (2026-10-19), Chronofoil capture (2026-10-20) | Ryssfloh offers it (scene 0, a yield; decline is scene 4), cutscene 1 (`DISABLE_SKIP`), scene 2 (a yield) and the client's empty scene 3, which accepts it on returning (sequence 1) and calls the opening's scene 30 with 2, nested. Sequence 1: Grehfarr's Crow's Lift (scene 5, a yield: 1 rides; scene 6), then PrepareZoning within 181 to POPRANGE0 (`move_to_pop_range`) and sequence 255. Baderon's cutscene (11), then the reward (12, a yield). Upstream's script went straight to 255 on accepting and skipped scene 3. |
| 108 | Close to Home (Limsa Lominsa, Marauder) | `001/ManSea002_00108` | client script (2026-10-19); fixture adapted from 109's retail one, no retail recording of this version | As 109 with its own guild NPC (ACTOR2) and ITEM0. |
| 109 | Close to Home (Limsa Lominsa, Arcanist) | `001/ManSea003_00109` | client script (2026-10-19), Chronofoil capture (2026-10-20) | Offer (0), accept cutscene (1) with ITEM0 (UI8CH); unlike the other cities, accepting doesn't leave the opening's zone (181; quest 111 does). Three objectives in any order: attune in the Lower Decks (the nested aetheryte event and bar, scenes 2 → 3, then Return; UI8BL, BitFlag8 1, notice 0), hand ITEM0 to ACTOR1 (scene 4, a yield, then 5; UI8BH, BitFlag8 2, notice 1), visit the guild (ACTOR2, scene 6; UI8AL, BitFlag8 3, notice 2); UI8AH counts; reward from ACTOR3 in the Upper Decks (scene 7). Until 2026-10-20 accepting moved you into the Upper Decks (128, from Sapphire's ENpc positions). |
| 111 | Making a Name | `001/SubSea001_00111` | client script (2026-10-21), Chronofoil capture (2026-10-20) | Offered in the Limsa opening after 107, alongside Close to Home (scene 0, a yield; the client's empty scene 1, then sequence 1; retail sends no AcceptQuest). ACTOR1 at the opening's exit (scene 2: the client only offers the yes/no once Close to Home, QUEST0 or QUEST1, is accepted) sets sequence 2 and takes you through a loading screen into the Lower Decks (TERRITORYTYPE0 129, POPRANGE0); asked again in sequence 2 it's scene 15. Six herbs (EOBJECT0..5): scene 2n + 3, EVENT_ACTION_PROCESS (14), scene 2n + 4, all 0x2001, notice 1 `[count, 6, 25002]`; UI8AL counts, UI8BH is ITEM0 held, BitFlag8 n + 1; the sixth leaves only UI8BH (6) in sequence 255. ACTOR2 takes the herbs (scene 16, an NpcTrade), then the reward (17). |
| 462 | On to Summerford | `004/SubSea050_00462` | client script (2026-10-21), Chronofoil capture (2026-10-21) | ACTOR0 (Upper Decks) offers it (scene 0, a yield; the client's empty scene 1, then accepted straight into 255); ACTOR1 at Summerford Farms: the cutscene and reward (scene 2, 0xF8402EFB, a yield with the reward picked). |
| 463 | Dressed to Call | `004/SubSea051_00463` | client script (2026-10-21), Chronofoil capture (2026-10-21) | As Passing Muster: ACTOR0 offers it (scene 0, accepted into 255); scene 1 checks your gear's item level on the client (checkEquippedItemLevel) and returns 1 and the reward, or 0. |
| 543 | Lurkers in the Grotto | `005/ManSea005_00543` | client script (2026-10-21), Sapphire, Chronofoil capture (2026-10-21, quest battle lost then won) | ACTOR0 offers it (scene 0, a yield; cutscene 1, sequence 1). The stone monument (EOBJECT0): scene 2, EVENT_ACTION_PROCESS_MIDDLE (16), the quest battle's yes/no (scene 3), all 0x2001; quest battle 35 in territory 252. Winning sets sequence 2 and UI8BH = 1 (ITEM0) in one update; back in Middle La Noscea (TERRITORYTYPE1) the aftermath (scene 4, 0xF8402EFB) sets 255; ACTOR0 takes ITEM0 (scene 5, an NpcTrade) and the reward (6). Retail then sends FinishQuest for 464 too (`mark_quest_completed`: no rewards), which Washed Up checks. |
| 466 | Double Dealing | `004/SubSea054_00466` | client script (2026-10-21), Chronofoil capture (2026-10-21) | ACTOR0 offers it (scene 0, a yield). ACTOR1 won't talk (scene 1) until you use an emote on him (scene 2, the first emote event: notice 0, sequence 2). EVENTRANGE0: scene 16 (a pop message), then the ambush, ENEMY1 and ENEMY0 (layer `QB_SubSea054_001` of `sea_s1f1`; killing them changes nothing). ACTOR2..4 in any order: scene 3n + 1, EVENT_ACTION_ (35), scene 3n + 2 (0x2001), notice 1 `[count, 3]`, UI8AL counts, BitFlag8 n - 1; ACTOR1 (scene 24) notice 2; EOBJECT0, the sack of oranges: scene 28, EVENT_ACTION_PROCESS_MIDDLE, scene 29, ITEM0 (UI8BH, notice 3 `[25919]`); ACTOR5 takes it (trade 33 → 34) for ITEM1 (UI8BL, notice 4); ACTOR0 takes that (trade 36 → reward 37). |
| 467 | Loam Maintenance | `004/SubSea055_00467` | client script (2026-10-21), Chronofoil capture (2026-10-21) | ACTOR0 offers it (scene 0); ACTOR1 (scene 1) notice 0. EOBJECT0..2 in any order: scene n + 2, EVENT_ACTION_PROCESS_MIDDLE, scene 100 - 2n (0x2001), notice 1 `[count, 3, 21466]`; UI8AH counts, UI8CH the items held, UI8AL/BH/BL and BitFlag8 n + 1 the object; the third leaves UI8BH = 3 in 255. ACTOR2 takes them (trade 5 → reward 94). |
| 468 | Plowshares to Swords | `004/SubSea056_00468` | client script (2026-10-21), Chronofoil capture (2026-10-21) | ACTOR0 offers it (scene 0). Three kills of ENEMY0 or ENEMY1 (each gave an item on retail: three of ENEMY1), notice 0 `[count, 3, 21002]`, UI8AL counts, UI8BH the items; the third leaves UI8BH = 3 in sequence 2. ACTOR0 takes them (trade 1 → 2, notice 1, 255); ACTOR1 rewards it (scene 4). |
| 469 | Just Deserts | `004/SubSea057_00469` | client script (2026-10-21), Chronofoil capture (2026-10-21, quest battle won) | ACTOR0 offers it (scene 0, then the cutscene 1, accepted after it). ACTOR1 (2), ACTOR2 (3), ACTOR4 (10, EVENT_ACTION_, 11): notices 0–2, sequences 2–4. ACTOR5: scene 18, EVENT_ACTION_, the quest battle's yes/no (19): quest battle 29 in territory 272; winning sets 5; back in Middle La Noscea the aftermath (26, notice 4, 6); ACTOR0 (27, notice 5, 7; 29, notice 6, 255); ACTOR10 in Limsa Lominsa rewards it (30, 0xF8482EFB; the server grants the unlocks). The other NPCs' one-liners from the client script. |
| 397 | Sky-high | `003/SubSea100_00397` | client script (2026-10-21), Chronofoil capture (2026-10-21) | ACTOR0 (Limsa Lominsa) offers it (scene 0, a yield; the client's empty scene 1, then accepted straight into 255); ACTOR1 rewards it (scene 2). Retail then sends FinishQuest for 398 too (`mark_quest_completed`). |
| 402 | Thanks a Million | `004/SubSea105_00402` | client script (2026-10-21), Chronofoil capture (2026-10-21) | ACTOR0 offers it (scene 0). Five spots, EOBJECT0..4: scene n + 1, EVENT_ACTION_SEARCH, then 100 - 2n (found) or 99 - 2n (not); only EOBJECT0's found scene has anything (the ambush message), so it has ITEM0: notice 0 `[25004]`, UI8BH = 1, 255, then ENEMY1 and ENEMY0 come out (layer `QB_SubSea105_001` of `sea_s1f1`). ACTOR1 in Western La Noscea takes it (trade 6 → reward 90); retail then completes 404 too. |
| 403 | Relighting the Torch | `004/SubSea106_00403` | client script (2026-10-21), Chronofoil capture (2026-10-21) | ACTOR0 offers it (scene 0); ACTOR1 gives ITEM0 (scene 1, notice 0 `[21003]`, UI8BH); ITEM0 used on EOBJECT0 (scene 2 is the client's prompt, then the key item's event: scene 3, notice 1); EOBJECT1: scene 5, EVENT_ACTION_ (32), scene 6, then ENEMY0 comes out (layer `QB_SubSea106_001` of `sea_s1f4`, Western La Noscea); killing it gives ITEM1 (notice 2 `[22623]`, UI8BL, 4); ACTOR1 takes it (trade 11 → 12, notice 3, 5); ACTOR0 (17, notice 4, 255); ACTOR2 in Limsa Lominsa rewards it (18). |
| 406 | On to the Drydocks | `004/SubSea109_00406` | client script (2026-10-21), Chronofoil capture (2026-10-21) | ACTOR0 offers it with ITEM0 (scene 0: accepted straight into 255, UI8BH = 1); ACTOR1 in Lower La Noscea takes it (trade 1) and the cutscene and reward follow (100, 0xF8402EFB). |
| 465 | Washed Up | `004/SubSea053_00465` | client script (2026-10-21), Chronofoil capture (2026-10-21) | ACTOR0 offers it (scene 0, a yield: sequence 1; the client refuses it while QST_ACCEPT_CHECK, 464, is accepted). ACTOR1..5 in any order: ACTORn plays scene n (0x2001, a menu that changes nothing here), notice 0 `[count, 5]`, UI8AL counts, BitFlag8 n; the fifth sets 255. ACTOR0: the reward (scene 6). |
| 471 | Rocks in a Hard Place | `004/SubSea059_00471` | client script (2026-10-22), Chronofoil capture (2026-10-21) | ACTOR0 offers it (scene 0). The shovel (EOBJECT0): scene 1, EVENT_ACTION_SEARCH_SHORT (2), scene 100: ITEM0 with three uses (UI8BL), notice 0 `[21003]`. Then in any order: the rocks (EOBJECT1..3), scene n + 1 is the item window and using ITEM0 on one plays 100 - 2n (UI8BH counts, UI8BL down by one, notice 1 `[count, 3]`); the debris (EOBJECT4..6), scene n + 1, EVENT_ACTION_SEARCH, 100 - 2n (UI8AL counts, the third sets UI8AH, notice 2 `[count, 3]`); BitFlag8 n for each; all six clear the variables and set 255. ACTOR0 rewards it (8). |
| 412 | Without a Doubt | `004/SubSea115_00412` | client script (2026-10-21), Chronofoil capture (2026-10-21) | ACTOR0 offers it (scene 0, accepted straight into 255). ACTOR1 won't say (scene 1) until you use an emote on him: emote 12 (retail's, the only one seen) plays scene 100, the reward; any other plays 99. |
| 415 | Righting the Shipwright | `004/SubSea118_00415` | client script (2026-10-21), Chronofoil capture (2026-10-21) | ACTOR0 offers it (scene 0); ACTOR1 (1, notice 0). EOBJECT0..2 in any order: scene 6n + 2, EVENT_ACTION_GATHER_SHORT, 6n + 3 (0x2001), notice n + 1 with the item's icon (21602, 21003, 20910), then ENEMYn comes out (base 170, layer `QB_SubSea118_001` of `sea_s1f2`); UI8AH counts, UI8AL/BH/BL and BitFlag8 n + 1 the object, UI8CH/CL/DH the items; the third moves them to UI8BH/BL/CH, sequence 3. ACTOR1: a line (20) in 2; takes the items (trade 21 → 22, notice 4, 255); ACTOR2 rewards it (24). |
| 413 | Do Angry Pirates Dream | `004/SubSea116_00413` | client script (2026-10-21), Chronofoil capture (2026-10-21) | ACTOR0 offers it (scene 0); ACTOR1 (1, notice 0), ACTOR2 (2, notice 1), ACTOR3's cutscene (3, 0xF8402FFF, notice 2 `[1, 1]`, 255); ACTOR4 and ACTOR5 have a line each (4, 5); ACTOR1 rewards it (12). |
| 414 | Victory in Peril | `004/SubSea117_00414` | client script (2026-10-21), Chronofoil capture (2026-10-21, quest battle lost then won) | ACTOR0 offers it (scene 0); ACTOR1 (1) or ACTOR2 (2): notice 0 `[1, 1]`, 2; ACTOR3 (3, notice 1, 3); ACTOR0: the quest battle's yes/no (4): quest battle 24 in territory 249; winning sets 4; ACTOR4 (7, notice 3, 255); ACTOR0 rewards it (8; closing the window leaves UI8AH/AL at 1, as retail). |
| 689 | Men of the Blue Tattoos | `006/ManSea006_00689` | client script (2026-10-21), Chronofoil capture (2026-10-21) | ACTOR0 offers it (scene 0, the client's empty scene 1: accepted into 255 with ITEM0, UI8BH); Baderon takes it (trade 2) and the cutscene and reward follow (3, 0xF8402EFB). |
| 544 | Feint and Strike | `005/ManSea007_00544` | client script (2026-10-21), Chronofoil capture (2026-10-21, quest battle lost three times, then won) | Baderon offers it (scene 0, cutscene 1: sequence 1 with ITEM0, UI8BH); ACTOR1 takes it (trade 2 → cutscene 3, notice 0, 2); ACTOR2 in Western La Noscea: the quest battle's yes/no (4): quest battle 36 in territory 280; ACTOR3..5's lines (5, 6, 7); EOBJECT0's scenes are empty. Winning sets 3; back in Western La Noscea the cutscene (10) sets 255; ACTOR1 (11), then the cutscene and reward (12). |
| 690 | High Society | `006/ManSea008_00690` | client script (2026-10-21), Chronofoil capture (2026-10-21) | ACTOR0 offers it (scene 0, the client's empty 1: sequence 1); Baderon (2, notice 0, 2); ACTOR2 in the Lower Decks (3, notice 1 `[21002]`, ITEM0 in UI8BH, 255); Baderon takes it (trade 4) and rewards it (5). |
| 545 | A Mizzenmast Repast | `005/ManSea009_00545` | client script (2026-10-21), Chronofoil capture (2026-10-21) | As A Royal Reception: Baderon offers it (scene 0, the empty 1: sequence 1); Zanthael (ACTOR1, offered while you hold RITEM0): scenes 2, 3, the cutscene 4, then you're moved to the Mizzenmast inn (177, POPRANGE0) at 255; ACTOR2's line (6, with RITEM0); Zanthael's yes/no (8) takes you to the Command room (198, POPRANGE1); Merlwyb rewards it (7). Scene 5 (Zanthael) is played by nothing. |
| 546 | The Lominsan Envoy | `005/ManSea200_00546` | client script (2026-10-21), Chronofoil capture (2026-10-21) | The Ul'dahn Envoy from Limsa Lominsa: Merlwyb offers it (scene 0, cutscene 1) with two letters (UI8BH/BL); Baderon (cutscene 3, notice 0); L'nophlo's yes/no (4) to the landing (POPRANGE1); the gangplank: 6, the Waiting bar (11), the yes/no 7, flight 8 to Gridania (POPRANGE2); the attendant down (12, POPRANGE3); the serpent honor guard (11, notice 3); ACTOR6 in Old Gridania takes ITEM0 (16 → 17 → cutscene 18, UI8BL cleared as retail, notice 4); Lionnellais (24, POPRANGE4); gangplank 28 → bar → 29 → flight 30 to Ul'dah (POPRANGE5); the attendant down (36, POPRANGE6); the flame honor guard (35, notice 7); ACTOR10 at the Steps of Thal takes ITEM1 (42 → 43 → cutscene 44) and rewards it (45). Zanthael's yes/no (scene 2) back into the Command room and the ticketers' later flights (14/15 … 51/52) are from the client script only. |
| 245 | It's Probably Pirates | `002/ManSea203_00245` | client script (2026-10-23), Chronofoil capture (2026-10-23, Sastasha with Duty Support) | ACTOR0 (Baderon) offers it (scene 0, a yield, then cutscene 1), sequence 1; ACTOR1 (V'mellpa, scene 2, notice 0); ACTOR2 (the seasoned adventurer, scene 3, notice 1, unlocks REWARD_BEGINNER_01, 247); ACTOR3 (Yellowjacket, scene 5) unlocks Sastasha (`unlock_content(INSTANCEDUNGEON0)`), notice 2, sequence 4; leaving Sastasha cleared (`onDutyCleared`) sends notice 3, sequence 255; ACTOR0 rewards it (cutscene 21, a yield) and quest 675 is marked completed too. One line each meanwhile: ACTOR1 (4, 13), ACTOR2 (6), ACTOR4–9 (7–12, 14–19), ACTOR3 (20), matched to them by order. No variables. |
| 676 | Call of the Forest | `006/SubSea150_00676` | client script (2026-10-24), Chronofoil capture (2026-10-24) | ACTOR0 (Baderon, Limsa Lominsa) offers it (scene 0, a yield; straight to 255); ACTOR1 (Mother Miounne, New Gridania) rewards it (scene 1, a yield). Needs It's Probably Pirates (245). |
| 677 | Fire in the Gloom | `006/ManFst204_00677` | client script (2026-10-24), Chronofoil capture (2026-10-24, the Tam-Tara Deepcroft with Duty Support) | ACTOR0 (Mother Miounne) offers it (scene 0, a yield, then cutscene 1), sequence 1; ACTOR1 (the Gods' Quiver bow, Central Shroud) unlocks the Tam-Tara Deepcroft (scene 2: `unlock_content(INSTANCEDUNGEON0)`, InstanceContent 2), notice 0, sequence 2; leaving it cleared (`onDutyCleared`) sends notice 1, sequence 255; ACTOR0 rewards it (cutscene 14, a yield). One line each meanwhile: ACTOR2–6 (3–7 in sequence 1, 8–12 in 2), ACTOR1 (13). No variables. |
| 678 | Call of the Desert | `006/SubFst103_00678` | client script (2026-10-25), Chronofoil capture (2026-10-25) | ACTOR0 (Mother Miounne, New Gridania) offers it (scene 0, a yield; straight to 255); ACTOR1 (Momodi, Ul'dah) rewards it (scene 1, a yield). |
| 660 | Into a Copper Hell | `006/ManFst205_00660` | client script (2026-10-25), Sapphire, Chronofoil capture (2026-10-25: the Copperbell Mines three times, quest battle 44 declined once and entered four times) | Momodi offers it (scene 0, a yield, then cutscene 1), sequence 1; Painted Mesa (ACTOR1, scene 2, notice 0), 2; Stone Torch (ACTOR2) unlocks the Copperbell Mines (scene 3: `unlock_content(INSTANCEDUNGEON0)`, InstanceContent 3), notice 1, 3; leaving it cleared (`onDutyCleared`) sends notice 2, 4; Painted Mesa's YesNoQuestBattle (scene 10, a yield) enters QUESTBATTLE0 44 (`join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)`, territory 274); the battle's director sets 5 on a win; back in Ul'dah (130) the cutscene (12, `onEnterTerritory`) sets 255; Momodi rewards it (cutscene 13, a yield). Lines meanwhile: ACTOR1/3/4 in 2 (6, 4, 5), ACTOR2/3/4 in 3 (9, 7, 8); EOBJECT0's scene 11 is empty. No variables. |
| 509 | The Scions of the Seventh Dawn | `005/ManFst206_00509` | client script (2026-10-25), Sapphire, Chronofoil capture (2026-10-25) | Momodi offers it (scene 0, a yield, then 1), sequence 1; Tataru in Vesper Bay (cutscene 2, notice 0), 255; ACTOR2 in the Waking Sands rewards it (scene 3, a yield), then the cutscenes (4, given the chosen item; UI8AH and UI8AL set to 1 as it starts), after which the player is moved within the Waking Sands to POPRANGE0 and the quest completes (retainers, UnlockLink 3, from the Quest sheet). Tataru's line in 255 (5). |
| 510 | A Wild Rose by Any Other Name | `005/ManFst207_00510` | client script (2026-10-25), Sapphire, Chronofoil capture (2026-10-25) | Minfilia offers it (scene 0, a yield with a menu), then cutscene 50 accepts it; retail then moved the player within the Waking Sands to a recorded position (no pop range in the quest: `change_territory(212, …)`), sequence 1; Thancred (ACTOR2, scene 2, flags 0x6001) sends notice 0 `[1, 1]`, 255 (ACTOR1's scene 1 does the same, unseen on retail); Isembard rewards it (scene 3, a yield). |
| 699 | Color Your World | `006/SubSea921_00699` | client script (2026-10-25), Sapphire, Chronofoil capture (2026-10-25) | Swyrgeim offers it (scene 0, a yield; straight to 255); with RITEM0 in the bags he asks for it (NpcTrade scene 1, a yield; scene 3 without it) and rewards it (2, a yield); the handed-in item leaves its bag only then (`take_handed_in_items`). Unlocks dyeing (UnlockLink 14). |
| 3017 | If I Had a Glamour | `030/SubCts896_03017` | client script (2026-10-25), Chronofoil capture (2026-10-25) | Swyrgeim offers it (scene 0, a yield, then 1), sequence 1; Folclind gives ITEM0 (scene 2: notice 0 `[24401]`, UI8BH 1), 255; Swyrgeim takes it (event-item NpcTrade 4) and rewards it (5, a yield). Lines: Swyrgeim in 1 (3), Folclind in 255 (6). Unlocks glamours (UnlockLink 15). |
| 3018 | Absolutely Glamourous | `030/SubCts897_03018` | client script (2026-10-25); Chronofoil capture (2026-10-25) of the offer only, declined | Swyrgeim offers it (scene 0, then 1), sequence 1; Folclind gives ITEM0 (scene 2, notice 0), 2, ITEM0 in UI8BL; Swyrgeim takes it (event-item NpcTrade 4) and gives ITEM1 (5: notice 1, UI8BH 1), 255; Goberin (ACTOR2) takes ITEM1 (NpcTrade 7) and rewards it (8, a yield). Lines: Swyrgeim in 1 and 255 (3, 9), Folclind in 2 (6). Everything after the offer is unverified. |
| 22 | Way of the Conjurer | `000/ClsCnj000_00022` | client script, Chronofoil capture b23b91ef (2026-10-26) | Madelle offers it (scene 0, a yield), sequence 1; E-Sumi-Yan's cutscene (scene 1, 0xF8482EFB) ends on a yes/no: yes is sequence 2 and notice 0. Three each of ENEMY0–2 (UI8AL, UI8BH, UI8BL; notice 1–3 with [count, 3]; UI8AH counts the kinds done), then 255 with the variables cleared. E-Sumi-Yan's reward (scene 2) gives the Hunting Log (SystemReward 21). |
| 48 | Trial by Earth | `000/ClsCnj001_00048` | client script, Chronofoil capture b23b91ef (2026-10-26) | E-Sumi-Yan offers it (scene 0), sequence 1; ACTOR1 in the North Shroud (scene 1), notice 0, sequence 2; searching EOBJECT0 (scene 2, the bar, scene 100) brings out ENEMY0, a tainted earth sprite (drop-in layer `QB_ClsCnj001_001`, timeline `TaintedEarthSprite_294`); its kill is notice 1 and sequence 3; ACTOR1's cutscene (scene 3, 0xF8402FFF), notice 2, 255; E-Sumi-Yan's reward (scene 4). |

Upstream's own scripts cover the three city openings and the Close to Home offers. Gridania's story from Close to
Home to The Gridanian Envoy is complete; Call of the Sea starts the next part (Limsa Lominsa). Limsa Lominsa's runs from
Coming to Limsa Lominsa through Close to Home and Making a Name (111, the opening's way out), then On to Summerford,
Dressed to Call, Lurkers in the Grotto (with its quest battle), Washed Up, and on through Just Deserts, Victory in
Peril and Feint and Strike (with theirs) to The Lominsan Envoy (546). Ul'dah's runs from
Coming to Ul'dah to The Ul'dahn Envoy and Call of the Sea (the whole Ul'dah opening), with Takin' What They're Givin' and the Western Thanalan side quests on the way (all three classes' Close to Home versions; 569 and 570 from their client scripts
only).

**Attuning inside a quest** (Close to Home, Ul'dah; Bob's Chronofoil capture of 2026-10-18, game 2026.10.06): the
client talks to the aetheryte with the quest's handler. Retail answers with the quest's EventStart, then the
aetheryte's own (handler 0x50009, **type 7, Nest**, same target), swaps OccupiedInQuestEvent (7) for OccupiedInEvent
(6) and starts EventAction 19 on the aetheryte (ActorControlTarget 3: the attunement animation and its 5 s bar). When
the bar ends: LearnTeleport (ActorControlSelf 509: 9, 1), the aetheryte event's notice 0 with one argument 0
(EventNotice2), its EventFinish (type 7, result 1), the quest's scene 1 (flags 0x2000; the client's script only
waits in it), condition 7 back, and the EventAction's end. Scene 2 is the attunement cutscene; Return's unlock
(ActorControlSelf 41, UnlockLink 1) comes after it, then the quest's notice 0. `ManWil002_00568` does all of this
(`start_nested_event`, `event_action`, `onEventActionCast`); until 2026-10-18 it skipped the bar and the nested event.

The client talks to an aetheryte (or shard) by its **layout ID**, with the quest's handler: aetherytes are part of the
map, not objects the server spawns, so until 2026-10-18 nothing gave them a `BASE_ID` and Ul'dah's Close to Home, which
checks `BASE_ID == AETHERYTE0`, ended the event at once (the aetheryte "couldn't be clicked"; Gridania's version
dodges it with `BASE_ID ~= ACTOR0`). The zone now caches the map's Aetheryte entries (layout ID → Aetheryte row) next
to the event objects', so `BASE_ID` is the Aetheryte row, the same number as the quest's `AETHERYTE*`.

## Scenario Guide

The Scenario Guide (the MSQ marker) is sent at login and whenever quests change (`send_scenario_guide`,
`zone_connection/quest.rs`): the accepted main scenario quest, or else the next one that follows from what's completed
(ScenarioTree sheet, with Sapphire's class and Grand Company checks), the last one completed, and where its issuer
stands. When a step has variants (Close to Home per starting class), once one is done or accepted the others no longer
count; between untouched variants the starting class's is preferred. ScenarioTree is read in whatever language it has.

## Quest rewards and EXP

`finish_quest` gives the Quest sheet's gil, EXP, fixed items and crystals (crystals go to the crystal bag) and the
chosen optional item. Quest EXP is `ExpFactor × ScaledQuestXP × QuestExpModifier / 100` at the player's level (matches
retail: Eggs over Queasy 2240 at level 7, Surveying the Damage 2720 at level 8). Rested EXP does not apply to quest
rewards. Kill EXP is in [Combat](combat.md).

**Unlock links.** A quest's `REWARD…` parameters are unlock links it grants when finished (`get_quest_unlock_links`,
`grant_quest_unlocks`): Spirithold Broken's `REWARD0 = 9` is the Recommendations window and `REWARD_LEVE = 5`
levequests, Close to Home's `REWARD_DESION = 1` is Return (Sapphire's ManWil002 grants Return for its `Reward0 = 1`
too). `REWARD_GIL…`/`REWARD_EXP…` are amounts and are skipped. The server sets them at `finish_quest` (with actor
control ToggleUnlock) and, at login, for every quest already finished (`check_quest_unlocks`). Scripts can still
unlock something mid-quest with `player:unlock(id)`, as Close to Home does for Return when attuning.

Not every unlock is a `REWARD…` parameter (2026-10-26). Retail's ToggleUnlock right before FinishQuest (Bob's
Chronofoil capture afe32479): The Scions of the Seventh Dawn **3** (retainers), Color Your World **14** (dyeing), If I
Had a Glamour **15** (glamours); none has a `REWARD…` parameter, so none unlocked anything here and the bell answered
"no approval" after The Scions of the Seventh Dawn (Bob, 2026-10-26). A quest now also grants the unlock links of
what it rewards: its general actions (GeneralActionReward, and the ones its script names, `LOC_GENERAL_ACTION*`:
Color Your World's 15, If I Had a Glamour's 22 and 25), its ActionReward and EmoteReward (each row's UnlockLink), its
SystemReward (taken as unlock links), and what retail was recorded granting (`RECORDED_QUEST_UNLOCKS`: the three
above, so they hold whichever column is theirs). Each link granted is logged ("Quest … unlocks unlock link …"), which
tells from a bundle which sources are right. Characters who finished them before get them at their next login.
If I Had a Glamour's LogMessage 789 (ActorControlSelf 517, with the item 13258) isn't sent.
