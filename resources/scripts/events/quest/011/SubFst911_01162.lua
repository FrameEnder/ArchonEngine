-- Quest: My Feisty Little Chocobo
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Bob's Chronofoil capture 898dd0b9 (2026-11-12) (fixture tools/questkit/fixtures/1162-898dd0b9.json) and Bob's Chronofoil capture 0abe6734 (2026-11-12) (fixture tools/questkit/fixtures/1162-0abe6734.json) and Bob's Chronofoil capture da424662 (2026-11-12) (fixture tools/questkit/fixtures/1162.json):
-- Sequence 0:
--   ACTOR0 (Docette): scene 0 (HIDE_HOTBAR, a yield), 1 (HIDE_HOTBAR) -> accepted, sequence 1
--   ACTOR1 (Luquelot): scene 2 (HIDE_HOTBAR) -> accepted, sequence 2, notices [[0, []]]
-- Sequence 1:
--   ACTOR1 (Luquelot): scene 2 (HIDE_HOTBAR) -> sequence 2, notices [[0, []]]
-- Sequence 2:
--   EOBJECT0 (gysahl greens): scene 3 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_PROCESS_MIDDLE), 4 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 3, vars 00 10 00 00 00 00, notices [[1, [25218]]]
-- Sequence 3:
--   ACTOR1 (Luquelot): scene 5 (HIDE_HOTBAR, a yield), 6 (HIDE_HOTBAR) -> sequence 4, vars 00 00 00 00 00 00, notices [[2, [25218]]], unlock 18; RITEM0 given before the unlock (capture da424662)
-- Sequence 4:
--   defeating ENEMY0, vars 01 00 00 00 00 00, notices [[3, [1, 3]]]
--   defeating ENEMY0, vars 02 00 00 00 00 00, notices [[3, [2, 3]]]
--   defeating ENEMY0 -> sequence 255, vars 00 00 00 00 00 00, notices [[3, [3, 3]]]
--   defeating ENEMY0, vars 01 00 00 00 00 00, notices [[3, [1, 3]]]
--   defeating ENEMY0, vars 02 00 00 00 00 00, notices [[3, [2, 3]]]
--   defeating ENEMY0 -> sequence 255, vars 00 00 00 00 00 00, notices [[3, [3, 3]]]
-- Sequence 255:
--   ACTOR1 (Luquelot): scene 7 (HIDE_HOTBAR, a yield) -> completed

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
    elseif sequence == 2 and BASE_ID == EOBJECT0 then
        step = 3
        play(player, 3, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 3 and BASE_ID == ACTOR1 then
        step = 4
        play(player, 5, HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        step = 8
        play(player, 7, HIDE_HOTBAR)
        return
    elseif sequence == nil and BASE_ID == ACTOR1 then
        step = 12
        play(player, 2, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

-- Three leafbleed mites (ENEMY0, any of them), counted in UI8A with notice 3 [n, 3]; the third clears it and moves the
-- quest to 255 (retail, Bob's Chronofoil captures da424662, 0abe6734 and 898dd0b9; written by hand, the transcription
-- had one branch per kill.)
MITES_NEEDED = 3

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 4 or base_id ~= ENEMY0 then
        return
    end
    local count = player:get_quest_var(EVENT_ID, 0) + 1
    player:event_notice(EVENT_ID, 3, { count, MITES_NEEDED })
    if count >= MITES_NEEDED then
        player:set_quest_var(EVENT_ID, 0, 0)
        player:quest_sequence(EVENT_ID, 255)
    else
        player:set_quest_var(EVENT_ID, 0, count)
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
    elseif step == 2 and scene == 2 then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif step == 3 and scene == 3 then
        after_bar = { scene = 4, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_PROCESS_MIDDLE)
        return
    elseif step == 3 and scene == 4 then
        player:event_notice(EVENT_ID, 1, { 25218 })
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8B
        player:quest_sequence(EVENT_ID, 3)
    elseif step == 4 and scene == 5 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        play(player, 6, HIDE_HOTBAR)
        return
    elseif step == 4 and scene == 6 then
        player:obtain_item(RITEM0, 1)
        player:unlock(18)
        player:event_notice(EVENT_ID, 2, { 25218 })
        player:set_quest_var(EVENT_ID, 1, 0x00) -- UI8B
        player:quest_sequence(EVENT_ID, 4)
    elseif step == 8 and scene == 7 then
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
    elseif step == 12 and scene == 2 then
        player:accept_quest(EVENT_ID)
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    end

    player:finish_event()
end

function onYield(scene, id, results, player)
    sceneFinished(scene, results, player)
end

function onReturn(scene, results, player)
    sceneFinished(scene, results, player)
end
