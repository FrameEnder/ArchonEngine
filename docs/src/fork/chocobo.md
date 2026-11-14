# Chocobo Companion

The player's own chocobo: summoned with Gysahl Greens, following and fighting beside its owner, told what to do from
its menu, wearing barding, ridden, and its saddlebag. Unlocking it is the quests' business: My Little Chocobo names it
(`ZoneConnection::name_companion`, [Grand Companies](grand-company.md#my-little-chocobo)).

Code: `servers/world/src/server/companion.rs` (the companion in the world: summoning, its time, withdrawing, riding,
following its owner into fights), `servers/world/src/zone_connection/companion.rs` (the Companion window and the
saddlebag, what's kept with the character), the `buddy` table (`update-2026-10-24_companion`: skill points, command,
favourite feed, barding), `Inventory::saddlebag`.

**Sources**: Bob's Chronofoil captures of 2026-10-24: e8158807 (visit 6, Middle La Noscea: summoning twice, riding
and dismounting, barding on every slot, two stances, two skills learnt, withdrawing twice, the saddlebag) and
8d85b18d (Lakeland: a level 88 companion with Attacker at 6 fighting five monsters beside his White Mage in Attacker
and Free Stance, Greens added while out, withdrawn); the client's PlayerSetup layout (Kawari's) and ClientStructs'
`CompanionInfo`; the BuddyAction, BuddyEquip, BuddySkill, BuddyRank, Trait and Action sheets (XIVAPI v2, checked
against the captures); the Console Games Wiki's Chocobo Companion page for EXP, ranks, skill points and Healer and
Defender Stance, which no capture has.

## What the character keeps

PlayerSetup carries it at login: name, EXP, rank, stars, skill points, its command, colour, favourite feed, the three
skill lines' levels (Defender, Attacker, Healer), the barding unlocked and the barding worn (BuddyEquip rows: head,
body, legs). Retail's: EXP at offset 0x20, rank/stars/points/command/colour/feed at 148-153, the name and levels after
the supply satisfaction. The zone connection sends the same to the server (`ToServer::SetCompanion`) at every zone-in
and change.

