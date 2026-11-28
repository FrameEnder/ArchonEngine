-- The Porta Decumana (ContentFinderCondition 830, territory 1048, director 0x80034E72): the trial at the end of The
-- Ultimate Weapon, two fights with a cutscene between them.
--
-- Three retail runs with Duty Support: Connor's Chronofoil capture af05916c (2026-11-09, visit 0.12, a healer with
-- the Marauder, the Thaumaturge and the Lancer), Bob's 9ba7d981 (2026-11-17, visit 0.18, a damage dealer with the
-- Marauder, the Conjurer and the Thaumaturge) and his 50e72631 (the same day, visit 0.7, a healer again);
-- `questkit chronofoil --battle` and the visits' packets. Both fights are **one clock from their pull** in every
-- run (every cast, talk and map effect the same to 0.3 s), up to the
-- places the boss's health decides, so the tables below are that clock, Connor's run's numbers. Numbers only.
--
-- The first fight (the arena round (-772, -600), 20 yalms): the boss (9088367, base 14587, BNpcName 2137, 262421
-- HP) auto-attacks and casts by its timeline (`PortaDecumanaBoss1_14587.json`); 28628 takes it away and it comes
-- out 1.7 s later at (-784, -600) facing east (SnapNextMove). Round it, all the director's:
--   * three that stand unseen in the middle (9088369 base 14589, 9088368 base 14588, 9088371 base 14591: BNpcNames
--     2140, 2141, 2139), each shown by an ActionTimeline for its casts: 14589's 28998 from the middle and 28999 then
--     29000 from the edge (11.3 to 78.5 s), 14588's 28996 and three 28997s (162.0 to 226.5), 14591's 29002 and
--     29003 (287.8 on, 29003 again every 40.9 s); each goes for good (ActionTimelines 4575, 4576, 4578) as the boss
--     uses 28994, 28993, 28995;
--   * helpers (base 9020, layout 0; retail had 25): 29001 on ten spots, twice (47.0 and 51.1 s, then 87.4 and 91.5:
--     the two sets in either order each time), 28980 from the middle every 5.1 s after the boss's 28979, three
--     or eight 28983s with each of its 28982s, twelve 28991s with its 28990;
--   * 105.8 to 135.1: its 28987, four of base 14590 (9088370) brought in at 111.9, their 28989 and its 29435 (32.7 s
--     casts) from 116.0; the map effect 4 and weather 105 at 119.9 with the lines under BNpcName 2183 (their battle
--     talk's style byte is 6), the helpers' 28511 on the party at 132.0, and at 134.9 both casts are stopped
--     (ActorControl 15 [537, 1, action]) and the four fall; 244.8 to 262.9 the same for its 28510 (27.7 s), and
--     after the 36 % line for its 28990 (31.7 s);
--   * from 301.7 its 28982 three times in 20.6 s, and again every 40.9 s until its health is below 36 % (the talk
--     came at 342.6, 367.5 and 356.4 s in the three runs, the last two rounds in, between its second and third
--     28982). Then talk 33214 and a set chain: 28628 at 5.1 s,
--     28990 at 7.6, the map effect at 17.5, the party's 28511 at 26.6, the casts stopped at 29.5, 28995 at 29.6,
--     talk 33218 at 41.8;
--   * below 22 % (62.0, 71.1 and 50.0 s into the chain) the cutscene: the director's scene 3
--     with cutscene 120 `[0, 150, 0, 0]`, everything of the first fight gone as it begins.
--
-- As the cutscene ends everyone is put by the second arena's east (-689.55, -185.73, 480.35, PrepareZoning and
-- ActorSetPos) and the second boss (9088373, base 14592, 244634 HP) stands at (-713, 480) with ten helpers. From its
-- pull (talk 33219: 39.4 and 10.4 s after the cutscene's end), `PortaDecumanaBoss2_14592.json` and:
--   * a helper's 29008, 29009, 29010 from where the boss stands with its 29005, 29006, 29007 (2.4 s apart): the way
--     it faces, then 45 degrees to either side; 29012 on every one of the party with its 29011; 29014 on one of them with its
--     29013;
--   * its 29015 (57.4 s) throws everyone to the edge, and four of base 14594 (9088375) come at the arena's four
--     corners 0.8 s later, set off for the middle 5 s on, and each uses 29016 on whoever of the party it reaches and
--     is gone (two of the four in each run; the party felled the other two);
--   * five times (83.1, 128.6, 172.4, 231.6 and 275.8 s in: 5.2 and 3.3 s after its 29020 began, or 9.4 s before)
--     ten of base 14593 (9088374) come where the boss stands, 4.8 s later leap (29018) to a row of ten through the
--     middle, 4 yalms apart, and cast 29019 (3.7 s) across it (every other one the other way along the east-west
--     row, each half its own way on a diagonal); 7.7 s on another row;
--     7.4 s on they leap again and are gone. The rows: along the east-west line, either diagonal, or two half rows
--     crossed;
--   * four helpers' 29021 (6.7 s) 20 yalms north, south, east and west of the middle, four times, the map effects
--     0-3 1.9 s into the cast;
--   * 310.3 s in, with its health at 30 % or less (28.0 %: Connor's) its last part: 28628, the 70.7 s cast 29024, the
--     lines, map effect 5 and weather 106, the helpers' 28542 on the party; above it (32.9 and 32.5 %: Bob's) 29022
--     and the same question every 7.6 s (the last part 15.5 and 7.1 s later.) It fell 54 to 63 s into 29024.
-- Its health at 0: the cast stopped (ActorControl 15 [540, 1, 29024, 1]), the closing cutscene (scene 3 with
-- cutscene 121 `[1, 18, 0, 0]`) and the duty complete.
--
-- Not known, so guessed or left out: what 29024 does if it lands (never seen: 9999999 here); what the four of base
-- 14594 do at the middle (none got there: gone); whether the party can fell the four of base 14590 early and what
-- that changes (they stood until 134.9 in both: nothing here); the first boss below the cutscene's line before the
-- chain (a floor of 1 HP here: it never fell on retail); the second fight past 325.7 s without the last part (the
-- question again every 7.6 s, 29022 every other time); what ActorControls 20, 49, 62 and 70 on the bosses and
-- statuses 3009 to 3012 do (sent and given as retail did: 3009 to 3011 on the boss from the first). After a wipe the fight its party fell in begins again
-- (retail's wasn't seen.)

-- Written with the dungeon framework (content/lib/Dungeon.lua, loaded first): its two bosses, their pulls, the
-- helpers and the clock tables (`Dungeon:schedule`) are the framework's; the fights' own steps are below.

-- Which way players face as they come in (retail's, facing the boss in the west.)
ENTRANCE_ROTATION = -1.5708

BOSS_1 = 9088367
THREE_A = 9088369 -- base 14589: 28998, 28999, 29000
THREE_B = 9088368 -- base 14588: 28996, 28997
THREE_C = 9088371 -- base 14591: 29002, 29003
FOUR = 9088370 -- base 14590
BOSS_2 = 9088373
ROW = 9088374 -- base 14593
CORNER = 9088375 -- base 14594
HELPER = 0
-- (Retail had 25 helpers in the first fight and ten in the second; the most at work at once is twelve.)
HELPER_COUNT = 13

NAME_BOSS = 2136
NAME_ALLY = 2183
NAME_LAST = 2664
STYLE_ALLY = 6
STYLE_LAST = 7

FLOOR_1 = -400.0
MIDDLE_1 = { x = -772.0, y = -400.0, z = -600.0 }
BOSS_1_SPOT = { x = -784.0, z = -600.0, rotation = 1.5708 }
-- (Both arenas are 20 yalms round; whoever is further than this from the middle isn't in the fight.)
ARENA_REACH = 30.0
FLOOR_2 = -185.66
MIDDLE_2 = { x = -704.0, y = -185.66, z = 480.0 }
BOSS_2_START = { x = -713.0, z = 480.0, rotation = 1.5708 }
-- Where retail put the player after the cutscene (the party's members stood a yalm or two west of there.)
SECOND_ARENA = { x = -689.55, y = -185.73, z = 480.35, rotation = -1.5708 }

-- The cutscene between the fights and the closing one (scene 3: the cutscene, then these.)
CUTSCENE_MIDDLE = 120
CUTSCENE_MIDDLE_PARAMS = { 0, 150, 0, 0 }
SECOND_COMES_AFTER = 2.5
CUTSCENE_END = 121
CUTSCENE_END_PARAMS = { 1, 18, 0, 0 }

-- Its health, in whole percent as the server gives it, below these: the first fight's chain (once the rounds of 28982
-- have begun; three runs: the talk came 1.6 s after 35.95 %, as 36.3 became 35.97 %, and 3.2 s after 35.94 %, the
-- third in the middle of a round it had begun at 36.7 %) and cutscene (21.96, 21.7 and 21.9 % 0.1 to 2.4 s
-- before), the second fight's last part (looked at 310.3 s in and every 7.6 s from then: 32.9, 30.7 and 32.5 %
-- weren't enough; 28.0, 29.4, 29.7 and 29.5 % were.)
STATUSES_FROM_THE_FIRST = { 3009, 3010, 3011 }
CHAIN_AT = 36
CUTSCENE_AT = 22
LAST_PART_AT = 30

CONTROL_STOP_CAST = 15
CAST_STOPPED = 537
CAST_STOPPED_DEAD = 540
CONTROL_APPEAR_TIMELINE = 142
WEATHER_FADE = 3.0
WEATHER_OWN = 2

ACTION_AWAY = 28628
AWAY_FOR = 1.7
ACTION_ROUND = 28982
ACTION_ROUND_HELPER = 28983
ACTION_ROW_LEAP = 29018
ACTION_ROW_CAST = 29019
ACTION_CORNER = 29016
ACTION_PARTY_SHIELD = 28511
ACTION_HELPER_SHIELD = 28992
ACTION_PARTY_LAST = 28542
ACTION_STILL_ABOVE = 29022
ACTION_LAST = 29024

-- The helpers' spots (x, z.)
SPOTS_29001 = {
    { { -772.0, -600.0 }, { -789.0, -600.0 }, { -780.5, -614.7 }, { -780.5, -585.3 }, { -779.0, -596.0 },
      { -772.0, -608.0 }, { -765.1, -596.0 }, { -763.5, -614.7 }, { -763.5, -585.3 }, { -755.0, -600.0 } },
    { { -772.0, -600.0 }, { -786.7, -608.5 }, { -786.7, -591.5 }, { -779.0, -604.0 }, { -772.0, -617.0 },
      { -772.0, -592.0 }, { -772.0, -583.0 }, { -765.1, -604.0 }, { -757.3, -608.5 }, { -757.3, -591.5 } },
}
-- With a 28982: three spots 15 yalms from the middle and the middle (three ways round seen), or eight 17 yalms out.
SPOTS_28983_THREE = {
    { { -772.0, -600.0 }, { -781.6, -611.5 }, { -777.1, -585.9 }, { -757.2, -602.6 } },
    { { -772.0, -600.0 }, { -785.0, -592.5 }, { -772.0, -615.0 }, { -759.0, -592.5 } },
    { { -772.0, -600.0 }, { -786.8, -602.6 }, { -766.9, -585.9 }, { -762.4, -611.5 } },
}
SPOTS_28983_EIGHT = { { -789.0, -600.0 }, { -784.0, -612.0 }, { -784.0, -588.0 }, { -772.0, -617.0 },
                      { -772.0, -583.0 }, { -760.0, -612.0 }, { -760.0, -588.0 }, { -755.0, -600.0 } }
SPOTS_28991 = { { -788.0, -600.0 }, { -785.0, -613.0 }, { -785.0, -587.0 }, { -777.0, -605.0 }, { -777.0, -595.0 },
                { -772.0, -616.0 }, { -772.0, -584.0 }, { -767.0, -605.0 }, { -767.0, -595.0 }, { -759.0, -613.0 },
                { -759.0, -587.0 }, { -756.0, -600.0 } }
-- Where the four of base 14590 came (Connor's run; Bob's had two of them elsewhere near the middle.)
-- The four come each where one of the party is held (Bob's Chronofoil captures 9ba7d981, 6320ae56 and 50e72631: each of
-- them on a member's spot, to the tenth of a yalm, and each with that member as its target 1.07 s into its cast);
-- these spots only for one of them with no one to hold.
SPOTS_FOUR = { { -777.0, -600.0 }, { -775.8, -596.6 }, { -772.0, -605.0 }, { -769.2, -605.5 } }
FOUR_Y = -399.73
-- The edge spots the three cast from, 15 yalms from the middle and facing it (one at random each time: the runs
-- differed.)
EDGE_SPOTS = { { -772.0, -585.0, 3.1416 }, { -772.0, -615.0, 0.0 }, { -787.0, -600.0, 1.5708 },
               { -757.0, -600.0, -1.5708 } }
-- The second arena: the corners the four of base 14594 come at, the helpers' 29021 spots (x, z, facing.)
SPOTS_CORNER = { { -718.14, 465.86 }, { -718.14, 494.14 }, { -689.86, 465.86 }, { -689.86, 494.14 } }
SPOTS_29021 = { { -684.0, 480.0, -1.5708 }, { -704.0, 460.0, 0.0 }, { -704.0, 500.0, 3.1416 },
                { -724.0, 480.0, 1.5708 } }

-- The first fight's clock: { seconds from the pull, what, ... }.
--   "talk", text, name, ms, style        "control", layout, category, { params }
--   "timeline", layout, ActionTimeline   "cast", layout, action, "middle" | "edge" | "same"
--   "helpers", action, spots             "map", index, timeline, state, weather
--   "back", (the boss out again)         "stop", layout     "fn", function
FIRST = {
    { 0.0, "talk", 33200, NAME_BOSS, 6000 },
    { 9.8, "back" },
    { 10.5, "talk", 33207, NAME_BOSS, 4000 },
    { 10.5, "control", BOSS_1, 70, { 8201055 } },
    { 10.6, "timeline", THREE_A, 7747 },
    { 11.3, "cast", THREE_A, 28998, "middle" },
    { 21.3, "timeline", THREE_A, 7737 },
    { 25.7, "cast", THREE_A, 28999, "edge" },
    { 33.1, "cast", THREE_A, 29000, "same" },
    { 38.3, "timeline", THREE_A, 7737 },
    { 47.0, "helpers", 29001, "either" },
    { 51.1, "helpers", 29001, "other" },
    { 66.0, "cast", THREE_A, 28999, "edge" },
    { 73.4, "cast", THREE_A, 29000, "same" },
    { 78.5, "timeline", THREE_A, 7737 },
    { 87.4, "helpers", 29001, "either" },
    { 91.5, "helpers", 29001, "other" },
    { 105.1, "back" },
    { 105.7, "talk", 33208, NAME_BOSS, 5000 },
    { 105.9, "fn", "markParty" },
    { 109.8, "fn", "gaolParty" },
    { 110.8, "gain", BOSS_1, 325 },
    { 111.3, "fn", "bringFour" },
    { 111.9, "talk", 33209, NAME_BOSS, 4000 },
    { 116.0, "fn", "fourCast" },
    { 119.9, "talk", 33235, NAME_ALLY, 5000, STYLE_ALLY },
    { 119.9, "map", 4, 2, 1, 105 },
    { 125.9, "talk", 33204, NAME_ALLY, 6000, STYLE_ALLY },
    { 125.9, "fn", "allyGathers" },
    { 132.0, "fn", "shieldParty" },
    { 132.9, "map", 4, 8, 4, WEATHER_OWN },
    { 134.9, "stop", BOSS_1 },
    { 134.9, "fn", "fourFall" },
    { 134.9, "fn", "lineAsItStops" },
    { 135.0, "goes", THREE_A, 4575 },
    { 135.9, "control", BOSS_1, 62, { 1 } },
    { 147.3, "talk", 33205, NAME_BOSS, 5000 },
    { 153.3, "talk", 33206, NAME_BOSS, 5000 },
    { 160.2, "back" },
    { 160.6, "talk", 33201, NAME_BOSS, 5000 },
    { 160.6, "control", BOSS_1, 70, { 8201054 } },
    { 160.8, "timeline", THREE_B, 4561 },
    { 162.0, "cast", THREE_B, 28996, "middle" },
    { 170.1, "timeline", THREE_B, 4573 },
    { 176.9, "helpers", 28980, "middle" },
    { 179.0, "timeline", THREE_B, 4561 },
    { 180.2, "cast", THREE_B, 28997, "edge" },
    { 182.0, "helpers", 28980, "middle" },
    { 187.1, "helpers", 28980, "middle" },
    { 187.3, "timeline", THREE_B, 4573 },
    { 192.2, "helpers", 28980, "middle" },
    { 194.0, "timeline", THREE_B, 4561 },
    { 195.2, "cast", THREE_B, 28997, "edge" },
    { 197.2, "helpers", 28980, "middle" },
    { 202.3, "timeline", THREE_B, 4573 },
    { 216.2, "helpers", 28980, "middle" },
    { 218.2, "timeline", THREE_B, 4561 },
    { 219.4, "cast", THREE_B, 28997, "edge" },
    { 221.3, "helpers", 28980, "middle" },
    { 226.3, "helpers", 28980, "middle" },
    { 226.5, "timeline", THREE_B, 4573 },
    { 231.4, "helpers", 28980, "middle" },
    { 233.1, "fn", "stopHelpers" },
    { 234.9, "back" },
    { 235.6, "talk", 33202, NAME_BOSS, 5000 },
    { 244.7, "talk", 33203, NAME_BOSS, 5000 },
    { 249.7, "map", 4, 2, 1, 105 },
    { 253.7, "talk", 33210, NAME_ALLY, 6000, STYLE_ALLY },
    { 253.7, "fn", "allyGathers" },
    { 259.8, "fn", "shieldParty" },
    { 261.7, "map", 4, 8, 4, WEATHER_OWN },
    { 262.7, "stop", BOSS_1 },
    { 262.75, "goes", THREE_B, 4576 },
    { 263.7, "control", BOSS_1, 49, { 16 } },
    { 275.1, "talk", 33211, NAME_BOSS, 4000 },
    { 280.2, "talk", 33212, NAME_BOSS, 4000 },
    { 286.0, "back" },
    { 286.4, "talk", 33213, NAME_BOSS, 5000 },
    { 286.4, "control", BOSS_1, 70, { 8201056 } },
    { 286.6, "timeline", THREE_C, 7747 },
    { 287.8, "cast", THREE_C, 29002, "middle" },
    { 297.9, "timeline", THREE_C, 7737 },
}
-- The rounds of 28982 (the boss's timeline: 301.7 s, 4.3 and 20.6 s later, and every 40.9 s): what goes with each
-- round, from its first 28982.
ROUND_FIRST = 301.7
ROUND_EVERY = 40.9
ROUND = {
    { 13.6, "timeline", THREE_C, 7747 },
    { 14.9, "cast", THREE_C, 29003, "middle" },
    { 21.0, "timeline", THREE_C, 7737 },
}
-- From talk 33214.
CHAIN = {
    { 0.0, "talk", 33214, NAME_BOSS, 5000 },
    { 6.8, "back" },
    { 7.6, "talk", 33215, NAME_BOSS, 6000 },
    { 7.6, "helpers", 28991, "twelve" },
    { 14.6, "talk", 33216, NAME_BOSS, 4000 },
    { 17.5, "map", 4, 2, 1, 105 },
    { 20.5, "talk", 33217, NAME_ALLY, 6000, STYLE_ALLY },
    { 20.5, "fn", "allyGathers" },
    { 26.6, "fn", "shieldParty" },
    { 27.5, "map", 4, 8, 4, WEATHER_OWN },
    { 29.5, "stop", BOSS_1 },
    { 29.5, "fn", "stopHelpers" },
    { 29.6, "goes", THREE_C, 4578 },
    { 30.4, "control", BOSS_1, 62, { 0 } },
    { 41.8, "talk", 33218, NAME_BOSS, 5000 },
    { 41.8, "control", BOSS_1, 70, { 8201058 } },
    { 41.8, "fn", "cutsceneMayCome" },
}
-- The second fight's clock, from its pull.
SECOND = {
    { 0.0, "talk", 33219, NAME_BOSS, 5000 },
    { 16.5, "fn", "lineFromBoss", 29008 },
    { 18.9, "fn", "lineFromBoss", 29009 },
    { 21.3, "fn", "lineFromBoss", 29010 },
    { 27.7, "fn", "onEveryone", 29012 },
    { 38.7, "talk", 33220, NAME_BOSS, 6000 },
    { 38.7, "control", BOSS_2, 70, { 8201057 } },
    { 52.7, "back" },
    { 58.2, "fn", "bringCorners" },
    { 71.6, "fn", "onSomeone", 29014 },
    { 83.1, "fn", "bringRows" },
    { 106.0, "talk", 33221, NAME_BOSS, 6000 },
    { 113.0, "talk", 33222, NAME_BOSS, 6000 },
    { 119.2, "fn", "fourSides" },
    { 128.6, "fn", "bringRows" },
    { 133.5, "fn", "onEveryone", 29012 },
    { 144.7, "fn", "onSomeone", 29014 },
    { 156.5, "back" },
    { 170.5, "fn", "lineFromBoss", 29008 },
    { 172.4, "fn", "bringRows" },
    { 172.8, "fn", "lineFromBoss", 29009 },
    { 175.2, "fn", "lineFromBoss", 29010 },
    { 188.8, "fn", "fourSides" },
    { 222.2, "fn", "fourSides" },
    { 231.6, "fn", "bringRows" },
    { 236.6, "fn", "onEveryone", 29012 },
    { 247.8, "fn", "onSomeone", 29014 },
    { 259.5, "back" },
    { 273.7, "fn", "lineFromBoss", 29008 },
    { 275.8, "fn", "bringRows" },
    { 276.0, "fn", "lineFromBoss", 29009 },
    { 278.4, "fn", "lineFromBoss", 29010 },
    { 292.1, "fn", "fourSides" },
    { 310.3, "fn", "lastPartOrNot" },
}
-- Seconds between looks at its health from 310.3 s on while it's above the line: the last part began 15.5 s after
-- the first look in Bob's run of 2026-11-17 (9ba7d981: 30.7 % at 7.6 s, 29.4 % at 15.5) and 7.1 s after it in his run
-- of 2026-11-18 (6320ae56: 29.5 % by then).
STILL_ABOVE_EVERY = 7.6
last_part_asks = 0
-- From the second boss's last 28628.
LAST = {
    { 1.7, "back" },
    { 4.2, "talk", 33223, NAME_BOSS, 6000 },
    { 11.2, "talk", 33224, NAME_BOSS, 6000 },
    { 20.3, "talk", 33225, NAME_ALLY, 5000, STYLE_ALLY },
    { 20.3, "map", 5, 2, 1, 106 },
    { 26.3, "talk", 33226, NAME_ALLY, 6000, STYLE_ALLY },
    { 26.3, "fn", "allyGathers" },
    { 32.3, "map", 5, 8, 4, WEATHER_OWN },
    { 32.4, "fn", "lastShield" },
    { 53.3, "talk", 33227, NAME_LAST, 4000, STYLE_LAST },
    { 63.4, "talk", 33228, NAME_LAST, 4000, STYLE_LAST },
}
-- The second boss's timeline's last part (`restart_timeline`.)
PART_LAST = 33223
-- The first's: its chain, and nothing.
PART_CHAIN = 33214
PART_CUTSCENE = 33218

-- The rows of base 14593 (from their coming): they leap, cast, leap, cast, leap and go. Which rows the five times
-- had: "line" is the east-west row, "slant" either diagonal, "cross" two half rows crossed.
ROWS_LEAP = { 4.8, 12.5, 19.9 }
ROWS_CAST_AFTER_LEAP = 1.4
ROWS_GONE = 22.2
ROWS_FADE_AFTER_LEAP = 1.02
ROWS_OF = { { "line", "slant" }, { "slant", "line" }, { "slant", "cross" }, { "slant", "line" }, { "slant", "cross" } }
ROW_SPACING = 4.0
ROW_Y = -185.53
-- The crossed half rows as retail's stood (x, z, facing.)
ROW_CROSS = { { -687.0, 491.3, -2.356 }, { -692.7, 496.9, -2.356 }, { -712.5, 460.2, 0.785 }, { -718.2, 465.8, 0.785 },
              { -723.8, 471.5, 0.785 }, { -718.2, 494.1, 2.356 }, { -712.5, 499.8, 2.356 }, { -723.8, 488.5, 2.356 },
              { -687.0, 468.7, -0.785 }, { -692.7, 463.0, -0.785 } }

-- The four of base 14594: they set off for the middle 5.0 s after coming, 1.7 yalms a second, and use 29016 on
-- whoever of the party is within 2 yalms.
FAN_ANGLE = 0.7854
CORNER_SETS_OFF = 5.0
CORNER_SPEED = 1.7
CORNER_REACH = 2.0
CORNER_WALK = 20 -- ActorMove's speed byte: a walk

-- The Duty Support members' lines (NpcYell): 14404 at the first pull (the Thaumaturge), 14407 or 14408 as the first
-- long cast is stopped (the Lancer; the Conjurer), 14409 and 13980 0.7 and 0.9 s after the last part's 28542 (the
-- Marauder, the Thaumaturge.)
LINE_PULL = 14404
LINE_STOPPED_MELEE = 14407
LINE_STOPPED_HEALER = 14408
LINE_LAST_TANK = 14409
LINE_LAST_RANGED = 13980

DUNGEON = Dungeon.new()
D = DUNGEON

part = "first-wait"
attempts_of = "first"
-- The helpers of the fight under way (the framework hands them out: `D.helpers`.)
helpers = {}
boss_1 = nil
boss_2 = nil
three = {}
four = {}
four_holds = {}
corners = {}
rounds = 0
rows = 0
last_edge = nil
last_set = nil
cutscene_may_come = false
boss_at = nil
boss_health = nil
boss_target = nil
fan_facing = nil
rounds_begun = false
last_three = nil

function floor()
    return (attempts_of == "first") and FLOOR_1 or FLOOR_2
end

function middle()
    return (attempts_of == "first") and MIDDLE_1 or MIDDLE_2
end

-- The party in the arena: the player and their party's members (not one left behind or yet to arrive.)
D.in_the_party = function(actor)
    local at = middle()
    local dx, dz = actor.x - at.x, actor.z - at.z
    return dx * dx + dz * dz <= ARENA_REACH * ARENA_REACH
end

function bringHelpers(director)
    local at = middle()
    helpers = {}
    for i = 1, HELPER_COUNT do
        -- (The simulator's spawns return nothing: the layout ID stands in there.)
        helpers[i] = director:spawn_bnpc(HELPER, at.x, floor(), at.z, 0.0) or HELPER
        director:hold_bnpc(helpers[i])
    end
    D.helpers = helpers
    D.next_helper = 0
end

function bringFirst(director)
    attempts_of = "first"
    part = "first-wait"
    boss_1 = director:spawn_bnpc(BOSS_1) or BOSS_1
    -- Statuses 3009 to 3011 are on it from the first, with no timer (Kawari Witness in Bob's run of 2026-11-18: on
    -- it as it was first seen; its 28976 to 28978 gave nothing); its 28993 to 28995 take them off one by one.
    for _, status in ipairs(STATUSES_FROM_THE_FIRST) do
        director:gain_effect(boss_1, status, 0, 0)
    end
    -- Never to 0 (retail's went to the cutscene at 21 %.)
    director:set_health_floor(BOSS_1, 1)
    three = {}
    for _, layout in ipairs({ THREE_A, THREE_B, THREE_C }) do
        three[layout] = director:spawn_bnpc(layout) or layout
        director:hold_bnpc(three[layout])
        -- Nobody's to hit them: retail's client let no one target them (display flags 393224) and no action of
        -- retail's ever named them, so they were there to cast and to go as each part ends (Bob's Chronofoil captures
        -- 9ba7d981, 6320ae56). Bob's bots killed all three within 27 s of the pull (2026-11-28), and Titan's going and
        -- Garuda's part never came. The server's side alone: the client shows them so already.
        director:set_targetable(three[layout], false, true)
    end
    bringHelpers(director)
end

function bringSecond(director)
    attempts_of = "second"
    part = "second-wait"
    boss_2 = director:spawn_bnpc(BOSS_2) or BOSS_2
    bringHelpers(director)
end

function reset()
    D.later = {}
    four = {}
    corners = {}
    rounds = 0
    rows = 0
    last_edge = nil
    last_set = nil
    last_three = nil
    cutscene_may_come = false
    rounds_begun = false
end

-- The next helper in turn, put at a spot on the arena's floor.
function helperAt(director, x, z, facing)
    return D:helperAt(director, x, floor(), z, facing)
end

function helpersCast(director, action, spots)
    D:helpersCast(director, action, spots, floor())
end

function stopHelpers(director)
    for _, helper in ipairs(helpers) do
        director:stop_cast(helper, CAST_STOPPED)
    end
end

-- As each cutscene begins the helpers are let go (ActorControl 417 `[0xE0000000]`, `release_bnpc`), all of them at once
-- 0.04 s into the scene and taken away 0.4 s after (the 15 of the first fight and the 10 of the second, Bob's
-- 9ba7d981.)
function letHelpersGo(director)
    for _, helper in ipairs(helpers) do
        director:release_bnpc(helper)
    end
end

function someoneOfTheParty()
    if #D.party == 0 then
        return nil
    end
    return D.party[math.random(#D.party)]
end

-- The helpers' 28988 on each of the party as the boss's 28987 begins (no cast.)
function markParty(director)
    for _, member in ipairs(D.party) do
        local helper = helperAt(director, MIDDLE_1.x, MIDDLE_1.z)
        if helper ~= nil then
            director:actor_use_action(helper, 28988, member.id)
        end
    end
end

-- A helper's 28511 on the whole party (aimed at itself, each member a target: `on_party`) and 28992 on itself, 2.9 s before the long cast is stopped.
-- The ally's help, on the whole party from no one: as the ally's line comes (33204, 33210, 33217, 33226) status 3013
-- with no timer, its count 1, then 2 2.01 s on and 3 4.01 s on; as the helper's 28511 or 28542 lands it goes and 3014
-- comes, 5 s after 28511 (0.49 s after it) and 50 s after 28542 (0.59 s after it, till the duty is done). The player
-- and the three Duty Support members every time in Bob's Chronofoil captures 9ba7d981 and 6320ae56.
STATUS_ALLY_GATHERS = 3013
STATUS_ALLY_SHIELD = 3014
ALLY_GATHERS_EVERY = { 0.0, 2.01, 4.01 }
function allyGathers(director)
    for count, at in ipairs(ALLY_GATHERS_EVERY) do
        D:after(at, function(d)
            for _, member in ipairs(D.party) do
                d:gain_effect(member.id, STATUS_ALLY_GATHERS, count, 0)
            end
        end)
    end
end

function allyShields(director, after, seconds)
    D:after(after, function(d)
        for _, member in ipairs(D.party) do
            d:lose_effect(member.id, STATUS_ALLY_GATHERS)
            d:gain_effect(member.id, STATUS_ALLY_SHIELD, 1, seconds)
        end
    end)
end

function shieldParty(director)
    local helper = helperAt(director, MIDDLE_1.x, MIDDLE_1.z)
    if helper == nil then
        return
    end
    director:actor_use_action(helper, ACTION_PARTY_SHIELD)
    director:actor_use_action(helper, ACTION_HELPER_SHIELD)
    allyShields(director, 0.49, 5.0)
end

function lastShield(director)
    local helper = helperAt(director, MIDDLE_2.x, MIDDLE_2.z)
    if helper ~= nil then
        director:actor_use_action(helper, ACTION_PARTY_LAST)
    end
    allyShields(director, 0.59, 50.0)
    director:party_npc_yell("tank", LINE_LAST_TANK, 0.7)
    director:party_npc_yell("ranged", LINE_LAST_RANGED, 0.9)
end

function lineAsItStops(director)
    director:party_npc_yell("healer", LINE_STOPPED_HEALER, 0, "melee", LINE_STOPPED_MELEE)
end

-- As the boss's 28987 lands the whole party is gaoled: status 3249 on each (no timer), untargetable and doing nothing
-- till the four fall (`modifiers::PORTA_DECUMANA_GAOL`; the player, the three Duty Support members, every run.)
STATUS_GAOL = 3249
-- And the boss's own status 325 from then till they fall (gained 1.0 s after, lost with them.)
STATUS_BOSS_HOLDS = 325
gaoled = {}
function gaolParty(director)
    gaoled = {}
    for _, member in ipairs(D.party) do
        director:gain_effect(member.id, STATUS_GAOL, 0, 0)
        gaoled[#gaoled + 1] = { id = member.id, x = member.x, z = member.z }
    end
end

function bringFour(director)
    four = {}
    four_holds = {}
    for i, spot in ipairs(SPOTS_FOUR) do
        local held = gaoled[i]
        local x, z = spot[1], spot[2]
        if held ~= nil then
            x, z = held.x, held.z
        end
        four[i] = director:spawn_bnpc(FOUR, x, FOUR_Y, z, 0.0) or FOUR
        four_holds[i] = held
        director:hold_bnpc(four[i])
        director:appear_bnpc(four[i], CONTROL_APPEAR_TIMELINE, 0.6)
    end
end

-- Their 28989; 0.89 s into it each has the one it holds on its list (the player's ActorControl 503 naming theirs),
-- and 0.18 s after turns to them (SetTarget, all four at once.)
FOUR_HATES_AFTER = 0.89
FOUR_TURN_AFTER = 1.07
function fourCast(director)
    for _, one in ipairs(four) do
        director:actor_use_action(one, 28989)
    end
    local these, holds = four, four_holds
    D:after(FOUR_HATES_AFTER, function(d)
        for i, one in ipairs(these) do
            if holds[i] ~= nil then
                d:set_hate(one, holds[i].id)
            end
        end
    end)
    D:after(FOUR_TURN_AFTER, function(d)
        for i, one in ipairs(these) do
            d:aim_bnpc(one, holds[i] and holds[i].id or 0)
        end
    end)
end

-- Their casts stopped as the boss's, and they fall: the party is free and the boss's 325 gone, that moment.
function fourFall(director)
    for _, one in ipairs(four) do
        director:stop_cast(one, CAST_STOPPED_DEAD)
        director:kill_bnpc(one)
    end
    four = {}
    four_holds = {}
    for _, held in ipairs(gaoled) do
        director:lose_effect(held.id, STATUS_GAOL)
    end
    gaoled = {}
    director:lose_effect(boss_1 or BOSS_1, STATUS_BOSS_HOLDS)
end

-- One of the three goes for good as its part ends: put where the boss comes out, facing east (ActorSetPos at
-- (-784, -399.735, -600)), and its going timeline 0.09 s later (all three, both of Bob's runs.)
GOES_SPOT = { x = -784.0, y = -399.735, z = -600.0, rotation = 1.5708 }
GOES_TIMELINE_AFTER = 0.09
function threeGoes(director, layout, timeline)
    director:warp_bnpc(three[layout] or layout, GOES_SPOT.x, GOES_SPOT.y, GOES_SPOT.z, GOES_SPOT.rotation)
    D:after(GOES_TIMELINE_AFTER, function(d)
        d:play_action_timeline(three[layout] or layout, timeline)
    end)
end

function cutsceneMayCome(director)
    cutscene_may_come = true
end

-- One of the three casts: from the middle, from an edge spot (not the last one's), or where it last cast.
function threeCast(director, layout, action, where)
    local x, z, facing = MIDDLE_1.x, MIDDLE_1.z, 1.5708
    if where == "edge" then
        local pick = math.random(#EDGE_SPOTS)
        if pick == last_edge then
            pick = pick % #EDGE_SPOTS + 1
        end
        last_edge = pick
    end
    if where ~= "middle" and last_edge ~= nil then
        local spot = EDGE_SPOTS[last_edge]
        x, z, facing = spot[1], spot[2], spot[3]
    end
    director:warp_bnpc(layout, x, FLOOR_1, z, facing)
    director:actor_use_action(three[layout] or layout, action)
end

-- A helper's line from where the boss stands: 29008 the way the boss faces (towards whom it's fighting), 29009 45
-- degrees to one side of that and 29010 45 to the other (all nine in the three runs; 29010 came before 29009 once.)
function lineFromBoss(director, action)
    local at = boss_at or MIDDLE_2
    if action == 29008 or fan_facing == nil then
        local whom = nil
        for _, member in ipairs(D.party) do
            if member.id == boss_target then
                whom = member
            end
        end
        whom = whom or someoneOfTheParty()
        fan_facing = (whom ~= nil) and math.atan(whom.x - at.x, whom.z - at.z) or 0.0
    end
    local facing = fan_facing
    if action == 29009 then
        facing = fan_facing - FAN_ANGLE
    elseif action == 29010 then
        facing = fan_facing + FAN_ANGLE
    end
    local helper = helperAt(director, at.x, at.z, facing)
    if helper ~= nil then
        director:actor_use_action(helper, action)
    end
end

-- A helper's cast on each of the party (29012), where they stand.
function onEveryone(director, action)
    D:helpersOnEach(director, action, floor())
end

-- A helper's cast on one of them (29014.)
function onSomeone(director, action)
    local member = someoneOfTheParty()
    if member == nil then
        return
    end
    local helper = helperAt(director, member.x, member.z)
    if helper ~= nil then
        director:actor_use_action(helper, action, member.id)
    end
end

function fourSides(director)
    helpersCast(director, 29021, SPOTS_29021)
    D:after(1.9, function(d)
        for index = 0, 3 do
            d:map_effect(index, 2, 4)
        end
    end)
end

-- The four of base 14594 at the corners; 5 s on they set off for the middle.
function bringCorners(director)
    corners = {}
    for i, spot in ipairs(SPOTS_CORNER) do
        local id = director:spawn_bnpc(CORNER, spot[1], FLOOR_2, spot[2], 0.0) or CORNER
        director:hold_bnpc(id)
        corners[i] = { id = id, index = i, x = spot[1], z = spot[2], off_at = D.ticks + CORNER_SETS_OFF * 2, gone = false }
    end
    D:after(CORNER_SETS_OFF, function(d)
        for _, corner in ipairs(corners) do
            if not corner.gone then
                d:move_bnpc(corner.id, MIDDLE_2.x, FLOOR_2, MIDDLE_2.z, 0.0, CORNER_WALK)
            end
        end
    end)
end

-- One of the four reaches the party: as retail's did, it bursts round itself (29016 at itself, an area on whoever's
-- near: ActorControl 2 `[2, 2]` with it, Kill 1.0 s later, its death's log 2 s after that and its fading 6 s after),
-- and the one at the corner across from it fades away that moment (ActorControl 47 and 607 `[itself, 1, 0, 100]`
-- 0.09 s before the burst), the two pairs (1, 4) and (2, 3) of `SPOTS_CORNER` in Bob's 9ba7d981 and 6320ae56. Kawari
-- aimed 29016 at the member and took the corner away, and the other two stayed for the party to fell.
CORNER_ACROSS = { 4, 3, 2, 1 }
CONTROL_CORNER_GOES = 47
CONTROL_MODE = 2
function cornerReaches(director, corner)
    corner.gone = true
    local across = corners[CORNER_ACROSS[corner.index]]
    if across ~= nil and not across.gone then
        across.gone = true
        director:actor_control(across.id, CONTROL_CORNER_GOES, {})
        director:fade_out_bnpc(across.id)
    end
    director:actor_control(corner.id, CONTROL_MODE, { 2, 2 })
    director:actor_use_action(corner.id, ACTION_CORNER)
    D:after(1.0, function(d) d:kill_bnpc(corner.id) end)
end

-- Where each of the four is by now, and whom it has reached.
function watchCorners(director)
    for _, corner in ipairs(corners) do
        if not corner.gone then
            local walked = math.max(0, (D.ticks - corner.off_at) * 0.5) * CORNER_SPEED
            local dx, dz = MIDDLE_2.x - corner.x, MIDDLE_2.z - corner.z
            local whole = math.sqrt(dx * dx + dz * dz)
            local share = math.min(1.0, walked / whole)
            local x, z = corner.x + dx * share, corner.z + dz * share
            for _, member in ipairs(D.party) do
                local mx, mz = member.x - x, member.z - z
                if not corner.gone and mx * mx + mz * mz <= CORNER_REACH * CORNER_REACH then
                    cornerReaches(director, corner)
                end
            end
            -- At the middle (never seen on retail): gone.
            if not corner.gone and share >= 1.0 then
                corner.gone = true
                director:despawn_bnpc(corner.id)
            end
        end
    end
end

-- Ten spots of a row through the middle (x, z, facing): `kind` "line", "slant" or "cross".
function rowSpots(kind)
    if kind == "cross" then
        return ROW_CROSS
    end
    local angle = 1.5708
    if kind == "slant" then
        angle = (math.random(2) == 1) and 0.7854 or 2.3562
    end
    local spots = {}
    for i = 0, 9 do
        local along = (i - 4.5) * ROW_SPACING
        -- Across the row: every other one the other way along the east-west row, each half its own way on a
        -- diagonal (retail's.)
        local one_way
        if kind == "line" then
            one_way = (i % 2 == 0)
        else
            one_way = (i < 5)
        end
        local facing = angle + (one_way and -1.5708 or 1.5708)
        table.insert(spots, { MIDDLE_2.x + along * math.sin(angle), MIDDLE_2.z + along * math.cos(angle), facing })
    end
    return spots
end

-- Ten of base 14593, where the boss stands.
function bringRows(director)
    rows = rows + 1
    local kinds = ROWS_OF[(rows - 1) % #ROWS_OF + 1]
    local at = boss_at or BOSS_2_START
    local ten = {}
    for i = 1, 10 do
        ten[i] = director:spawn_bnpc(ROW, at.x, ROW_Y, at.z, 0.0) or ROW
        director:hold_bnpc(ten[i])
    end
    for n, at in ipairs(ROWS_LEAP) do
        D:after(at, function(d)
            local spots = kinds[n] and rowSpots(kinds[n]) or nil
            for i, one in ipairs(ten) do
                d:actor_use_action(one, ACTION_ROW_LEAP)
                if spots ~= nil then
                    d:jump_bnpc(one, spots[i][1], ROW_Y, spots[i][2], spots[i][3])
                end
            end
        end)
        if kinds[n] ~= nil then
            D:after(at + ROWS_CAST_AFTER_LEAP, function(d)
                for _, one in ipairs(ten) do
                    d:actor_use_action(one, ACTION_ROW_CAST)
                end
            end)
        end
    end
    -- They fade (ActorControl 607 `[itself, 1, 0, 100]`) 1.02 s after their last leap and are gone 1.3 s after that
    -- (every one of the 50 in Bob's 9ba7d981.)
    D:after(ROWS_LEAP[#ROWS_LEAP] + ROWS_FADE_AFTER_LEAP, function(d)
        for _, one in ipairs(ten) do
            d:fade_out_bnpc(one)
        end
    end)
    D:after(ROWS_GONE, function(d)
        for _, one in ipairs(ten) do
            d:despawn_bnpc(one)
        end
    end)
end

-- 310.3 s into the second fight, and every 7.6 s while its health is above the line: the last part. Above it, 29022
-- the first time (and every other time after, a guess: retail's two runs above the line had one 29022 each, and the
-- last part 7.1 and 15.5 s after it began.)
function lastPartOrNot(director)
    last_part_asks = 0
    askLastPart(director)
end

function askLastPart(director)
    if part ~= "second" then
        return
    end
    if boss_health ~= nil and boss_health < LAST_PART_AT then
        part = "last"
        director:restart_timeline(BOSS_2, PART_LAST)
        schedule(director, LAST, "last")
    else
        if boss_2 ~= nil and last_part_asks % 2 == 0 then
            director:actor_use_action(boss_2, ACTION_STILL_ABOVE)
        end
        last_part_asks = last_part_asks + 1
        D:after(STILL_ABOVE_EVERY, askLastPart)
    end
end

-- One row of a clock table (see FIRST.)
function step(director, row)
    local what = row[2]
    local boss = (attempts_of == "first") and BOSS_1 or BOSS_2
    if what == "talk" then
        director:battle_talk(boss, row[4], row[3], 0, row[5], nil, row[6] or 0)
    elseif what == "control" then
        director:bnpc_control(row[3], row[4], row[5])
    elseif what == "timeline" then
        director:play_action_timeline(row[3], row[4])
    elseif what == "goes" then
        threeGoes(director, row[3], row[4])
    elseif what == "gain" then
        director:gain_effect(row[3] == BOSS_1 and (boss_1 or BOSS_1) or row[3], row[4], 0, 0)
    elseif what == "cast" then
        threeCast(director, row[3], row[4], row[5])
    elseif what == "helpers" then
        local spots = row[4]
        if row[3] == 29001 then
            if spots == "either" then
                last_set = math.random(2)
                spots = last_set
            elseif spots == "other" then
                spots = 3 - (last_set or 1)
            end
            helpersCast(director, 29001, SPOTS_29001[spots])
        elseif spots == "middle" then
            helpersCast(director, row[3], { { MIDDLE_1.x, MIDDLE_1.z, 1.5708 } })
        elseif spots == "twelve" then
            helpersCast(director, row[3], SPOTS_28991)
        end
    elseif what == "map" then
        director:map_effect(row[3], row[4], row[5])
        director:set_weather(row[6], WEATHER_FADE, true)
    elseif what == "back" then
        if attempts_of == "first" then
            director:jump_bnpc(BOSS_1, BOSS_1_SPOT.x, FLOOR_1, BOSS_1_SPOT.z, BOSS_1_SPOT.rotation)
        else
            director:jump_bnpc(BOSS_2, MIDDLE_2.x, FLOOR_2, MIDDLE_2.z, -1.5708)
        end
    elseif what == "stop" then
        director:stop_cast(row[3], CAST_STOPPED)
    elseif what == "fn" then
        _G[row[3]](director, row[4])
    end
end

-- A table's rows on the clock from now, while the fight is in `during` (one of the parts) or `also`.
function schedule(director, rows_of, during, also)
    D:schedule(director, rows_of, function() return part == during or part == also end, step)
end

-- Once the rounds of 28982 have begun: the chain as soon as its health is below the line, whatever the round is at.
function chainOrNot(director)
    if part == "first" and rounds_begun and boss_health ~= nil and boss_health < CHAIN_AT then
        part = "chain"
        director:restart_timeline(BOSS_1, PART_CHAIN)
        schedule(director, CHAIN, "chain")
    end
end

-- Everything of a fight gone.
function clear(director)
    director:despawn_bnpc(FOUR)
    director:despawn_bnpc(ROW)
    director:despawn_bnpc(CORNER)
    for _, helper in ipairs(helpers) do
        director:despawn_bnpc(helper)
    end
    helpers = {}
    D.helpers = {}
    for _, layout in ipairs({ THREE_A, THREE_B, THREE_C }) do
        director:despawn_bnpc(layout)
    end
    three = {}
end

-- ---- The two bosses ----

-- At each pull the helpers draw their weapons and go into battle with the boss (ActorControl 0 `[1, 1]` then 4 `[1]`,
-- at the pull: all 15 in the first fight, the 10 of the second), and the three of the first 0.04 s later (Bob's
-- 9ba7d981).
CONTROL_WEAPON = 0
CONTROL_BATTLE = 4
THREE_IN_BATTLE_AFTER = 0.04
function intoBattle(director, ids)
    for _, id in ipairs(ids) do
        director:actor_control(id, CONTROL_WEAPON, { 1, 1 })
        director:actor_control(id, CONTROL_BATTLE, { 1 })
    end
end

D:boss({ id = 1, layout = BOSS_1, middle = MIDDLE_1, comes_later = true, eobjs_at_setup = false, bgm = false,
    ready = function() return part == "first-wait" end,
    on_pull = function(director)
        part = "first"
        D.helpers = helpers
        intoBattle(director, helpers)
        D:after(THREE_IN_BATTLE_AFTER, function(d)
            local ids = {}
            for _, id in pairs(three) do
                ids[#ids + 1] = id
            end
            intoBattle(d, ids)
        end)
        director:party_npc_yell("ranged", LINE_PULL)
        schedule(director, FIRST, "first")
        D:after(ROUND_FIRST, function(d)
            if part == "first" then
                rounds_begun = true
            end
        end)
    end,
    on_cast = {
        [ACTION_ROUND] = function(director)
            if part ~= "first" then
                return
            end
            rounds = rounds + 1
            local nth = (rounds - 1) % 3 + 1
            if nth == 3 then
                helpersCast(director, ACTION_ROUND_HELPER, SPOTS_28983_EIGHT)
            else
                local pick = math.random(#SPOTS_28983_THREE)
                if nth == 2 and pick == last_three then
                    pick = pick % #SPOTS_28983_THREE + 1
                end
                last_three = pick
                helpersCast(director, ACTION_ROUND_HELPER, SPOTS_28983_THREE[pick])
            end
            if nth == 1 then
                -- Base 14591 shows itself, casts and hides; once shown it hides again even if the chain has begun
                -- since (Bob's run of 2026-11-18, 6320ae56: the chain's talk 20.6 s into a round, 14591 hidden 0.4 s
                -- after it). A round the chain overtakes before it shows has none of the three (50e72631.)
                D:after(ROUND[1][1], function(d)
                    if part == "first" then
                        step(d, ROUND[1])
                        D:after(ROUND[3][1] - ROUND[1][1], function(d2)
                            if part == "first" or part == "chain" then
                                step(d2, ROUND[3])
                            end
                        end)
                    end
                end)
                schedule(director, { ROUND[2] }, "first")
            end
        end,
    } })

D:boss({ id = 2, layout = BOSS_2, middle = MIDDLE_2, comes_later = true, eobjs_at_setup = false, bgm = false,
    last = true,
    ready = function() return part == "second-wait" end,
    on_pull = function(director)
        part = "second"
        D.helpers = helpers
        intoBattle(director, helpers)
        schedule(director, SECOND, "second")
    end,
    -- Its health at 0: its cast stopped, the closing cutscene, the duty complete.
    on_death = function(director)
        part = "over"
        if boss_2 ~= nil then
            director:bnpc_control(BOSS_2, CONTROL_STOP_CAST, { CAST_STOPPED_DEAD, 1, ACTION_LAST, 1 })
        end
        stopHelpers(director)
        letHelpersGo(director)
        -- The ally's 3014 goes as the closing cutscene comes (both runs.)
        for _, member in ipairs(D.party) do
            director:lose_effect(member.id, STATUS_ALLY_SHIELD)
        end
        clear(director)
        boss_2 = nil
        director:set_weather(WEATHER_OWN, WEATHER_FADE, true)
        director:play_cutscene(CUTSCENE_END, CUTSCENE_END_PARAMS)
        director:complete_duty()
        -- (ActionTimeline 73 on it a second after it fell, in both runs.)
        director:delay(1.0)
        director:play_action_timeline(BOSS_2, 73)
        director:delay(0)
    end })

-- Below the line in the chain: the first fight is over, the cutscene between them.
function cutscene(director)
    part = "cutscene"
    D:fell(director, D:bossById(1))
    director:restart_timeline(BOSS_1, PART_CUTSCENE)
    stopHelpers(director)
    letHelpersGo(director)
    clear(director)
    director:despawn_bnpc(BOSS_1)
    boss_1 = nil
    director:set_weather(WEATHER_OWN, WEATHER_FADE, true)
    director:play_cutscene(CUTSCENE_MIDDLE, CUTSCENE_MIDDLE_PARAMS)
end

-- ---- What the server calls ----

function onSetup(director)
    director:set_sequence(1)
    reset()
    bringFirst(director)
end

function onSceneFinished(director, scene)
    if part ~= "cutscene" then
        return
    end
    reset()
    part = "between"
    director:warp_players(SECOND_ARENA.x, SECOND_ARENA.y, SECOND_ARENA.z, SECOND_ARENA.rotation)
    -- A wipe from here on sends everyone back to this arena, not the duty's entrance (retail's wasn't seen.)
    director:set_start(SECOND_ARENA.x, SECOND_ARENA.y, SECOND_ARENA.z, SECOND_ARENA.rotation)
    -- The second boss and its helpers came 3.9 and 1.9 s after the cutscene's end in the runs, as the player was in.
    D:after(SECOND_COMES_AFTER, bringSecond)
end

function onActorCast(director, layout_id, action_id, actor_id, x, y, z)
    D:cast(director, layout_id, action_id, actor_id, x, y, z)
end

-- After the pulls: where the boss is and whom it's after, the chain, the cutscene, the four of the second fight.
local function between(director)
    local boss = (attempts_of == "first") and BOSS_1 or BOSS_2
    local him = D.info[boss]
    boss_health = D.health[boss]
    if him == nil then
        return
    end
    boss_at = { x = him.x, z = him.z }
    boss_target = him.target_id or him.target
    chainOrNot(director)
    if part == "chain" and cutscene_may_come and boss_health ~= nil and boss_health < CUTSCENE_AT then
        cutscene(director)
    end
    if part == "second" or part == "last" then
        watchCorners(director)
    end
end

function onUpdate(director, health, player_health, info)
    D:update(director, health, player_health, info, between)
end

function onActorDeath(director, layout_id, position)
    if layout_id == BOSS_2 and part ~= "over" then
        D:died(director, layout_id)
    end
end

-- Everyone down: the fight they fell in begins again (retail's wasn't seen.)
function onPartyWipe(director)
    if part == "over" or part == "cutscene" or part == "between" then
        return
    end
    D:wipe(director)
    stopHelpers(director)
    -- Nobody stays gaoled into the next attempt.
    for _, held in ipairs(gaoled) do
        director:lose_effect(held.id, STATUS_GAOL)
    end
    gaoled = {}
    clear(director)
    director:set_weather(WEATHER_OWN, WEATHER_FADE, true)
    reset()
    if attempts_of == "first" then
        director:despawn_bnpc(BOSS_1)
        bringFirst(director)
    else
        director:despawn_bnpc(BOSS_2)
        bringSecond(director)
    end
end

-- No shortcuts in a trial.
function getDebugShortcut(id)
    return 0
end
