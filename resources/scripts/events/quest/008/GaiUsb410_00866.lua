-- Quest: The Monster of Bronze Lake
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and scenes and effects from Bob's Chronofoil capture of 2026-11-16 (fixture tools/questkit/fixtures/866.json):
-- Sequence 0:
--   ACTOR0 (Rukusa Farusa): scene 0 (HIDE_HOTBAR, a yield), 1 (HIDE_HOTBAR) -> accepted, sequence 1
-- Sequence 1:
--   defeating ENEMY0, vars 00 10 00 00 00 00, notices [[0, [1, 4, 22603]]]
--   defeating ENEMY0, vars 00 20 00 00 00 00, notices [[0, [2, 4, 22603]]]
--   defeating ENEMY0, vars 00 30 00 00 00 00, notices [[0, [3, 4, 22603]]]
--   defeating ENEMY0 -> sequence 2, vars 00 40 00 00 00 00, notices [[0, [4, 4, 22603]]]
-- Sequence 2:
--   ACTOR1 (Godbert): scene 2 (HIDE_HOTBAR, a yield), 3 (HIDE_HOTBAR) -> sequence 3, notices [[1, []]]
-- Sequence 3:
--   key item 2000673 on ACTOR1 (Godbert): scene 5 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 4, vars 00 20 00 00 00 00, notices [[2, []]]
-- Sequence 4:
--   key item 2000673 on ACTOR1 (Godbert): scene 7 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 5, vars 00 00 00 00 00 00, notices [[3, []]]
-- Sequence 5:
--   ACTOR1 (Godbert): scene 8 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_SEARCH), 9 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 6, notices [[4, []]]
-- Sequence 6:
--   ACTOR1 (Godbert): scene 10 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_PROCESS_UPPER_MIDDLE), 11 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 255, notices [[5, []]]
-- Sequence 255:
--   ACTOR0 (Rukusa Farusa): scene 12 (HIDE_HOTBAR, a yield) -> completed
-- The 4 enemies of sequence 1 count in any order (UI8BH).

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
    elseif sequence == 2 and BASE_ID == ACTOR1 then
        step = 6
        play(player, 2, HIDE_HOTBAR)
        return
    elseif sequence == 5 and BASE_ID == ACTOR1 then
        step = 9
        play(player, 8, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 6 and BASE_ID == ACTOR1 then
        step = 10
        play(player, 10, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        step = 11
        play(player, 12, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEventItem(item, target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == 3 and item == ITEM0 and BASE_ID == ACTOR1 then
        step = 7
        play(player, 5, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 4 and item == ITEM0 and BASE_ID == ACTOR1 then
        step = 8
        play(player, 7, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

-- The enemies of sequence 1, in any order: UI8BH counts them.
KILLS_1 = { [ENEMY0] = true }

function onBNpcKill(base_id, layout_id, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    if sequence == 1 and KILLS_1[base_id] then
        local count = getVar(player, 1, true) + 1
        if count >= 4 then
            player:event_notice(EVENT_ID, 0, { 4, 4, 22603 })
            player:set_quest_var(EVENT_ID, 1, 0x40) -- UI8B
            player:quest_sequence(EVENT_ID, 2)
        else
            setVar(player, 1, true, count)
            player:event_notice(EVENT_ID, 0, { count, 4, 22603 })
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
        play(player, 1, HIDE_HOTBAR)
        return
    elseif step == 1 and scene == 1 then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif step == 6 and scene == 2 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        play(player, 3, HIDE_HOTBAR)
        return
    elseif step == 6 and scene == 3 then
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif step == 7 and scene == 5 then
        player:event_notice(EVENT_ID, 2, {})
        player:set_quest_var(EVENT_ID, 1, 0x20) -- UI8B
        player:quest_sequence(EVENT_ID, 4)
    elseif step == 8 and scene == 7 then
        player:event_notice(EVENT_ID, 3, {})
        player:set_quest_var(EVENT_ID, 1, 0x00) -- UI8B
        player:quest_sequence(EVENT_ID, 5)
    elseif step == 9 and scene == 8 then
        after_bar = { scene = 9, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif step == 9 and scene == 9 then
        player:event_notice(EVENT_ID, 4, {})
        player:quest_sequence(EVENT_ID, 6)
    elseif step == 10 and scene == 10 then
        after_bar = { scene = 11, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_PROCESS_UPPER_MIDDLE)
        return
    elseif step == 10 and scene == 11 then
        player:event_notice(EVENT_ID, 5, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif step == 11 and scene == 12 then
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
