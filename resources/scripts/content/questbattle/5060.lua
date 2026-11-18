-- Cape Westwind (Operation Archon, quest 4521: its QUESTBATTLE0 is InstanceContent 5060, not a QuestBattle; director
-- 0x800313C4, territory 1049, duty 865.) A quest's solo battle (it was a trial once, and its sheet rows still are):
-- entered from the quest (`join_quest_battle`, which makes an InstanceContent's director for it), begun as a duty is
-- (DutyCommence, no scene 60000), won into the quest's sequence 4 and out to Western Thanalan, and lost out to where
-- the player came in from, as a quest battle is: nobody is raised inside.
--
-- Six retail runs: Bob's Chronofoil captures 667a68c5 of 2026-11-16 (`questkit chronofoil --battle 0.16`: won, 527 s,
-- a level 50 White Mage synced to 49), 9ba7d981 of 2026-11-17 (a Summoner; visits 0.6, lost 315 s in, and 0.8, won
-- in 361 s) and 178b0de0 of 2026-11-18 (a level 50 Paladin; visits 0.4, lost in the second part, and 0.6, won, its
-- first part 311 s long), and Connor's e834e436 (visit 0.240: a White Mage, won.) With the last two came Kawari
-- Witness's facts (statuses' timers, his health after the cutscene.) Layout IDs, levels, HP, positions and flags are
-- their SpawnNpcs' (drop-in `territory_1049.json`.) Times below are seconds from the talk that begins each part.
--
-- The boss (9082104, base 14581, BNpcName 2160, 24609 HP) auto-attacks (6499) every 3 s when he isn't casting, and
-- stands through each cast's animation lock. Twelve helpers (base 9020, layout 0, 44 HP) stand on his spot and cast
-- where the director puts them.
--
--   The pull: talk 33400. His first part (the timeline's points before any anchor), the same clock in all six runs:
--         28871 at 6.1 and 28870 at 11.3 where he stands, facing the player; 28864 at 21.5 (talk 33403) and 6.0 s
--         after it began a row of four of the helpers' 28865 a second apart, the row of the three nearest the player
--         as the cast began (`nearestRows`: the second in the four older runs, the first in the Paladin's two, and
--         in all three of its uses in the long one); 28866 at 37.7 (talk 33402, ActorControl
--         63 [7]): his shield up, statuses 680 and 2195 on him from its end for 7.76 s, then 63 [0]. He doesn't move,
--         turn or attack from the cast's start until they're gone (12.8 s), and a hit from in front of him then is
--         blocked for nothing and answered a second later by a helper's 28867 from his spot on whoever struck (769
--         and Knockback 7): the one hit from 0 degrees off his facing in the four runs; the nineteen from 46 to 68
--         degrees off landed. 28872 at 52.9, a leap to the middle (an AoeEffect8 at no one whose position is the
--         middle, `aim_action`; SnapNextMove and the move there 1.1 s after it; he does nothing else for its 2.1 s
--         lock, and stays in the middle through what he casts next); 28868 there (talk
--         33424 the first time) and 4.1 s after it began the arena's map effect and two helpers' 28869 (7.7 s casts,
--         their markers 5.5 s in), the effect off as they land; again at 68.1 with the other two spots; 28871 at
--         82.4, 28870 at 93.5 and 103.7, 28871 at 114.9. Still above 15 % then (the long run alone, which went
--         round twice), a round of 117.5 s from 126.1: 28864; 28870 at 7.2 and 13.4; 28872 at 19.6 and 28871 at
--         21.6; 28872 at 31.8, 28868 at 33.9 and 28870 at 39.2; the same three at 45.3, 47.4 and 52.6; 28872 at
--         58.7 and 28871 at 60.8; 28870 at 72.0; 28866 at 84.1; 28870 at 105.3. The talks (33403, 33402, 33424)
--         came with each action's first use only. 28868's map effects and spots there: the first two uses as
--         above, then (64, 128) with spots 3 and 5 and (1, 2) with 1 and 2, turn about.
--   Below 15 % of his health he leaves that clock as soon as he's free (his cast landed and its animation lock
--         over): 28872; talk 33425 2.2 s after it and 28873 every 6.15 s from 2.4 s (the timeline's points after
--         33425.) The six runs, by his health in the samples: 14.9 % during a 28871, the leap 2.1 s after it landed
--         (its lock); 14.5 % in a 28870's lock, the leap as it ended; 13.8 % in the clock's own leap's lock, the
--         next as it ended in place of 28868; 15.27 % with him free and the leap 0.7 s later (14.6 % just after it);
--         14.4 % with him free, the leap 1.1 s later; and in the long run 15.26 % as a round's 28868 began (no
--         push), 14.96 % half a second into it, and he was free 5.1 s after it began but cast the round's 28870
--         5.2 s after it as ever, then leapt as that one's lock ended: 28868 and the 28870 behind it go together
--         (`BUSY_SPOTS_THEN_LINE`; seen once.)
--   His health at 1 (it goes no lower): his cast stopped (ActorControl 15 [537, 1, action]), untargetable, talk 33404,
--         (at once in five runs, two of them in a 28873's lock; in the long one 2.8 s after his health was at 1,
--         as the 28873 that had just landed's lock ended: not explained, and not done here)
--         ActionTimeline 10500, ActorControl 63 [27] 1.5 s on; at 2.5 s three of base 14582 and one of 14583 where
--         the player came in, standing; at 7.0 s talk 33405 and they run in to four spots by the entrance; at 11.0 s
--         they can be targeted and fight, with the director's 0x10000001 and 0x10000002 [33421]; talks 33406, 33407,
--         33429 at 14.1, 21.1 and 28.1 s. He casts 28874 from the middle 43.4 s in and every 14.25 s. Each of the
--         four yells as it falls; his NpcYell 13651 35.2 s in (whenever the four fell.) The four refuse Stun.
--   The four down: the music off (SetBGM 1); NpcYell 13809 and ActionTimeline 10502 at 5.0 s, talk 33409 at 15.1 s,
--         ActionTimeline 10503 at 18.2 s, 63 [0] at 21.15 s, status 2056 (param 95) at 21.6 s, SetBGM 107 and
--         NpcYell 13810 at 22.2 s; at 26.2 s targetable again at full health, 0x10000001 and 0x10000002 [33422],
--         talk 33410.
--   His second part (anchor 33410): 28875 on the player (status 2656 for 3.5 s), and 0.6 s after it lands he steps
--         up to 2 yalms from them (SnapNextMove and a move); 28877 and 28876 (5.7 s casts; NpcYells 13650 and 13649
--         the first two times each), 28878 (talk 33426) and 5.07 s after it began three rows of the helpers' 28865,
--         four seconds apart, the nearest to the player as the cast began first and the furthest last (seven uses
--         of seven: 1, 2, 3 three times, 3, 1, 2 twice, 1, 3, 2 twice); 28876; 28872 and 28868 (talk 33427 with the
--         first) with four spots, then with four again; 28877, 28876, 28877, and round again from 28878 every
--         113.3 s (seen once.) Status 2056 is gone as his health reaches 1.
--   His health at 1 again: ActorControl 537 [himself], the helpers' casts stopped (15 [537, 1, 28865]),
--         untargetable, NpcYell 13811, ActionTimeline 10506, 63 [5] at 2.0 s; at 6.0 s SetBGM 1 and the director's
--         scene 3 (cutscene 2874, then 0, 173, 0, 0), 47 s. As it ends: 63 [0] and his health 55 % (13534: from 1
--         to that within 0.7 s of the control, Kawari Witness), him in the middle facing north; the
--         player put at (-680.02, 63.86, -810.64) 1.5 s on; at 6.0 s ActorControl 35 [0, 10, player, 15], his 28879
--         on the player (status 3134: the chain, which holds them where they stand, free to act, until they leave;
--         the server's condition 43, `modifiers::is_held_in_place`) and 63 [6]; at 6.9 s a helper's 28880 on the
--         player from 2.5 yalms in front of him (`PULLED_TO`): the Attract sheet's row 17 (effect 32, not a
--         Knockback), which draws them to the helper; at 7.8 s targetable, talk 33414. 35 is the tether drawn
--         between them. (Until 2026-11-18 the helper stood on the player and knocked them back with the Knockback
--         sheet's row 17, 5 yalms the other way, and nothing held them: Bob's mark, "third stage chain doesn't work
--         in anyway like it should it should pull you to him, and freeze you in place only to attack".)
--   The last part (anchor 33414): 28881 at once; a helper's 28882 beside him at 4.1 s; status 3008 on him, one
--         stack more before each 28883 (4.7 s casts, 13.15 s apart, talks 33415-33419 with the first five); status
--         2194 on the player from 7.8 s to his death. Neither has a timer, nor has 3134 (28879's, on the player
--         until they leave.)
--   He falls: the director's scene 4 (flags 0x08502403; cutscene 263, then 1) in a GameComplete event; as it ends the
--         quest to sequence 4 and out to Western Thanalan at (-462.43, 22.94, -477.23), where the quest's entering
--         scene takes it to 255.
--   The player falls (9ba7d981, visit 0.6): at once the director's scene 5 (flags 0x0C002001, params 1, 2) in a
--         GameProgress event with argument 2; 3.1 s later the event ends and the player is in Western Thanalan again.
--         No wipe, no recommence, nobody raised inside (Kawari's did both until 2026-11-18.)
--
-- Not seen on retail, so guessed: how far to either side of his facing the raised shield reaches (45 degrees here:
-- blocked up to 23 degrees off, landed from 44, in the six runs and Kawari Witness's count); whether talk 33427
-- comes again with the second part's third 28868 (not here: the first part's 33424 didn't); the director's
-- 0x10000002 [33420] comes with DutyCommence here, retail's before the opening scene.
-- Unknown: what ActorControl 63 and statuses 2056, 2656, 3008 and 2194 do (sent and given as retail did, nothing
-- more.)
--
-- Where each action reaches is the Action sheet's, and the four runs agree with it (the player's place as each cast
-- ended, hit or missed): 28871 a circle of 7 round him (missed at 7.1, hit at 5.9); 28877 one of 14 (missed at 14.1,
-- hit at 10.3); 28870 and 28874 a line 5 wide before him (missed 2.7 to its side, hit at 2.0 and 0.7); 28876
-- everything before him, 30 wide (hit in front, missed beside and behind); 28873 and 28883 everyone (hit at 4.7 to
-- 11.1); the helpers' 28865 a circle of 7 where each is put and 28869 one of 11. None of them reaches further for
-- his hitbox (1.5), and none past its marker's edge (`action::AREA_EDGE_REACH`, `ActionInfo::area_hitbox`.)

START_POSITION = { x = -678.72, y = 63.67, z = -808.74 }
START_ROTATION = -2.115
OPENING_SCENE_FLAGS = 0x0C002001
EXIT_AFTER_WIN = { territory = 140, x = -462.43, y = 22.94, z = -477.23, rotation = 0.0 }

BOSS = 9082104
HELPER = 0
HELPER_COUNT = 12
ADDS = { 9283147, 9283146, 9082105, 9082106 }
-- What each of the four yells as it falls.
ADD_YELL = { [9082106] = 13647, [9283147] = 13822, [9082105] = 13648, [9283146] = 13821 }
-- Where each runs to from where it came in, 7.0 s after talk 33404 (there by 10 s, at about 5.7 yalms a second; the
-- same spots in the three runs that got there.)
ADD_STANDS = {
    [9283147] = { -683.2, 64.0, -806.7 },
    [9283146] = { -678.8, 63.7, -807.9 },
    [9082105] = { -676.9, 63.5, -811.7 },
    [9082106] = { -678.7, 63.5, -815.6 },
}
ADD_RUN_SPEED = 5.7

NAME_BOSS = 2160
ICON_BOSS = 73175
NAME_OTHER = 11276
ICON_OTHER = 73176

MIDDLE = { x = -689.02, y = 64.35, z = -815.03 }
-- Where he stands for the last part: the middle, 0.73 higher, facing north.
LAST_STAND = { x = -689.02, y = 65.08, z = -815.03 }
PLAYER_AFTER_CUTSCENE = { x = -680.02, y = 63.86, z = -810.64, rotation = -2.024 }
HELPER_BESIDE_HIM = { x = -687.89, y = 64.96, z = -814.48 }
-- Where the helper that draws the player in stands (its ActorSetPos, 0.1 s before its 28880): 2.5 yalms from him on
-- the way to where the player was put. The player's own client had them 0.1 yalms from it 0.9 s later, in four runs.
PULLED_TO = { x = -686.75, y = 64.86, z = -813.90 }
LEAP_MOVE_SPEED = 60

-- The helpers' 28865: three rows of four spots. The first part uses the second row alone.
ROWS = {
    {
        { -698.45, 66.18, -819.61 }, { -692.16, 65.42, -816.56 }, { -685.88, 64.71, -813.47 },
        { -679.56, 63.80, -810.42 },
    },
    {
        { -702.39, 66.33, -811.52 }, { -696.10, 65.51, -808.44 }, { -689.81, 64.71, -805.39 },
        { -683.50, 64.19, -802.33 },
    },
    {
        { -694.51, 66.15, -827.70 }, { -688.23, 65.48, -824.64 }, { -681.94, 64.41, -821.59 },
        { -675.62, 63.61, -818.51 },
    },
}
-- The helpers' 28869: its spots, and for each use of 28868 the map effect (index 0: timeline, state) and the spots
-- that go with it. The first part's two, then the second's two.
SPOTS = {
    { -679.16, 63.95, -816.77 }, { -695.95, 65.45, -807.83 }, { -684.32, 64.16, -806.18 },
    { -697.99, 66.06, -819.39 }, { -687.65, 65.42, -824.92 },
}
PATTERNS = {
    [1] = { { timeline = 2, state = 1, spots = { 1, 2 } }, { timeline = 32, state = 16, spots = { 3, 4 } } },
    [2] = {
        { timeline = 512, state = 256, spots = { 3, 1, 4, 2 } },
        { timeline = 2048, state = 1024, spots = { 1, 5, 4, 3 } },
    },
}
-- The first part's from its third use on, turn about (the long run's four: at 160.0, 173.5, 277.7 and 291.4 s.)
PATTERNS_ROUND = { { timeline = 128, state = 64, spots = { 3, 5 } }, { timeline = 2, state = 1, spots = { 1, 2 } } }
MAP_EFFECT_OFF = { timeline = 8, state = 4 }
HELPER_FACING = 1.118
-- After his 28875 lands he steps up to the one it hit: SnapNextMove and a move 0.6 s on, to 2 yalms from them (he
-- began his next cast 2.0 yalms off in all four runs.)
STEP_AFTER = 0.5
STEP_TO = 2.0

ACTION_ROW = 28864
ACTION_ROW_HELPER = 28865
ACTION_STANCE = 28866
ACTION_SPOTS = 28868
ACTION_SPOTS_HELPER = 28869
ACTION_LEAP = 28872
ACTION_THRICE = 28873
ACTION_ON_PLAYER = 28875
ACTION_SIDE_A = 28876
ACTION_SIDE_B = 28877
ACTION_ROWS = 28878
ACTION_BIND = 28879
ACTION_DRAW_IN = 28880
ACTION_RISE = 28881
ACTION_BESIDE = 28882
ACTION_LAST = 28883

-- The timeline's anchors (`restart_timeline`): the talk each part began with.
PART_PUSH = 33425
PART_ADDS = 33404
PART_SECOND = 33410
PART_NONE = 13811
PART_LAST = 33414

STATUS_STANCE_1 = 680
STATUS_STANCE_2 = 2195
STANCE_SECONDS = 7.76
-- From the cast's start: how long he stands as he is, and when the shield is up (the statuses are given 5.05 s in; a
-- hit 5.4 s in landed, one 5.9 s in was blocked.)
STANCE_HELD = 12.8
STANCE_GUARD_FROM = 5.5
-- How far to each side of his facing it reaches (a guess: see the header.)
STANCE_GUARD_DEGREES = 45
ACTION_COUNTER = 28867
COUNTER_AFTER = 1.0
-- His health (%) below which the first part's clock gives way to 28872 and 28873.
PUSH_BELOW = 15
-- How long from an action's start until he's free again: its cast (the bar and 0.3 s) and its animation lock (his
-- ActionEffects': 2.1 s for 28868, 28871 and 28872, 3.1 for 28870 and 28873, 4.1 for 28864); 28866 holds him until
-- his shield is down.
BUSY_SECONDS = {
    [28864] = 7.1, [28866] = STANCE_HELD, [28868] = 5.1, [28870] = 6.1, [28871] = 5.1, [28872] = 2.1,
    [28873] = 6.1,
}
BUSY_SECONDS_DEFAULT = 5.1
-- A round's 28868 (the first part's third use on) has a 28870 5.2 s behind it that the push doesn't take the place
-- of: he's held until that one has begun.
BUSY_SPOTS_THEN_LINE = 5.6
STATUS_AFTER_ADDS = 2056
STATUS_AFTER_ADDS_PARAM = 95
STATUS_LAST = 3008
STATUS_PLAYER_LAST = 2194
HEALTH_LAST = 13534

CONTROL_63 = 63
CONTROL_35 = 35
CONTROL_537 = 537
COMMAND_TEXT_1 = 0x10000001
COMMAND_TEXT_2 = 0x10000002
TEXT_START = 33420
TEXT_ADDS = 33421
TEXT_SECOND = 33422
BGM_NONE = 1
BGM_SECOND = 107

CUTSCENE_LAST_STAND = 2874
CUTSCENE_LAST_STAND_PARAMS = { 0, 173, 0, 0 }
CUTSCENE_SCENE = 3
QUEST_SEQUENCE_AFTER_WIN = 4
FINISH_SCENE = 4
FINISH_SCENE_FLAGS = 0x08502403
FINISH_PARAMS = { 263, 1 }
-- The player down: the director's scene 5, and out 3.1 s after it began.
LOSS_SCENE = 5
LOSS_SCENE_FLAGS = 0x0C002001
LOSS_SCENE_PARAMS = { 1, 2 }
LOSS_EVENT_ARG = 2
LOSS_OUT_AFTER = 3.0

boss = nil
helpers = {}
next_helper = 0
-- 0 before the pull, 1 and 2 his two parts, "adds", "between", "cutscene", "rising", "last", "over"; "lost" and "out"
-- once the player is down.
part = 0
part_number = 1
adds_left = 0
recovered = false
used = {}
last_stacks = 0
player_id = nil
player_at = nil
boss_at = nil
-- The first part's push (below `PUSH_BELOW`) has begun; the update his cast lands in; his shield is up; a counter is
-- on its way.
pushing = false
free_at = 0
guarding = false
countering = false
ticks = 0
updating = false
later = {}

-- Half-second updates from now. Set between two updates (a death, a cast, a scene's end), it counts from the next.
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

function talk(director, text, ms, other)
    if other then
        director:battle_talk(BOSS, NAME_OTHER, text, ICON_OTHER, ms or 6000)
    else
        director:battle_talk(BOSS, NAME_BOSS, text, ICON_BOSS, ms or 6000)
    end
end

function count(action)
    used[action] = (used[action] or 0) + 1
    return used[action]
end

function bring(director)
    -- (The battle simulator's spawns return nothing: the layout IDs stand in there.)
    boss = director:spawn_bnpc(BOSS) or BOSS
    director:set_health_floor(BOSS, 1)
    helpers = {}
    for i = 1, HELPER_COUNT do
        helpers[i] = director:spawn_bnpc(HELPER) or HELPER
        director:hold_bnpc(helpers[i])
    end
end

function reset()
    part = 0
    part_number = 1
    adds_left = 0
    recovered = false
    pushing = false
    free_at = 0
    guarding = false
    countering = false
    used = {}
    last_stacks = 0
    later = {}
    next_helper = 0
end

function onSetup(director)
    director:set_sequence(1)
    -- The player down isn't a duty's wipe here (`onPlayerDeath`.)
    director:hold_wipe(true)
    reset()
    bring(director)
    -- His leap's landing names where he comes down (retail's AoeEffect8 for 28872: the middle, at the higher of its
    -- two heights, wherever he leapt from.)
    director:aim_action(BOSS, ACTION_LEAP, MIDDLE.x, LAST_STAND.y, MIDDLE.z)
end

function onCommence(director)
    director:command(COMMAND_TEXT_2, { TEXT_START })
end

-- The next helper in turn, put at a spot.
function helperAt(director, spot, facing)
    next_helper = next_helper % #helpers + 1
    local helper = helpers[next_helper]
    director:warp_bnpc(helper, spot[1], spot[2], spot[3], facing or math.pi)
    return helper
end

-- The three rows, the nearest to the player first (to the line through each row's spots, between its ends): which
-- row 28864 uses and the order of 28878's, as each cast begins. Nine uses of 28864 and seven of 28878 in the six runs.
function nearestRows()
    local at = player_at or START_POSITION
    local order = {}
    for i, row in ipairs(ROWS) do
        local ax, az = row[1][1], row[1][3]
        local bx, bz = row[#row][1], row[#row][3]
        local along = ((at.x - ax) * (bx - ax) + (at.z - az) * (bz - az)) / ((bx - ax) ^ 2 + (bz - az) ^ 2)
        along = math.max(0, math.min(1, along))
        local dx, dz = at.x - (ax + (bx - ax) * along), at.z - (az + (bz - az) * along)
        order[i] = { row = i, distance = math.sqrt(dx * dx + dz * dz) }
    end
    table.sort(order, function(a, b)
        if a.distance == b.distance then
            return a.row < b.row
        end
        return a.distance < b.distance
    end)
    return { order[1].row, order[2].row, order[3].row }
end

-- A row of the helpers' 28865, one a second, `first` seconds from now.
function castRow(director, row, first)
    for i, spot in ipairs(ROWS[row]) do
        after(first + (i - 1), function(d)
            if part == 1 or part == 2 then
                d:actor_use_action(helperAt(d, spot), ACTION_ROW_HELPER)
            end
        end)
    end
end

function castSpots(director)
    local patterns = PATTERNS[part_number]
    local use = count("pattern")
    local pattern = patterns[(use - 1) % #patterns + 1]
    if part_number == 1 and use > #patterns then
        pattern = PATTERNS_ROUND[(use - #patterns - 1) % #PATTERNS_ROUND + 1]
    end
    director:map_effect(0, pattern.timeline, pattern.state)
    for _, spot in ipairs(pattern.spots) do
        director:actor_use_action(helperAt(director, SPOTS[spot], HELPER_FACING), ACTION_SPOTS_HELPER)
    end
    after(8.0, function(d)
        d:map_effect(0, MAP_EFFECT_OFF.timeline, MAP_EFFECT_OFF.state)
    end)
end

function onActorCast(director, layout_id, action_id, actor_id)
    if layout_id ~= BOSS then
        return
    end
    local uses = count(action_id)
    free_at = ticks + math.floor((BUSY_SECONDS[action_id] or BUSY_SECONDS_DEFAULT) * 2 + 0.5)
    if action_id == ACTION_ROW then
        if uses == 1 then
            talk(director, 33403, 5000)
        end
        castRow(director, nearestRows()[1], 6.0)
    elseif action_id == ACTION_STANCE then
        if uses == 1 then
            talk(director, 33402)
        end
        director:bnpc_control(BOSS, CONTROL_63, { 7 })
        -- He stands as he is until the shield is down: no step, no turn, no auto-attack (none in 13.7 s, four runs.)
        director:busy_bnpc(BOSS, STANCE_HELD)
        local this_part = part
        after(5.0, function(d)
            if boss ~= nil and part == this_part then
                d:gain_effect(boss, STATUS_STANCE_1, 0, STANCE_SECONDS)
                d:gain_effect(boss, STATUS_STANCE_2, 0, STANCE_SECONDS)
            end
        end)
        after(STANCE_GUARD_FROM, function(d)
            if part == this_part then
                guarding = true
                d:guard_front(BOSS, STANCE_GUARD_DEGREES)
            end
        end)
        after(STANCE_HELD, function(d)
            lowerGuard(d)
            d:bnpc_control(BOSS, CONTROL_63, { 0 })
        end)
    elseif action_id == ACTION_LEAP then
        -- (SnapNextMove and an ActorMove with a run's speed byte, 60, facing as he had: Bob's capture 178b0de0.)
        director:delay(1.1)
        director:jump_bnpc(BOSS, MIDDLE.x, MIDDLE.y, MIDDLE.z, nil, LEAP_MOVE_SPEED, true)
        director:delay(0)
    elseif action_id == ACTION_SPOTS then
        if uses == 1 then
            talk(director, part_number == 1 and 33424 or 33427, 5000)
        end
        if part_number == 1 and uses > #PATTERNS[1] then
            free_at = ticks + math.floor(BUSY_SPOTS_THEN_LINE * 2 + 0.5)
        end
        local this_part = part
        after(4.0, function(d)
            if part == this_part then
                castSpots(d)
            end
        end)
    elseif action_id == ACTION_THRICE then
        if uses == 1 then
            talk(director, 33425)
        end
    elseif action_id == ACTION_SIDE_B then
        if uses <= 2 then
            director:yell(BOSS, 13650)
        end
    elseif action_id == ACTION_SIDE_A then
        if uses <= 2 then
            director:yell(BOSS, 13649)
        end
    elseif action_id == ACTION_ROWS then
        talk(director, 33426)
        -- The three rows four seconds apart, the nearest to the player first.
        for i, row in ipairs(nearestRows()) do
            castRow(director, row, 5.0 + (i - 1) * 4.0)
        end
    elseif action_id == ACTION_LAST then
        last_stacks = last_stacks + 1
        if boss ~= nil then
            director:gain_effect(boss, STATUS_LAST, last_stacks, 0)
        end
        if last_stacks <= 5 then
            talk(director, 33414 + last_stacks)
        end
    end
end

function lowerGuard(director)
    if guarding then
        guarding = false
        director:guard_front(BOSS, nil)
    end
end

-- His 28875 landed on the player: he steps up to them.
function onActionLanded(director, layout_id, action_id, players)
    if layout_id ~= BOSS or action_id ~= ACTION_ON_PLAYER then
        return
    end
    after(STEP_AFTER, function(d)
        if part ~= 2 or boss_at == nil or player_at == nil then
            return
        end
        local dx, dz = boss_at.x - player_at.x, boss_at.z - player_at.z
        local distance = math.sqrt(dx * dx + dz * dz)
        if distance > STEP_TO + 0.2 then
            d:jump_bnpc(BOSS, player_at.x + dx / distance * STEP_TO, player_at.y, player_at.z + dz / distance * STEP_TO)
        end
    end)
end

-- His raised shield took a hit from in front (`director:guard_front`): a second later a helper on his spot answers
-- whoever struck.
function onActorGuarded(director, layout_id, attacker)
    if layout_id ~= BOSS or not guarding or countering then
        return
    end
    countering = true
    after(COUNTER_AFTER, function(d)
        countering = false
        if (part == 1 or part == 2) and boss_at ~= nil then
            d:actor_use_action(helperAt(d, { boss_at.x, boss_at.y, boss_at.z }), ACTION_COUNTER, attacker)
        end
    end)
end

-- The first part, his health below `PUSH_BELOW`: once he's free, the clock gives way to the timeline's points after
-- 33425 (28872 at once, then 28873 until his health is at its floor.)
function push(director)
    pushing = true
    if part ~= 1 then
        return
    end
    if free_at <= ticks then
        director:restart_timeline(BOSS, PART_PUSH)
        return
    end
    -- (Looked at again then: a cast begun meanwhile holds him longer, as a round's 28870 behind its 28868 does.)
    after((free_at - ticks) / 2, push)
end

-- His health at 1 the first time: the four come.
function bringAdds(director)
    part = "adds"
    lowerGuard(director)
    director:stop_cast(BOSS)
    director:restart_timeline(BOSS, PART_ADDS)
    director:root_bnpc(BOSS, 3600)
    director:set_targetable(BOSS, false)
    director:set_invulnerable(BOSS, true)
    talk(director, 33404)
    director:play_action_timeline(BOSS, 10500)
    director:jump_bnpc(BOSS, MIDDLE.x, MIDDLE.y, MIDDLE.z)
    after(1.5, function(d)
        d:bnpc_control(BOSS, CONTROL_63, { 27 })
    end)
    after(2.5, function(d)
        adds_left = #ADDS
        for _, add in ipairs(ADDS) do
            d:spawn_bnpc(add)
            d:hold_bnpc(add)
            d:set_targetable(add, false)
        end
    end)
    after(7.0, function(d)
        talk(d, 33405, 5000, true)
        for _, add in ipairs(ADDS) do
            local stand = ADD_STANDS[add]
            d:sweep_bnpc(add, stand[1], stand[2], stand[3], ADD_RUN_SPEED, 0)
        end
    end)
    after(11.0, function(d)
        d:command(COMMAND_TEXT_1, { TEXT_ADDS })
        d:command(COMMAND_TEXT_2, { TEXT_ADDS })
        for _, add in ipairs(ADDS) do
            d:set_targetable(add, true)
            d:hold_bnpc(add, false)
            d:set_hate(add, nil, 1)
        end
    end)
    after(14.0, function(d) talk(d, 33406) end)
    after(21.0, function(d) talk(d, 33407, 6000, true) end)
    after(28.0, function(d) talk(d, 33429, 6000, true) end)
    -- His NpcYell 13651 is on the same clock: 35.14, 35.22 and 35.17 s after talk 33404 in three runs (Bob's
    -- Chronofoil captures 667a68c5 and 9ba7d981), whenever the four fell (the first at 32.0, 44.9 and 19.2 s.)
    after(35.0, function(d) d:yell(BOSS, 13651) end)
end

-- The four down: he gets up, at full health.
function afterAdds(director)
    part = "between"
    -- His 28874 is for while they're up: none came after the last of them fell in the six runs (the next would have
    -- been due before talk 33410 in five.) Here he went on casting it through the talks and into the second part
    -- (Bob's mark of 2026-11-18: "he used his ground effect during the stage transition".)
    director:restart_timeline(BOSS, PART_NONE)
    director:stop_cast(BOSS)
    director:set_bgm(BGM_NONE)
    after(5.0, function(d)
        d:yell(BOSS, 13809)
        d:play_action_timeline(BOSS, 10502)
    end)
    after(15.0, function(d) talk(d, 33409) end)
    after(18.0, function(d) d:play_action_timeline(BOSS, 10503) end)
    after(21.0, function(d) d:bnpc_control(BOSS, CONTROL_63, { 0 }) end)
    after(21.5, function(d)
        if boss ~= nil then
            d:gain_effect(boss, STATUS_AFTER_ADDS, STATUS_AFTER_ADDS_PARAM, 0)
        end
    end)
    after(22.0, function(d)
        d:set_bgm(BGM_SECOND)
        d:yell(BOSS, 13810)
    end)
    after(26.0, function(d)
        part = 2
        part_number = 2
        used = {}
        d:set_health(BOSS)
        d:set_invulnerable(BOSS, false)
        d:set_targetable(BOSS, true)
        d:bnpc_control(BOSS, CONTROL_63, { 0 })
        d:command(COMMAND_TEXT_1, { TEXT_SECOND })
        d:command(COMMAND_TEXT_2, { TEXT_SECOND })
        talk(d, 33410)
        d:root_bnpc(BOSS, 0)
        d:restart_timeline(BOSS, PART_SECOND)
    end)
end

-- His health at 1 the second time: the cutscene, then the last part.
function lastStand(director)
    part = "cutscene"
    later = {}
    lowerGuard(director)
    countering = false
    director:restart_timeline(BOSS, PART_NONE)
    director:stop_cast(BOSS)
    for _, helper in ipairs(helpers) do
        director:stop_cast(helper)
    end
    director:busy_bnpc(BOSS, 3600)
    director:root_bnpc(BOSS, 3600)
    director:set_invulnerable(BOSS, true)
    if boss ~= nil then
        director:bnpc_control(BOSS, CONTROL_537, { boss })
        -- (Gone 131.7 s after it came in Bob's run of 2026-11-18, which is here: Kawari Witness.)
        director:lose_effect(boss, STATUS_AFTER_ADDS)
    end
    director:set_targetable(BOSS, false)
    director:yell(BOSS, 13811)
    director:play_action_timeline(BOSS, 10506)
    director:map_effect(0, MAP_EFFECT_OFF.timeline, MAP_EFFECT_OFF.state)
    after(2.0, function(d) d:bnpc_control(BOSS, CONTROL_63, { 5 }) end)
    after(6.0, function(d)
        d:set_bgm(BGM_NONE)
        d:play_cutscene(CUTSCENE_LAST_STAND, CUTSCENE_LAST_STAND_PARAMS)
    end)
end

-- Out to where the player came in from, the quest as it was.
function leave(director)
    if part == "lost" then
        part = "out"
        director:fail_quest_battle()
    end
end

-- The player down: retail's scene 5 at once, and out as it ends (3.1 s after it began on retail, whether or not the
-- client has said it's done.) Not a duty's wipe: nobody comes back inside.
function onPlayerDeath(director)
    if part == "over" or part == "lost" or part == "out" then
        return
    end
    part = "lost"
    later = {}
    lowerGuard(director)
    director:restart_timeline(BOSS, PART_NONE)
    director:play_scene(LOSS_SCENE, LOSS_SCENE_FLAGS, LOSS_SCENE_PARAMS, LOSS_EVENT_ARG)
    after(LOSS_OUT_AFTER, leave)
end

function onSceneFinished(director, scene)
    if scene == LOSS_SCENE then
        leave(director)
        return
    end
    if scene ~= CUTSCENE_SCENE or part ~= "cutscene" then
        return
    end
    part = "rising"
    director:bnpc_control(BOSS, CONTROL_63, { 0 })
    -- His health with it: 1 to 13534 (55 %) within 0.7 s of the control (Kawari Witness, Bob's run of 2026-11-18;
    -- the captures first show it 9 s on, less what had landed since: 13425 after a 109, 13461 after a 73.)
    director:set_health(BOSS, HEALTH_LAST)
    director:warp_bnpc(BOSS, LAST_STAND.x, LAST_STAND.y, LAST_STAND.z, 0)
    after(1.5, function(d)
        d:warp_players(PLAYER_AFTER_CUTSCENE.x, PLAYER_AFTER_CUTSCENE.y, PLAYER_AFTER_CUTSCENE.z,
            PLAYER_AFTER_CUTSCENE.rotation)
    end)
    after(6.0, function(d)
        d:busy_bnpc(BOSS, 0)
        if player_id ~= nil then
            d:bnpc_control(BOSS, CONTROL_35, { 0, 10, player_id, 15 })
        end
        if boss ~= nil then
            d:actor_use_action(boss, ACTION_BIND, player_id)
        end
        d:bnpc_control(BOSS, CONTROL_63, { 6 })
    end)
    after(7.0, function(d)
        local helper = helperAt(d, { PULLED_TO.x, PULLED_TO.y, PULLED_TO.z }, PLAYER_AFTER_CUTSCENE.rotation)
        d:actor_use_action(helper, ACTION_DRAW_IN, player_id)
    end)
    after(8.0, function(d)
        part = "last"
        d:set_health_floor(BOSS, 0)
        d:set_invulnerable(BOSS, false)
        d:set_targetable(BOSS, true)
        talk(d, 33414)
        d:restart_timeline(BOSS, PART_LAST)
        after(4.0, function(d2)
            if part == "last" then
                local spot = HELPER_BESIDE_HIM
                d2:actor_use_action(helperAt(d2, { spot.x, spot.y, spot.z }), ACTION_BESIDE)
            end
        end)
        after(8.0, function(d2)
            if part == "last" and player_id ~= nil then
                d2:gain_effect(player_id, STATUS_PLAYER_LAST, 0, 0)
            end
        end)
    end)
end

function onUpdate(director, health, player_health, info)
    ticks = ticks + 1
    updating = true
    runLater(director)
    watch(director, health, info)
    updating = false
end

function watch(director, health, info)
    if info == nil then
        return
    end
    if info.player ~= nil then
        player_id = info.player.id
        player_at = { x = info.player.x, y = info.player.y, z = info.player.z }
    end
    local him = info[BOSS]
    if him == nil then
        return
    end
    if him.x ~= nil then
        boss_at = { x = him.x, y = him.y, z = him.z }
    end
    -- The pull: he has someone to go for, or has been hit.
    local left = health[BOSS]
    if part == 0 and (him.target ~= nil or (left ~= nil and left < 100)) then
        part = 1
        talk(director, 33400)
    end
    -- His health at its floor (1 HP; the simulator's health is a percentage, 0.1 at the least for one still up.)
    local at_floor = (him.hp ~= nil and him.hp <= 1) or (left ~= nil and left <= 0.1)
    -- (His health to the tenth of a percent where the update gives his HP; the whole percent otherwise.)
    local percent = left
    if him.hp ~= nil and him.max_hp ~= nil and him.max_hp > 0 then
        percent = him.hp * 100 / him.max_hp
    end
    if part == 1 and at_floor then
        bringAdds(director)
    elseif part == 1 and not pushing and percent ~= nil and percent < PUSH_BELOW then
        push(director)
    elseif part == 2 and not at_floor then
        -- (His health is back before it can fall again.)
        recovered = true
    elseif part == 2 and recovered and at_floor then
        lastStand(director)
    end
end

function onActorDeath(director, layout_id, position)
    if ADD_YELL[layout_id] ~= nil and part == "adds" then
        director:yell(layout_id, ADD_YELL[layout_id])
        adds_left = adds_left - 1
        if adds_left <= 0 then
            afterAdds(director)
        end
    elseif layout_id == BOSS and part ~= "over" then
        part = "over"
        later = {}
        if player_id ~= nil then
            director:lose_effect(player_id, STATUS_PLAYER_LAST)
        end
        for _, helper in ipairs(helpers) do
            director:despawn_bnpc(helper)
        end
        helpers = {}
        boss = nil
        director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS, nil, false,
            nil, FINISH_PARAMS)
    end
end
