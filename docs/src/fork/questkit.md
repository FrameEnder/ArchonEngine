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
| `compare --capture … --recording … [quests]` | What Kawari sent for a quest against what retail did |
| `lua-syntax [folder]` | Checks every Lua script compiles (Lua 5.4, like the server) |
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
    packets), filtered to the tested quest when there is one.

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
| `{"talk": base}` | Talk to / examine an NPC or object (the event's `BASE_ID`); `"layout"` gives its layout ID, which the script sees as `target.object_id` (upstream's Ul'dah scripts tell NPCs apart by it). `"territory"` says where it happened, written when retail moved the player within that territory (PrepareZoning to the same one), so replay counts a script's `move_to_pop_range` as that zone change. |
| `{"kill": [base, layout]}` | A battle NPC with this BNpcBase and layout ID died (`onBNpcKill`) |
| `{"event_item": [item, base], "layout": …}` | A key item used on this NPC or object (`onEventItem`, after the server's EventItem event starts) |
| `{"range": arg}` | Walked into an event range (`onEnterTrigger`) |
| `{"action": [action, base], "layout": …}` | An action (its Action row) used on this NPC or object, which the quest listens for (`onActionResult`, after the server's ActionResult event starts): Trial by Water's Cure on the wounded Wood Wailer |
| `{"emote": [emote, base], "layout": …}` | An emote used on this NPC (`onEmote`, after the server's Emote event starts); `fixture --chronofoil` writes it from retail's EventStart of type 2 |
| `{"enter_territory": id}` | Zoned in (`onEnterTerritory`) |
| `{"battle_won": sequence}` | A quest battle was won (the director sets the sequence) |
| `{"duty_cleared": content}` | Leaving a duty the player cleared, its InstanceContent ID (`onDutyCleared`); `fixture --chronofoil` leaves it as a todo ("changed … outside an event") to fill in |
| `"expect": {"sequence", "vars", "notices"}`, `{"completed": true}` or `{"accepted": false}` | The state after the step (`accepted: false`: the offer was turned down); `notices` is left out where the recording had no packets |
| `"scenes": [{"scene", "flags", "params", "results", "yield", "error"}, …]` | Every scene retail played in the step, in order, with its flags (hex), parameters (when it had any) and the client's answer. The script must play the same scenes with the same flags and parameters, and each gets retail's answer (a fixture from Chronofoil) |
| `"effects": [{"zone": id}, {"aetheryte": id}, {"unlock": id}, {"content": id}]` | What retail did outside the quest while the event was open: zone changes (`change_territory`, a pop range, joining a quest battle), attunements, unlocks (ActorControlSelf 41, `unlock`), duties unlocked (ActorControlSelf 131 with the InstanceContent ID, `unlock_content`). With `scenes`, a step without `effects` must do none |
| `"spawns": [layout, …]` | The step must bring these out (`spawn_quest_bnpc`) |
| `"results": {"scene": [..]}` | What the client answers a scene with, if not `[1, 0]`; `{"results": [..], "error": n}` also sets the scene's error code (`player.scene_error_code`), for yes/nos that answer through it; `"yield": true` sends it back through `onYield` as retail's client did (EventYieldHandler), even when the script has `onReturn`; a yield the server answered with EventResume has `"resume"` (its parameters, which the script's `resume_event` must match) and `"after"` (the client's next answer, which replay gives the script after the resume) |
| `"what"` | Which parameter the target is (for reading) |

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
a kill. Talks that change nothing are steps too (agreeing to a quest battle); a progress bar broken off (a scene the
client never answered) isn't, since nothing happened. The talked-to actor is the layout ID of an event NPC or object;
its base ID comes from the server's SpawnObject packets, the Recorder export's NPC list (`--recording`, optional) or
the existing fixture, which also gives what the capture can't (kills, `spawns`) by matching the state each step left
the quest in. Unlocks in the step that completes a quest are left out: those are the Quest sheet's, sent by the
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
walking into an ambush) aren't recorded at all; add them with `spawns` when they matter.

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
  its allies. A rule that holds 6 times out of 6 is a rule; 3 out of 6 is a coincidence. Then each action: how many,
  how often (median and range), on whom, damage or heal amounts, and for heals how hurt the target was and whether it
  was the most hurt of its side.
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
of oldest, newest, nearest, most hurt or healthiest fits its target changes, judged among the enemies it was choosing
from), its range (median distance to its target when it acted, melee or to the nearest 5 yalms), how near it stayed to
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
  2026-10-20, on every push since).
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
* **`--dropin TERRITORY --gathering --appends <lgb>`** writes the territory's gathering points instead: every
  SpawnObject of kind 6 seen there, each position with its layout ID, GatheringPoint and retail's `args1`
  (`spawn_args`), leaving out those sent with no integrity (leve points). See [Gathering](gathering.md).
* **`--battle N.V`** turns a visit into a battle run (`chronofoil_battle.py`), saved as `.json.gz` for `questkit
  battle` (or straight to a report with `--out x.md`). From the packets, with Kawari's struct layouts: actors from
  SpawnNpc/SpawnPlayer (BNpcBase and BNpcName, layout ID, HP, battalion, level, class, position; players other than
  you are just "player"), positions (ActorMove, ActorSetPos), HP (UpdateHpMpTp, EffectResult(Basic): the HP is the
  packet's own actor's), **targets** (ActorControlTarget SetTarget, which retail sends for NPCs too), deaths
  (Defeated), every action with its targets and effects, casts in the Recorder's text, the director's sequence and
  variables, its scenes (60000 starts the fight, 60001 is "Duty Complete") and when the client finished each, battle
  talk, NPC yells, map effects, every actor control, and the enemy list (HaterList, EnmityList). It
  has no names (no game data here: NPCs show as `base N [layout]`), no Action sheet (shapes, omens, class kits) and no
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
when it's over), battle talk by text ID, the NPCs that arrived during the fight, and "won" (scene 60001). An NPC
the script spawns outside `onUpdate` (in `onCommence`, `onActorDeath`, `onSceneFinished`) has no health on the next
update, as on the server, which runs the director's spawns after `onUpdate`.

`battle_sim.py` runs the director script with `content/lib/Battle.lua` in Lua 5.4: `onSetup` (not compared: retail's
sequence 1 comes before the opening cutscene too), `onCommence` at 0, `onUpdate` every half second with each spawned
NPC's health from retail's curve, counted from when the *script* spawned it (a wave brought in late also falls late),
`onActorDeath` when its health reaches 0 (when Kawari calls it; retail's Defeated comes about 2 s later, after the
death animation), and `onSceneFinished` after as long as the client took on retail. Retail's events are then matched
to the script's, in order, each within the fixture's tolerance (2 s). It also fails on an NPC the script spawns that
no drop-in has, or that retail never had.

`--make` leaves out a sequence set before the fight that another replaces by the time it starts (the script's
`onSetup`: Trial by Wind's sequence 1 came 0.4 s before scene 60000, inside the second it used to keep), and when
retail spawned one layout ID twice (Trial by Wind's 4248510, a second wind sprite while the first still stood) the
first is the NPC and the second only a spawn event; before 2026-10-27 the second overwrote the first, so the first
looked late. A second run of the same battle goes beside it as `<id>-lost.json` (`--make` writes `<id>.json`;
rename it), and `battle-sim <id>` runs both.

A fixture can also hold retail's **positions** (`positions`: `{"player": [[t, x, y, z], …], "<layout>": […]}`, an
NPC's times counted like its health; written by hand: Victory in Peril's are the player's from the capture's
`UpdatePositionHandlerInstance` and the NPCs' from the run's samples, on one clock), which the script sees in `info`
as the server gives them, for steps that wait for the player to come close. Without them everyone stands at the
origin, and anything waiting for the player to come close happens at once.

And retail's **casts** (`casts`: `[[t, layout, action], …]`, the run's ActorCast events by the caster's layout ID,
written by `--make` since 2026-10-28): the script hears `onActorCast(director, layout, action, 0)` at each, as from the
server, for steps that wait for an NPC's cast (the companies' battles' Drill Cannons).

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

## lint

`questkit lint` finds what fails silently in game:

* an action an NPC uses with no action script (the server only logs "isn't scripted yet" and the NPC does nothing):
  every action in a timeline (auto-attack, timepoints, rotation, recorded amounts, on death) and the `ACTION_*`
  numbers in quest battle scripts;
* a battle NPC in a quest battle's drop-in layer (`QB_*`) with no timeline (it fights with the defaults); open-world
  layers are only counted;
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
| Quest replays and scripts | every push and PR | questkit's tests (with lupa and zstandard), `lua-syntax`, `replay -v`, `battle-sim -v`, `lint` |
| Formatting and changelog | every push and PR | `tools/ci/check-changes.sh`: `rustfmt --check --edition 2024` on the changed `.rs` files only, and a changelog entry for any change outside `docs/` (skip with `[no-docs]` in a commit message, e.g. upstream merges) |
| Documentation builds | every push and PR | `mdbook build docs` (mdBook 0.5.4) |
| Build and test | PRs, pushes to master, by hand | `cargo build --workspace --locked`, `cargo test --workspace --locked -- --skip test_item_level` |

The repository is private, so Actions minutes are limited: the quick jobs take about a minute each, the full build
several, which is why it doesn't run on every push to Latest-Quest. Run it by hand from Actions → Fork checks → Run
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

Kawari Web does the same from its dashboard, with no source tree: Quest Lab → Script Check (2.23.0; see [Kawari Web](kawari-web.md)). Kawari's scene 255 (returning from a nested event) isn't counted. `client-scripts/` and `client-check.md` are gitignored, as `quest-dumps/` is: the dumps are Square Enix's files; the report has only scene numbers, constant and parameter names and the server's own lines. Its first run
(2026-10-27, Jonathas and the six Conjurer quest dumps) found Trial by Earth's scene 99 never played: the
`SEQ_2_EOBJECT0_EVENTACTIONNO` scene retail plays when a search's bar is broken off (see Known issues).

The first full run (Bob's Script Check bundle of 2026-10-27, build `fde463d`: 94 NPC and opening scripts) found two
that play a scene the client doesn't have: Berkoeya Loetahlsyn (`RegSeaPvpColosseum_00102`) played 1, and the client
has only 0; and `RegYok6Tuliyollal_00864`, which only defined a `SCENE_00001` it never played (counted as played until
then, so the check now counts only constants used). `CtsMkdEntrance_00929` played the client's menu-again scene (1)
on the first talk, so Jeffroy never turned to the player: it plays 0 now. What's left are client scenes the server
never plays, mostly systems not built (the Grand Company officer, the retainer vocate's other menus, Wondrous Tails,
Free Companies, the Triple Triad tournaments); the openings' scene 30 is played by the first quests, not the opening
script.
