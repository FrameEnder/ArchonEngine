-- Quest: I Believe You Can Fly
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Bob's Chronofoil capture dba9b575 of 2026-11-18 (fixture tools/questkit/fixtures/224.json):
-- Sequence 0:
--   ACTOR0 (Oswell): scene 0 (HIDE_HOTBAR, a yield) -> accepted, sequence 1, vars 00 10 00 00 00 00
-- Sequence 1:
--   defeating ENEMY0, vars 01 11 00 00 00 00, notices [[0, [1, 5, 21906]]]
--   defeating ENEMY0, vars 02 12 00 00 00 00, notices [[0, [2, 5, 21906]]]
--   defeating ENEMY0, vars 03 13 00 00 00 00, notices [[0, [3, 5, 21906]]]
--   defeating ENEMY0, vars 04 14 00 00 00 00, notices [[0, [4, 5, 21906]]]
--   defeating ENEMY0 -> sequence 255, vars 00 15 00 00 00 00, notices [[0, [5, 5, 21906]]]
-- Sequence 255:
--   ACTOR1 (Kikipu): scene 1 (HIDE_HOTBAR), 100 (HIDE_HOTBAR, a yield) -> completed

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
        step = 7
        play(player, 1, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

NEEDED = 5

function onBNpcKill(base_id, layout_id, player)
    -- One of ITEM1 from each: UI8A counts them and UI8BL the kills that gave one, beside UI8BH's 1 from the offer
    -- (retail's 5 kills each gave one, a notice with each; the transcriber took the changes for different kills.)
    -- With the last, UI8A is cleared and the quest moves on.
    local sequence = player:get_quest_sequence(EVENT_ID)
    if sequence == 1 and base_id == ENEMY0 then
        local held = player:get_quest_var(EVENT_ID, 1)
        local count = (held & 0x0F) + 1
        player:event_notice(EVENT_ID, 0, { count, NEEDED, 21906 })
        player:set_quest_var(EVENT_ID, 1, (held & 0xF0) | count) -- UI8BL
        if count >= NEEDED then
            player:set_quest_var(EVENT_ID, 0, 0x00) -- UI8A
            player:quest_sequence(EVENT_ID, 255)
        else
            player:set_quest_var(EVENT_ID, 0, count) -- UI8A
        end
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
        player:accept_quest(EVENT_ID)
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8B
        player:quest_sequence(EVENT_ID, 1)
    elseif step == 7 and scene == 1 then
        play(player, 100, HIDE_HOTBAR)
        return
    elseif step == 7 and scene == 100 then
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
