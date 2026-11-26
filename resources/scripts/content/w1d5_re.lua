-- The Praetorium (ContentFinderCondition 16, territory 1044, director 0x80030056), during The Ultimate Weapon (quest
-- 4522).
--
-- Five retail runs with Duty Support (the fourth set, level 50): Connor's Chronofoil capture af05916c (2026-11-09,
-- visit 0.10, a healer), Bob's 9ba7d981 (2026-11-17, visits 0.14, left at the last boss, and 0.16, a damage
-- dealer), 50e72631 (the same day, visit 0.5, a healer) and 6320ae56 (2026-11-18, visit 0.3, a Paladin, with Kawari
-- Witness); `questkit chronofoil --battle` and the visits' packets (docs/src/fork/dungeons.md). Numbers only. **No
-- layout dump**: the duty's event ranges aren't known, so what the party sets off by walking somewhere is set off
-- here by where they were when it happened in the runs (the WHERE_* lines below).
--
-- The way through:
--   * talk 32920 ten yalms in. The four at the entrance (9081944-9081947) dead: talk 32921, the lift
--     (EObj 2012811) works, five more below (9081963-9081967);
--   * the lift (event action 23, a second): everyone 30 yalms down; map effect 1 4.6 s on. The five dead: map effect 2
--     and ten more (9081969-9081978); map effect 4 further in; the ten dead: map effect 3. The second lift (EObj
--     2012813) to the first boss's floor;
--   * near its room: objective 0 and cutscene 280; the first boss (9054638) is there 2.2 s in. It dead: music 149,
--     objective 1, talk 32916, the sixteen of the way ahead (9082001-9082005, 9082022-9082032), and two objects
--     work: EObj 2012815 (everyone up to the hangar; talks 32917 and 32918 5.7 and 12.7 s on) and the armour (EObj
--     2012819: Mount 17, whose action 1128 does 4500 to all it hits);
--   * the ride: the last five of the sixteen dead (9082028-9082032): map effect 5, and the way's end takes
--     everyone below (still mounted) to seven more (9168161-9168164,
--     9168175-9168177); four of BNpcBase 14566 one after another (9082092-9082095: map effects 8 and 12, 9 and 13, 10
--     and 11, then 14 and 18) and BNpcBase 14616 (9089992); it dead: map effects 6 and 7 and cutscene 392, the mount
--     gone;
--   * further in: objective 2 and cutscene 283; the second boss (9054847) is there 2.3 s in. It dead: objective 3, map
--     effects 0 and 15, cutscene 284, and EObj 2012817 works: objective 4, map effect 16, cutscene 118, and as it ends
--     everyone before the last boss (9055066). It dead: the duty complete, cutscene 374, everyone out.
--
-- The fights (the bosses' own casts are their timelines, PraetoriumBoss1_14450, PraetoriumBoss2_14451 and
-- PraetoriumBoss3_14453; what goes with them is here):
--   * the first: 28467 takes it to the middle; with 28468 four helpers' 28469 6 yalms round the middle and four 28470
--     12 out (a cross the first time; the second time a cross or turned, at random); 5.9 s into 28471 a helper's
--     28472 on each of the party;
--   * the second: talks 32900 (4.1 s after the pull, with map effect 0), 32902 and 32901 9.2 and 29.6 s after it,
--     32903 below 20 %;
--     0.9 s after each 28479 lands one of BNpcBase 14452 comes (north, then south);
--   * the last: talks 32904 (the pull), 32906, 32905 and 32907 at 39.5, 62.1 and 80.5 s. 28496 takes it to the
--     middle; with 28486 a helper's 29779 and, 3.1 s on, three of BNpcBase 14456's 28487 across the middle the way it
--     faces; 3.1 s after each 28484 five of BNpcBase 14454 at the west edge (one of two sets of places), and five
--     14456s' 28488 from there 4.9 s later; with 29051 four rows of three helpers' 29052 2 s apart (north to south
--     or south to north); 3.9 s into 28491 a helper's 28492 on each of the party. **The second part** from 141 s
--     (139.7 to 142.9 in four runs, its health at 11 to 15 %): talk 32919; 5 s on it can't be targeted and four of
--     BNpcBase 14455 fight in its place (28498 on each of the party at 11.8, 34.9 and 63.0 s; 29051 at 27.8 with
--     the rows; 28491 at 49.9 with the 28492s), the 14454s and 28488s again at 16.8, 48.9 and 68.0 s, and a gauge
--     (the director's 0x8000000C [58, n], n up 111 a second from 7.5 s). At 77.9 s talk 32908, 28499 on everyone 2.1
--     s later, talk 32909 at 90.1 and it fights on (28495 7.1 s later; it fell within 12 s of it in the runs).
--
-- Not known, so guessed or left out: the event ranges (above); the gauge's full (10000 here, at 97.5 s, when 28500
-- wipes the party with any of the four standing: Harmony's visual guide; no run let it fill) and its 28500's area
-- and amount (more than anyone's HP); which edge 28475 takes the second boss to (one at random here); the second
-- boss's ring of lightning at the edge (the guide: a damage-over-time to whoever touches it; no run did, so its
-- status isn't known: map effect 0 shows it, nothing hurts); Iron Uprising's (28482) knockback (it hit no one in ten
-- casts); the bosses' clocks past what the runs show (the rounds go on); a fall at a boss (it wasn't seen: the party is
-- sent back to where the last lift or cutscene put it, `set_start`, and the fight begins again); whether the 14455s
-- attack between their casts (they do here); the members on the ride (they walk); ActorControls 20, 62, 63 and 70 on
-- the last boss (sent as retail did); Cid's NpcYell 13805 every 8 s while the party watches cutscene 280 (four runs of
-- five; left out: nobody sees it); coffers (none in a Duty Support run).
--
-- Since 2026-11-24 (Bob's marks on `5e9e306`: "cid is not moving to the terminal like he did in retail", and his ask
-- for everything else retail does here), from the same five runs: Cid's way to the terminal and on the first boss's
-- floor (CID_*), the members' story lines (LINE_*), the bosses' own (YELL_*), the second boss's 28475 out to an edge
-- and back, every leap an AoeEffect8 at no one where it comes down (`cast_aims.json`, `aim_action`), and the ten
-- launchers' 29049 (`launchers.json`: the first five from map effect 4 to the ten's fall, the other five from the
-- ride's first six falling to map effect 5). Later the same day (Bob's run on `f3c32e2`: "the arena that lead to the
-- cutscene was blocked before entering ... the dungeon is literally impossible to play"): both boss rooms' walls open
-- from the start as retail spawned them; Cid runs his points in one go (`D:lead` by `sweep_bnpc_path`); the Death
-- Claw at one of four edges, tethered to one of the party and going for them alone (`D:pursue`); Grand Sword at one
-- of the party at random and Augmented Uprising at the middle (the timelines' facings), the second boss kept on its
-- edge till it leaps back; the last boss's enrage.

-- Written with the dungeon framework (content/lib/Dungeon.lua, loaded first): its packs, steps and bosses are
-- declared below, and what only this duty does is in their hooks.

GUIDE = 9168666 -- Cid (BNpcBase 14567)
NAME_GUIDE = 1486
TALK_MS = 6000

EVENT_ACTION_OBJECT = 23
EOBJ_LIFT1 = 2012811
EOBJ_LIFT2 = 2012813
EOBJ_HANGAR = 2012815
EOBJ_ARMOUR = 2012819
EOBJ_LAST = 2012817
EOBJ_AFTER_LAST = 2012818
EOBJ_EXIT = 2000848
-- Where retail put the player (its ActorSetPos, capture 6320ae56.)
LIFT1_TO = { x = 200.10, y = 156.50, z = 13.63, rotation = 0.522 }
LIFT2_TO = { x = 240.75, y = 71.50, z = 0.35, rotation = -1.571 }
HANGAR_TO = { x = 146.86, y = 102.00, z = -37.53, rotation = 2.654 }
BELOW_TO = { x = 108.73, y = -106.01, z = 0.45, rotation = -1.529 }
BOSS3_TO = { x = -550.22, y = -268.00, z = 220.35, rotation = -1.571 }
OUT_TO = { x = -764.70, y = -344.00, z = 0.45, rotation = -1.554 }
MOUNT_ARMOUR = 17

FOUR = { 9081944, 9081945, 9081946, 9081947 }
FIVE = { 9081963, 9081964, 9081965, 9081966, 9081967 }
TEN = { 9081969, 9081970, 9081971, 9081972, 9081973, 9081974, 9081975, 9081976, 9081977, 9081978 }
AHEAD = { 9082001, 9082002, 9082003, 9082004, 9082005 }
RIDE = { 9082022, 9082023, 9082024, 9082025, 9082026, 9082027, 9082028, 9082029, 9082030, 9082031, 9082032 }
LAST_FIVE = { 9082028, 9082029, 9082030, 9082031, 9082032 }
BELOW = { 9168161, 9168162, 9168163, 9168164, 9168175, 9168176, 9168177 }
BELOW_THREE = { 9168162, 9168163, 9168164 }
WALKERS = { 9082092, 9082093, 9082094, 9082095 }
-- The arm units spawn hidden (display flags 0x40028) and come into view 0.58-0.85 s on with ActionTimeline 142
-- (ActorControl 36 `[1, 142]`, then 54 `[1]`; Bob's Chronofoil capture 35e30205, each of the four.)
WALKER_APPEARS = 142
WALKER_LAST = 9089992
-- The armour standing below (ENpc 1043118, layout 9283424): retail spawned it 2.3 s into cutscene 392 (capture
-- 35e30205), not before.
ARMOUR_BELOW = 9283424
ARMOUR_BELOW_AFTER = 2.3

-- Where the party was as retail set these off (no layout dump: see above.)
WHERE_START_X = 159.0 -- east of this at the entrance (ten yalms in): talk 32920 (2.1 to 54 s after the start)
WHERE_EFFECT_4 = { z = -51.5, x = 225.0 } -- north of z and east of x on the lift's floor: map effect 4
WHERE_BOSS1_X = 208.0 -- west of this on the second lift's floor: objective 0 and cutscene 280
-- The way's end, once map effect 5 is on: east of x on the ramp down between the two zs: everyone below
WHERE_BELOW = { x = 225.0, z_south = 76.0, z_north = 55.0 }
WHERE_EFFECT_8_Z = -29.5 -- a player north of this below: map effect 8 and the first of BNpcBase 14566
WHERE_EFFECT_9_X = 54.5 -- a player west of this with the three of 9168162-9168164 dead: map effect 9 and the second
WHERE_EFFECT_10_X = -16.5 -- and of this with all seven dead: map effect 10 and the third
WHERE_BOSS2_X = -130.0 -- objective 2 and cutscene 283

BOSS1 = 9054638
BOSS1_WALL = 2000828
BOSS1_LINE = 2000829
-- Open (event state 7) from the start, closed as the first boss is pulled, with log 2012; the wall closes 15 s later
-- with 2013 (captures 9ba7d981 twice, 50e72631 and 6320ae56). Kawari spawned it closed, so the room was sealed when Bob
-- came back from cutscene 280 (2026-11-25: "the arena chamber seal was active when it shouldn't have been").
BOSS1_BEHIND = 2000789
BOSS1_AHEAD = 2000876 -- gone as it falls
PLACE_BOSS1 = 451
BOSS1_MIDDLE = { x = 192.0, y = 76.0, z = 0.2 }
-- The bosses' floors, which the party bots stand and dodge on and never leave while the boss is fought (the rooms'
-- ranges aren't known here): where retail's party stood from each boss's first action to its fall, in the five runs
-- (x 175.4 to 209.8 and z -9.8 to 7.8 here; -9.8 in all five), with `ARENA_MARGIN`'s half yalm. Bob's mark of
-- 2026-11-25: with none, the bots ran out through this room's walls from 28474, an attack on everyone.
BOSS1_FLOOR = { x = 192.6, z = -1.0, half_x = 17.75, half_z = 9.35 }
HELPER_BOSS1 = 1
HELPERS_BOSS1 = 8
BGM_AFTER_BOSS1 = 149
CUTSCENE_BOSS1 = 280
CUTSCENE_BOSS1_PARAMS = { 1, 37, 0, 0 }
BOSS1_AFTER_CUTSCENE = 2.2
-- Gaius (ENpc 1043117, layout 9283423): retail let him go unseen as the first boss came, 2.4 s into cutscene 280 (his
-- spawn index went to the boss's helpers' SpawnNpc, no DeleteActor; the client swapped him out as 280 ended: Bob's
-- Chronofoil capture 35e30205 and his Kawari Witness runs of 2026-11-25.)
GAIUS = 9283423
-- Its 28467s (seconds from the pull), as its timeline has them: twice, then two a round of 52.7 s.
AWAY_FOR = 1.0
BOSS1_AWAY_ONCE = { 18.3, 38.6 }
BOSS1_AWAY_ROUND = { 71.0, 90.4 }
BOSS1_AWAY_EVERY = 52.7
ROUNDS = 10
-- (x, z) from the middle.
SPOTS_28469 = {
    { { -6.0, 0.0 }, { 0.0, -6.0 }, { 0.0, 6.0 }, { 6.0, 0.0 } },
    { { -5.2, -3.0 }, { -3.0, 5.2 }, { 3.0, -5.2 }, { 5.2, 3.0 } },
}
SPOTS_28470 = {
    { { -12.0, 0.0 }, { 0.0, -12.0 }, { 0.0, 12.0 }, { 12.0, 0.0 } },
    { { -10.4, -6.0 }, { -6.0, 10.3 }, { 6.0, -10.4 }, { 10.4, 6.0 } },
}
AFTER_28471 = 5.9
-- 28472 (Prototype Laser Beta) and the last boss's 28492: each helper casts from the room's middle at one of the
-- party, a circle that follows them, with Lockon 96 over them as it begins (captures 9ba7d981, 50e72631 and 6320ae56:
-- all four helpers moved to the middle first, every time: (192.0, 0.18) and (-562.0, 220.0)). Bob's mark of
-- 2026-11-25: "it should have 2 indicators a ground aoe, and an overhead symbol".
LOCKON_SPREAD = 96
ON_EACH_BOSS1 = { lockon = LOCKON_SPREAD, at = { x = 192.0, z = 0.18 } }
ON_EACH_BOSS3 = { lockon = LOCKON_SPREAD, at = { x = -562.0, z = 220.0 } }

BOSS2 = 9054847
BOSS2_WALL = 2000822
BOSS2_LINE = 2000823
BOSS2_AHEAD = 2001135
-- Two that retail spawned open (event state 7) and never touched, by layout (their bases have other copies): the
-- second boss's way west, 2000826 at (-216.2, -0.0), and a 2000789 by the terminal at (-237.2, -21.6) (Bob's capture
-- 35e30205: his party walked west through the first 113 s after the boss fell). Kawari spawned both shut: the way was
-- barred after the boss and the other stood by the terminal (Bob's mark of 2026-11-26: "the barrier to the next area
-- did not lift ... a shortcut that shouldn't be active yet next to the magitek terminal").
OPEN_AT_START = { 9247278, 9058476 }
PLACE_BOSS2 = 454
BOSS2_MIDDLE = { x = -164.0, y = -104.2, z = 0.0 }
-- Round: retail's party stood to 20.1 yalms from the middle (22.3 once), from x -184 to -143 and z -20 to 20.
BOSS2_FLOOR = { x = -164.0, z = 0.0, half_x = 20.6, round = true }
NAME_BOSS2 = 2135
HELPER_BOSS2 = 2
HELPERS_BOSS2 = 8
CUTSCENE_RIDE = 392
CUTSCENE_RIDE_PARAMS = { 1, 0, 0, 0 }
CUTSCENE_BOSS2 = 283
CUTSCENE_BOSS2_PARAMS = { 1, 151, 0, 0 }
BOSS2_AFTER_CUTSCENE = 2.3
CUTSCENE_AFTER_BOSS2 = 284
CUTSCENE_AFTER_BOSS2_PARAMS = { 1, 149, 0, 0 }
-- Its talks, seconds from its pull (4.1 s in all five runs; then 9.2 and 29.6 s after the first.)
BOSS2_TALKS = { { 4.1, 32900 }, { 13.3, 32902 }, { 33.7, 32901 } }
BOSS2_TALK_BELOW = 20
BOSS2_ADD = 9054848
BOSS2_ADD_AFTER = 3.6 -- its 28479's 2.7 s and 0.9 more
-- Where each came: one of the four edges, 20 yalms out, facing the middle (ten in five runs: north, south, east and
-- west, no order to it), tethered to one of the party (SpawnNpc's tether target, Channeling 17: a member or the
-- player). It stood 4.9 s, walked after them alone at 3.0 yalms a second and, the three times it got there, used
-- 28480 on them from 3.2 to 3.7 yalms (knockback row 70); it never auto-attacked (`D:pursue`.)
BOSS2_ADD_AT = { { -164.0, -104.2, -20.0 }, { -164.0, -104.2, 20.0 }, { -144.0, -104.2, 0.0 },
    { -184.0, -104.2, 0.0 } }
BOSS2_ADD_TETHER = 17
BOSS2_ADD_STANDS = 4.9
BOSS2_ADD_SPEED = 3.0
BOSS2_ADD_REACH = 3.5
BOSS2_ADD_ACTION = 28480
-- A Death Claw after the one it's tethered to: it stands `BOSS2_ADD_STANDS`, walks after them and, reaching them,
-- knocks them back (28480); then, still tethered, it stands and goes after them again till it falls (Bob,
-- 2026-11-26: "never repeats until dead". Retail's stood on where it struck for the 2 s each lived after it, capture
-- 35e30205, so the stand before each go is the one it began with.)
function clawAfter(director, claw, quarry)
    D:pursue(director, claw, quarry, { speed = BOSS2_ADD_SPEED, after = BOSS2_ADD_STANDS, reach = BOSS2_ADD_REACH,
                                       action = BOSS2_ADD_ACTION,
                                       on_reach = function()
                                           D:after(0.1, function(d)
                                               if D.fight == 2 and D.info ~= nil and D.info["actor" .. claw] ~= nil then
                                                   clawAfter(d, claw, quarry)
                                               end
                                           end)
                                       end })
end

-- The ring of lightning round the second boss's room (map effect 0 shows it as the fight begins): one of the party
-- who goes 20 yalms from its middle has Electrocution 3073 (no end, from no one) while out there, and as they come
-- back 3074 for 15 s, losing 260-279 every 3 s (Bob's Chronofoil capture 35e30205: he at 20.4 yalms, 0.15 s on; a
-- Lancer knocked out to 23.3 by the Death Claw; ten ticks, 270 on average; nothing hit them as they touched it.)
BOSS2_RING = 20.0
BOSS2_RING_STATUS = 3073
BOSS2_RING_AFTER = 3074
BOSS2_RING_AFTER_FOR = 15.0
BOSS2_RING_TICK = 270

BOSS3 = 9055066
BOSS3_MIDDLE = { x = -562.0, y = -268.0, z = 220.0 }
-- Square: retail's party stood from x -575.0 to -552.0 and z 202.0 to 238.0 (each edge in four or five runs.)
BOSS3_FLOOR = { x = -563.5, z = 220.0, half_x = 12.05, half_z = 18.55 }
NAME_BOSS3 = 2136
HELPER_BOSS3 = 3
HELPERS_BOSS3 = 16
CUTSCENE_BOSS3 = 118
CUTSCENE_BOSS3_PARAMS = { 0, 152, 1, 0 }
CUTSCENE_END = 374
CUTSCENE_END_PARAMS = { 0, 18, 0, 0 }
BOSS3_TALKS = { { 39.5, 32906 }, { 62.1, 32905 }, { 80.5, 32907 } }
BOSS3_AWAY = { 15.3, 60.9, 79.3, 109.2 }
BOSS3_SLASH_AT = 17.2 -- its 28486, from the timeline
SLASH_AFTER = 3.1
SLASH_Z = { -4.0, 0.0, 4.0 }
SWORD = 9055067 -- BNpcBase 14454, five
SWORDS = 5
PHANTOM = 9055068 -- BNpcBase 14455, four
BLADE = 9055069 -- BNpcBase 14456: each comes for one cast, 1.9 s before it, and goes 6 s after it began
BLADE_BEFORE = 1.9
BLADE_GONE_AFTER = 6.0
WAITING = { x = -575.0, z_phantoms = 228.0, z_swords = 232.0 } -- where retail kept them
SWORDS_AFTER_28484 = 3.1
SWORDS_X = -576.6
SWORDS_Z = { { 206.0, 214.0, 222.0, 230.0, 238.0 }, { 202.0, 210.0, 218.0, 226.0, 234.0 } }
SWORDS_FACING = 1.571
-- From the 14454s' going to their places: their 28486, ActionTimeline 7737, and the 14456s' 28488.
SWORDS_SHOW = 1.8
SWORDS_TIMELINE = 3.8
SWORDS_CAST = 4.9
SWORDS_CAST_PART2 = 4.2
ROWS_AFTER_29051 = 3.9
ROWS_APART = 2.0
ROWS = {
    { { -573.0, 203.0 }, { -562.0, 237.0 }, { -551.0, 203.0 } },
    { { -573.0, 214.5 }, { -562.0, 225.5 }, { -551.0, 214.5 } },
    { { -573.0, 225.5 }, { -562.0, 214.5 }, { -551.0, 225.5 } },
    { { -573.0, 237.0 }, { -562.0, 203.0 }, { -551.0, 237.0 } },
}
AFTER_28491 = 3.9
-- The second part, seconds from its talk.
-- 139.7 s twice and 142.2 and 142.8 s (7.0 or 10.0 s after its 28495 began; its health 11 to 15 %): between them.
PART2_AT = 141.2
PART_SECOND = 32919
PART_AFTER = 32909
PHANTOMS_AT = { { -554.5, 210.0 }, { -554.5, 230.0 }, { -569.5, 210.0 }, { -569.5, 230.0 } }
PART2_PHANTOMS = 5.0
PART2_ON_EACH = { 11.8, 34.9, 63.0 }
PART2_SWORDS = { 16.8, 48.9, 68.0 }
PART2_ROWS = 27.8
PART2_SPREAD = 49.9
PART2_ENDS = { 67.9, 77.9, 87.9 }
-- From its end (talk 32908): 28499 on everyone, the gauge gone, and the boss back (talk 32909.)
PART2_ALL = 2.1
PART2_GAUGE_GONE = 9.1
PART2_BACK = 12.2
PHANTOM_LOCK = 2.2
COMMAND_GAUGE = 0x8000000C
GAUGE = 58
GAUGE_FROM = 7.5
GAUGE_FULL_SECONDS = 9.0 -- n is 1000 nine seconds in: 111 a second
-- Full at 10000 (a guess: 100 %, as the client's gauge shows hundredths; it stopped at 64 to 77 % in the runs), 90 s on,
-- when, with any of the four still standing, the boss's 28500 wipes the party (Harmony's visual guide: "If all four
-- phantoms haven't been destroyed before the blade energy gauge has reached 100% ... this attack will instantly wipe
-- the party"; none of the five runs let it fill.)
GAUGE_FULL = 10000
GAUGE_SECONDS = 90
ENRAGE = 28500

-- Cid, the same in all five runs to a tenth of a yalm: 0.2 s after talk 32920 he runs at 6.0 yalms a second along
-- the entrance's floor to the magitek terminal (CID_TO_TERMINAL), and as he gets there ActorControl 63 [48]; with
-- talk 32921 63 [18]. 2.3 s into cutscene 280 he's put on the first boss's floor (ActorSetPos, CID_BOSS1) and 63 [11]
-- 2.0 s later twice. 0.25 s after the first boss falls he runs on to CID_AFTER_BOSS1 and from there 63 [0] every 2 s.
CID_SPEED = 6.0
CID_FROM = { x = 155.78, y = 186.1, z = -4.29 }
CID_TO_TERMINAL = {
    { x = 170.37, y = 186.11, z = -10.27 }, { x = 177.9, y = 186.21, z = -18.9 }, { x = 190.0, y = 186.11, z = -24.0 },
    { x = 198.9, y = 186.05, z = -22.0 }, { x = 200.4, y = 186.24, z = -18.7 }, { x = 199.94, y = 186.48, z = -15.43 },
    { x = 196.83, y = 186.05, z = -4.84, rotation = -0.526 },
}
CID_AFTER_TALK = 0.2
CID_AT_TERMINAL = 48
CID_LIFT_OPEN = 18
CID_BOSS1 = { x = 210.08, y = 76.0, z = -2.04, rotation = -1.592 }
CID_BOSS1_AFTER = 2.3
CID_ON_BOSS1_FLOOR = 11
CID_AFTER_BOSS1 = { { x = 192.07, y = 76.04, z = 0.14 }, { x = 172.96, y = 76.04, z = -0.02, rotation = -1.571 } }
CID_AFTER_FALL = 0.25
CID_PULSE_EVERY = 2.0
CID_CONTROL = 63

-- The members' story lines (NpcYell, `party_npc_yell`): the role retail's party had, else the other's line (the
-- Conjurer's 14395 or the Lancer's 14394 1.3 to 1.7 s before talk 32920; the Conjurer's 14397 or the Marauder's 14396
-- as the first boss is pulled; the Thaumaturge's 14398 10.9 to 12.0 s after the hangar (four runs; 26.8 in Connor's);
-- the Lancer's 14400 or the Marauder's 14399 1.1 s after cutscene 283 (four runs; 12.4 in Connor's); the Conjurer's
-- 14402 or the Lancer's 14401 as the last boss is pulled.)
LINE_START = { "healer", 14395, "melee", 14394 }
LINE_START_BEFORE_TALK = 1.5
LINE_BOSS1 = { "healer", 14397, "tank", 14396 }
LINE_HANGAR = { "ranged", 14398 }
LINE_HANGAR_AFTER = 11.5
LINE_BOSS2 = { "melee", 14400, "tank", 14399 }
LINE_BOSS2_AFTER = 1.1
LINE_BOSS3 = { "healer", 14402, "melee", 14401 }

-- The bosses' own lines (NpcYell, `yell`), each once, 0.1 s before what they go with: the second's 13639 with its
-- first 28478 and 13640 with its second 28475; the last's 13643 with its first 28486, 13644 with its first 28484,
-- 13645 with its first 28493 and 13642 2.9 s after its 28499 (all five runs.)
YELL_BOSS2_28478 = 13639
YELL_BOSS2_BACK = 13640
YELL_BOSS3_28486 = 13643
YELL_BOSS3_28484 = 13644
YELL_BOSS3_28493 = 13645
YELL_BOSS3_28499 = 13642
YELL_AFTER_28499 = 2.9

-- The second boss's 28475 (no cast; its timeline's times): the first of each pair takes it 20 yalms out from the
-- middle to the west, east, south or north (west six times of eleven, east twice, south twice, north once: nothing
-- it faced or targeted chose), the second back to the middle; it's there 1.1 s later, facing as it was. ActorControl
-- 70 [8201044] 1.4 s after the first one only (all five runs.)
BOSS2_LEAP = 28475
BOSS2_LEAPS_ONCE = { 45.1, 65.9, 109.4, 133.2 }
BOSS2_LEAPS_ROUND = { 176.7, 200.5 }
BOSS2_LEAPS_EVERY = 67.3
BOSS2_EDGES = { { -20.0, 0.0 }, { 20.0, 0.0 }, { 0.0, 20.0 }, { 0.0, -20.0 } }
BOSS2_LEAP_LANDS = 1.1
BOSS2_AIM_BEFORE = 0.5 -- aimed this long before its timeline's 28475 (an update's worth)
BOSS2_FIRST_LEAP_CONTROL = { 70, 8201044, 1.4 }
-- The leaps (`aim_action`: where the AoeEffect8 at no one is shown.)
BOSS1_LEAP = 28467
BOSS3_LEAP = 28496
BOSS3_AWAY_FOR = 1.1

-- The launchers (BNpcBase 6358: `launchers.json`, `launcher.rs`), out while their stretch is fought.
LAUNCHERS_FIRST = { 9082875, 9082876, 9082877, 9082878, 9082880 }
LAUNCHERS_RIDE = { 9083019, 9083030, 9083041, 9083042, 9083043 }
RIDE_FIRST_SIX = { 9082022, 9082023, 9082024, 9082025, 9082026, 9082027 }

DUNGEON = Dungeon.new()
D = DUNGEON

crosses = 0
adds = 0
leaps = 0
part2 = false
part2_over = false
part2_ending = false
floored = {}
phantoms_free_at = 0
last_boss_due = false
-- The cutscene under way: `onSceneFinished` says only that one ended (and says so once for each player: the first
-- to end it counts.)
playing = nil
swords = {}
blades = {}
phantoms = {}
gone = {}

local within = Dungeon.within
local function done(name)
    return function() return D.done[name] == true end
end

function talk(director, text)
    director:battle_talk(GUIDE, NAME_GUIDE, text, 0, TALK_MS)
end

-- A members' story line (LINE_*), `delay` seconds from now.
function line(director, which, delay)
    director:party_npc_yell(which[1], which[2], delay or 0, which[3], which[4])
end

-- Cid runs the way he did (`D:lead`), and does `then` there.
function cidRuns(director, from, way, after)
    local legs = {}
    for _, spot in ipairs(way) do
        table.insert(legs, { to = spot, speed = CID_SPEED, rotation = spot.rotation })
    end
    D:lead(director, GUIDE, from, legs, after)
end

function cidControl(director, value)
    director:bnpc_control(GUIDE, CID_CONTROL, { value })
end

function cidPulse(director)
    if D.done.duty_over then
        return
    end
    cidControl(director, 0)
    D:after(CID_PULSE_EVERY, cidPulse)
end

-- The launchers stand there from the start (unseen) and cast only while `on`.
function setLaunchers(director, layouts, on)
    for _, layout in ipairs(layouts) do
        director:launcher(layout, on)
    end
end

-- An arm unit, brought into view as retail's came (`WALKER_APPEARS`.)
function spawnWalker(director, layout_id)
    director:spawn_bnpc(layout_id)
    director:appear_bnpc(layout_id, WALKER_APPEARS)
end

function cutscene(director, id, params)
    playing = id
    director:play_cutscene(id, params)
end

-- Whoever of the party has this actor ID (the one at an object), or the first of it.
function whoIs(actor_id)
    for _, member in ipairs(D.party) do
        if member.id == actor_id then
            return member
        end
    end
    return D.party[1]
end

function sendTo(director, to)
    director:warp_players(to.x, to.y, to.z, to.rotation)
    director:set_start(to.x, to.y, to.z, to.rotation)
end

-- ---- The packs whose fall opens the way ----

D:pack({ name = "four_dead", layouts = FOUR, spawns = FIVE,
    on_fall = function(d)
        talk(d, 32921)
        cidControl(d, CID_LIFT_OPEN)
        d:show_eobj(EOBJ_LIFT1)
    end })
D:pack({ name = "five_dead", layouts = FIVE, ready = done("four_dead"), effects = { { 2, 2, 1 } }, spawns = TEN })
D:pack({ name = "ten_dead", layouts = TEN, ready = done("five_dead"), effects = { { 3, 2, 1 } },
    on_fall = function(d) setLaunchers(d, LAUNCHERS_FIRST, false) end })
D:pack({ name = "ride_six_dead", layouts = RIDE_FIRST_SIX, ready = function() return D.dead[BOSS1] == true end,
    on_fall = function(d) setLaunchers(d, LAUNCHERS_RIDE, true) end })
D:pack({ name = "effect_5", layouts = LAST_FIVE, ready = function() return D.dead[BOSS1] == true end,
    effects = { { 5, 2, 1 } },
    on_fall = function(d) setLaunchers(d, LAUNCHERS_RIDE, false) end })

-- ---- What the party's steps set off ----

-- A player, not a member: Connor's party's tank was past each of these before he set it off.
local function riding()
    return D.done.below and not D.done.ride_over
end

D:step({ name = "start",
    where = function(m) return m.y > 175 and m.x > WHERE_START_X end,
    run = function(d)
        line(d, LINE_START)
        D:after(LINE_START_BEFORE_TALK, function(d2) talk(d2, 32920) end)
        D:after(LINE_START_BEFORE_TALK + CID_AFTER_TALK, function(d2)
            cidRuns(d2, CID_FROM, CID_TO_TERMINAL, function(d3) cidControl(d3, CID_AT_TERMINAL) end)
        end)
    end })
D:step({ name = "effect_4", ready = done("five_dead"),
    where = function(m) return m.y > 140 and m.y < 170 and m.z < WHERE_EFFECT_4.z and m.x > WHERE_EFFECT_4.x end,
    run = function(d)
        d:map_effect(4, 2, 1)
        setLaunchers(d, LAUNCHERS_FIRST, true)
    end })
D:step({ name = "boss1_room", ready = done("lift2"),
    where = function(m) return m.y > 60 and m.y < 90 and m.x < WHERE_BOSS1_X end,
    run = function(d)
        d:objective(0, 1)
        cutscene(d, CUTSCENE_BOSS1, CUTSCENE_BOSS1_PARAMS)
        D:after(BOSS1_AFTER_CUTSCENE, function(d2)
            d2:let_go_unseen(GAIUS)
            D:bringBoss(d2, D:bossById(1))
        end)
        D:after(CID_BOSS1_AFTER, function(d2)
            local at = CID_BOSS1
            d2:warp_bnpc(GUIDE, at.x, at.y, at.z, at.rotation)
            cidControl(d2, CID_ON_BOSS1_FLOOR)
            D:after(2.0, function(d3) cidControl(d3, CID_ON_BOSS1_FLOOR) end)
        end)
    end })
D:step({ name = "below", ready = done("effect_5"),
    where = function(m)
        return m.y > 50 and m.y < 72 and m.x > WHERE_BELOW.x and m.z < WHERE_BELOW.z_south and m.z > WHERE_BELOW.z_north
    end,
    run = function(d)
        sendTo(d, BELOW_TO)
        -- (There as the party is: 1.5 s after the fade begins.)
        D:after(1.5, function(d2)
            for _, layout in ipairs(BELOW) do
                d2:spawn_bnpc(layout)
            end
        end)
    end })
D:step({ name = "effect_8", ready = riding,
    where = function(m) return m.player and m.y < -90 and m.x < 95 and m.z < WHERE_EFFECT_8_Z end,
    run = function(d)
        d:map_effect(8, 2, 1)
        spawnWalker(d, WALKERS[1])
    end })
D:step({ name = "effect_9", ready = function() return riding() and D:allDead(BELOW_THREE) end,
    where = function(m) return m.player and m.y < -90 and m.x < WHERE_EFFECT_9_X end,
    run = function(d)
        d:map_effect(9, 2, 1)
        spawnWalker(d, WALKERS[2])
    end })
D:step({ name = "effect_10", ready = function() return riding() and D.done.effect_9 and D:allDead(BELOW) end,
    where = function(m) return m.player and m.y < -90 and m.x < WHERE_EFFECT_10_X end,
    run = function(d)
        d:map_effect(10, 2, 1)
        spawnWalker(d, WALKERS[3])
    end })
D:step({ name = "boss2_room", ready = done("ride_over"),
    where = function(m) return m.y < -90 and m.x < WHERE_BOSS2_X end,
    run = function(d)
        d:objective(2, 3)
        cutscene(d, CUTSCENE_BOSS2, CUTSCENE_BOSS2_PARAMS)
        D:after(BOSS2_AFTER_CUTSCENE, function(d2) D:bringBoss(d2, D:bossById(2)) end)
    end })

-- ---- The first boss ----

D:boss({ id = 1, layout = BOSS1, wall = BOSS1_WALL, line = BOSS1_LINE, place = PLACE_BOSS1, middle = BOSS1_MIDDLE,
    arena = BOSS1_FLOOR, comes_later = true, eobjs_at_setup = false, bgm = false, after_bgm = BGM_AFTER_BOSS1,
    helpers = { layout = HELPER_BOSS1, count = HELPERS_BOSS1 },
    ready = done("boss1_room"),
    on_pull = function(director)
        crosses = 0
        line(director, LINE_BOSS1)
        director:aim_action(BOSS1, BOSS1_LEAP, BOSS1_MIDDLE.x, BOSS1_MIDDLE.y, BOSS1_MIDDLE.z)
        director:show_eobj(BOSS1_BEHIND)
        local function away(at)
            D:after(at + AWAY_FOR, function(d)
                if D.fight == 1 then
                    d:jump_bnpc(BOSS1, BOSS1_MIDDLE.x, BOSS1_MIDDLE.y, BOSS1_MIDDLE.z, 1.571)
                end
            end)
        end
        for _, at in ipairs(BOSS1_AWAY_ONCE) do
            away(at)
        end
        for round = 0, ROUNDS - 1 do
            for _, at in ipairs(BOSS1_AWAY_ROUND) do
                away(at + round * BOSS1_AWAY_EVERY)
            end
        end
    end,
    on_cast = {
        [28468] = function(director)
            crosses = crosses + 1
            -- A cross the first time (all five runs); the second, a cross twice and turned twice.
            local set = 1
            if crosses > 1 then
                set = math.random(2)
            end
            for _, action in ipairs({ { 28469, SPOTS_28469 }, { 28470, SPOTS_28470 } }) do
                local spots = {}
                for _, spot in ipairs(action[2][set]) do
                    table.insert(spots, { BOSS1_MIDDLE.x + spot[1], BOSS1_MIDDLE.z + spot[2], -3.142 })
                end
                D:helpersCast(director, action[1], spots, BOSS1_MIDDLE.y)
            end
        end,
        [28471] = function(director)
            D:after(AFTER_28471, function(d)
                if D.fight == 1 then
                    D:helpersOnEach(d, 28472, BOSS1_MIDDLE.y, ON_EACH_BOSS1)
                end
            end)
        end,
    },
    -- Open again with the room after a wipe (none of the five runs wiped here: as the wall, `reopen_rooms_after_wipe`.)
    on_wipe = function(d)
        d:hide_eobj(BOSS1_BEHIND)
    end,
    on_death = function(d)
        d:objective(1, 2)
        d:hide_eobj(BOSS1_AHEAD)
        d:show_eobj(EOBJ_HANGAR)
        d:show_eobj(EOBJ_ARMOUR)
        for _, layout in ipairs(AHEAD) do
            d:spawn_bnpc(layout)
        end
        for _, layout in ipairs(RIDE) do
            d:spawn_bnpc(layout)
        end
        D:after(1.0, function(d2) talk(d2, 32916) end)
        D:after(CID_AFTER_FALL, function(d2)
            cidRuns(d2, CID_BOSS1, CID_AFTER_BOSS1, cidPulse)
        end)
    end })

-- ---- The second boss ----

D:boss({ id = 2, layout = BOSS2, wall = BOSS2_WALL, line = BOSS2_LINE, place = PLACE_BOSS2, middle = BOSS2_MIDDLE,
    arena = BOSS2_FLOOR, comes_later = true, eobjs_at_setup = false, bgm = false,
    helpers = { layout = HELPER_BOSS2, count = HELPERS_BOSS2 },
    ready = done("boss2_room"),
    on_pull = function(director)
        adds = 0
        leaps = 0
        local function leap(at, next_at)
            D:after(at - BOSS2_AIM_BEFORE, function(d)
                if D.fight ~= 2 then
                    return
                end
                leaps = leaps + 1
                local to = { x = BOSS2_MIDDLE.x, z = BOSS2_MIDDLE.z }
                if leaps % 2 == 1 then
                    local edge = BOSS2_EDGES[math.random(#BOSS2_EDGES)]
                    to = { x = BOSS2_MIDDLE.x + edge[1], z = BOSS2_MIDDLE.z + edge[2] }
                elseif leaps == 2 then
                    D:after(BOSS2_AIM_BEFORE - 0.1, function(d2)
                        if D.fight == 2 then
                            d2:yell(BOSS2, YELL_BOSS2_BACK)
                        end
                    end)
                end
                -- Aimed as its timeline's 28475 goes out (the same half-second), so its AoeEffect8 shows where it lands.
                d:aim_action(BOSS2, BOSS2_LEAP, to.x, BOSS2_MIDDLE.y, to.z)
                local first = leaps == 1
                local out = leaps % 2 == 1
                D:after(BOSS2_AIM_BEFORE + BOSS2_LEAP_LANDS, function(d2)
                    if D.fight == 2 then
                        if out then
                            -- On the edge facing the middle, for its 28478 (timeline `as_it_stands`), and kept there
                            -- till it leaps back: retail's turned to the middle as the cast began and stood on the edge
                            -- the whole 20.8 to 23.8 s, turning only for its 28483 (five runs.)
                            local facing = math.atan(BOSS2_MIDDLE.x - to.x, BOSS2_MIDDLE.z - to.z)
                            d2:jump_bnpc(BOSS2, to.x, BOSS2_MIDDLE.y, to.z, facing)
                            if next_at ~= nil then
                                d2:root_bnpc(BOSS2, next_at - at)
                            end
                        else
                            d2:jump_bnpc(BOSS2, to.x, BOSS2_MIDDLE.y, to.z, nil, nil, true)
                        end
                    end
                end)
                if first then
                    local control = BOSS2_FIRST_LEAP_CONTROL
                    D:after(BOSS2_AIM_BEFORE + control[3], function(d2)
                        if D.fight == 2 then
                            d2:bnpc_control(BOSS2, control[1], { control[2] })
                        end
                    end)
                end
            end)
        end
        -- Every leap's time in order, each told the next (an edge leap is rooted till then.)
        local times = {}
        for _, at in ipairs(BOSS2_LEAPS_ONCE) do
            table.insert(times, at)
        end
        for round = 0, ROUNDS - 1 do
            for _, at in ipairs(BOSS2_LEAPS_ROUND) do
                table.insert(times, at + round * BOSS2_LEAPS_EVERY)
            end
        end
        for index, at in ipairs(times) do
            leap(at, times[index + 1])
        end
        for index, row in ipairs(BOSS2_TALKS) do
            D:after(row[1], function(d)
                if D.fight == 2 then
                    d:battle_talk(BOSS2, NAME_BOSS2, row[2], 0, TALK_MS)
                    if index == 1 then
                        d:map_effect(0, 2, 1)
                    end
                end
            end)
        end
    end,
    on_cast = {
        [28478] = function(director)
            if D:once("yell_28478") then
                director:yell(BOSS2, YELL_BOSS2_28478)
            end
        end,
        -- 0.9 s after each 28479 lands one of BNpcBase 14452 comes, at one of the edges (`BOSS2_ADD_AT`.)
        [28479] = function(director)
            adds = adds + 1
            local at = BOSS2_ADD_AT[math.random(#BOSS2_ADD_AT)]
            D:after(BOSS2_ADD_AFTER, function(d)
                if D.fight == 2 and #D.party > 0 then
                    local quarry = D.party[math.random(#D.party)]
                    local facing = math.atan(BOSS2_MIDDLE.x - at[1], BOSS2_MIDDLE.z - at[3])
                    local claw = d:spawn_tethered_bnpc(BOSS2_ADD, quarry.id, BOSS2_ADD_TETHER, at[1], at[2], at[3],
                                                       facing)
                    clawAfter(d, claw, quarry.id)
                    -- Retail's damage dealer went for it at once (hits 2.3 and 4.8 s after each came; both fell in 8
                    -- to 12 s, capture 35e30205.)
                    d:focus_bnpc(claw)
                end
            end)
        end,
    },
    on_death = function(d)
        d:objective(3, 4)
        d:map_effect(0, 8, 4)
        d:map_effect(15, 1, 4)
        d:hide_eobj(BOSS2_AHEAD)
        d:show_eobj(EOBJ_LAST)
        d:despawn_bnpc(BOSS2_ADD)
        cutscene(d, CUTSCENE_AFTER_BOSS2, CUTSCENE_AFTER_BOSS2_PARAMS)
    end,
    on_wipe = function(d)
        d:despawn_bnpc(BOSS2_ADD)
        D.done.talk_32903 = nil
        D.done.yell_28478 = nil
    end })

-- ---- The last boss ----

-- The way the last boss faces as it stands in the middle: at the first of the party.
function facingFromMiddle()
    local member = D.party[1]
    if member == nil then
        return 1.571
    end
    return math.atan(member.x - BOSS3_MIDDLE.x, member.z - BOSS3_MIDDLE.z)
end

function toTheMiddle(director)
    if D.fight == 3 and not part2 then
        director:jump_bnpc(BOSS3, BOSS3_MIDDLE.x, BOSS3_MIDDLE.y, BOSS3_MIDDLE.z, facingFromMiddle())
    end
end

-- Its first of an action: its line (YELL_BOSS3_*), once a fight.
function bossThreeSays(director, name, yell)
    if D:once(name) then
        director:yell(BOSS3, yell)
    end
end

-- With the last boss's 28486: a helper's 29779 in the middle and, 3.1 s on, three 14456s' 28487 across it.
function slash(director)
    bossThreeSays(director, "yell_28486", YELL_BOSS3_28486)
    local facing = facingFromMiddle()
    D:helpersCast(director, 29779, { { BOSS3_MIDDLE.x, BOSS3_MIDDLE.z, facing } }, BOSS3_MIDDLE.y)
    local spots = {}
    for _, z in ipairs(SLASH_Z) do
        table.insert(spots, { BOSS3_MIDDLE.x, BOSS3_MIDDLE.z + z, facing })
    end
    bladesCast(SLASH_AFTER, 28487, spots)
end

-- BNpcBase 14456s come to the spots, cast so many seconds from now, and go.
function bladesCast(cast_after, action, spots)
    D:after(math.max(0.0, cast_after - BLADE_BEFORE), function(d)
        if D.fight ~= 3 then
            return
        end
        local mine = {}
        for index, spot in ipairs(spots) do
            mine[index] = d:spawn_bnpc(BLADE, spot[1], BOSS3_MIDDLE.y, spot[2], spot[3]) or BLADE
            d:hold_bnpc(mine[index])
            table.insert(blades, mine[index])
        end
        D:after(BLADE_BEFORE, function(d2)
            if D.fight == 3 then
                for _, blade in ipairs(mine) do
                    d2:actor_use_action(blade, action)
                end
            end
        end)
        D:after(BLADE_BEFORE + BLADE_GONE_AFTER, function(d2)
            for _, blade in ipairs(mine) do
                d2:despawn_bnpc(blade)
            end
        end)
    end)
end

-- The five 14454s go to the west edge and show themselves; the 14456s' 28488 from their places follow.
function swordsCome(director, cast_after)
    local places = SWORDS_Z[math.random(#SWORDS_Z)]
    for index, sword in ipairs(swords) do
        director:warp_bnpc(sword, SWORDS_X, BOSS3_MIDDLE.y, places[index], SWORDS_FACING)
        director:actor_use_action(sword, 28485)
    end
    D:after(SWORDS_SHOW, function(d)
        if D.fight == 3 then
            for _, sword in ipairs(swords) do
                d:actor_use_action(sword, 28486)
            end
        end
    end)
    D:after(SWORDS_TIMELINE, function(d)
        if D.fight == 3 then
            for _, sword in ipairs(swords) do
                d:play_action_timeline(sword, 7737)
            end
        end
    end)
    local spots = {}
    for index = 1, SWORDS do
        table.insert(spots, { SWORDS_X, places[index], SWORDS_FACING })
    end
    bladesCast(cast_after, 28488, spots)
end

-- Four rows of three helpers' 29052, 2 s apart: north to south, or south to north.
function rows(director, first_after, reversed)
    for index = 1, #ROWS do
        local row = ROWS[reversed and (#ROWS + 1 - index) or index]
        D:after(first_after + (index - 1) * ROWS_APART, function(d)
            if D.fight == 3 then
                D:helpersCast(d, 29052, row, BOSS3_MIDDLE.y)
            end
        end)
    end
end

-- The last boss's second part: it leaves, four of BNpcBase 14455 fight in its place, and it comes back.
function secondPart(director)
    if D.fight ~= 3 or part2 or part2_over then
        return
    end
    part2 = true
    floored = {}
    director:restart_timeline(BOSS3, PART_SECOND)
    director:battle_talk(BOSS3, NAME_BOSS3, PART_SECOND, 0, 3000)
    local function at(seconds, fn)
        D:after(seconds, function(d)
            if D.fight == 3 and part2 and not part2_ending then
                fn(d)
            end
        end)
    end
    at(PART2_PHANTOMS, function(d)
        d:set_targetable(BOSS3, false)
        d:hold_bnpc(BOSS3)
        d:actor_use_action(BOSS3, 28496)
        for index, phantom in ipairs(phantoms) do
            local spot = PHANTOMS_AT[index]
            d:warp_bnpc(phantom, spot[1], BOSS3_MIDDLE.y, spot[2], 0.0)
            d:actor_use_action(phantom, 28485)
            d:set_targetable(phantom, true)
            d:release_bnpc(phantom)
        end
    end)
    at(PART2_PHANTOMS + 1.6, function(d)
        d:actor_use_action(BOSS3, 28497)
        d:bnpc_control(BOSS3, 20, { 2552, 388 })
        d:bnpc_control(BOSS3, 62, { 0, 1 })
        d:bnpc_control(BOSS3, 63, { 27 })
    end)
    -- The gauge, a second at a time while any of the four is above 1 HP.
    for second = 0, GAUGE_SECONDS do
        local n = math.floor(second * 1000 / GAUGE_FULL_SECONDS)
        at(GAUGE_FROM + second, function(d)
            if not allFloored() then
                d:command(COMMAND_GAUGE, { GAUGE, n })
            end
        end)
    end
    -- It ends at the first of these with all four at 1 HP (67.9 s once, 77.9 three times in the runs); with any of them
    -- still up as the gauge fills, the boss's 28500 wipes the party.
    for _, seconds in ipairs(PART2_ENDS) do
        at(seconds, function(d)
            if allFloored() and not part2_ending then
                secondPartEnds(d)
            end
        end)
    end
    at(GAUGE_FROM + GAUGE_FULL * GAUGE_FULL_SECONDS / 1000, function(d)
        if part2_ending then
            return
        end
        if allFloored() then
            -- (All four down since the last look: it ends now.)
            secondPartEnds(d)
        else
            d:actor_use_action(BOSS3, ENRAGE)
        end
    end)
    for _, seconds in ipairs(PART2_ON_EACH) do
        at(seconds, function(d) phantomsUse(d, 28498, 4.7, true) end)
    end
    for _, seconds in ipairs(PART2_SWORDS) do
        at(seconds, function(d) swordsCome(d, SWORDS_CAST_PART2) end)
    end
    at(PART2_ROWS, function(d)
        if phantomsUse(d, 29051, 2.7, false) then
            rows(d, ROWS_AFTER_29051, true)
        end
    end)
    at(PART2_SPREAD, function(d)
        if phantomsUse(d, 28491, 2.7, false) then
            D:after(AFTER_28491, function(d2)
                if D.fight == 3 then
                    D:helpersOnEach(d2, 28492, BOSS3_MIDDLE.y, ON_EACH_BOSS3)
                end
            end)
        end
    end)
end

function allFloored()
    for _, phantom in ipairs(phantoms) do
        if not floored[phantom] then
            return false
        end
    end
    return true
end

function secondPartEnds(director)
    part2_ending = true
    local function at(seconds, fn)
        D:after(seconds, function(d)
            if D.fight == 3 and part2 then
                fn(d)
            end
        end)
    end
    director:battle_talk(BOSS3, NAME_BOSS3, 32908, 0, 4000)
    director:bnpc_control(BOSS3, 70, { 8201036 })
    at(PART2_ALL, function(d) d:actor_use_action(BOSS3, 28499) end)
    at(PART2_ALL + YELL_AFTER_28499, function(d) d:yell(BOSS3, YELL_BOSS3_28499) end)
    at(PART2_ALL + 3.0, function(d) d:bnpc_control(BOSS3, 70, { 8201619 }) end)
    at(PART2_ALL + 5.3, function(d)
        d:bnpc_control(BOSS3, 62, { 0, 0 })
        d:bnpc_control(BOSS3, 63, { 0 })
    end)
    at(PART2_GAUGE_GONE, function(d) d:command(COMMAND_GAUGE, { GAUGE, 0xFFFFFFFF }) end)
    at(PART2_BACK, function(d)
        for _, phantom in ipairs(phantoms) do
            if not gone[phantom] then
                d:despawn_bnpc(phantom)
            end
        end
        phantoms = {}
        d:battle_talk(BOSS3, NAME_BOSS3, PART_AFTER, 0, 5000)
        d:set_targetable(BOSS3, true)
        d:release_bnpc(BOSS3)
        d:restart_timeline(BOSS3, PART_AFTER)
        part2_over = true
    end)
end

-- The four use something together, those still there; whether any did.
function phantomsUse(director, action, cast, on_each)
    local any = false
    for index, phantom in ipairs(phantoms) do
        if not gone[phantom] then
            local member = D.party[(index - 1) % math.max(1, #D.party) + 1]
            if not on_each then
                director:actor_use_action(phantom, action)
                any = true
            elseif member ~= nil then
                director:actor_use_action(phantom, action, member.id)
                any = true
            end
        end
    end
    phantoms_free_at = D.ticks + math.floor((cast + PHANTOM_LOCK) * 2 + 0.5)
    return any
end

-- The last boss and its four of BNpcBase 14455 at 1 HP: each of the four goes once it's free of its cast (at once, or
-- 2.2 s after the cast it was in: 17 times in four runs); the boss ends the duty.
function lastBossHealth(director, info)
    for _, phantom in ipairs(phantoms) do
        local it = info["actor" .. phantom]
        if it ~= nil and it.hp <= 1 then
            floored[phantom] = true
        end
        if floored[phantom] and not gone[phantom] and D.ticks >= phantoms_free_at then
            gone[phantom] = true
            director:set_targetable(phantom, false)
            director:hold_bnpc(phantom)
            director:play_action_timeline(phantom, 7737)
            D:after(1.5, function(d) d:despawn_bnpc(phantom) end)
        end
    end
    local boss = info[BOSS3]
    if boss ~= nil and boss.hp <= 1 and (not part2 or part2_over) then
        D:fell(director, D:bossById(3))
    end
end

function dropLastBossOwn(director)
    for _, group in ipairs({ swords, blades, phantoms }) do
        for _, actor in ipairs(group) do
            director:despawn_bnpc(actor)
        end
    end
    swords, blades, phantoms = {}, {}, {}
end

function bringLastBoss(director)
    director:spawn_bnpc(BOSS3)
    -- It doesn't fall: at 1 HP the duty is done (its health was 1 as the director said so in every run.)
    director:set_health_floor(BOSS3, 1)
    D:bringHelpers(director, D:bossById(3))
    -- Its four of BNpcBase 14455 and five of 14454 are there with it (retail spawned them as it came), waiting:
    -- they can't be targeted until their time.
    swords, blades, phantoms, gone, floored = {}, {}, {}, {}, {}
    for index = 1, #PHANTOMS_AT do
        phantoms[index] = director:spawn_bnpc(PHANTOM, WAITING.x, BOSS3_MIDDLE.y, WAITING.z_phantoms, SWORDS_FACING) or PHANTOM
        -- They don't fall either: at 1 HP each is gone (untargetable, ActionTimeline 7737.)
        director:set_health_floor(phantoms[index], 1)
        director:hold_bnpc(phantoms[index])
        director:set_targetable(phantoms[index], false)
    end
    for index = 1, SWORDS do
        swords[index] = director:spawn_bnpc(SWORD, WAITING.x, BOSS3_MIDDLE.y, WAITING.z_swords, SWORDS_FACING) or SWORD
        director:hold_bnpc(swords[index])
        director:set_targetable(swords[index], false)
    end
end

D:boss({ id = 3, layout = BOSS3, middle = BOSS3_MIDDLE, arena = BOSS3_FLOOR,
    comes_later = true, eobjs_at_setup = false, bgm = false,
    last = true,
    helpers = { layout = HELPER_BOSS3, count = HELPERS_BOSS3 },
    ready = done("last_room"),
    talker = { name = NAME_BOSS3, pull = 32904, at = BOSS3_TALKS, ms = TALK_MS,
               still = function() return not part2 end },
    on_pull = function(director)
        part2 = false
        part2_over = false
        part2_ending = false
        line(director, LINE_BOSS3)
        director:aim_action(BOSS3, BOSS3_LEAP, BOSS3_MIDDLE.x, BOSS3_MIDDLE.y, BOSS3_MIDDLE.z)
        for _, name in ipairs({ "yell_28486", "yell_28484", "yell_28493" }) do
            D.done[name] = nil
        end
        for _, at in ipairs(BOSS3_AWAY) do
            D:after(at + BOSS3_AWAY_FOR, toTheMiddle)
        end
        D:after(BOSS3_SLASH_AT, function(d)
            if D.fight == 3 and not part2 then
                slash(d)
            end
        end)
        D:after(PART2_AT, secondPart)
    end,
    on_cast = function(director, action_id)
        if part2 then
            return
        end
        if action_id == 28493 then
            bossThreeSays(director, "yell_28493", YELL_BOSS3_28493)
        elseif action_id == 28484 then
            bossThreeSays(director, "yell_28484", YELL_BOSS3_28484)
            D:after(SWORDS_AFTER_28484, function(d)
                if D.fight == 3 then
                    swordsCome(d, SWORDS_CAST)
                end
            end)
        elseif action_id == 29051 then
            rows(director, ROWS_AFTER_29051, math.random(2) == 1)
        elseif action_id == 28491 then
            D:after(AFTER_28491, function(d)
                if D.fight == 3 then
                    D:helpersOnEach(d, 28492, BOSS3_MIDDLE.y, ON_EACH_BOSS3)
                end
            end)
        end
    end,
    -- At 1 HP (`lastBossHealth`) or its death: the duty is done.
    on_death = function(d)
        D:once("duty_over")
        dropLastBossOwn(d)
        d:show_eobj(EOBJ_EXIT)
        cutscene(d, CUTSCENE_END, CUTSCENE_END_PARAMS)
        d:complete_duty()
        d:despawn_bnpc(BOSS3)
    end,
    -- Everything of its fight goes, and comes back as the party does.
    on_wipe = function(d)
        part2 = false
        d:despawn_bnpc(BOSS3)
        D:dropHelpers(d, D:bossById(3))
        dropLastBossOwn(d)
        last_boss_due = true
    end,
    on_recommence = function(d)
        if last_boss_due then
            last_boss_due = false
            bringLastBoss(d)
        end
    end })

-- ---- What the server calls ----

function onSetup(director)
    director:set_sequence(1)
    setLaunchers(director, LAUNCHERS_FIRST, false)
    setLaunchers(director, LAUNCHERS_RIDE, false)
    for _, object in ipairs({ EOBJ_LIFT1, EOBJ_HANGAR, EOBJ_ARMOUR, EOBJ_LAST, EOBJ_AFTER_LAST, EOBJ_EXIT }) do
        director:hide_eobj(object)
    end
    -- The bosses' walls stand open until their pulls: retail spawned 2000828 and 2000822 with event state 7 (open) at
    -- the start and closed them (ActorControl 106 [0]) 15 s into each fight (captures 6320ae56 and af05916c). Kawari
    -- spawned them closed, so the first boss's room was sealed before anyone came (Bob's marks of 2026-11-24: "the
    -- arena that lead to the cutscene was blocked before entering", "the dungeon is literally impossible to play").
    -- The first boss's 2000789 too: open (7) at the start, closed at his pull (`BOSS1_BEHIND`.)
    for _, wall in ipairs({ BOSS1_WALL, BOSS2_WALL, BOSS1_BEHIND }) do
        director:hide_eobj(wall)
    end
    for _, layout in ipairs(OPEN_AT_START) do
        director:hide_eobj(layout)
    end
    D:setup(director)
end

function onCommence(director)
end

function onActorCast(director, layout_id, action_id, actor_id, x, y, z)
    D:cast(director, layout_id, action_id, actor_id, x, y, z)
end

-- Who's out on the ring (actor ID: true), as `BOSS2_RING` says.
ringed = {}

function ring(director)
    local on = D.fight == 2
    for _, member in ipairs(D.party or {}) do
        if member.id ~= nil then
            local dx, dz = member.x - BOSS2_MIDDLE.x, member.z - BOSS2_MIDDLE.z
            local out = on and math.sqrt(dx * dx + dz * dz) >= BOSS2_RING
            if out and not ringed[member.id] then
                ringed[member.id] = true
                director:gain_effect(member.id, BOSS2_RING_STATUS, 0, 0)
            elseif ringed[member.id] and not out then
                ringed[member.id] = nil
                director:lose_effect(member.id, BOSS2_RING_STATUS)
                if on then
                    director:gain_effect(member.id, BOSS2_RING_AFTER, 0, BOSS2_RING_AFTER_FOR, BOSS2_RING_TICK)
                end
            end
        end
    end
end

local function between(director)
    ring(director)
    if D.fight == 3 then
        lastBossHealth(director, D.info)
    end
    if D.fight == 2 and not D.done.talk_32903 and D.health[BOSS2] ~= nil and D.health[BOSS2] < BOSS2_TALK_BELOW then
        D:once("talk_32903")
        director:battle_talk(BOSS2, NAME_BOSS2, 32903, 0, 3000)
    end
end

function onUpdate(director, health, player_health, info)
    D:update(director, health, player_health, info, between)
end

function onActorDeath(director, layout_id, position)
    D:died(director, layout_id)
    if layout_id == WALKERS[1] then
        director:map_effect(12, 1, 1)
    elseif layout_id == WALKERS[2] then
        director:map_effect(13, 1, 1)
    elseif layout_id == WALKERS[3] then
        director:map_effect(11, 2, 1)
        D:after(0.6, function(d) spawnWalker(d, WALKERS[4]) end)
    elseif layout_id == WALKERS[4] then
        director:map_effect(14, 1, 1)
        director:map_effect(18, 2, 1)
        director:spawn_bnpc(WALKER_LAST)
    elseif layout_id == WALKER_LAST then
        D:once("ride_ended")
        director:map_effect(6, 1, 4)
        cutscene(director, CUTSCENE_RIDE, CUTSCENE_RIDE_PARAMS)
        -- The mount goes 1.6 s into the cutscene (SetMode 1 and ActorControl 930 on retail.)
        D:after(1.6, function(d)
            for _, member in ipairs(D.party) do
                d:mount_player(member.id, 0)
            end
        end)
        D:after(2.0, function(d) d:map_effect(7, 4, 4) end)
        D:after(ARMOUR_BELOW_AFTER, function(d) d:spawn_bnpc(ARMOUR_BELOW) end)
    end
end

function onSceneFinished(director, scene)
    local ended = playing
    playing = nil
    if ended == CUTSCENE_RIDE then
        D:once("ride_over")
    elseif ended == CUTSCENE_BOSS2 then
        line(director, LINE_BOSS2, LINE_BOSS2_AFTER)
    elseif ended == CUTSCENE_BOSS3 then
        sendTo(director, BOSS3_TO)
        -- The boss and its own are there 1.7 to 3.9 s after the cutscene ends (as the party arrives.)
        D:after(1.5, function(d)
            D:once("last_room")
            bringLastBoss(d)
        end)
    elseif ended == CUTSCENE_END then
        director:warp_players(OUT_TO.x, OUT_TO.y, OUT_TO.z, OUT_TO.rotation)
    end
end

-- Everyone down: the fight they fell in is forgotten (the framework's: no boss pulled again till it's home and whole);
-- the last boss and everything of its fight goes, and comes back as the party does.
function onPartyWipe(director)
    D:wipe(director)
end

function onDutyRecommence(director)
    D:recommence(director)
end

-- The lifts and the other objects: each works from its own floor once the way to it is open.
function objectWorks(id, member)
    local y = member ~= nil and member.y or nil
    local function floor(low, high)
        return y == nil or (y > low and y < high)
    end
    if id == EOBJ_LIFT1 then
        return D.done.four_dead == true and floor(175, 200)
    elseif id == EOBJ_LIFT2 then
        return D.done.ten_dead == true and floor(145, 170)
    elseif id == EOBJ_HANGAR then
        return D.dead[BOSS1] == true and floor(60, 90)
    elseif id == EOBJ_ARMOUR then
        return D.dead[BOSS1] == true and not D.done.ride_ended and floor(95, 110)
    elseif id == EOBJ_LAST then
        return D.dead[BOSS2] == true and not D.done.object_last and floor(-120, -90)
    end
    return false
end

function onGimmickAccessor(director, actor_id, id, params)
    if objectWorks(id, whoIs(actor_id)) then
        director:event_action(EVENT_ACTION_OBJECT, actor_id, id)
        return
    end
    director:finish_gimmick(actor_id)
end

function isGimmickFunctional(id)
    return objectWorks(id, nil)
end

function onEventActionCast(director, actor_id, target)
    director:finish_gimmick(actor_id)
    if not objectWorks(target, whoIs(actor_id)) then
        return
    end
    if target == EOBJ_LIFT1 then
        sendTo(director, LIFT1_TO)
        if D:once("lift1") then
            D:after(4.6, function(d) d:map_effect(1, 2, 1) end)
        end
    elseif target == EOBJ_LIFT2 then
        D:once("lift2")
        sendTo(director, LIFT2_TO)
    elseif target == EOBJ_HANGAR then
        sendTo(director, HANGAR_TO)
        if D:once("hangar") then
            D:after(5.7, function(d) talk(d, 32917) end)
            D:after(12.7, function(d) talk(d, 32918) end)
            line(director, LINE_HANGAR, LINE_HANGAR_AFTER)
        end
    elseif target == EOBJ_ARMOUR then
        director:mount_player(actor_id, MOUNT_ARMOUR)
    elseif target == EOBJ_LAST then
        D:once("object_last")
        director:objective(4, 5)
        director:map_effect(16, 4096, 8192)
        director:hide_eobj(EOBJ_LAST)
        director:show_eobj(EOBJ_AFTER_LAST)
        cutscene(director, CUTSCENE_BOSS3, CUTSCENE_BOSS3_PARAMS)
    end
end

function getDebugShortcut(id)
    return 0
end
