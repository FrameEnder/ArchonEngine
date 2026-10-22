# Research notes

Content that has been recorded and analysed but not implemented. The full notes are in the private
`kawari-diagnostics` repository (`notes/`), because they quote recorded packets; this page says what each covers and
where the work stands.

## Palace of the Dead (`notes/palace-of-the-dead.md`)

* **Source**: Kawari Recorder 2.2 export of three runs of floors 1–10 on retail 2026.10.06 (one full run with the
  boss). Recorder 2.2 cut duty packets to 48 bytes, so spawn packets and the end of DeepDungeonMap are incomplete;
  a re-record with **Recorder 2.3** (1 KB) is needed before implementing spawns.
* **What's known**: territory 561, ContentFinderCondition 174, entered from Quarrymill (South Shroud), director
  0x8003EA61 (InstanceContent 60001) and its DirectorVars, the independent level system (log 7249, normal EXP 0),
  floor transitions (PrepareZoning + new layout in the same territory, log 7270 "Floor N"), the deep dungeon packets
  by opcode, monsters per floor with fixed level and HP, the gameplay log messages.
* **Published rules** (Console Games Wiki, Icy Veins): coffers, aetherpool, pomanders, spawns per floor.
* **Aetherpool**: Bob's recording save had arm +76 and armour +86→+91, which skews damage numbers (not HP or levels).
  A fresh player starts at +0/+0.
* **Where it goes**: the `Deep-Dungeon` branch (from master `d28f9cf`). Nothing implemented yet.

## Blunderville (`notes/blunderville.md`)

* **Source**: Recorder 2.1 export of five Fall Guys collaboration matches (retail 2026.10.06). 2.1 only kept packets
  addressed to quests/directors, so this covers the director's side.
* **What's known**: territory 1165, ContentFinderCondition 958, PublicContent director 0x8004002B (row 43), up to 24
  players, the client windows (FGSHud…, FGSResult…), the timeline of a match (log messages 10512–10520, round starts,
  MGP payouts), the director packets by opcode, and that obstacles run on timers.
* **Missing**: packets inside the rounds (needs Recorder 2.2+ duty packets).
* Not scheduled.

## Sastasha with a Duty Finder party (Bob's Chronofoil capture of 2026-10-22)

Five runs of Sastasha (territory 1036, ContentFinderCondition 4) with Duty Finder parties of real players, Bob on
White Mage (runs 1–3), Dark Knight (4) and Gunbreaker (5): 8.1, 8.6, 13.0, 11.3 and 8.2 minutes (the first two with
practised parties, the third with a new tank and side rooms explored, the last two with Bob tanking). For the party
bots' stage 3 (Sastasha's route and rules) and loot:

* **The Duty Finder**: ContentFinderUpdate twice as you queue, ContentFinderFound 9–249 s later (a second Found when
  someone declined), ContentFinderCommencing (state 3, then 4 once you accept with ContentFinderAction), then the
  duty's ZoneInit with UnkContentFinder2 (4, 0, …) and (4, 4, …); (4, 5, …) as you leave.
* **The party**: see [Party Bots](party-bots.md#the-duty-finder-stage-2).
* **Enemies**: 130–155 per run of 36–39 kinds, level 15; the bosses' HP 5869 (base 417), 4785 (416, 413) and 3828
  (411); 16–19 different enemy actions per run, every boss's kit in every run.
* **Coffers and loot**: 43 coffers spawned over the five runs (SpawnTreasure), 15 opened (OpenedTreasure; 5 by
  Bob, OpenTreasure from his client); 278 LootMessage. The Need/Greed/Pass packets are now known and done: see
  [Coffers and Loot](loot.md). The client's 769 (16 bytes, 4 times: the Sastasha director's handler, 0, and an ID) came
  once a run near its end, before the loot window was opened again; not known.
* Not implemented yet: the route and rules for bots (stage 3). Upstream's Sastasha script
  (`content/s1d1_re.lua`) runs the duty itself.
