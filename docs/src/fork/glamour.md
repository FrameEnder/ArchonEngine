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
| ClientTrigger 2356 (always `[0]` on retail: the plate list opening at the dresser or in the character window, and before equipping a gear set linked to a plate) | **GlamourPlates** (opcode 400, 1440 bytes): 20 × `{u32 item[12], u8 stain0[12], u8 stain1[12]}` |
| ClientTrigger 2351 `[container, slot]` (store an item) | the item out of its slot (UpdateInventorySlot), ActorControlSelf **1800** `[dresser index, item]`, LogMessage 4380 with the item |
| ClientTrigger 2352 `[index]` (restore one) | ActorControlSelf 1800 `[index, 0]`, LogMessage 4383, InventoryActionAck (0xFFFFFFFF, 6), the item in the first free bag slot, LogMessage 789 |
| **SaveGlamourPlate** (client opcode 156, 248 bytes): `u8 plate, u8 changed[12], u8 what[12], 3 bytes, u32 dresser index[12]`, then the dyes: `u32 dye0 bag[12], u16 dye0 slot[12], u32 dye1 bag[12], u16 dye1 slot[12], u8 stain0[12], u8 stain1[12], 4 bytes` | the dyes used taken from their bag slots (UpdateInventorySlot each), then **GlamourPlateUpdated** (opcode 621, 80 bytes): `u32 plate`, the plate, 4 bytes; LogMessage 4382 with the plate's number (1-based) |

The client sends only the slots it **changed** (Bob's bundles of 2026-10-27: his second save of plate 1 named just the
two pieces he'd dyed; the server used to empty every other slot, which left the plate empty.) A slot not changed keeps
what the plate had. For a changed slot, **what** says what changed (bits): 1 its item, from the dresser index it
names, with that item's stains; 2 its first dye, 4 its second (bit 4 taken by analogy: only 3, item and first dye, was
seen). A dye names the bag (0–3; 9999 for none) and slot (0xFFFF) of the dye the client used, and the stain it gives
(0xFF for none); one of that dye is taken. A changed slot with neither its item nor a dye changed is emptied. Bob's
first save: head, hands and feet from the dresser (what 1), body and legs from the dresser dyed in the plate (what 3,
stains 1 and 6, both with the dye in the fourth bag's slot 29); the server used to read the dye's bag (3) as the item's
source and leave both pieces out, so they showed until the plate was reloaded. The client sends ClientTrigger 830 after
each change (below).

Not done: what retail answers a dyed save with beyond the plate (no capture of one; Kawari sends the dye's slot), outfit
sets (MirageStoreSetItem), items from the armoire on a plate, and
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
sent nothing). The client decides whether to apply it: it asks for the plates first (ClientTrigger 2356) when it
hasn't got them since the zone loaded, then sends the gearset with the plate's number (from 1) at byte 60, or 0 for
none. It sends that in either packet: **EquipGearsetGlamour** (605, EquipGearset's 72 bytes then 56 that held nothing
recognisable; upstream's EquipGearset2, 1487, is the same size with an old opcode) or the plain **EquipGearset** (774);
which of the two isn't tied to the plate (retail's clients sent 605 with 0 and 774 with plates 1, 3 and 20). Retail
applied the plate whenever byte 60 had one, in both: LogMessage 4364 and the plate's 4309 lines, in all 15 gearset
changes of Bob's Chronofoil captures 63fcdf06, 8d85b18d, ba1b86c5, cc26e0ea and f4316611 (October 2026). Kawari read
it only from 605 until 2026-11-02; now from both.

When the client sends 0 for a gearset with a plate linked, the server has nothing to apply. Bob's bundles of
2026-11-02 (a Conjurer's gearset linked to plate 1): in Old Gridania (133) the client sent 0, three times at two
places; in New Gridania (132), right after zoning, it sent 1 and the plate went on; its later clicks there sent 0, the
plate being on already. Why it sent 0 in Old Gridania isn't known: both were sanctuaries to the server (rested EXP),
and each time in Old Gridania he had just set a portrait (SetAdventurerPlatePortrait, 4 s before).

**When the client's bags are out of date.** EquipGearset names, for each of the 14 slots, the container and slot the
client thinks the piece is in (`Invalid`, -1 for none; `Equipped` for one already worn). Kawari used to read a named
slot that it had empty as "nothing for this slot" and took off what was worn. Now it leaves that slot as it is, logs
a warning and sends the slot as it really is (UpdateInventorySlot), so the client's view catches up and the next try
works. Lira Ormont's gear sets "sometimes" not working (2026-10-26) were thought to be that (crafts had left her bags
out of step with the client's, [Crafting](crafting.md)), but her bundle has the same pattern as Bob's of 2026-11-04:
she changed class by the weapon (White Mage to Carpenter), equipped the recommended gear, and the client sent
nothing more: after a gearset, Kawari's reply left the client waiting until the zone changed ([Quests](quests.md),
"A gearset's moves").

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
