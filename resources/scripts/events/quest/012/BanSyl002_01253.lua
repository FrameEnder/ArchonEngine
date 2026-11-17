-- Quest: Voyce of Concern
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Bob's Chronofoil capture 7a4214b7 (2026-11-17) (fixture tools/questkit/fixtures/1253.json):
-- Sequence 0:
--   ACTOR0 (Olmxio): scene 0 (HIDE_HOTBAR, a yield), 1 (HIDE_HOTBAR) -> accepted, sequence 1
-- Sequence 1:
--   EOBJECT0 (destination): scene 6 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_WAITING_SHOR), 7 (SET_BASE | HIDE_HOTBAR | INVINCIBLE) -> sequence 2, zone 152
-- Sequence 2:
--   ACTOR1 (Voyce): scene 8 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_RESCUE_UNDER_MIDDLE), 9 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 3, notices [[1, []]]
-- Sequence 3:
--   ACTOR3 (Voyce): scene 10 (SET_BASE | HIDE_HOTBAR) -> sequence 255, notices [[2, []]]
-- Sequence 255:
--   ACTOR0 (Olmxio): scene 11 (HIDE_HOTBAR, a yield) -> completed

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
    elseif sequence == 1 and BASE_ID == EOBJECT0 then
        step = 2
        play(player, 6, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 2 and BASE_ID == ACTOR1 then
        step = 3
        play(player, 8, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 3 and BASE_ID == ACTOR3 then
        step = 4
        play(player, 10, SET_BASE | HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        step = 5
        play(player, 11, HIDE_HOTBAR)
        return
    end
    player:finish_event()
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
    elseif step == 2 and scene == 6 then
        after_bar = { scene = 7, flags = SET_BASE | HIDE_HOTBAR | INVINCIBLE }
        player:event_action(EVENT_ACTION_WAITING_SHOR)
        return
    elseif step == 2 and scene == 7 then
        player:quest_sequence(EVENT_ID, 2)
        player:finish_event()
        -- Where retail put the player (ActorSetPos; no pop range of the quest's.)
        player:change_territory(152, { x = 289.15, y = -1.04, z = 148.18 }, 2.222)
        return
    elseif step == 3 and scene == 8 then
        after_bar = { scene = 9, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_RESCUE_UNDER_MIDDLE)
        return
    elseif step == 3 and scene == 9 then
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif step == 4 and scene == 10 then
        player:event_notice(EVENT_ID, 2, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif step == 5 and scene == 11 then
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
