# Glamour Dresser, Plates and Armoire

An inn room's glamour dresser (with the glamour plates) and armoire, each character's own and kept with the inventory.
Code: `servers/world/src/glamour.rs` (the storage and its rules; unit tests), `servers/world/src/zone_connection/glamour.rs`
(the packets), the ClientTriggers in `main.rs`, `resources/scripts/events/custom/004/CmnDefPrismBox_00451.lua` and
`000/CmnDefCabinet_00082.lua` (the two menus).

**Sources**: Bob's Chronofoil capture of 2026-10-24 (20 minutes in the Hourglass: the dresser opened, an item stored
and restored, a plate saved, the armoire opened, an item withdrawn and stored again), the 7.57 client's zone dispatcher
(the cases these opcodes go to) and FFXIVClientStructs' `MirageManager` and `Cabinet` for the sizes.

## Storage

`Inventory::glamour` (`GlamourStorage`, in the inventory's JSON; inventories saved before it read it as empty):

* **Dresser**: 800 slots (MirageManager.PrismBoxItemIds), each an item and two stains. HQ items are their ID +
  1,000,000 (retail had 1026447). A stored item goes to the first empty slot (retail's 444 items filled 0–443 and the
  next went to 444); restoring empties its slot.
* **Plates**: 20, each 12 slots (main hand, off hand, head, body, hands, legs, feet, earrings, necklace, bracelets,
  both rings) of an item and two stains.
* **Armoire**: one bit per Cabinet row (row 204 is byte 25's bit 4), 136 bytes.

## The glamour dresser

Talking to it (CustomTalk 451) plays scene 0 with no parameters: the menu (retail's; the old script sent `{0}`, which
said the dresser wasn't unlocked). Then, as retail:

| Client | Server |
|---|---|
| ClientTrigger 2350 (the dresser window) | **GlamourDresser** (opcode 436, 2408 bytes) twice: page 0 and 1, each `u32 page, u32 item[400], u8 stain0[400], u8 stain1[400]`, 4 bytes |
| ClientTrigger 2356 `[1]` (Glamour Plates opens; `[0]` as it closes) | **GlamourPlates** (opcode 400, 1440 bytes): 20 × `{u32 item[12], u8 stain0[12], u8 stain1[12]}` |
| ClientTrigger 2351 `[container, slot]` (store an item) | the item out of its slot (UpdateInventorySlot), ActorControlSelf **1800** `[dresser index, item]`, LogMessage 4380 with the item |
| ClientTrigger 2352 `[index]` (restore one) | ActorControlSelf 1800 `[index, 0]`, LogMessage 4383, InventoryActionAck (0xFFFFFFFF, 6), the item in the first free bag slot, LogMessage 789 |
| **SaveGlamourPlate** (client opcode 156, 248 bytes): `u8 plate, u8 set[12], u8 ?[12], 3 bytes, u32 index[12], u32 container[12]`, then 124 bytes (9999s and 0xFFFFFFFFs, not understood) | **GlamourPlateUpdated** (opcode 621, 80 bytes): `u32 plate`, the plate, 4 bytes; LogMessage 4382 with the plate's number (1-based) |

A plate's slot that's set takes the item and stains of the dresser index it names when its container is 9999 (the
dresser); a slot that isn't set comes back empty (retail: the off hand). The client sends ClientTrigger 830 after each
change (below).

Not done: outfit sets (MirageStoreSetItem), items from the armoire on a plate, and
glamour prisms (the capture showed a stack of item 21800 resent as an item was stored; no count changed that's known).

## Applying a plate

From Bob's Chronofoil capture of 2026-10-24 (Middle La Noscea: the character window's plate list, five plates, a
gearset with plate 9 linked):

| Client | Server |
|---|---|
| ClientTrigger 2356 `[0]` (the character window's plates) | GlamourPlates, as at the dresser |
| ClientTrigger **2357** `[1]` / `[0]` (the plate list opens / closes to apply one) | condition 14 (`Occupied39`) on / off |
| ClientTrigger **2358** `[plate]` (0–19) | the slots worn that changed (UpdateInventorySlot: the glamour, and the plate's stains on the item), PlayerStats, the model (Equip), LogMessage **4364** `[plate + 1]`, a **4309** line `[item worn, glamour]` for each plate slot with an item (ActorControlSelf 517 with three words), ActorControlSelf **1801** `[1]` |
| **EquipGearsetGlamour** (client opcode 605, 128 bytes) | the gearset equipped as EquipGearset, then the plate applied as above |

A plate's 12 slots dress main hand, off hand, head, body, hands, legs, feet, earrings, necklace, bracelets, right and
left ring (no waist). A plate slot with no item leaves that piece's glamour as it was (retail kept the main hand's
under three plates without one). Retail also sent ActorControlSelf 521 (item level), 57 and 804 (805 after the
gearset), meaning unknown; Kawari sends its own stats.

**Gearset links.** A gearset's linked plate lives with the gearset on the client (the Gear Set List's link and unlink
sent nothing). Equipping a gearset with one sends **EquipGearsetGlamour**: EquipGearset's 72 bytes with the plate's
number (from 1) at byte 60, where EquipGearset has 0, then 56 bytes that held nothing recognisable (upstream's
EquipGearset2, 1487, is the same size with an old opcode).

## The armoire

Talking to it (CustomTalk 82) plays scene 0 with no parameters. Then:

| Client | Server |
|---|---|
| ClientTrigger 423 and 427 (the window opens) | **ArmoireContents** (opcode 772, 136 bytes, the bitmask) for 423, **ArmoireInfo** (opcode 445, 504 bytes: 1 and zeros on retail, meaning unknown) for 427 |
| ClientTrigger 424 `[Cabinet row]` (store) | the first of that item in the bags or armoury chest taken; ActorControlSelf **513** `[0, row, 1]`; `[3, row, 0]` when it isn't there (retail's first try) |
| ClientTrigger 425 `[Cabinet row]` (withdraw) | ActorControlSelf 513 `[0, row, 0]`, InventoryActionAck (0xFFFFFFFF, 6), the item in the first free bag slot, LogMessage 789 |

The item is the Cabinet sheet's (`GameData::get_cabinet_item`; workspace icarus feature `Cabinet`).

## Around them

* **The Gold Saucer window** (same capture): the General tab (ClientTrigger 1850) gets GoldSaucerInformation (750,
  40 bytes; retail's layout isn't worked out, Kawari sends zeros). The Chocobo tab (1900) got **opcode 661**, 32 bytes,
  and ActorControlSelf 933 `[3]`: 661 fills RaceChocoboManager (FFXIVClientStructs; the 7.57 handler copies it):
  u16 EXP, u16 EXP to next, u8 flags (stats from the mother, foul weather, female), u8 rank, u8 colour (Stain), u8
  sessions available, u8 ?, u16 father's and u16 mother's pedigree/stars, i16 first and last name
  (RacingChocoboName), five stat percentages, u8 hereditary and learned ability, u8 ?, then head, body and leg barding.
  Kawari has no racing chocobos and sends nothing. The tab of ClientTrigger 2010 got **opcode 410** (32 bytes, for
  AgentId 0xB9: a flag, a u16 rank, two Unix times, more small numbers; not sent). Doman Mahjong's tab (2550) got
  ActorControlSelf **2001** `[0, 1500, 1500]`, which Kawari sends (likely the starting rating).
* **The toy chest** sent nothing to the server in the capture: its mini games and lists are the client's.

* **ClientTrigger 830** (`InventoryChanged`) comes after every inventory change (moves, the dresser, the armoire, an item
  used); retail answered with ActorControlSelf MapMarkerUpdateBegin (1) and MapMarkerUpdateEnd, and so does Kawari.
* **The Mogpendium** (ClientTrigger 9003 with command 24) is answered with the Mogpendium packet (second and third
  words 0x18 and 0x20, as retail; nothing done that week), where it used to get a ServerRequestReply.
