# Mender (Merchant & Mender)

Repairing gear at a Merchant & Mender: one item, everything worn, or a group of the armoury chest. From Bob's
Chronofoil capture of 2026-10-24 (Middle La Noscea).

Code: `servers/world/src/mender.rs` (the cost, the groups; unit tests), `servers/world/src/zone_connection/mender.rs`
(the packets), `GameData::repair_price`, `resources/scripts/events/custom/000/CmnDefNpcRepair_00019.lua` (the NPC).

## The flow

Talking to the mender (CustomTalk 19) plays scene 0 (flags 0x2000); the repair window's buttons send ClientTriggers
while the scene is open, and its close returns the scene (the event finishes).

| Client | Server |
|---|---|
| ClientTrigger **1600** `[container, slot, item]` (one item) | the item's slot (condition 30,000), the gil's slot, LogMessage **1385** `[item]`, PlayerStats, ActorControlSelf **90** `[container]`, **91** |
| ClientTrigger **1602** `[1000]` (everything worn) | each repaired slot, LogMessage **1388** `[how many]`, the gil's slot, an Update InventoryTransaction for each item and the gil (one sequence), InventoryTransactionFinish **0xF6** (`ItemOperationKind::Repair`; second word `(transactions + 1) << 8`), PlayerStats, ActorControlSelf 91, **93** `[1]` |
| ClientTrigger **1601** `[group]` (an armoury chest group) | the same without PlayerStats |

Groups: 0 main and off hand, 1 head, body, hands, 2 waist, legs, feet, 3 earrings and necklaces, 4 bracelets and rings.
Only items below full condition that can be repaired (Item.ItemRepair) count; Kawari repairs nothing when the gil
doesn't cover the lot (retail's answer then isn't recorded).

## The cost

Each item costs its **ItemRepairPrice** row at the item's equip level (Item.LevelEquip; 10 up to level 10, …, 700 for
81–90, 800 for 91–100) times the share of its condition missing, rounded up per item. Retail: the ten pieces Bob wore
(equip level 82/84) at 20,051–23,571 condition cost 2,059 gil, exactly that sum (`mender::tests::retail_costs`).
Workspace icarus feature `ItemRepairPrice`.

## Not done

Self-repair (Dark Matter) and materia extraction at the mender's other tabs weren't in the capture.
