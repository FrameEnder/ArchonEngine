-- Quest: The Gifted
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Connor's Chronofoil capture af05916c (2026-11-07/09) (fixture tools/questkit/fixtures/1192.json):
-- Sequence 0:
--   ACTOR0 (Minfilia): scene 0 (HIDE_HOTBAR, a yield), 1 (HIDE_HOTBAR) -> accepted, sequence 1
-- Sequence 1:
--   ACTOR1 (Urianger): scene 2 (HIDE_HOTBAR), 3 (SET_BASE | HIDE_HOTBAR) -> sequence 2, zone 212
-- Sequence 2:
--   walking into EVENTRANGE0: scene 4 (SET_BASE | INVIS_ENPC | HIDE_HOTBAR, a yield) -> sequence 3, notices [[1, []]]
-- Sequence 3:
--   walking into EVENTRANGE1: scene 9 (NO_DEFAULT_CAMERA | HIDE_HOTBAR)
--   defeating 2477, vars 01 00 00 00 00 00, notices [[2, [1, 3]]]
--   defeating 2477, vars 02 00 00 00 00 00, notices [[2, [2, 3]]]
--   defeating 2477 -> sequence 4, vars 00 00 00 00 00 00, notices [[2, [3, 3]]]
-- Sequence 4:
--   walking into EVENTRANGE2: scene 14 (NO_DEFAULT_CAMERA | HIDE_HOTBAR)
--   defeating 2478 -> sequence 5, notices [[3, []]]
-- Sequence 5:
--   walking into EVENTRANGE3: scene 19 (NO_DEFAULT_CAMERA | HIDE_HOTBAR)
--   defeating 2479, vars 01 00 00 00 00 00, notices [[4, [1, 2]]]
--   defeating 2479 -> sequence 6, vars 00 00 00 00 00 00, notices [[4, [2, 2]]]
-- Sequence 6:
--   walking into EVENTRANGE4: scene 24 (NO_DEFAULT_CAMERA | HIDE_HOTBAR)
--   defeating 2481, vars 01 00 00 00 00 00, notices [[5, [1, 4]]]
--   defeating 2481, vars 02 00 00 00 00 00, notices [[5, [2, 4]]]
--   defeating 2480, vars 03 00 00 00 00 00, notices [[5, [3, 4]]]
--   defeating 2480 -> sequence 7, vars 00 00 00 00 00 00, notices [[5, [4, 4]]]
-- Sequence 7:
--   ACTOR4 (white-robed Ascian): scene 30 (SET_BASE | HIDE_HOTBAR) -> sequence 255, notices [[6, []]]
-- Sequence 255:
--   ACTOR0 (Minfilia): scene 34 (HIDE_HOTBAR, a yield) -> completed
-- The 3 enemies of sequence 3 count in any order (UI8AL).
-- The 2 enemies of sequence 5 count in any order (UI8AL).
-- The 4 enemies of sequence 6 count in any order (UI8AL).

-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855, 960.)
REWARD_CLOSED = 0x11

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- Which step of the quest this event is (the sequence notes above), the scene after a progress bar, and the reward
-- the player picked (the scenes after it show it.)
step = nil
after_bar = nil
chosen = 0

function play(player, scene, flags, params)
    current_scene = scene
    player:play_scene(scene, flags, params or {})
end

-- Quest variables: UI8A..F are bytes 0..5, H is the high nibble and L the low one.
function getVar(player, byte, high)
    local value = player:get_quest_var(EVENT_ID, byte)
    if high then
        return value >> 4
    end
    return value & 0x0F
end

