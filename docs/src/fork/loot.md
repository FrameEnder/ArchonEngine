# Coffers and Loot

Dungeon coffers and their Need / Greed / Pass rolls, as retail does them (Bob's Chronofoil capture of five Sastasha
runs with Duty Finder parties, 2026-10-22: 43 coffers spawned, 15 opened, 35 lots cast by the other players, 18 items
decided). Party bots roll as those players did. With no other human in the party (Duty Support, or bots) a coffer
is yours alone, no rolls ([below](#one-player-personal-loot)). Code: `servers/world/src/server/loot.rs`; the duty's script spawns the
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
numbers: `director:treasure_loot(base_id, { { items of one opening }, ... })`, and, as the starting point (Bob,
2026-10-28: a few runs miss much of a pool), the wiki's list for each coffer: `director:treasure_pool(base_id, { items
})`. A coffer picks how many items from one of its recorded openings (one if it was never recorded opened), then each
item from everything recorded in it and the wiki's list together, never two of a kind (EquipSlotCategory, weapons'
one- and two-handed slots as one; ItemUICategory for anything not worn: `loot::item_kind`): Chopper's two items are a
piece of gear and an accessory, as retail's were. A coffer with neither holds one item from all of the duty's
recordings. A recording that shows an item the wiki lacks adds it (it's in `treasure_loot`).

The wiki's lists (FFXIV Console Games Wiki, each dungeon's page: every boss's coffer, and each "Treasure Coffer" with
its map position) are matched to the coffers' base IDs by the recorded items and by position: the coffers' spawn
positions in Bob's Chronofoil captures as map coordinates (`41 / c × ((x × c + 1024) / 2048) + 1`, c the Map's
SizeFactor / 100: Sastasha 2.0, Tam-Tara 3.0, Copperbell 2.0) fall within 0.1 of the wiki's. All 26 items ever
recorded in these three dungeons are on the wiki's lists. The level 15-17 sets are shared (`HEAD_HANDS`,
`ACCESSORIES`, `BODY_LEGS`, `WEAPONS`, `FEET`, `OLDER_PIECES` in each script):

| Dungeon | Coffer | The wiki's |
|---|---|---|
| Sastasha | 94, 95, 93 | Chopper (head, hands, accessories), Captain Madison (body, legs), Denn (weapons, feet, From the Depths) |
| Sastasha | 98, 99, 100, 101 | Treasure Coffers 1-4: feet and Seagrot Water; head and hands; body and legs; weapons and older pieces |
| Tam-Tara | 16, 17, 62, 11 | the Void Soulcounters' three (head and hands; the same and accessories; body and legs), Galvanth (weapons, feet) |
| Tam-Tara | 15, 118, 119 | Treasure Coffers 1-3: feet and accessories; body and legs; weapons and older pieces |
| Copperbell | 52, 53, 51 | Kottos (head, hands, accessories), Ichorous Ire (body, legs), Gyges (weapons, feet, Below) |
| Copperbell | 65, 57, 58, 59 | Treasure Coffers 1-4: feet and accessories; head and hands; body and legs; weapons and older pieces |

The wiki's fixed-rate extras (Sastasha's Triple Triad card, Tam-Tara's orchestrion material) aren't given.

Sastasha (`content/s1d1_re.lua`), from the five runs:

| Coffer | Where (retail) | Recorded openings |
|---|---|---|
| 98 | the coral room, every run (never 97) | none: one of the others' items |
| 94 | Chopper (`BNPC_CHOPPER`) | 4: a piece of level 15 gear (hat, goggles, celata, mitts) and Aetherial Brass Wristlets (3) or earrings |
| 95, 100 | the first captain (`BNPC_CAPTAIN1`): both coffers together, every run | 95: 3 (a body piece or trousers); 100 never opened |
| 99, 101 | The Hole, both every run | 99: halfgloves; 101: a battleaxe |
| 93 | Denn (`BNPC_DENN`) | 3: a weapon (bow, knives, cane), with boots or an orchestrion roll |

The second captain (`BNPC_CAPTAIN2`) dropped nothing in any run, so the script no longer spawns a coffer (96) for him.

A sixth Sastasha run (Bob's capture c99fe374, a Duty Finder party) added an opening of each boss's coffer: 94 the
Plundered Cavalier's Hat and Aetherial Brass Gorget, 95 Plundered Trousers, 93 Foestriker's Boots and the Plundered
Guisarme, all on the wiki's lists. The Tam-Tara Deepcroft's and the Copperbell Mines' scripts have a fourth opening of
each coffer opened in the same capture's Duty Support runs (`LOOT`), all on the wiki's lists too.

An arcanist's Duty Support runs (Bob's capture cc62ad94, 2026-10-28) added more openings: Sastasha's 94 (item 4310),
95 (3329) and 93 (38467); the Tam-Tara Deepcroft's 16 (3555), 62 (3331) and 11 (3786 and 31422); the Copperbell Mines'
52 (2683 and 4207; 2684 and 4206), 53 (3330; 3331), 58 (3331), 59 (31425) and 51 (31421; 31422). Again a coffer's items
weren't chosen for the class.

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

A player who wins gets the item as retail sent it (Bob's Chronofoil capture c99fe374, Sastasha with a Duty Finder
party: LootResult, then the ack 0xFFFFFFFF 6, the item's slot and LogMessage 789, the "obtained" pop-up;
`ZoneConnection::obtain_loot`). Once all of a coffer's items are decided it fades out (TreasureFadeOut, from the
coffer).

## One player: personal loot

Items are rolled for only when another human is in the instance (Bob's rule, 2026-10-28): with one player there
(Duty Support, whose members are NPCs, or a party of bots) a coffer's items are that player's at once (`loot::open`
counts the players). Retail's ten in the Tam-Tara Deepcroft and the Copperbell Mines went that way with Duty Support
(Bob's Chronofoil capture c99fe374 of 2026-10-28, Dags, a gladiator):

1. **OpenedTreasure** from the coffer with no times (all zeros but the coffer); no LootList, no LootItem.
2. For each item, to the opener (`ZoneConnection::obtain_personal_loot`): **LootMessage 3** (who, the item, 1), then the
   item as a won one (the ack, its slot, LogMessage 789). A unique item they already have is lost instead:
   **LootMessage 8** (who, the item, 1, 8), then **LootMessage 2** (the item, 1), and nothing else (a second Acolyte's
   Skirt in Tam-Tara, the Plundered Gauntlets and a second Foestriker's Tabard in Copperbell).
3. The coffer fades out 4 s later (3.96-4.00 s, all nine that faded while recorded; the last boss's was still there as
   the duty ended).

What a coffer holds isn't chosen for the class: a gladiator's held an Acolyte's Skirt (Disciple of Magic), a Plundered
Grimoire (arcanist) and a Plundered Cane (conjurer).

## Party bots

Alone with bots the player gets everything at once (above). With another human in the party the items are rolled for,
and each bot passes on every item (Bob's call), after one of the delays the other players
took on retail (the 35 lots: half within 6 s of the coffer opening, a quarter after 15 s, up to 249 s). Those players
passed 33 times and greeded twice, and never needed.

Bots don't open coffers themselves yet (retail's players opened the bosses' 1.5–2.2 s after they appeared).

## Enemies' gil

In a duty with another player there (or a Duty Finder party bot), the duty's own enemies drop a few gil for each player (LootMessage 9, then
ActorControlSelf 42); bosses and the adds and waves the director spawns don't, and with Duty Support nothing drops,
like the coffers' rolls. Amounts and retail's counts: [Dungeons: clear rewards](dungeons.md#clear-rewards).

## Loot rules

Greed Only, and Lootmaster as Kawari plays it, turn every Need into a Greed when the party went in full in a duty that
allows them: [Duty Finder Settings](duty-finder.md#loot-rules).
