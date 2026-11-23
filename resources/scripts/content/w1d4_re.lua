-- Castrum Meridianum (ContentFinderCondition 15, territory 1043, director 0x8003000F), during Rock the Castrum
-- (quest 3873).
--
-- Three retail runs with Duty Support (the fourth set, level 50): Connor's Chronofoil capture af05916c (2026-11-09,
-- visit 0.8: a healer, three attempts at the last boss), Bob's 9ba7d981 (2026-11-17, visit 0.12, a damage dealer)
-- and his 50e72631 (the same day, visit 0.3, a healer); `questkit chronofoil --battle` and the visits' packets
-- (docs/src/fork/dungeons.md). Numbers only. **No layout dump**: the duty's event ranges aren't known, so what the
-- party sets off by walking somewhere is set off here by where they were when it happened in the runs (the WHERE_*
-- lines below: Bob's two runs agree to a yalm or two; Connor's, a healer behind the tank, was set off by the tank).
--
-- The way through:
--   * a line as the party sets off (talks 33506, 33507), map effects 2 and 3 a little further;
--   * the six of layouts 9088382-9088387 dead: map effect 0. The five of 9088400-9088404 dead: map effect 1, and 2
--     and 3 again;
--   * the first boss and its four come out as the five fall (the way into its room opens); its room (objective 0 as the
--     party comes in); it fallen, the device by its room (EObj 2000597) can
--     be used (EventAction 23, 1 s) and puts the party at (-11, 56.4, 205.9), far to the south;
--   * map effects 8 and 9; the three of 9088917-9088919 dead: map effect 5; the two of 9088929 and 9088930 dead:
--     map effect 6, and 8 and 9 again;
--   * the second boss's room (objective 2); it fallen, map effects 7 and 10;
--   * back past the first boss's room three come out of hiding (9088949-9088951: Appear with ActionTimeline 142);
--     they dead: talk 33512, map effect 11 and EObj 2007457's ActorControl 409 3 s later;
--   * the six of 9088952-9088955, 9162432 and 9162434: with three of them dead two more come out (9088956, 9088957,
--     talk 33522); all eight dead: 2.3 s later the cutscene (scene 3 with cutscene 275 `[0, 151, 0, 0]`), map effects 12-14;
--   * as it ends everyone is put in the last room (-83.9, 72.0, -35.2), objective 4, and the last boss is there.
--
-- The three bosses are **one clock from their pull** in every run (to 0.3 s; `CastrumMeridianumBoss1_14538.json`,
-- `Boss2_14541`, `Boss3_14542`), and the director does what goes with their casts:
--   * the first (9060841): each of its 28774s brings a wave (a second after it lands), its 29268 three helpers'
--     29269 on everyone (5.1, 5.6 and 6.2 s after it begins);
--   * the second (9060849): a helper's 28779 from one of three spots 4.9 s into each 28778; helpers' 28781 and 28782
--     from the middle with each 28780; a helper's 28784 under each of the party 6.4 s into each 28783;
--   * the last (9060634): 28788 takes it to the middle; with 28786 eight helpers' 28787 round the room and 29163 in
--     the middle; after 28791 four volleys 2.5 s apart, each a helper's 28793 at the room's edge and six 28794s in
--     three pairs marching across it (the first two volleys in either order); two 28797s in the middle with 28796
--     and 28795; with 28789 six 28790s (one of three sets) and 29164 in the middle; 9.6 s into each 29357 two
--     29356s (an opposite pair) or four, in turn. Below 20 % (20.0, 15.6, 20.0 and 11.7 % as the talk came: at once, or
--     3.1 and 3.3 s after the cast it was in) its last part: talk 33520, then the timeline's points after it.
--
-- Not known, so guessed or left out: the event ranges (above); the second and last bosses' clocks past what the
-- runs show (the rounds go on: the first's to 243 s, the last's 74.2 s round twice); what the last part does past 40 s
-- (its 29352 and three 29353s every 17.3 s, seen twice);
-- where 28788 takes the last boss when it isn't the middle (the middle here); a wipe (the party fell twice at the
-- last boss in Connor's run and was back at it within 107 s: the fight begins again, everyone back in its room;
-- at the first two the duty's entrance, as everywhere); the director's 0x80000007 [42] (sent as retail did); what
-- EObj 2007457's ActorControl 409 does; coffers (none in a Duty Support run).

