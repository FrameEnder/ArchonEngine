# From captures to server data: the plan

How Bob's Chronofoil captures become what the server does, and the order the work goes in from 2026-11-10. Written
after the three job stages ([Combat](combat.md)), which leaned on the captures question by question; this makes it one
pipeline, so a fact is extracted once, kept with where it came from, and checked against the server every time either
changes.

## Where it stands (measured 2026-11-10)

* **The captures**: 124, five game versions (2026.06.18 to 2026.10.16), 7.66 million zone packets (6.32 million from
  the server, 1.34 million from the client).
* **Named**: 98.7-99.4 % of the server's packets and 85.7-99.9 % of the client's have a name in their version's
  table (`questkit/data/chronofoil_opcodes.json`); 51-98 opcodes a version have none (mostly 8- and 24-byte ones). The
  two June-September versions name only 10-11 kinds of client packet.
* **Read**: of the 252 kinds of packet in the newest version's captures, questkit's code mentions about 66. Unread and
  plentiful: ClientTrigger (25,000), Condition, FirstAttack, the inventory packets, LootMessage, UnkJobGauge,
  UpdateRecastTimes, EventScene2, the map markers.
* **Bob's own character** (the only one whose gauge, MP, recasts and stats a capture has): Arcanist / Summoner,
  White Mage / Conjurer, Gladiator / Paladin and Black Mage in number (130-1,400 gauge packets each), Dark Knight a
  little (60), ten or fewer for eight other jobs, none for the rest. Everyone else's actions, statuses and damage are
  there for every job that was in his parties.
* **Extractors** (`questkit chronofoil`): the index, `--export`, `--battle`, `--quest`, `--dropin`, `--gathering`,
  `--hits`. Quests have the whole loop (fixture, `replay`, `compare`); combat doesn't.
* **What was done by hand in the job stages**, with scripts that lived in a session's scratchpad and are gone with
  it: each job's uses a minute by level band, the statuses each action puts on, the gauge bytes by job, how much of
  the captures the tables name, a rotation simulator running the real action scripts. Their numbers went into
  comments and these pages by hand.

So the weak points, in the order they cost: (1) one-off extraction that isn't kept; (2) facts copied by hand with no
check that the server still agrees; (3) no way to see what the captures *don't* cover, so Bob can't aim a capture;
(4) most packet kinds unread; (5) older versions half-named.

## The pipeline

```text
captures ──▶ ledger ──▶ fact tables ──▶ generated data ──▶ server
 (.cfcap)   (events,    (one query      (timelines,          │
            numbers     each, with      drop-ins,            ▼
            only)       samples and     potencies)      conformance:
                        sources)        + checks on     replay Bob's own
                                        the scripts     requests, diff
                 ▲                                      against retail
                 └────────── coverage map ◀─────────────────┘
                             (what's seen, what isn't: the capture wish list)
```

Rules that keep it stable:

* **One decoder.** Every extractor reads the ledger, never the raw capture; a packet layout is written once
  (`questkit/ledger.py`), with the game versions it holds for.
* **Numbers only.** The ledger and the fact tables hold IDs, amounts, times and positions; actors are `self`, `party`,
  `other`, `pet`, or a BNpcBase, numbered per capture. No names, no chat (CLAUDE.md's rule, enforced by the decoder
  rather than by care).
* **Every fact has its sources**: the captures (by their eight-character ID), the sample count and the spread. A
  fact with fewer than its table's minimum samples is marked `thin` and never overrides a job guide number.
* **Nothing is copied by hand.** A number reaches the server through a generated file or a check that fails when the
  script and the fact disagree.
* **Game files and captures stay out of git**; fact tables (numbers) go in the private diagnostics repository, the
  code and what it found in this one.

## Stages

Each ends with something that runs in CI or in one command, and a line in [questkit and CI](questkit.md).

### A. Keep what exists (small)

1. `questkit chronofoil --coverage`: the table above per version (named, unnamed by size, wrong sizes).
2. `questkit chronofoil --uses`, `--statuses`, `--gauges`: the three scratch scripts as commands (uses a minute by job
   and level band with what follows what; statuses by action; ActorGauge bytes by job).
3. `questkit rotation <classjob> <level> [targets]`: the simulator that runs the real action scripts under a mock
   EffectsBuilder, with a test that every rotation uses every GCD at every band with no script errors. In CI.
4. A capture registry (`notes/captures.json` in the diagnostics repository): each capture's ID, version, date,
   visits, and Bob's job and level in each. `notes/chronofoil-index.md` is made from it.

Done when: a fresh session can reproduce every number in [Combat](combat.md)'s job sections with a command.

### B. The ledger

One pass over a capture into typed events (JSONL, gzipped, cached by the capture's hash and the decoder's version):

| Event | From | Carries |
|---|---|---|
| `request` | the client's ActionRequest | action, target, sequence: the time to its effect is the animation lock and latency |
| `cast`, `cancel` | ActorCast, ActorControl | action, cast time, target, position |
| `effect` | ActionEffect1, AoeEffect8-32 | action, source, each target's effects decoded (damage with its kind and flags, heal, status given, combo, gauge), positions of source and targets |
| `status` | EffectResult, StatusEffectList (diffed), ActorControl gain/lose | status, param, duration, source, on whom |
| `vitals` | UpdateHpMpTp, ActorControl HoT/DoT ticks | HP, MP, barrier |
| `gauge` | ActorGauge | the bytes, with the job |
| `recast` | UpdateRecastTimes, ActorControlSelf 17 | group, seconds |
| `spawn`, `despawn`, `move` | SpawnNpc/Player/Pet, Delete, ActorMove | base, level, HP, owner, position |
| `control` | ActorControl* by category, Condition, FirstAttack, ClientTrigger | category and arguments |
| `director`, `scene`, `quest` | DirectorVars, EventScene*, UpdateQuest... | as `--quest` and `--battle` read them now |
| `item` | the inventory packets, LootMessage | container, slot, item, count |

