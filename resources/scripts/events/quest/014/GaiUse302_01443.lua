-- Quest: Desperate Times
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Connor's Chronofoil capture af05916c (2026-11-07/09) (fixture tools/questkit/fixtures/1443.json):
-- Sequence 0:
--   ACTOR0 (Alphinaud): scene 0 (NO_DEFAULT_CAMERA | HIDE_HOTBAR, a yield), 1 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> accepted, sequence 1
-- Sequence 1:
--   ACTOR1 (Raubahn): scene 2 (SET_BASE | HIDE_HOTBAR) -> sequence 2, notices [[0, []]]
-- Sequence 2:
--   ACTOR2 (Swift): scene 3 (HIDE_HOTBAR) -> sequence 3, notices [[1, []]]
-- Sequence 3:
--   ACTOR3 (Amalberga): scene 4 (HIDE_HOTBAR), vars 01 00 00 00 00 80, notices [[2, [1, 2]]]
--   ACTOR4 (Leofric): scene 6 (HIDE_HOTBAR) -> sequence 4, vars 00 00 00 00 00 00, notices [[2, [2, 2]]]
-- Sequence 4:
--   ACTOR5 (Zazawaka): scene 9 (HIDE_HOTBAR) -> sequence 5, notices [[3, []]]
-- Sequence 5:
--   ACTOR6 (terrified refugee): scene 13 (HIDE_HOTBAR) -> sequence 255, notices [[4, []]]
-- Sequence 255:
--   emote 35 on ACTOR6 (terrified refugee): scene 16 (NO_DEFAULT_CAMERA | HIDE_HOTBAR, a yield) -> completed
-- The 2 talks of sequence 3 work in any order: UI8AL counts them and each sets its BitFlag8 (retail's), the last moves the quest on.

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
        play(player, 0, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        step = 2
        play(player, 2, SET_BASE | HIDE_HOTBAR)
        return
    elseif sequence == 2 and BASE_ID == ACTOR2 then
        step = 3
        play(player, 3, HIDE_HOTBAR)
        return
    elseif sequence == 3 and BASE_ID == ACTOR3 and not hasFlag(player, 1) then
        step = 4
        play(player, 4, HIDE_HOTBAR)
        return
    elseif sequence == 3 and BASE_ID == ACTOR4 and not hasFlag(player, 2) then
        step = 5
        play(player, 6, HIDE_HOTBAR)
        return
    elseif sequence == 4 and BASE_ID == ACTOR5 then
        step = 6
        play(player, 9, HIDE_HOTBAR)
        return
    elseif sequence == 5 and BASE_ID == ACTOR6 then
        step = 7
        play(player, 13, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEmote(emote, target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == 255 and emote == 35 and BASE_ID == ACTOR6 then
        step = 8
        play(player, 16, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
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
        play(player, 1, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif step == 1 and scene == 1 then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif step == 2 and scene == 2 then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif step == 3 and scene == 3 then
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif step == 4 and scene == 4 then
        local count = getVar(player, 0, false) + 1
        player:event_notice(EVENT_ID, 2, { count, 2 })
        if count >= 2 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x00)
            player:set_quest_var(EVENT_ID, 2, 0x00)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 4)
        else
            setVar(player, 0, false, count)
            setFlag(player, 1)
        end
    elseif step == 5 and scene == 6 then
        local count = getVar(player, 0, false) + 1
        player:event_notice(EVENT_ID, 2, { count, 2 })
        if count >= 2 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x00)
            player:set_quest_var(EVENT_ID, 2, 0x00)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 4)
        else
            setVar(player, 0, false, count)
            setFlag(player, 2)  -- not seen on retail (its step was the last): the next free one
        end
    elseif step == 6 and scene == 9 then
        player:event_notice(EVENT_ID, 3, {})
        player:quest_sequence(EVENT_ID, 5)
    elseif step == 7 and scene == 13 then
        player:event_notice(EVENT_ID, 4, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif step == 8 and scene == 16 then
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
