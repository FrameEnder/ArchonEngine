# Levequests

**Not built.** Kawari's levemetes refuse everyone (`resources/scripts/events/generic/Levemete.lua`, scene 2), there
are no leve directors, and nothing calls a quest's `onLeveCompleted` (Leves of Hawthorne, 221, waits on it). This page
is what retail's packets show so far, from one capture, and what's still needed before it can be built as retail
does it.

## What Bob's capture b9af1bb2 shows (2026-11-24, a White Mage 88)

Times are seconds into each visit (`questkit chronofoil` numbering, capture 0 of its folder).

* **Leve 573 was already in the journal** when the capture began: the client asked about it (ClientTrigger 0x2C1
  [2, 573]) 5.7 s into visit 3, before either levemete was spoken to. Accepting a leve isn't in the capture.
* **The levemetes' menu** (GuildLeveAssignment events): talking to one starts its event, scene 0, flags 0x2000. The
  client yields (EventYieldHandler2) and the server resumes with the leves it offers, two Leve IDs (u16) to a u32:
  * the Hawthorne Hut's (GuildLeveAssignment 25, East Shroud, visit 3 at 105.5): yield [0, 1, 0, 4], resume of 4 values
    `[0x04000200, 31517, 800 | 801 << 16, 812 | 813 << 16]`; returned [20, 22, 16, 16] at 127.9, nothing taken;
  * New Gridania's (GuildLeveAssignment 4, visit 4 at 25.0): yield [0, 1, 0, 2], resume of 12 values
    `[0x00000A00, 31517, 503, 504, 506, 505, 521, 522, 524, 523, 539, 540, 538, 541, 591, 592, 594, 593]` (sixteen
    Leve IDs); a second yield [.., 1, 0x6FFF] gave four more (732, 733, 736, 737), a third (Unknown945) a list of
    small numbers; nothing taken. What the first two values are (31517 both times) isn't known.
* **Starting it** (visit 5): ClientTrigger 0x2C1 [2, 573] then 0x324 [573, 4] at 71.3 (the journal's Initiate;
  what 2 and 4 mean isn't known), and the server: Unknown128 [0, 573, 1, …] and [0, 573, 1, 0x20, …], EventLogMessage
  1377 from the leve's director, **0x8001054E** (handler type BattleLeve, 1358: the BattleLeve row behind Leve 573),
  EventNotice4, DirectorVars, EventLogMessage 1357 [573].
* **The leve** (73 to 346 s): an emote leve. The player used an emote on NPCs (StartEmoteEvent on an NPC with the
  director as handler, emote 8); each answer is the director's EventStart/EventFinish with EventLogMessage 1821
  [507, the player], and DirectorVars count it (byte 4 a stage, 1 to 6; byte 6 a running total); NpcYell and
  EventLogMessage 1824 [n] as stages pass, EventLogMessage4 1369 once.
* **Its end** (346.4): Unknown128 [0, 573, 0xFF, …], UpdateQuest (221 to sequence 255), EventNotice2 from the
  director, its EventStart and EventScene2 (scene 2, flags 0x2001, params [1, 1]: the result), DirectorVars byte 4 =
  0xFE.
* **Turn-in** (378.7): talking to Qina Lyehga (221's ACTOR0, also GuildLeveAssignment 2) played 221's scene 3, then
  her levemete event, scene 5 with [1, 573]; at its yield (461.6) the rewards (EventLogMessage 1606, gil and items:
  UpdateInventorySlot), Unknown244 [573], EventResume; then 221's scene 4.

## Still needed

* **Accepting leves**: a capture where leves are taken at a levemete (the menu's answer, the journal's packets,
  allowances), and abandoning one.
* **A battle leve with enemies** (the common kind): its spawns, the director's counts, the timer, failing it.
* **Gathering and crafting leves**, if they're to come too.
* The leve sheets (Leve, BattleLeve, LeveClient, GuildleveAssignment, LeveRewardItem…) are read at runtime from the
  game data, as everything else.
