# Party NPCs (Duty Support)

Retail's Duty Support members: the story's adventurers who go through a duty with you when you register through the
Duty Support window. They're the fork's [Party Bots](party-bots.md) underneath (`PartyBot` with `party_npc` set), so
they follow, pick their targets, die and get up as the bots do, but each one is a member retail recorded, spawned,
listed, fighting and talking as it did there. The made-up bots (Player NPCs) are unchanged and still fill the Duty
Finder and come from `!spawn playernpc`.

Their stats are a player's of their class, race and level in the best gear they could wear then, **weapon included**
(f(WD), [Combat](combat.md); until 2026-11-12 the weapon wasn't passed, so their hits and heals without a recorded
amount in their timeline, such as the Conjurer's Cure outside Sastasha's levels, got only the level's share of f(WD)).

Code: `servers/world/src/server/party_npc.rs`; the members' timelines `resources/timelines/DutySupportSastasha*_<base>.json`
and action scripts `resources/scripts/actions/*/DutySupport_<id>.lua`.

Everything here comes from Bob's Chronofoil capture of 2026-10-23 (`700cc294`, visit 0.3): Sastasha with Duty
Support, 30 minutes, Bob a level 17 Conjurer, the three others a Marauder, a Lancer and a Thaumaturge (`questkit
chronofoil --battle 0.3` and the visit's packets, numbers only). Only Sastasha's three are recorded. The Copperbell Mines (2026-10-25) and the Tam-Tara Deepcroft (his capture of 2026-10-24, `3ec702a8`
visit 0.3) had the same three, registered the same way (DawnContent 201, members 36, 38, 37), listed with the same
ActorControlSelf 276 and 1525 values, fighting with the same kits, at level 16 with 443, 347 and 316 HP: each duty's
level and HP are in `party_npc::DUTY_LEVELS` (by ContentFinderCondition). See [Dungeons](dungeons.md).

**The trials' three** (the Bowl of Embers, Bob's capture b8432613 of 2026-10-27, visit 0.16): registered under
DawnContent 203 as members 40, 42, 41, the same Marauder, Lancer and Thaumaturge but other DawnQuestMember rows with
BNpcBases 14650-14652, BNpcNames 11330-11332 and ActorControlSelf 276/1525 values 10004-10006 and 1042780-1042782
(slots 0-2: the Marauder, the Thaumaturge, the Lancer), at level 20 with 618, 469 and 428 HP, fighting with the same
kits: `party_npc::TRIAL_MEMBERS`, their timelines `TrialSupport*_<base>.json` (what each action
did to Ifrit and the Nail). Retail spawned them a step ahead of the player in a wedge (the Marauder 2 yalms ahead, the
other two 1 ahead and 1.72 to each side); Kawari spawns them on the player, as in the dungeons.

**The Conjurer, for a tank** (Bob's Chronofoil capture c99fe374 of 2026-10-28, Dags, a gladiator): the client chose
39, 38, 37 in the Tam-Tara Deepcroft and the Copperbell Mines (DawnContent 201 and 202) and 43, 42, 41 in the Bowl of
Embers (203), the Conjurer taking the tank's place and its slot (0: 276 10003 and 1525 1042779 in the dungeons, 10007
and 1042783 in the Bowl), the other two as before. Its BNpcBase is 14649 (BNpcName 11329) in the dungeons and 14653
(11333) in the Bowl, ClassJob 6, at 16 with 316 HP, 17 with 339 and 20 with 432: member 39 in `party_npc::MEMBERS` and
43 in `TRIAL_MEMBERS`, its timelines `DutySupportConjurer_14649.json` and `TrialSupportConjurer_14653.json`.

**For a ranged damage dealer** (Bob's Chronofoil capture cc62ad94 of 2026-10-28, an arcanist): the client chose 36, 39
and 38 in Sastasha, the Marauder, the Conjurer and the Thaumaturge, which is what Kawari already picked. The Conjurer
had 293 HP there at level 15 (`party_npc::DUTY_LEVELS`). With the Conjurer and no Lancer in Sastasha, other members
said three lines that a healer's party doesn't hear: the Marauder's 14294 at the start instead of the Lancer's 14296,
the Conjurer's 14299 at Chopper and its 14301 at the first captain ([Dungeons](dungeons.md): `party_npc_yell`'s
`other_role` and `other_line`).

**The third set** (Brayflox's Longstop, Bob's Chronofoil capture 44df02fc of 2026-11-03, visit 0.20, the player a
level 34 arcanist): registered as 44, 47, 46 under DawnContent 206, the Marauder, the Conjurer and the Thaumaturge with
BNpcBases 14654, 14657 and 14656 (the trials' BNpcNames 11330, 11333 and 11332), ActorControlSelf 276 10008, 10011,
10010 and 1525 1042784, 1042787, 1042786 for slots 0-2, at level 32 with 1476, 1022 and 1013 HP:
`party_npc::THIRD_MEMBERS` (the Lancer, 45 and 14655, follows the others' numbers, with a guessed 1120 HP). At 32 the
Marauder's combo has a third step (29299, 29300, 29301) and its burst one each of 29764, 29765 and 29768 (58, 58 and 54
uses) between 26230/29303 and 29766; the Thaumaturge's and the Conjurer's kits are the trials' (29314/29315 with 29313;
29323 with 29324 and Cure 29318). Their chatter: the Marauder's 13955 and 13956, the Thaumaturge's 13975 and 13976.
Not taken up: the Marauder's 29295, 29297 (status 87 on itself) and 29777, the Thaumaturge's 29763 and 3269, the
Conjurer's Medica (29320) and Esuna (25043).
In the Navel (Bob's capture 650dbfd8 of 2026-11-06, visit 0.1) the same three (44, 47, 46, the same slot values) came
at level 34 with 1607, 1103 and 1113 HP (`DUTY_LEVELS`; the Lancer's 1220 a guess as before); their lines there are in
[Dungeons](dungeons.md#the-navel).

**The third set's Lancer, and the set at 41 with a tank** (the Stone Vigil, Bob's Chronofoil capture 216f28eb of
2026-11-17: seven runs of 19 to 22 minutes, Bob a Paladin synced to 43): slots 0-2 held the Conjurer, the Thaumaturge
and the Lancer (47, 46, 45: ActorControlSelf 276 10011, 10010, 10009 and 1525 1042787, 1042786, 1042785), at 41 with
1678, 1662 and **1884** HP (the Lancer's was a guess of 1838; BNpcBase 14655, BNpcName 11331 as assumed). How they
fought, the same in all seven:

* **The Lancer** (auto-attack 17059; 2.67 s GCD): 29305, 29306, 29307, 29305, 29308 in turn, from the first again in
  every fight; after the first eight of a fight and every five from then its burst: 26230 at its target 1.07 s after
  the last, 29310 on itself 0.18 s later, then 29769, 29770 and 29771 on its GCD, 29311 on itself 1.07 s after 29770
  and 29767 on itself 1.07 s after 29771 (a round of 21.6 s). 29304 opens a fight it has to run to (100 times). Median
  damage: 81-82 for 29304, 29305, 29306 and 29308, 163 for 29307, 247 for 29769, 329 for 29770, 742 for 29771 (which
  also healed it 85), 8 an auto-attack. 29778 (83 a hit, 62 times) came at no fixed place in the round, in the fights
  where the party was dodging; what calls for it isn't known. `THIRD_MEMBERS`' 45 has this kit at every level (the
  only one recorded for it), `Kit::burst_after_fillers` the count (the clock put the burst a filler early after an
  opener), action scripts `DutySupport_29307`, `_29308`, `_29310`, `_29769`, `_29770`, `_29771` with potencies in
  those proportions to 29305's.
* **The Thaumaturge** (3.07 s between spells): 29314, 29315 and a third spell, 29316 (181 to the others' 90), in
  turn, and 29313 after the first six of a fight and every fourth spell from then (963 in the median, on up to three
  enemies: two 68 times and three 14 times of 330). 29763 came 63 times (91). `Member::kit_from`: its kit from 41
  (`DutySupport_29316`, `_29313`, the latter on its target alone); at 32 and 34 it stays the trials'.
* **Its one big hit**: once a run, as the second boss was pulled (0.1 s after its health first fell, all seven), the
  Thaumaturge used 3269 on itself, 27839 on itself 0.6 s later (saying 13980 with it) and 17995 on the boss 2.1 s
  after that, for 2196-2214: a limit break by the look of it (never at the first or the last boss). Not built (see
  known issues).
* **The Conjurer** (as a tank's healer): Stone 29323 (53), Aero 29324 (555), Cure 29318 (1174 in the median, 486
  times, 470 of them on someone else), Medica 29320 (586 to each of four, 143 times), Esuna 25043 (75 times), its
  auto-attack 17235 for 1. Kawari's still heals with 29317 and casts no Medica or Esuna.
* **Their lines**: at the start one of the three said its own (the Thaumaturge's 14368 twice, the Lancer's 14369
  three times, the Conjurer's 14370 twice); with no tank of theirs the Thaumaturge said 14372 as the first boss fell
  (six runs of seven) and the Lancer 14377 in the last room (all seven); 14374 and 14375 as before. Between fights:
  the Lancer's 13965 and 13966 (15 and 7 times; its `chatter` now), the Thaumaturge's 13978-13979, the Conjurer's
  13985. The script's lines: [Dungeons](dungeons.md#the-stone-vigil).

**The set at 41 with a ranged damage dealer** (the Stone Vigil again, Bob's capture d1f99707 of 2026-11-17: five
runs, a Summoner synced to 43): slots 0-2 held the Marauder, the Conjurer and the Thaumaturge (44, 47, 46) with
2422, 1678 and 1662 HP, as on 2026-11-07.

* **The Marauder** fought as its kit says (29299, 29300, 29301 twice round, then 26230, 29303, 29764, 29765, 29768,
  29766), but **29301 and 29768 had no action script**: in the kit since Brayflox's Longstop, they did nothing in
  Kawari (a third of its fillers). `DutySupport_29301` and `_29768` now, and the third set, which has no timeline to
  scale its hits, hits by the scripts' potencies: 29299 and 29300 100 (54 and 53 in the median), 29301 145 (78.5),
  29764 300 (163), 29765 500 (270), 29768 430 (233); the Conjurer's Aero 1040 to Stone II's 100 (469 and 45). Also
  seen and not built: 25136 on itself (26 times, between its fillers), 29296 (10), 29297 (5, 484 a hit), 29777 (10).
* **The Thaumaturge** as with the Paladin (29314, 29315, 29316 and 29313: 345, 312, 286 and 220 uses), but **no
  17995 and no 27839 in any of the five**: its big hit at the second boss came only in the Paladin's runs. 3269 on
  itself came 39 times here (seven there, once a run).
* **Their lines**: the Marauder's own at the start, 14367 (three runs; the Thaumaturge's 14368 in the other two), its
  14371 and 14376 as before; between fights its 13954-13956, the Thaumaturge's 13978-13979, the Conjurer's 13985 and
  13991.

**`pet_id` is the slot's**, not the member's: 1, 5 and 9 for slots 0, 1 and 2, whoever is in them (the Marauder had 1
in slot 0 in Sastasha, the Conjurer 1 in slot 0 with a tank; the Thaumaturge 5 and the Lancer 9 in slots 1 and 2 in
every run).

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
rows), in party slot order, up to the first zero byte. The words after it aren't members: Bob's client sent
0x6FFF in the third, which Kawari took for members 255 and 111 until 2026-10-23, so a Player NPC joined for them. Upstream already queued the duty from the first word; the fork keeps the members
(`ZoneConnection::duty_support`, cleared by any other registration) and sends them with `JoinContent`. When the player
has loaded into the duty, `party_npc::fill_duty_support` spawns the recorded members they chose and the Duty Finder's
bot filling (`party_bot::fill_duty_party`) takes any role still missing, so a member nobody has recorded yet is a
Player NPC of that role for now.

**The fourth set, at 50** (the Porta Decumana: Connor's Chronofoil capture af05916c of 2026-11-09, visit 0.12, a
healer, and Bob's 9ba7d981 of 2026-11-17, visit 0.18, a damage dealer; Castrum Meridianum and the Praetorium in the
same captures registered the same members under DawnContent 210 and 211, the Porta Decumana 212). Registered as 48,
50, 49 (Connor's) and 48, 51, 50 (Bob's):

| DawnQuestMember | BNpcBase | BNpcName | Class | Level | HP | 276 | 1525 |
|---|---|---|---|---|---|---|---|
| 48 | 14658 | 11334 | Marauder (3) | 50 | 4375 | 10012 | 1042788 |
| 49 | 14659 | 11335 | Lancer (4) | 50 | 3132 | 10013 | 1042789 |
| 50 | 14660 | 11336 | Thaumaturge (7) | 50 | 2770 | 10014 | 1042790 |
| 51 | 14661 | 11337 | Conjurer (6) | 50 | 2796 | 10015 | 1042791 |

`party_npc::FOURTH_MEMBERS`, their level and HP for duty 830 in `DUTY_LEVELS`, their hits and heals scaled by
`FourthSupport*_<base>.json`. The trial's mechanics kept breaking their turns, so their kits are from the stretches
nothing broke:

* **The Marauder**: 29299, 29300, 29301 twice round, then 26230, 29303 on itself, 29764, 29765, **29302** (where the
  third set had 29768: 998 a hit to 29299's 110) and 29766 on itself, 23 s from burst to burst. Seen and not built:
  29296 on itself (status 89, 12.2 s into the first fight in both runs and four more times as a big hit was cast),
  25136 on the second boss (status 1193, four and five times), 29297 on itself once (875 healed, status 87).
* **The Lancer** (Connor's run): six fillers now (29305, 29306, 29307, 29305, 29308, **29309**), then 26230, 29310 on
  itself, 29769, **29774**, 29311 on itself, **29773**, 29767 on itself, 24 s from burst to burst (nine fillers before
  the first: `burst_after_fillers` 9, 6). 29773 healed it 158-167 a hit (not done).
* **The Thaumaturge**: as the third set's at 41 (29314, 29315, 29316, then 29313, 12 s apart). Its 3269 on itself
  (ten times in Bob's run), and once at the very end of Connor's 27841, 25528 (10089 to the boss) and 27838, aren't
  built.
* **The Conjurer** (Bob's run): 29323 every 2.1 s and **29325** every 19 s (1065 a hit); **29318** on the lowest of
  the party (28 times, 1654-1732); and **with most of the party hurt, 29321 (510-541 to everyone round it, and
  status 150) and 29320 (816-865), in turn**, three seconds apart, after what hit them all (15 and 11 times). Kawari:
  `Kit::area_heals`, used by the party bot's healer when three or more of the living party are below its heal's line
  (`party_bot::AREA_HEAL_HURT`, `area_heal_due`; test `a_healer_heals_everyone_when_most_are_hurt`). What status 150
  healed over time wasn't measured: it's only given.

Their lines there: 14404 at the first pull (the Thaumaturge), 14407 or 14408 as the first long cast was stopped (the
Lancer; the Conjurer), 14409 and 13980 in the last part (the Marauder, the Thaumaturge): the duty's
(`party_npc_yell` in `content/w1d5_battle.lua`); 13958 (the Marauder), 13978 and 13979 (the Thaumaturge) are their own
chatter.

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
  were 0x2 (HOSTILE), which Kawari reads as an enemy's; the fork leaves it out, as for bots shown as party members. (Since
  2026-11-05 the server takes sides from battalions, not that flag, [Combat](combat.md); leaving it out is no longer needed
  for that, but stays as it was checked.)
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
| Conjurer | 17235 (1 a hit, on what's close) | 2.1 s (1 s casts) | Stone 29322 (Stone II 29323 in the Bowl) | Aero 29324 every 12 s (first 12 s in) | Cure 29317 on the lowest of the party below 85 % (`Kit::heal`) |

**The Conjurer heals** (the same capture, three duties): Cure (29317, 1.5 s cast) 143 times in the Tam-Tara
Deepcroft, 76 in the Copperbell Mines, 26 in the Bowl, nearly all on the player (the tank: 133 of 143, 67 of 76, all
26); its target was at 73-76 % in the median, never above 85 % (`PartyBot::heal_below`, 85 for it; the made-up
healers' is 60 %). It healed 124 at 16, 126 at 17 and 182 at 20 (the timelines' `players_by_level` and `players`).
Medica (29320) came 1, 5 and 9 times, mostly with two of the party below 80 % (Kawari doesn't cast it: an NPC's area
heal reaches only players); Esuna (25043) once and an instant Cure (29355) three times. Its Aero showed no status (no
damage over time). It fought from 6.9 yalms (the median), Stone hitting for 14 and Aero for 152 in the dungeons (12
and 129 in the Bowl).

**The Marauder's leap** (29291, a pack's opener): it's used from about 10 yalms off, and the Marauder lands beside
the enemy. In Bob's Copperbell Mines capture (bd298651 visit 0.5, 2026-10-25), three times out of three: the hit
(ActionEffect1, a 1.1 s lock), then 0.58 s later ActorControl 239 (`SnapNextMove`) and an ActorMove (speed byte 60)
1.1–2.4 yalms short of the enemy, facing it; it fought from there. Kawari sent the hit only, so the client played the
leap and put him back where the server had him (Bob, 2026-10-26: "he jumps to the enemy and teleports back").
`npc_behavior::NPC_LEAPS` lists the leaps (any NPC's: a kit's, a timeline's or a director's `use_action`), and
`leap_to` lands them, 2 yalms short (`LEAP_SHORT_OF`), on the floor.

Damage to Sastasha's enemies (medians, the timelines' `recorded_amounts`, which scale the action scripts' hits): the
weaponskills and spells 24–27, the auto-attacks 3, the Marauder's 29764 76 and 29765 128, the Lancer's 29772 275,
the Thaumaturge's 29312 277; Sastasha's trash has 905 HP. That's what "fighting like someone of the duty's level"
looks like: steady, with a big hit every few GCDs, unlike a synced player's rotation.

## Getting out of area attacks

Bob's Chronofoil capture of the Tam-Tara Deepcroft with Duty Support (2026-10-24, 3ec702a8, visit 0.3; `questkit
chronofoil --battle`, positions about every 0.31 s): both times the last boss (BNpcBase 76) began 987, its 4.7 s
attack around itself (cast on itself), all three members set off 1.0–1.4 s into the cast (the first samples showing
them moving; at 1.0 s none had), ran straight out at running speed (about 1.9 yalms a sample, 6 a second), stopped at
11.0 yalms from its middle (all three, both times: just outside, not at the room's edge) and waited there, then went
back in 6.1–6.2 s into the cast, 1.3–1.5 s after it had gone off, to fight from where they had (4.8 yalms). Its
self-buff 430 and the rooms' bosses' and trash's other casts (720, 911, 912) moved nobody; at the third room boss's
second 912 the three bunched up on it (0.5–2.5 yalms), which looks like getting out of something around it (912's
shape isn't known here: it hit nobody in any run). The bots and party NPCs dodge alike: see
[Party Bots](party-bots.md#how-it-fights) (`server/dodge.rs`).

The Stone Vigil (Bob's capture 1cefedf4 of 2026-11-07, visit 0.1, the third set's Marauder, Thaumaturge and Conjurer at
41) showed what that first capture couldn't: for a boss's big moves the members went to **set spots**, by role, the
same every time the boss did the same thing from the same place, whether the attack would have caught them or not,
setting off as it began and holding there until it had gone off, still fighting:

* **Swinge** (Chudo-Yudo, a 40-yalm cone after it runs to one of three spots at its room's edge): all ten times
  behind it, by that spot (from (0, 130): the tank (-5, 133), the Thaumaturge (5, 132), the Conjurer (6, 134)), some
  of them running past it from 10-17 yalms in front. When it Swinged again from there at its target, the tank where it
  stood, only the tank moved, to about 40 degrees off its facing: the cone's edge (it caught members up to 28 degrees
  off and none at 40 or more: 60 degrees wide, `resources/data/action_areas.json`).
* **Cauterize** (Isgebind diving across its room, five spots): the same spots for each spot, all eleven times
  (from (0, -271.8): the tank (-11, -253), the Thaumaturge (11, -245), the Conjurer (11, -242)); from the far side ones
  they stayed where they fought (0, -253), (4, -247), (2, -243), or stepped 2 yalms over.
* **The Sheets of Ice** (Isgebind flying, its helpers' circles at each of the party, twice 3 s apart): all three times
  spread out from 1.5 s into the flight to (0, -267), (17.75, -253.44) and (0, -229) (the tank, the Thaumaturge, the
  Conjurer), stepped 7-10 yalms aside as the first circles were cast, and went back to fighting from the second's.
* **Typhoon** (Koshchei's whirlwinds down four lanes, in two pairs): all five times waiting by the boss as they came
  (the tank (47, -80), the others (40-41, -80/-81)), into the middle lane the first pair left alone (at z -77 or
  -83) within a second of it setting off, and into the other middle lane, behind the first pair, as the second went.
* Telegraphed circles on someone (Sonic Storm, a single Sheet of Ice) they left by the shape 1.0-2.4 s in, a short
  way, and were sometimes caught.

The director gives these (`director:party_spots`, `director:party_positions`, `director:sweep_bnpc`; [Party
Bots](party-bots.md#how-it-fights)), and the bots go by them too. Their HP at 41: 2422, 1662 and 1678 (`DUTY_LEVELS`;
the Lancer's 1884 from the runs of 2026-11-17). Their lines there: the Thaumaturge's 14368 4.3 s after the duty began and 14375 as Koshchei
fell, the tank's 14371 as Chudo-Yudo fell and 14376 as the party walked into Isgebind's room, the Conjurer's 14374 as
they came up to Koshchei's; between fights the tank's 13954-13956, the Thaumaturge's 13978 and 13979 and the
Conjurer's 13985 (not the lines they had at 32 in Brayflox's Longstop: not added, see known issues).

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
* **In the Tam-Tara Deepcroft** the Lancer said 14307 and 14309, the Thaumaturge 14311 and the tank 14312 (when, in
  [Dungeons](dungeons.md)); the tank's chatter there was 13953 three times and 13951, so 13953 joined its lines.
* **In the Copperbell Mines** (Bob's capture of 2026-10-25, `bd298651` visit 0.5) the same three at level 17 (476,
  372, 339 HP): the Thaumaturge said 14315 and 14322, the tank 14318, the Lancer 14320 and 14324 (when, in
  [Dungeons](dungeons.md#the-copperbell-mines)); their chatter was lines they already had.
* **In the Thousand Maws of Toto-Rak** (Bob's capture of 2026-10-30, `caa5e908` visit 0.15) the trials' Marauder,
  Thaumaturge and Conjurer (40, 42, 43) at level 24 (842, 583, 588 HP; the Lancer's 639 is a guess): the tank said
  14331 and 14337, the Conjurer 14336 (when, in [Dungeons](dungeons.md#the-thousand-maws-of-toto-rak)); the tank's
  chatter 13951, 13952 and 13953, the Thaumaturge's 13972 seven times, which are now the trials' members' lines too.
* **In Haukke Manor** (Bob's capture of 2026-11-02, `5d9c7d7f` visit 0.2) the same three at level 28 (1026, 710, 717
  HP; the Lancer's 779 is a guess): the tank said 14339 and 14345, the Conjurer 14344 and 14349, the Thaumaturge 14347
  (when, in [Dungeons](dungeons.md#haukke-manor)); the tank's chatter 13952 three times, 13953 four times, 13951 and
  13961 (now among the trials' Marauder's lines too), the Thaumaturge's 13972.
* **The duty's story lines**, from the director (`director:party_npc_yell(role, line, delay, other_role,
  other_line)`, the role "tank", "healer", "melee" or "ranged"; nobody of that role, no line, or when no party NPC
  plays it at all (the player does), `other_line` from `other_role`; line 0 is nothing): in Sastasha (`content/s1d1_re.lua`) the Lancer's 14296
  43.6 s after the duty began, the Marauder's 14298 as Chopper came, the Lancer's 14300 2 s after the first captain's
  last line, the Thaumaturge's 14302 2 s after the second's, the Marauder's 14303 15.5 s after Denn's cutscene began.
* **With the Conjurer and no Lancer in Sastasha** (an arcanist's party, Bob's capture cc62ad94): the Marauder said
  14294 at the start where the Lancer says 14296 (23.4 s after the duty began on retail; Kawari keeps 43.6), the
  Conjurer 14299 as Chopper came (where the Marauder says 14298; the Marauder was there and didn't) and 14301 2 s
  after the first captain's last line (the Lancer's 14300). The script asks for the Lancer's and the Conjurer's first
  and falls back on the others' (`s1d1_re`'s `PARTY_NPC_LINE_START_TANK`, `_CHOPPER_HEALER`, `_CAPTAIN1_HEALER`).
* **With a tank** (Bob's capture c99fe374) the tank's lines went to another member as lines of their own, at the same
  moment: the Tam-Tara Deepcroft's last, 14313 from the Conjurer 26.8 s after the last seal's cutscene (the tank's
  14312 at 26.5 s); the Copperbell Mines' first boss's, 14319 from the Thaumaturge 6.2 s after it appeared (14318 at
  6.8). The other lines came from the same members as before. In the Bowl of Embers the Conjurer said its own at the
  first pull (14328, where the Thaumaturge had said 14326) and 0.2 s before each Nail (14330, the Thaumaturge's 13693),
  and nobody said 13981 as Ifrit fell; the Lancer's 14327 at the second pull was as before. The Conjurer had no
  chatter of its own in any of them.

## Not done yet

* Recorded: the Marauder, Lancer, Thaumaturge and Conjurer of the three dungeons and the Bowl, and the third set in Brayflox's Longstop (at 32). Other duties'
  members are Player NPCs until a run with them is recorded. Sastasha's lines for the tank, with a player tank, aren't
  recorded (no line then).
* The Conjurer's Medica, Esuna and instant Cure aren't cast, and the third set's still heals with 29317 (retail's:
  29318).
* The fourth set's kits are from the Porta Decumana only; in Castrum Meridianum and the Praetorium its members had
  the same level and HP (50; 4375, 3132, 2770, 2796 in every run), which are their own (`FOURTH_MEMBERS`). Its Marauder's
  29296, 25136 and 29297, its Thaumaturge's 3269 and its Lancer's heal from 29773 aren't built; the earlier sets'
  Conjurers still have no area heal (29320 and 29321 weren't seen below 50).
* The Thaumaturge's 17995 (once a run as the Stone Vigil's second boss was pulled, with a Paladin; never with a
  Summoner) isn't built, nor the Marauder's 25136, 29296, 29297 and 29777, the Lancer's 29778
  isn't used (what calls for it isn't known) and its 29771 doesn't heal it. The third set's Lancer has its kit of 41
  at 32 and 34 too (not recorded there), with guessed HP.
* What 26230, 29303, 29766, 29311, 29767, 25082 and 25083 do (statuses, enmity) isn't known: they're only their
  animation. The Lancer's 29772 healed it 30 HP on retail; not here.
* Retail's DawnContent, DawnQuestMember and DawnMemberUIParam sheets aren't read: the members are recorded numbers.
* Untested in game.
