-- Quest: Return to Your Dust
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and scenes and effects from Bob's Chronofoil capture of 2026-11-16 (fixture tools/questkit/fixtures/861.json):
-- Sequence 0:
--   ACTOR0 (Bloeidin): scene 0 (HIDE_HOTBAR, a yield), 1 (HIDE_HOTBAR) -> accepted, sequence 1, vars 00 01 00 00 00 00
-- Sequence 1:
--   defeating ENEMY0 -> sequence 2, vars 00 11 00 00 00 00, notices [[0, [21202]]]
-- Sequence 2:
--   EOBJECT0 (destination): scene 2 (NO_DEFAULT_CAMERA | HIDE_HOTBAR)
--   key item 2000668 on EOBJECT0 (destination): scene 3 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 3, vars 00 10 00 00 00 00, notices [[1, []]]
-- Sequence 3:
--   EOBJECT1 (destination): scene 4 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_WAITING2_MIDDLE), 5 (NO_DEFAULT_CAMERA | HIDE_HOTBAR)
--   key item 2000960 on ENEMY2 (frozen circle)
--   defeating ENEMY2 -> sequence 255, vars 00 00 00 00 00 00, notices [[2, []]]
-- Sequence 255:
--   ACTOR0 (Bloeidin): scene 8 (HIDE_HOTBAR, a yield) -> completed

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
    elseif sequence == 2 and BASE_ID == EOBJECT0 then
        step = 3
        play(player, 2, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 3 and BASE_ID == EOBJECT1 then
        step = 5
        play(player, 4, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        step = 8
        play(player, 8, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEventItem(item, target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == 2 and item == ITEM1 and BASE_ID == EOBJECT0 then
        step = 4
        play(player, 3, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 3 and item == ITEM0 and BASE_ID == ENEMY2 then
        step = 6
        player:finish_event()
        return
    end
    player:finish_event()
end

function onBNpcKill(base_id, layout_id, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    if sequence == 1 and base_id == ENEMY0 then
        player:event_notice(EVENT_ID, 0, { 21202 })
        player:set_quest_var(EVENT_ID, 1, 0x11) -- UI8B
        player:quest_sequence(EVENT_ID, 2)
    elseif sequence == 3 and layout_id == ENEMY1 then
        player:event_notice(EVENT_ID, 2, {})
        player:set_quest_var(EVENT_ID, 1, 0x00) -- UI8B
        player:quest_sequence(EVENT_ID, 255)
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
        player:set_quest_var(EVENT_ID, 1, 0x01) -- UI8B
        player:quest_sequence(EVENT_ID, 1)
    elseif step == 3 and scene == 2 then
    elseif step == 4 and scene == 3 then
        player:event_notice(EVENT_ID, 1, {})
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8B
        player:quest_sequence(EVENT_ID, 3)
    elseif step == 5 and scene == 4 then
        after_bar = { scene = 5, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_WAITING2_MIDDLE)
        return
    elseif step == 5 and scene == 5 then
        player:spawn_quest_bnpc(ENEMY1)
    elseif step == 8 and scene == 8 then
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
