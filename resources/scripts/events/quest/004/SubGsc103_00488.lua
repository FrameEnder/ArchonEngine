-- Quest: Scratch It Rich
--
-- Written from the client's quest script (kawari-questdump, 2026-10-29) and checked against Bob's Chronofoil capture
-- deaf7e28 (2026-10-29, the whole quest: its fixture is tools/questkit/fixtures/488.json). It opens the Mini Cactpot.
-- ACTOR0 (the Mini Cactpot broker) offers it and rewards it in one scene (scene 0: a yes/no, then QuestReward; a yield
-- returning 1 when taken): retail sent no AcceptQuest or UpdateQuest, only FinishQuest, then EventFinish. Saying no
-- cancels the scene.

SCENE_OFFER = 0

function onTalk(target, player)
    if player:get_quest_sequence(EVENT_ID) ~= nil or BASE_ID ~= ACTOR0 then
        player:finish_event()
        return
    end
    player:play_scene(SCENE_OFFER, HIDE_HOTBAR, {})
end

function sceneFinished(scene, results, player)
    if scene == SCENE_OFFER and results[1] == 1 then
        player:finish_quest(EVENT_ID, 0)
    end
    player:finish_event()
end

function onYield(scene, id, results, player)
    sceneFinished(scene, results, player)
end

function onReturn(scene, results, player)
    sceneFinished(scene, results, player)
end
