-- Quest: Storms on the Horizon (Ul'dah)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-20) and retail (Bob's Chronofoil capture of
-- 2026-10-18, questkit replay 334):
-- Sequence 0:   ACTOR0 (Nunuzofu) offers it (scene 0, a yield: QuestOffer, 1 when accepted); accepting goes straight
--               to sequence 255.
-- Sequence 255: ACTOR1 (Raffe) rewards it (scene 1, a yield: QuestReward, 1 and the chosen item.)

SCENE_OFFER = 0
SCENE_FINISH = 1

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
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_FINISH, HIDE_HOTBAR)
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
            player:accept_quest(EVENT_ID)
        end
    elseif scene == SCENE_FINISH then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID, results[2])
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
