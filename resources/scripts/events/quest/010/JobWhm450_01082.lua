-- Quest: Following in His Footsteps
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Bob's Chronofoil captures 898dd0b9 and d1c46dff (2026-11-12) (fixture tools/questkit/fixtures/1082.json):
-- Sequence 0:
--   ACTOR0 (Raya-O-Senna): scene 0 (HIDE_HOTBAR, a yield), 1 (HIDE_HOTBAR) -> accepted, sequence 1, vars 00 00 11 10 00 00
-- Sequence 1:
--   emote 58 on EOBJECT0 (epitaph to philosophy): scene 3 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), vars 11 00 01 10 00 80, notices [[0, [48242]]]
--   emote 58 on EOBJECT1 (epitaph to discovery): scene 5 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), vars 21 10 01 00 00 c0, notices [[1, [46219]]]
--   emote 58 on EOBJECT2 (epitaph to fluidity): scene 7 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 2, vars 00 00 00 00 00 00, notices [[2, [45189]]]
-- Sequence 2:
--   ACTOR0 (Raya-O-Senna): scene 8 (HIDE_HOTBAR) -> sequence 3, vars 00 10 00 00 00 00, notices [[3, [21611]]]
-- Sequence 3:
--   EOBJECT3 (epitaph to infancy): scene 9 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 4, notices [[4, []]]
-- Sequence 4:
--   ACTOR0 (Raya-O-Senna): scene 14 (HIDE_HOTBAR, a yield), 15 (SET_BASE | HIDE_HOTBAR) -> sequence 5, notices [[5, [21611]]]
-- Sequence 5:
--   emote 58 on EOBJECT6 (epitaph to expiry): scene 21 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 255, vars 00 00 00 00 00 00, notices [[6, [40616]]]
-- Sequence 255:
--   ACTOR0 (Raya-O-Senna): scene 22 (HIDE_HOTBAR, a yield), 23 (HIDE_HOTBAR, a yield) -> completed
-- The 3 emotes of sequence 1 work in any order: UI8AH counts them and each sets its BitFlag8 (retail's), the last moves the quest on.

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
    elseif sequence == 2 and BASE_ID == ACTOR0 then
        step = 5
        play(player, 8, HIDE_HOTBAR)
        return
    elseif sequence == 3 and BASE_ID == EOBJECT3 then
        step = 6
        play(player, 9, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 4 and BASE_ID == ACTOR0 then
        step = 7
        play(player, 14, HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        step = 9
        play(player, 22, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEmote(emote, target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == 1 and emote == 58 and BASE_ID == EOBJECT0 and not hasFlag(player, 1) then
        step = 2
        play(player, 3, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 1 and emote == 58 and BASE_ID == EOBJECT1 and not hasFlag(player, 2) then
        step = 3
        play(player, 5, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 1 and emote == 58 and BASE_ID == EOBJECT2 and not hasFlag(player, 3) then
        step = 4
        play(player, 7, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 5 and emote == 58 and BASE_ID == EOBJECT6 then
        step = 8
        play(player, 21, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
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
        play(player, 1, HIDE_HOTBAR)
        return
    elseif step == 1 and scene == 1 then
        player:accept_quest(EVENT_ID)
        player:set_quest_var(EVENT_ID, 2, 0x11) -- UI8C
        player:set_quest_var(EVENT_ID, 3, 0x10) -- UI8D
        player:quest_sequence(EVENT_ID, 1)
    elseif step == 2 and scene == 3 then
        local count = getVar(player, 0, true) + 1
        player:event_notice(EVENT_ID, 0, { 48242 })
        if count >= 3 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x00)
            player:set_quest_var(EVENT_ID, 2, 0x00)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 2)
        else
            setVar(player, 0, true, count)
            setVar(player, 0, false, 1) -- UI8AL
            setVar(player, 2, true, 0) -- UI8CH
            setFlag(player, 1)
        end
    elseif step == 3 and scene == 5 then
        local count = getVar(player, 0, true) + 1
        player:event_notice(EVENT_ID, 1, { 46219 })
        if count >= 3 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x00)
            player:set_quest_var(EVENT_ID, 2, 0x00)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 2)
        else
            setVar(player, 0, true, count)
            setVar(player, 1, true, 1) -- UI8BH
            setVar(player, 3, true, 0) -- UI8DH
            setFlag(player, 2)
        end
    elseif step == 4 and scene == 7 then
        local count = getVar(player, 0, true) + 1
        player:event_notice(EVENT_ID, 2, { 45189 })
        if count >= 3 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x00)
            player:set_quest_var(EVENT_ID, 2, 0x00)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 2)
        else
            setVar(player, 0, true, count)
            setFlag(player, 3)  -- not seen on retail (its step was the last): the next free one
        end
    elseif step == 5 and scene == 8 then
        player:event_notice(EVENT_ID, 3, { 21611 })
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8B
        player:quest_sequence(EVENT_ID, 3)
    elseif step == 6 and scene == 9 then
        player:event_notice(EVENT_ID, 4, {})
        player:quest_sequence(EVENT_ID, 4)
    elseif step == 7 and scene == 14 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        play(player, 15, SET_BASE | HIDE_HOTBAR)
        return
    elseif step == 7 and scene == 15 then
        player:event_notice(EVENT_ID, 5, { 21611 })
        player:quest_sequence(EVENT_ID, 5)
    elseif step == 8 and scene == 21 then
        player:event_notice(EVENT_ID, 6, { 40616 })
        player:set_quest_var(EVENT_ID, 1, 0x00) -- UI8B
        player:quest_sequence(EVENT_ID, 255)
    elseif step == 9 and scene == 22 then
        if results[1] ~= 1 then
            -- The reward window closed: the quest stays open.
            if player:get_quest_var(EVENT_ID, 0) ~= REWARD_CLOSED then
                player:set_quest_var(EVENT_ID, 0, REWARD_CLOSED)
            end
            player:finish_event()
            return
        end
        chosen = results[2] or 0
        play(player, 23, HIDE_HOTBAR, { chosen })
        return
    elseif step == 9 and scene == 23 then
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
