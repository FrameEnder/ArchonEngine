# Items that teach things

Mounts, minions, emotes, hairstyles, orchestrion rolls, facewear, barding, fashion accessories and Triple Triad cards
are all items whose use teaches something. What an item does is its **ItemAction** (Item sheet `ItemAction`, then
ItemAction's `Type` and `Data`). `resources/scripts/items/Items.lua` (`dispatchItem`) decides by Type.

## What each Type teaches

The types are the ones the client's own "already learned" check knows (UIState::IsItemActionUnlocked, `0xC49990`,
whose switch is at `0x868890`), with the ID where the client reads it:

| ItemAction Type | Teaches | ID from | Server keeps it in |
|---|---|---|---|
| 853 | Minion | Data[0] | `unlock.minions` |
| 1013 | Barding (BuddyEquip) | Data[0] | `companion.unlocked_equip` |
| 1322 | Mount | Data[0] (stored by the Mount's `Order`) | `unlock.mounts` |
| 2633, 43141 | An **unlock link**: emotes, hairstyles, and many others | Data[0] | `unlock.unlocks` |
| 3357 | Triple Triad card | the item's AdditionalData | `unlock.triple_triad_cards` |
| 20086 | Fashion accessory (Ornament) | Data[0] | `unlock.ornaments` |
| 25183 | Orchestrion roll | the item's AdditionalData | `unlock.orchestrion_rolls` |
| 37312 | Facewear (a Glasses row; its `Style` is what's unlocked) | the item's AdditionalData | `unlock.glasses_styles` |
| 2136, 4107 | Secret recipe book, folklore tome | Data[0] | not kept yet |

## Using one

`Items.lua` calls `player:learn_from_item(kind, id, item_id)` (LuaTask::LearnFromItem); the connection
(`zone_connection/unlock.rs`, `learn_from_item`) checks it isn't known already, unlocks it with the same actor control
the GM commands use (ToggleMountUnlock, ToggleUnlock, ToggleMinionUnlock…), and **uses up one of the item** from the
bags (`Inventory::use_up_one`), as retail does. If it's already known nothing happens and the item is kept.

Before this, only minions and orchestrion rolls were handled, by toggling (using the item again un-learned it) and
without using the item up; mounts, unlock links, cards, accessories, barding and facewear did nothing.

## Selling to NPCs, wear and repair

* **Sell price** (`ItemRow::sell_price`): PriceLow, and for materia (ItemUICategory 58) a tenth more rounded up in
  integers (PriceLow 90 sells for 99): exact for all 53 sales in Bob's Chronofoil captures. HQ wasn't seen.
* **Wear:** dying takes 600 (2 %) off the condition of everything the player wears (not the soul crystal), as retail
  right after the death (374 items in the captures). Syntheses, gathers and completed content wear and bond gear too:
  [Spiritbond, Condition and Materia Extraction](spiritbond.md).
* **Repair** ([Mender](mender.md); self-repair in [Spiritbond](spiritbond.md)): retail's NPC repair costs `ceil(ItemRepairPrice[equip level] × (30000 − condition) / 30000)`
  per item (7 of 7 repairs exact): client trigger 1602 repairs what's worn, 1601 an armoury page; answered with each
  slot, ActorControlSelf 517 (LogMessage 1388, the number of items), the gil, then the transactions.