`--battle`, `--hits`, `--quest` and the commands of stage A move onto it, their output unchanged (their tests say so).
Unnamed opcodes go in as `unknown` with their size, so a later table fills them without another pass.

Done when: the five extractors give byte-identical output from the ledger, and the ledger of all 124 captures builds
in one run.

### C. Fact tables

Each a query over the ledgers, written to `facts/*.json` with samples and sources:

1. **Actions**: cast time, recast (gaps between uses, by level), animation lock, MP cost (the drop after it, own
   character), the statuses it gives with their durations and stacks, how many it hits and its shape (who was hit and
   who wasn't, by position), which actions follow it (combo, proc).
2. **Potency ratios** by job, action and level band: within a job, damage ratios between plain hits are potency
   ratios (the method that fixed the ARR classes at 1-50). Replaces `tierPotency`'s guesses below each job's lowest
   trait; anchors on actions no trait changes.
3. **Statuses**: duration, what refreshing does, tick size, and what they do to damage and healing taken or dealt
   (hits with and without, same source and target).
4. **Gauges**: for Bob's own jobs, each byte against the actions before it: the layout checked rather than assumed,
   and what each action adds or spends.
5. **Barriers**: the shield amounts in `vitals` against the heals that gave them and the hits they took.
6. **Pets**: spawns with an owner, their actions and timing against their owner's.
7. **Enemies** by BNpcBase and territory: level, HP, kit (each action's cast, gap, target choice, shape, amounts by the
   target's role), aggro and leash, what they do at HP thresholds. Feeds the timelines as `--battle` does now.
8. **Rotations** by job and band: uses a minute, what follows what, openers, where abilities are woven. The rotation
   scripts' targets.
9. **Timing**: server ticks, GCD spacing under haste statuses, request-to-effect.
10. **The rest**: loot, gathering, shops, quests (as now).

Done when: each table has a command, a minimum-sample rule, and its `thin` entries listed.

### D. Into the server, checked

1. **Generated, not typed**: timelines, drop-ins and (new) `resources/data/potencies.json`, read by `tierPotency`,
   come from the fact tables; the generator's diff is the review.
2. **`questkit facts check`** (CI): runs every action script under the harness at each level band and compares what
   it does (statuses and durations, gauge changes, hit count, potency ratio to its job's anchor) with the action
   facts. A mismatch fails unless it's listed, with its reason, in `facts/exceptions.yml` (the job guide over a thin
   fact, say).
3. **Conformance replay**: for a visit where Bob fought, his own `request`s go through the scripts with the state the
   ledger has at that moment (level, statuses, gauge, combo), and what the server would send is compared field by
   field with what retail sent: effects, statuses, gauge bytes, MP. One score per job and visit, kept in the diagnostics
   repository; a fix raises it, a regression shows. This is `compare` for combat.
4. **Rotation targets**: `questkit rotation --against facts` puts a bot's uses a minute beside retail's for the same
   job and band, and fails outside a tolerance where the fact isn't thin.

Done when: CI fails on a script that contradicts a fact, and each job Bob has played has a conformance score.

### E. The coverage map and what to capture

A page made from the ledgers ([Research Notes](research.md) links it): job × level band × kind of evidence (own
gauge, own hits, others' actions, statuses, samples), and content × runs. Its holes are the capture wish list, by what
they unblock. As it stands:

| Capture | Unblocks | How |
|---|---|---|
| Each job at a striking dummy, 5 minutes: every action once, then its rotation | That job's gauge bytes, MP costs, recasts, status durations, potency ratios at its level; its conformance score | Bob's own character is the only one with a gauge: eight jobs have ten packets or fewer, most none |
| The same, synced (a low-level duty, or level sync at a FATE) at 15-50 for the jobs after ARR | Low-level potencies (`tierPotency`'s guesses) | Hits against the same target at one level |
| A Scholar, Sage or Astrologian shielding a tank through a pull | Barriers | Shield amounts against hits |
| Summoner, Scholar, Machinist, Dark Knight with their pet out | Pets | Pet spawns and their actions |
| A melee job hitting a dummy from the front, flank and rear | Positionals | Same action, three sides |
| Samurai, Viper, Bard (Army's Paeon), Black Mage (Ley Lines) with and without their haste | Haste | GCD spacing |

### F. When the game is patched

Before merging upstream's patch: `--save-opcodes`, then `--coverage` on the first capture of the new version. Facts
carry their versions; a fact whose newest samples disagree with its older ones is flagged (a potency change), and the
job guide's patch notes decide.

## Order

A (small) → B → C.1-C.4 → D.2 and D.3 for the ten jobs with evidence → E's first page. Then the gameplay stages
take their numbers from it as they come: **healers and barriers** from C.3 and C.5, **pets** from C.6, **positionals
and haste** from E's captures and C.9, **low-level potencies** from C.2. C.7 (enemies) goes on as each duty is done, as
now.

What this doesn't do: it can't see what no capture holds. Other players' gauges, MP and recasts are never sent; for
those the job guide stays the source and the fact tables only check what shows from outside (statuses, damage, order).
