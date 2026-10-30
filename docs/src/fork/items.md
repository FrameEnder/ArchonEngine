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
(`ZoneConnection::open_coffer`) uses the coffer up and gives the set in retail's order (Bob's Chronofoil capture
c99fe374 of 2026-10-28, Dags, a gladiator, opening the Level 18 Weapon Coffer: the Carnage Sword and the Iron Lantern
Shield, the gladiator's set the script has): ActorControlSelf **527** (1232, 0, the item) for each piece, then the
coffer's slot emptied, then for each piece the ack (0xFFFFFFFF, 6), its slot and LogMessage 789 (the "obtained"
pop-up); retail sent no log line of its own (no EventLogMessage 750). It does nothing, keeping the coffer, in
instanced content (the item's description says it can't be opened there), when the class has no set, or when the
bags can't hold the set. A coffer not in the table says so in chat.

Filled in, from the FFXIV Console Games Wiki's pages (every "Level N Weapon Coffer" since 2026-10-30; until then
only the Level 18 and 24 ones were, and the Level 21 one said in chat that its contents weren't known: Bob's bundle
20261030-201929). Each wiki name matched exactly one item in the Item sheet, of the class its place in the list says. The
level 50 coffers have the scholar's book apart from the arcanist's and summoner's (ClassJobCategory ACN SMN, then SCH):

| Coffer | Item | From | Sets |
|---|---|---|---|
| Level 18 Weapon Coffer | 31329 | Life, Materia and Everything | 1611 + 2241 (GLA), 1758, 1827, 1688, 8662, 1897, 2111, 2149, 2002 |
| Level 21 Weapon Coffer | 31330 | Microbrewing | 1614 + 2242 (GLA), 1764, 1833, 1694, 8663, 1899, 2116, 2151, 2010 |
| Level 24 Weapon Coffer | 31331 | Meeting with the Resistance | 1625 + 2246 (GLA), 1766, 1836, 1697, 9193, 1905, 2118, 2156, 2011 |
| Level 28 Weapon Coffer | 31332 | Skeletons in Her Closet | 1627 + 2250 (GLA), 1769, 1838, 1699, 8664, 1908, 2069, 2160, 1967 |
| Level 34 Weapon Coffer | 31333 | You Can't Take It with You | 1637 + 2260 (GLA), 1778, 1847, 1708, 8665, 1917, 2123, 2166, 2023 |
| Level 36 Weapon Coffer | 31334 | Road to Redemption | 1639 + 2262 (GLA), 1781, 1850, 1711, 8666, 1920, 2078, 2169, 2026 |
| Level 40 Weapon Coffer | 31335 | By the Lights of Ishgard | 1650 + 2270 (GLA), 1786, 1855, 1716, 9209, 1925, 2083, 2174, 1974 |
| Level 45 Weapon Coffer | 31336 | Fool Me Twice | 1659 + 2287 (GLA), 1796, 1867, 1728, 8667, 1937, 2094, 2185, 1981 |
| Level 49 Weapon Coffer | 31337 | A Hero in Need | 1663 + 2296 (GLA), 1803, 1872, 1733, 8668, 1942, 2133, 2189, 2044 |
| Level 50 Weapon Coffer (IL 70) | 31691 | It's Possibly a Primal | 1666 + 2302 (GLA), 1806, 1875, 1736, 9225, 1945, 2136, 2193 (ACN SMN), 2048, 2194 (SCH) |
| Level 50 Weapon Coffer (IL 90) | 31696 | When Yugiri Met the Fraternity | 1679 + 2311 (GLA), 1818, 1888, 1748, 9241, 1957, 2141, 2217 (ACN SMN), 2054, 2218 (SCH) |
| Level 50 Weapon Coffer (IL 110) | 31701 | The Reason Roaille | 7351 + 7350 (GLA), 7353, 7354, 7352, 9247, 7355, 7357, 7358 (ACN SMN), 7356, 7359 (SCH) |

The order in each row is the script's: GLA, MRD, LNC, PGL, ROG, ARC, THM, ACN, CNJ (and SCH). The jobs that came
later (DRK, MCH, AST and on) have nothing in them, as the wiki lists them, so the coffer says so and stays.

## Selling to NPCs, wear and repair

* **Sell price** (`ItemRow::sell_price`): PriceLow, and for materia (ItemUICategory 58) a tenth more rounded up in
  integers (PriceLow 90 sells for 99): exact for all 53 sales in Bob's Chronofoil captures. HQ wasn't seen.
* **Wear:** dying takes 600 (2 %) off the condition of everything the player wears (not the soul crystal), as retail
  right after the death (374 items in the captures). Syntheses, gathers and completed content wear and bond gear too:
  [Spiritbond, Condition and Materia Extraction](spiritbond.md).
* **Repair** ([Mender](mender.md); self-repair in [Spiritbond](spiritbond.md)): retail's NPC repair costs `ceil(ItemRepairPrice[equip level] × (30000 − condition) / 30000)`
  per item (7 of 7 repairs exact): client trigger 1602 repairs what's worn, 1601 an armoury page; answered with each
  slot, ActorControlSelf 517 (LogMessage 1388, the number of items), the gil, then the transactions.
