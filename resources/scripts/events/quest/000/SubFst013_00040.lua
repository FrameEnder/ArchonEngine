-- Quest: For Friendship
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Bob's Chronofoil capture b58bf0d5 (2026-11-23) (fixture tools/questkit/fixtures/40.json):
-- Sequence 0:
--   ACTOR0 (Nicoliaux): scene 0 (HIDE_HOTBAR, a yield) -> accepted, sequence 1
-- Sequence 1:
--   ACTOR1 (Aunillie): scene 1 (HIDE_HOTBAR)
--   emote 5 on ACTOR1 (Aunillie): scene 100 (HIDE_HOTBAR) -> sequence 2, notices [[0, []]]
-- Sequence 2:
--   ACTOR0 (Nicoliaux): scene 2 (HIDE_HOTBAR) -> sequence 3, notices [[1, []]]
-- Sequence 3:
--   ACTOR1 (Aunillie): scene 3 (HIDE_HOTBAR)
--   emote 18 on ACTOR1 (Aunillie): scene 98 (HIDE_HOTBAR) -> sequence 4, notices [[2, []]]
-- Sequence 4:
--   ACTOR0 (Nicoliaux): scene 4 (HIDE_HOTBAR) -> sequence 5, notices [[3, []]]
-- Sequence 5:
--   ACTOR1 (Aunillie): scene 5 (HIDE_HOTBAR)
--   emote 11 on ACTOR1 (Aunillie): scene 96 (HIDE_HOTBAR) -> sequence 255, notices [[4, []]]
-- (By hand, 2026-11-23: any other emote on ACTOR1 in sequences 1, 3 and 5 plays that sequence's empty EMOTENO scene.)
-- Sequence 255:
--   ACTOR0 (Nicoliaux): scene 6 (HIDE_HOTBAR, a yield) -> completed

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
        play(player, 1, HIDE_HOTBAR)
        return
    elseif sequence == 2 and BASE_ID == ACTOR0 then
        step = 4
        play(player, 2, HIDE_HOTBAR)
        return
    elseif sequence == 3 and BASE_ID == ACTOR1 then
        step = 5
        play(player, 3, HIDE_HOTBAR)
        return
    elseif sequence == 4 and BASE_ID == ACTOR0 then
        step = 7
        play(player, 4, HIDE_HOTBAR)
        return
    elseif sequence == 5 and BASE_ID == ACTOR1 then
        step = 8
        play(player, 5, HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        step = 10
        play(player, 6, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEmote(emote, target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == 1 and emote == 5 and BASE_ID == ACTOR1 then
        step = 3
        play(player, 100, HIDE_HOTBAR)
        return
    elseif sequence == 3 and emote == 18 and BASE_ID == ACTOR1 then
        step = 6
        play(player, 98, HIDE_HOTBAR)
        return
    elseif sequence == 5 and emote == 11 and BASE_ID == ACTOR1 then
        step = 9
        play(player, 96, HIDE_HOTBAR)
        return
    -- Any other emote: the sequence's SEQ_n_ACTOR1_EMOTENO, an empty scene (the client's; not seen on retail.)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, 99, HIDE_HOTBAR)
        return
    elseif sequence == 3 and BASE_ID == ACTOR1 then
        play(player, 97, HIDE_HOTBAR)
        return
    elseif sequence == 5 and BASE_ID == ACTOR1 then
        play(player, 95, HIDE_HOTBAR)
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
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif step == 2 and scene == 1 then
    elseif step == 3 and scene == 100 then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif step == 4 and scene == 2 then
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif step == 5 and scene == 3 then
    elseif step == 6 and scene == 98 then
        player:event_notice(EVENT_ID, 2, {})
        player:quest_sequence(EVENT_ID, 4)
    elseif step == 7 and scene == 4 then
        player:event_notice(EVENT_ID, 3, {})
        player:quest_sequence(EVENT_ID, 5)
    elseif step == 8 and scene == 5 then
    elseif step == 9 and scene == 96 then
        player:event_notice(EVENT_ID, 4, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif step == 10 and scene == 6 then
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
