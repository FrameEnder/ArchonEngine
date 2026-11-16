-- Quest: Deduce, Reuse, Recycle
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and scenes and effects from Bob's Chronofoil capture of 2026-11-16 (fixture tools/questkit/fixtures/862.json):
-- Sequence 0:
--   ACTOR0 (Swygrael): scene 0 (HIDE_HOTBAR, a yield), 1 (HIDE_HOTBAR) -> accepted, sequence 1
-- Sequence 1:
--   EOBJECT0 (velociraptor claw): scene 2 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_SEARCH), 3 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), vars 01 01 00 00 80 00, notices [[0, [1, 3, 22301]]]
--   EOBJECT3 (serrated arrowhead): scene 14 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_SEARCH), 15 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), vars 01 11 10 00 80 40, notices [[1, [1, 3, 27241]]]
--   EOBJECT4 (serrated arrowhead): scene 16 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_SEARCH), 17 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), vars 01 21 20 00 80 60, notices [[1, [2, 3, 27241]]]
--   EOBJECT1 (velociraptor claw): scene 4 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_SEARCH), 5 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), vars 02 22 20 00 c0 60, notices [[0, [2, 3, 22301]]]
--   EOBJECT2 (velociraptor claw): scene 6 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_SEARCH), 7 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), vars 13 23 20 00 e0 60, notices [[0, [3, 3, 22301]]]
--   EOBJECT5 (serrated arrowhead): scene 18 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_SEARCH), 19 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 255, vars 00 33 00 00 00 00, notices [[1, [3, 3, 27241]]]
-- Sequence 255:
--   ACTOR0 (Swygrael): scene 20 (HIDE_HOTBAR, a yield), 21 (HIDE_HOTBAR, a yield) -> completed

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
    elseif sequence == 1 and BASE_ID == EOBJECT3 then
        step = 3
        play(player, 14, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == EOBJECT4 then
        step = 4
        play(player, 16, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == EOBJECT1 then
        step = 5
        play(player, 4, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == EOBJECT2 then
        step = 6
        play(player, 6, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == EOBJECT5 then
        step = 7
        play(player, 18, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        step = 8
        play(player, 20, HIDE_HOTBAR)
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
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif step == 2 and scene == 3 then
        player:event_notice(EVENT_ID, 0, { 1, 3, 22301 })
        player:set_quest_var(EVENT_ID, 0, 0x01) -- UI8A
        player:set_quest_var(EVENT_ID, 1, 0x01) -- UI8B
        player:set_quest_var(EVENT_ID, 4, 0x80) -- UI8E
    elseif step == 3 and scene == 14 then
        after_bar = { scene = 15, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif step == 3 and scene == 15 then
        player:event_notice(EVENT_ID, 1, { 1, 3, 27241 })
        player:set_quest_var(EVENT_ID, 1, 0x11) -- UI8B
        player:set_quest_var(EVENT_ID, 2, 0x10) -- UI8C
        player:set_quest_var(EVENT_ID, 5, 0x40) -- UI8F
    elseif step == 4 and scene == 16 then
        after_bar = { scene = 17, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif step == 4 and scene == 17 then
        player:event_notice(EVENT_ID, 1, { 2, 3, 27241 })
        player:set_quest_var(EVENT_ID, 1, 0x21) -- UI8B
        player:set_quest_var(EVENT_ID, 2, 0x20) -- UI8C
        player:set_quest_var(EVENT_ID, 5, 0x60) -- UI8F
    elseif step == 5 and scene == 4 then
        after_bar = { scene = 5, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif step == 5 and scene == 5 then
        player:event_notice(EVENT_ID, 0, { 2, 3, 22301 })
        player:set_quest_var(EVENT_ID, 0, 0x02) -- UI8A
        player:set_quest_var(EVENT_ID, 1, 0x22) -- UI8B
        player:set_quest_var(EVENT_ID, 4, 0xC0) -- UI8E
    elseif step == 6 and scene == 6 then
        after_bar = { scene = 7, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif step == 6 and scene == 7 then
        player:event_notice(EVENT_ID, 0, { 3, 3, 22301 })
        player:set_quest_var(EVENT_ID, 0, 0x13) -- UI8A
        player:set_quest_var(EVENT_ID, 1, 0x23) -- UI8B
        player:set_quest_var(EVENT_ID, 4, 0xE0) -- UI8E
    elseif step == 7 and scene == 18 then
        after_bar = { scene = 19, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif step == 7 and scene == 19 then
        player:event_notice(EVENT_ID, 1, { 3, 3, 27241 })
        player:set_quest_var(EVENT_ID, 0, 0x00) -- UI8A
        player:set_quest_var(EVENT_ID, 1, 0x33) -- UI8B
        player:set_quest_var(EVENT_ID, 2, 0x00) -- UI8C
        player:set_quest_var(EVENT_ID, 4, 0x00) -- UI8E
        player:set_quest_var(EVENT_ID, 5, 0x00) -- UI8F
        player:quest_sequence(EVENT_ID, 255)
    elseif step == 8 and scene == 20 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        play(player, 21, HIDE_HOTBAR)
        return
    elseif step == 8 and scene == 21 then
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
