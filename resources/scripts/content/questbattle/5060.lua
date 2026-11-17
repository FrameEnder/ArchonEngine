-- Cape Westwind (Operation Archon, quest 4521: its QUESTBATTLE0 is InstanceContent 5060, not a QuestBattle; director
-- 0x800313C4, territory 1049, duty 865.) A solo duty: entered from the quest (`join_quest_battle`, which makes an
-- InstanceContent's director for it), begun as a duty is (DutyCommence, no scene 60000), won into the quest's
-- sequence 4 and out to Western Thanalan.
--
-- Everything here is one retail run: Bob's Chronofoil capture 667a68c5 of 2026-11-16 (`questkit chronofoil --battle
-- 0.16`: won, 527 s, a level 50 White Mage synced to 49.) Layout IDs, levels, HP, positions and flags are its
-- SpawnNpcs' (drop-in `territory_1049.json`.) Times below are seconds from arriving.
--
-- The boss (9082104, base 14581, BNpcName 2160, 24609 HP) never auto-attacks; he walks after the player and casts.
-- Twelve helpers (base 9020, layout 0, 44 HP) stand on his spot and cast where the director puts them.
--
--   8.2   the pull: talk 33400. His first part (the timeline's points before any anchor): 28871 and 28870 where he
--         stands, facing the player; 28864 (talk 33403) and 6.0 s after it began a row of four of the helpers' 28865
--         a second apart; 28866 (talk 33402, ActorControl 63 [7]; statuses 680 and 2195 on him from its end for
--         7.76 s, then 63 [0]); 28872, a leap to the middle (SnapNextMove 1.1 s after it); 28868 there (talk 33424
--         the first time) and 4.1 s after it began the arena's map effect and two helpers' 28869 (7.7 s casts), the
--         effect off as they land; again with the other two spots; 28871, 28870, 28870, 28871, 28872, then 28873
--         three times 6.15 s apart (talk 33425.)
--   146.9 his health at 1 (it goes no lower): untargetable, talk 33404, ActionTimeline 10500, ActorControl 63 [27]
--         1.5 s on; at 2.45 s three of base 14582 and one of 14583 where the player came in, talk 33405 at 7.0 s,
--         targetable at 11.0 s with the director's 0x10000001 and 0x10000002 [33421]; talks 33406, 33407, 33429 at
--         14.1, 21.1 and 28.1 s. He casts 28874 from the middle 43.4 s in and every 14.25 s. Each of the four yells
--         as it falls; his NpcYell 13651 3.15 s after the first.
--   214.1 the four down: the music off (SetBGM 1); NpcYell 13809 and ActionTimeline 10502 at 5.0 s, talk 33409 at
--         15.1 s, ActionTimeline 10503 at 18.2 s, 63 [0] at 21.15 s, status 2056 (param 95) at 21.6 s, SetBGM 107 and
--         NpcYell 13810 at 22.2 s; at 26.2 s targetable again at full health, 0x10000001 and 0x10000002 [33422],
--         talk 33410.
--   240.3 his second part (anchor 33410): 28875 on the player (status 2656 for 3.5 s), 28877 and 28876 (5.7 s casts;
--         NpcYells 13650 and 13649 the first two times each), 28878 (talk 33426) and 5.07 s after it began three
--         rows of the helpers' 28865, four seconds apart; 28876; 28872 and 28868 (talk 33427) with all four spots,
--         then with four again; 28877, 28876, 28877, 28878.
--   393.0 his health at 1 again: his cast cancelled (ActorControl 537), untargetable, NpcYell 13811, ActionTimeline
--         10506, 63 [5] at 2.0 s; at 6.0 s SetBGM 1 and the director's scene 3 (cutscene 2874, then 0, 173, 0, 0),
--         47 s. As it ends: 63 [0], him in the middle facing north; the player put at (-680.02, 63.86, -810.64) 1.5 s
--         on; at 6.0 s ActorControl 35 [0, 10, player, 15], his 28879 on the player (status 3134) and 63 [6]; a
--         helper's 28880 from where the player stands at 6.9 s (Knockback 17, towards him); at 7.8 s targetable,
--         talk 33414.
--   454.2 the last part (anchor 33414): 28881 at once, and his health 13373 as it lands; a helper's 28882 beside him
--         at 4.1 s; status 3008 on him, one stack more before each 28883 (4.7 s casts, 13.15 s apart, talks
--         33415-33419 with the first five); status 2194 on the player from 7.8 s to his death.
--   523.2 he falls: the director's scene 4 (flags 0x08502403; cutscene 263, then 1) in a GameComplete event; as it
--         ends the quest to sequence 4 and out to Western Thanalan at (-462.43, 22.94, -477.23), where the quest's
--         entering scene takes it to 255.
--
-- Not seen on retail, so guessed: what he does once a part's points run out before his health does (the first part's
-- 28873 goes on every 6.15 s, the second's points from 28878 come round again); losing (a duty's wipe: back to the
-- start and again, `onPartyWipe`); the director's 0x10000002 [33420] comes with DutyCommence here, retail's before
-- the opening scene. Unknown: what ActorControls 63, 35 and 537 and statuses 680, 2195, 2056, 2656, 3134, 3008 and
-- 2194 do (sent and given as retail did, nothing more); 28870's damage (never landed.) Where each action reaches is
-- the Action sheet's.

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

NAME_BOSS = 2160
ICON_BOSS = 73175
NAME_OTHER = 11276
ICON_OTHER = 73176

MIDDLE = { x = -689.02, y = 64.35, z = -815.03 }
-- Where he stands for the last part: the middle, 0.73 higher, facing north.
LAST_STAND = { x = -689.02, y = 65.08, z = -815.03 }
PLAYER_AFTER_CUTSCENE = { x = -680.02, y = 63.86, z = -810.64, rotation = -2.024 }
HELPER_BESIDE_HIM = { x = -687.89, y = 64.96, z = -814.48 }

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
MAP_EFFECT_OFF = { timeline = 8, state = 4 }
HELPER_FACING = 1.118

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
PART_ADDS = 33404
PART_SECOND = 33410
PART_NONE = 13811
PART_LAST = 33414

STATUS_STANCE_1 = 680
STATUS_STANCE_2 = 2195
STANCE_SECONDS = 7.76
STATUS_AFTER_ADDS = 2056
STATUS_AFTER_ADDS_PARAM = 95
STATUS_LAST = 3008
STATUS_PLAYER_LAST = 2194
HEALTH_LAST = 13373

CONTROL_63 = 63
CONTROL_35 = 35
CONTROL_CANCEL_CAST = 537
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

boss = nil
helpers = {}
next_helper = 0
-- 0 before the pull, 1 and 2 his two parts, "adds", "between", "cutscene", "last", "over".
part = 0
part_number = 1
adds_left = 0
first_add_fell = false
recovered = false
used = {}
last_stacks = 0
player_id = nil
player_at = nil
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
    first_add_fell = false
    recovered = false
    used = {}
    last_stacks = 0
    later = {}
    next_helper = 0
end

function onSetup(director)
    director:set_sequence(1)
    reset()
    bring(director)
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
    local pattern = patterns[(count("pattern") - 1) % #patterns + 1]
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
    if action_id == ACTION_ROW then
        talk(director, 33403, 5000)
        castRow(director, 2, 6.0)
    elseif action_id == ACTION_STANCE then
        talk(director, 33402)
        director:bnpc_control(BOSS, CONTROL_63, { 7 })
        after(5.0, function(d)
            if boss ~= nil then
                d:gain_effect(boss, STATUS_STANCE_1, 0, STANCE_SECONDS)
                d:gain_effect(boss, STATUS_STANCE_2, 0, STANCE_SECONDS)
            end
        end)
        after(5.0 + STANCE_SECONDS, function(d)
            d:bnpc_control(BOSS, CONTROL_63, { 0 })
        end)
    elseif action_id == ACTION_LEAP then
        director:delay(1.1)
        director:jump_bnpc(BOSS, MIDDLE.x, MIDDLE.y, MIDDLE.z)
        director:delay(0)
    elseif action_id == ACTION_SPOTS then
        if uses % 2 == 1 then
            talk(director, part_number == 1 and 33424 or 33427, 5000)
        end
        after(4.0, castSpots)
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
        castRow(director, 1, 5.0)
        castRow(director, 2, 9.0)
        castRow(director, 3, 13.0)
    elseif action_id == ACTION_RISE then
        after(3.0, function(d)
            d:set_health(BOSS, HEALTH_LAST)
        end)
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

-- His health at 1 the first time: the four come.
function bringAdds(director)
    part = "adds"
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
    after(7.0, function(d) talk(d, 33405, 5000, true) end)
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
end

-- The four down: he gets up, at full health.
function afterAdds(director)
    part = "between"
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
    director:restart_timeline(BOSS, PART_NONE)
    director:busy_bnpc(BOSS, 3600)
    director:root_bnpc(BOSS, 3600)
    director:set_invulnerable(BOSS, true)
    if boss ~= nil then
        director:bnpc_control(BOSS, CONTROL_CANCEL_CAST, { boss })
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

function onSceneFinished(director, scene)
    if scene ~= CUTSCENE_SCENE or part ~= "cutscene" then
        return
    end
    part = "rising"
    director:bnpc_control(BOSS, CONTROL_63, { 0 })
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
        local at = player_at or PLAYER_AFTER_CUTSCENE
        local helper = helperAt(d, { at.x, at.y, at.z }, PLAYER_AFTER_CUTSCENE.rotation)
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
    -- The pull: he has someone to go for, or has been hit.
    local left = health[BOSS]
    if part == 0 and (him.target ~= nil or (left ~= nil and left < 100)) then
        part = 1
        talk(director, 33400)
    end
    -- His health at its floor (1 HP; the simulator's health is a percentage, 0.1 at the least for one still up.)
    local at_floor = (him.hp ~= nil and him.hp <= 1) or (left ~= nil and left <= 0.1)
    if part == 1 and at_floor then
        bringAdds(director)
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
        if not first_add_fell then
            first_add_fell = true
            after(3.0, function(d) d:yell(BOSS, 13651) end)
        end
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

-- The player down: a duty's wipe sends them back to the start; everyone goes and comes back as at the beginning.
function onPartyWipe(director)
    director:despawn_bnpc(BOSS)
    for _, add in ipairs(ADDS) do
        director:despawn_bnpc(add)
    end
    for _, helper in ipairs(helpers) do
        director:despawn_bnpc(helper)
    end
    director:map_effect(0, MAP_EFFECT_OFF.timeline, MAP_EFFECT_OFF.state)
    reset()
    bring(director)
end
