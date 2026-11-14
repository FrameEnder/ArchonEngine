# questkit and CI

`tools/questkit` is a Python toolkit for writing, testing and debugging the fork's quests, and
`.github/workflows/fork.yml` runs its checks (and the build) on GitHub. Everything here works offline from files: a
Kawari Recorder export, a diagnostics bundle, quest dumps.

```shell
python3 tools/questkit --help
pip install lupa        # Lua 5.4 for Python; replay, scaffold and lua-syntax need it, triage and compare don't
```

| Command | What it does |
|---|---|
| `triage <bundle>` | The first read of a diagnostics bundle, as Markdown |
| `replay [quests]` | Runs each scripted quest against what retail did (its fixture) |
| `fixture <quests\|all> --recording … --dumps …` | Makes replay fixtures from a Recorder export |
| `scaffold <quest> --dumps … --recording …` | Drafts a quest script (and its fixture) from the dump and the recording |
| `transcribe <quest> --dumps … [--fixture …]` | Writes a quest script that does what retail did, from the quest's Chronofoil fixtures, and replays it |
| `compare --capture … --recording … [quests]` | What Kawari sent for a quest against what retail did |
| `lua-syntax [folder]` | Checks every Lua script compiles (Lua 5.4, like the server) |
| `duty-sim [duties] [-v] [--make SCRIPT.lua --run …] [--update-known]` | Runs dungeons' and trials' director scripts against what the same duties did on retail, and rehearses a wipe in every boss fight ([Testing](testing.md#duty-sim)) |
| `rotation <classjob> <level> [targets] [--against uses.json]`, `rotation --check` | Runs a party bot's rotation against a dummy with its real action scripts; `--check` (in the tests) is every rotation at every level band |
| `sources [--audit] [--duty CFC [--territory T]] [--action N] [--server-data] [--build …]` | What Boss Mod and cactbot know: an audit of the fork's duties against them, a draft of a duty the fork hasn't, the server's fallback cone angles and ring holes |
| `battle <exports, battles folder or files…> [--list] [--where …] [--run N,M\|all] [--summary-only] [--npc …] [--out file]` | What recorded duties and quest battles show: cast, waves, targeting rules, heals, who never acted; totals across runs of the same battle |

Code: `questkit/recording.py` (Recorder exports), `capture.py` (Kawari's packet captures), `bundle.py` (diagnostics
bundles), `harness.py` (running a quest script like the server), `fixtures.py`, `replay.py`, `scaffold.py`,
`compare.py`, `triage.py`, `cli.py`. Tests: `python3 -m unittest discover tools/questkit/tests` (made-up data only).

## triage

```shell
python3 tools/questkit triage ../kawari-diagnostics            # the newest bundle (latest.txt)
python3 tools/questkit triage ../kawari-diagnostics/bundles/<name> --write   # writes TRIAGE.md into it
```

The report has:

* **The note, build, game version**, and the Quest Lab test if the bundle ends one (`meta.json` → `quest_test`).
* **Quests, server and client**: each character's saved quests (`characters.json`) next to the newest Kawari Probe
  snapshot's (`raw_hex` decoded), marked *same* / **differ** / missing on one side.
* **Server log**: counts per level; Lua errors, panics, unknown packets and dropped connections shown in full (first
  of each kind); every WARN/ERROR grouped with its numbers taken out, with counts and first/last time.
* **Per player in the captures**:
  * unknown packets by opcode and size, naming the fork opcode with that size (`856, 72 bytes, the size of
    EventReturnHandler16` is the Pauline bug);
  * **events still open** at the end (EventStart without EventFinish: the client is stuck in them);
  * **slow or missing replies**: client packets that need an answer (StartTalkEvent, EventReturn/Yield, ActionRequest)
    and got nothing, or only after more than a second;
  * **late casts**: the player's casts that landed more than 0.3 s after the bar, and warps more than 1.5 s after a
    cast landed (the Teleport bug: 8.2 s);
  * a **timeline** of quest and event packets (scenes, returns with their results, quest states, notices, unknown
    packets), filtered to the tested quest when there is one;
  * the **marks** (`!mark` in game, the captures' `note` lines starting `MARK`), each with its time.
* **What the client said was stopping the player**: each Kawari Probe snapshot with a `stuck` part (Probe 0.7.0,
  `/kprobe stuck`): conditions, character mode, animation lock, cast, queued action, action sequence, pending item
  moves, statuses, the last 20 refused actions and log messages, and any hook that failed.

Checked against the old bundles: it points straight at the Pauline (open event + unknown 856), teleport (8.2 s warp)
and pre-fix cast timing (+0.3–0.5 s) problems.

## replay

```shell
python3 tools/questkit replay          # every fixture
python3 tools/questkit replay 387 -v   # one quest, every step
```

**Fixtures** (`tools/questkit/fixtures/<quest>.json`) are what a quest did on retail, step by step, **numbers only**
(no game text, so they're committed): the Quest sheet's parameters, and each step the player took with the quest's
state after it:

| Step | Meaning |
|---|---|
| `{"talk": base}` | Talk to / examine an NPC or object (the event's `BASE_ID`); `"layout"` gives its layout ID, which the script sees as `target.object_id` (upstream's Ul'dah scripts tell NPCs apart by it). `"territory"` says where it happened, written when retail moved the player within that territory (PrepareZoning to the same one), so replay counts a script's `move_to_pop_range` as that zone change, and the server's own warp when the step moves that territory's layer filter on (`resources/data/layer_filters.json`: an entry of the quest's starts or stops applying, and the script didn't move the player itself; Wrath of the Titan's accepting cutscene, 2026-11-03). |
| `{"kill": [base, layout]}` | A battle NPC with this BNpcBase and layout ID died (`onBNpcKill`) |
| `{"event_item": [item, base], "layout": …}` | A key item used on this NPC or object (`onEventItem`, after the server's EventItem event starts) |
| `{"range": arg}` | Walked into an event range (`onEnterTrigger`) |
| `{"action": [action, base], "layout": …}` | An action (its Action row) used on this NPC or object, which the quest listens for (`onActionResult`, after the server's ActionResult event starts): Trial by Water's Cure on the wounded Wood Wailer |
| `{"emote": [emote, base], "layout": …}` | An emote used on this NPC (`onEmote`, after the server's Emote event starts); `fixture --chronofoil` writes it from retail's EventStart of type 2 |
| `{"enter_territory": id}` | Zoned in (`onEnterTerritory`) |
| `{"battle_won": sequence}` | A quest battle was won (the director sets the sequence, with the battle script's `QUEST_VARS_AFTER_WIN`; its `QUEST_LOG` is the step's `logs`) |
| `{"duty_cleared": content}` | Leaving a duty the player cleared, its InstanceContent ID (`onDutyCleared`); `fixture --chronofoil` leaves it as a todo ("changed … outside an event") to fill in |
| `"expect": {"sequence", "vars", "notices"}`, `{"completed": true}` or `{"accepted": false}` | The state after the step (`accepted: false`: the offer was turned down); `notices` is left out where the recording had no packets; `items` ([[item, quantity], …], the `obtain_item`s in order) and `logs` ([[LogMessage, params], …], the quest's `event_log_message`s), each checked when given; `fixture --chronofoil` writes them from retail's EventLogMessages (an item: LogMessage 750 [item] with handler 0 while the quest's event is open; a line from outside an event goes with the next kill or win), left out where retail had none and items left out of the step that completes the quest (the reward is the server's) |
| `"scenes": [{"scene", "flags", "params", "results", "yield", "error"}, …]` | Every scene retail played in the step, in order, with its flags (hex), parameters (when it had any) and the client's answer. The script must play the same scenes with the same flags and parameters, and each gets retail's answer (a fixture from Chronofoil) |
| `"effects": [{"zone": id}, {"aetheryte": id}, {"unlock": id}, {"content": id}, {"mount": id}, {"dismount": 1}]` | What retail did outside the quest while the event was open: zone changes (`change_territory`, a pop range, joining a quest battle), attunements, unlocks (ActorControlSelf 41, `unlock`), duties unlocked (ActorControlSelf 131 with the InstanceContent ID, `unlock_content`), the player put on a mount (the Mount packet to and from them, `player:mount`) or taken off (PlayDismountAnimation from them: the server's answer to a yield 232, which the harness gives as the server does). A scene entry's `"yield_id"` is the yield's ID when it isn't 0 (`fixture --chronofoil` writes it). With `scenes`, a step without `effects` must do none |
| `"spawns": [layout, …]` | The step must bring these out (`spawn_quest_bnpc`); `fixture --chronofoil` writes the quest's `ENEMY*` layouts SpawnNpc brought out while the event was open (or just after it) |
| `"results": {"scene": [..]}` | What the client answers a scene with, if not `[1, 0]`; `{"results": [..], "error": n}` also sets the scene's error code (`player.scene_error_code`), for yes/nos that answer through it; `"yield": true` sends it back through `onYield` as retail's client did (EventYieldHandler), even when the script has `onReturn`; a yield the server answered with EventResume has `"resume"` (its parameters, which the script's `resume_event` must match; `name_companion` counts as a resume with 1, as the server answers a usable name) and `"after"` (the client's next answer, which replay gives the script after the resume); `"yield_id"` is the yield's ID (0 if not given: a text answer's is 231) |
| `"what"` | Which parameter the target is (for reading) |
| `"from": {"sequence", "vars"}` | A run recorded from the middle of the quest (the capture began after it was accepted): the quest is active in that state before the first step (701's run in capture 9df4ff01) |
| `"interrupted": true` (a step) | Retail broke the step's progress bar off (moving, a hit, paralysis: CancelCast with kind 4 in the capture): the script's `event_action` ends the event there, without `onEventActionCast` (Seeing Eye to Winged Eye's look-out in captures 412ba91d and b166797f, by hand; `fixture --chronofoil` writes it since 2026-11-09 when the step's last scene was followed by a bar and nothing came of it) |
| `"bar": action` (a scene) | A progress bar followed this scene (retail's ActorControlTarget 3, EventAction, with 1 and the EventAction row); `fixture --chronofoil` writes it, `transcribe` reads it |
| `"pop_ranges": [id]`, `"arrivals": [[x, y, z, rotation]]` (a step) | Where the step's zone change put the player: ZoneInit's pop range, and the spot (ZoneInit's position and the client's first rotation, or retail's ActorSetPos for a move within the territory, which has no ZoneInit); written by `fixture --chronofoil`, read by `transcribe` |

**The harness** (`harness.py`) runs the real script under Lua 5.4 the way the world server does:

* a new Lua state per event: the scene flags (read from `event_scene.rs`, combinations such as `SET_BASE` included),
  `Global.lua`, the parameters, the script, then `EVENT_ID` and `BASE_ID`;
* the script sees a copy of the quests: sequence/variable changes show up in it straight away, accepting doesn't;
* queued tasks run in order after each callback; accepting starts a quest at sequence 255 (as `accept_quest` does);
* the client's side is automatic: with `scenes`, each scene gets retail's own answer; otherwise scenes return
  `[1, 0]` (accepted, handed over, first reward). Progress bars complete, and `onReturn` (or `onYield`) gets the
  scene back;
* every scene the script plays (number, flags, parameters) and what it does outside the quest are kept. Scenes after
  `start_event` are the nested event's (the cities' openings) and aren't compared.

A step fails if the script errors, leaves the event **unfinished** (no `finish_event`, the client would hang), plays
something after finishing, sends a notice with more than 4 arguments, ends in a different sequence, variables or
notices than retail, or (fixtures from Chronofoil) plays other scenes or other flags than retail, or does other
things outside the quest. What a scene *shows* is the client's, so a scene number right for the wrong reason still
needs the listing.

All 28 scripted quests have fixtures from Bob's Chronofoil captures (2026-10-14 to 17) and match. Checking scenes and
effects found real differences in 15 of them, all fixed (2026-10-17, see the [Changelog](changelog.md)): cutscenes
played with `FADE_OUT | CONDITION_CUTSCENE | HIDE_UI` where retail uses `SET_BASE | HIDE_HOTBAR` (0xF8402EFB, which
hides the other players and NPCs), searches and other progress bars missing the scene retail plays before the bar
(the object's scene, the bar, then its "found it" scene, all `HIDE_HOTBAR | NO_DEFAULT_CAMERA`), The Gridanian Envoy
moving you about without the loading screen retail shows and not honouring a "No" to the attendant, and a few scenes
with their own flags. The fixtures for quests with a quest battle use a `battle_won` step for it; the battle itself
is simulated by `battle-sim` (below).

### Making fixtures

From **Chronofoil** (the better source: every talk, scene and effect):

```shell
python3 tools/questkit fixture 568 595 --chronofoil chronofoil/ --capture 3,4,5 --recording rec.zip --dumps quest-dumps
```

`chronofoil_quest.py` reads each quest's events out of the captures (`questkit chronofoil <folder> --quest 568`
prints them): StartTalkEvent, EventStart, the scenes and the client's answers (EventReturnHandler or
EventYieldHandler), notices, AcceptQuest/UpdateQuest/FinishQuest, EventFinish, and while the quest's event is open
(or within a second after it), PrepareZoning and the ActorControlSelf LearnTeleport (509) and ToggleUnlock (41). Each
talk or zone-entry event becomes a step; a quest change outside any event is a quest battle won (in TERRITORYTYPE0) or
a kill. A kill is named from the capture itself since 2026-11-05: the battle NPC (its BNpcBase and layout ID, from its
SpawnNpc) whose ActorControl Defeated came at most 0.5 s before the change (`KILL_WINDOW`; retail sends them together).
Before, kills came only from an existing fixture, matched by state, so a second run's fixture made over the first one's
(`--force`) got the first run's sprites: That Weight's three runs each killed a different five. Talks that change nothing are steps too (agreeing to a quest battle); a progress bar broken off (a scene the
client never answered) isn't, since nothing happened, nor a talk the server never started (no EventStart: the player
was still in combat, 1358's first talk to ACTOR2). Since 2026-11-09 it also reads each scene's progress bar
(ActorControlTarget 3 with 1 and the EventAction row: the scene's `bar`; a step whose bar was the last thing, with
nothing coming of it, is `interrupted`), the quest's `ENEMY*` layouts SpawnNpc brings out during the event (`spawns`),
and where a zone change put the player (`pop_ranges` from ZoneInit; `arrivals`, the spot and facing, from ZoneInit and
the client's first position, or from ActorSetPos for a move within the territory, which has no ZoneInit: before, the
next ZoneInit's pop range was taken, often a later teleport's, whose pop range field held small numbers: apparently the aetheryte's row, 9 in Ul'dah, 24 in Mor Dhona). The talked-to actor is the layout ID of an event NPC or object;
its base ID comes from the server's SpawnObject packets, the Recorder export's NPC list (`--recording`, optional),
the dumps (`--dumps`: a todo's locations are the layout IDs of who it's done with, and the listeners whose
`despawn_sequence` is the todo's index are their base IDs, taken when there are as many of each; since 2026-10-30,
`fixtures.dump_layout_bases`. It can miss: Brotherly Love's todo 2 is at 4309068, but retail's Claxio was talked to at
3853930) or the existing fixture, whose base IDs win over the dumps' (since 2026-11-07: the dumps' pairing gave
Following the Evidence's EOBJECT3 and 4 the wrong way round in every run, fixed by hand in `911.json` once), which also gives what the capture can't (`spawns`, and kills it can't name) by matching the state each step left
the quest in. A key item used on an object (EventStart type 8: the actor is the object, the argument the item) is an `event_item`
step since 2026-11-12, its base from the existing fixture's step with the same layout (before, a todo: Drowning Out the
Voices' five uses and Fool Me Twice's in Bob's capture da424662). A step the capture can't name takes the existing
fixture's action of its own kind (a talk, an emote or an item used): Escape from Castrum Centri's emote on ACTOR11 took a
talk's base before. Unlocks in the step that completes a quest are left out: those are the Quest sheet's, sent by the
server's `finish_quest`. Without a dump, the existing fixture's parameters are used.

From a **Kawari Recorder** export:

```shell
python3 tools/questkit fixture all --recording rec.zip --recording older.zip --dumps quest-dumps --source "…"
```

Recorder 2.0+ records who was last targeted, so each step comes out filled in (`talk`/`kill`), with the notices
matched to the change that follows them (within 0.5 s). Older exports don't, and their steps come out as `{"todo": …}`
to fill in by hand from the script's header. A change inside the quest battle's territory (`TERRITORYTYPE0`) is a
`battle_won` step whoever the player was hitting, and one on arriving in another territory with nothing targeted an
`enter_territory` step (Underneath the Sultantree's came out as a kill of the blanga and a second battle win before).
Talk steps carry the target's layout ID. Steps that change nothing (searching the egg in Migrant Marauders,
walking into an ambush) aren't recorded at all; add them with `spawns` when they matter. Recorder 2.7's battle files (`battles/*.json.gz`) log
the EventPlays (scene, flags) they catch in a fight, and the export's `packets.json` every EventFinish (its type says
an emote, type 2 with the emote, from a talk): what there is of a quest's scenes without a Chronofoil capture. A job
quest's reward comes out as two steps, a 0x11 UpdateQuest and the completion ten seconds later; they're one talk
(`quests.md`, the TALKS table).

## scaffold

```shell
python3 tools/questkit scaffold 388 --dumps quest-dumps --recording kawari-recorder-<date>.zip
```

Writes `resources/scripts/events/quest/<id/100>/<script>.lua` (refuses to overwrite without `--force`; `--out`
elsewhere) and the fixture, then replays the draft and prints the result. How it works:

1. **Scenes** from the client listing: `QuestOffer`/`QuestAccepted` (offer, accept, or both in one scene), `NpcTrade`
   (trade), `QuestReward`/`QuestCompleted` (reward), `YesNoQuestBattle`, `ScenarioMessage` (the "they appear"
   message), cutscenes, plain talk, and empty scenes (the ones after a search). Who talks in each comes from its text
   keys (`TEXT_<SCRIPT>_<SPEAKER>_…`) matched to the parameters' NPC names.
2. **Steps** from the fixture: the first talk is the offer; a talk that completes the quest is the reward (trade then
   reward, if the NPC has a trade scene); an EOBJECT with an `EVENT_ACTION_*` parameter is a search; kills,
   ranges, zoning and quest battles as recorded.
3. **Scene per step**: the Quest sheet's `SEQ_<n>_<ACTOR>` parameter when there is one, else the scene that NPC
   speaks in, else (for searches) the next empty scene, else a guess marked `TODO`.
4. **Effects**: each step sets the variables, notices and sequence retail had after it, literally.
5. **Any-order groups**: two or more searches/kills in one sequence become one handler when a nibble counts them:
   a table of the objects with their scene and the BitFlag8 retail set (the last object's flag is a guess, as retail
   clears it), the counter, other nibbles that go up with it (key items), notices as a template (`{ count, needed,
   21003 }`, or only on the last step), and the final state when the count is reached.

Checked on the 16 known quests: every draft replays against its fixture. Compared with the hand-written scripts, the
main thing to fix by hand is guessed scene numbers (marked TODO), and steps that change nothing on retail (spawns,
remembering an examined object).

## transcribe

```shell
python3 tools/questkit transcribe 981 --dumps quest-dumps/20261109          # every fixture of 981 (981.json, 981-*.json)
python3 tools/questkit transcribe 981 --dumps quest-dumps --fixture tools/questkit/fixtures/981.json --out /tmp/x.lua
```

For quests with Chronofoil fixtures, which hold everything retail did (`transcribe.py`): every step becomes a branch
of the script, chosen by what started it (a talk, a key item, an emote, an event range, zoning in, a kill, a duty
cleared) and the sequence; it plays the step's scenes with retail's flags and parameters, in order, and changes what
retail changed when the last one ends. Header: one line per step, by sequence. In detail:

* **Answers**: a scene answered with a yield of 1 that more follows (an offer, a trade, a quest battle's yes/no, a
  reward) goes on only on 1; a closed reward window sets `REWARD_CLOSED` (0x11). The reward's item goes to the scenes
  after it that showed it as their parameter (`{ chosen }`).
* **Bars**: a scene followed by a progress bar (the fixture's `bar`) starts that event action, named by the quest's
  `EVENT_ACTION_*` parameter, and plays the next scene in `onEventActionCast`.
* **Runs merged**: the same step in several runs (fixtures) is one branch; a run broken off (a bar interrupted, a
  scene with an error code, an offer turned down) is the start of the full one; a talk that changed nothing at first
  (a quest battle lost) and later moved the quest on is the later one; the union of what the server did. A scene
  retail saw broken off (an error code, nothing changed: Wedge's lines in 1005) checks `player.scene_error_code`.
* **Moves**: the step before a `battle_won` joins `QUESTBATTLE0` in the territory retail went to; another zone change
  goes to the pop range retail landed in when it's one of the quest's `POPRANGE*` (`move_to_pop_range` within the
  territory, `change_territory_pop_range` elsewhere), else to retail's spot and facing (the fixture's `arrivals`) with
  `change_territory`. Completing comes before the move, as on retail.
* **Any order**: talks, emotes or key item uses of one sequence that count up one nibble, each setting one BitFlag8,
  the last moving the quest on, become one group (the last one's flag is the next free one, as retail clears it):
  any order works, notices as a `{ count, needed }` template or each target's own. Kills of one sequence that count
  one nibble likewise (`KILLS_<sequence>`), by layout for the quest's enemies and by BNpcBase for open-world ones.
* **Left to do by hand**: what retail didn't show (lines in sequences nobody talked in, the client's gating in
  IsAcceptEvent) and steps whose counting doesn't fit (two counters at once, Dressed for Conquest's two hauls; a key
  item that brings enemies out and counts when they fall, Drowning Out the Voices).

Checked on the 50 quests of Connor's captures: 48 replay with no hand edits (1001 and 3871 were finished by hand).

## compare

```shell
python3 tools/questkit compare --capture ../kawari-diagnostics --recording rec.zip            # newest bundle
python3 tools/questkit compare --capture capture.jsonl.gz --recording rec.zip 387 --actor 3209401247
```

For each quest in both, it lines up the states (sequence + variables after each change; several UpdateQuest packets
within 0.3 s count as one, as the client only sees where they end) and the notices, and prints a diff: `=` same, `-`
retail only, `+` Kawari only, `.` retail went further than the capture. States that differ only in which BitFlag8s
are set, with as many set, are flagged as probably another order (replay checks the same order exactly). A run starts
at AcceptQuest (or wherever the capture first sees the quest) and the last run is compared. Exit code 1 when anything
differs.

On Bob's teleport bundle against the retail recording: 381, 384 and 387 match, 377 matches apart from the trumpets
being searched in another order.

## battle

Reads the Recorder's battles and writes a Markdown report per battle (`questkit/battle.py`). It takes any mix of
Recorder exports (zip), the Recorder's `battles` folder itself (or an unpacked export), single battle files
(`.json.gz`) and 2.4's `battles.json`; labels come from `index.json`. `--list` shows what it found, `--where` filters
by zone, content or territory, `--run 2,5,7` or `--run all` picks several.

**Across runs**: given several runs of the same battle (same territory and content), it first adds up each NPC's
evidence over all of them (matched by layout ID): in how many runs it appeared and never acted, when it died, every
target choice's explanations summed, each action per run and its interval, heals and how hurt their targets were. A
rule that held in every run stands out from one that held once. `--summary-only` gives just that.

For each battle:

* **Who's in it**: each fighter's battalion (same number, same side), level, max HP, when it appeared and died, its
  layout layer and instance name, and its layout settings (`NormalAI`, `MoveAi`, `ServerPathId`, link group...).
* **Timeline**: NPCs appearing within 2 s of each other as one wave, with who had just died and the hurt NPCs' HP
  then (the most hurt first: a wave brought by an HP threshold shows the boss low), deaths, sequence and variable
  changes, battle talk, cutscenes, map effects and marks.
* **Each NPC**: its targets over time and, for every change of target, which explanations fit, counted over all of
  them: the old target died or left; the new one had just appeared; it's the newest / oldest / nearest / most hurt
  enemy; the player, or an NPC over the player; it had just hit this NPC, or one of its allies; it was after one of
  its allies, or after this NPC itself. A rule that holds 6 times out of 6 is a rule; 3 out of 6 is a coincidence. Then
  each action: how many, how often (median and range), on whom, damage or heal amounts, and for heals how hurt the
  target was and whether it was the most hurt of its side.
  Since 2026-11-12 also where it stands to fight, for the timelines and the director (Bob's bundle 20261112-195353:
  Notorious Biggs' placement wasn't retail's): **"Appeared N s into the fight and stood M s before it walked"** for one
  that came once the fight was on (retail's waves stand 4.8-5.0 s: `Battle.ARRIVAL_STAND_SECONDS`); **"Stopped N
  yalms from whom it was after"** (median and range of where its walks ended while they stood still, centre to centre:
  its timeline's `attack_range`); and under an action used on someone other than whom it was fighting, half the time
  or more, how far away they were and how often it walked to them first (an action point's `target: other` and
  `reach`). Kawari's own capture reads the same way once it's a battle run, so the two can be set side by side.
* **Never acted**: battle NPCs that never used an action (scenery, or a script Kawari is missing).

**Class and rotation** (Recorder 2.7 files): for each NPC, which class it fights as (the class its actions belong
to, from `ActionInfo`), its auto-attack, how often its weaponskills and spells come (its pace), whether each combo step
follows the step before it, whether each ability is used on cooldown or less often, which GCD action follows a buff,
which actions it never re-applies to the same target within N seconds (damage over time, debuffs), and what its class
kit had at its level that it never used. Then a ready **`rotation` block** for its timeline (see
[Combat](combat.md#class-rotations-serverrotationrs)), with each entry's evidence in its comment.

**The director's outline (draft)**: the fight cut where the director's sequence changes, and for each ally in each
phase what it was there to do, read from what it did: its **role** (its class's, from the class kit's ClassJob `Role`,
and the one it played: taking half or more of its side's hits is the tank's part (a DPS taking the toughest enemy on
plays it too); healing at least half as often as it attacks is a support's; a DPS that heals now and then is still a
DPS, "with some support"; when class and part differ it says so), its focus (on the boss or the adds, by how much of its time
while there were both; the boss being the standing enemy with the most max HP), its order within that group (which
of oldest, newest, nearest, most hurt, healthiest or weakest (the least health left) fits its target changes, judged
among the enemies it was choosing from; and when half or more of its changes were to an enemy after it, the boss aside,
`"mine"` first in its focus, or right after `"boss"` for one that kept to the boss), its range (median distance to its target when it acted, melee or to the nearest 5 yalms), how near it stayed to
you (90th percentile), and its heals (whom, with what, how low they were). As `content/lib/Battle.lua` roles: a
`BATTLE:ally` line each for the start, then a `"change"` step for each phase where a role changed.

**Telegraphs**: every enemy cast with an area: its shape and size (CastType and EffectRange read as Kawari does), omen,
cast time, how often and when, and whether it was aimed at a target or the ground.

From Recorder 2.6 files, also: a warning naming any hook that wasn't working (what the recording can't have); **your
setup** (job, level, sync, item level, Duty Finder settings, party, statuses at the start); action names from the
recording's `ActionInfo`; tethers, event objects changing, fast moves (knockbacks) and auto marks in the timeline;
**your enmity** (who topped the list of enemies on you, when it changed); and **actor controls** counted by category,
named from Kawari's `ActorControlCategory` (`repo.actor_control_names()`), so a category Kawari doesn't know stands
out.

Tried on Bob's Kawari capture of To Guard a Guardian (converted to the same shape): its "never acted" list started
with the three background NPCs that stood idle there (3854723, 3854588, 3854715).

**Who's in the fight.** The outline only looks at NPCs linked to you by hits, heals or targets (directly or through
others, `Battle.in_fight`), and its "boss" is the healthiest enemy that's engaged: it dies, loses 5 % of its HP or more,
you hit it, or it comes after you (`Battle.engaged`). Without that, To Guard a Guardian's background Ixali (79750 HP,
fighting guards nobody else touches) were taken for the boss, and Galfrid looked like he never went for it.

## chronofoil

`questkit chronofoil <Chronofoil folder or captures>` indexes retail sessions recorded with Chronofoil (see
[Outside sources](sources.md#chronofoil)): one entry per capture (start, length, game version, censored or not), then
every zone and duty in it from one ZoneInit to the next, with what happened there (NPCs spawned, casts, action effects,
DirectorVars, battle talk, NPC yells). `--duties` keeps only duties, `--min-minutes` drops short visits, `--capture 3,5`
picks captures, and `--names` takes a JSON of territory and duty names from the game data (never committed).
`--export N.V --out file.jsonl.gz` writes visit V of capture N as a Kawari packet capture (the format `capture.py`
reads, chat left out), so `compare` and the rest can use it.

* **The ledger** (`ledger.py`, since 2026-11-10; [the plan](capture-pipeline.md), stage B): one pass over a capture
  into typed events with nobody named (an actor is a number counted per capture, 0 the capturing player; entity IDs,
  names and chat never leave the decoder): `zone`, `class`, `spawn`, `despawn`, `request` (the client's ActionRequest),
  `cast`, `effect` (with each target's effects and where source and targets stood), `result` and `statuses` (statuses
  with their durations and who gave them), `vitals`, `gauge`, `recast`, `control` and `trigger` (every ActorControl and
  ClientTrigger with its category and arguments), `condition`, `first_attack`, `loot`, `director`, and a closing
  `coverage`. The module's docstring has each event's fields. Ledgers are cached as gzipped JSON lines
  (`$QUESTKIT_LEDGERS`, `--ledgers FOLDER`, or `~/.cache/questkit/ledgers`; `<capture ID>.v<decoder version>`): all 124
  captures build in 80 s into 30 MB, and a fact table over them takes 8 s. `--ledger OUT.jsonl.gz` writes one out.
* **Fact tables** (`ledger_facts.py`), each a flag, printed as text or written as JSON with `--out` (those of
  2026-11-10 are in the diagnostics repository, `facts/`):

  | Flag | Gives |
  |---|---|
  | `--coverage` | Per game version: packets named of all, kinds, unnamed opcodes with their sizes, packets not their table's size |
  | `--uses [--job SMN]` | Per job and level band: each action's uses and uses a minute in combat (gaps under 8 s), what follows what; `thin` under two minutes |
  | `--statuses [--npcs]` | Per action: the statuses it gives its target or its user (effect kinds 14 and 15), in how many of its uses, with the durations and parameters the EffectResults carried |
  | `--gauges` | The capturing player's ActorGauge bytes by job: which of the 15 move and between what, and which of their own actions came before each change |
  | `--actions [--all]` | Per action the capturing player used: cast time, animation lock, the shortest gap between uses (the fifth percentile: its recast), request-to-effect time, the most it hit at once |
  | `--areas` | Enemies' cones as Boss Mod has them against who they hit: how far off the caster's facing the people hit stood ([Combat](combat.md)) |
  | `--controls [--all] [--sapphire CommonActorControl.h]` | The ActorControl and ClientTrigger categories Kawari's enums don't have (or all), with counts, an example's arguments, how many values each argument took, and Sapphire's name for the category when its header is given |
  | `--spawns` | Every instanced territory's battle NPCs as retail spawned them (BNpcBase, BNpcName, level, HP, character data flags, display flags, Mount, ModelChara; first sighting, visits; since 2026-11-14 whether it ever `moved` or `acted` in any visit, and the event NPCs with a layout ID: `kind` 3, their ENpcBase, `late` when never there as a visit began), merged into `tools/questkit/retail_spawns.json` (numbers only), which `lint` holds the quest battle layers to |
  | `--registry` | Each capture: version, start, minutes, visits, duties, the capturing player's jobs and levels, their own actions (`notes/captures.json` in the diagnostics repository) |

  `--battle`, `--hits`, `--quest` and `--dropin` still read the capture themselves (see [Known issues](known-issues.md)).
* **Reading captures** (`chronofoil.py`): finished captures are a zip of `VersionInfo`, `CaptureInfo` (protobuf) and
  `Data` (zstd-compressed, delimited protobuf frames holding the game's frames, already decompressed); unfinished ones
  are a folder of the same files, `Data` not compressed. Finished captures need zstd: `pip install zstandard`, or
  Python 3.14. One copy per capture ID is read (the censored one if both are there).
* **Opcodes by game version** (`data/chronofoil_opcodes.json`, made by `chronofoil_opcodes.py`): Kawari's own table for
  the version it supports, plus saved tables for older ones. 2026.06.10 and 2026.09.15 are Kawari's opcodes.yml as it
  was then; 2026.06.18 and 2026.09.01 are a hotfix away from those, shifted with FFXIVOpcodes' history (it has about a
  third of Kawari's packets, by its own names, for every patch), then filled in by unique sizes and by hand
  (DirectorVars, the only 24-byte packet starting with a director ID). Every table was checked against the size of
  every packet in Bob's captures of that version; the index warns when packets aren't the size their table says.
  **Before merging upstream's next patch update, run `questkit chronofoil --save-opcodes`**, so captures from the
  outgoing version stay readable. Kawari Recorder exports use the saved tables too: their notices' opcodes come from
  `fork_opcodes.yml` for the current version and from `data/chronofoil_opcodes.json` for older ones
  (`repo.notice_opcodes`). Until 2026-10-21 it read only `fork_opcodes.yml`, so once 7.57h1's opcodes replaced
  2026.10.06's every older recording lost its notices and five of questkit's own tests failed in CI (from `85124c4`,
  2026-10-20, on every push since). A Recorder export's own GameVersion isn't trusted either
  (`Recording.notice_opcodes`, since 2026-11-12): the known version whose notice opcodes fit its quest packets best
  (a notice to the quest's own handler) wins. Bob's export of 2026-11-12 said 2026.10.06 with 7.58's packets, so every
  fixture made from it had no notices until then.
* About 0.5 s per hour of play: the whole of Bob's four months (42 sessions) index in 25 s.
* **`--dropin TERRITORY --appends <lgb>`** writes the territory's open-world battle NPCs as a drop-in
  (`chronofoil_dropin.py`): every battle NPC SpawnNpc seen there gives a spawn point with its layout ID, BNpcBase,
  BNpcName, level, retail HP, hostile flag (character flags 0x2), links, display flags, position and facing (the
  quantized u16 at CommonSpawn + 98). Left out: FATE monsters, NPCs someone owns (pets, chocobos), spawns without a
  layout ID or turned from an event NPC, and `--exclude-base`s; `--one-spot BASE` keeps one spawn point of something
  that moves between them (a hunt mark). Monsters wander, so each goes where it was seen nearest the average of its
  sightings. Humanoids (ModelChara 0, CommonSpawn + 96) keep how retail drew them as `appearance`: ClassJob (+ 119),
  the two weapons (+ 16, + 24, 8 bytes each), the ten gear slots (+ 508, 40 bytes) and the customize data (+ 594, 26
  bytes), which the server puts over what BNpcBase gives (2026-10-27). Guards (character data flags 0x4) keep the flag
  and their Behavior row (CommonSpawn + 94). Each sighting
  records how long the NPC was seen (until DeleteActor, whose actor ID is at byte 4, or leaving the zone), whether it
  moved more than a yalm while idle (ActorMove's packed position at byte 6) and whether it fought (its own
  ActorControlTarget, ActorCast or actions): a spot is `stationary` when it was seen idle a minute or more without
  moving and never seen idle and moving, and so are the briefly seen spots of a kind that never moved in two or more
  such sightings. Level 50+ NPCs are listed after the summary, to find hunt marks (`--exclude-base` the S ranks,
  `--one-spot` the A and B ranks). The LGB is the TerritoryType's `Bg` folder plus `planmap.lgb` (Central Thanalan:
  `bg/ffxiv/wil_w1/fld/w1f2/level/planmap.lgb`). Flying around a zone with Chronofoil running is enough; Central
  Thanalan's drop-in came out of two such sessions (see [Combat](combat.md#open-world-spawns)).
* **`--hits OUT.jsonl.gz`** (`chronofoil_hits.py`, since 2026-11-10) writes every hit and heal the capturing player
  dealt, one JSON line each, numbers only: the class and levels (UpdateClassInfo), PlayerStats, the main hand's item ID
  (UpdateItem, container 1000 slot 0), the statuses on the player then, the territory, the action, the amount and its
  flags (damage critical 0x20 / direct hit 0x40 in the first parameter, heal critical 0x20 in the second), how many it
  hit, and the target's level and BNpcBase. The damage formula was fitted from it ([Combat](combat.md)).
* **No `zstandard`?** Finished captures are also read through the system's libzstd (ctypes, `_LibZstdReader`) when
  neither `zstandard` nor Python 3.14's `compression.zstd` is there (since 2026-11-10).
* **`--dropin TERRITORY --gathering --appends <lgb>`** writes the territory's gathering points instead: every
  SpawnObject of kind 6 seen there, each position with its layout ID, GatheringPoint and retail's `args1`
  (`spawn_args`), leaving out those sent with no integrity (leve points). See [Gathering](gathering.md).
* **`--battle N.V`** turns a visit into a battle run (`chronofoil_battle.py`), saved as `.json.gz` for `questkit
  battle` (or straight to a report with `--out x.md`). From the packets, with Kawari's struct layouts: actors from
  SpawnNpc/SpawnPlayer (BNpcBase and BNpcName, layout ID, HP, battalion, level, class, position; players other than
  you are just "player"), positions (ActorMove, ActorSetPos), HP (UpdateHpMpTp, EffectResult(Basic): the HP is the
  packet's own actor's), **targets** (ActorControlTarget SetTarget, which retail sends for NPCs too), deaths
  (Defeated), every action with its targets and effects (an AoeEffect's from byte 42, as an ActionEffect1's; until
  2026-10-30 they were read from 48, six bytes off: Blizzard II's 15 damage came out as an effect of kind 15 and its
  first target's miss was lost, so AoE amounts read from earlier runs are suspect), casts in the Recorder's text, the director's sequence and
  variables, its scenes (60000 starts the fight, 60001 is "Duty Complete") and when the client finished each, battle
  talk, NPC yells, map effects, every actor control, and the enemy list (HaterList, EnmityList). Facings
  (`R`) since 2026-11-05: a spawn's, each ActorMove's (the quantized u16 at byte 0), and your own position and facing
  from your client's UpdatePositionHandler (rotation f32 at 0, position at 8) or UpdatePositionHandlerInstance
  (position at 12); before, your position only changed when the server moved you, so a run's player stood at their
  spawn. It has no names (no game data here: NPCs show as `base N [layout]`), no Action sheet (shapes, omens, class kits) and no
  layout layers; positions only change when a packet moves someone.
* **`--quest Q[,Q]`** prints what those quests did (`chronofoil_quest.py`): talks, scenes with their flags and the
  client's answers (yield or return, results, error code), notices, quest updates, and the zone changes, attunements
  and unlocks while their events were open. `questkit fixture --chronofoil` makes fixtures from the same reading.
* Checked on To Guard a Guardian against the Recorder export of the same run (Recorder 2.3, 6241 server packets): the
  capture has every one of them. 4257 are byte for byte the same; in the rest (ActionEffect1, ActorCast, AoeEffect8,
  StatusEffectList, a few spawns) the client had already changed fields in memory (action IDs, effect entries, status
  timers) by the time the Recorder's hook saw the packet. **For packet layouts, Chronofoil's bytes are the wire's.**

## battle-sim

```shell
python3 tools/questkit battle-sim                 # every quest battle against retail
python3 tools/questkit battle-sim 12 -v           # one, with every event's timing
python3 tools/questkit chronofoil chronofoil/ --battle 41.6 --out run.json.gz
python3 tools/questkit battle-sim --make 12 --run run.json.gz --source "…"   # its fixture
```

**Battle fixtures** (`tools/questkit/battles/<quest battle>.json`, numbers only) hold what the battle did on retail,
in seconds from the fight starting (the director's scene 60000, Kawari's `onCommence`): each NPC's health curve, when
it appeared (or `null`: there from the start) and when its health reached 0, how long the client took over each of
the director's own scenes, and the events to match: the director's sequence changes (not the 0xFE the server sets
when it's over), battle talk by text ID, the NPCs that arrived during the fight, and "won" (scene 60001). From a
Kawari Recorder battle file (`--run battles/….json.gz`) the battle talk is read from its DirectorPopupMessage packets
(236 on 7.58; Recorder 2.7 logs no "battle talk" event, `_talk_events`, since 2026-11-12), and the director's scenes'
lengths aren't in it (no scene end): give `scenes` by hand from its EventFinish packets (Austerities of Earth's scene
6, 3.33 s). One of a pool counts as retail's when another of it is (Austerities of Wind's plumes, one of three at
random each time, only two of which came in the won run.) An NPC
the script spawns outside `onUpdate` (in `onCommence`, `onActorDeath`, `onSceneFinished`) has no health on the next
update, as on the server, which runs the director's spawns after `onUpdate`.

`battle_sim.py` runs the director script with `content/lib/Battle.lua` in Lua 5.4: `onSetup` (not compared: retail's
sequence 1 comes before the opening cutscene too), `onCommence` at 0, `onUpdate` every half second with each spawned
NPC's health from retail's curve, counted from when the *script* spawned it (a wave brought in late also falls late),
`onActorDeath` when its health reaches 0 (when Kawari calls it; retail's Defeated comes about 2 s later, after the
death animation), and `onSceneFinished` after as long as the client took on retail. Retail's events are then matched
to the script's, in order, each within the fixture's tolerance (2 s). It also fails on an NPC the script spawns that
no drop-in has, or that retail never had; on one retail had from the start that the script never brings out; and on
one the script brings out more often than retail did (`spawn_problems`; the timed events only cover later arrivals,
so Like Mother, Like Daughter's first wave, written the wrong way round, passed until 2026-11-01). NPCs the script spawns
in `onSetup` count as there from the start (since 2026-11-06: The Rematch's 39, out before its opening cutscene,
as retail had them).

**The line-up every run had** (`retail_line_up`, `line_up_problems`, since 2026-11-14). A fixture is one run's battle
NPCs, so it can't say who stood by or who never stirred; `retail_spawns.json` (every capture, `chronofoil --spawns`)
can, for the territory the fixture's NPCs are in. `battle-sim` fails when:

* retail had an **event NPC** (ObjectKind 3, with a layout ID) standing there in every run and the script never
  brings it out (`director:spawn_bnpc` of its layout, from the battle's drop-in layer). One who is never there as a
  visit begins (the registry's `late`: the three who come as Escape from Castrum Centri's cutscene ends) isn't
  asked of a lost run;
* a battle NPC **never moved and never acted** in any run and the script brings it out without holding it
  (`director:hold_bnpc`, which a role's `still = true` does), or lets it go again.

The first run found 17 in four battles that had passed: Every Little Thing She Does Is Magitek (64: two event NPCs,
the reaper), Escape from Castrum Centri (65: seven event NPCs, three field generators), On Holy Ground (22) and
Victory in Peril (24), one event NPC each ([Quest battles](quest-battles.md)). Bob's mark of 2026-11-14 in 64 ("the
magitek reaper shouldn't move, and Biggs, and Wedge are gone") is what it would have caught.

`--make` takes an NPC's death from its ActorControl 2 with 2 (its status: dead) when no HP of 0 came first (since
2026-11-12: Notorious Biggs's boss stood at 25 HP until it, and fell 2.1 s before Defeated, in Bob's capture da424662;
before, his death came out at Defeated). `--make` leaves out a sequence set before the fight that another replaces by the time it starts (the script's
`onSetup`: Trial by Wind's sequence 1 came 0.4 s before scene 60000, inside the second it used to keep), and when
retail spawned one layout ID twice (Trial by Wind's 4248510, a second wind sprite while the first still stood) the
first is the NPC and the second only a spawn event; before 2026-10-27 the second overwrote the first, so the first
looked late. A second run of the same battle goes beside it as `<id>-<suffix>.json` (`<id>-lost.json`, or the
capture's ID: `57-1fac2416.json`; `--make` writes `<id>.json`, so rename it), and `battle-sim <id>` runs them all. A
quest's second run does the same (`fixtures/<quest>-1fac2416.json`, renamed from what `fixture` writes), and
`replay` runs every one.

A fixture can also hold retail's **positions** (`positions`: `{"player": [[t, x, y, z], …], "<layout>": […]}`, an
NPC's times counted like its health; written by hand: Victory in Peril's are the player's from the capture's
`UpdatePositionHandlerInstance` and the NPCs' from the run's samples, on one clock), which the script sees in `info`
as the server gives them, for steps that wait for the player to come close. Without them everyone stands at the
origin, and anything waiting for the player to come close happens at once.

A battle whose director played a scene of its own between the opening and the fight has a **lead-in** (`lead_in`:
`{"<scene>": seconds}`, how long the client took over each, written by `--make` since 2026-11-03). The Lominsan Way's
battle (59) plays scene 2 for 16 s, with sequence 2 and three lines of battle talk, before scene 60000: Kawari calls
`onCommence` as the opening ends, so the simulator does that long before 0, the script plays the scene from there
and starts the fight as it ends (`onSceneFinished`), and retail's sequence and talk meanwhile are matched at negative
times. NPCs the script brings out before the fight follow retail's health from its start. Since 2026-11-08 `--make`
also writes when retail's first lead-in scene began (`lead_in_from`, seconds before the fight), and the simulator
calls `onCommence` then: History Repeating's battle (86) has two lead-in scenes and a second's wait after them, which
counting back their lengths alone put a second late. Older fixtures without it are as before.

NPCs retail brought out **at random from a few**, over and over, are a **pool** (`pools`: `[[layout, …], …]`, written
by hand since 2026-11-08): a spawn of any layout of a pool matches a retail spawn of any other of it, the script can
have several of one out at once (each spawn of one is followed), the counts are checked per pool, and their retail
deaths aren't used (the script has them fall). History Repeating's bombs, one of three layouts each time.

And retail's **casts** (`casts`: `[[t, layout, action], …]`, the run's ActorCast events by the caster's layout ID,
written by `--make` since 2026-10-28): the script hears `onActorCast(director, layout, action, 0)` at each, as from the
server, for steps that wait for an NPC's cast (the companies' battles' Drill Cannons).

And the player **talking to an object** the script brought out (`object_talks`: `[[t, base, bar], …]`, written by
hand from retail's StartTalkEvent on it and how long the event action that followed took; since 2026-11-06): at `t`,
if the script has spawned that object (`spawn_eobj`), it hears `onObjectTalk(director, 1, 2, base)`, and an
`event_action` it answers with ends `bar` seconds later (`onEventActionCast(director, 1, 2)`). In the Eyes of Gods
and Men's rosary (quest battle 60), without which its battle never ends.

`--make` writes a health of a few HP as 0.1 %, not 0 (since 2026-11-06): In the Eyes of Gods and Men's boss stood at 1
of 9150 HP for 3 s in one run, which rounded to 0.0 and put its death 3 s early.

It doesn't simulate enmity or who hits whom (the health curves are retail's whatever the script's
targeting does), or the NPCs' casts beyond `casts` and their yells (not compared). Losing is: when the player's health (`player`) reaches 0
the script hears `onPlayerDeath`, as from the server (since 2026-10-27: Like Mother, Like Daughter's lost run went on
spawning past retail's end). An event NPC the script spawns (retail's runs list battle NPCs only) goes in the
fixture's `npcs` by hand, there from the start (56's Sylphie, 4249177). The player's health is retail's when the fixture
has it (`player`, written by `--make` since 2026-10-21: for lines said as it drops), as the last value seen (it
jumps, with hits and potions, so it isn't interpolated); full otherwise. `tolerances` (by label, e.g.
`{"talk 86": 4.5}`) gives one event more slack where the runs disagree; `--make` doesn't write it. NPCs' health is
interpolated between samples, unless the fixture says `"npc_health": "held"` (written by hand): then it holds until
the next one, for runs where the player stopped attacking for a while (quest battle 29: interpolating put a 30 % mark
53 s early). Holding it for every fixture throws Chasing Shadows' and Lurkers in the Grotto's marks off by 3–5 s.

Every quest battle has a fixture from Bob's Chronofoil captures and matches. The first run found three of them
timed from entering the zone rather than from the fight starting (Chasing Shadows' and Spirithold Broken's sequence 2
and first lines came 8–10 s late), Sapphire's health marks where retail's differ (Chasing Shadows' second wave at 45 %,
not 50 %; Spirithold Broken's phases at 70 % and 45 %, not 65 % and 30 %), To Guard a Guardian's last line at 42 %
and its "Duty Complete" 2 s after the mage falls, and the lines after a group falls 2–3 s early (see
[Quest battles](quest-battles.md)).

## sources

`questkit sources` reads two public projects' knowledge of duties ([Outside sources](sources.md#boss-mod)), kept in
the repository as numbers-only extracts so nothing needs downloading:

* `questkit/data/public/bossmod.json` (`public_sources.py`, from Boss Mod's `BossMod.Modules/`): per module its duty
  (ContentFinderCondition) and boss (BNpcName), its actors' BNpcBase, the arena, and every action with who uses it on
  whom, its cast time, its shape (circle, cone, rectangle, ring, cross, with sizes; a cone's angle and a rectangle's
  width are whole widths, where Boss Mod writes halves), what it is (Boss Mod's component: raidwide, tankbuster,
  stack, spread, knockback, gaze...) and whether Boss Mod marks the shape as its guess. 902 modules, 284 duties,
  18,683 actions (Boss Mod `6e410ef`, 2026-11-03). Boss Mod's names for actors, actions and statuses are the game's
  text and are left out: an actor is `Boss`, `Helper` or `actor<BNpcBase>`.
* `questkit/data/public/job_actions.json` (from Boss Mod's `BossMod/ActionQueue/<role>/<JOB>.cs`): each job's actions
  with the level, whether it's on the global cooldown as the server counts it (the Action row's own CooldownGroup:
  one with its own recast that also sets the global cooldown off, the Summoner's demi-summons or Phlegma, isn't), its
  recast and cast time, and whether it's for oneself or friends. 22 jobs; what `questkit rotation` paces a bot by.
* `questkit/data/public/cactbot.json` (from cactbot's `ui/raidboss/data/**/*.txt`): each timeline's entries (time,
  action IDs, whether it's the cast's start, where it loops) by territory. 306 timelines; ability names left out.

| Option | Does |
|---|---|
| (none) | Counts what the extracts hold |
| `--build --bossmod <checkout> --cactbot <checkout>` | Remakes the extracts from sparse checkouts (`git clone --depth 1 --filter=blob:none --sparse`, then `git sparse-checkout set BossMod.Modules` / `ui/raidboss/data resources/zone_id.ts`); one module or timeline a line, so the diff shows what changed |
| `--audit [--out file]` | Every timeline in `resources/timelines/` whose BNpcBase a Boss Mod module has: the module's actions for that NPC the fork never mentions (the timeline, the duty scripts, `action_areas.json`), and cones recorded at another angle than Boss Mod's. Run on 2026-11-10: 44 timelines have a module, one action unmentioned (835, Sastasha's last boss's adds, [Known issues](known-issues.md#sastasha)), no angle differs |
| `--duty CFC [--territory T] [--out folder]` | A draft of a duty: one timeline file per boss in the fork's format (the boss's own actions, each with its cast, shape and kind in a comment; `every` from cactbot where it has the territory) and a `README.md` saying the arena, the other actors, the helpers' actions and what's missing. Marked as a draft in its `comment`; no damage, HP or levels (a capture or the game's data) |
| `--duty CFC --captures <captures…>` | The same draft with what retail did in Bob's captures of the duty (from their ledgers, `capture_facts`): each boss's level and HP, each action's uses, median time between uses (`every`) and median damage to players and to NPCs (`recorded_amounts`), where the helpers cast from (the spots the director places them at), actions the captures have that Boss Mod's module hasn't, and a timeline file for every other enemy of the duty that fought (`Duty<CFC>Enemy_<BNpcBase>.json`: its auto-attack, actions, amounts, level and HP in the comment). Castrum Meridianum (15) from one capture: 3 bosses and 17 other enemies in 11 s. Drafts for the 83 duties in the captures where something fought are in the diagnostics repository (`drafts/`) |
| `--action N` | What Boss Mod says about one action |
| `--server-data` | Writes `resources/data/public_action_areas.json` ([Combat](combat.md): area shapes) |

Code: `public_sources.py` (the readers and `Public`), `duty_sources.py` (`audit`, `draft`, `server_data`). Tests:
`PublicSourcesTest` (a made-up module and timeline).

## questionable

`questkit questionable <Questionable checkout>` checks every quest script against
[Questionable](https://github.com/PunishXIV/Questionable)'s path for its quest ([Outside Sources](sources.md)): every
ENpc and EObj a step talks to or uses must be one of the script's constants that the script refers to (the values
from the quest's fixtures' `params` or its quest dump), every key item used on something one of its ITEMs, and every
sequence the path goes through one the script's header speaks of ("Sequences 2, 4, 6:"). Steps for another quest
(PickUpQuestId, TurnInQuestId) or for getting somewhere (TargetTerritoryId) and items used on nothing (Gysahl greens, a
ticket) are left out. `--quest N ...` checks only those, `--out FILE` writes the report (outside the repository: the
paths hold game text), `--hints N` prints a quest's steps as numbers (targets, positions, items, the variables after
each step as seen on retail) before it's scripted. Code: `questkit/questionable.py`; tests `QuestionableTest`.

## autoduty

`questkit autoduty <AutoDuty checkout>` checks the duty scripts in `dungeons.md`'s table against
[AutoDuty](https://github.com/ffxivcode/AutoDuty)'s route for their territory: every EObj it uses and every key item a
door waits on must be a number the script has. `--draft TERRITORY` prints a duty's route for starting one: its objects
in order with positions, the keys, where each boss was fought (and its arena), the coffers. AutoDuty is unlicensed:
nothing of it goes in git, reports included. Code: `questkit/autoduty.py`; tests `AutoDutyTest`.

## fishing

`questkit fishing <AutoHook checkout>` remakes `resources/data/fishing/fishing.json` from AutoHook's fish data
(BSD-3; Teamcraft's catch logs and FF14 Fish Tracker's, MIT): per fishing spot and bait the fish caught there with
their counts and bite seconds (minimum, tenth, median, ninetieth, maximum; no lures; nothing caught only once), and
per fish its tug, hookset, bait, mooch, predators, weathers, window (Eorzea minutes), least Gathering, Fish Eyes and
intuition. Code: `questkit/fishing_data.py`; test `FishingDataTest`.

## rotation

`questkit rotation <classjob> <level> [targets]` (`rotation_sim.py`) plays a party bot's rotation for three minutes
against a dummy as the server would: the job's script in `resources/scripts/rotations/` is asked every half second,
and each action it picks is run from its own script in `resources/scripts/actions/` against a mock EffectsBuilder,
whose calls (statuses on itself and its target with their durations and stacks, gauge changes within the ranges
`effects_builder.rs` clamps them to, MP) are what the next choice sees. It has the global cooldown and cast times,
recasts (`data/public/job_actions.json`), combos by combo steps, `action.rs`'s instant casts and the gauge's tick; it
has no damage (every hit is 1), no enemy that dies and no movement. It prints each action's uses a minute, the first
forty in order, the actions for enemies on the global cooldown it never used, and any script error.

* `--against uses.json` puts retail's uses a minute for the job and level band beside the bot's (`questkit chronofoil
  <captures> --uses --out uses.json`). The Summoner at 80: Astral Impulse 4.0 a minute against retail's 2.8, Topaz Rite
  4.0 / 4.2, Emerald Rite 4.0 / 3.7, Ruin III 3.7 / 1.8, each egi 1.0 / 0.9-1.1, Bahamut 0.7 / 0.6.
* `--check` runs every rotation for each of its ClassJobs at levels 1, 10, 15, 20, 30 ... 100 (a class to 30, a job
  from 30 or where it starts), alone and against three, and fails on a script error or a bot using fewer than 15
  actions on the global cooldown a minute. `RotationSimTest` runs it in CI (12 s). Its first run found the
  Thaumaturge's rotation idle at level 1 ([Party bots](party-bots.md)).

## travel

`questkit travel <dump folders>` writes `resources/data/quest_travel.json`: the NPCs quests offer rides with, in every
sequence the client offers them, answered by `events/quest/Travel.lua` ([Quests](quests.md), Quest travel). It reads
the client scripts from kawari-questdump's output, replays each recorded ride for its move, and probes each quest's
script for what it answers. It prints the rides given from another quest's (`inferred`), questions with no known way
(`questions`), sequences left without a copy (`unknown`) and, with `--silent`, every talk the client offers that the
script doesn't answer. `--check` fails if the file isn't what the dumps give. A fixture with `"travel": true` tests a
ride it gave and isn't read as a recording.

## lint

`questkit lint` finds what fails silently in game:

* an action an NPC uses with no action script (the server only logs "isn't scripted yet" and the NPC does nothing):
  every action in a timeline (auto-attack, timepoints, rotation, recorded amounts, on death) and the `ACTION_*`
  numbers in quest battle scripts;
* a battle NPC in a quest battle's drop-in layer (`QB_*`) with no timeline (it fights with the defaults); open-world
  layers are only counted;
* a battle NPC in a quest battle layer unlike retail's spawn of it (`retail_spawns.json`: BNpcBase, BNpcName, level,
  HP, character data flags, display flags, Mount), and an NPC retail spawned in that territory that no drop-in has;
  `lint --fix-spawns` writes retail's values in (Fool Me Twice's boss rode a Magitek Reaper on retail and stood on
  foot in Kawari: Bob's bundle 20261114-015635);
* a fork quest script whose header says what it was "Checked against" with no fixture, a quest battle script with no
  battle fixture, and fixture steps still marked `todo`.

Its first run found Chasing Shadows' frenzied oaks (BNpcBase 886) without a timeline; they have one now (from the
Recorder and the capture: hitbox 1.5, auto-attack 872, 12 a swing on Papalymo, misses included).

## lua-syntax

Compiles every script with Lua 5.4's `load` (nothing runs). It found upstream's `FesHlwTransform_00946.lua`
(`SCENE_00000` without a value), which never loaded; fixed in the fork.

## CI (`.github/workflows/fork.yml`)

| Job | Runs on | Does |
|---|---|---|
| Quest replays and scripts | every push and PR | questkit's tests (with lupa and zstandard), `lua-syntax`, `replay -v`, `battle-sim -v`, `duty-sim` (since 2026-11-10), `lint` |
| Formatting and changelog | every push and PR | `tools/ci/check-changes.sh`: `rustfmt --check --edition 2024` on the changed `.rs` files only, and a changelog entry for any change outside `docs/` (skip with `[no-docs]` in a commit message, e.g. upstream merges) |
| Documentation builds | every push and PR | `mdbook build docs` (mdBook 0.5.4) |
| Build and test | PRs, pushes to master, by hand | `cargo build --workspace --locked`, `cargo test --workspace --locked -- --skip test_item_level` |

The repository is private, so Actions minutes are limited: the quick jobs take about a minute each, the full build
several, which is why it doesn't run on every push to the working branch (`test-pending`). Run it by hand from Actions → Fork checks → Run
workflow. Upstream's own CI (`.forgejo/workflows/main.yml`) is for Codeberg and doesn't run on GitHub.

## Adding to it

* A new quest: `scaffold` (or write it by hand), then keep its fixture; CI replays it from then on.
* A new kind of step: add it to `harness.py` (what the server does), `replay.py` (the step) and `fixtures.py`
  (recording it), with a test.
* A new packet the tools should understand: `capture.py` reads Kawari's `decoded` text, `recording.py` the raw bytes.

## fields

`questkit fields --retail <Chronofoil> --kawari <Kawari captures>` lists what retail sends that Kawari doesn't: packets
it never sends, and bytes retail fills in that Kawari always sends as zero, with retail's values
(`questkit/fields.py`). What it found is tracked in [What Retail Sends That Kawari Doesn't](unfilled-fields.md).

## clientcheck

`questkit clientcheck <dumps> [--out report.md] [--all]` checks the server's event scripts against the client's
(`questkit/clientcheck.py`). Dump them first, from the server folder:

```shell
kawari-questdump scripts custom --server resources/scripts --out client-scripts
kawari-questdump scripts opening --server resources/scripts --out client-scripts
python3 tools/questkit clientcheck client-scripts --out client-check.md
```

Quest dumps (`kawari-questdump dump`'s `<script>/<script>.txt` and `quest.json`) in the same folder are checked too.
For every server script with a client script there it reports, worst first:

* **scenes the server plays that the client doesn't have** (a constant `SCENE*` the script uses, not only defines;
  a number or a constant passed to `play_scene`; or one of the sheet's `SEQ_*` scene parameters);
* **constants named as the client's but with another value** (the client sets them in its script's second chunk,
  `SETTABLE R0["MAIN_MENU_RESULT_SPECIALSHOP"] = 2`);
* **names that look like the sheet's parameters** (an upper-case name with a number: `ACTOR0`, `SEQ_1_EOBJECT2`) that
  the sheet row doesn't have and the script and Global.lua don't define: `nil` in Lua;
* client scenes the server never plays (a branch left out on purpose, or missed), and the script's `TODO`, `HACK`,
  `FIXME` and "not sure" lines;
* and which server scripts had no client script in the dumps.

ArchonEngine ControlPanel does the same from its dashboard, with no source tree: Quest Lab → Script Check (2.23.0; see
[ArchonEngine ControlPanel](kawari-web.md)). Kawari's scene 255 (returning from a nested event) isn't counted.
`client-scripts/` and `client-check.md` are gitignored, as `quest-dumps/` is: the dumps are Square Enix's files; the
report has only scene numbers, constant and parameter names and the server's own lines. Its first run (2026-10-27,
Jonathas and the six Conjurer quest dumps) found Trial by Earth's scene 99 never played: the
`SEQ_2_EOBJECT0_EVENTACTIONNO` scene retail plays when a search's bar is broken off (see Known issues).

The first full run (Bob's Script Check bundle of 2026-10-27, build `fde463d`: 94 NPC and opening scripts) found two
that play a scene the client doesn't have: Berkoeya Loetahlsyn (`RegSeaPvpColosseum_00102`) played 1, and the client
has only 0; and `RegYok6Tuliyollal_00864`, which only defined a `SCENE_00001` it never played (counted as played until
then, so the check now counts only constants used). `CtsMkdEntrance_00929` played the client's menu-again scene (1)
on the first talk, so Jeffroy never turned to the player: it plays 0 now. What's left are client scenes the server
never plays, mostly systems not built (the Grand Company officer, the retainer vocate's other menus, Wondrous Tails,
Free Companies, the Triple Triad tournaments); the openings' scene 30 is played by the first quests, not the opening
script.
