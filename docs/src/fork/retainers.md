# Retainers

A character can have up to ten retainers. They're hired (and let go) at a retainer vocate, or with a debug command. At
a summoning bell the character can call one, see what they hold, entrust and withdraw items and gil, put items up for
sale and change their prices, and send them away.

Read from Bob's Chronofoil captures of retail: 2026-09-15 (captures 26 and 27: two retainers, eleven bell visits,
entrusting, withdrawing, twelve items put up for sale), and **2026-10-18 on 2026.10.06** (hiring a retainer at the
vocate, five bell visits with entrusting and withdrawing from the menu, gil, a listing and a price change, letting the
retainer go), which confirmed every 2026.10.06 opcode below. See [Known Issues](known-issues.md#retainers).

* Server: `servers/world/src/retainer.rs` (a retainer, their inventory, their NPC), `zone_connection/retainer.rs`
  (the bell), `database/retainer.rs` and the `retainer` table (migration `update-2026-10-18_retainers`).
* The bell's event: `events/custom/005/CmnDefRetainerBell_00544.lua`; the vocate's:
  `events/custom/000/CmnDefRetainerDesk_00009.lua`. The command: `commands/debug/Retainer.lua`.
* Packets: `core/src/ipc/zone/server/retainer.rs`, `RetainerInfo`/`RetainerInfoEnd` (upstream's, now with their
  fields named), client trigger 9003 (`OpenUnk1`), and fork opcodes ([Fork Opcodes](opcodes.md)).

## Hiring

At a **retainer vocate** (CustomTalk `0x000B0009`, CmnDefRetainerDesk 9), as retail did it:

| Step | Retail (2026.10.06) | Kawari |
|---|---|---|
| Talk | EventStart (condition 7), scene 4 (`0x2000`), then scene 0, the menu, which yields 37 (answered 0) and returns the choice | Same |
| A choice | The retainer list, with the choice in RetainerInfoEnd's u32 at offset 8 (and 0 at offset 6, where the bell has 1), then the choice's scene: 1 hire → scene 1, 3 let go → scene 2 (5 → 7 and 8 → 9 too, not understood) | Same for hiring and letting go; the others end the visit |
| Hire | Scene 1: yield 3 (answered with eight zeros, on resume opcode 329), 24 (0), 25 with 1 (condition 31 CreatingCharacter on: the client's appearance editor), the client's **SetRetainerAppearance** (670: voice, CustomizeData), 25 with 0 (condition off), 36 (answered 2), the name as an **event text** (308, yield 5) | Same; yield 3 is answered with one 0 (upstream's EventResume8 is 513, not retail's 329) |
| Hired | RetainerStateNotice (144), RetainerState (state 1), yield 5 answered 0; scene 1 returns 11: the list again (choice 11) | Same: the retainer is made in the character's city from the appearance and name (`hire_retainer_at_vocate`); a name that isn't usable is answered 1 |
| Let go | Scene 2: yield 13 (0), yield 9 with their ID: `[1, 0]` let go, `[0, 1]` refused while they have items on the market | Same (`release_retainer_at_vocate`) |
| Leave | ActorControlSelf **152** (how many retainers the character has), EventFinish | Same |

The event text is upstream's `CreateLocalLinkshellRequest` opcode (308), which on 2026.10.06 is also how the client
answers an event's text input: 48 bytes, handler u32, scene u16, yield u8, u8, u32 4, u32 0, the text from byte 16
(`event_text`.) Retail's new retainer ID was `0x0078_0017_xxxx_xxxx` like the older ones; Kawari's are random.

Or with the debug command:

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
| Talk to the bell | EventStart (CustomTalk handler `0x000B0220`, CmnDefRetainerBell 544, condition 23 OccupiedSummoningBell), scene 0 with flags `0x2000` | Same; scene **1001** ("You have not yet hired a retainer.") without retainers, as retail |
| The list | Client trigger 9003 command 2 → ten RetainerInfo (empty slots only have their index) and RetainerInfoEnd, all with the inventory sequence | Same, from the table |
| Choose one | Scene 0 returns their ID as two numbers, high half first, **signed** (a half with its top bit set comes back negative) | `player:summon_retainer(high, low)`, which keeps each half's 32 bits; it took them as unsigned, so a retainer whose ID's low half had its top bit set couldn't be called and the player was left stuck in the bell's event (2026-10-18 bundle) |
| Call them | RetainerStateNotice (144), RetainerState (state 4), their containers, ItemMarketBoardInfo per listing, ItemMarketBoardSummary, RetainerStateNotice, ActorControlSelf 570 and 574 (zeros), then scene 2 with `{4, retainer actor ID, n, 0}` and their NPC (SpawnNpc, object kind 10). n was 3 with two retainers hired, 4 with one | Same, n = 5 − retainers hired (a guess from the two); the containers once (retail sent them two or three times, an Equip from the retainer between), no ACS 570/574 |
| Their menu | Client trigger 9003 command 4 → ActorControlSelf 925 (ServerRequestReply) with command 4, result 0 | Same |
| Drag and drop | ItemOperation Move/Exchange between the bags and containers 10000–10006 → InventoryActionAck only | Same; the retainer's items are saved after each change |
| Entrust / withdraw from the menu | ItemOperation **BASE+22** (entrust, 0x132 on 2026.10.06) / **BASE+23** (withdraw, also from the market 12002): source and quantity only, the **server picks the destination** (the same item's stack with room, else the first empty slot) → UpdateInventorySlot for source and destination, InventoryActionAck. The client then sends client trigger **419** (container, slot) and gets that slot again | Same (`transfer_with_retainer`, `retainer_item_operation`, `send_inventory_slot`) |
| Gil | ItemOperation **BASE+24**: entrusting carried the amount in both stacks, "withdraw everything" all zeros → both gil totals (CurrencyCrystalInfo 2000 and 12000), ack, InventoryTransaction, ACS 925 | Amount > 0 entrusts it, 0 withdraws everything; both totals and the ack |
| Put up for sale | ItemOperation **BASE+25** (0x215 on 2026.09.15, 0x135 on 2026.10.06; 0x216 from the crystals) into container 12002, the asking price per item in the destination item field → InventoryActionAck | Same (`RetainerMarketList`, `RetainerMarketListCrystal`) |
| Change a price | Client trigger **400** (slot, price), no reply | Same (`set_retainer_market_price`) |
| Open the sale list | The client sends RequestPlayerRetainerInfo (580 on 2026.09.15, **818** on 2026.10.06) → PlayerRetainerInfo (480 bytes) | Same |
| Sale history | RequestRetainerSaleHistory (243, **274** on 2026.10.06) with the retainer's ID → RetainerSaleHistory | Same, answered empty |
| Close the menu | Scene 2 returns nothing → RetainerStateNotice (`0, 1` if their market changed), RetainerState (state 3, flags `0x81` if it did), scene 1 with `{2, ID high, ID low}`: the list again, which returns like scene 0 | `player:dismiss_retainer()`; the NPC goes away |
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
| RetainerInfo (FFXIVOpcodes' RetainerInformation) | 757 | 466 (upstream, seen) | 80 | sequence, u32, retainer ID u64, slot u8, item count u8, 2 empty, gil u32, market item count u8, town u8 (1 Limsa Lominsa, 2 Gridania, 3 Ul'dah), class/job u8, level u8, market listings' end (Unix time) u32, venture u32, venture done u32, available u32 (1), u8, name (32), 3 empty |
| RetainerInfoEnd (FFXIVOpcodes' RetainerSummary) | 786 | 120 (upstream, seen) | 24 | sequence, entries u8 (10 or 1), retainers allowed u8 (2 on retail; Kawari sends 10), u8 1 at the bell (0 at the vocate), empty, u32 0 (the vocate's menu choice), `FFFFFFFF`, `FFFFFFFF`, `FFFF` |
| RetainerState | 908 | 249 | 56 | retainer ID u64, u32, flags u32 (top byte `0x01` with items on the market, `0x81` sent away after the market changed), u32, state u8 (4 called, 3 sent away, 1 hired), name (32), 3 empty |
| PlayerRetainerInfo | 599 | 712 | 480 | ten of: retainer ID u64, u32 (looks like a Unix time, the same for both), u8 1, u8 0/1, u8 5, name (32), 1 empty |
| ItemMarketBoardInfo | 313 | 387 | 32 | sequence, container u32 (12002), slot u16, 2 empty, u32, price per item u32, 12 empty |
| ItemMarketBoardSummary | 442 | 303 | 8 | sequence, how many items on the market |
| RetainerSaleHistory | 161 | 832 | 1048 | retainer ID u64, then the sales (not read yet) |
| RetainerStateNotice | 897 | 144 | 8 | All zeros (`00000000 01000000` before sending away a retainer whose market changed), before each RetainerState and after the containers |
| RequestPlayerRetainerInfo (client) | 580 | 818 | 8 | All zeros |
| RequestRetainerSaleHistory (client) | 243 | 274 | 8 | Retainer ID |
| SetRetainerAppearance (client) | | 670 | 32 | voice u8, CustomizeData (26), u8 `FF`, 4 empty |
| Event text (client; upstream's CreateLocalLinkshellRequest) | | 308 | 48 | handler u32, scene u16, yield u8, u8, u32 4, u32 0, text (32) |
| ActorControlSelf 152 RetainerCount | | | | how many retainers (as a vocate visit ends) |
| Client trigger 400 / 419 | | | | (slot, price): a new asking price; (container, slot): send this slot again |

The containers: pages 10000–10006 (25 slots each, 175 in all), 11000 what they wear, 12000 gil and 12001 crystals
(both sent as CurrencyCrystalInfo, like the player's), 12002 the market (20 slots). They're sent in that order except
the market comes before what they wear. The ItemMarketBoardInfos all have one new sequence after the containers, and
the summary has it too.

Item operation types on 2026.10.06: BASE_INVENTORY_ACTION is 284 (0x11C), Move 0x123 (BASE+7); the retainer ones are
BASE+22 to BASE+25 (above), the same offsets as 2026.09.15's (where BASE was 0x1FC: 0x212 was an entrust from the menu.)

Checked by tests: `retainer_info_layout`, `retainer_state_layout`, `item_market_board_info_layout`,
`read_retainer_entry_request`, `reads_event_text` (core); `hire_list_and_release`, `entrust_and_withdraw`,
`entrust_from_the_menu`, `list_on_retainer_market` (world). The bell and vocate scripts were run against the retail
flows with lupa (bell: scene 0 → 2 → 1 → 2 → 1 → finish; vocate: 4 → 0 → hire with its yields → 11, and 4 → 0 → let
go.)
