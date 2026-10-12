-- Quest: Passing Muster (Lv. 4)
-- Sequence 0: accept from ACTOR0
-- Sequence 255: report back to ACTOR0

function onTalk(target, player)
    local sequence = questSequence(player)

    if BASE_ID ~= ACTOR0 then
        player:finish_event()
    elseif sequence == nil then
        questPlay(player, SEQ_0_ACTOR0, HIDE_HOTBAR)
    elseif sequence == SEQ_FINISH then
        questPlay(player, SEQ_1_ACTOR0, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onSceneFinished(scene, results, player)
    if scene == SEQ_0_ACTOR0 then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID, SEQ_FINISH)
        end
    elseif scene == SEQ_1_ACTOR0 then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
        end
    end

    return false
end

onYield = questOnYield
onReturn = questOnReturn
