-- Quest: Threat Level Elevated (Lv. 10)
-- A chain of deliveries. Each key item is held in a quest variable:
-- Sequence 0: accept from ACTOR0, receiving ITEM0 (UI8BH)
-- Sequence 1: hand ITEM0 to ACTOR1, receiving ITEM1 (UI8BL)
-- Sequence 2: hand ITEM1 to ACTOR2, receiving ITEM2 (UI8CH)
-- Sequence 255: hand ITEM2 to ACTOR3

function onTalk(target, player)
    local sequence = questSequence(player)

    if sequence == nil and BASE_ID == ACTOR0 then
        questPlay(player, SEQ_0_ACTOR0, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        questPlay(player, SEQ_1_ACTOR1, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR2 then
        questPlay(player, SEQ_2_ACTOR2, HIDE_HOTBAR)
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR3 then
        questPlay(player, SEQ_3_ACTOR3, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- Plays the "handed over" or "didn't hand over" scene for a trade prompt.
function trade(player, results, ok_scene, no_scene)
    if results[1] == 1 then
        questPlay(player, ok_scene, HIDE_HOTBAR)
    else
        questPlay(player, no_scene, HIDE_HOTBAR)
    end
    return true
end

function onSceneFinished(scene, results, player)
    if scene == SEQ_0_ACTOR0 then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID, 1)
            setQuestVar(player, "BH", 1) -- receive ITEM0
        end
    elseif scene == SEQ_1_ACTOR1 then
        return trade(player, results, SEQ_1_ACTOR1_NPCTRADEOK, SEQ_1_ACTOR1_NPCTRADENO)
    elseif scene == SEQ_1_ACTOR1_NPCTRADEOK then
        setQuestVar(player, "BH", 0)
        setQuestVar(player, "BL", 1) -- receive ITEM1
        setQuestSequence(player, 2)
    elseif scene == SEQ_2_ACTOR2 then
        return trade(player, results, SEQ_2_ACTOR2_NPCTRADEOK, SEQ_2_ACTOR2_NPCTRADENO)
    elseif scene == SEQ_2_ACTOR2_NPCTRADEOK then
        setQuestVar(player, "BL", 0)
        setQuestVar(player, "CH", 1) -- receive ITEM2
        setQuestSequence(player, SEQ_FINISH)
    elseif scene == SEQ_3_ACTOR3 then
        return trade(player, results, SEQ_3_ACTOR3_NPCTRADEOK, SEQ_3_ACTOR3_NPCTRADENO)
    elseif scene == SEQ_3_ACTOR3_NPCTRADEOK then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
        end
    end

    return false
end

onYield = questOnYield
onReturn = questOnReturn
