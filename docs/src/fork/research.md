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
