-- Quest: The Mother of Exiles
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Connor's Chronofoil capture af05916c (2026-11-07/09) (fixture tools/questkit/fixtures/1347.json):
-- Sequence 0:
--   ACTOR0 (Thancred): scene 0 (HIDE_HOTBAR, a yield), 1 (HIDE_HOTBAR) -> accepted, sequence 1
-- Sequence 1:
--   ACTOR1 (Alphinaud): scene 2 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 2, notices [[0, []]]
-- Sequence 2:
--   walking into EVENTRANGE0: scene 4 (SET_BASE | HIDE_HOTBAR) -> sequence 3, notices [[1, []]]
-- Sequence 3:
--   ACTOR6 (Alphinaud): scene 9 (SET_BASE | HIDE_HOTBAR) -> sequence 4, notices [[2, []]]
-- Sequence 4:
--   ACTOR10 (Yugiri): scene 13 (HIDE_HOTBAR) -> sequence 5, notices [[3, []]]
-- Sequence 5:
--   ACTOR13 (Momodi): scene 16 (HIDE_HOTBAR) -> sequence 6, vars 00 01 10 00 00 00, notices [[4, [25918]]]
-- Sequence 6:
--   ACTOR15 (Katherine): scene 22 (HIDE_HOTBAR, a yield), 23 (HIDE_HOTBAR), vars 11 01 00 00 00 40, notices [[6, []]]
--   ACTOR14 (Fridurih): scene 20 (HIDE_HOTBAR, a yield), 21 (HIDE_HOTBAR) -> sequence 7, vars 00 00 00 00 00 00, notices [[5, []]]
-- Sequence 7:
--   ACTOR13 (Momodi): scene 27 (HIDE_HOTBAR) -> sequence 8, notices [[7, []]]
-- Sequence 8:
--   ACTOR10 (Yugiri): scene 31 (HIDE_HOTBAR), 32 (SET_BASE | HIDE_HOTBAR) -> sequence 255, notices [[8, []]]
-- Sequence 255:
--   ACTOR16 (Raubahn): scene 35 (HIDE_HOTBAR), 36 (SET_BASE | HIDE_HOTBAR), 37 (HIDE_HOTBAR, a yield) -> completed
-- The 2 talks of sequence 6 work in any order: UI8AH counts them and each sets its BitFlag8 (retail's), the last moves the quest on.

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
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        step = 2
        play(player, 2, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 3 and BASE_ID == ACTOR6 then
        step = 4
        play(player, 9, SET_BASE | HIDE_HOTBAR)
        return
    elseif sequence == 4 and BASE_ID == ACTOR10 then
        step = 5
        play(player, 13, HIDE_HOTBAR)
        return
    elseif sequence == 5 and BASE_ID == ACTOR13 then
        step = 6
        play(player, 16, HIDE_HOTBAR)
        return
    elseif sequence == 6 and BASE_ID == ACTOR15 and not hasFlag(player, 2) then
        step = 7
        play(player, 22, HIDE_HOTBAR)
        return
    elseif sequence == 6 and BASE_ID == ACTOR14 and not hasFlag(player, 1) then
        step = 8
        play(player, 20, HIDE_HOTBAR)
        return
    elseif sequence == 7 and BASE_ID == ACTOR13 then
        step = 9
        play(player, 27, HIDE_HOTBAR)
        return
    elseif sequence == 8 and BASE_ID == ACTOR10 then
        step = 10
        play(player, 31, HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR16 then
        step = 11
        play(player, 35, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEnterTrigger(player, arg)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == 2 and arg == EVENTRANGE0 then
        step = 3
        play(player, 4, SET_BASE | HIDE_HOTBAR)
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
        player:quest_sequence(EVENT_ID, 1)
    elseif step == 2 and scene == 2 then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif step == 3 and scene == 4 then
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif step == 4 and scene == 9 then
        player:event_notice(EVENT_ID, 2, {})
        player:quest_sequence(EVENT_ID, 4)
    elseif step == 5 and scene == 13 then
        player:event_notice(EVENT_ID, 3, {})
        player:quest_sequence(EVENT_ID, 5)
    elseif step == 6 and scene == 16 then
        player:event_notice(EVENT_ID, 4, { 25918 })
        player:set_quest_var(EVENT_ID, 1, 0x01) -- UI8B
        player:set_quest_var(EVENT_ID, 2, 0x10) -- UI8C
        player:quest_sequence(EVENT_ID, 6)
    elseif step == 7 and scene == 22 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        play(player, 23, HIDE_HOTBAR)
        return
    elseif step == 7 and scene == 23 then
        local count = getVar(player, 0, true) + 1
        player:event_notice(EVENT_ID, 6, {})
        if count >= 2 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x00)
            player:set_quest_var(EVENT_ID, 2, 0x00)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 7)
        else
            setVar(player, 0, true, count)
            setVar(player, 0, false, 1) -- UI8AL
            setVar(player, 2, true, 0) -- UI8CH
            setFlag(player, 2)
        end
    elseif step == 8 and scene == 20 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        play(player, 21, HIDE_HOTBAR)
        return
    elseif step == 8 and scene == 21 then
        local count = getVar(player, 0, true) + 1
        player:event_notice(EVENT_ID, 5, {})
        if count >= 2 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x00)
            player:set_quest_var(EVENT_ID, 2, 0x00)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 7)
        else
            setVar(player, 0, true, count)
            setFlag(player, 1)  -- not seen on retail (its step was the last): the next free one
        end
    elseif step == 9 and scene == 27 then
        player:event_notice(EVENT_ID, 7, {})
        player:quest_sequence(EVENT_ID, 8)
    elseif step == 10 and scene == 31 then
        play(player, 32, SET_BASE | HIDE_HOTBAR)
        return
    elseif step == 10 and scene == 32 then
        player:event_notice(EVENT_ID, 8, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif step == 11 and scene == 35 then
        play(player, 36, SET_BASE | HIDE_HOTBAR)
        return
    elseif step == 11 and scene == 36 then
        play(player, 37, HIDE_HOTBAR)
        return
    elseif step == 11 and scene == 37 then
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
