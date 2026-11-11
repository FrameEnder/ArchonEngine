# The Gold Saucer

The Manderville Gold Saucer: how it's unlocked, its lotteries, arcade machines and GATEs, and what isn't there yet. From
Bob's Chronofoil capture **deaf7e28** (2026-10-29, 51 minutes: It Could Happen to You through So You Want to Be a Jockey,
three Mini Cactpot tickets, three Jumbo Cactpot tickets, 50 arcade games, The Slice Is Right and Air Force One) and
**2c98f9e8** (the same evening, an hour: Air Force One twice, Leap of Faith, Any Way the Wind Blows, the GATE Keepers),
his archive's Gold Saucer sessions, his quest dumps of 434-437, 488 and 489, and the public wikis where the captures
didn't show something (marked so).

Where it lives: the quests in `resources/scripts/events/quest/004/`; the games in `servers/world/src/events/gold_saucer.rs`
(the events) and `servers/world/src/gold_saucer.rs` (the rules, the numbers and what the character keeps,
`unlock.gold_saucer`); the flight there in `resources/scripts/events/generic/Warp.lua`
([Travel](travel.md#warps-and-their-conditions)).

## Getting there

* **It Could Happen to You** (434) opens it: the well-heeled youth in Ul'dah gives the Golden Airship Ticket (ITEM0 in
  UI8BH), Elyenora takes it (an event-mode NpcTrade), asks to go (a yes/no) and the cutscene flies you to the Gold
  Saucer (POPRANGE0 in territory 144; retail: the zone change, then FinishQuest, then EventFinish.)
* Afterwards the Ul'dahn ticketer's **Seek Passage to the Gold Saucer** (Warp 108) goes there: its WarpCondition (29)
  asks for 434 done; before, the warp's ConditionFailEvent (DefaultTalk 591990) answers.
* **World of Wonders** (435) is the tour: one stop a sequence (the attendants, the Triple Triad trader, two GATE
  Keepers, Veleda), each a cutscene (0xF8402EF3) ending in a notice and the next sequence; the attendant rewards it.
* The **Chocobo Square** lifts (Warps 105 and 106) play their ConditionSuccessEvent first (DefaultTalks 591975, 591981,
  nested, type 7), then the yes/no, as every warp with one does now.
* **So You Want to Be a Jockey** (436): the race chocobo registrar offers it (straight to 255); Katering rewards it with
  one of two items (the reward scene wasn't captured.) Chocobo racing itself isn't there.
* **Scratch It Rich** (488) and **Hitting the Cactpot** (489) are offered and completed in one scene by the Mini and
  Jumbo Cactpot brokers (a yes/no and the reward; retail sent no AcceptQuest or UpdateQuest, only FinishQuest.)

## MGP

The currency's slot 9 (item 29). Retail's packets around it:

* **Won**: ActorControlSelf **528** `[4765, the player's actor ID, MGP]` (LogMessage 4765, "You obtain ... MGP"), then
  the slot; after an arcade machine or a GATE also **910** `[source, MGP]` (8 Cuff-a-Cur, 10 The Moogle's Paw, 13 The
  Finer Miner, 14 Out on a Limb, 16 a GATE; Monster Toss's and Crystal Tower Striker's weren't seen, Kawari sends 11 and
  9.) The Mini Cactpot's win has an InventoryActionAck (`-1`, 6) between the log line and the slot.
* **Bought with gil** (the Gold Saucer attendant, CustomTalk 240, `CmnGscGilToCoin`): 10 gil an MGP (500 for 5,000);
  retail's log line is **527** `[4765, 0, MGP]`, then 913 `[0]`. Kawari's script takes the gil and gives the MGP, without
  the log line.
* The cap is 9,999,999 (the public wikis).
* **MGP Vouchers** (ItemAction type 3800, the MGP in Data[0]: Scratch It Rich's MGP Voucher, item 10131, is 100): used,
  one fewer, the log line (528) and the MGP (`redeem_mgp_voucher`, `items/Items.lua`). Not recorded on retail; before,
  using one did nothing (Bob's bundle 20261030-021002.)
* **Achievements** (Type 1 counters, the Achievement sheet's Keys): MGP won at the attractions, 356 (What Happens in the
  Saucer, 1,000 to 10,000,000: every MGP won, `change_mgp`); GATEs completed, 363 (Enemy at the GATE, 1 to 100: Air Force
  One); the Jumbo Cactpot's MGP, 365 (You Could Already Be a Winner); the Mini Cactpot's, 366 (Dream Small, 1 to
  1,000,000.) Retail completed Dream Small I with the first Mini Cactpot win, Enemy at the GATE I with the first Air
  Force One and What Happens in the Saucer II as Leap of Faith paid. Kawari counts them from now on, so a character's
  earlier play doesn't count.
* At every zone change retail sends **912** `[1378000800]` (`SetWeeklyLotOffsetTime`, when the Jumbo Cactpot's weeks
  begin), **914** (`SetGoldSaucerFlags`: 8 once a Mini Cactpot ticket has been played that day, 16 once a Jumbo Cactpot
  ticket was bought that week: 0, then 8, then 24 in the capture; Bob's older character had 126, the other bits unknown)
  and **911** `[21]` (unknown; 0 on some characters, not sent by Kawari.)

## The Mini Cactpot

LotteryDaily (handler 0x250000), the broker in the Entrance Square. Three tickets a day at 10 MGP; the day turns at
15:00 UTC (day 4813 was 2026-10-29: the days are counted from 2013-08-25 15:00 UTC.)

1. The talk: ActorControlSelf 109 (`DirectorEvent`) `[28, 1 if today's tickets are used]` and `[30, tickets left, 10]`,
   then scene 0 (0x2000) with `[0x11, 10]` the first time ever (the explanation), `[0x13, 10, MGP, day, tickets left]`
   afterwards, `[0x12]` with no tickets left. The first number looks like flags on 0x10 (1 a greeting, 2 the game
   explained already). The first talk ever is the explanation, and it ends without a ticket: retail's player (no MGP
   then) came back for one. Kawari sent the MGP, the day and the tickets left after 0x11 for a while (2026-10-29,
   reading Bob's bundle 20261030-020833 as the client not seeing his MGP); in his bundle 20261030-102912, with 500 MGP,
   the first talk ended all the same and the second sold the ticket, so it's retail's `[0x11, 10]` again.
2. Buying is the scene's yield 5: the MGP, then **LotteryDailyBoard** (fork opcode 326, 136 bytes): the handler, two
   zero bytes, 19 and 23 (bytes 6 and 7), the day, the cell already showing (row, column, number) and what each line's
   sum pays. Kawari had the 19 and 23 before the zeros until 2026-10-30, and the client never showed the ticket: no
   scratch came, and the talk had to be ended with `!finishevent` (Bob's bundle 20261030-102912).
3. Each of the three scratches is client trigger 808 `[8, row, column]`, answered with 109 `[2, row, column, number]`.
4. The line is 808 `[9, line]`: 109 `[3, row, column, number]` for all nine cells, the log line (528), the
   InventoryActionAck, the MGP, 914 and 109 `[4, MGP, sum]`.
5. The scene returns `[0]` to buy another (`[30, ...]` and scene 0 with 0x10) or nothing to leave.

The lines, as the client sends them: 0-2 the rows, 3-5 the columns (each of the capture's three tickets agreed), 6 and
7 the diagonals from the top left and the top right (not played; the order MiniCactpotSolver uses. The client's radio
buttons are in another order: diagonal, three columns, diagonal, three rows.) The payouts by sum, 6 to 24: 10000, 36,
720, 360, 80, 252, 108, 72, 54, 180, 72, 180, 119, 36, 306, 1080, 144, 1800, 3600 (retail's, and the wikis'.)

## The Jumbo Cactpot

LotteryWeekly (0x260000), the broker in the Cactpot Board. Three tickets a week at 100, 150 and 200 MGP, numbers 0000-9999;
the week turns at the offset time 912 gives, Sunday 02:00 UTC (North America's drawing; week 686 was 2026-10-29.)

* The talk: 109 `[29]`, then scene 9 (0x2000, EventScene32) with 21 parameters: 0, the week, the three tickets (10000
  for none), 1820 and ten numbers about the participants and their bonus that aren't understood (Kawari sends last
  week's number, then retail's ten), and the five prizes (retail's 1,008,000, 200,000, 52,500, 6,550, 1,160.)
* A ticket is the scene's yield 6 with the number: the MGP, 914, ActorControlSelf 517 `[4735, number, price]`
  (`LogMessageValues`), then resume 20 `[1, 0, the three tickets]`.
* The drawing: retail draws one number a data centre; Kawari draws one a week from the week alone
  (`jumbo_cactpot_number`), the same for everyone. Digits are matched from the last one: all four 1,008,000, three
  200,000, two 50,000, one 5,000, none 1,000 (the base prizes since patch 4.4; retail adds a participant bonus Kawari has
  no players for.)
* **Claiming isn't recorded**: retail hands the tickets in one at a time at the broker, within a week, with a bonus for
  coming within the hour. Kawari pays a drawn week's tickets the next time the player talks to the broker, each with
  the MGP log line.

## Arcade machines

GoldSaucerArcadeMachine (0x24xxxx; the row is the event's ID): 1 Monster Toss, 2 The Moogle's Paw, 3 Crystal Tower
Striker, 4 Cuff-a-Cur, 5 The Finer Miner, 6 Out on a Limb. Each is scene 14 (0x2001); the coin is yield 7 (1 MGP),
answered with resume 21 `[1, deadline]` (`[1, deadline, 2000]` for 5 and 6; the deadline is now plus the row's
`Timeout`: 15, 30 or 60 s.)

* **1-4** send the result as yield 8 `[tier]` (0 poor, 1 good, 2 great, 3 excellent): 109 `[1, the sheet's MGP]`,
  resume 22 `[1, tier, the sheet's MGP twice, the tier's text, 1]`. The MGP comes after the event finishes (528,
  910.) The result screen shows the sheet's numbers (10, 15, 20) but retail paid The Moogle's Paw 10 and 25 and Cuff-a-Cur
  10, 15 and 25, as their payout texts say; Monster Toss (20/30/50) and Crystal Tower Striker (10/15/25) are paid by
  their texts too (`arcade_payout`.)
* **5 and 6 are staged.** The first swing (yield 9 `[choice]`: 0 Cactuar, 1 Morbol, 2 Titan) sets the game up, resume
  23 (EventResume16, fork opcode 529) `[1, 10, swings, gauge ms, deadline, 5, 2000, payout, 60]`: Out on a Limb 10
  swings and a gauge of 750, 375 or 250 ms; The Finer Miner 6, 5 or 4 swings at 750 ms. Each swing is yield 10
  `[position 0-99]`: its tier is the distance from a hidden spot by the row's ranges (Out on a Limb 21-100 poor, 11-20
  good, 3-10 great, 0-2 excellent; The Finer Miner 15-100, 8-14, 2-7, 0-1), which takes 0, 1, 4 or 10 off the 10 the tree
  or rock has; resume 24 `[1, tier, left, text, icon, over, a later round, payout, next payout]` (and 109 `[1, left]` when
  the round is over). Emptying it wins the round; running out of swings loses everything. Doubling down is yield 11:
  resume 25 `[1, 10, deadline, gauge, payout, 1 for The Finer Miner, seconds left]`, a new spot, the swings back. Up to
  six rounds; the payouts (Out on a Limb 50/100/150/400/500/600 for Cactuar, 60/125/185/500/625/750 Morbol,
  85/175/260/700/875/1050 Titan; The Finer Miner 35/85/125/175/210/250, 50/150/360/735/1425/2625, 60/200/600/1600/4000/9600)
  are the wikis', which retail's first two rounds match. Stopping pays the round won, after the event finishes.
* The icons in resume 24 (Out on a Limb 4790-4793, The Finer Miner 4785-4788) are retail's numbers; where they come
  from isn't known.

## GATEs

From Bob's Chronofoil captures deaf7e28 and 2c98f9e8 (2026-10-29: The Slice Is Right, two Air Force Ones, Leap of
Faith, Any Way the Wind Blows and the GATE Keepers between them) and his archive (dd09ac61, ea704111, 61a9a4f3: five
more), the public wikis for the schedule and the rewards. Where it lives: `servers/world/src/gate.rs` (the schedule,
what each type is), `zone_connection/gate.rs` (what the client in the Gold Saucer is told), `events/gold_saucer.rs`
(`GateEventHandler`, the entrances), `events/content.rs` (`ContentEntryEventHandler`), `zone_connection/ride_shooting.rs`
(Air Force One) and `events/custom/002/CmnGscGATENotice_00242.lua` (the GATE Keepers.)

**Kawari plays Air Force One only.** Its schedule follows from the time alone, so every connection works out the same
GATE without asking the server: in each slot the Air Force One that slot's minute has (the Cieldalaes' courses at :00 and
:40, the Gold Saucer's at :20), its course picked from the slot. Untested in game.

### The schedule

* A GATE every 20 minutes, a second past the hour, :20 and :40 (retail's NpcYells at 17:40:01, 18:00:01, 18:20:01,
  18:40:01.) Since 7.4 (the wikis): at :00 Air Force One (the Cieldalaes), Cliffhanger or Leap of Faith; at :20 Any Way
  the Wind Blows, The Slice Is Right or Air Force One (the Gold Saucer); at :40 The Slice Is Right, Leap of Faith or Air
  Force One (the Cieldalaes.) Retail played Air Force One rows 96 and 98 at :00 and :40; the :20 one wasn't seen.
* GFATE rows: Cliffhanger type 1, Any Way the Wind Blows 5 (rows 5, 19, 55-59, 61), Leap of Faith 6 (65-73, 78-95), Air
  Force One 7 (74-76 the Gold Saucer's, 96-98 the Cieldalaes'), The Slice Is Right 8 (77); the location (1 Wonder Square
  East, 2 Event Square, 3 Round Square) is the row's last column, the entrance's pop range its first GFATEParams.
* A row's **place** among its type's rows (counted from 1) is the row of its type's own sheet: GFateRideShooting for Air
  Force One (row 98 is 6), GFateClimbing2 for Leap of Faith (80 is 12). Any Way the Wind Blows' 19 was 2, but The Slice
  Is Right's 77 was 8, not 1.
* The **director** is GoldSaucer (0x800A) with an ID retail counts up by one a GATE (680-685 from 17:00 to 18:40, other
  numbers on other days); Kawari's comes from the slot.

| Type | Registration | InitDirector's content ID | Entrance's first parameter | Instanced |
|---|---|---|---|---|
| Any Way the Wind Blows (5) | 120 s | 6 | 89 | no |
| Leap of Faith (6) | 480 s | 9 | 493 | yes (PublicContent) |
| Air Force One (7) | 600 s | 10 | 498, 501 once played | yes (InstanceContent) |
| The Slice Is Right (8) | 120 s | 11 | 503 | no |

### What the client in the Gold Saucer is told

* **It begins**: to everyone in territory 144, NpcYell 2450 `[location, type]`, ActorControlSelf 100 (InitDirector) with
  the director and its content ID, and 109 `[11, 3]` (the phase: 3 registration, 4 starting, 5 under way; The Slice Is
  Right used 6-10 as it went; 1 over.) Coming into the zone while it's on gives the InitDirector alone, right after
  ZoneInit. **Leaving it** while a GATE is on, retail sent ActorControlSelf 101 (TerminateDirector) right after the
  PrepareZoning, every time (12 departures in eight captures); Kawari does since 2026-11-11 (`gate::leaves_gold_saucer`,
  the connection's `leave_gate`, as it sends the PrepareZoning).
* **The client asks** (client trigger 808 to the director, `[1]` then `[0]`), answered after the second with 109 `[9,
  row, place]`, `[6, the entrance's pop range, when registration ends]`, once the player has played it `[8, 1, 0]` and
  `[5, 1, the start, the registration window]`, and `[11, phase]`.
* **An instanced GATE is over when registration ends**: 109 `[11, 1]` and ActorControlSelf 101. The in-zone ones went on:
  Any Way the Wind Blows sent `[7, its game's start]`, `[11, 4]`, `[11, 5]`, NpcYells 2452 and 2462-2463 (from
  ENpcResident 1010476) and `[26, round]` each round, then `[8, 1, 1]`, `[11, 1]`, 7,500 MGP (528, 910 `[6, 7500]`) and
  WTFungah I (1118), 101 a second later and NpcYell 2451 ten seconds after.

### The entrance

The entrance is an event NPC the server spawns as the GATE begins and removes as it's over (`server/gate.rs`), its
handler the GATE's director, so talking to it starts an event on the director (`GateEventHandler`). Its layout ID is the
GFATE row's first LGBPopRange, which isn't a pop range at all: Air Force One's and Leap of Faith's 7773571 and 7621822
(Sabotender Guia, ENpcResident 1016306, at (-57.86, 3.26, -65.40) in the Round Square), Any Way the Wind Blows' 5126495
(1010476), The Slice Is Right's 8138065 (1031796), both in the Event Square. Retail's spawns had the 0x40000 display flag
as the GATE began and none for whoever came by later. They're in `dropins/w1eb_planmap.json`, `nonpop`: upstream's
drop-in had Air Force One's out all the time, with no handler, so she couldn't be talked to (Bob's bundle
20261030-010154.)

* Scene 0 (0x2000) `[the type's number, 0, MGP]` (the table above; not the time left.) Joining is its yield `[1]`.
* **In the zone**: 109 `[5, 1, start, window]`, the event finishes and the player is moved onto the arena (PrepareZoning,
  warp type 2; Kawari answers without the move, none of these being played.)
* **Instanced**: the scene is resumed with nothing; when it returns, the GATE's ContentEntry event (GFateRideShooting's or
  GFateClimbing2's row: Air Force One 117, Leap of Faith 91) plays, nested, its scene 0 (0x2000, nothing), which returns
  `[2, 0]` to go (`ContentEntryEventHandler`.) The duty goes into the Duty Finder: ContentFinderUpdate, both events
  finish, ContentFinderFound, and the player enters from its window. The duty is the InstanceContent of type 15
  (RideShooting) whose ContentRoute is that row (Air Force One 62000-62005, ContentFinderCondition 640-642 in
  territory 832 for the Gold Saucer's, 1081-1083 in 1335 for the Cieldalaes'), or the PublicContent of type 5 whose
  AdditionalData is (Leap of Faith 7-42: row 12 is PublicContent 19, ContentFinderCondition 707, territory 899.)
* Each GATE can be played once: Kawari remembers it for the connection from the ContentEntry's answer.

### The GATE Keepers

CustomTalk 242 (`CmnGscGATENotice`), flags 0x2000: scene 0 `[type, location, row]` while a GATE takes registrations,
scene 1 with the same while one is under way that can't be joined any more, scene 2 `[the minute the next begins (18:00:00
for 18:00:01's), type, location, row]` otherwise. Each returns `[1, row]` to go to the GATE's entrance: the event finishes
and the player is moved to the entrance (PrepareZoning with warp type 2, then ActorSetPos 4.5 yalms in front of its
NPC, facing her: retail's at (-53.36, 3.26, -65.00), facing -1.82). `player:move_to_pop_range` takes a warp type, and
given an NPC's layout ID instead of a pop range it puts the player in front of that NPC (`change_zone_warp_to_pop_range`;
before, the warp found no pop range and left the player at the aetheryte.)

### The screens and the squares

As a GATE begins, every one of the Gold Saucer's GATE screens (EObj 2005037: layouts 5485250-5485253, 5668781, 5668782)
gets ActorControl 413 `[4, 8]` from itself and shows the GATE; as its registration ends, `[1, 2]` (the in-zone ones'
games go on after it). The Round Square's set piece (EObj 2007457, 7833432) plays Air Force One's (`[16, 32]` at the
start, `[64, 128]` at the end) or Leap of Faith's (`[1, 2]`, `[4, 8]`); the Event Square's (EObj 2005129, 5614704) went
`[256, 512]` and `[4, 8]` by turns through Any Way the Wind Blows. 413's two numbers look like a pair of the EObj's
shared group timeline states (an object spawned later had the first as its SpawnObject `args1`, which the server keeps.)
From all of the captures, whichever screens were near the player.

### Air Force One

InstanceContent 62005 (`ride2_003`, ContentFinderCondition 1083, territory 1335), twice on retail (scores 4980 and 4930):

* In the duty the client asks its course (808 `[0]` to the InstanceContent director): 109 `[0, 6]`, its
  GFateRideShooting row (`server/director.rs`.) Then DirectorVars, and the opening (EnterTerritory, scene 1) with flags
  0x0C002001 and `[0, 0, 0, 5, 1793225644, 0, 0, 0, 0, 0, 3600, 262152]`: the fifth parameter was the same in both runs,
  an hour apart (Kawari's other duties send 14400 there.) Its end: 109 `[0x40000001, 3600]` and `[0x80000004, 3599]`.
* The client flies and shoots on its own (2.3 minutes); as the ride ends it sends **RideShootingScore** (fork client
  opcode 796, 16 bytes): 6, the score, 0-5, then the silvers, bronzes and ruby bombs hit (14, 42, 1), and again and again
  until the player leaves (568 and 626 times.)
* The answer (the first only): **RideShootingEnd** (fork opcode 318, the same 40 bytes both times), 109 `[0x40000002, 0]`,
  `[0x40000007, 0]`, `[0x40000007, 1]`, a GameProgress event (argument 7) with scene 11 (0x0C002001) `[0, score, rank,
  300, golds, 100, silvers, 50, bronzes, -20, ruby bombs, MGP]` (the worths are RideShootingTargetType's; the golds,
  5 both times, are what's left of the score), the MGP (528, the InventoryActionAck, the slot; no 910) and Enemy at the
  GATE I (1115).
* Ranks (the wikis): Perfect (5,000, every target) 4,000 MGP, S (4,000) 3,000, A (1,500) 1,500, B (500) 500, C 100. Both
  of retail's were S, with 1 as the rank: Kawari counts them 0 (Perfect) to 4 (C.)
* The player leaves from the result (client trigger 819): its event finishes first, then the way out.

### The Slice Is Right (not played)

GFATE 77, director 0x800A02A9 (deaf7e28, 17:20): scene 0 `[503, 0, MGP]`, the yield `[1]`, then 109 `[5, 1, start,
120]`; the gold piles are EObjs whose events give 1,000 a pile (109 `[33, score, ...]`, EventLogMessage 4728); the end
109 `[8, 1, 2]` and 3,000 MGP (528, 910 `[16, 3000]`.)

### Leap of Faith (not played)

PublicContent 19 (`climb2_012`, ContentFinderCondition 707, director 0x80040013, territory 899), once on retail: the
opening (scene 1, 0x2000, `[0, 0, 0, 5, 600, 0]`), 109 `[0x40000001, 600]`; then `[0, n]` for each of 0-9 in no order over
the run (perhaps the other players reaching the goal), One Small Leap (2126) as the player reached it, `[1, 8230738]`; an
object's event (0xF0304) and the director's GameProgress scene 4 (argument 2, 0x08002001) `[0, 2000, 0, 500, 300, 100]`
with 4,000 MGP and What Happens in the Saucer II. Its platforms and cactuars need more than the captures show.

## Known gaps

Every GATE but Air Force One (Cliffhanger, Any Way the Wind Blows, Leap of Faith, The Slice Is Right); chocobo racing; Triple Triad (437, Triple Triad Trial, isn't scripted); the Jumbo Cactpot's claim scene and
participant bonus; the Gold Saucer window's information (`GoldSaucerInformation` is still upstream's zeros);
Gold Saucer bonuses (the VIP card and the Free Company's Jackpot actions); The Finer Miner's Morbol and Titan, Monster Toss
and Crystal Tower Striker weren't played on retail.
