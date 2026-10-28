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

## Casting an item

An item is cast through its ItemAction's own action: ItemAction `Type` is an Action row, whose Cast100ms is the cast
and whose ID the cast bar (ActorCast's spell ID) shows (`server/action.rs`). Retail's Vesper Bay Aetheryte Ticket (item
30362, ItemAction type 9994) cast 9994 for 5 s, then its ActionEffect named the item with animation 9994 (Bob's
Chronofoil capture b8432613, five uses). The server used to look the item's own ID up as an Action row, which is some
unrelated action's.

## Aetheryte tickets

ItemAction type **9994** (`items/TeleportTicket.lua`): after the cast, one ticket is used up (its slot sent again) and the
player teleports (`player:teleport_by_ticket(item, territory, pop range)`): PrepareZoning with warp type **6**
(`WarpType::Ticket`, hidden and faded as a teleport, flags 0x0A), into the territory at the pop range, and the
landing's ZoneIn carries a teleport's 110. Where each ticket goes is retail's landing, by item: the Vesper Bay ticket
went to Western Thanalan (140), pop range 8366917 (ZoneInit's, just outside Vesper Bay's gates) all five times. Other
tickets say in chat that where they go isn't known. Retail also sent an InventoryTransaction for the ticket; Kawari
sends the slot only.

## Gear coffers

Every gear coffer (the "Level N Weapon Coffer"s, the raid and tome coffers, also the ward potions) has ItemAction
**388**, whose action is **4647** (ItemAction 2462 has it too); `items/Items.lua` sends action 4647 to
`items/Coffer.lua`. What a coffer holds isn't in the client: its AdditionalData (1000238 for the Level 18 Weapon
Coffer, 1000239… for the next ones) links to no sheet, so the contents are a table in the script, by coffer: sets of
item IDs, one per class, the first piece deciding (its ClassJobCategory includes the current class or job: a
paladin gets the gladiator's set), a gladiator's sword coming with its shield. `player:open_coffer(coffer, sets)`
(`ZoneConnection::open_coffer`) uses the coffer up and gives the set as an event gives an item (the ack, the
inventory, LogMessage 789's pop-up and EventLogMessage 750 for each piece). It does nothing, keeping the coffer, in
instanced content (the item's description says it can't be opened there), when the class has no set, or when the
bags can't hold the set. A coffer not in the table says so in chat.

Filled in, from the FFXIV Console Games Wiki's pages (no coffer is opened in Bob's captures):

| Coffer | Item | From | Sets |
|---|---|---|---|
| Level 18 Weapon Coffer | 31329 | Life, Materia and Everything | 1611 + 2241 (GLA), 1758, 1827, 1688, 8662, 1897, 2111, 2149, 2002 |
| Level 24 Weapon Coffer | 31331 | Meeting with the Resistance | 1625 + 2246 (GLA), 1766, 1836, 1697, 9193, 1905, 2118, 2156, 2011 |

## Selling to NPCs, wear and repair

* **Sell price** (`ItemRow::sell_price`): PriceLow, and for materia (ItemUICategory 58) a tenth more rounded up in
  integers (PriceLow 90 sells for 99): exact for all 53 sales in Bob's Chronofoil captures. HQ wasn't seen.
* **Wear:** dying takes 600 (2 %) off the condition of everything the player wears (not the soul crystal), as retail
  right after the death (374 items in the captures). Syntheses, gathers and completed content wear and bond gear too:
  [Spiritbond, Condition and Materia Extraction](spiritbond.md).
* **Repair** ([Mender](mender.md); self-repair in [Spiritbond](spiritbond.md)): retail's NPC repair costs `ceil(ItemRepairPrice[equip level] × (30000 − condition) / 30000)`
  per item (7 of 7 repairs exact): client trigger 1602 repairs what's worn, 1601 an armoury page; answered with each
  slot, ActorControlSelf 517 (LogMessage 1388, the number of items), the gil, then the transactions.
