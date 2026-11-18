-- Quest: Parsemontrenomics
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Bob's Chronofoil capture dba9b575 of 2026-11-18 (fixture tools/questkit/fixtures/38.json):
-- Sequence 0:
--   ACTOR0 (Parsemontret): scene 0 (HIDE_HOTBAR, a yield) -> accepted, sequence 1
-- Sequence 1:
--   ACTOR3 (Alaric): scene 3 (HIDE_HOTBAR), vars 10 01 00 10 00 20, notices [[2, [26103]]]
--   ACTOR2 (Admiranda): scene 2 (HIDE_HOTBAR), vars 20 11 01 10 00 60, notices [[1, [22636]]]
--   ACTOR1 (Gurthcid): scene 1 (HIDE_HOTBAR) -> sequence 255, vars 00 11 10 00 00 00, notices [[0, [22303]]]
-- Sequence 255:
--   ACTOR4 (Ceinguled): scene 4 (HIDE_HOTBAR), 100 (HIDE_HOTBAR, a yield) -> completed
-- The 3 talks of sequence 1 work in any order: UI8AH counts them and each sets its BitFlag8 (retail's), the last moves the quest on.

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
    elseif sequence == 1 and BASE_ID == ACTOR3 and not hasFlag(player, 3) then
        step = 2
        play(player, 3, HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == ACTOR2 and not hasFlag(player, 2) then
        step = 3
        play(player, 2, HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == ACTOR1 and not hasFlag(player, 1) then
        step = 4
        play(player, 1, HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR4 then
        step = 5
        play(player, 4, HIDE_HOTBAR)
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
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif step == 2 and scene == 3 then
        local count = getVar(player, 0, true) + 1
        player:event_notice(EVENT_ID, 2, { 26103 })
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
            setVar(player, 3, true, 1) -- UI8DH
            setFlag(player, 3)
        end
    elseif step == 3 and scene == 2 then
        local count = getVar(player, 0, true) + 1
        player:event_notice(EVENT_ID, 1, { 22636 })
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
            setVar(player, 1, true, 1) -- UI8BH
            setVar(player, 2, false, 1) -- UI8CL
            setFlag(player, 2)
        end
    elseif step == 4 and scene == 1 then
        local count = getVar(player, 0, true) + 1
        player:event_notice(EVENT_ID, 0, { 22303 })
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
            setFlag(player, 1)  -- not seen on retail (its step was the last): the next free one
        end
    elseif step == 5 and scene == 4 then
        play(player, 100, HIDE_HOTBAR)
        return
    elseif step == 5 and scene == 100 then
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
