# Spiritbond, Condition and Materia Extraction

What wearing gear does to it: condition (durability) lost and spiritbond gained after syntheses, gathers, deaths and
completed content; materia extraction from 100% spiritbound gear; self-repair with Dark Matter. Menders are their own
page ([Mender](mender.md)).

Code: `servers/world/src/spiritbond.rs` (the rules and tables; unit tests), `servers/world/src/zone_connection/spiritbond.rs`
(`use_gear`, `wear_gear_for_content`, `wear_gear_for_fate`, `begin_materialize`, `finish_materialize`),
`servers/world/src/events/materialize.rs` (the Materialize event), the death wear in `zone_connection/item.rs`
(`wear_gear_on_death`). Hooks: a synthesis' end (`events/crafting.rs`), a gather (`finish_gather`), `FromServer::DutyCleared`
and `FromServer::FateReward` (`main.rs`). Workspace icarus features `Materia`, `ItemRepairResource`; `ItemRow` gained
`equip_level`, `materialize_type`, `item_repair`, `class_job_repair`.

## What retail sends

Every change to worn gear came as one **UpdateInventorySlot** per changed piece of the equipped container (1000), in a
batch at the event, never between: right after a death, after each synthesis and each gather, and at a duty's or
FATE's completion (next to UnkContentFinder2). A piece reaching 100% was followed by **ActorControlSelf 517**
(LogMessage) `[744, item]` (Bob's Chronofoil capture of 2026-10-24, two pieces at a gather). The ItemInfo fields are
condition u16 at 34 (0–30,000) and spiritbond u16 at 36 (0–10,000).

Kawari sends the same: the changed slots (`send_inventory_slot`), LogMessage 744 after a piece's slot when it reaches
100%, and the stats again when a piece drops to 0%.

## Condition

From every batch in Bob's captures (June to October 2026):

| What | Weapons and armour | Body | Accessories |
|---|---|---|---|
| Death | 600 | 600 | 600 |
| A synthesis (done or failed) | 150 ±5% (142–157) | 100 | 200 |
| A gathering attempt (normal node) | 3 | 2 | 5 |
| Completed content, per unit, above its level (synced, or unsynced above) | 55 | 37 | 75 |
| Completed content, per unit, at its own level | 145 | 98 | 196 |

Units: trials, raid turns and FATEs 1; treasure dungeons 2; dungeons about 10; alliance raids about 14 (Duty Support
runs at their own level took 10–12 units of 150–170). The total for content is spread ±5%. The soul crystal doesn't
wear. Sastasha once took 300 at level 17 from a death (700cc294), unexplained. Unspoiled Botanist nodes took 5–6 from
armour; Kawari uses the normal node's 3.

**At 0%** an item gives no stats (the wiki's Gear Durability page): `equipped_item_stats` skips it, and it gains no
spiritbond.

## Spiritbond

The wiki's formula (The Balance, patch 7.0), exact for every synthesis and gather in the captures:

```
Min = floor((Base + SBGain/2) × (1 + 0.25 × (materia melded + 1 if HQ)))
Max = floor(Min × 1.1); points = uniform(Min, Max); 10,000 = 100%
```

* **Crafting**: the difference is the gear's item level minus the item made's (0 or less: 0). Base 33 at 0, one less
  per level to 20 at 13, then 15 (14), 10 (15–20), 5 (21–29), 3 (30–39), 1 (40–50), 0 (51+). Retail: a level 71–72
  Carpenter's HQ armour gained 41–45 a synthesis, its NQ weapon 34–35.
* **Gathering**: the gear's equip level minus the node's (Kawari: the gathering item's level). Unspoiled/legendary
  nodes 50, 49 (1–4), 48 (5–9), 47 (10–14), 46 (15–19), 0 (20+); normal nodes a tenth (5 down to 1). Spiritbond Gain
  counts up to 2 at a normal node, 5 at a legendary one. Nothing for an attempt that gathered nothing. Retail: lv 58
  gear (equip 50) at level 40 nodes +2/+3; Botanist unspoiled +50..54, normal +5. Kawari treats GatheringPointType 2
  as unspoiled.
* **Combat**: only at a duty's or FATE's completion (not overworld kills since 5.3; a failed duty gives none except
  treasure dungeons and FATEs). The formula isn't known; `content_base` is a stand-in fitted to the captures: low-level
  dungeons about 2,650 an item at their level (the same with 67 or 112 kills), Mt. Gulg's weapon 1,782, a ring of
  equip level 50 in a level 19 duty 1,410; trials 396 and 101 (Kawari 250), raid turns about 30 (HQ accessories),
  a treasure dungeon 56. Alliance raids use the dungeon numbers and FATEs the treasure dungeon's (not seen below 100%).
* Only gear that can be extracted from (Item.MaterializeType ≠ 0) bonds; gear at 0% doesn't.

## Materia extraction

Unlocked by **Forging the Spirit** (Quest 66174). Any 100% piece with a MaterializeType.

**The request** is a **ClientTrigger 2800** `[handler, container, slot, item]` with handler **0x390001** (the Materialize
handler type 57; desynthesis is 0x390000, aetherial reduction 0x390002). Found in the client (2026.10.16):
EventFramework.MaterializeItem → `0x140B804E0` → `0x141A1B1F0` (starts the event locally, type 0x19, and stores the
item at handler +0x1B8) → the handler's vfunc +0x8C8, `0x141A1B5C0`, which sends ClientTrigger 2800 through
`0x140B35860` (the ClientTrigger sender, opcode 893). Bob's captures have eleven desyntheses (2026-09-15) in that form,
none of extraction.

**Retail's desynthesis flow**, which Kawari follows for extraction and self-repair:

| Client | Server |
|---|---|
| ClientTrigger 2800 `[0x390000, 0, 3, 8345]` | ActorControl 0 `[0, 1, 1]` (weapon away), **EventStart** (the player, handler, type **0x19**, flags 0), Condition **14**, ActorControlSelf **138** `[handler]`, **EventScene2** scene 0, flags **0x40000**, no params |
| ActionRequest (action 4582, the animation) | ActionEffect |
| EventYieldHandler2 (scene 0, yield 0) | the item's slot emptied, InventoryActionAck `[-1, 6]`, the results' slots, ActorControl 148, ActorControlSelf 2400 `[8, 2601]`, two result packets (opcodes 613 and 251 on 2026.09.15: `[handler, 0x0A01, item, result item, qty, result item, qty, …]`) |
| EventReturnHandler2 | **EventFinish** (handler, 0x19, result 1) |

Kawari answers the extraction's yield with the piece's slot (spiritbond 0), InventoryActionAck `[-1, 6]`, the
inventory, and EventLogMessage **1200** `[materia, item]`. The weapon toggle, ActorControl 148, 2400 and the result
packets aren't sent (their 2026.10.16 opcodes and the extraction's contents aren't known).

**The materia**:

* Its stat, at random from the gear's role (the wiki): War — Tenacity, Skill Speed, Direct Hit, Critical Hit,
  Determination; Magic — Piety, Spell Speed, Direct Hit, Critical Hit, Determination; Hand — Craftsmanship, Control,
  CP; Land — Gathering, Perception, GP. The role comes from Item.MaterializeType (seen in the Item sheet): 2–6 war
  (3 tank, 4 maiming, 5 striking, 6 aiming/scouting), 8–9 magic (casting, healing), 11 gatherers', 12 crafters'; 1, 7
  and 10 are gear for every class (Weathered, Hempen…), which Kawari gives the role of the class the player is on.
  The client's MateriaParam sheet has 14 flags a stat that may be by type, but its columns don't line up with the types.
* Its grade, from the wiki's Materia page: I to equip level 14; 90% the lower and 10% the next grade at 15–29 (I/II),
  30–44 (II/III), 45–50 (III/IV; IV 75% at item level 60–130 and always above); V 51–60, VI 61–70, VII 71–80, IX
  81–90, XI 91–100. The item is the Materia sheet's row for the stat, Item[grade].
* Spiritbond goes back to 0 (since 5.2 the gear is kept; the wiki's Materia Extraction page says 1%, unchecked).

Refused (an EventFinish, and LogMessage 4160 when the bags are full): the quest not done, the item not where the client
says, below 100%, MaterializeType 0.

## Self-repair (Dark Matter)

The Repair window outside a mender starts handler **0x390003** (the client's Repair agent, `0x14088C77D`: a mender's
window sends ClientTrigger 1600 instead). Its handler's vfunc +0x8C8 (`0x141C53F80`) sends ClientTrigger 2800
`[0x390003, (slot << 16) | 1, container, item]` for one item, `[0x390003, 2, container, 0]` for a container's "repair
all" and `[0x390003, 3, ?, 0]` for a third (not done). Not seen on retail; Kawari plays it like the desynthesis.

Rules (the wiki's Gear Durability page and the Item sheet): Item.ClassJobRepair's class at least the equip level less
10; one Dark Matter of Item.ItemRepair's grade (ItemRepairResource row) or higher, the lowest the player has, per item;
only below 100%; back to 30,000. Answered with each slot, LogMessage 1385 `[item]` (one) or 1388 `[how many]`, the
inventory and the stats. Refusals: 1386 (can't be repaired), 1390 (level), 1392 (no Dark Matter), 1394 (not needed).

## Not done or unknown

* The extraction's own packets on retail (and the 2026.10.16 result packets): nothing recorded.
* Combat spiritbond's real formula; alliance raids, FATEs, quest battles and guildhests not seen below 100%.
* Quick synthesis doesn't wear the gear per item made.
* Failed duties and FATEs (treasure dungeons and FATEs should still wear and bond).
* Fishing (the wiki has a table: rod 24 at difference 0, spearfishing 4).
* Reduced Durability Loss and the daily extraction fatigue message (1261).

## What Bob could capture

* A materia extraction (Materia window, one item) and a self-repair of one item and of "repair all", with Chronofoil:
  the packets after ClientTrigger 2800 decide the flow, and the result packets' layout.
* A dungeon, a trial and an alliance raid with gear below 100% spiritbond at their level and synced, and a FATE: the
  combat spiritbond formula.