-- Written with the dungeon framework (content/lib/Dungeon.lua, loaded first): its packs, steps and boss rooms are
-- declared below, and what only this duty does is in their hooks.

-- Retail's facing at the entrance (the first position, all three runs.)
ENTRANCE_ROTATION = -1.475

BOSS1 = 9060841
BOSS1_WALL = 2000587
BOSS1_LINE = 2000588
PLACE_BOSS1 = 434
BOSS2 = 9060849
BOSS2_WALL = 2000589
BOSS2_LINE = 2000590
PLACE_BOSS2 = 433
BOSS3 = 9060634
BOSS3_WALL = 2000593
BOSS3_LINE = 2002735
PLACE_BOSS3 = 428

HELPER_BOSS1 = 0
HELPER_BOSS2 = 9124008
HELPER_BOSS3 = 9060713
HELPERS_BOSS1 = 3
HELPERS_BOSS2 = 7
-- (Retail's numbers: three, seven and fifty; the last boss's volleys keep 28 at work at once.)
HELPERS_BOSS3 = 50

BGM_BATTLE = 37
BGM_DUTY = 148
COMMAND_LINE = 0x80000007
COMMAND_LINE_ARG = 42
TALK_MS = 6000
NAME_GUIDE = 1486
GUIDE = 9089788