**The colour** isn't PlayerSetup's byte, which was 0 in every retail login: retail sent ActorControlSelf **274**
`[stain]` right after it, at every login, 36 (Desert Yellow) for a chocobo never recoloured (all of Bob's Chronofoil
captures to 2026-10-29), and the Companion window's appearance tab shows that. Kawari sends it at login since
2026-10-29 (`CompanionColor`, upstream's `UpdateHater`; `CompanionProfile::stain`); before, the tab had no colour.
What retail sends when the colour changes (feeding the chocobo fruit) isn't recorded, and changing it isn't done.

## Summoning

Gysahl Greens (item action 851) cast for 1 s; then the item is used up (UpdateInventorySlot) and the ActionEffect,
and the companion comes out (retail's order, all to the owner):

1. CompanionUpdate (ActorControlSelf 253): `[out, 1, time left in centiseconds, EXP, rank, skill points]`: 180,000
   after one Greens (30 minutes). More Greens add 30 minutes, up to an hour (Bob; capture 8d85b18d: 140,971 left,
   one more Greens, 318,680.)
2. 264 `[companion, its command]` (Kawari's `UnkCompanionRelated`): 4 was Free Stance.
3. SetupPet (257) `[owner, 0, companion, 1]`.
4. The companion's StatusEffectList, then CompanionUpdate again, then its SpawnNpc.
5. ~0.8 s later, on it: ActorControl 36 `[1, 18009]` and 54 `[1]` (54 is Sapphire's ToggleNameHidden; 36 unknown.)
   Coming along at a zone-in instead, ~1.6 s after its spawn: 267 `[0]` and 54 `[1]`.

Its spawn as retail's: BNpcBase 952, its name, owned by the player, sub-kind Chocobo, the owner's level, MP 7,000, behavior 634, ModelChara 1, ModelScale 27, character data
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
* **Barding** (client trigger 1701 `[slot, BuddyEquip row]`, slot 0 head, 1 body, 2 legs): ActorControlSelf 255
  `[slot, row]` to the owner, summoned or not (`CompanionBarding`: the Companion window's appearance tab shows only what
  it says; until 2026-10-29 Kawari sent nothing and the tab never changed, Bob's bundle 20261029-152644), then, when
  it's out, the companion's Equip, with BuddyEquip's ModelTop/ModelBody/ModelLegs (model | variant << 16) in head, body
  and feet. A row the character doesn't have is refused.
* **The company's saddle**: My Little Chocobo ends with ActorControlSelf 254 `[row]` (the barding given) and 255
  `[slot, row]` for the three slots, before FinishQuest: row 5 the Gridanian Saddle (0845b9a2), 9 the Ul'dahn Saddle
  (bab8c255); the Maelstrom's 1 is BuddyEquip's row for that company with Order 1, as the other two are. The server
  gives it as the quest finishes (`COMPANY_SADDLES` in `zone_connection/quest.rs`, `grant_company_saddle`); at login a
  character who finished one without it (before 2026-10-29) is given it, and wears it in all three slots if the
  chocobo wears nothing. Other content can call `player:give_companion_barding(row)`.
* **Skills** (client trigger 1702 `[line]`, 0 Defender, 1 Attacker, 2 Healer): the line's next skill costs its place
  in the line in points (retail: 11 points, Attacker at 4, then 6, then 0), answered with CompanionUpdate (no time)
  and the companion's StatusEffectList.

**Its HP** is its level's, not a share of its owner's: 36,330 at level 90 and 35,038 at 88 (Bob's level 88 White
Mage had 35,118, his level 90 character 69,756), both with Attacker's +5% HP. Kawari: ParamGrow's HpModifier × 9.42
(both within 0.3%), plus its traits (`companion_hp`.)

## Fighting

In Free, Defender or Attacker Stance the companion goes after what its owner fights (their target if it has them on
its enmity list, else any enemy that has); in Follow or Healer Stance it stays out of fights. Enemies hit it as they
hit players. It walks after its owner like a pet and is put back beside them when far behind. As retail's level 88
companion with Attacker at 6 fought in Lakeland:

* **Auto-attack** 871 (piercing) every 3.03 s, about 1,743 a hit on those monsters (49 hits), critical 4 times (1.53×).
* **Skills**: the first 5.1-8.1 s into a fight, then one every 7.9-8.5 s: Choco Slash (824, blunt, potency 150,
  2,664) and Choco Beak (823, piercing, 130, 2,251, and its damage over time, Choco Beak 236, on the target each
  time) in no fixed order in Attacker Stance; in Free Stance only Choco Beak, four times. Kawari picks at random from
  what the stance uses: Defender's or Attacker's learnt attacks, or in Free Stance each attacking line's latest.
* **Damage**: 17.5 a potency at level 88 with +5% Strength (all three numbers agree); the companion's timeline gets
  it as `recorded_amounts` for every attack (and the heals), scaled by HpModifier for other levels (a stand-in) and by
  its Strength traits (Mind for heals.)
* On the companion at a fight's start ActorControl 0 `[1, 1]` and 4 `[1]`, at its end 0 `[0, 1, 1]` and 4 `[0]`
  (its weapon and auto-attack, as for other NPCs); CompanionUpdate (no time) ~2 s after each kill, EXP unchanged
  (487,338 at rank 6: monsters far below Bob gave neither of them EXP.)

**The skill lines** (BuddySkill; a level's active skill from the Action sheet, the others traits from Trait 71-88):

| Level | Defender | Attacker | Healer |
|---|---|---|---|
| 1 | Choco Drop (820): 150, more enmity | Choco Slash (824): 150 | Choco Regen (828): regen 25, 18 s (status 237) |
| 2 | Strength +5% | Strength +5% | Mind +10% |
| 3 | HP +10% | HP +5% | HP +5% |
| 4 | Choco Kick (819): 130, Slow 20% 12 s | Choco Beak (823): 130, DoT 20 18 s (236) | Choco Cure (827): cure 300 |
| 5 | Accuracy +5% | Accuracy +5% | Accuracy +5% |
| 6 | Critical hit +15% | Critical hit +15% | Critical hit +15% |
| 7 | Choco Guard (818): 15% less damage, 15 s | Choco Rush (822): 130, Stun 5 s | Choco Surge (826): healing +30%, 30 s (238) |
| 8 | Strength +5% | Strength +10% | Mind +15% |
| 9 | HP +10% | Attack speed +10% | HP +5% |
| 10 | Choco Strike (817): 150 cone, more enmity | Choco Blast (821): 170 cone | Choco Medica (825): cure 150 around it |

Healer spells cast for 1.5 s. Their scripts are `resources/scripts/actions/008/Choco*_008xx.lua`. **Healer Stance**
(no auto-attack) and Free Stance with Healer skills learnt look after the owner and the companion every 3 s, as the
Wiki tells it: Choco Medica when both are low, Choco Cure below 75% (50% in Free Stance), Choco Surge in Healer Stance
when it has worn off, Choco Regen on someone hurt without it.

## Ranks, EXP and skill points

PlayerSetup's and CompanionUpdate's EXP is within its rank (Bob's: 487,338 at rank 6, whose BuddyRank ExpRequired is
490,000.) A kill while it's out gives it EXP (a stand-in: its owner's base EXP for the kill; the Wiki: only monsters
around the player's level count); below rank 10 it ranks up as the bar fills. From rank 10 the bar stops full and
it ranks up by eating a **Thavnairian Onion** (ItemAction 3292: `items/Items.lua` → `player:feed_companion_onion`),
which is used up only then, up to rank 20 (`gain_exp`, `eat_onion`.) Each rank brings skill points: the rank itself
to 10 (55 in all), 10 each to 16, then 11 to 14 (165 at 20, enough for all three lines; the Wiki, and retail's rank 6
companion had spent 21 = 1 + ... + 6); a skill costs its level. Its HP and attacks follow its skill lines as they
change.

## Riding

Mounting (any mount; four of Bob's Chronofoil captures): the request; ActorCast at +0.05 s (action 4 Mount, action
type 13, the mount's ID, 1.0 s) and Condition 0x04; at +1.03 s the condition cleared and ActionEffect1 (the mount's
ID, a 0.1 s lock); at +1.6 s ActorControl 0 `[0, 1]`, ActorControl 2 `[4]`, Condition, 928 `[12]`, the Mount packet
and 263 `[1]`. Kawari casts action 4 for every mount (`MOUNT_ACTION`; until 2026-10-29 it looked the mount's ID up as
an Action row, so mount 1 had no cast and 412 a 2.5 s one) and sends the Mount packet 0.57 s after the cast lands
(`MOUNT_ARRIVES_AFTER`), if still mounted. The mode ActorControls (0, 2) around it aren't sent. **Moving doesn't
interrupt it**: 266 of the 439 mount summons in Connor's Chronofoil captures (2026-11-03 to 09) were made on the run
(2 to 5.6 yalms covered during the bar, from its first 0.1 s) and none of the 439 was cancelled; Kawari cancelled one if
the player moved in its first half second, like a spell, until 2026-11-09 (the cast isn't `interruptible`.)

**Zoning while mounted** (Bob's Chronofoil captures of 2026-10-28 and 29, every zone change made mounted): into a zone whose
TerritoryType has no Mount (Ul'dah, the Waking Sands, the Twins' Gridania) the spawn had no mount and the Condition
after the load was empty; into one that has (Eastern Thanalan) retail re-sent ActorControl 0 `[0, 1]`, 2 `[4]`, the
mounted Condition, 928 `[9]` and the Mount packet 1.5 s after PrepareZoning. Kawari takes the mount off on the way
into a zone without them (`handle_zone_change`, `GameData::zone_allows_mounts`); until 2026-10-29 it kept it in the
spawn and they rode through Ul'dah (Bob's bundle 20261029-162434). Into a zone with mounts the spawn keeps it, without
retail's re-sent packets.

Mounting one's own chocobo (Mount 1) with the companion out: ActorControl 272 `[0]` on the companion (it goes), 928
`[12]` to the owner, the Mount packet (`[1, colour, head, body, legs]`, each barding model as u16 model and u16
variant), 263 `[1]`. Its clock stops while ridden (180,000 cs before, 179,074 after a ride that began 9.3 s after the
summon.) Dismounting brings it back: 264, 263 `[1]`, SetupPet, its StatusEffectList, CompanionUpdate, 263 `[0]`,
CompanionUpdate, SetPetParameters, then its spawn (a new actor.) Others see the barding in the rider's spawn
(`mount_head/body/feet/color`: the BuddyEquip rows.)

**Put on a mount by an event** (2026-11-14; Bob's Chronofoil captures da424662 and d1c46dff, Connor's e834e436):
Every Little Thing She Does Is Magitek (1005) seats the player on a magitek reaper (MOUNT0, 14) as Wedge's scene 9 ends,
without a cast: ActorControlSelf 928 `[9]` (as after a zone change made mounted; 12 when the player summons one), the
Mount packet, then the notice, UpdateQuest and EventFinish, and 263 `[1]`. `player:mount(id)` in a quest script
(`ZoneConnection::mount_by_event`, `chocobo_taxi.rs`) sends them, makes the mode Mounted and puts the mount in the spawn
others see (`ToServer::ChocoboTaxiMount`, the porter's path, now any mount the server seats the player on). The player
gets off as from any mount. The client hides the reaper itself (ACTOR7, an event NPC from the zone's layout, which
retail never spawns) in the sequences of the ride, 5 to 7, mounted or not (its listener table lists it in 4 and 8-10);
Wedge, listed in each of 5-7, puts the player back on (the client's scenes 17, 23, 30; unseen on retail).

**Yield 232: off the mount before a scene goes on.** A scene can open by asking to take the player off whatever they
ride: the client yields with ID 232. Retail answered all six in the captures itself: the five mounted with SetMode 1 and
PlayDismountAnimation (ActorControl 2 `[1]`, 930 with where they land, from the player), then EventResume with nothing;
the one on foot with the resume alone (1005's scene 27 on the reaper at its third event range; 1190's scene 3 and
1192's scene 4, and 3885's 12, on Connor's chocobo). The server answers it before the event's script sees it
(`ZoneConnection::answer_dismount_yield`, `dismount_by_event`, `events` in `main.rs`): the script gets the scene's return
as for any other. Until 2026-11-14 the transcribed scripts took that yield as the scene's end, and nobody was taken off.

## The saddlebag

Client trigger 444 opens it: UpdateItem for each item of containers 4000 and 4001 (35 slots each) and their
ContainerInfo, then 4100 and 4101 empty (the premium saddlebag.) Moving items uses the usual ItemOperations, plus
two of its own: **SaddlebagStore** (BASE + 20, 0xFE on 2026.10.16) from the bags and **SaddlebagTake** (BASE + 21,
0xFF), where the server picks the slot as for a retainer (onto the same item's stack if it has room, else the first
empty slot) and sends both slots, then the acknowledgement. Moves within the saddlebag were acknowledged only.

Retail's reply to 444 (Bob's Chronofoil capture e8158807, 2026-10-16) is the same shape: each container's UpdateItems
and its ContainerInfo share one item sequence (0x23b for 4000 with 4 items, 0x23c for 4001 with 12), then 4100 and 4101
empty. When a saddlebag "doesn't open", look at what's in it: Lira Ormont's of 2026-10-26 answered every 444 with four
empty ContainerInfo, since her saddlebag had never been filled (ArchonEngine ControlPanel's importer before 2.22.3,
[ArchonEngine ControlPanel](kawari-web.md#auracite-backups-2220)); an empty saddlebag shows nothing, which reads as not
opening.

The saddlebag is kept in the inventory (`Inventory::saddlebag`, missing in inventories saved before it: read as two
empty pages), and its items get their sheet data at login like the bags' (`prepare_player_inventory`).

## Not done yet

See [Known Issues](known-issues.md#chocobo-companion).
