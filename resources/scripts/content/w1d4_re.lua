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
--   * the first boss's room (objective 0 as the party comes in); it fallen, the device by its room (EObj 2000597) can
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
-- runs show (the rounds go on); what the last part does past 25 s (its 29352 and three 29353s every 17.3 s here);
-- where 28788 takes the last boss when it isn't the middle (the middle here); a wipe (the party fell twice at the
-- last boss in Connor's run and was back at it within 107 s: the fight begins again, everyone back in its room;
-- at the first two the duty's entrance, as everywhere); the director's 0x80000007 [42] (sent as retail did); what
-- EObj 2007457's ActorControl 409 does; coffers (none in a Duty Support run).

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
    { layouts = { 9088400, 9088401, 9088402, 9088403, 9088404 },
      effects = { { 1, 2, 1 }, { 2, 4, 4 }, { 3, 4, 4 } } },
    { layouts = { 9088917, 9088918, 9088919 }, effects = { { 5, 2, 1 } } },
    { layouts = { 9088929, 9088930 }, effects = { { 6, 2, 1 }, { 8, 4, 4 }, { 9, 4, 4 } } },
}
-- Where the party was as retail set these off (no layout dump: see above.)
ENTRANCE = { x = 247.7, z = -95.0 }
WHERE_START_FROM = 8.0 -- yalms from the entrance: talks 33506 and, 7.0 s on, 33507
WHERE_EFFECTS_2_3_X = 208.0 -- west of this: map effects 2 and 3 (timeline 2, state 2)
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

