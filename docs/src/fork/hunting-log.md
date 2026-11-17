# Hunting Log

Each class (and Grand Company) has a log of enemies to defeat: ranks of ten entries, each entry up to four kinds of
enemy and how many of each, with a reward for finishing it and a bonus for finishing a rank. Until 2026-10-27 Kawari
had none of it (Bob's bundle of 2026-10-27, a Conjurer killing ground squirrels and little ladybugs); from then the
classes' entries counted. Since 2026-11-14 the whole of it is in, from retail and the game's sheets: every class's log
through its five ranks, the Grand Companies' logs for seals, rank bonuses, and which ranks the player has looked at.

Code: `servers/world/src/hunting_log.rs` (the rules, tested against retail's numbers) and
`servers/world/src/zone_connection/hunting_log.rs` (what the client is sent); game data in `GameData::monster_note`,
`hunting_log_index`, `hunting_log_class`, `hunting_log_rank_reward`, `company_hunting_log_ranks`. Progress is kept in
`unlock.hunting_log` (migration `update-2026-10-27-2_hunting_log`; a log's `seen` since 2026-11-14, absent in older
saves).

## What retail does

From every Chronofoil capture at hand (Bob's and Connor's, 2026-10-18 to 2026-11-16: 93 logins of six characters, 159
targets counted, 45 entries finished and four ranks, by a Conjurer and a Gladiator), the game's sheets, and the
community's wiki where neither shows it (said where). Most of it is Bob's capture 216274f3 (2026-11-16, 2.7 hours on
his White Mage of 48: the Conjurer's log from the five entries its first rank had left to the fifth rank's first,
132 counts and 35 entries, and one entry of the Twin Adder's).

**The logs.** Twelve, by ClassJob's `MonsterNote` column: 0 Gladiator, 1 Pugilist, 2 Marauder, 3 Lancer, 4 Archer,
5 Conjurer, 6 Thaumaturge, 7 Arcanist, 8 the Maelstrom, 9 the Twin Adder, 10 the Immortal Flames, 11 Rogue (jobs share
their class's). An entry is a MonsterNote row, whose targets are MonsterNoteTarget rows (their BNpcName) with a count:

| Log | Rows | Ranks | An entry's `Reward` | A finished rank gives |
|---|---|---|---|---|
| a class's | ClassJob × 10,000 + rank × 10 + entry + 1 (60002: the Conjurer's rank 1 entry 2; 260001 Arcanist 01, 290001 Rogue 01) | 5 | EXP (60002: 180, as retail gave; 75 to 21,500) | EXP: 2,500, 10,000, 20,000, 30,000, 40,000 (ParamGrow's `HuntingLogExpReward`, rows 1, 10, 20, 30, 40; the wiki's rank bonuses) |
| a company's | company × 1,000,000 + the same (1000001 Maelstrom 01) | 3 (30 of its 50 rows are used) | seals (240 to 1,619) | seals: 1,181, 2,141, 3,603 (ParamGrow's `MonsterNoteSeals`, rows 1, 10, 20) |

**HuntingLog** (server opcode 385 on 2026.10.16, 980 on 2026.11.03, 64 bytes, `fork_opcodes.yml`; the client's
MonsterNoteManager rank info), one log:

| Bytes | Field |
|---|---|
| 0-3 | -1 at login, 0 after a kill |
| 4 | the ranks finished, so the rank being worked on (0-4; 5 when a class's is finished, 3 a company's) |
| 5 | the log's index (the client's handler reads it first) |
| 6-45 | kills of each target of each of the rank's ten entries (entry * 4 + target) |
| 48-55 | five bits an entry: one per target done, then one for the entry (60002 done: 0x220; then 60001: 0x231) |
| 56-59 | a bit per rank **the player hasn't looked at**: 0xFFFFFFFF until the log is opened, then a bit cleared for each rank viewed. The Rogue's log has only the low byte (0xFF, 0xFE). |

All twelve are sent at login. A fresh Rogue's log has every entry's third and fourth targets counted 0xFF and done
(both characters, every login); the others are zeros.

**Bytes 56-59 aren't whether a log is open.** Until 2026-11-14 Kawari took them for "a bit cleared per rank open" and
made them from the rank. What retail has:

* kills counted and entries finished with them 0xFFFFFFFF: Bob's Conjurer as a Gladiator of level 1 to 4 (10002 and
  10004 done), Connor's Conjurer to level 50 (60005, 60006);
* Bob's paladin of 45 and arcanist of 47, their class quests done, at 0xFFFFFFFF with nothing counted (they took their
  first class quest late, and at that level meet none of the first rank's enemies);
* the client sends **ClientTrigger 401** `[log, rank]` when the player looks at a log: `[5, 0]` in capture b23b91ef,
  after which the Conjurer's read 0xFFFFFFFE (0xFFFFFFFF at login), and `[10, 0]` in c99fe374, after which the
  Immortal Flames' did, the classes' still 0xFFFFFFFF. Retail answers nothing; the next HuntingLog has it;
* Bob's older character: 0xFFFFFFFC, ...F8 and ...E0 for logs at ranks 1, 2 and 5, and a Marauder's at rank 1 (its
  first rank finished) still 0xFFFFFFFE: its second rank never looked at.

So the server keeps which ranks were viewed and the field is their complement. The 401's second number is the rank
looked at (`[5, 1]`, `[5, 2]`, `[5, 3]` in 216274f3, and the field went 0xFFFFFFFE, ...FC, ...F8, ...F0 with them);
finishing a rank doesn't clear its bit or the next one's.

**A kill that counts** sends:

