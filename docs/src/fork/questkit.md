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
| `{"talk": base}` | Talk to / examine an NPC or object (the event's `BASE_ID`) |
| `{"kill": [base, layout]}` | A battle NPC with this BNpcBase and layout ID died (`onBNpcKill`) |
| `{"range": arg}` | Walked into an event range (`onEnterTrigger`) |
| `{"enter_territory": id}` | Zoned in (`onEnterTerritory`) |
| `{"battle_won": sequence}` | A quest battle was won (the director sets the sequence) |
| `"expect": {"sequence", "vars", "notices"}` or `{"completed": true}` | The state after the step; `notices` is left out where the recording had no packets |
| `"spawns": [layout, …]` | The step must bring these out (`spawn_quest_bnpc`) |
| `"results": {"scene": [..]}` | What the client answers a scene with, if not `[1, 0]` |
| `"what"` | Which parameter the target is (for reading) |

**The harness** (`harness.py`) runs the real script under Lua 5.4 the way the world server does:

* a new Lua state per event: the scene flags (read from `event_scene.rs`), `Global.lua`, the parameters, the script,
  then `EVENT_ID` and `BASE_ID`;
* the script sees a copy of the quests: sequence/variable changes show up in it straight away, accepting doesn't;
* queued tasks run in order after each callback; accepting starts a quest at sequence 255 (as `accept_quest` does);
* the client's side is automatic: scenes return `[1, 0]` (accepted, handed over, first reward), progress bars
  complete, and `onReturn` (or `onYield`) gets the scene back.

A step fails if the script errors, leaves the event **unfinished** (no `finish_event`, the client would hang), plays
something after finishing, sends a notice with more than 4 arguments, or ends in a different sequence, variables or
notices than retail. **It doesn't check scene numbers** (the client decides what a scene shows), so those still need
the listing or a test in game.

All 23 scripted quests have fixtures and match (Close to Home and Chasing Shadows only after the fixes the replay
found, see the [Changelog](changelog.md)). The fixtures for quests with a quest battle use a `battle_won` step for it;
the battle itself isn't replayed (its director script was simulated with lupa separately).

### Making fixtures

```shell
python3 tools/questkit fixture all --recording rec.zip --recording older.zip --dumps quest-dumps --source "…"
```

Recorder 2.0+ records who was last targeted, so each step comes out filled in (`talk`/`kill`), with the notices
matched to the change that follows them (within 0.5 s). Older exports don't, and their steps come out as `{"todo": …}`
to fill in by hand from the script's header. Steps that change nothing (searching the egg in Migrant Marauders,
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

From Recorder 2.6 files, also: a warning naming any hook that wasn't working (what the recording can't have); **your
setup** (job, level, sync, item level, Duty Finder settings, party, statuses at the start); action names from the
recording's `ActionInfo`; tethers, event objects changing, fast moves (knockbacks) and auto marks in the timeline;
**your enmity** (who topped the list of enemies on you, when it changed); and **actor controls** counted by category,
named from Kawari's `ActorControlCategory` (`repo.actor_control_names()`), so a category Kawari doesn't know stands
out.

Tried on Bob's Kawari capture of To Guard a Guardian (converted to the same shape): its "never acted" list started
with the three background NPCs that stood idle there (3854723, 3854588, 3854715).

## lua-syntax

Compiles every script with Lua 5.4's `load` (nothing runs). It found upstream's `FesHlwTransform_00946.lua`
(`SCENE_00000` without a value), which never loaded; fixed in the fork.

## CI (`.github/workflows/fork.yml`)

| Job | Runs on | Does |
|---|---|---|
| Quest replays and scripts | every push and PR | questkit's tests, `lua-syntax`, `replay -v` |
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
