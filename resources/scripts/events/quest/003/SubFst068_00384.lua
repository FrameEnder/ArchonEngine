-- Quest: A Hearer Is Often Late (Lv. 10)
-- Sequence 0: accept from ACTOR0
-- Sequence 1: speak with ACTOR1
-- Sequence 255: report to ACTOR2 (ACTOR1 has a line here too)

function onTalk(target, player)
    local sequence = questSequence(player)

    if sequence == nil and BASE_ID == ACTOR0 then
        questPlay(player, SEQ_0_ACTOR0, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        questPlay(player, SEQ_1_ACTOR1, HIDE_HOTBAR)
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR2 then
        questPlay(player, SEQ_2_ACTOR2, HIDE_HOTBAR)
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR1 then
        questPlay(player, SEQ_2_ACTOR1, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onSceneFinished(scene, results, player)
    if scene == SEQ_0_ACTOR0 then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID, 1)
        end
    elseif scene == SEQ_1_ACTOR1 then
        setQuestSequence(player, SEQ_FINISH)
    elseif scene == SEQ_2_ACTOR2 then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
        end
    end

    return false
end

onYield = questOnYield
onReturn = questOnReturn
