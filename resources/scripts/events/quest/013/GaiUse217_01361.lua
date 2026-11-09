-- Quest: Lord of the Whorl
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Connor's Chronofoil capture af05916c (2026-11-07/09) (fixture tools/questkit/fixtures/1361.json):
-- Sequence 0:
--   ACTOR0 (Eynzahr Slafyrsyn): scene 0 (HIDE_HOTBAR, a yield), 1 (SET_BASE | HIDE_HOTBAR) -> accepted, sequence 1, content 20017
-- Sequence 1:
--   clearing INSTANCEDUNGEON0 -> sequence 255, notices [[0, []]]
-- Sequence 255:
--   ACTOR10 (Zanthael): scene 17 (HIDE_HOTBAR, a yield), zone 198
--   ACTOR8 (Merlwyb): scene 14 (SET_BASE | HIDE_HOTBAR), 15 (HIDE_UI | HIDE_HOTBAR, a yield) -> completed, zone 129

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
    elseif sequence == 255 and BASE_ID == ACTOR10 then
        step = 3
        play(player, 17, HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR8 then
        step = 4
        play(player, 14, SET_BASE | HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onDutyCleared(content_id, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    if sequence == 1 and content_id == INSTANCEDUNGEON0 then
        -- Out of the duty at POPRANGE0 in 135 wherever it was queued from, as retail (ZoneInit's pop range and
        -- position, rotation the client's first; Connor's Chronofoil capture af05916c.)
        player:set_duty_exit(135, { x = 158.8, y = 8.97, z = 584.73 }, 1.557)
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 255)
    end
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
        play(player, 1, SET_BASE | HIDE_HOTBAR)
        return
    elseif step == 1 and scene == 1 then
        player:accept_quest(EVENT_ID)
        player:unlock_content(INSTANCEDUNGEON0)
        player:quest_sequence(EVENT_ID, 1)
    elseif step == 3 and scene == 17 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        player:finish_event()
        player:change_territory_pop_range(TERRITORYTYPE0, POPRANGE1)
        return
    elseif step == 4 and scene == 14 then
        play(player, 15, HIDE_UI | HIDE_HOTBAR)
        return
    elseif step == 4 and scene == 15 then
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
        player:change_territory_pop_range(129, POPRANGE2)
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
