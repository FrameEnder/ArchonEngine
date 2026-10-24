# Gathering (Miner and Botanist)

The open world's gathering points, where they are, which of their positions is shown and how they move on, the timed
ones, and what happens at a node: the window, each gather (chance, yield, boon), integrity, GP and the gatherer's
actions. Unlocking the classes is the quests' business (Bob's); this is what a gatherer has once unlocked. Fishing
isn't covered.

Code: `servers/world/src/gathering.rs` (the rules: actions, boon and Bountiful Yield, Eorzea time windows; unit tests),
`servers/world/src/zone_connection/gathering.rs` (a player at a node: the window, gathers, actions, the phase table),
`servers/world/src/events/gathering.rs` (the event), `server/zone.rs` (`gathering_point_spawns`),
`server/instance.rs` (`update_timed_gathering_points`), `resources/dropins/*_gathering.json` (the points),
`resources/scripts/gathering/` (the actions).

**Sources**: Bob's Chronofoil captures (numbers only) for the points and their spawn packets; the 7.57 client
(`ffxiv_dx11.exe` 2026.10.16) for how it shows them; GatherBuddy and GatherBuddy Reborn (Apache-2.0) for the timed
windows, the statuses and the boon and Bountiful Yield formulas; the 7.57 Action sheet for the actions. **No retail
gathering has been recorded**: Bob never gathered in the captures, so the node's window and what follows a gather or
an action are upstream's reading or guesses. The research notes are in Bob's session scratchpad
(`research/gathering.md`).

## Where the points are

The client's layouts have no gathering points, so the server sends them (drop-ins). Bob's captures saw 1,014 point
spawns in 21 zones; without the leve points (sent with no integrity, they appear for their leve) that's 643 positions
of 305 points in 18 zones: Middle La Noscea, Lower La Noscea, Eastern La Noscea (s1f1–s1f3), Central Thanalan (w1f2),
Central, East and South Shroud (f1f1–f1f3), Coerthas Central Highlands (r1f1), Coerthas Western Highlands (r2f1), The
Dravanian Forelands, Hinterlands, Churning Mists (d2f1–d2f3), Azys Lla (a2f2), and Lakeland, Kholusia, Amh
Araeng, Il Mheg and The Tempest (n4f1–n4f4, n4f6). `questkit chronofoil <captures> --dropin <territory> --gathering
--appends <lgb>` writes them; flying through a zone with Chronofoil running records the rest.

Each object has retail's SpawnObject `args1` (`spawn_args` in the drop-in), whose bytes are: the integrity (the
GatheringPoint's Count), the point's **phase slot**, which of the point's positions this one is, and how many
positions it has (1–3). Retail sends **every** position of a point (all three spawned together in the captures); the
client shows the one its phase table picks (7.57 `0x140E8E8F0`): position `phase[slot] % positions`, from a table of
160 bytes (QuestManager +0x7AC) that retail sends once at login, **GatheringPointPhases** (309 on 7.57h1; values 0–5,
different at every one of 42 logins), and changes one slot at a time with **GatheringPointPhase** (845; slot, phase).
Kawari sends a random table at login as retail does.

Upstream's drop-in for Middle La Noscea (three positions of point 30454, no arguments) is the same three layouts as
the captured one; a layout in two drop-ins is spawned once, from the one with arguments, and points without them get
theirs worked out (positions by descending layout ID, as retail numbered them, and a free slot).

## Integrity and moving on

What's left at each point is the player's own (`GatheringState`), sent in SetupGatheringPoint (ActorControlSelf 384)
when the client asks (ClientTrigger 706) and shown in the window. Each gather costs one, gathered or not. When the last
is used the window closes, the point's slot steps to the next phase (GatheringPointPhase), which shows the point at its
next position, and its integrity is back (SetupGatheringPoint again): normal nodes "come back elsewhere" on retail.
How retail does it isn't recorded; this is what the client's own visibility check allows.

## Timed points

