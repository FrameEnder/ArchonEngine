# Crafting (Disciples of the Hand)

A synthesis from the Crafting Log's *Synthesize* to the finished item or the materials lost, with every 7.x action,
the conditions, buffs as statuses, HQ chance and HQ materials, *Synthesize again* and quick synthesis. Unlocking the classes is the quests' business
(Bob's); this is what a crafter has once unlocked.

Code: `servers/world/src/crafting.rs` (the rules of a step, no packets; unit tests), `servers/world/src/events/crafting.rs`
(the event: packets, materials, the item), `GameData::get_recipe_info` (Recipe, RecipeLevelTable,
GathererCrafterLvAdjustTable, the ingredients' Item rows), `ToServer::SetCraftingState` (`server/effect.rs`: CP and
the statuses in one StatusEffectList).

**Sources.** Not Sapphire: its crafting is from before Endwalker (TP, other actions and numbers). The rules are what
today's crafting tools agree on, read in their code: raphael-rs (723b17e, 2026-10-18, the solver Teamcraft and
Craftimizer users compare against), Teamcraft's simulator (74e167a) and Craftimizer (1d4a2c6); the packets and the
CP rules are the 7.57 client's (`ffxiv_dx11.exe` 2026.10.16, its craft event handler), and Teamcraft's packet reader
agrees. **Bob's Chronofoil capture of 2026-10-24** (a level 71–72 Carpenter: syntheses of recipes 3812 and 31854,
*Synthesize again*, a quick synthesis, a failure and quits) settled the packets, their order and the numbers: every
gain, CP cost and HQ % of its steps came out of the rules below exactly. The research notes behind this page are in
Bob's session scratchpad (`research/crafting.md`).

## The flow

All scenes are scene 0 of handler 0x000A0001 (Craft), flags `NO_DEFAULT_CAMERA`. "LiveEvent n" is ActorControl
LiveEvent (category 300) with that event number. As retail sent it:

