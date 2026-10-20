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
| `start_nested_event(handler)` / `finish_nested_event(handler)` | Sends only the packets of a nested event (EventStart / EventFinish, type Nest, on the current event's target), as retail does with the aetheryte's event while attuning in Close to Home; no handler runs for it. If an event action is interrupted, the nested event is finished too (and OccupiedInEvent removed). |
| `event_action(id)` | Starts an event action (the progress bar: Searching, Aiding, Interacting…); the script gets `onEventActionCast(player)` when it completes. |
| `spawn_quest_bnpc(layout_id)` | Brings out a battle NPC from the zone's drop-ins for this quest. |
| `join_quest_battle(id, zone)` | Enters a quest battle (see [Quest battles](quest-battles.md)). |

Callbacks a quest script can define:

| Callback | When |
|---|---|
| `onTalk(target, player)` | The player talks to/examines `BASE_ID`. |
| `onYield(scene, id, results, player)` / `onReturn(scene, results, player)` | A scene finished. The client can report a scene through **both**, so scripts guard with `current_scene` (below). `results` is the scene's return values. Scenes with more than 8 results arrive as EventReturnHandler16 (opcode 200 on 7.57). Which one a scene comes back as is the client script's choice: in the Ul'dah quests the offers, the rewards and Close to Home's trade were yields (EventYieldHandler4), Nothing to See Here's trades and the cutscenes returns. |
| `onEventActionCast(player)` | An `event_action` progress bar completed. Not called if it was interrupted. |
| `onEnterTrigger(player, arg)` | The player walked into one of the quest's event ranges (e.g. `EVENTRANGE0`). |
| `onEnterTerritory(player)` | Entering a zone where the quest has something to do (e.g. post-battle cutscenes). A script the client starts this for must define it and finish the event; otherwise the error is logged and the client keeps waiting (a fix for that, `02e22b5`, went with the rollback). |
| `onBNpcKill(base_id, layout_id, player)` | A battle NPC the player (or their party in the same instance) had enmity with died. `base_id` is the **BNpcBase** ID, which is what `ENEMY*` parameters are (an anole is base 142, name 120). |

### Notices (the progress pop-up)

`player:event_notice(EVENT_ID, n, {args})`. From retail recordings, **notice n means todo n was completed or
advanced**, and its arguments are either:

* `[count, needed]` (e.g. `[2, 4]`: "2/4"), optionally followed by an icon (`[count, needed, icon]`), or
* a single key item icon (`[25919]`), or nothing.

0–2 arguments are sent as **EventNotice2** (opcode 386 on 7.57), 3–4 as **EventNotice4** (210); more than 4 is refused
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

### Quest-spawned enemies

`player:spawn_quest_bnpc(layout_id)` (`ToServer::SpawnQuestBattleNpc`, handled in `server/mod.rs`):

1. looks up the layout ID in the zone's battle NPCs (from `resources/dropins/<zone>.json` layers such as
   `QB_SubFst060_001`);
2. does nothing if that NPC is already out (one of each at a time);
3. inserts it, records it in `instance.quest_spawned_npcs` so it **doesn't respawn** when defeated;
4. gives it enmity on the player, so it attacks at once.

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
| 108 | Close to Home (Limsa Lominsa, Marauder) | `001/ManSea002_00108` | client script (2026-10-19); flags, attunement and notices from 568's retail capture; fixture adapted from 124's, no retail recording | Offer (0), accept cutscene (1) moves the player from the opening's Upper Decks (181) into the real one (128, from Sapphire's ENpc positions: the client script has no TERRITORYTYPE0) with ITEM0 (UI8CH); three objectives in any order: attune (568's nested aetheryte event and bar, scenes 2 → 3, then Return; UI8BL, BitFlag8 1, notice 0), hand ITEM0 to ACTOR1 (trade 4 → 5; UI8BH, BitFlag8 2, notice 1), visit the guild (ACTOR2, scene 6; UI8AL, BitFlag8 3, notice 2); UI8AH counts; reward from ACTOR3 (scene 7). Upstream's script waited for a scene 50 that doesn't exist, so it could never be accepted. |
| 109 | Close to Home (Limsa Lominsa, Arcanist) | `001/ManSea003_00109` | client script (2026-10-19), as 108 | As 108 with its own guild NPC (ACTOR2) and ITEM0. |

Upstream's own scripts cover the three city openings and the Close to Home offers. Gridania's story from Close to
Home to The Gridanian Envoy is complete; Call of the Sea starts the next part (Limsa Lominsa). Ul'dah's runs from
Coming to Ul'dah to Way Down in the Hole, then Takin' What They're Givin' and the Western Thanalan side quests after it (all three classes' Close to Home versions; 569 and 570 from their client scripts
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