CUTSCENE_LAST = 275
CUTSCENE_LAST_PARAMS = { 0, 151, 0, 0 }
LAST_ROOM = { x = -83.88, y = 72.0, z = -35.23, rotation = -1.401 }
-- Where the shortcut put Bob after a fall at the last boss (retail's ActorSetPos, capture 6320ae56, twice). Where it
-- goes before the last room, and when it first shows, isn't known: it is shown and pointed from the last room on.
SHORTCUT_LAST = { x = -67.92, y = 69.48, z = -38.36, rotation = -1.396 }
CUTSCENE_END = 277
CUTSCENE_END_PARAMS = { 1, 18, 0, 0 }

ticks = 0
updating = false
later = {}
dead = {}
done = {}
-- Each boss's helpers (retail brought them with their boss), and whose are at work.
helpers_of = { {}, {}, {} }
helpers = {}
next_helper = 0
party = {}
fight = nil -- 1, 2 or 3 while that boss is fought
waves = 0
flares = 0
boss_health = nil
last_part = false
last_boss_due = false
-- When the last boss's cast in hand is over (in updates.)
boss3_free_at = 0

-- Runs fn so many seconds from now (an update is half a second; a call made outside onUpdate waits for the next.)
function after(seconds, fn)
    local updates = math.max(1, math.floor(seconds * 2 + 0.5))
    if not updating then
        updates = updates + 1
    end
    table.insert(later, { at = ticks + updates, fn = fn })
end

function runLater(director)
    local due, waiting = {}, {}
    for _, entry in ipairs(later) do
        if entry.at <= ticks then
            table.insert(due, entry)
        else
            table.insert(waiting, entry)
        end
    end
    later = waiting
    table.sort(due, function(a, b) return a.at < b.at end)
    for _, entry in ipairs(due) do
        entry.fn(director)
    end
end

function once(name)
    if done[name] then
        return false
    end
    done[name] = true
    return true
end

function talk(director, text)
    director:battle_talk(GUIDE, NAME_GUIDE, text, 0, TALK_MS)
end

function onSetup(director)
    director:set_sequence(1)
    director:boss_room(BOSS1, BOSS1_WALL, BOSS1_LINE, PLACE_BOSS1, {})
    director:boss_room(BOSS2, BOSS2_WALL, BOSS2_LINE, PLACE_BOSS2, {})
    -- The walls stand open until a pull; the rooms' lines, the device and the way out wait.
    for _, wall in ipairs({ BOSS1_WALL, BOSS2_WALL, BOSS3_WALL }) do
        director:hide_eobj(wall)
    end
    for _, line in ipairs({ BOSS1_LINE, BOSS2_LINE, BOSS3_LINE }) do
        director:show_eobj(line)
    end
    director:hide_eobj(EOBJ_DEVICE)
    director:hide_eobj(EOBJ_EXIT)
    bringHelpers(director, 1, HELPER_BOSS1, HELPERS_BOSS1, BOSS1_MIDDLE)
    bringHelpers(director, 2, HELPER_BOSS2, HELPERS_BOSS2, BOSS2_MIDDLE)
end

function bringHelpers(director, which, layout, count, at)
    helpers_of[which] = {}
    for i = 1, count do
        -- (The simulator's spawns return nothing: the layout ID stands in there.)
        helpers_of[which][i] = director:spawn_bnpc(layout, at.x, at.y, at.z, 0.0) or layout
        director:hold_bnpc(helpers_of[which][i])
    end
end

function useHelpers(which)
    helpers = helpers_of[which]
    next_helper = 0
end

function dropHelpers(director, which)
    for _, helper in ipairs(helpers_of[which]) do
        director:despawn_bnpc(helper)
    end
    helpers_of[which] = {}
    helpers = {}
end

-- The next helper in turn, put at a spot.
function helperAt(director, x, y, z, facing)
    if #helpers == 0 then
        return nil
    end
    next_helper = next_helper % #helpers + 1
    local helper = helpers[next_helper]
    director:warp_bnpc(helper, x, y, z, facing or 0.0)
    return helper
end

function helpersCast(director, action, spots, y)
    for _, spot in ipairs(spots) do
        local helper = helperAt(director, spot[1], y, spot[2], spot[3])
        if helper ~= nil then
            director:actor_use_action(helper, action)
        end
    end
end

function anyone(test)
    for _, member in ipairs(party) do
        if test(member) then
            return true
        end
    end
    return false
end

function within(member, at, yalms)
    local dx, dz = member.x - at.x, member.z - at.z
    return dx * dx + dz * dz <= yalms * yalms
end

function allDead(layouts)
    for _, layout in ipairs(layouts) do
        if not dead[layout] then
            return false
        end
    end
    return true
end

function countDead(layouts)
    local count = 0
    for _, layout in ipairs(layouts) do
        if dead[layout] then
            count = count + 1
        end
    end
    return count
end

function comeOut(director, layouts)
    for _, layout in ipairs(layouts) do
        director:spawn_bnpc(layout)
        director:appear_bnpc(layout, APPEAR_TIMELINE, APPEAR_AFTER)
    end
end

-- What the party's steps set off.
function theWay(director)
    if not done.start and anyone(function(m) return not within(m, ENTRANCE, WHERE_START_FROM) end) then
        once("start")
        director:command(COMMAND_LINE, { COMMAND_LINE_ARG })
        talk(director, 33506)
        after(7.0, function(d) talk(d, 33507) end)
    end
    if not done.effects_2_3 and anyone(function(m) return m.x < WHERE_EFFECTS_2_3_X and m.z < 0 end) then
        once("effects_2_3")
        director:map_effect(2, 2, 2)
        director:map_effect(3, 2, 2)
    end
    if not done.boss1_room and anyone(function(m) return within(m, BOSS1_MIDDLE, WHERE_BOSS1) end) then
        once("boss1_room")
        director:objective(0, 1)
    end
    if done.device and not done.talk_33510 and anyone(function(m) return m.z > WHERE_TALK_33510_Z end) then
        once("talk_33510")
        director:command(COMMAND_LINE, { COMMAND_LINE_ARG })
        talk(director, 33510)
    end
    if done.talk_33510 and not done.effects_8_9
        and anyone(function(m) return m.x < WHERE_EFFECTS_8_9_X and m.z > 150 end) then
        once("effects_8_9")
        director:map_effect(8, 2, 2)
        director:map_effect(9, 2, 2)
    end
    if done.device and not done.boss2_room and anyone(function(m) return within(m, BOSS2_MIDDLE, WHERE_BOSS2) end) then
        once("boss2_room")
        director:objective(2, 3)
    end
    if dead[BOSS2] and not done.ambush
        and anyone(function(m) return m.z < WHERE_AMBUSH_Z and m.z > -60 and m.x > -10 and m.x < 40 end) then
        once("ambush")
        comeOut(director, AMBUSH)
    end
    if done.talk_33512 and not done.talk_33513
        and anyone(function(m) return m.z < WHERE_TALK_33513_Z and m.x < 30 end) then
        once("talk_33513")
        talk(director, 33513)
    end
    if done.talk_33513 and not done.talk_33514 and anyone(function(m) return m.x < WHERE_TALK_33514_X and m.z < -40 end) then
        once("talk_33514")
        talk(director, 33514)
    end
end

function pull1(director)
    fight = 1
    waves = 0
    director:set_bgm(BGM_BATTLE)
    director:battle_talk(BOSS1, NAME_BOSS1, 33523, 0, TALK_MS)
    for _, row in ipairs(BOSS1_TALKS) do
        after(row[1], function(d)
            if fight == 1 then
                d:battle_talk(BOSS1, NAME_BOSS1, row[2], 0, TALK_MS)
            end
        end)
    end
    useHelpers(1)
end

function pull2(director)
    fight = 2
    director:set_bgm(BGM_BATTLE)
    useHelpers(2)
end

function pull3(director)
    fight = 3
    flares = 0
    last_part = false
    director:battle_talk(BOSS3, NAME_BOSS3, 33515, 0, BOSS3_TALK_MS)
    for _, row in ipairs(BOSS3_TALKS) do
        after(row[1], function(d)
            if fight == 3 and not last_part then
                d:battle_talk(BOSS3, NAME_BOSS3, row[2], 0, BOSS3_TALK_MS)
            end
        end)
    end
    -- Its 28788s, as its timeline has them.
    local function away(at)
        after(at + AWAY_FOR, function(d)
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
    useHelpers(3)
end

function bringLastBoss(director)
    director:spawn_boss(BOSS3, BOSS3_WALL, BOSS3_LINE, PLACE_BOSS3)
    bringHelpers(director, 3, HELPER_BOSS3, HELPERS_BOSS3, BOSS3_MIDDLE)
end

-- The volleys after the last boss's 28791.
function volleys(director)
    local order = { 1, 2, 3, 4 }
    if math.random(2) == 1 then
        order = { 2, 1, 3, 4 }
    end
    for n, which in ipairs(order) do
        local volley = VOLLEYS[which]
        local at = VOLLEY_FIRST + (n - 1) * VOLLEY_EVERY
        after(at, function(d)
            if fight ~= 3 then
                return
            end
            helpersCast(d, 28793, { volley[1] }, BOSS3_MIDDLE.y)
            helpersCast(d, 28794, volley[2], BOSS3_MIDDLE.y)
            d:delay(0.2)
            helpersCast(d, 28794, volley[3], BOSS3_MIDDLE.y)
            d:delay(0.4)
            helpersCast(d, 28794, volley[4], BOSS3_MIDDLE.y)
            d:delay(0)
        end)
    end
    -- It shows itself (ActionTimeline 7747) four times, 9.6 s after 28791 began and every 2.5 s (1.7 s after each of
    -- its 28792s.)
    for n = 0, 3 do
        after(SHOWS_FIRST + n * VOLLEY_EVERY, function(d)
            if fight == 3 then
                d:play_action_timeline(BOSS3, TIMELINE_SHOWS)
            end
        end)
    end
    -- Two 28797s in the middle 16.0 s after 28791 began, as its 28796 comes.
    after(16.0, function(d)
        if fight == 3 then
            twoInTheMiddle(d)
        end
    end)
end

function twoInTheMiddle(director)
    helpersCast(director, 28797, { { BOSS3_MIDDLE.x, BOSS3_MIDDLE.z, 1.763 }, { BOSS3_MIDDLE.x, BOSS3_MIDDLE.z, 1.763 } },
        BOSS3_MIDDLE.y)
end

function onActorCast(director, layout_id, action_id, actor_id, x, y, z)
    if layout_id == BOSS1 and fight == 1 then
        if action_id == ACTION_WAVE then
            waves = waves + 1
            local wave = WAVES[waves]
            if wave ~= nil then
                after(WAVE_AFTER, function(d)
                    if fight == 1 then
                        for _, layout in ipairs(wave) do
                            d:spawn_bnpc(layout)
                        end
                    end
                end)
            end
        elseif action_id == ACTION_BOSS1_ALL then
            for _, at in ipairs(BOSS1_HELPER_AFTER) do
                after(at, function(d)
                    if fight == 1 then
                        helpersCast(d, ACTION_BOSS1_HELPER, { { BOSS1_MIDDLE.x, BOSS1_MIDDLE.z } }, BOSS1_MIDDLE.y)
                    end
                end)
            end
        end
    elseif layout_id == BOSS2 and fight == 2 then
        if action_id == 28778 then
            after(4.9, function(d)
                if fight == 2 then
                    helpersCast(d, 28779, { SPOTS_28779[math.random(#SPOTS_28779)] }, BOSS2_MIDDLE.y)
                end
            end)
        elseif action_id == 28780 then
            helpersCast(director, 28781, { { BOSS2_MIDDLE.x, BOSS2_MIDDLE.z, -2.95 } }, BOSS2_MIDDLE.y)
            helpersCast(director, 28782, { { BOSS2_MIDDLE.x, BOSS2_MIDDLE.z, -2.95 } }, BOSS2_MIDDLE.y)
        elseif action_id == 28783 then
            after(6.4, function(d)
                if fight == 2 then
                    for _, member in ipairs(party) do
                        helpersCast(d, 28784, { { member.x, member.z } }, BOSS2_MIDDLE.y)
                    end
                end
            end)
        end
    elseif layout_id == BOSS3 and fight == 3 then
        -- (The last part waits for a cast in hand, and 3.3 s more: 3.1 and 3.3 s after its 28798 the two times.)
        boss3_free_at = ticks + math.floor(((BOSS3_CASTS[action_id] or 0) + LAST_PART_AFTER_CAST) * 2 + 0.5)
        if action_id == 28786 then
            helpersCast(director, 28787, SPOTS_28787, BOSS3_MIDDLE.y)
            helpersCast(director, 29163, { { BOSS3_MIDDLE.x, BOSS3_MIDDLE.z, 1.763 } }, BOSS3_MIDDLE.y)
        elseif action_id == 28791 then
            volleys(director)
        elseif action_id == 28795 then
            after(0.5, function(d)
                if fight == 3 then
                    twoInTheMiddle(d)
                end
            end)
        elseif action_id == 28789 then
            helpersCast(director, 28790, SPOTS_28790[math.random(#SPOTS_28790)], BOSS3_MIDDLE.y)
            helpersCast(director, 29164, { { BOSS3_MIDDLE.x, BOSS3_MIDDLE.z, 1.763 } }, BOSS3_MIDDLE.y)
        elseif action_id == 29357 then
            flares = flares + 1
            local spots = SPOTS_29356_FOUR
            if flares % 2 == 1 then
                spots = SPOTS_29356_TWO[math.random(#SPOTS_29356_TWO)]
            end
            after(AFTER_29357, function(d)
                if fight == 3 then
                    helpersCast(d, 29356, spots, BOSS3_MIDDLE.y)
                end
            end)
        end
    end
end

-- The last boss's 28788 (no cast, from its timeline): it comes out in the middle 0.7 s later.
function toTheMiddle(director)
    if fight == 3 then
        director:jump_bnpc(BOSS3, BOSS3_MIDDLE.x, BOSS3_MIDDLE.y, BOSS3_MIDDLE.z, 1.763)
    end
end

function onUpdate(director, health, player_health, info)
    ticks = ticks + 1
    updating = true
    if info ~= nil then
        party = {}
        for key, actor in pairs(info) do
            -- (Someone with no place yet, at the origin, isn't anywhere.)
            if (key == "player" or (type(key) == "string" and key:sub(1, 5) == "party"))
                and not (actor.x == 0 and actor.z == 0) then
                table.insert(party, { x = actor.x, z = actor.z, id = actor.id })
            end
        end
        theWay(director)
        local function pulled(layout)
            local it = info[layout]
            return it ~= nil and (it.target ~= nil or (health[layout] ~= nil and health[layout] < 100))
        end
        if fight == nil then
            if not dead[BOSS1] and pulled(BOSS1) then
                pull1(director)
            elseif not dead[BOSS2] and pulled(BOSS2) then
                pull2(director)
            elseif done.last_room and not dead[BOSS3] and pulled(BOSS3) then
                pull3(director)
            end
        end
        boss_health = health[BOSS3]
        if fight == 3 and not last_part and boss_health ~= nil and boss_health < LAST_PART_AT
            and ticks >= boss3_free_at then
            last_part = true
            director:restart_timeline(BOSS3, PART_LAST)
            director:battle_talk(BOSS3, NAME_BOSS3, 33520, 0, TALK_MS)
            after(0.1 + AWAY_FOR, toTheMiddle)
        end
    end
    runLater(director)
    updating = false
end

function onActorDeath(director, layout_id, position)
    dead[layout_id] = true
    for index, pack in ipairs(PACKS) do
        if not done["pack" .. index] and allDead(pack.layouts) then
            once("pack" .. index)
            for _, effect in ipairs(pack.effects) do
                director:map_effect(effect[1], effect[2], effect[3])
            end
        end
    end
    if layout_id == BOSS1 then
        fight = nil
        later = {}
        dropHelpers(director, 1)
        for _, wave in ipairs(WAVES) do
            for _, layout in ipairs(wave) do
                director:despawn_bnpc(layout)
            end
        end
        director:set_bgm(BGM_DUTY)
        director:command(COMMAND_LINE, { COMMAND_LINE_ARG })
        director:objective(1, 2)
        talk(director, 33508)
        director:map_effect(4, 2, 1)
        director:show_eobj(EOBJ_DEVICE)
        director:eobj_control(EOBJ_DEVICE, CONTROL_TARGETABLE, { 1 })
        after(7.0, function(d) talk(d, 33509) end)
    elseif layout_id == BOSS2 then
        fight = nil
        later = {}
        dropHelpers(director, 2)
        director:set_bgm(BGM_DUTY)
        director:command(COMMAND_LINE, { COMMAND_LINE_ARG })
        director:objective(3, 4)
        talk(director, 33511)
        director:map_effect(7, 2, 1)
        after(2.1, function(d) d:map_effect(10, 2, 1) end)
        after(7.0, function(d) talk(d, 33521) end)
    elseif layout_id == BOSS3 then
        fight = "over"
        later = {}
        dropHelpers(director, 3)
        director:show_eobj(EOBJ_EXIT)
        director:play_cutscene(CUTSCENE_END, CUTSCENE_END_PARAMS)
        director:complete_duty()
    end
    if done.ambush and not done.talk_33512 and allDead(AMBUSH) then
        once("talk_33512")
        talk(director, 33512)
        after(3.0, function(d)
            d:map_effect(11, 2, 1)
            d:eobj_control(EOBJ_WAY, CONTROL_WAY, { 1, DIRECTOR_ID })
        end)
    end
    if not done.two and countDead(SIX) >= SIX_DEAD_FOR_TWO then
        once("two")
        comeOut(director, TWO)
        after(1.0, function(d) talk(d, 33522) end)
    end
    -- All eight, not the two alone: in Bob's run of 2026-11-18 (6320ae56) one of the six outlived the two by 4.9 s
    -- and the cutscene waited for it. 2.1 to 2.4 s after the last of them fell in the four runs.
    if done.two and not done.cutscene and allDead(TWO) and allDead(SIX) then
        once("cutscene")
        after(CUTSCENE_LAST_AFTER, function(d)
            d:play_cutscene(CUTSCENE_LAST, CUTSCENE_LAST_PARAMS)
            d:hide_eobj(EOBJ_BEFORE_LAST)
            bringLastBoss(d)
        end)
        after(CUTSCENE_LAST_AFTER + 1.0, function(d)
            for index = 12, 14 do
                d:map_effect(index, 2, 1)
            end
        end)
    end
end

-- The cutscene before the last room over: everyone in it, where a wipe sends them from now on.
function onSceneFinished(director, scene)
    if done.cutscene and not done.last_room then
        once("last_room")
        director:warp_players(LAST_ROOM.x, LAST_ROOM.y, LAST_ROOM.z, LAST_ROOM.rotation)
        -- A fall from here on sends the party to the entrance as anywhere (Bob's two falls of 2026-11-18, capture
        -- 6320ae56: there 4.5 s after he fell, whole again), and the shortcut by it (GimmickAccessor 74) put him
        -- outside the last room both times, the boss and its helpers new 0.4 s after he came.
        director:update_shortcut_spot(SHORTCUT_LAST.x, SHORTCUT_LAST.y, SHORTCUT_LAST.z, SHORTCUT_LAST.rotation)
        after(1.8, function(d) d:objective(4, 5) end)
    end
end

-- Everyone down: the fight they fell in is forgotten (the bosses' rooms reset themselves); the last boss comes back.
function onPartyWipe(director)
    if fight == "over" then
        return
    end
    local was = fight
    fight = nil
    later = {}
    helpers = {}
    if was == 1 then
        for _, wave in ipairs(WAVES) do
            for _, layout in ipairs(wave) do
                director:despawn_bnpc(layout)
            end
        end
    elseif was == 3 then
        director:despawn_bnpc(BOSS3)
        dropHelpers(director, 3)
        last_boss_due = true
    end
end

-- Back in after a wipe: the last boss and its helpers come back then (31 and 73 s after the party fell in Connor's
-- run, as he came back into the room.)
function onDutyRecommence(director)
    if last_boss_due then
        last_boss_due = false
        bringLastBoss(director)
    end
end

-- The device by the first boss's room.
function onGimmickAccessor(director, actor_id, id, params)
    if id == EOBJ_DEVICE and dead[BOSS1] then
        director:event_action(EVENT_ACTION_DEVICE, actor_id, id)
        return
    end
    director:finish_gimmick(actor_id)
end

function isGimmickFunctional(id)
    if id == EOBJ_DEVICE then
        return dead[BOSS1] == true
    end
    return true
end

function onEventActionCast(director, actor_id, target)
    director:finish_gimmick(actor_id)
    if target == EOBJ_DEVICE and dead[BOSS1] then
        once("device")
        director:warp_players(DEVICE_TO.x, DEVICE_TO.y, DEVICE_TO.z, DEVICE_TO.rotation)
    end
end

function getDebugShortcut(id)
    return 0
end
