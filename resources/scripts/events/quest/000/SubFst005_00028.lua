-- Quest: To the Bannock (Lv. 4)
-- Sequence 0: accept from ACTOR0
-- Sequence 255: report to ACTOR1

function onTalk(target, player)
    local sequence = questSequence(player)

    if sequence == nil and BASE_ID == ACTOR0 then
        questPlay(player, SEQ_0_ACTOR0, HIDE_HOTBAR)
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR1 then
        questPlay(player, SEQ_1_ACTOR1, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onSceneFinished(scene, results, player)
    if scene == SEQ_0_ACTOR0 then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID, SEQ_FINISH)
        end
    elseif scene == SEQ_1_ACTOR1 then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
        end
    end

    return false
end

onYield = questOnYield
onReturn = questOnReturn
