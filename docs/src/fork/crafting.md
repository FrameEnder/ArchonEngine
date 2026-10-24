# Crafting (Disciples of the Hand)

A synthesis from the Crafting Log's *Synthesize* to the finished item or the materials given back, with every 7.x
action, the conditions, buffs as statuses, HQ chance and HQ materials. Unlocking the classes is the quests' business
(Bob's); this is what a crafter has once unlocked.

Code: `servers/world/src/crafting.rs` (the rules of a step, no packets; unit tests), `servers/world/src/events/crafting.rs`
(the event: packets, materials, the item), `GameData::get_recipe_info` (Recipe, RecipeLevelTable,
GathererCrafterLvAdjustTable, the ingredients' Item rows), `ToServer::SetCraftingState` (`server/effect.rs`: CP and
the statuses in one StatusEffectList).

**Sources.** Not Sapphire: its crafting is from before Endwalker (TP, other actions and numbers). The rules are what
today's crafting tools agree on, read in their code: raphael-rs (723b17e, 2026-10-18, the solver Teamcraft and
Craftimizer users compare against), Teamcraft's simulator (74e167a) and Craftimizer (1d4a2c6); the packets and the
CP rules are the 7.57 client's (`ffxiv_dx11.exe` 2026.10.16, its craft event handler), and Teamcraft's packet reader
agrees. **No retail synthesis has been recorded** (Bob's Chronofoil captures have none), so nothing here is checked
against a retail packet yet. The research notes behind this page are in Bob's session scratchpad (`research/crafting.md`).

## The flow

All scenes are scene 0 of handler 0x000A0001 (Craft), flags `NO_DEFAULT_CAMERA` (retail's unknown).

| Step | Client | Server |
|---|---|---|
| Synthesize | ClientTrigger 700 `BeginCraft` (the recipe) | EventStart, scene `[1, 0, item, 0]` (upstream) |
| Materials chosen | yield `[0, NQ×6, HQ×6]` (each slot's NQ and HQ amounts) | works out the recipe for the crafter's level, the start quality from the HQ materials, CP to full; LiveEvent StartCraft; scene `[2, recipe, start quality, start HQ %]` |
| The stacks used | yield 1, 77 results: u32 command, item[30] (HQ = ID + 1,000,000), then u16 amount[30], container[30], slot[30], u32 count | takes them from the bags (crystals keep their slot), checking each against the bag; scene `[3, 0, 0, 0]` opens the Synthesis window |
| An action | yield `[10, id]` for a CraftAction row, `[9, id]` for an Action row (the buffs) | the step (below); LiveEvent PlayAnimation for CraftAction rows (their AnimationStart/End); CP and statuses; the step's scene (EventScene64) |
| Progress full | yield 11 (after the scene's StateFlags 0x4) | the item (HQ by the HQ chance; the crafter's signature), LogMessage 789, EXP, CP full and the statuses off, LiveEvent EndCraft, scene `[4, 0, 0, 0]` (back to the log) |
| Durability gone | yield 12 (StateFlags 0x8) | scene 8 with the materials given back, then as below |
| Quit | yield 7 | the same scene 8 |
| After scene 8 | yield 3 | CP full, statuses off, EndCraft, scene `[4, …]` |
| Leaving the log | yield 7 with no synthesis under way | the event finishes |

The client works the window's maxima out itself (7.57 `0x140EA7960`): progress `Difficulty × DifficultyFactor / 100`,
quality and durability the same with their factors; level-adjusted recipes take the recipe level of
GathererCrafterLvAdjustTable at the crafter's level (up to MaxAdjustableJobLevel) and durability 80 (raphael).

### A step's scene

EventScene64, 33 params (the client reads these; `0x140EA4FB6` and the functions it calls):

| Param | Meaning |
|---|---|
| 0 | 9 after a CraftAction row, 10 after an Action row (the reverse of the client's yields) |
| 1–3, 18 | unknown, 0 |
| 4 | the action's row |
| 5 | CP recovered (Tricks of the Trade; log 1174 and flytext) |
| 6 | the step count (upstream's recorded step had 1 after the first action) |
| 7, 8 | progress (capped at the maximum, as upstream's recording) and its change |
| 9, 10 | quality and its change |
| 11 | HQ chance % |
| 12, 13 | durability and its change (signed) |
| 14 | the next step's condition (1-based: 1 Normal … 11 Robust) |
| 15 | the condition the action was used under |
| 16 | 150 when the next condition is Good (the tooltip's number), else 100 (upstream's recording under Normal) |
| 17 | StateFlags: 0x2 not the first step, 0x4 done, 0x8 failed, 0x10 the action succeeded, 0x08000000 after Basic Touch (Standard Touch costs 18), 0x10000000 after Standard Touch's combo or Observe (Advanced Touch costs 18), 0x40000000 no more Careful Observation, 0x80000000 Heart and Soul used |
| 19–25, 26–32 | up to seven statuses that ran out or were used up this step, and their params (the client logs them wearing off) |

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

* EXP (recipes with ExpRewarded) is a stand-in: GatheringExp for the recipe level's ClassJobLevel, plus as much again
  in proportion to the quality reached. No source has retail's formula.
* A quit or a failed synthesis gives back each ingredient (not the crystals) with an even chance, listed in scene 8
  (params 4–9 the items, 10–15 NQ, 16–21 HQ amounts). Retail's rule isn't known.

## Not done yet

See [Known Issues](known-issues.md#crafting).