| Step | Client | Server |
|---|---|---|
| Synthesize | ClientTrigger 700 `BeginCraft` (the recipe) | EventStart; conditions 15 and 73 (`Synthesizing`) set, 16 off; SetMode 5; LiveEvent 38 and 39 (the main and off hand models, low and high halves); LiveEvent 8 StartCraft `[0, 136153, 1]` (0 from the log, 3 for *Synthesize again*, 4 for a quick synthesis; 136153 was the same every time, meaning unknown); scene `[1, 0, item, 0]`; StatusEffectList (CP full) |
| Materials chosen | **EventReturnHandler16** `[0, NQ×6, HQ×6]` (a scene return, not a yield) | the recipe for the crafter's level, the start quality; scene `[2, recipe, start quality, start HQ %]` (`[12, recipe, how many, 0]` for a quick synthesis) |
| The stacks used | **EventReturnHandler128** (opcode 866, 77 results): `[1, item[30] (HQ = ID + 1,000,000), amount[30], container[30], slot[30]` (u16 pairs)`, count]`; crystals' container is 255 | kept, not taken yet; LiveEvent 10 `[2]` (the condition's look: the condition + 1); CraftingActionPreviews; scene `[3, 0, 0, 0]` opens the Synthesis window |
| An action | yield `[10, id]` for a CraftAction row, `[9, id]` for an Action row (the buffs; these also come as an ActionRequest) | LiveEvent 12 PlayAnimation (CraftAction rows); retail's ~2 s later: ActorControl GainEffect for each status gained or changed (`[2191, 6]` Muscle Memory, `[251, n]` Inner Quiet); LiveEvent 10 `[condition + 1]` when the condition changed; StatusEffectList (CP already taken); CraftingActionPreviews; the step's scene (EventScene64) |
| Progress full | yield 11 (after StateFlags 0x4) | the materials taken; the item (HQ flag, the crafter's signature); LiveEvent 15 (NQ) or 16 (HQ, then 17); EventLogMessage 1158 and 1156 with the item (+1,000,000 if HQ); EXP; then (retail 1.5–3 s later) back to the log: condition 16 on, 15 and 73 off, LiveEvent 11, scene `[4, 0, 0, 0]`, StatusEffectList (CP full, no statuses) |
| Durability gone | yield 12 (StateFlags 0x8) | LiveEvent 20; **every material lost**; EventLogMessage 1177 and 1160 (no params); scene 8 with 22 params, all 0 (nothing back) |
| After scene 8 | yield 3 | back to the log, as above |
| Synthesize again | yield `[4, recipe, 0, 0]` | the Synthesize row's sequence with StartCraft 3 |
| Quick synthesis | yield `[5, recipe, how many, ?]`, then the return and the list (its command 13) | StartCraft 4 …; after the list: log 1150 with the item, LiveEvent 10 `[2]`, LiveEvent 21; per item (retail: one every ~2 s) log 1158, the item, LiveEvent 18 `[item, made so far]`, EXP; back to the log (retail after the client's ClientTrigger 712) |
| Quit | yield 7 (from the log, or the synthesis' Quit) | scene `[6, 0, 0, 0]` |
| After scene 6 | yield 8 | the materials lost if a synthesis was under way; StatusEffectList; conditions off; LiveEvent 38/39, 40 and 11; EventFinish (the whole log closes) |

The client works the window's maxima out itself (7.57 `0x140EA7960`): progress `Difficulty × DifficultyFactor / 100`,
quality and durability the same with their factors; level-adjusted recipes take the recipe level of
GathererCrafterLvAdjustTable at the crafter's level (up to MaxAdjustableJobLevel) and durability 80 (raphael).

### CraftingActionPreviews

Opcode 347 (fork, 424 bytes; its handler found in the 10.16 client), before the window opens and with every step: 21
entries of `{u32 action, u16 progress efficiency, u16 quality efficiency, u32 progress, u32 quality, u32 state}`, then
4 bytes. The client shows them in the actions' tooltips. Order: Basic Synthesis, Rapid Synthesis, Muscle Memory,
Careful Synthesis, Groundwork, Delicate Synthesis, Intensive Synthesis, Prudent Synthesis (progress), then Basic
Touch, Hasty Touch, Standard Touch, Byregot's Blessing, Precise Touch, Prudent Touch, Advanced Touch, Reflect,
Preparatory Touch, Delicate Synthesis, Trained Eye, Trained Finesse, Refined Touch (quality), each as the crafter's
class' own CraftAction row (`GameData::craft_action_for_class`). Efficiency is the potency with 0x8000 when it differs
from the action's usual one (a buff, a level trait) and 0x4000 for a halved Groundwork; progress and quality what the
action would add now; state 0 usable, 1 level too low, 2 not enough CP, 3 not usable now (`Craft::previews`).

### A step's scene

EventScene64, 33 params (the client reads these; `0x140EA4FB6` and the functions it calls):

| Param | Meaning |
|---|---|
| 0 | 9 after a CraftAction row, 10 after an Action row (the reverse of the client's yields) |
| 1–3, 18 | unknown, 0 |
| 4 | the action's row |
| 5 | CP recovered (Tricks of the Trade; log 1174 and flytext) |
| 6 | the step count + 1, not counted up any more once the synthesis is over (`Craft::display_step`; retail) |
| 7, 8 | progress, capped at the maximum, and its change (not capped) |
| 9, 10 | quality and its change |
| 11 | HQ chance % |
| 12, 13 | durability and its change (signed) |
| 14 | the next step's condition (1-based: 1 Normal … 11 Robust); 1 once the synthesis is over |
| 15 | the condition the action was used under |
| 16 | 150 when the next condition is Good, else 100 (retail) |
| 17 | StateFlags: 0x2 not the first step, 0x4 done, 0x8 failed, 0x10 the action succeeded, 0x08000000 after Basic Touch (Standard Touch costs 18), 0x10000000 after Standard Touch's combo or Observe (Advanced Touch costs 18), 0x40000000 no more Careful Observation, 0x80000000 Heart and Soul used |
| 19–25, 26–32 | up to seven statuses that ran out or were used up this step, and their params: Inner Quiet's stacks, 0 for the rest (retail; the client logs them wearing off) |

### CP

UpdateHpMpTp carries no CP: its handler (7.57 `0x140B383A0`) writes HP, MP and, from its last field, GP; only
StatusEffectList fills CP (`0x140B38500` picks the resource by ClassJobCategory: 32 GP, 33 CP, else MP). So a crafter's
resource is their CP (`common::max_resource_points`, 180 plus gear: Craftimizer's base), every change is sent as their
StatusEffectList (`update_actor_hp_mp` sends that for crafters), it doesn't come back during a synthesis and fills up
when one ends (retail: "CP regenerates fully after ending a craft").

## The rules of a step

Order (Craftimizer's `BaseAction.Use`, raphael's `use_action`): roll success (+25 under Centered), apply the action
with the buffs as they were, take CP and durability (even on failure), then if the action takes a step: Manipulation
mends 5 (not on its own step, nor if the synthesis just ended), the step count goes up, the condition changes, and the
timed buffs tick. Buffs are added with one step more so they last their full duration; Primed adds two.

* **Base gains** (all three tools): progress `craftsmanship × 10 / ProgressDivider + 2`, quality
  `control × 10 / QualityDivider + 35`, times ProgressModifier / QualityModifier % when the crafter's level is at or
  below the recipe level's ClassJobLevel. The tools do this in floats and disagree by one at some stats; Kawari uses
  exact integers, `(stat × 10 × mod + add × div × mod) / (div × 100)`, which only Teamcraft matched everywhere.
* **Progress**: `base × potency × condition × buff / 10⁶`, buff 100 (+100 Muscle Memory, used up; +50 Veneration),
  condition 150 under Malleable. Final Appraisal stops it one short once.
* **Quality**: `base × potency × condition × (100 + 10 × Inner Quiet) × buff / 10⁸`, buff 100 (+100 Great Strides,
  used up; +50 Innovation), condition Poor 50, Good 150, Excellent 400; Inner Quiet +1 (to 10) from level 11 on each
  quality gain.
* **Durability**: halved under Waste Not (I or II) and again under Sturdy or Robust, rounding up once; nothing after
  Trained Perfection, which the next action that would cost durability uses up.
* **CP**: the client's own rule (`0x140EA4440`): Standard Touch and Advanced Touch cost 18 in their combos; Pliant
  halves, rounding up.
* **HQ chance**: the 101-entry table the three tools share, by whole percent of the maximum quality (the client only
  shows the number). 0 for recipes that can't be HQ.
* **Start quality**: `max quality × MaterialQualityFactor × (HQ ingredients' item levels) / (all HQ-able
  ingredients' item levels) / 100` (raphael).

Every action (each class's row is mapped to the same action: Teamcraft's ID lists, checked against the CraftAction
sheet's CRP column for all 433 rows): Basic Synthesis (120 from 31), Basic Touch, Master's Mend, Standard Touch,
Observe, Hasty Touch (Expedience from 96), Rapid Synthesis (500 from 63), Tricks of the Trade, Precise Touch, Muscle
Memory, Careful Synthesis (180 from 82), Prudent Touch, Advanced Touch, Reflect, Preparatory Touch, Groundwork (360
from 86, halved when durability is short), Delicate Synthesis (150 progress from 94), Intensive Synthesis, Trained
Eye, Byregot's Blessing, Heart and Soul, Careful Observation (three), Prudent Synthesis, Trained Finesse, Refined Touch
(extra Inner Quiet after Basic Touch), Daring Touch, Quick Innovation, Immaculate Mend, Trained Perfection, and the
Action-sheet buffs Waste Not I/II, Veneration, Great Strides, Innovation, Final Appraisal, Manipulation. Costs and
levels match the sheets. An action the step can't use (no CP, not the first step, no Good condition…) gets scene 7,
which clears the client's "executing" flag.

**Conditions** (only those the recipe level's ConditionsFlag allows): Good 15% at 63+ (18% below, 12% on expert
recipes), Excellent 4%, Centered, Sturdy, Primed 15%, Pliant, Robust 10%, Malleable 13%, Good Omen 12%; after
Excellent always Poor, after Good Omen Good, after Robust Sturdy, after Good or Poor Normal. These are Craftimizer's;
Teamcraft's differ and retail's aren't known.

**Statuses**: Inner Quiet 251 (stacks), Waste Not 252, Waste Not II 257, Great Strides 254, Innovation 2189,
Veneration 2226, Final Appraisal 2190, Muscle Memory 2191, Manipulation 1164, Heart and Soul 2665, Expedience 3812,
Trained Perfection 3813 (Craftimizer's IDs), each with its steps left as its param and no time limit.

## EXP and materials

* EXP (recipes with ExpRewarded) is a stand-in fitted to the capture: 0.72356% of ParamGrow's ExpToNext at the recipe
  level's ClassJobLevel, times `1 + 2.125 × quality / max quality`, less 4.1% a level for a recipe more than five
  levels below the crafter (at least 5%). Retail gave 21,837 before rested EXP for a level 71 recipe at quality 0 and
  68,250 at full quality, and a level 50 recipe at level 72 gave 923; the full-quality synthesis also brought a second
  message of 102,372 (+350%), likely a first-time bonus, which isn't done.
* The materials are taken when the synthesis ends (done, failed or quit), not when they're listed (retail). A failure
  or a quit loses them all: retail's scene 8 listed nothing given back.

## Not done yet

See [Known Issues](known-issues.md#crafting).