Unspoiled, legendary (GatheringPoint.Type 2, 5) and ephemeral (4) points are only spawned while they're up
(`update_timed_gathering_points`, every server tick): GatheringRarePopTimeTable's windows (StartTime HHMM; Duration 160
is two Eorzea hours and 300 three, as GatherBuddy and Teamcraft agree) and GatheringPointTransient's ephemeral windows
(end 0 is midnight). One Eorzea hour is 175 s. While up, their `args2` bytes are 1, 1, the position shown, 0: the
client shows unspoiled and legendary points only with both flags set and at that position, and ephemeral ones only
with them (7.57 `GatheringEventHandler.Setup`; the captures had 01 01 nn 00 on the points that were up and zeros on the
rest). The position shown is picked from the point, the window and the day. A used-up timed point stays empty for four
Eorzea hours, longer than any window.

## At the node

The window (scene 0; 0 when it opens, 7 to refresh): the event, 0x80000688 (retail's, unknown), `[count, ?, left, ?]`,
then eight items of six words: the GatheringItem, `[?, ?, level, ?]`, `[chance, 255 (no HQ since Endwalker), yield,
flags]` and three unknown words (upstream's recorded scene). Flags: 4 always (both recorded items had it), 8 hidden
(until Luck), 64 not in the gathering log yet. A gather is the client's yield `[?, slot, 2]`; scene 1 `[2, 266]` plays
the animation, and its end brings the window back (or closes it).

* **Gathering chance**: a stand-in (no source has retail's): 100% with the item's base Gathering (ItemLevel at the
  GatheringItemLevelConvertTable row, which GatherBuddy Reborn's table matches from row 151), falling in proportion
  below it; Sharp Vision (+5/15/50) and Clear Vision (+15, next gather) on top; 0 without the item's PerceptionReq.
* **Yield**: 1, King's Yield / Blessed Harvest +1 or +2, Bountiful Yield +1 or Bountiful Yield II +1 to +3 (Gathering
  against the base: 110% → 3, 90% → 2) on the next gather, Twelve's Bounty +3 crystals (+2 below 71).
* **Gatherer's Boon**: GatherBuddy Reborn's curve of Perception against the base (0 below 60%, 35% at 100%, 60% at
  150%), + Gift I 10% and Gift II 30%; +1 item, +1 more with Tidings.
* **GP**: a gatherer's resource is GP (gear only for now); actions cost it through the action pipeline (Action sheet
  cost type 7); 5 back per gather (6 from level 80), and away from nodes 5 a tick (6/7/8 from 70/80/83, the GP traits;
  their quests aren't checked). UpdateHpMpTp's last field is GP (the client writes it to CharacterData's GP).
* **EXP**: a stand-in: GatheringExp at the item's level per gather, twice that the first time (it goes in the gathering
  log). Rested EXP applies.

### Actions

Every Miner and Botanist action of the 7.57 Action sheet but collectables' is registered to one script
(`gathering/GathererAction.lua`, from `Init.lua`), which hands it to the connection (`player:gathering_action(id)`;
scripts get the action's ID as a third argument now). At a node: Sharp Vision / Field Mastery I–III, Clear Vision /
Flora Mastery, Gifts I and II, Twelve's Bounty, Bountiful Yield / Harvest I and II, Solid Reason / Ageless Words (+1
attempt; Eureka Moment half the time from 90), King's Yield / Blessed Harvest I and II, Luck (shows hidden items), the
Giving Land, Tidings, Wise to the World (+1 attempt, uses Eureka Moment). Each gives its status (218, 754, 2666, 759,
825, 756, 1286, 219, 1802, 2667; GatherBuddy Reborn's IDs) and the window is sent again; the node's statuses come off
when the window closes. Away from a node, Prospect / Triangulate, Truth of Mountains / Forests and Sneak toggle their
statuses (225/217, 222/221, 47).

## Not done yet

See [Known Issues](known-issues.md#gathering).
