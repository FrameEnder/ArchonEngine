-- Quest: The Ultimate Weapon
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Connor's Chronofoil capture af05916c (2026-11-07/09) (fixture tools/questkit/fixtures/4522.json):
-- Sequence 0:
--   ACTOR0 (Raubahn): scene 0 (HIDE_HOTBAR, a yield), 1 (HIDE_HOTBAR) -> accepted, sequence 1
-- Sequence 1:
--   ACTOR1 (Cid): scene 2 (HIDE_HOTBAR) -> sequence 2, notices [[0, []]], content 86
-- Sequence 2:
--   clearing INSTANCEDUNGEON0 -> sequence 3
-- Sequence 3:
--   entering the zone: scene 10 (SET_BASE | HIDE_HOTBAR | INVINCIBLE) -> sequence 4, notices [[2, []]], content 20082
-- Sequence 4:
--   clearing INSTANCEDUNGEON1 -> sequence 5
-- Sequence 5:
--   entering the zone: scene 19 (SET_BASE | HIDE_HOTBAR | INVINCIBLE) -> sequence 6, zone 1053
-- Sequence 6:
--   ACTOR5 (Lahabrea): scene 24 (HIDE_HOTBAR), 25 (SET_BASE | HIDE_HOTBAR | INVINCIBLE, a yield), into the quest battle, zone 1052
-- Sequence 7:
--   entering the zone: scene 30 (SET_BASE | INVIS_ENPC | INVIS_AETHERYTE | HIDE_HOTBAR | INVINCIBLE) -> sequence 255, notices [[6, []]]
-- Sequence 255:
--   ACTOR6 (Minfilia): scene 36 (SET_BASE | INVIS_ENPC | INVIS_AETHERYTE | HIDE_HOTBAR | INVINCIBLE, a yield) -> completed, zone 212

-- Both duties (the Praetorium, then the Porta Decumana) let out at POPRANGE2 in TERRITORYTYPE0, where the client asks
-- for the next cutscene (ZoneInit's pop range and position; rotation the client's first. Connor's Chronofoil capture
-- af05916c.)
DUTY_EXIT = { x = -690.0, y = -185.73, z = 480.0 }
DUTY_EXIT_ROTATION = -1.57

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
    elseif sequence == 6 and BASE_ID == ACTOR5 then
        step = 7
        play(player, 24, HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR6 then
        step = 9
        play(player, 36, SET_BASE | INVIS_ENPC | INVIS_AETHERYTE | HIDE_HOTBAR | INVINCIBLE)
        return
    end
    player:finish_event()
end

function onEnterTerritory(player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == 3 then
        step = 4
        play(player, 10, SET_BASE | HIDE_HOTBAR | INVINCIBLE)
        return
    elseif sequence == 5 then
        step = 6
        play(player, 19, SET_BASE | HIDE_HOTBAR | INVINCIBLE)
        return
    elseif sequence == 7 then
        step = 8
        play(player, 30, SET_BASE | INVIS_ENPC | INVIS_AETHERYTE | HIDE_HOTBAR | INVINCIBLE)
        return
    end
    player:finish_event()
end

function onDutyCleared(content_id, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    if sequence == 2 and content_id == INSTANCEDUNGEON0 then
        player:set_duty_exit(TERRITORYTYPE0, DUTY_EXIT, DUTY_EXIT_ROTATION)
        player:quest_sequence(EVENT_ID, 3)
    elseif sequence == 4 and content_id == INSTANCEDUNGEON1 then
        player:set_duty_exit(TERRITORYTYPE0, DUTY_EXIT, DUTY_EXIT_ROTATION)
        player:quest_sequence(EVENT_ID, 5)
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
        play(player, 1, HIDE_HOTBAR)
        return
    elseif step == 1 and scene == 1 then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif step == 2 and scene == 2 then
        player:unlock_content(INSTANCEDUNGEON0)
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif step == 4 and scene == 10 then
        player:unlock_content(INSTANCEDUNGEON1)
        player:event_notice(EVENT_ID, 2, {})
        player:quest_sequence(EVENT_ID, 4)
    elseif step == 6 and scene == 19 then
        player:quest_sequence(EVENT_ID, 6)
        player:finish_event()
        -- Where retail put the player (ActorSetPos; no pop range of the quest's.)
        player:change_territory(TERRITORYTYPE0, { x = -690.0, y = -185.73, z = 480.0 }, -1.57)
        return
    elseif step == 7 and scene == 24 then
        play(player, 25, SET_BASE | HIDE_HOTBAR | INVINCIBLE)
        return
    elseif step == 7 and scene == 25 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        player:finish_event()
        player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE1)
        return
    elseif step == 8 and scene == 30 then
        player:event_notice(EVENT_ID, 6, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif step == 9 and scene == 36 then
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
        -- Where retail put the player (ActorSetPos; no pop range of the quest's.)
        player:change_territory(TERRITORYTYPE2, { x = 36.68, y = 1.2, z = 2.17 }, 2.056)
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
