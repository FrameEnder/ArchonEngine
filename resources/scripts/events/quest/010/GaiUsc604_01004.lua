-- Quest: Fool Me Twice
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Connor's Chronofoil capture e834e436 (2026-11-07/09) (fixture tools/questkit/fixtures/1004.json):
-- Sequence 0:
--   ACTOR0 (Glaumunt): scene 0 (HIDE_HOTBAR, a yield), 1 (HIDE_HOTBAR) -> accepted, sequence 1, vars 00 10 00 00 00 00
-- Sequence 1:
--   emote 59 on ACTOR1 (imperial centurion): scene 3 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 2, notices [[0, []]]
-- Sequence 2:
--   key item 2000771 on EOBJECT0 (destination): scene 12 (NO_DEFAULT_CAMERA | HIDE_HOTBAR, a yield), into the quest battle, zone 308
-- Sequence 255:
--   ACTOR9 (Slafborn): scene 21 (HIDE_HOTBAR, a yield), zone 156
--   ACTOR8 (Cid): scene 20 (HIDE_HOTBAR, a yield) -> completed

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
    elseif sequence == 255 and BASE_ID == ACTOR9 then
        step = 4
        play(player, 21, HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR8 then
        step = 5
        play(player, 20, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEventItem(item, target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == 2 and item == ITEM0 and BASE_ID == EOBJECT0 then
        step = 3
        play(player, 12, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEmote(emote, target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == 1 and emote == 59 and BASE_ID == ACTOR1 then
        step = 2
        play(player, 3, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
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
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8B
        player:quest_sequence(EVENT_ID, 1)
    elseif step == 2 and scene == 3 then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif step == 3 and scene == 12 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        player:finish_event()
        player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
        return
    elseif step == 4 and scene == 21 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        player:finish_event()
        -- Where retail put the player (ActorSetPos; no pop range of the quest's.)
        player:change_territory(TERRITORYTYPE1, { x = 0.62, y = -158.58, z = 10.42 }, 3.118)
        return
    elseif step == 5 and scene == 20 then
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
