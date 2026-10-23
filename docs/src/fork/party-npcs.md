# Party NPCs (Duty Support)

Retail's Duty Support members: the story's adventurers who go through a duty with you when you register through the
Duty Support window. They're the fork's [Party Bots](party-bots.md) underneath (`PartyBot` with `party_npc` set), so
they follow, pick their targets, die and get up as the bots do, but each one is a member retail recorded, spawned,
listed, fighting and talking as it did there. The made-up bots (Player NPCs) are unchanged and still fill the Duty
Finder and come from `!spawn playernpc`.

Code: `servers/world/src/server/party_npc.rs`; the members' timelines `resources/timelines/DutySupportSastasha*_<base>.json`
and action scripts `resources/scripts/actions/*/DutySupport_<id>.lua`.

Everything here comes from Bob's Chronofoil capture of 2026-10-23 (`700cc294`, visit 0.3): Sastasha with Duty
Support, 30 minutes, Bob a level 17 Conjurer, the three others a Marauder, a Lancer and a Thaumaturge (`questkit
chronofoil --battle 0.3` and the visit's packets, numbers only). Only Sastasha's three are recorded.

## Registering

**At the duty's entrance** (Sastasha's, EObj 4374460 by its Duty Finder notice in Western La Noscea): it has two
events, and retail played one scene for each, the client's script doing the rest (Bob's Chronofoil capture of
2026-10-23):

* its **InstanceContentGuide** (4 for Sastasha; `events/generic/InstanceContentGuide.lua`): scene 0 with
  `HIDE_HOTBAR | NO_DEFAULT_CAMERA` (0x2001), back at once with no results, then the client's ShownActiveHelp 103;
* its **CustomTalk** 772 (`CmnDefSdsEntrance`, `events/custom/007/CmnDefSdsEntrance_00772.lua`): scene 0 with 0x2001,
  the client's menu; choosing Duty Support sends **ClientTrigger 2650** with the duty (DawnContent 200), which the
  server answers with **DutySupportContent** (opcode 701 on 7.57h1: 200, 0), then the scene returns, the event
  finishes and the Duty Support window opens (retail sent no DutySupportInformation then).

Both had no handler in Kawari until 2026-10-23 (Bob's bundle: "isn't scripted yet", so nothing happened at the
entrance); the window from the Duty Finder (ClientTrigger 2653) worked already.

The Duty Support window's *Register* sends `ClientTrigger` 2654 (`RegisterDutySupport`): the DawnContent row (200
for Sastasha in the capture), then the members chosen, one byte each (`0x00252624`: 36, 38, 37, DawnQuestMember
rows), in party slot order. Upstream already queued the duty from the first word; the fork keeps the members
(`ZoneConnection::duty_support`, cleared by any other registration) and sends them with `JoinContent`. When the player
has loaded into the duty, `party_npc::fill_duty_support` spawns the recorded members they chose and the Duty Finder's
bot filling (`party_bot::fill_duty_party`) takes any role still missing, so a member nobody has recorded yet (a tank's
healer, for one) is a Player NPC of that role for now.

## The members (Sastasha)

| Slot | DawnQuestMember | BNpcBase | BNpcName | Class | Level | HP | `pet_id` | 276 | 1525 |
|---|---|---|---|---|---|---|---|---|---|
| 0 | 36 | 14646 | 11326 | Marauder (3) | 15 | 411 | 1 | 10000 | 1042776 |
| 1 | 38 | 14648 | 11328 | Thaumaturge (7) | 15 | 293 | 5 | 10002 | 1042778 |
| 2 | 37 | 14647 | 11327 | Lancer (4) | 15 | 322 | 9 | 10001 | 1042777 |

