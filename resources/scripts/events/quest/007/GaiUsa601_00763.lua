-- Quest: Meeting with the Resistance
--
-- Written from the client's quest script (kawari-questdump, 2026-10-30) and checked against Bob's Chronofoil capture
-- 3a705e18 (2026-10-30, the whole quest: its fixture is tools/questkit/fixtures/763.json):
-- Sequence 0:   ACTOR0 (Minfilia) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1, where the
--               client accepts it: sequence 1.
-- Sequence 1:   ACTOR1 (Haribehrt) talks (scene 2): notice 0, sequence 255.
-- Sequence 255: ACTOR2 (Albreda) rewards it (scene 3, a yield: QuestReward, 1.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_HARIBEHRT = 2
SCENE_REWARD = 3

NOTICE_HARIBEHRT = 0

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_HARIBEHRT, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            play(player, SCENE_ACCEPTED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_HARIBEHRT then
        player:event_notice(EVENT_ID, NOTICE_HARIBEHRT, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
        end
    end

    player:finish_event()
end

function onYield(scene, id, results, player)
    sceneFinished(scene, results, player)
end

function onReturn(scene, results, player)
    sceneFinished(scene, results, player)
end
