-- Quest: So You Want to Be a Jockey
--
-- Written from the client's quest script (kawari-questdump, 2026-10-29) and its listeners, and checked against Bob's
-- Chronofoil capture deaf7e28 (2026-10-29: accepted only; its fixture is tools/questkit/fixtures/436.json).
-- Sequence 0:   ACTOR0 (the race chocobo registrar, Chocobo Square) offers it (scene 0, a yield returning 1 when
--               accepted), then scene 1 (QuestAccepted): sequence 255.
-- Sequence 255: ACTOR1 (Katering) rewards it (scene 2: QuestReward then QuestCompleted, returning 1 and the item
--               chosen of the two optional ones.) Not captured: played as the other reward scenes.
-- ACTOR0's line at sequence 255 is scene 3.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_REWARD = 2
SCENE_ACTOR0_LINE = 3

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene)
    current_scene = scene
    player:play_scene(scene, HIDE_HOTBAR, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_REWARD)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_ACTOR0_LINE)
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
            play(player, SCENE_ACCEPTED)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID, results[2] or 0)
            player:finish_event()
            return
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