-- The first boss's waves: after its first, second and third 28774 (a second after each lands, 4.7 s after it
-- begins.)
ACTION_WAVE = 28774
WAVE_AFTER = 4.7
WAVES = {
    { 9169258, 9169259, 9169260, 9169261, 9169262, 9169263 },
    { 9169264, 9169265 },
    { 9060843, 9060844, 9060847, 9060848, 9169268, 9169269 },
}
ACTION_BOSS1_ALL = 29268
ACTION_BOSS1_HELPER = 29269
BOSS1_HELPER_AFTER = { 5.1, 5.6, 6.2 }
BOSS1_MIDDLE = { x = 11.8, y = 70.2, z = -38.6 }
-- The bosses' floors (their rooms' ranges aren't known here): Boss Mod's arenas (github.com/awgil/ffxiv_bossmod,
-- BSD-3-Clause), which the party bots stand and dodge on and never leave while the boss is fought. The second's is
-- square to the axes: Boss Mod's has a third number (20), but the walls Kawari's bots and boss were held by ran along
-- x = 6 to 7, its east side so (Bob's run of 2026-11-22.)
BOSS1_FLOOR = { x = 10.0, z = -40.0, half_x = 20.0 }
BOSS2_FLOOR = { x = -13.0, z = 31.0, half_x = 20.0, half_z = 20.0 }
BOSS3_FLOOR = { x = -98.0, z = -33.0, half_x = 20.0, round = true }
BOSS1_TALKS = { { 14.6, 33524 }, { 40.8, 33525 }, { 78.3, 33526 } }
NAME_BOSS1 = 557

BOSS2_MIDDLE = { x = -13.0, y = 70.1, z = 31.0 }
SPOTS_28779 = { { -5.1, 19.3 }, { -20.9, 42.7 }, { -24.7, 23.1 } }

BOSS3_MIDDLE = { x = -98.0, y = 72.0, z = -33.0 }
NAME_BOSS3 = 2118
BOSS3_TALK_MS = 5000
BOSS3_TALKS = { { 12.2, 33516 }, { 45.8, 33517 }, { 67.9, 33518 }, { 72.9, 33519 } }
LAST_PART_AT = 20
LAST_PART_AFTER_CAST = 3.3
-- Its casts' lengths.
BOSS3_CASTS = { [28033] = 4.7, [28786] = 4.1, [28798] = 4.7, [28791] = 4.6, [29357] = 3.7, [28789] = 3.0,
                [28795] = 1.2 }
PART_LAST = 33520
-- Its 28788s (seconds from the pull): twice, then three a round of 74.2 s (ten minutes of rounds here.)
AWAY_FOR = 0.7
AWAY_ONCE = { 17.3, 51.0 }
AWAY_ROUND = { 91.3, 122.8, 139.2 }
AWAY_ROUND_EVERY = 74.2
AWAY_ROUNDS = 8
-- (x, z, facing.)
SPOTS_28787 = { { -113.7, -29.9, 1.763 }, { -111.3, -42.0, 0.977 }, { -107.0, -19.8, 2.548 },
                { -101.1, -48.7, 0.192 }, { -95.0, -17.3, -2.949 }, { -89.1, -46.3, -0.593 },
                { -84.8, -24.1, -2.164 }, { -82.3, -36.1, -1.379 } }
-- The volleys after 28791: the 28793's spot, then the 28794s' three pairs, 0.2 s apart.
VOLLEYS = {
    { { -78.4, -36.8, -1.379 },
      { { -81.9, -44.3, -2.950 }, { -78.8, -28.6, -2.950 } },
      { { -99.5, -40.9, -2.950 }, { -96.5, -25.2, -2.950 } },
      { { -117.2, -37.4, -2.950 }, { -114.2, -21.7, -2.950 } } },
    { { -94.2, -13.4, -2.950 },
      { { -102.4, -13.8, 1.763 }, { -86.7, -16.9, 1.763 } },
      { { -105.9, -31.5, 1.763 }, { -90.2, -34.5, 1.763 } },
      { { -109.3, -49.1, 1.763 }, { -93.6, -52.2, 1.763 } } },
    { { -109.2, -16.4, 2.548 },
      { { -114.7, -22.6, 0.977 }, { -101.5, -13.6, 0.977 } },
      { { -104.7, -37.5, 0.977 }, { -91.4, -28.6, 0.977 } },
      { { -94.6, -52.4, 0.977 }, { -81.3, -43.5, 0.977 } } },
    { { -81.4, -21.8, -2.164 },
      { { -87.6, -16.3, 2.548 }, { -78.6, -29.6, 2.548 } },
      { { -102.5, -26.4, 2.548 }, { -93.5, -39.7, 2.548 } },
      { { -117.4, -36.5, 2.548 }, { -108.5, -49.7, 2.548 } } },
}
VOLLEY_FIRST = 1.5
VOLLEY_EVERY = 2.5
SHOWS_FIRST = 9.6
TIMELINE_SHOWS = 7747
SPOTS_28790 = {
    { { -111.3, -42.0, 0.977 }, { -107.8, -31.1, 1.763 }, { -107.0, -19.8, 2.548 }, { -89.1, -46.3, -0.593 },
      { -88.2, -34.9, -1.379 }, { -84.8, -24.1, -2.164 } },
    { { -111.3, -42.0, 0.977 }, { -107.0, -19.8, 2.548 }, { -99.9, -42.8, 0.192 }, { -96.1, -23.2, -2.949 },
      { -89.1, -46.3, -0.593 }, { -84.8, -24.1, -2.164 } },
    { { -113.7, -29.9, 1.763 }, { -106.3, -38.6, 0.977 }, { -101.1, -48.7, 0.192 }, { -95.0, -17.3, -2.949 },
      { -89.7, -27.4, -2.164 }, { -82.3, -36.1, -1.379 } },
}
SPOTS_29356_TWO = {
    { { -100.7, -46.8, 1.763 }, { -95.3, -19.3, 1.763 } },
    { { -111.8, -30.4, 1.763 }, { -84.3, -35.7, 1.763 } },
    { { -105.8, -21.4, 2.548 }, { -90.2, -44.6, 2.549 } },
}
SPOTS_29356_FOUR = { { -109.6, -40.9, 0.977 }, { -105.8, -21.4, 2.548 }, { -90.2, -44.6, 2.549 },
                     { -86.4, -25.2, 0.977 } }
AFTER_29357 = 9.6

-- The packs whose fall opens the way: every layout dead, then the map effects { index, timeline, state }.
PACKS = {
    { layouts = { 9088382, 9088383, 9088384, 9088385, 9088386, 9088387 }, effects = { { 0, 2, 1 } } },
    -- With them dead the way into the first boss's room opens, and it and its four come out (0.3 to 0.4 s after the
    -- map effects in every run where the party was near: Bob's mark of 2026-11-20, "the black eft, and his 4 Cohort
    -- allies seem to not be spawned after the wall barrier blocking entry to them is removed instead their always
    -- there"; `nonpop` in the drop-in.)
    { layouts = { 9088400, 9088401, 9088402, 9088403, 9088404 },
      effects = { { 1, 2, 1 }, { 2, 4, 4 }, { 3, 4, 4 } },
      spawns = { BOSS1, 9169252, 9169255, 9169256, 9169257 } },
    { layouts = { 9088917, 9088918, 9088919 }, effects = { { 5, 2, 1 } } },
    { layouts = { 9088929, 9088930 }, effects = { { 6, 2, 1 }, { 8, 4, 4 }, { 9, 4, 4 } } },
}
-- Where the party was as retail set these off (no layout dump: see above.)
ENTRANCE = { x = 247.7, z = -95.0 }
WHERE_START_FROM = 8.0 -- yalms from the entrance: talks 33506 and, 7.0 s on, 33507
WHERE_EFFECTS_2_3_X = 208.0 -- west of this: map effects 2 and 3 (timeline 2, state 2)
ROOM_BOSS1 = { BOSS1, 9169252, 9169255, 9169256, 9169257 }
ROOM_BOSS1_RADIUS = 24.0 -- its line's distance from the middle
WHERE_BOSS1 = 26.0 -- yalms from the first boss's middle (its line is 24 out): objective 0
WHERE_TALK_33510_Z = 230.0 -- south of this after the device: talk 33510
WHERE_EFFECTS_8_9_X = -36.0 -- west of this then: map effects 8 and 9
WHERE_BOSS2 = 21.0 -- yalms from the second boss's middle (its line is 20 out): objective 2
WHERE_AMBUSH_Z = -18.0 -- north of this, back by the first boss's room: the three in hiding
WHERE_TALK_33513_Z = -77.0 -- past the way talk 33512 opens
WHERE_TALK_33514_X = -42.0

-- The device by the first boss's room and where it puts the party.
EOBJ_DEVICE = 2000597
EVENT_ACTION_DEVICE = 23
DEVICE_TO = { x = -10.96, y = 56.35, z = 205.87, rotation = 0.011 }
EOBJ_WAY = 2007457
CONTROL_WAY = 409
DIRECTOR_ID = 0x8003000F
EOBJ_BEFORE_LAST = 2012983
EOBJ_EXIT = 2000596
CONTROL_TARGETABLE = 54

AMBUSH = { 9088949, 9088950, 9088951 }
SIX = { 9088952, 9088953, 9088954, 9088955, 9162432, 9162434 }
SIX_DEAD_FOR_TWO = 3
TWO = { 9088956, 9088957 }
CUTSCENE_LAST_AFTER = 2.3 -- seconds from the last of the eight's fall to the cutscene (2.1 to 2.4 in four runs)
APPEAR_TIMELINE = 142
APPEAR_AFTER = 0.7

-- Who comes out with the second boss's fall and with the last cutscene (`nonpop` in the drop-in.)
AFTER_BOSS2 = { GUIDE, 9091115 }
AT_THE_LAST_ROOM = { 9206933, 9206934 }
CUTSCENE_LAST = 275
CUTSCENE_LAST_PARAMS = { 0, 151, 0, 0 }
LAST_ROOM = { x = -83.88, y = 72.0, z = -35.23, rotation = -1.401 }
-- Where the shortcut put Bob after a fall at the last boss (retail's ActorSetPos, capture 6320ae56, twice). Where it
-- goes before the last room, and when it first shows, isn't known: it is shown and pointed from the last room on.
SHORTCUT_LAST = { x = -67.92, y = 69.48, z = -38.36, rotation = -1.396 }
CUTSCENE_END = 277
CUTSCENE_END_PARAMS = { 1, 18, 0, 0 }

DUNGEON = Dungeon.new()
D = DUNGEON

flares = 0
last_part = false
last_boss_due = false
-- When the last boss's cast in hand is over (in updates.)
boss3_free_at = 0

function talk(director, text)
    director:battle_talk(GUIDE, NAME_GUIDE, text, 0, TALK_MS)
end

-- ---- The packs whose fall opens the way ----

for _, pack in ipairs(PACKS) do
    D:pack(pack)
end

-- ---- What the party's steps set off ----

local within = Dungeon.within
local function done(name)
    return function() return D.done[name] == true end
end

D:step({ name = "start",
    where = function(m) return not within(m, ENTRANCE, WHERE_START_FROM) end,
    run = function(d)
        d:command(COMMAND_LINE, { COMMAND_LINE_ARG })
        talk(d, 33506)
        D:after(7.0, function(d2) talk(d2, 33507) end)
    end })
D:step({ name = "effects_2_3",
    where = function(m) return m.x < WHERE_EFFECTS_2_3_X and m.z < 0 end,
    run = function(d)
        d:map_effect(2, 2, 2)
        d:map_effect(3, 2, 2)
    end })
D:step({ name = "boss1_room",
    where = function(m) return within(m, BOSS1_MIDDLE, WHERE_BOSS1) end,
    run = function(d) d:objective(0, 1) end })
D:step({ name = "talk_33510", ready = done("device"),
    where = function(m) return m.z > WHERE_TALK_33510_Z end,
    run = function(d)
        d:command(COMMAND_LINE, { COMMAND_LINE_ARG })
        talk(d, 33510)
    end })
D:step({ name = "effects_8_9", ready = done("talk_33510"),
    where = function(m) return m.x < WHERE_EFFECTS_8_9_X and m.z > 150 end,
    run = function(d)
        d:map_effect(8, 2, 2)
        d:map_effect(9, 2, 2)
    end })
D:step({ name = "boss2_room", ready = done("device"),
    where = function(m) return within(m, BOSS2_MIDDLE, WHERE_BOSS2) end,
    run = function(d) d:objective(2, 3) end })
D:step({ name = "ambush", ready = function() return D.dead[BOSS2] == true end,
    where = function(m) return m.z < WHERE_AMBUSH_Z and m.z > -60 and m.x > -10 and m.x < 40 end,
    run = function(d) D:comeOut(d, AMBUSH, APPEAR_TIMELINE, APPEAR_AFTER) end })
D:step({ name = "talk_33513", ready = done("talk_33512"),
    where = function(m) return m.z < WHERE_TALK_33513_Z and m.x < 30 end,
    run = function(d) talk(d, 33513) end })
D:step({ name = "talk_33514", ready = done("talk_33513"),
    where = function(m) return m.x < WHERE_TALK_33514_X and m.z < -40 end,
    run = function(d) talk(d, 33514) end })

-- ---- The first boss ----

D:boss({ id = 1, layout = BOSS1, wall = BOSS1_WALL, line = BOSS1_LINE, place = PLACE_BOSS1, middle = BOSS1_MIDDLE,
    arena = BOSS1_FLOOR,
    -- It and its four keep to their room: they notice only whoever comes in, are pulled together, and don't follow
    -- anyone out (retail's five, all five runs: pulled within 2.1 s of each other as the player came 7 to 11 yalms
    -- inside the line, none more than 10.7 yalms from the middle; Bob's mark of 2026-11-21: "enemies are not suppose
    -- to leave the wall barrier".)
    room = { members = ROOM_BOSS1, radius = ROOM_BOSS1_RADIUS },
    helpers = { layout = HELPER_BOSS1, count = HELPERS_BOSS1 },
    bgm = BGM_BATTLE, after_bgm = BGM_DUTY,
    talker = { name = NAME_BOSS1, pull = 33523, at = BOSS1_TALKS, ms = TALK_MS },
    -- After its first, second and third 28774 (a second after each lands, 4.7 s after it begins.)
    waves = { action = ACTION_WAVE, after = WAVE_AFTER, list = WAVES },
    on_cast = {
        [ACTION_BOSS1_ALL] = function(director)
            for _, at in ipairs(BOSS1_HELPER_AFTER) do
                D:after(at, function(d)
                    if D.fight == 1 then
                        D:helpersCast(d, ACTION_BOSS1_HELPER, { { BOSS1_MIDDLE.x, BOSS1_MIDDLE.z } }, BOSS1_MIDDLE.y)
                    end
                end)
            end
        end,
    },
    on_death = function(d)
        d:command(COMMAND_LINE, { COMMAND_LINE_ARG })
        d:objective(1, 2)
        talk(d, 33508)
        d:map_effect(4, 2, 1)
        d:show_eobj(EOBJ_DEVICE)
        d:eobj_control(EOBJ_DEVICE, CONTROL_TARGETABLE, { 1 })
        D:after(7.0, function(d2) talk(d2, 33509) end)
    end })

-- ---- The second boss ----

D:boss({ id = 2, layout = BOSS2, wall = BOSS2_WALL, line = BOSS2_LINE, place = PLACE_BOSS2, middle = BOSS2_MIDDLE,
    arena = BOSS2_FLOOR,
    helpers = { layout = HELPER_BOSS2, count = HELPERS_BOSS2 },
    bgm = BGM_BATTLE, after_bgm = BGM_DUTY,
    on_cast = {
        [28778] = function(director)
            D:after(4.9, function(d)
                if D.fight == 2 then
                    D:helpersCast(d, 28779, { SPOTS_28779[math.random(#SPOTS_28779)] }, BOSS2_MIDDLE.y)
                end
            end)
        end,
        [28780] = function(director)
            D:helpersCast(director, 28781, { { BOSS2_MIDDLE.x, BOSS2_MIDDLE.z, -2.95 } }, BOSS2_MIDDLE.y)
            D:helpersCast(director, 28782, { { BOSS2_MIDDLE.x, BOSS2_MIDDLE.z, -2.95 } }, BOSS2_MIDDLE.y)
        end,
        [28783] = function(director)
            D:after(6.4, function(d)
                if D.fight == 2 then
                    for _, member in ipairs(D.party) do
                        D:helpersCast(d, 28784, { { member.x, member.z } }, BOSS2_MIDDLE.y)
                    end
                end
            end)
        end,
    },
    on_death = function(d)
        d:command(COMMAND_LINE, { COMMAND_LINE_ARG })
        d:objective(3, 4)
        talk(d, 33511)
        d:map_effect(7, 2, 1)
        D:after(2.1, function(d2) d2:map_effect(10, 2, 1) end)
        D:after(7.0, function(d2) talk(d2, 33521) end)
        -- Cid (BNpcName 1486, 9089788) and the one at the hidden three's corner (9091115) come out with its fall:
        -- retail showed 9091115 1.5 to 1.7 s before the death came, both only after it in all five runs (Bob's mark of
        -- 2026-11-20 at the first boss's room: "why is cid here he is not suppose to be in this fight at this moment".)
        for _, layout in ipairs(AFTER_BOSS2) do
            d:spawn_bnpc(layout)
        end
    end })

-- ---- The last boss ----

-- The volleys after the last boss's 28791.
function volleys(director)
    local order = { 1, 2, 3, 4 }
    if math.random(2) == 1 then
        order = { 2, 1, 3, 4 }
    end
    for n, which in ipairs(order) do
        local volley = VOLLEYS[which]
        local at = VOLLEY_FIRST + (n - 1) * VOLLEY_EVERY
        D:after(at, function(d)
            if D.fight ~= 3 then
                return
            end
            D:helpersCast(d, 28793, { volley[1] }, BOSS3_MIDDLE.y)
            D:helpersCast(d, 28794, volley[2], BOSS3_MIDDLE.y)
            d:delay(0.2)
            D:helpersCast(d, 28794, volley[3], BOSS3_MIDDLE.y)
            d:delay(0.4)
            D:helpersCast(d, 28794, volley[4], BOSS3_MIDDLE.y)
            d:delay(0)
        end)
    end
    -- It shows itself (ActionTimeline 7747) four times, 9.6 s after 28791 began and every 2.5 s (1.7 s after each of
    -- its 28792s.)
    for n = 0, 3 do
        D:after(SHOWS_FIRST + n * VOLLEY_EVERY, function(d)
            if D.fight == 3 then
                d:play_action_timeline(BOSS3, TIMELINE_SHOWS)
            end
        end)
    end
    -- Two 28797s in the middle 16.0 s after 28791 began, as its 28796 comes.
    D:after(16.0, function(d)
        if D.fight == 3 then
            twoInTheMiddle(d)
        end
    end)
end

function twoInTheMiddle(director)
    D:helpersCast(director, 28797, { { BOSS3_MIDDLE.x, BOSS3_MIDDLE.z, 1.763 }, { BOSS3_MIDDLE.x, BOSS3_MIDDLE.z, 1.763 } },
        BOSS3_MIDDLE.y)
end

-- The last boss's 28788 (no cast, from its timeline): it comes out in the middle 0.7 s later.
function toTheMiddle(director)
    if D.fight == 3 then
        director:jump_bnpc(BOSS3, BOSS3_MIDDLE.x, BOSS3_MIDDLE.y, BOSS3_MIDDLE.z, 1.763)
    end
end

BOSS3_ON_CAST = {
    [28786] = function(director)
        D:helpersCast(director, 28787, SPOTS_28787, BOSS3_MIDDLE.y)
        D:helpersCast(director, 29163, { { BOSS3_MIDDLE.x, BOSS3_MIDDLE.z, 1.763 } }, BOSS3_MIDDLE.y)
    end,
    [28791] = function(director) volleys(director) end,
    [28795] = function(director)
        D:after(0.5, function(d)
            if D.fight == 3 then
                twoInTheMiddle(d)
            end
        end)
    end,
    [28789] = function(director)
        D:helpersCast(director, 28790, SPOTS_28790[math.random(#SPOTS_28790)], BOSS3_MIDDLE.y)
        D:helpersCast(director, 29164, { { BOSS3_MIDDLE.x, BOSS3_MIDDLE.z, 1.763 } }, BOSS3_MIDDLE.y)
    end,
    [29357] = function(director)
        flares = flares + 1
        local spots = SPOTS_29356_FOUR
        if flares % 2 == 1 then
            spots = SPOTS_29356_TWO[math.random(#SPOTS_29356_TWO)]
        end
        D:after(AFTER_29357, function(d)
            if D.fight == 3 then
                D:helpersCast(d, 29356, spots, BOSS3_MIDDLE.y)
            end
        end)
    end,
}

D:boss({ id = 3, layout = BOSS3, wall = BOSS3_WALL, line = BOSS3_LINE, place = PLACE_BOSS3, middle = BOSS3_MIDDLE,
    arena = BOSS3_FLOOR,
    comes_later = true, last = true, bgm = false,
    helpers = { layout = HELPER_BOSS3, count = HELPERS_BOSS3 },
    ready = done("last_room"),
    talker = { name = NAME_BOSS3, pull = 33515, at = BOSS3_TALKS, ms = BOSS3_TALK_MS,
               still = function() return not last_part end },
    on_pull = function(director)
        flares = 0
        last_part = false
        -- Its 28788s, as its timeline has them.
        local function away(at)
            D:after(at + AWAY_FOR, function(d)
                if not last_part then
                    toTheMiddle(d)
                end
            end)
        end
        for _, at in ipairs(AWAY_ONCE) do
            away(at)
        end
        for round = 0, AWAY_ROUNDS - 1 do
            for _, at in ipairs(AWAY_ROUND) do
                away(at + round * AWAY_ROUND_EVERY)
            end
        end
    end,
    on_cast = function(director, action_id, x, y, z)
        -- (The last part waits for a cast in hand, and 3.3 s more: 3.1 and 3.3 s after its 28798 the two times.)
        boss3_free_at = D.ticks + math.floor(((BOSS3_CASTS[action_id] or 0) + LAST_PART_AFTER_CAST) * 2 + 0.5)
        if BOSS3_ON_CAST[action_id] then
            BOSS3_ON_CAST[action_id](director)
        end
    end,
    on_death = function(d)
        d:show_eobj(EOBJ_EXIT)
        d:play_cutscene(CUTSCENE_END, CUTSCENE_END_PARAMS)
        d:complete_duty()
    end,
    on_wipe = function(d)
        d:despawn_bnpc(BOSS3)
        D:dropHelpers(d, D:bossById(3))
        last_boss_due = true
    end,
    -- Back in after a wipe: it and its helpers come back then (31 and 73 s after the party fell in Connor's run, as he
    -- came back into the room.)
    on_recommence = function(d)
        if last_boss_due then
            last_boss_due = false
            D:bringBoss(d, D:bossById(3))
        end
    end })

-- ---- What the server calls ----

function onSetup(director)
    director:set_sequence(1)
    D:setup(director)
    director:hide_eobj(EOBJ_DEVICE)
    director:hide_eobj(EOBJ_EXIT)
    D:bringStartingHelpers(director)
end

function onActorCast(director, layout_id, action_id, actor_id, x, y, z)
    D:cast(director, layout_id, action_id, actor_id, x, y, z)
end

-- Below 20 % the last boss's last part (once its cast in hand is over.)
local function lastPart(director)
    local health = D.health[BOSS3]
    if D.fight == 3 and not last_part and health ~= nil and health < LAST_PART_AT and D.ticks >= boss3_free_at then
        last_part = true
        director:restart_timeline(BOSS3, PART_LAST)
        director:battle_talk(BOSS3, NAME_BOSS3, 33520, 0, TALK_MS)
        D:after(0.1 + AWAY_FOR, toTheMiddle)
    end
end

function onUpdate(director, health, player_health, info)
    D:update(director, health, player_health, info, lastPart)
end

function onActorDeath(director, layout_id, position)
    D:died(director, layout_id)
    if D.done.ambush and not D.done.talk_33512 and D:allDead(AMBUSH) then
        D:once("talk_33512")
        talk(director, 33512)
        D:after(3.0, function(d)
            d:map_effect(11, 2, 1)
            d:eobj_control(EOBJ_WAY, CONTROL_WAY, { 1, DIRECTOR_ID })
        end)
    end
    if not D.done.two and D:countDead(SIX) >= SIX_DEAD_FOR_TWO then
        D:once("two")
        D:comeOut(director, TWO, APPEAR_TIMELINE, APPEAR_AFTER)
        D:after(1.0, function(d) talk(d, 33522) end)
    end
    -- All eight, not the two alone: in Bob's run of 2026-11-18 (6320ae56) one of the six outlived the two by 4.9 s
    -- and the cutscene waited for it. 2.1 to 2.4 s after the last of them fell in the four runs.
    if D.done.two and not D.done.cutscene and D:allDead(TWO) and D:allDead(SIX) then
        D:once("cutscene")
        D:after(CUTSCENE_LAST_AFTER, function(d)
            d:play_cutscene(CUTSCENE_LAST, CUTSCENE_LAST_PARAMS)
            d:hide_eobj(EOBJ_BEFORE_LAST)
            D:bringBoss(d, D:bossById(3))
            -- The two by the last room's door (9206933, 9206934) come out with it: never seen before the cutscene in
            -- the five runs (seen within 20 yalms of them, 1 s into it where the party came near.)
            for _, layout in ipairs(AT_THE_LAST_ROOM) do
                d:spawn_bnpc(layout)
            end
        end)
        D:after(CUTSCENE_LAST_AFTER + 1.0, function(d)
            for index = 12, 14 do
                d:map_effect(index, 2, 1)
            end
        end)
    end
end

-- The cutscene before the last room over: everyone in it, where a wipe sends them from now on.
function onSceneFinished(director, scene)
    if D.done.cutscene and not D.done.last_room then
        D:once("last_room")
        director:warp_players(LAST_ROOM.x, LAST_ROOM.y, LAST_ROOM.z, LAST_ROOM.rotation)
        -- A fall from here on sends the party to the entrance as anywhere (Bob's two falls of 2026-11-18, capture
        -- 6320ae56: there 4.5 s after he fell, whole again), and the shortcut by it (GimmickAccessor 74) put him
        -- outside the last room both times, the boss and its helpers new 0.4 s after he came.
        director:update_shortcut_spot(SHORTCUT_LAST.x, SHORTCUT_LAST.y, SHORTCUT_LAST.z, SHORTCUT_LAST.rotation)
        D:after(1.8, function(d) d:objective(4, 5) end)
    end
end

-- Everyone down: the fight they fell in is forgotten (the framework's), the last boss comes back on recommencing.
function onPartyWipe(director)
    D:wipe(director)
end

function onDutyRecommence(director)
    D:recommence(director)
end

-- The device by the first boss's room.
function onGimmickAccessor(director, actor_id, id, params)
    if id == EOBJ_DEVICE and D.dead[BOSS1] then
        director:event_action(EVENT_ACTION_DEVICE, actor_id, id)
        return
    end
    director:finish_gimmick(actor_id)
end

function isGimmickFunctional(id)
    if id == EOBJ_DEVICE then
        return D.dead[BOSS1] == true
    end
    return true
end

function onEventActionCast(director, actor_id, target)
    director:finish_gimmick(actor_id)
    if target == EOBJ_DEVICE and D.dead[BOSS1] then
        D:once("device")
        director:warp_players(DEVICE_TO.x, DEVICE_TO.y, DEVICE_TO.z, DEVICE_TO.rotation)
    end
end

function getDebugShortcut(id)
    return 0
end
