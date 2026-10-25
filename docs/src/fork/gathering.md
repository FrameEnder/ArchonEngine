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
windows, the statuses, the boon and Bountiful Yield formulas and the items' base stats; the 7.57 Action sheet for the
actions. **Bob's Chronofoil capture of 2026-10-24** (a level 58 Miner and a level 60 Botanist: normal, node-bonus,
unspoiled and ephemeral nodes, actions at nodes, nodes used up) gave the window, every packet around a gather and an
action, the EXP and how used-up points come back. The research notes are in Bob's session scratchpad
(`research/gathering.md`).

## Where the points are

The client's layouts have no gathering points, so the server sends them (drop-ins). Bob's captures saw 1,014 point
spawns in 21 zones; without the leve points (sent with no integrity, they appear for their leve) that's 643 positions
of 305 points in 18 zones; with Bob's flight through Mor Dhona and the Shrouds (2026-10-25, merged with the older
captures) and Coerthas Central Highlands (the same day) it's 933 positions of 405 points in 20 zones: Middle La Noscea, Lower La Noscea, Eastern La Noscea
(s1f1–s1f3), Central Thanalan (w1f2), Central, East, South and North Shroud (f1f1–f1f4), Mor Dhona (l1f1), Coerthas Central Highlands (r1f1), Coerthas Western Highlands (r2f1), The
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

What's left at each point is the player's own (`GatheringState`), sent in SetupGatheringPoint (ActorControlSelf 384:
point, base, level, left, count, bonus) when the client asks (ClientTrigger 706) and in the window. Each gather costs
one, gathered or not. The client hides a point with nothing left. As retail: each player keeps the last **two** points
they used up hidden; when a third is used up, the oldest comes back with its integrity: GatheringPointPhase with a new
phase for its slot (random 0–5, showing another of its positions; not sent for a point with one position) and
ActorControlSelf **109** `[0x30000 | point, 0, count, count, 1]`, both as the third one's window closes.

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

Opening a node: EventStart (handler 0x30000 | point), EventLogMessage 1097, conditions 17 and 63 (Kawari's
`WaitingToVisitOtherWorld`; `extra_conditions`), SetMode 6, then the window.

**The window** is scene 0 with 60 params (`send_gathering_window`):

| Params | Meaning |
|---|---|
| 0 | mode: 0 opened, 7 refresh, 4 after a gather, 5 after an action, 19 refresh with nothing left |
| 1 | the point; after a gather the slot gathered; after an action 0xFFFFFFFF |
| 2 | 0x80000680, + 8 at a node that isn't mining, + 0x20000 at an unspoiled one (`gathering::window_flags`) |
| 3 | bytes `[count, 0, left, 0]` |
| 4–51 | eight items of six words: the GatheringItem; `[100, ?, level row (u16)]`; `[chance, boon (255: none), yield, flags]`; base Gathering << 16; base Perception; 0 |
| 52–59 | `[?, the node bonus' value, ?, 0, attempts made, 0, 0x8000 + 0x100 with the bonus + the last slot gathered, 0]` |

Item flags: 64 not in the gathering log yet (crystals never), 2 a boon bonus (the node's or a Gift), 1 Sharp Vision,
8 hidden. Retail's slots are shuffled against the GatheringPointBase's order (crystals always in slot 4, the rest
differently per base); Kawari keeps the sheet's order. The unknown words (`?`) looked like leftover memory.

* **Base stats**: GatherBuddy Reborn's IlvConvertTable (`resources/data/gatherbuddy/`), by the item's level row:
  retail's window had these on every item (one Perception off by one), not ItemLevel's.
* **Gathering chance** (`gathering::gathering_chance`): retail showed 100 on every item of the capture (Bob's
  gatherers were well above the items). No source has the formula, so it's fitted to the few numbers recorded: 100
  with the item's base Gathering, 1.55 points less per percent short of it (BlueGartr's ARR discussion, 2013: 72% at
  271 Gathering on a level 50 item, base 330), 7 points less per level the item is above the gatherer (same thread:
  74/67/60% at 0/+1/+2); Sharp Vision (+5/15/50) and Clear Vision (+15, next gather) on top.
* **Gatherer's Boon**: GatherBuddy Reborn's curve of Perception against the base (0 below 60%, 35% at 100%, 60% at
  150%; retail showed 59 and 60), + the node's bonus, Gift I 10%, Gift II 30%. 255 (none) for crystals and items whose
  GatheringItem.Unknown2 isn't 0xFFFF. +1 item, +1 more with Tidings.
