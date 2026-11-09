-- Quest: Casing the Castrum
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Bob's Chronofoil capture 689b15da (2026-11-09) (fixture tools/questkit/fixtures/978-689b15da.json) and Connor's Chronofoil capture e834e436 (2026-11-07/09) (fixture tools/questkit/fixtures/978.json):
-- Sequence 0:
--   ACTOR0 (Y'shtola): scene 0 (HIDE_HOTBAR, a yield), 1 (HIDE_HOTBAR) -> accepted, sequence 255
-- Sequence 255:
--   ACTOR1 (Portelaine): scene 2 (HIDE_HOTBAR, a yield), 3 (SET_BASE | HIDE_HOTBAR, a yield), 4 (SET_BASE | HIDE_HOTBAR, a yield) -> completed

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
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        step = 2
        play(player, 2, HIDE_HOTBAR)
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
        player:quest_sequence(EVENT_ID, 255)
    elseif step == 2 and scene == 2 then
        if results[1] ~= 1 then
            -- The reward window closed: the quest stays open.
            if player:get_quest_var(EVENT_ID, 0) ~= REWARD_CLOSED then
                player:set_quest_var(EVENT_ID, 0, REWARD_CLOSED)
            end
            player:finish_event()
            return
        end
        chosen = results[2] or 0
        play(player, 3, SET_BASE | HIDE_HOTBAR, { chosen })
        return
    elseif step == 2 and scene == 3 then
        play(player, 4, SET_BASE | HIDE_HOTBAR, { chosen })
        return
    elseif step == 2 and scene == 4 then
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