function setVar(player, byte, high, nibble)
    local value = player:get_quest_var(EVENT_ID, byte)
    if high then
        value = (value & 0x0F) | (nibble << 4)
    else
        value = (value & 0xF0) | nibble
    end
    player:set_quest_var(EVENT_ID, byte, value)
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == nil and BASE_ID == ACTOR0 then
        step = 1
        play(player, 0, HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        step = 2
        play(player, 2, HIDE_HOTBAR)
        return
    elseif sequence == 7 and BASE_ID == ACTOR4 then
        step = 18
        play(player, 30, SET_BASE | HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        step = 19
        play(player, 34, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEnterTrigger(player, arg)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == 2 and arg == EVENTRANGE0 then
        step = 3
        play(player, 4, SET_BASE | INVIS_ENPC | HIDE_HOTBAR)
        return
    elseif sequence == 3 and arg == EVENTRANGE1 then
        step = 4
        play(player, 9, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 4 and arg == EVENTRANGE2 then
        step = 8
        play(player, 14, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 5 and arg == EVENTRANGE3 then
        step = 10
        play(player, 19, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 6 and arg == EVENTRANGE4 then
        step = 13
        play(player, 24, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

-- The enemies of sequence 3, in any order: UI8AL counts them.
KILLS_3 = { [ENEMY1] = true, [ENEMY2] = true, [ENEMY0] = true }

-- The enemies of sequence 5, in any order: UI8AL counts them.
KILLS_5 = { [ENEMY5] = true, [ENEMY4] = true }

-- The enemies of sequence 6, in any order: UI8AL counts them.
KILLS_6 = { [ENEMY8] = true, [ENEMY9] = true, [ENEMY7] = true, [ENEMY6] = true }

function onBNpcKill(base_id, layout_id, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    if sequence == 3 and KILLS_3[layout_id] then
        local count = getVar(player, 0, false) + 1
        if count >= 3 then
            player:event_notice(EVENT_ID, 2, { 3, 3 })
            player:set_quest_var(EVENT_ID, 0, 0x00) -- UI8A
            player:quest_sequence(EVENT_ID, 4)
        else
            setVar(player, 0, false, count)
            player:event_notice(EVENT_ID, 2, { count, 3 })
        end
    elseif sequence == 5 and KILLS_5[layout_id] then
        local count = getVar(player, 0, false) + 1
        if count >= 2 then
            player:event_notice(EVENT_ID, 4, { 2, 2 })
            player:set_quest_var(EVENT_ID, 0, 0x00) -- UI8A
            player:quest_sequence(EVENT_ID, 6)
        else
            setVar(player, 0, false, count)
            player:event_notice(EVENT_ID, 4, { count, 2 })
        end
    elseif sequence == 6 and KILLS_6[layout_id] then
        local count = getVar(player, 0, false) + 1
        if count >= 4 then
            player:event_notice(EVENT_ID, 5, { 4, 4 })
            player:set_quest_var(EVENT_ID, 0, 0x00) -- UI8A
            player:quest_sequence(EVENT_ID, 7)
        else
            setVar(player, 0, false, count)
            player:event_notice(EVENT_ID, 5, { count, 4 })
        end
    elseif sequence == 4 and layout_id == ENEMY3 then
        player:event_notice(EVENT_ID, 3, {})
        player:quest_sequence(EVENT_ID, 5)
    end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if step == 1 and scene == 0 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        play(player, 1, HIDE_HOTBAR)
        return
    elseif step == 1 and scene == 1 then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif step == 2 and scene == 2 then
        play(player, 3, SET_BASE | HIDE_HOTBAR)
        return
    elseif step == 2 and scene == 3 then
        player:quest_sequence(EVENT_ID, 2)
        player:finish_event()
        -- Where retail put the player (ActorSetPos; no pop range of the quest's.)
        player:change_territory(212, { x = 33.12, y = 1.2, z = -0.72 }, 1.571)
        return
    elseif step == 3 and scene == 4 then
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif step == 4 and scene == 9 then
        player:spawn_quest_bnpc(ENEMY2)
        player:spawn_quest_bnpc(ENEMY1)
        player:spawn_quest_bnpc(ENEMY0)
    elseif step == 8 and scene == 14 then
        player:spawn_quest_bnpc(ENEMY3)
    elseif step == 10 and scene == 19 then
        player:spawn_quest_bnpc(ENEMY5)
        player:spawn_quest_bnpc(ENEMY4)
    elseif step == 13 and scene == 24 then
        player:spawn_quest_bnpc(ENEMY9)
        player:spawn_quest_bnpc(ENEMY8)
        player:spawn_quest_bnpc(ENEMY7)
        player:spawn_quest_bnpc(ENEMY6)
    elseif step == 18 and scene == 30 then
        player:event_notice(EVENT_ID, 6, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif step == 19 and scene == 34 then
        if results[1] ~= 1 then
            -- The reward window closed: the quest stays open.
            if player:get_quest_var(EVENT_ID, 0) ~= REWARD_CLOSED then
                player:set_quest_var(EVENT_ID, 0, REWARD_CLOSED)
            end
            player:finish_event()
            return
        end
        chosen = results[2] or 0
        player:finish_event()
        player:finish_quest(EVENT_ID, chosen)
        return
    end

    player:finish_event()
end

function onYield(scene, id, results, player)
    sceneFinished(scene, results, player)
end

function onReturn(scene, results, player)
    sceneFinished(scene, results, player)
end
