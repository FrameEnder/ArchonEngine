# Coffers and Loot

Dungeon coffers and their Need / Greed / Pass rolls, as retail does them (Bob's Chronofoil capture of five Sastasha
runs with Duty Finder parties, 2026-10-22: 43 coffers spawned, 15 opened, 35 lots cast by the other players, 18 items
decided). Party bots roll as those players did. Code: `servers/world/src/server/loot.rs`; the duty's script spawns the
coffers and says what they hold.

## Opening a coffer

The duty's script spawns a coffer (`director:spawn_treasure(base_id)`) as a closed dungeon coffer: `coffer_kind` 2
(DungeonRaid) and event state 1, as retail's. Its actor ID is the entity ID in its SpawnTreasure, which is what the
client's OpenTreasure names (they used to be two different random numbers, so nothing could be opened).

The first player to open it (OpenTreasure) opens it for the party. Every player gets:

1. **LootList** (272, 88 bytes): the coffer, how many items, the seconds left to roll (300, three times) and the
   items.
2. **LootItem** (975, 56 bytes) for each item: the coffer, the item, the time left and the total (300), its slot, whether
   it's unique (the Item sheet's IsUnique), and a serial counting the duty's loot. A player opening it again later gets
   these again, marked as opened and with the time left (retail's had 290 s left 9.5 s on).
3. **OpenedTreasure**, from the coffer: 300.0 three times, 1, the coffer.

## What a coffer holds

There's no loot table in the client's data, so each duty's script records what its coffers held on retail, as item
numbers: `director:treasure_loot(base_id, { { items of one opening }, ... })`. A coffer picks how many items from one of
its recorded openings, and each slot one of the items seen in that slot across them; a coffer never recorded open
holds one item from all of the duty's.

Sastasha (`content/s1d1_re.lua`), from the five runs:

| Coffer | Where (retail) | Recorded openings |
|---|---|---|
| 98 | the coral room, every run (never 97) | none: one of the others' items |
| 94 | Chopper (`BNPC_CHOPPER`) | 4: a piece of level 15 gear (hat, goggles, celata, mitts) and Aetherial Brass Wristlets (3) or earrings |
| 95, 100 | the first captain (`BNPC_CAPTAIN1`): both coffers together, every run | 95: 3 (a body piece or trousers); 100 never opened |
| 99, 101 | The Hole, both every run | 99: halfgloves; 101: a battleaxe |
| 93 | Denn (`BNPC_DENN`) | 3: a weapon (bow, knives, cane), with boots or an orchestrion roll |

The second captain (`BNPC_CAPTAIN2`) dropped nothing in any run, so the script no longer spawns a coffer (96) for him.

## Rolling

Each member of the party (the players in the duty and their bots) chooses for each item:

* A player's choice is **LootRoll** (766, 16 bytes, client → server): the coffer, the slot, and 1 need, 2 greed or 5
  pass (greed and pass were seen; need's 1 is the LootMessages' value for it). The server answers with
  **LootRollResult** (462, 16 bytes): the coffer, the slot, then 1 and the roll (1–99) for need or greed, 4 passed, or 8
  refused (retail refused Bob's Greed on a unique item he already had; Kawari doesn't check that yet).
* Everyone is told a member **cast their lot** (LootMessage 7: who, the item, 1), the first time they choose.

An item is **decided** once every member has chosen, when its 300 seconds are up, or when the duty is completed
(retail decided what was left then):

* The Need rolls if there are any, otherwise the Greed rolls: LootMessage 0 for each (who, 1 need / 2 greed, the roll,
  the item, 1) and LootMessage 1 for the highest (the first member of a tie), then **LootResult** (807, 16 bytes: the
  coffer, the slot, the winner, the roll).
* With no rolls, a member who didn't pass gets it with a roll of 0 (LootMessage 3, "obtained": the others had passed and
  Bob hadn't chosen when the time ran out, twice).
* If everyone passed it's **lost** (LootMessage 2: the item, 1; LootResult with 0xE0000000 and 0).

A player who wins gets the item in their inventory (an ack and the inventory, as for an enemy's drop). Once all of a
coffer's items are decided it fades out (TreasureFadeOut, from the coffer).

## Party bots

Each bot passes on every item, so the loot is the player's (Bob's call), after one of the delays the other players
took on retail (the 35 lots: half within 6 s of the coffer opening, a quarter after 15 s, up to 249 s). Those players
passed 33 times and greeded twice, and never needed.

Bots don't open coffers themselves yet (retail's players opened the bosses' 1.5–2.2 s after they appeared).

## Loot rules

Greed Only, and Lootmaster as Kawari plays it, turn every Need into a Greed when the party went in full in a duty that
allows them: [Duty Finder Settings](duty-finder.md#loot-rules).
