-- Quest: We Come in Peace
--
-- Written from the client's quest script (kawari-questdump, 2026-10-30) and checked against Bob's Chronofoil capture
-- 5fcc970c (2026-10-30, the whole quest: its fixture is tools/questkit/fixtures/3856.json):
-- Sequence 0:   ACTOR0 (Vorsaile Heuloix) offers it (scene 0, a yield: QuestOffer, 1 when accepted; the client turns
--               it down itself while QST_ACCEPT_CHECK, the old We Come in Peace (708), is accepted), then the
--               cutscene (scene 1, 0xF8402EFB), where the client accepts it: sequence 1.
-- Sequence 1:   ACTOR1 (Mitainie) talks (scene 2): notice 0 and sequence 255 (todo 0 is "UI8AL >= 1", but retail
--               went straight to 255 with the variables 0.)
-- Sequence 255: ACTOR2 (Amelain) rewards it (scene 3, a yield: QuestReward, 1; gil only). Retail then completed the
--               old We Come in Peace (708) too, with no rewards: FinishQuest 3856, then 708.

SCENE_OFFER = 0
SCENE_CUTSCENE = 1
SCENE_MITAINIE = 2
SCENE_REWARD = 3

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB on retail
NOTICE_MITAINIE = 0
-- The quest this one replaced, completed with it (retail's FinishQuest 708 right after 3856's.)
QUEST_ALSO_COMPLETED = 708

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
        play(player, SCENE_MITAINIE, HIDE_HOTBAR)
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
            play(player, SCENE_CUTSCENE, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_CUTSCENE then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_MITAINIE then
        player:event_notice(EVENT_ID, NOTICE_MITAINIE, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
            player:mark_quest_completed(QUEST_ALSO_COMPLETED)
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
