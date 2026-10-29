# The Gold Saucer

The Manderville Gold Saucer: how it's unlocked, its lotteries and arcade machines, and what isn't there yet. From
Bob's Chronofoil capture **deaf7e28** (2026-10-29, 51 minutes: It Could Happen to You through So You Want to Be a Jockey,
three Mini Cactpot tickets, three Jumbo Cactpot tickets, 50 arcade games, The Slice Is Right and Air Force One), his
quest dumps of 434-437, 488 and 489, and the public wikis where the capture didn't show something (marked so).

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
* At every zone change retail sends **912** `[1378000800]` (`SetWeeklyLotOffsetTime`, when the Jumbo Cactpot's weeks
  begin), **914** (`SetGoldSaucerFlags`: 8 once a Mini Cactpot ticket has been played that day, 16 once a Jumbo Cactpot
  ticket was bought that week: 0, then 8, then 24 in the capture; Bob's older character had 126, the other bits unknown)
  and **911** `[21]` (unknown; 0 on some characters, not sent by Kawari.)

## The Mini Cactpot

LotteryDaily (handler 0x250000), the broker in the Entrance Square. Three tickets a day at 10 MGP; the day turns at
15:00 UTC (day 4813 was 2026-10-29: the days are counted from 2013-08-25 15:00 UTC.)

1. The talk: ActorControlSelf 109 (`DirectorEvent`) `[28, 1 if today's tickets are used]` and `[30, tickets left, 10]`,
   then scene 0 (0x2000) with `[0x11, 10]` the first time ever (the explanation), `[0x13, 10, MGP, day, tickets left]`
   afterwards, `[0x12]` with no tickets left.
2. Buying is the scene's yield 5: the MGP, then **LotteryDailyBoard** (fork opcode 326, 136 bytes): the handler, 19 and
   23, the day, the cell already showing (row, column, number) and what each line's sum pays.
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

**Not done**: none of the GATEs can be played. Upstream's GoldSaucer director sends one hardcoded GATE (Cliffhanger) when
the client asks (client trigger 808 `[1]`, `[0]` to a GoldSaucer director); retail answered with 109 `[9, GFATE row,
type]`, `[6, its pop range, the end]` and `[11, phase]`, and moved the phase on as it ran (`[11, 3]` announced, 4-10 as it
went, 1 when over). From the capture:

* **The Slice Is Right** (GFATE 77, GFateType 8, director 0x800A02A9): talking to its entrance, scene 0 `[503, 0, MGP]`,
  a yield `[1]` to join, then 109 `[5, 1, start, 120]`; the gold piles are EObjs whose events give 1000 a pile (109
  `[33, score, ...]`, EventLogMessage 4728); the end 109 `[8, 1, 2]` and 3,000 MGP (528, 910 `[16, 3000]`.)
* **Air Force One** (director 0x800A02AA) sends you through ContentEntry 117 to InstanceContent 62005 (ContentFinderCondition
  1083, territory 1335): the client's shooting is client packet 796 (568 in 2.3 minutes), the result Unknown318 and scene 11
  of the InstanceContent director, 3,000 MGP.
* The schedule since 7.4 (the patch notes): a GATE every 20 minutes, one of three for its slot.

## Known gaps

The GATEs; chocobo racing; Triple Triad (437, Triple Triad Trial, isn't scripted); the Jumbo Cactpot's claim scene and
participant bonus; the Gold Saucer window's information (`GoldSaucerInformation` is still upstream's zeros);
achievements for the games (retail completed 1125 after the first Mini Cactpot win, 1115 after Air Force One); Gold
Saucer bonuses (the VIP card and the Free Company's Jackpot actions); The Finer Miner's Morbol and Titan, Monster Toss
and Crystal Tower Striker weren't played on retail.
