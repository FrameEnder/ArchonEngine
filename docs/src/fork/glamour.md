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

Not done: applying a plate to the gear worn, outfit sets (MirageStoreSetItem), items from the armoire on a plate, and
glamour prisms (the capture showed a stack of item 21800 resent as an item was stored; no count changed that's known).

## The armoire

Talking to it (CustomTalk 82) plays scene 0 with no parameters. Then:

| Client | Server |
|---|---|
| ClientTrigger 423 and 427 (the window opens) | **ArmoireContents** (opcode 772, 136 bytes, the bitmask) for 423, **ArmoireInfo** (opcode 445, 504 bytes: 1 and zeros on retail, meaning unknown) for 427 |
| ClientTrigger 424 `[Cabinet row]` (store) | the first of that item in the bags or armoury chest taken; ActorControlSelf **513** `[0, row, 1]`; `[3, row, 0]` when it isn't there (retail's first try) |
| ClientTrigger 425 `[Cabinet row]` (withdraw) | ActorControlSelf 513 `[0, row, 0]`, InventoryActionAck (0xFFFFFFFF, 6), the item in the first free bag slot, LogMessage 789 |

The item is the Cabinet sheet's (`GameData::get_cabinet_item`; workspace icarus feature `Cabinet`).

## Around them

* **ClientTrigger 830** (`InventoryChanged`) comes after every inventory change (moves, the dresser, the armoire, an item
  used); retail answered with ActorControlSelf MapMarkerUpdateBegin (1) and MapMarkerUpdateEnd, and so does Kawari.
* **The Mogpendium** (ClientTrigger 9003 with command 24) is answered with the Mogpendium packet (second and third
  words 0x18 and 0x20, as retail; nothing done that week), where it used to get a ServerRequestReply.
