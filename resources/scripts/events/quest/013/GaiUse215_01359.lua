-- Quest: The Gift of Eternity
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Connor's Chronofoil capture af05916c (2026-11-07/09) (fixture tools/questkit/fixtures/1359.json):
-- Sequence 0:
--   ACTOR0 (Falkbryda): scene 0 (HIDE_HOTBAR, a yield), 1 (SET_BASE | HIDE_HOTBAR) -> accepted, sequence 1
-- Sequence 1:
--   walking into EVENTRANGE1: scene 5 (NO_DEFAULT_CAMERA | HIDE_HOTBAR)
--   defeating 2890, vars 01 00 00 00 00 00
--   defeating 2891 -> sequence 2, vars 00 00 00 00 00 00, notices [[0, [1, 1]]]
-- Sequence 2:
--   walking into EVENTRANGE3: scene 20 (NO_DEFAULT_CAMERA | HIDE_HOTBAR)
--   defeating 2892, vars 01 00 00 00 00 00
--   defeating 2893, vars 02 00 00 00 00 00
--   defeating 2894, vars 03 00 00 00 00 00
--   defeating 2895 -> sequence 3, vars 00 00 00 00 00 00, notices [[1, [1, 1]]]
-- Sequence 3:
--   walking into EVENTRANGE4: scene 32 (NO_DEFAULT_CAMERA | HIDE_HOTBAR)
--   defeating 2898, vars 01 00 00 00 00 00
--   defeating 2896, vars 02 00 00 00 00 00
--   defeating 2897 -> sequence 4, vars 00 00 00 00 00 00, notices [[2, []]]
-- Sequence 4:
--   ACTOR6 (Y'shtola): scene 42 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_RESCUE_UPPER_MIDDLE), 43 (NO_DEFAULT_CAMERA | HIDE_HOTBAR | INVINCIBLE), 44 (SET_BASE | HIDE_HOTBAR | INVINCIBLE) -> sequence 255, zone 138
-- Sequence 255:
--   ACTOR7 (Merlwyb): scene 50 (HIDE_HOTBAR, a yield) -> completed
-- The 2 enemies of sequence 1 count in any order (UI8AL).
-- The 4 enemies of sequence 2 count in any order (UI8AL).
-- The 3 enemies of sequence 3 count in any order (UI8AL).

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
    elseif sequence == 4 and BASE_ID == ACTOR6 then
        step = 14
        play(player, 42, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR7 then
        step = 15
        play(player, 50, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEnterTrigger(player, arg)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == 1 and arg == EVENTRANGE1 then
        step = 2
        play(player, 5, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 2 and arg == EVENTRANGE3 then
        step = 5
        play(player, 20, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 3 and arg == EVENTRANGE4 then
        step = 10
        play(player, 32, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

-- The enemies of sequence 1, in any order: UI8AL counts them.
KILLS_1 = { [ENEMY2] = true, [ENEMY3] = true }

-- The enemies of sequence 2, in any order: UI8AL counts them.
KILLS_2 = { [ENEMY8] = true, [ENEMY9] = true, [ENEMY10] = true, [ENEMY11] = true }

-- The enemies of sequence 3, in any order: UI8AL counts them.
KILLS_3 = { [ENEMY14] = true, [ENEMY12] = true, [ENEMY13] = true }

function onBNpcKill(base_id, layout_id, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    if sequence == 1 and KILLS_1[layout_id] then
        local count = getVar(player, 0, false) + 1
        if count >= 2 then
            player:event_notice(EVENT_ID, 0, { 1, 1 })
            player:set_quest_var(EVENT_ID, 0, 0x00) -- UI8A
            player:quest_sequence(EVENT_ID, 2)
        else
            setVar(player, 0, false, count)
        end
    elseif sequence == 2 and KILLS_2[layout_id] then
        local count = getVar(player, 0, false) + 1
        if count >= 4 then
            player:event_notice(EVENT_ID, 1, { 1, 1 })
            player:set_quest_var(EVENT_ID, 0, 0x00) -- UI8A
            player:quest_sequence(EVENT_ID, 3)
        else
            setVar(player, 0, false, count)
        end
    elseif sequence == 3 and KILLS_3[layout_id] then
        local count = getVar(player, 0, false) + 1
        if count >= 3 then
            player:event_notice(EVENT_ID, 2, {})
            player:set_quest_var(EVENT_ID, 0, 0x00) -- UI8A
            player:quest_sequence(EVENT_ID, 4)
        else
            setVar(player, 0, false, count)
        end
    end
end

function onEventActionCast(player)
    if after_bar ~= nil then
        local scene = after_bar
        after_bar = nil
        play(player, scene.scene, scene.flags)
        return
    end
    player:finish_event()
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
        play(player, 1, SET_BASE | HIDE_HOTBAR)
        return
    elseif step == 1 and scene == 1 then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif step == 2 and scene == 5 then
        player:spawn_quest_bnpc(ENEMY3)
        player:spawn_quest_bnpc(ENEMY2)
    elseif step == 5 and scene == 20 then
        player:spawn_quest_bnpc(ENEMY9)
        player:spawn_quest_bnpc(ENEMY8)
    elseif step == 10 and scene == 32 then
        player:spawn_quest_bnpc(ENEMY14)
        player:spawn_quest_bnpc(ENEMY13)
        player:spawn_quest_bnpc(ENEMY12)
    elseif step == 14 and scene == 42 then
        after_bar = { scene = 43, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR | INVINCIBLE }
        player:event_action(EVENT_ACTION_RESCUE_UPPER_MIDDLE)
        return
    elseif step == 14 and scene == 43 then
        play(player, 44, SET_BASE | HIDE_HOTBAR | INVINCIBLE)
        return
    elseif step == 14 and scene == 44 then
        player:quest_sequence(EVENT_ID, 255)
        player:finish_event()
        -- Where retail put the player (ActorSetPos; no pop range of the quest's.)
        player:change_territory(138, { x = -422.78, y = -35.71, z = -397.74 }, 2.075)
        return
    elseif step == 15 and scene == 50 then
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
