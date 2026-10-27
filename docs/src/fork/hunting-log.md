# Hunting Log

Each class (and Grand Company) has a log of enemies to defeat: five ranks of ten entries, each entry up to four kinds
of enemy and how many of each, with EXP for finishing it. Until 2026-10-27 Kawari had none of it: the log was empty
and kills never counted (Bob's bundle of 2026-10-27, a Conjurer killing ground squirrels and little ladybugs).

Code: `servers/world/src/hunting_log.rs` (the rules, tested against retail's numbers) and
`servers/world/src/zone_connection/hunting_log.rs` (what the client is sent); game data in `GameData::monster_note`,
`hunting_log_index`, `hunting_log_class`. Progress is kept in `unlock.hunting_log` (migration
`update-2026-10-27-2_hunting_log`).

## What retail does

From Bob's Chronofoil capture b23b91ef (2026-10-26: a new Conjurer's Way of the Conjurer, then Conjurer 01, 02, 04
and 06), and every login of his older character in the other captures (logs at ranks 0 to 5).

**The logs.** Twelve, by ClassJob's `MonsterNote` column: 0 Gladiator, 1 Pugilist, 2 Marauder, 3 Lancer, 4 Archer,
5 Conjurer, 6 Thaumaturge, 7 Arcanist, 8-10 the Grand Companies, 11 Rogue (jobs share their class's). An entry is a
MonsterNote row, `<class> * 10000 + rank * 10 + entry + 1` (60002 is the Conjurer's rank 1 entry 2), whose targets
are MonsterNoteTarget rows (their BNpcName) with a count, and whose `Reward` is its EXP (60002: 180, as retail gave).

**HuntingLog** (server opcode 385 on 2026.10.16, 64 bytes, `fork_opcodes.yml`; the client's MonsterNoteManager rank
info), one log:

| Bytes | Field |
|---|---|
| 0-3 | -1 at login, 0 after a kill |
| 4 | the rank being worked on (0-4; 5 when finished) |
| 5 | the log's index (the client's handler reads it first) |
| 6-45 | kills of each target of each of the rank's ten entries (entry * 4 + target) |
| 48-55 | five bits an entry: one per target done, then one for the entry (60002 done: 0x220; then 60001: 0x231) |
| 56-59 | a bit per rank, cleared for every rank open: 0xFFFFFFFF while the log is locked, 0xFFFFFFFE at rank 0, ...FC at 1, ...F8 at 2, ...E0 when finished. The Rogue's log has only the low byte. |

All twelve are sent at login. A fresh Rogue's log has every entry's third and fourth targets counted 0xFF and done
(both characters, every login); the others are zeros. After a kill that counts, retail sends:

1. ActorControlSelf **542** (note, target, count) for the target counted;
2. when that finished the entry, ActorControlSelf **543** (note, 6) and the entry's EXP (ActorControlSelf 7, as any
   EXP without a source);
3. the log's HuntingLog (byte 0 = 0);
4. then the kill's own EXP.

The kill counts for the current class's log, for the entries of its current rank that want that kind of enemy, up to
each target's count. Way of the Conjurer's reward unlocks it (unlock link 21, its SystemReward); nothing is sent then:
the log first appeared with the first kill. Opening the log sends nothing.

## In Kawari

As above. A log is open when the player has unlock 21 and has taken up the log's class (its level isn't 0); the Grand
Companies' logs stay locked. Every entry of the current rank whose target is the defeated enemy's BNpcName counts it
(a kind wanted by two entries counts for both). When every entry of a rank is done the log moves to the next rank,
its counts and bits cleared (`RankCompleted`): retail's packets for that weren't recorded, so only the HuntingLog
shows it.

## Not known yet

* 543's second number: 6 for the Conjurer's entries, which is both the class and the log's index plus one. Kawari
  sends the class.
* What retail sends when a rank is finished (EXP, a rank-up ActorControl) and what the Grand Companies' logs give
  (seals).
* Whether kills by party members count (Kawari counts every kill the player gets EXP for).
* Retail's older character had logs open for classes it had levelled; when exactly each opens (the class's first
  quest, or taking up the class) isn't recorded.
