-- Quest: Eggs over Queasy (Lv. 4)
-- Checked against the client's quest script (scenes, key items, objectives.)
-- Sequence 0: the quest offer (scene 0), then ACTOR0 talks you through it (scene 1)
-- Sequence 1: defeat 3 ENEMY0 (a BNpcName), each dropping an ITEM0. Both are counted in UI8BH.
-- Sequence 255: hand the 3 ITEM0 to ACTOR1 (trade scene 2, then reward scene 3)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_ACTOR1_TRADE = 2
SCENE_ACTOR1_REWARD = 3
ENEMIES_TO_DEFEAT = 3

function onTalk(target, player)
    local sequence = questSequence(player)

    if sequence == nil and BASE_ID == ACTOR0 then
        questPlay(player, SCENE_OFFER)
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR1 then
        questPlay(player, SCENE_ACTOR1_TRADE)
    else
        player:finish_event()
    end
end

function onBNpcKill(name_id, layout_id, player)
    if questSequence(player) ~= 1 or name_id ~= ENEMY0 then
        return
    end

    local collected = questIncrement(player, "BH")
    if collected >= ENEMIES_TO_DEFEAT then
        questAdvance(player, SEQ_FINISH, { BH = collected })
    end
end

function onSceneFinished(scene, results, player)
    if scene == SCENE_OFFER then
        if results[1] == 1 then
            questPlay(player, SCENE_ACCEPTED)
            return true
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID, 1)
    elseif scene == SCENE_ACTOR1_TRADE then
        if results[1] == 1 then
            questPlay(player, SCENE_ACTOR1_REWARD)
            return true
        end
    elseif scene == SCENE_ACTOR1_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
        end
    end

    return false
end

onYield = questOnYield
onReturn = questOnReturn
