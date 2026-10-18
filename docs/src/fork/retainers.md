# Retainers

A character can have up to ten retainers. At a summoning bell they can call one, see what they hold, entrust and
withdraw items, put items up for sale, and send them away. Retainers are hired with a debug command for now: hiring
them at a retainer vocate needs the retainer appearance editor, which isn't there yet.

Everything below was read from Bob's Chronofoil captures of retail on 2026-09-15 (captures 26 and 27: two retainers,
eleven visits to summoning bells, entrusting, withdrawing and twelve items put up for sale). The opcodes of that
version were matched to FFXIVOpcodes' names for it (7.56h); the 2026.10.06 opcodes of the same names are
FFXIVOpcodes' (7.57) and haven't been seen working on Kawari yet. See [Known Issues](known-issues.md#retainers).

* Server: `servers/world/src/retainer.rs` (a retainer, their inventory, their NPC), `zone_connection/retainer.rs`
  (the bell), `database/retainer.rs` and the `retainer` table (migration `update-2026-10-18_retainers`).
* The bell's event: `events/custom/005/CmnDefRetainerBell_00544.lua`. The command: `commands/debug/Retainer.lua`.
* Packets: `core/src/ipc/zone/server/retainer.rs`, `RetainerInfo`/`RetainerInfoEnd` (upstream's, now with their
  fields named), client trigger 9003 (`OpenUnk1`), and fork opcodes ([Fork Opcodes](opcodes.md)).

## Hiring

```
!retainer hire <name> [class/job ID] [level]   hire one (in the first free slot)
!retainer list                                 what each holds
!retainer release <name>                       let one go, with everything they hold
```

A name is one word of 3 to 20 letters (apostrophes and hyphens allowed) starting with a capital. The retainer looks
and sounds like the character (their CustomizeData and voice are copied) and belongs to the character's city; class
0 and level 1 unless given. The table keeps the appearance as the 26 bytes of CustomizeData in hex (`customize`) and
the items as JSON (`inventory`), so Kawari Web can change both later.

## The bell

| Step | Retail | Kawari |
|---|---|---|
| Talk to the bell | EventStart (CustomTalk handler `0x000B0220`, CmnDefRetainerBell 544), scene 0 with flags `0x2000` | Same; scene **1001** ("You have not yet hired a retainer.") without retainers |
| The list | Client trigger 9003 command 2 → ten RetainerInfo (empty slots only have their index) and RetainerInfoEnd, all with the inventory sequence | Same, from the table |
| Choose one | Scene 0 returns their ID as two numbers, high half first | `player:summon_retainer(high, low)` |
| Call them | RetainerState (state 4), their containers, ItemMarketBoardInfo per listing, ItemMarketBoardSummary, then scene 2 with `{4, retainer actor ID, 3, 0}` and their NPC (SpawnNpc, object kind 10) | Same (the containers once, retail sent them two or three times; no Equip packet, the spawn has the models) |
| Their menu | Client trigger 9003 command 4 → ActorControlSelf 925 (ServerRequestReply) with command 4, result 0 | Same |
| Entrust, withdraw, move | ItemOperation Move/Exchange (0x203/0x204 on 2026.09.15) between the bags and containers 10000–10006 → InventoryActionAck only | Same; the retainer's items are saved after each one |
| Put up for sale | ItemOperation type 0x215 (bags) or 0x216 (crystals) into container 12002, the asking price per item in the destination item field → InventoryActionAck | Same (`RetainerMarketList`, `RetainerMarketListCrystal`) |
| Open the sale list | The client sends 580 (8 bytes) → PlayerRetainerInfo (480 bytes) | `RequestPlayerRetainerInfo` (opcode not known yet) |
| Sale history | The client sends 243 (the retainer's ID) → RetainerSaleHistory (1048 bytes) | `RequestRetainerSaleHistory` (opcode not known yet), answered empty |
| Close the menu | Scene 2 returns nothing → RetainerState (state 3), scene 1 with `{2, ID high, ID low}`: the list again, which returns like scene 0 | `player:dismiss_retainer()`; the NPC goes away |
| Pick another | 9003 command 3 with the ID → that retainer's RetainerInfo and RetainerInfoEnd (count 1) | Same |
| Leave | Scene 1 returns nothing → EventFinish | Same |

A new character on 2026.10.06 got scene **1000** ("You have not yet received approval to hire retainers."); Kawari
doesn't check the quest that gives approval.

The retainer's NPC: SpawnNpc with object kind 10, no base or name ID, their name, CustomizeData, voice, class, level
and gear models, 0x40000 display flags, 400 MP (retail's HP were 721 and 1,124; Kawari sends 1,000). Retail placed them
at a spot set for each bell (the spawn's layout ID), 1.6 to 2.8 yalms from where the player stood and roughly facing
them; Kawari puts them 1.8 yalms in front of the player, facing them. Everyone nearby sees them. They don't wander
and nothing attacks them (`npc_behavior`, `find_possible_enemies`). A retainer still out when the player logs out
goes home.

While a retainer is out their containers are `Inventory::retainer`, so the usual item operations work on them; the
player's own inventory save leaves them out.

## Packets

| Packet | 2026.09.15 | 2026.10.06 | Size | Layout |
|---|---|---|---|---|
| RetainerInfo (FFXIVOpcodes' RetainerInformation) | 757 | 466 (upstream) | 80 | sequence, u32, retainer ID u64, slot u8, item count u8, 2 empty, gil u32, market item count u8, town u8 (1 Limsa Lominsa, 2 Gridania, 3 Ul'dah), class/job u8, level u8, market listings' end (Unix time) u32, venture u32, venture done u32, available u32 (1), u8, name (32), 3 empty |
| RetainerInfoEnd (FFXIVOpcodes' RetainerSummary) | 786 | 120 (upstream) | 24 | sequence, entries u8 (10 or 1), retainers allowed u8 (2 on retail; Kawari sends 10), u8 1, empty, u32 0, `FFFFFFFF`, `FFFFFFFF`, `FFFF` |
| RetainerState | 908 | 249 | 56 | retainer ID u64, u32, flags u32 (top byte `0x01`/`0x81` for one with items on the market), u32, state u8 (4 called, 3 sent away), name (32), 3 empty |
| PlayerRetainerInfo | 599 | 712 | 480 | ten of: retainer ID u64, u32 (looks like a Unix time, the same for both), u8 1, u8 0/1, u8 5, name (32), 1 empty |
| ItemMarketBoardInfo | 313 | 387 | 32 | sequence, container u32 (12002), slot u16, 2 empty, u32, price per item u32, 12 empty |
| ItemMarketBoardSummary | 442 | 303 | 8 | sequence, how many items on the market |
| RetainerSaleHistory | 161 | 832 | 1048 | retainer ID u64, then the sales (not read yet) |
| (unknown) | 897 | ? | 8 | All zeros (or `00000000 01000000` once), before each RetainerState and after the containers; not sent by Kawari |
| RequestPlayerRetainerInfo (client) | 580 | ? | 8 | All zeros |
| RequestRetainerSaleHistory (client) | 243 | ? | 8 | Retainer ID |

The containers: pages 10000–10006 (25 slots each, 175 in all), 11000 what they wear, 12000 gil and 12001 crystals
(both sent as CurrencyCrystalInfo, like the player's), 12002 the market (20 slots). They're sent in that order except
the market comes before what they wear. The ItemMarketBoardInfos all have one new sequence after the containers, and
the summary has it too.

Checked by tests: `retainer_info_layout`, `retainer_state_layout`, `item_market_board_info_layout`,
`read_retainer_entry_request` (core); `hire_list_and_release`, `entrust_and_withdraw`, `list_on_retainer_market`
(world). The bell script was run against the retail flow with lupa (scene 0 → 2 → 1 → 2 → 1 → finish).
