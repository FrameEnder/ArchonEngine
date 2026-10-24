# Chocobo Companion

The player's own chocobo: summoned with Gysahl Greens, following and fighting beside its owner, told what to do from
its menu, wearing barding, ridden, and its saddlebag. Unlocking it (the quest that names it) is the quests' business.

Code: `servers/world/src/server/companion.rs` (the companion in the world: summoning, its time, withdrawing, riding,
following its owner into fights), `servers/world/src/zone_connection/companion.rs` (the Companion window and the
saddlebag, what's kept with the character), the `buddy` table (`update-2026-10-24_companion`: skill points, command,
favourite feed, barding), `Inventory::saddlebag`.

**Sources**: Bob's Chronofoil capture of 2026-10-24 (e8158807, visit 6, Middle La Noscea: summoning twice, riding
and dismounting, barding on every slot, two stances, two skills learnt, withdrawing twice, the saddlebag); the
client's PlayerSetup layout (Kawari's) and ClientStructs' `CompanionInfo`; the BuddyAction, BuddyEquip, BuddySkill and
BuddyRank sheets (XIVAPI v2, checked against the capture).

## What the character keeps

PlayerSetup carries it at login: name, EXP, rank, stars, skill points, its command, colour, favourite feed, the three
skill lines' levels (Defender, Attacker, Healer), the barding unlocked and the barding worn (BuddyEquip rows: head,
body, legs). Retail's: EXP at offset 0x20, rank/stars/points/command/colour/feed at 148-153, the name and levels after
the supply satisfaction. The zone connection sends the same to the server (`ToServer::SetCompanion`) at every zone-in
and change.

## Summoning

Gysahl Greens (item action 851) cast for 1 s; then the item is used up (UpdateInventorySlot) and the ActionEffect,
and the companion comes out (retail's order, all to the owner):

1. CompanionUpdate (ActorControlSelf 253): `[out, 1, time left in centiseconds, EXP, rank, skill points]`: 180,000
   after one Greens (30 minutes). More Greens add 30 minutes, up to an hour (a guess.)
2. 264 `[companion, its command]` (Kawari's `UnkCompanionRelated`): 4 was Free Stance.
3. SetupPet (257) `[owner, 0, companion, 1]`.
4. The companion's StatusEffectList, then CompanionUpdate again, then its SpawnNpc.
5. ~0.8 s later, on it: ActorControl 36 `[1, 18009]` and 54 `[1]` (54 is Sapphire's ToggleNameHidden; 36 unknown.)

Its spawn as retail's: BNpcBase 952, its name, owned by the player, sub-kind Chocobo, the owner's level, HP 36,330
of Bob's 69,756 (52%: Kawari scales the owner's), MP 7,000, behavior 634, ModelChara 1, ModelScale 27, character data
flags 1, link range 20; its models: barding in head, body and feet, its body (model 1, variant 1, the colour as the
stain: 36 Desert Yellow for a chocobo never dyed) in legs. Retail's display flags were 0x40028; Kawari leaves out
0x20 (its INVISIBLE), which nothing seen afterwards would have cleared.

At zone-in with no companion out retail sent 263 `[0]`, CompanionUpdate `[0, 1, 0, …]` and SetPetParameters
`[0, 0, 0, 7]`. A companion out when its owner changes zone comes along (spawned again at the zone-in, its time kept.)

## Commands, barding, skills

* **Commands** (client trigger 1700, a BuddyAction row): 2 Withdraw; 3 Follow, 4 Free, 5 Defender, 6 Attacker, 7
  Healer Stance, answered with 264 `[companion, command]` and kept with the character.
* **Withdraw**: ActorControl 258 `[1]` on the companion, CompanionUpdate `[0, 1, 0, …]`, 264 with 2; 2 s later
  SetupPet with no pet (0xE0000000), CompanionUpdate again, and it's deleted. Its time is lost. The same when its
  time runs out or it falls.
* **Barding** (client trigger 1701 `[slot, BuddyEquip row]`, slot 0 head, 1 body, 2 legs): the companion's Equip,
  with BuddyEquip's ModelTop/ModelBody/ModelLegs (model | variant << 16) in head, body and feet.
* **Skills** (client trigger 1702 `[line]`, 0 Defender, 1 Attacker, 2 Healer): the line's next skill costs its place
  in the line in points (retail: 11 points, Attacker at 4, then 6, then 0), answered with CompanionUpdate (no time)
  and the companion's StatusEffectList.

## Fighting

In Free, Defender or Attacker Stance the companion goes after what its owner fights (their target if it has them on
its enmity list, else any enemy that has), with auto-attack 871 (another player's companion used it on a monster in
the capture) from 3 yalms (a guess); in Follow or Healer Stance it stays out of fights. Enemies hit it as they hit
players. It walks after its owner like a pet and is put back beside them when far behind.

## Riding

Mounting one's own chocobo (Mount 1) with the companion out: ActorControl 272 `[0]` on the companion (it goes), 928
`[12]` to the owner, the Mount packet (`[1, colour, head, body, legs]`, each barding model as u16 model and u16
variant), 263 `[1]`. Its clock stops while ridden (180,000 cs before, 179,074 after a ride that began 9.3 s after the
summon.) Dismounting brings it back: 264, 263 `[1]`, SetupPet, its StatusEffectList, CompanionUpdate, 263 `[0]`,
CompanionUpdate, SetPetParameters, then its spawn (a new actor.) Others see the barding in the rider's spawn
(`mount_head/body/feet/color`: the BuddyEquip rows.)

## The saddlebag

Client trigger 444 opens it: UpdateItem for each item of containers 4000 and 4001 (35 slots each) and their
ContainerInfo, then 4100 and 4101 empty (the premium saddlebag.) Moving items uses the usual ItemOperations, plus
two of its own: **SaddlebagStore** (BASE + 20, 0xFE on 2026.10.16) from the bags and **SaddlebagTake** (BASE + 21,
0xFF), where the server picks the slot as for a retainer (onto the same item's stack if it has room, else the first
empty slot) and sends both slots, then the acknowledgement. Moves within the saddlebag were acknowledged only.

## Not done yet

See [Known Issues](known-issues.md#chocobo-companion).
