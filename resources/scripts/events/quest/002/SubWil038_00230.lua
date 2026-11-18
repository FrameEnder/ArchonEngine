-- Quest: Search and Seizure
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Bob's Chronofoil capture dba9b575 of 2026-11-18 (fixture tools/questkit/fixtures/230.json):
-- Sequence 0:
--   ACTOR0 (Kikipu): scene 0 (HIDE_HOTBAR, a yield), 90 (SET_BASE | HIDE_HOTBAR) -> accepted, sequence 1
-- Sequence 1:
--   walking into EVENTRANGE2: scene 9 (NO_DEFAULT_CAMERA | HIDE_HOTBAR)
--   defeating 778, vars 00 01 00 00 00 00
--   EOBJECT2 (writ of seizure): scene 10 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_PROCESS_UPPER_SHORT), 11 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), vars 10 02 01 00 00 10, notices [[0, [1, 4, 25906]]]
--   walking into EVENTRANGE1: scene 6 (NO_DEFAULT_CAMERA | HIDE_HOTBAR)
--   defeating 779, vars 10 12 01 00 00 10
--   EOBJECT1 (writ of seizure): scene 7 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_PROCESS_UPPER_SHORT), 8 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), vars 20 22 02 00 00 30, notices [[0, [2, 4, 25906]]]
--   walking into EVENTRANGE0: scene 3 (NO_DEFAULT_CAMERA | HIDE_HOTBAR)
--   EOBJECT0 (writ of seizure): scene 4 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), vars 21 22 02 00 00 30
--   EOBJECT0 (writ of seizure): scene 4 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_PROCESS_UPPER_SHORT), 5 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), vars 32 22 03 00 00 70, notices [[0, [3, 4, 25906]]]
--   walking into EVENTRANGE3: scene 12 (NO_DEFAULT_CAMERA | HIDE_HOTBAR)
--   EOBJECT3 (writ of seizure): scene 13 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_PROCESS_UPPER_SHORT), 14 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 2, vars 00 40 00 00 00 00, notices [[0, [4, 4, 25906]]]
-- Sequence 2:
--   ACTOR1 (Kenrick): scene 15 (NO_DEFAULT_CAMERA | HIDE_HOTBAR | INVINCIBLE, a yield), 16 (NO_DEFAULT_CAMERA | HIDE_HOTBAR | INVINCIBLE) -> sequence 3, vars 00 00 00 00 00 00, notices [[1, []]]
-- Sequence 3:
--   ACTOR0 (Kikipu): scene 17 (SET_BASE | HIDE_HOTBAR) -> sequence 255, vars 00 10 00 00 00 00, notices [[2, [25918]]]
-- Sequence 255:
--   ACTOR2 (Momodi): scene 18 (HIDE_HOTBAR, a yield), 19 (HIDE_HOTBAR, a yield) -> completed

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
    elseif sequence == 1 and BASE_ID == EOBJECT2 then
        step = 4
        play(player, 10, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == EOBJECT1 then
        step = 7
        play(player, 7, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == EOBJECT0 and player:get_quest_var(EVENT_ID, 0) == 0x20 then
        step = 9
        play(player, 4, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == EOBJECT0 and player:get_quest_var(EVENT_ID, 0) == 0x21 then
        step = 10
        play(player, 4, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == EOBJECT3 then
        step = 12
        play(player, 13, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 2 and BASE_ID == ACTOR1 then
        step = 13
        play(player, 15, NO_DEFAULT_CAMERA | HIDE_HOTBAR | INVINCIBLE)
        return
    elseif sequence == 3 and BASE_ID == ACTOR0 then
        step = 14
        play(player, 17, SET_BASE | HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        step = 15
        play(player, 18, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEnterTrigger(player, arg)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == 1 and arg == EVENTRANGE2 then
        step = 2
        play(player, 9, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 1 and arg == EVENTRANGE1 then
        step = 5
        play(player, 6, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 1 and arg == EVENTRANGE0 then
        step = 8
        play(player, 3, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 1 and arg == EVENTRANGE3 then
        step = 11
        play(player, 12, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onBNpcKill(base_id, layout_id, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    if sequence == 1 and layout_id == ENEMY2 then
        player:set_quest_var(EVENT_ID, 1, 0x01) -- UI8B
    elseif sequence == 1 and layout_id == ENEMY1 then
        player:set_quest_var(EVENT_ID, 1, 0x12) -- UI8B
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
        play(player, 90, SET_BASE | HIDE_HOTBAR)
        return
    elseif step == 1 and scene == 90 then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif step == 2 and scene == 9 then
        player:spawn_quest_bnpc(ENEMY2)
    elseif step == 4 and scene == 10 then
        after_bar = { scene = 11, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_PROCESS_UPPER_SHORT)
        return
    elseif step == 4 and scene == 11 then
        player:event_notice(EVENT_ID, 0, { 1, 4, 25906 })
        player:set_quest_var(EVENT_ID, 0, 0x10) -- UI8A
        player:set_quest_var(EVENT_ID, 1, 0x02) -- UI8B
        player:set_quest_var(EVENT_ID, 2, 0x01) -- UI8C
        player:set_quest_var(EVENT_ID, 5, 0x10) -- UI8F
    elseif step == 5 and scene == 6 then
        player:spawn_quest_bnpc(ENEMY1)
    elseif step == 7 and scene == 7 then
        after_bar = { scene = 8, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_PROCESS_UPPER_SHORT)
        return
    elseif step == 7 and scene == 8 then
        player:event_notice(EVENT_ID, 0, { 2, 4, 25906 })
        player:set_quest_var(EVENT_ID, 0, 0x20) -- UI8A
        player:set_quest_var(EVENT_ID, 1, 0x22) -- UI8B
        player:set_quest_var(EVENT_ID, 2, 0x02) -- UI8C
        player:set_quest_var(EVENT_ID, 5, 0x30) -- UI8F
    elseif step == 8 and scene == 3 then
        player:spawn_quest_bnpc(ENEMY0)
    elseif step == 9 and scene == 4 then
        player:set_quest_var(EVENT_ID, 0, 0x21) -- UI8A
    elseif step == 10 and scene == 4 then
        after_bar = { scene = 5, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_PROCESS_UPPER_SHORT)
        return
    elseif step == 10 and scene == 5 then
        player:event_notice(EVENT_ID, 0, { 3, 4, 25906 })
        player:set_quest_var(EVENT_ID, 0, 0x32) -- UI8A
        player:set_quest_var(EVENT_ID, 2, 0x03) -- UI8C
        player:set_quest_var(EVENT_ID, 5, 0x70) -- UI8F
    elseif step == 11 and scene == 12 then
        player:spawn_quest_bnpc(ENEMY3)
    elseif step == 12 and scene == 13 then
        after_bar = { scene = 14, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_PROCESS_UPPER_SHORT)
        return
    elseif step == 12 and scene == 14 then
        player:event_notice(EVENT_ID, 0, { 4, 4, 25906 })
        player:set_quest_var(EVENT_ID, 0, 0x00) -- UI8A
        player:set_quest_var(EVENT_ID, 1, 0x40) -- UI8B
        player:set_quest_var(EVENT_ID, 2, 0x00) -- UI8C
        player:set_quest_var(EVENT_ID, 5, 0x00) -- UI8F
        player:quest_sequence(EVENT_ID, 2)
    elseif step == 13 and scene == 15 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        play(player, 16, NO_DEFAULT_CAMERA | HIDE_HOTBAR | INVINCIBLE)
        return
    elseif step == 13 and scene == 16 then
        player:event_notice(EVENT_ID, 1, {})
        player:set_quest_var(EVENT_ID, 1, 0x00) -- UI8B
        player:quest_sequence(EVENT_ID, 3)
    elseif step == 14 and scene == 17 then
        player:event_notice(EVENT_ID, 2, { 25918 })
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8B
        player:quest_sequence(EVENT_ID, 255)
    elseif step == 15 and scene == 18 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        play(player, 19, HIDE_HOTBAR)
        return
    elseif step == 15 and scene == 19 then
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