* **Yield**: 1, + the node's yield bonus (not on items with GatheringItem.Unknown5 false: retail gave one such item
  none), King's Yield / Blessed Harvest +1 or +2, Bountiful Yield +1 or II +1 to +3 on the next gather, Twelve's
  Bounty +3 crystals (+2 below 71).
* **Node bonuses** (GatheringPointBonus, the point's two IDs; the first met counts): conditions 14 Gathering, 15
  Perception, 19 max GP at least the value; bonuses 14/17 yield, 22/23 boon, 18/19 integrity, by BonusValue (the "～"
  ranged ones take its low end). Retail: +40% boon at Perception 182 and +1 yield at Gathering 237, shown in the tail.

**A gather** (`gather`, `finish_gather`): the client's `[point, slot, 2]` rolls it; scene 1 plays the animation,
`[2, 258]` for a Miner at a mining node (3 with the boon), Botanist 5 (6), 266 at quarrying and harvesting nodes. Its
end (`[point, 0, 0, x]` from scene 1) brings, as retail: the item, RecordGatheringLog and LiveEvent 5 the first time,
EXP, packet **618** (GatheringChainCount, fork opcode, 8 bytes) with the chain when it's 1 or more, the window in mode
4, EventLogMessages 1050 and 1054 with `[item, amount]`, then StatusEffectList with GP (+5, 6 from 80). The client
answers with `[point, 0, 4]` for the refresh (mode 7, or 19 with nothing left). `[point, 0, 6]` after the window opens
asks for a refresh too.

**Leaving**: `[point, 0, 11]` brings scene 0 `[8, point, param 2's low half, count]`; then `[point, 0, 1]`: 618 with 0,
the node's statuses off, LiveEvent 39 with nothing, EventFinish (and the point used up, above, if nothing's left).

**EXP** (`gathering::gather_exp`): 4.98% of GatheringExp at the gatherer's level (a level 58 Miner got 241 for items of
level 36–40, GatheringExp 4,845; 12 to 22 levels below changed nothing; further below a stand-in, 9.5% less a level).
Bonus: rested EXP, +100% with the boon, the chain +5/10/20% for the 2nd–4th gather in a row at the node and +50% from
the 5th. The first gather of an item adds a fifth of GatheringExp (retail 968) as its own message, with no bonus.

* **GP**: a gatherer's resource is GP (gear only for now); actions cost it through the action pipeline (Action sheet
  cost type 7); 5 back per gather (6 from level 80), and away from nodes 5 a tick (6/7/8 from 70/80/83, the GP traits;
  their quests aren't checked). UpdateHpMpTp's last field is GP (the client writes it to CharacterData's GP).

### Actions

In the node's window the client sends no ActionRequest: it yields `[point, action, 3]`, and retail answered with the
GP (ActorControl SetBaseStat 7), ActionEffect, the window in mode 5 and EffectResult; the client then yields
`[point, 0, 5]` for the refresh. Kawari hands that action to the action pipeline as an ActionRequest
(`request_gathering_action`), which charges the GP and shows it. Every Miner and Botanist action of the 7.57 Action
sheet but collectables' is registered to one script (`gathering/GathererAction.lua`, from `Init.lua`), which hands it
to the connection (`player:gathering_action(id)`; scripts get the action's ID as a third argument). At a node: Sharp Vision / Field Mastery I–III, Clear Vision /
Flora Mastery, Gifts I and II, Twelve's Bounty, Bountiful Yield / Harvest I and II, Solid Reason / Ageless Words (+1
attempt; Eureka Moment half the time from 90), King's Yield / Blessed Harvest I and II, Luck (shows hidden items), the
Giving Land, Tidings, Wise to the World (+1 attempt, uses Eureka Moment). Each gives its status (218, 754, 2666, 759,
825, 756, 1286, 219, 1802, 2667; GatherBuddy Reborn's IDs; param 0, as retail) and the window is sent in mode 5; the
node's statuses come off when the window closes. Away from a node, Prospect / Triangulate, Truth of Mountains / Forests and Sneak toggle their
statuses (225/217, 222/221, 47).

## Not done yet

See [Known Issues](known-issues.md#gathering).