Their level is the duty's (15), not synced to the player's: Bob was 17. Names come from their BNpcName in the
client; looks and gear from the game data (the BNpcBase's BNpcCustomize and NpcEquip). Their stats are a player's of
their class, race and level in the best gear they could wear then, with retail's HP.

## How retail spawned and listed them

* **SpawnNpc**, object kind BattleNpc sub-kind 9 (NpcPartyMember), owned by the player, battalion 0, no layout ID,
  the duty's content handler (`INSTANCE_CONTENT` 4), display flags 0x40000, 3,000 MP, behavior 634, link range 20,
  `normal_scaling` set, spawn indexes 1–3, an empty name. The Marauder spawned with status 3124. Character data flags
  were 0x2 (HOSTILE), which Kawari reads as an enemy's; the fork leaves it out, as for bots shown as party members.
* **The party list** came from no PartyList at all: before the spawns, for each slot from the last, ActorControlSelf
  **257** (the player, the slot, the member's actor ID, 1, 2), **276** (the slot, 1, the member number above) and
  **1525** (the slot, an ENpcResident: 1042776 + n), then the member's **StatusEffectList** (class, level, HP, MP),
  preceded once by 257 (the player, 0, nobody, 1, 1), which retail also sent at other moments (when Bob died, at
  the end). 276 came again
  for every slot as the duty began and after the player died. Kawari sends the same (`party_npc::announce`) right
  after spawning them.
* **At the clear**, as Denn died, 257 (the player, each slot, nobody, 1, 2) emptied the slots and the members were
  deleted 0.3 s later. Kawari does it when the director completes the duty (`party_npc::dismiss`).

## Out of combat: the diamond

They kept a sideways diamond around the player, the player in their own role's corner: with Bob (the healer) standing
still, the tank 2.0 yalms straight ahead, the Lancer 1.0 ahead and 1.72 to one side, the Thaumaturge the same to the
other side (183–185 samples each, all within a few hundredths). So the diamond's corners, from its middle, are the
tank 1 ahead, the healer 1 behind and the two damage dealers 1.72 either side; the player stands in their role's
corner and everyone else is placed from there (`party_bot::diamond_spots`, `spot_around`). A player tank has the
healer 2 behind them and the damage dealers 1 behind and 1.72 aside; a damage dealer takes their side's corner (melee
the Lancer's side, ranged the Thaumaturge's). **The Player NPCs use the same diamond now** (more bots than corners
stand in rows of three behind it).

## In combat

Targets and movement are the bots' (the tank takes what hits someone else, the others attack the tank's target).
Measured from what each hit: the tank fought from 3.0 yalms (centre to centre, the median), the Lancer 3.4, the
Thaumaturge 8.1 (5.4–12.2). Each fights with its own kit (`party_npc::Kit`), the actions and timings retail's used,
which are NPC-only actions (not the players'):

| Member | Auto-attack | GCD | Filler, in turn | Burst | Other |
|---|---|---|---|---|---|
| Marauder | 17058, every 3.0 s | 2.67 s | 29299, 29300 | every 20 s (first 9 s in): 26230, 29303 on itself, then 29764, 29765 twice at 2.13 s, then 29766 on itself | 29291 thrown at a fresh pack (21 uses), 29294 its area combo (34, up to 8 hit), 25083 on an enemy turning on someone (30 s at least between), 25082 on itself in pulls of three or more (100 s) |
| Lancer | 17059, every 3.0 s | 2.67 s | 29305, 29306 | every 11 s (first 9 s in): 26230, 29305, 29311 on itself, 29772, 29767 on itself | each fight opens with 29304 |
| Thaumaturge | none | 3.07 s (1.7 s casts) | 29314, 29315 | 29312 every 9 s (first 13 s in) | |

Damage to Sastasha's enemies (medians, the timelines' `recorded_amounts`, which scale the action scripts' hits): the
weaponskills and spells 24–27, the auto-attacks 3, the Marauder's 29764 76 and 29765 128, the Lancer's 29772 275,
the Thaumaturge's 29312 277; Sastasha's trash has 905 HP. That's what "fighting like someone of the duty's level"
looks like: steady, with a big hit every few GCDs, unlike a synced player's rotation.

## Taking hits

On retail the same enemies' attacks did about a third as much to the Marauder as to Bob (a level 17 Conjurer): the
median of 12 attacks seen landing on both (Sastasha's pirates' auto-attacks 16 against 49, 14/45, 10/32, 15/47 and
so on, all 0.27–0.44), and about the same to the Lancer and the Thaumaturge (33/49, 44/45, 54/45, 9–10/11). The tank
took 8,106 damage in the run and the other two 330 each: enemies stayed on the tank, and Bob healed it 7,372. So an
enemy's hit on a party NPC is the hit a player would take (scaled as enemies' hits on players are, see
[Combat](combat.md#enemy-damage-to-players)) times its member's `damage_taken`: 0.31 for the Marauder, 1 for the
others; where the enemy's timeline has an `npcs` amount recorded on the members (Sastasha's bosses), that instead.
Until 2026-10-23 enemies hit them as one NPC hits another, unscaled, about eight times a player's damage (Bob: they
died to two enemies).

## Talking

Their lines are NpcYell packets from the member, under its BNpcName:

* **During fights**, now and then (10 lines in 30 minutes): the Marauder 13951 (2.7 s into its first fight), 13952,
  13961; the Lancer 13962, 13964; the Thaumaturge 13972, 13974. Kawari says one of each member's every 150–400 s of
  fighting, the Marauder's 13951 first.
* **The duty's story lines**, from the director (`director:party_npc_yell(role, line, delay)`, the role "tank",
  "healer", "melee" or "ranged"; nobody of that role, no line): in Sastasha (`content/s1d1_re.lua`) the Lancer's 14296
  43.6 s after the duty began, the Marauder's 14298 as Chopper came, the Lancer's 14300 2 s after the first captain's
  last line, the Thaumaturge's 14302 2 s after the second's, the Marauder's 14303 15.5 s after Denn's cutscene began.

## Not done yet

* Only Sastasha's three are recorded; other duties' members, and Sastasha's healer (for a player who isn't one),
  are Player NPCs until a run with them is recorded.
* What 26230, 29303, 29766, 29311, 29767, 25082 and 25083 do (statuses, enmity) isn't known: they're only their
  animation. The Lancer's 29772 healed it 30 HP on retail; not here.
* Retail's DawnContent, DawnQuestMember and DawnMemberUIParam sheets aren't read: the members are recorded numbers.
* Untested in game.