1. ActorControlSelf **542** (note, target, count) for the target counted;
2. when that finished the entry, ActorControlSelf **543** (note, whose log: the class for a class's, the note over
   10,000, 1 for the Gladiator's and 6 for the Conjurer's; the company for a company's, 2 for the Twin Adder's
   2000006) and the entry's reward: EXP (ActorControlSelf 7, as any EXP without a source, and 43) or seals (the
   currency's UpdateInventorySlot and ActorControlSelf 517 `[1300, company, seals]`: `[1300, 2, 357]`);
3. when that finished the rank, ActorControlSelf **544** (the log's index, the rank finished from 0: `[5, 0]` to
   `[5, 3]`);
4. the log's HuntingLog (byte 0 = 0; after a finished rank, the rank byte moved on, the counts and flags zero, the
   last field as it was);
5. after a finished rank, its bonus (ActorControlSelf 7 and 43: 2,500, 10,000, 20,000 and 30,000 for the Conjurer's
   first four, the sheet's numbers), then ActorControlSelf **541** (whose log as 543's, the number of the rank now
   open: `[6, 2]` to `[6, 5]`) and the EXP's 43 once more;
6. then the kill's own EXP.

All of it in one frame (2796.95 s, 4054.37 s, 7004.98 s and 9015.24 s in 216274f3 for the four ranks; 706.14 s for
the company's entry). `kill_steps` in `hunting_log.rs` is this order, tested with those packets.

The kill counts for the current class's log, for the entries of its current rank that want that kind of enemy, up to
each target's count.

**When a log can be worked on.**

* A class's: once the Hunting Log is unlocked (unlock link 21, a class's first quest's SystemReward: Way of the
  Conjurer's) and the class has been taken up. Nothing counted for Connor before his first class quest (60001 and
  60002, the first enemies anyone meets, were never counted for him; 60006 was, later). Bob's Conjurer counted
  Gladiator kills with no Gladiator quest done. Nothing is sent when it unlocks.
* The next rank: when every entry of the one before is done (the wiki: "unlock new ranks by completing all the tasks
  in the previous rank"); no level is asked.
* A company's: for the company the player is in. Its second rank asks for the first finished and the rank of Sergeant
  Third Class, its third for the second and Second Lieutenant (the wiki's pages for the three logs). The sheets have
  the other half: GrandCompanyRank's `RequiredHuntingLogRank` is 1 for rank 5 (Sergeant Third Class) and 2 for rank 9
  (Second Lieutenant), so a promotion to those asks for the log's rank, which the client checks itself
  ([Grand Company](grand-company.md)). Some of its entries are dungeons' enemies (Halatali, Cutter's Cry, the
  Wanderer's Palace).

## In Kawari

As above:

* **Counting.** A kill counts for the current class's log and for the player's company's (`record_hunting_log_kill`),
  each where the enemy's BNpcName is a target of an unfinished entry of its current rank (a kind wanted by two entries
  counts for both).
* **Rewards.** An entry's `Reward` as EXP (`add_exp`) or as the company's seals (`give_company_seals`: the currency
  slot, LogMessage 1300 as a quest's seals, capped at the rank's MaxSeals).
* **Ranks.** When every used entry of a rank is done: 544, the log, the rank's bonus (`hunting_log_rank_reward`) and
  541 in retail's order (since 2026-11-16; before, the bonus came before the log and neither control was sent, and a
  company's 543 had 100, 200 or 300 for its second number where retail has the company). The rank byte moves
  on and the counts are emptied (the Rogue's with its unused targets set, as a fresh one). A company's next rank counts
  only from the company rank that opens it (`company_hunting_log_ranks`, from `RequiredHuntingLogRank`).
* **Seen ranks.** ClientTrigger 401 (`ViewHuntingLog`) sets the log's `seen` bit; bytes 56-59 are its complement. A
  log saved before 2026-11-14 has none seen until the player opens it, as on retail for a log never opened.
* **Promotions.** With the companies' logs kept, the client's own check for Sergeant Third Class and Second Lieutenant
  (the log's rank byte) can pass; before, a company's log was always at rank 0.

## A log finished

Bob's Chronofoil capture bb5fb70a (2026-11-17): the Conjurer's fifth rank, its last entry (60044) done with three
kills in 0.3 s. In one instant: 542 `[60044, 0, 4]`, 543 `[60044, 6]`, the entry's 18,500 EXP (7 and 43), **544
`[5, 4]`**, **ActorControlSelf 515 `[733]` and ActorControl 518 `[733]`** (Bump on a Log: Conjurer), the
**HuntingLog with rank 5 and every count and flag 0**, then the rank's 40,000 EXP, and **no 541**. Kawari sent all
of it but the achievement (and had guessed the missing 541 and the emptied log right): `HuntingLogStep::LogFinished`
now comes after a last rank's 544 and completes the log's achievement (the Achievement sheet's Type 7, Key the log's
index: `KIND_HUNTING_LOG` in `zone_connection/achievement.rs`), which a log already finished gets at the next
achievement check too. Its ring is claimed in the Achievements window
([Achievements](character-windows.md)). The other 16 kills of the session (542, the log; 543 and the EXP with an
entry's last) were as before.

## Not seen on retail

* What the client does with 541 wasn't read out of it. Retail's second 43 after 541 (the same numbers as the one
  before it) isn't sent.
* **A company's finished rank**: one entry of the Twin Adder's was counted, no rank. 544 would have the log's index
  (8-10) and 541 the company, by the classes' rule.
* **Company ranks and the log's ranks**: the wiki's rule; what the HuntingLog looks like while the next rank waits for
  a promotion isn't recorded (Kawari: the rank byte moved on, kills not counted).
* Whether kills by party members count (the community says they do for members nearby; Kawari counts every kill the
  player gets EXP for).
* The companies' "Bump on a Log" achievements: the classes' are Type 7 with the log's index as Key (below); the
  companies' are taken to be the same with 8-10, which no capture shows.
