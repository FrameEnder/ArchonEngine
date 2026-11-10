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

## Sastasha with Duty Support (Bob's Chronofoil capture of 2026-10-23)

One 30-minute run (`700cc294`, visit 0.3) for It's Probably Pirates (245), Bob a level 17 Conjurer with the three
Duty Support members, who took the hits other players would have and fought far longer than a Duty Finder party does,
so it shows what the faster runs never got to:

* **The members**: see [Party NPCs](party-npcs.md) (spawns, party slots, formation, kits, lines).
* **The quest**: moved to its last step as Bob used the duty's exit after the clear (GimmickAccessor 5, DutyExit;
  EventLogMessage from the director, the quest's notice 3, UpdateQuest, then PrepareZoning): see
  [Quests](quests.md) (`onDutyCleared`).
* **Denn's adds** (base 418, 315 HP, the arena's four corners: 4064938, 4064937, 3978761, 3978770): two waves, three
  47.3 s after the tank pulled him (he was at 41 %), then all four 15.7 s later (28 %), each using 835 with no target
  as it came, then 870 and 555 on whoever they reached. Of the five Duty Finder runs, two had one wave (four, 46.8 and
  71.4 s after he appeared, he at 2 % in one) and three killed him first. Kawari's script sends them now.
* **The bosses' kits**, across all six runs (uses, median gap): Chopper 351 (7, 14.8 s, a 2.7 s cast, no damage);
  the quarters' reaver 31 (3, 6.7 s); the first captain 9 once; the second 9 (21, 8.6 s) and 1000 (7, 24.4 s, no
  damage); Denn 722 (16, 15.5 s), 834 (16, 7.9 s), 556 (6, 46.5 s, a 3.2 s cast, 154 on Bob once). Their damage to
  players and to the members are their timelines' `recorded_amounts` now (Chopper 63 and 36 a hit, Denn 36 and 20).
* **Enemies**: the same 92 as the Duty Finder runs (none new), but more of the respawning base 942 (52 against 32–49).

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
* **Giant clams**: enemies (base 269, level 15, 319 HP) that never attack, in any run; each has a shell (EObj 2000222,
  2000260–2000264, bound by the clam's gimmick ID) whose shared group state goes 6 (shut) for 4 s, 21 (open) for 11 s,
  over and over, from the shell (ActorControl 409 with object type 0). Kawari's clam timeline does this now.
* **The second captain** (4035056, the boss by the gate at map X 7.9, Y 16.2; Kawari's `BNPC_CAPTAIN2`): four
  corsairs (4087815, 3988322, 4087814: base 414; 3988323: 415; 269 HP) stand behind the gate (2000235) from before his
  fight. Below half his health he plays animation 980, the gate opens 0.6 s later (SetEventState 7, as `hide_eobj`
  sends), each of the four goes on the party's enemy list (ActorControl 503, then 11), he yells 136 and they come.
  The yell came 0.7–5 s after he went under 50 % in all five runs (30–49 % by then), 22–31 s into his fight; in one he
  fell before they reached him. Kawari's script does this now.
* **Tanks**: see [Party Bots](party-bots.md#how-it-fights).
* Not implemented yet: bots' route through the dungeon. Upstream's Sastasha script
  (`content/s1d1_re.lua`) runs the duty itself.

## What the ledger's first pass showed (2026-11-10)

From `questkit chronofoil --actions`, `--controls`, `--gauges` over Bob's 124 captures ([questkit](questkit.md#chronofoil);
the tables are in the diagnostics repository, `facts/`). Not acted on yet:

* **UpdateRecastTimes** (640 bytes) is 80 floats of seconds elapsed, then 80 floats of each group's whole recast,
  by recast group; a group not running is 0 in both. Retail sends it on arriving in a zone (19 times in a 106-minute
  capture, twice with something running: group 60 at 3 s, 567 s ... 1,772 s of 1,800). Kawari sends it as unread bytes.
* **MP** isn't sent after an action: the capturing player's MP came only with UpdateHpMpTp's three-second tick (and
  status lists), already less the cost and plus the tick's 200. The client takes the cost off itself.
* **Request to effect**, the capturing player's own: 0.05-0.07 s for an instant action; a cast's effect comes about
  0.47 s before its cast time is up (1.5 s casts at 1.03 s, 1.92 s at 1.44-1.45 s, 2.7 s at 2.47 s, 5 s at 4.53 s).
  Animation locks: 0.6 s for instants, 0.1 s after a cast, 2.1 s for Teleport and Return (5, 6).
* **UnkJobGauge** (8 bytes): the twelve of one capture were all zero but one (a first byte of 14).
* **ActorControl**: 310 categories seen, 155 without a name in Kawari's enum; **ClientTrigger**: 105 seen, 31 without.
  The most plentiful unnamed: ActorControl 212 (6,493 times in 106 captures), 56, 47, 35, 105, 413, and to oneself 198
  and the nine sent on every zone change. Sapphire has names for 30 and 12 of them ([Outside sources](sources.md#sapphire)).
* **Gauges** by what moved: Gladiator/Paladin byte 1 (0 or 100), White Mage bytes 3-6 (the lily timer, lilies, blood
  lily), Black Mage byte 3 (the element, 255 for ice), Arcanist/Summoner bytes 1-5, 7 and 8 (Fester takes byte 8
  down by one, 274 times; the gem spells byte 7). The same offsets [Combat](combat.md) has for them.
