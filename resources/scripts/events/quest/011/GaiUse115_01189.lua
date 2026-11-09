-- Quest: Hail to the King, Kupo
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Connor's Chronofoil capture af05916c (2026-11-07/09) (fixture tools/questkit/fixtures/1189.json):
-- Sequence 0:
--   ACTOR0 (Vorsaile Heuloix): scene 0 (HIDE_HOTBAR, a yield), 1 (HIDE_HOTBAR) -> accepted, sequence 1
-- Sequence 1:
--   ACTOR6 (silent conjurer): scene 7 (HIDE_HOTBAR, a yield), zone 205
--   ACTOR1 (Kan-E-Senna): scene 2 (SET_BASE | HIDE_HOTBAR) -> sequence 255, notices [[0, [1, 1]]]
-- Sequence 255:
--   ACTOR7 (E-Sumi-Yan): scene 8 (HIDE_HOTBAR, a yield) -> completed

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
    elseif sequence == 1 and BASE_ID == ACTOR6 then
        step = 2
        play(player, 7, HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        step = 3
        play(player, 2, SET_BASE | HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR7 then
        step = 4
        play(player, 8, HIDE_HOTBAR)
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
    elseif step == 2 and scene == 7 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        player:finish_event()
        player:change_territory_pop_range(TERRITORYTYPE0, POPRANGE0)
        return
    elseif step == 3 and scene == 2 then
        player:event_notice(EVENT_ID, 0, { 1, 1 })
        player:quest_sequence(EVENT_ID, 255)
    elseif step == 4 and scene == 8 then
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
