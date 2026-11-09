-- Quest: You Have Selected Regicide
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Connor's Chronofoil capture af05916c (2026-11-07/09) (fixture tools/questkit/fixtures/1190.json):
-- Sequence 0:
--   ACTOR0 (E-Sumi-Yan): scene 0 (HIDE_HOTBAR, a yield), 1 (HIDE_HOTBAR) -> accepted, sequence 1
-- Sequence 1:
--   ACTOR1 (Kuplo Kopp): scene 2 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 2, notices [[0, []]]
-- Sequence 2:
--   walking into EVENTRANGE0: scene 3 (SET_BASE | INVIS_ENPC | INVIS_AETHERYTE | HIDE_HOTBAR, a yield) -> sequence 3, notices [[1, []]]
-- Sequence 3:
--   ACTOR6 (Kuplo Kopp): scene 8 (HIDE_HOTBAR) -> sequence 4, notices [[2, []]], unlock 565
-- Sequence 4:
--   walking into EVENTRANGE1: scene 11 (NO_DEFAULT_CAMERA | HIDE_HOTBAR)
--   walking into EVENTRANGE2: scene 12 (NO_DEFAULT_CAMERA | HIDE_HOTBAR)
--   walking into EVENTRANGE3: scene 13 (NO_DEFAULT_CAMERA | HIDE_HOTBAR)
--   walking into EVENTRANGE4: scene 14 (NO_DEFAULT_CAMERA | HIDE_HOTBAR)
--   walking into EVENTRANGE5: scene 15 (NO_DEFAULT_CAMERA | HIDE_HOTBAR)
--   EOBJECT0 (barrier threshold): scene 9 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 5, notices [[3, []]]
-- Sequence 5:
--   ACTOR7 (Kuplo Kopp): scene 20 (HIDE_HOTBAR) -> sequence 6, notices [[4, []]]
-- Sequence 6:
--   EOBJECT2 (barrier threshold): scene 23 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (19), 24 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), 25 (SET_BASE | INVIS_ENPC | INVIS_AETHERYTE | HIDE_HOTBAR) -> sequence 7, notices [[5, []]], content 20011
-- Sequence 7:
--   clearing INSTANCEDUNGEON0 -> sequence 8
-- Sequence 8:
--   entering the zone: scene 32 (SET_BASE | HIDE_HOTBAR) -> sequence 255, notices [[7, []]]
-- Sequence 255:
--   ACTOR9 (Pukni Pakk): scene 35 (HIDE_HOTBAR, a yield) -> completed

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
        play(player, 2, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 3 and BASE_ID == ACTOR6 then
        step = 4
        play(player, 8, HIDE_HOTBAR)
        return
    elseif sequence == 4 and BASE_ID == EOBJECT0 then
        step = 10
        play(player, 9, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 5 and BASE_ID == ACTOR7 then
        step = 11
        play(player, 20, HIDE_HOTBAR)
        return
    elseif sequence == 6 and BASE_ID == EOBJECT2 then
        step = 12
        play(player, 23, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR9 then
        step = 15
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
        play(player, 3, SET_BASE | INVIS_ENPC | INVIS_AETHERYTE | HIDE_HOTBAR)
        return
    elseif sequence == 4 and arg == EVENTRANGE1 then
        step = 5
        play(player, 11, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 4 and arg == EVENTRANGE2 then
        step = 6
        play(player, 12, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 4 and arg == EVENTRANGE3 then
        step = 7
        play(player, 13, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 4 and arg == EVENTRANGE4 then
        step = 8
        play(player, 14, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 4 and arg == EVENTRANGE5 then
        step = 9
        play(player, 15, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEnterTerritory(player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == 8 then
        step = 14
        play(player, 32, SET_BASE | HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onDutyCleared(content_id, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    if sequence == 7 and content_id == INSTANCEDUNGEON0 then
        -- Out of the duty at POPRANGE0 in 152 wherever it was queued from, as retail (ZoneInit's pop range and
        -- position, rotation the client's first; Connor's Chronofoil capture af05916c.)
        player:set_duty_exit(TERRITORYTYPE0, { x = -315.55, y = 11.92, z = -34.56 }, 1.435)
        player:quest_sequence(EVENT_ID, 8)
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
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif step == 4 and scene == 8 then
        player:spawn_quest_bnpc(ENEMY0)
        player:unlock(565)
        player:event_notice(EVENT_ID, 2, {})
        player:quest_sequence(EVENT_ID, 4)
    elseif step == 5 and scene == 11 then
        player:spawn_quest_bnpc(ENEMY1)
    elseif step == 6 and scene == 12 then
        player:spawn_quest_bnpc(ENEMY4)
        player:spawn_quest_bnpc(ENEMY3)
        player:spawn_quest_bnpc(ENEMY2)
    elseif step == 7 and scene == 13 then
        player:spawn_quest_bnpc(ENEMY5)
    elseif step == 8 and scene == 14 then
        player:spawn_quest_bnpc(ENEMY7)
        player:spawn_quest_bnpc(ENEMY6)
    elseif step == 9 and scene == 15 then
        player:spawn_quest_bnpc(ENEMY10)
        player:spawn_quest_bnpc(ENEMY9)
        player:spawn_quest_bnpc(ENEMY8)
    elseif step == 10 and scene == 9 then
        player:event_notice(EVENT_ID, 3, {})
        player:quest_sequence(EVENT_ID, 5)
    elseif step == 11 and scene == 20 then
        player:event_notice(EVENT_ID, 4, {})
        player:quest_sequence(EVENT_ID, 6)
    elseif step == 12 and scene == 23 then
        after_bar = { scene = 24, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_SYMPATHY)
        return
    elseif step == 12 and scene == 24 then
        play(player, 25, SET_BASE | INVIS_ENPC | INVIS_AETHERYTE | HIDE_HOTBAR)
        return
    elseif step == 12 and scene == 25 then
        player:unlock_content(INSTANCEDUNGEON0)
        player:event_notice(EVENT_ID, 5, {})
        player:quest_sequence(EVENT_ID, 7)
    elseif step == 14 and scene == 32 then
        player:event_notice(EVENT_ID, 7, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif step == 15 and scene == 35 then
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
