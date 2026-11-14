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

From every Chronofoil capture at hand (Bob's and Connor's, 2026-10-18 to 2026-11-12: 92 logins of six characters, 27
targets counted and 9 entries finished, by a Conjurer and a Gladiator), the game's sheets, and the community's wiki
where neither shows it (said where).

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

So the server keeps which ranks were viewed and the field is their complement. Only rank 0 was seen in a 401.

**A kill that counts** sends:

1. ActorControlSelf **542** (note, target, count) for the target counted;
2. when that finished the entry, ActorControlSelf **543** (note, the note over 10,000: 1 for the Gladiator's, 6 for
   the Conjurer's) and the entry's EXP (ActorControlSelf 7, as any EXP without a source);
3. the log's HuntingLog (byte 0 = 0);
4. then the kill's own EXP.

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
* **Ranks.** When every used entry of a rank is done: the rank's bonus (`hunting_log_rank_reward`), the rank byte moves
  on and the counts are emptied (the Rogue's with its unused targets set, as a fresh one). A company's next rank counts
  only from the company rank that opens it (`company_hunting_log_ranks`, from `RequiredHuntingLogRank`).
* **Seen ranks.** ClientTrigger 401 (`ViewHuntingLog`) sets the log's `seen` bit; bytes 56-59 are its complement. A
  log saved before 2026-11-14 has none seen until the player opens it, as on retail for a log never opened.
* **Promotions.** With the companies' logs kept, the client's own check for Sergeant Third Class and Second Lieutenant
  (the log's rank byte) can pass; before, a company's log was always at rank 0.

## Not seen on retail

* **A finished rank**: no capture has one. The bonus's amount is the sheet's; how it's announced (an ActorControl, a
  log line) isn't known, so it's given as an entry's reward is.
* **A company's entry**: no capture counted one. 543's second number for it would be 100, 200 or 300 by the rule the
  classes' follow (the note over 10,000); the seals' packets are a quest reward's.
* **Company ranks and the log's ranks**: the wiki's rule; what the HuntingLog looks like while the next rank waits for
  a promotion isn't recorded (Kawari: the rank byte moved on, kills not counted).
* 401's second number other than 0.
* Whether kills by party members count (the community says they do for members nearby; Kawari counts every kill the
  player gets EXP for).
* The achievements for finishing a log ("Bump on a Log": a ring for each class's, a minion for each company's) aren't
  given.
