-- Quest: The Price of Principles
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Connor's Chronofoil capture af05916c (2026-11-07/09) (fixture tools/questkit/fixtures/1175.json):
-- Sequence 0:
--   ACTOR0 (Minfilia): scene 0 (HIDE_HOTBAR, a yield), 1 (SET_BASE | HIDE_HOTBAR) -> accepted, sequence 1
-- Sequence 1:
--   ACTOR1 (Y'shtola): scene 2 (HIDE_HOTBAR), vars 11 00 00 00 00 80, notices [[0, []]]
--   ACTOR2 (Thancred): scene 3 (HIDE_HOTBAR), vars 21 10 00 00 00 c0, notices [[1, []]]
--   ACTOR3 (Papalymo): scene 4 (HIDE_HOTBAR), vars 31 11 00 00 00 e0, notices [[2, []]]
--   ACTOR4 (Yda): scene 5 (HIDE_HOTBAR), vars 41 11 10 00 00 f0, notices [[3, []]]
--   ACTOR5 (Urianger): scene 6 (HIDE_HOTBAR) -> sequence 255, vars 00 00 00 00 00 00, notices [[4, []]]
-- Sequence 255:
--   ACTOR6 (Alphinaud): scene 8 (SET_BASE | HIDE_HOTBAR, a yield) -> completed, zone 212
-- The 5 talks of sequence 1 work in any order: UI8AH counts them and each sets its BitFlag8 (retail's), the last moves the quest on.

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

-- BitFlag8: flag 1 is the top bit of UI8F, as retail sets it.
function hasFlag(player, flag)
    return (player:get_quest_var(EVENT_ID, 5) & (0x80 >> (flag - 1))) ~= 0
end

function setFlag(player, flag)
    player:set_quest_var(EVENT_ID, 5, player:get_quest_var(EVENT_ID, 5) | (0x80 >> (flag - 1)))
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == nil and BASE_ID == ACTOR0 then
        step = 1
        play(player, 0, HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == ACTOR1 and not hasFlag(player, 1) then
        step = 2
        play(player, 2, HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == ACTOR2 and not hasFlag(player, 2) then
        step = 3
        play(player, 3, HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == ACTOR3 and not hasFlag(player, 3) then
        step = 4
        play(player, 4, HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == ACTOR4 and not hasFlag(player, 4) then
        step = 5
        play(player, 5, HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == ACTOR5 and not hasFlag(player, 5) then
        step = 6
        play(player, 6, HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR6 then
        step = 7
        play(player, 8, SET_BASE | HIDE_HOTBAR)
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
    elseif step == 2 and scene == 2 then
        local count = getVar(player, 0, true) + 1
        player:event_notice(EVENT_ID, 0, {})
        if count >= 5 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x00)
            player:set_quest_var(EVENT_ID, 2, 0x00)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 255)
        else
            setVar(player, 0, true, count)
            setVar(player, 0, false, 1) -- UI8AL
            setFlag(player, 1)
        end
    elseif step == 3 and scene == 3 then
        local count = getVar(player, 0, true) + 1
        player:event_notice(EVENT_ID, 1, {})
        if count >= 5 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x00)
            player:set_quest_var(EVENT_ID, 2, 0x00)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 255)
        else
            setVar(player, 0, true, count)
            setVar(player, 1, true, 1) -- UI8BH
            setFlag(player, 2)
        end
    elseif step == 4 and scene == 4 then
        local count = getVar(player, 0, true) + 1
        player:event_notice(EVENT_ID, 2, {})
        if count >= 5 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x00)
            player:set_quest_var(EVENT_ID, 2, 0x00)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 255)
        else
            setVar(player, 0, true, count)
            setVar(player, 1, false, 1) -- UI8BL
            setFlag(player, 3)
        end
    elseif step == 5 and scene == 5 then
        local count = getVar(player, 0, true) + 1
        player:event_notice(EVENT_ID, 3, {})
        if count >= 5 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x00)
            player:set_quest_var(EVENT_ID, 2, 0x00)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 255)
        else
            setVar(player, 0, true, count)
            setVar(player, 2, true, 1) -- UI8CH
            setFlag(player, 4)
        end
    elseif step == 6 and scene == 6 then
        local count = getVar(player, 0, true) + 1
        player:event_notice(EVENT_ID, 4, {})
        if count >= 5 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x00)
            player:set_quest_var(EVENT_ID, 2, 0x00)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 255)
        else
            setVar(player, 0, true, count)
            setFlag(player, 5)  -- not seen on retail (its step was the last): the next free one
        end
    elseif step == 7 and scene == 8 then
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
        -- Where retail put the player (ActorSetPos; no pop range of the quest's.)
        player:change_territory(212, { x = 33.59, y = 1.2, z = -0.7 }, 1.57)
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
