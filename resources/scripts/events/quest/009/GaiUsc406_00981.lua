-- Quest: Footprints in the Snow
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Bob's Chronofoil capture 689b15da (2026-11-09) (fixture tools/questkit/fixtures/981-689b15da.json) and Connor's Chronofoil capture e834e436 (2026-11-07/09) (fixture tools/questkit/fixtures/981.json):
-- Sequence 0:
--   ACTOR0 (Pierremons): scene 0 (HIDE_HOTBAR, a yield), 1 (HIDE_HOTBAR) -> accepted, sequence 1
-- Sequence 1:
--   EOBJECT0 (footprints): scene 2 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_LOOKOUT_LONG), 3 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 2, notices [[0, []]]
-- Sequence 2:
--   EOBJECT6 (footprints): scene 14 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_LOOKOUT_LONG), 15 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 3, notices [[1, []]]
-- Sequence 3:
--   EOBJECT7 (footprints): scene 26 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_LOOKOUT_LONG), 27 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 4, notices [[2, []]]
-- Sequence 4:
--   EOBJECT8 (footprints): scene 38 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_LOOKOUT_LONG), 39 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 5, notices [[3, []]]
-- Sequence 5:
--   EOBJECT9 (footprints): scene 50 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_LOOKOUT_LONG), 51 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 6, notices [[4, []]]
-- Sequence 6:
--   ACTOR1 (Wedge): scene 62 (SET_BASE | HIDE_HOTBAR) -> sequence 255, notices [[5, []]]
-- Sequence 255:
--   ACTOR2 (Abelie): scene 73 (SET_BASE | INVIS_ENPC | INVIS_AETHERYTE | HIDE_HOTBAR, a yield) -> completed

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
        play(player, 2, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 2 and BASE_ID == EOBJECT6 then
        step = 3
        play(player, 14, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 3 and BASE_ID == EOBJECT7 then
        step = 4
        play(player, 26, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 4 and BASE_ID == EOBJECT8 then
        step = 5
        play(player, 38, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 5 and BASE_ID == EOBJECT9 then
        step = 6
        play(player, 50, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 6 and BASE_ID == ACTOR1 then
        step = 7
        play(player, 62, SET_BASE | HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        step = 8
        play(player, 73, SET_BASE | INVIS_ENPC | INVIS_AETHERYTE | HIDE_HOTBAR)
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
    elseif step == 2 and scene == 2 then
        after_bar = { scene = 3, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_LOOKOUT_LONG)
        return
    elseif step == 2 and scene == 3 then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif step == 3 and scene == 14 then
        after_bar = { scene = 15, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_LOOKOUT_LONG)
        return
    elseif step == 3 and scene == 15 then
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif step == 4 and scene == 26 then
        after_bar = { scene = 27, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_LOOKOUT_LONG)
        return
    elseif step == 4 and scene == 27 then
        player:event_notice(EVENT_ID, 2, {})
        player:quest_sequence(EVENT_ID, 4)
    elseif step == 5 and scene == 38 then
        after_bar = { scene = 39, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_LOOKOUT_LONG)
        return
    elseif step == 5 and scene == 39 then
        player:event_notice(EVENT_ID, 3, {})
        player:quest_sequence(EVENT_ID, 5)
    elseif step == 6 and scene == 50 then
        after_bar = { scene = 51, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_LOOKOUT_LONG)
        return
    elseif step == 6 and scene == 51 then
        player:event_notice(EVENT_ID, 4, {})
        player:quest_sequence(EVENT_ID, 6)
    elseif step == 7 and scene == 62 then
        player:event_notice(EVENT_ID, 5, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif step == 8 and scene == 73 then
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
