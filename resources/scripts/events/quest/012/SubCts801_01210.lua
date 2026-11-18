-- Quest: Beauty Is Only Scalp Deep
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Bob's Chronofoil capture dba9b575 of 2026-11-18 (fixture tools/questkit/fixtures/1210.json):
-- Sequence 0:
--   ACTOR1 (disgruntled madam): scene 2 (HIDE_HOTBAR) -> accepted, sequence 2, notices [[0, []]]
-- Sequence 2:
--   ACTOR2 (flustered man): scene 4 (HIDE_HOTBAR)
--   emote 12 on ACTOR2 (flustered man): scene 5 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 3, notices [[1, []]]
-- Sequence 3:
--   ACTOR3 (H'naanza): scene 8 (HIDE_HOTBAR), vars 10 01 10 00 00 80, notices [[2, [25919]]]
--   ACTOR4 (Beatin): scene 9 (HIDE_HOTBAR), vars 21 01 11 00 00 c0, notices [[3, [25919]]]
--   ACTOR5 (Severian): scene 10 (HIDE_HOTBAR) -> sequence 255, vars 00 11 10 00 00 00, notices [[4, [25919]]]
-- Sequence 255:
--   ACTOR2 (flustered man): scene 13 (HIDE_HOTBAR, a yield), 14 (SET_BASE | HIDE_HOTBAR, a yield) -> completed
-- The 3 talks of sequence 3 work in any order: UI8AH counts them and each sets its BitFlag8 (retail's), the last moves the quest on.

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
    if sequence == nil and BASE_ID == ACTOR1 then
        step = 1
        play(player, 2, HIDE_HOTBAR)
        return
    elseif sequence == 2 and BASE_ID == ACTOR2 then
        step = 2
        play(player, 4, HIDE_HOTBAR)
        return
    elseif sequence == 3 and BASE_ID == ACTOR3 and not hasFlag(player, 1) then
        step = 4
        play(player, 8, HIDE_HOTBAR)
        return
    elseif sequence == 3 and BASE_ID == ACTOR4 and not hasFlag(player, 2) then
        step = 5
        play(player, 9, HIDE_HOTBAR)
        return
    elseif sequence == 3 and BASE_ID == ACTOR5 and not hasFlag(player, 3) then
        step = 6
        play(player, 10, HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        step = 7
        play(player, 13, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEmote(emote, target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == 2 and emote == 12 and BASE_ID == ACTOR2 then
        step = 3
        play(player, 5, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if step == 1 and scene == 2 then
        player:accept_quest(EVENT_ID)
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif step == 2 and scene == 4 then
    elseif step == 3 and scene == 5 then
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif step == 4 and scene == 8 then
        local count = getVar(player, 0, true) + 1
        player:event_notice(EVENT_ID, 2, { 25919 })
        if count >= 3 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x11)
            player:set_quest_var(EVENT_ID, 2, 0x10)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 255)
        else
            setVar(player, 0, true, count)
            setVar(player, 1, false, 1) -- UI8BL
            setVar(player, 2, true, 1) -- UI8CH
            setFlag(player, 1)
        end
    elseif step == 5 and scene == 9 then
        local count = getVar(player, 0, true) + 1
        player:event_notice(EVENT_ID, 3, { 25919 })
        if count >= 3 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x11)
            player:set_quest_var(EVENT_ID, 2, 0x10)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 255)
        else
            setVar(player, 0, true, count)
            setVar(player, 0, false, 1) -- UI8AL
            setVar(player, 2, false, 1) -- UI8CL
            setFlag(player, 2)
        end
    elseif step == 6 and scene == 10 then
        local count = getVar(player, 0, true) + 1
        player:event_notice(EVENT_ID, 4, { 25919 })
        if count >= 3 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x11)
            player:set_quest_var(EVENT_ID, 2, 0x10)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 255)
        else
            setVar(player, 0, true, count)
            setFlag(player, 3)  -- not seen on retail (its step was the last): the next free one
        end
    elseif step == 7 and scene == 13 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        play(player, 14, SET_BASE | HIDE_HOTBAR)
        return
    elseif step == 7 and scene == 14 then
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
