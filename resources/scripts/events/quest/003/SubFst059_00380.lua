-- Quest: Threat Level Elevated (Lv. 10)
-- Checked against the client's quest script (scenes, key items, objectives.)
-- A chain of deliveries. Each key item is held in a quest variable:
-- Sequence 0: accept from ACTOR0 (scene 0), receiving ITEM0 (UI8BH)
-- Sequence 1: hand ITEM0 to ACTOR1 (trade scene 1, then 100), receiving ITEM1 (UI8BL)
-- Sequence 2: hand ITEM1 to ACTOR2 (trade scene 2, then 98), receiving ITEM2 (UI8CH)
-- Sequence 255: hand ITEM2 to ACTOR3 (trade scene 3, then reward scene 96)

function onTalk(target, player)
    local sequence = questSequence(player)

    if sequence == nil and BASE_ID == ACTOR0 then
        questPlay(player, SEQ_0_ACTOR0)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        questPlay(player, SEQ_1_ACTOR1)
    elseif sequence == 2 and BASE_ID == ACTOR2 then
        questPlay(player, SEQ_2_ACTOR2)
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR3 then
        questPlay(player, SEQ_3_ACTOR3)
    else
        player:finish_event()
    end
end

-- Trade scene -> the scene played once the item is handed over
TRADES = {
    [SEQ_1_ACTOR1] = SEQ_1_ACTOR1_NPCTRADEOK,
    [SEQ_2_ACTOR2] = SEQ_2_ACTOR2_NPCTRADEOK,
    [SEQ_3_ACTOR3] = SEQ_3_ACTOR3_NPCTRADEOK,
}

function onSceneFinished(scene, results, player)
    if scene == SEQ_0_ACTOR0 then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID, 1)
            setQuestVar(player, "BH", 1) -- receive ITEM0
        end
    elseif TRADES[scene] ~= nil then
        if results[1] == 1 then
            questPlay(player, TRADES[scene])
            return true
        end
    elseif scene == SEQ_1_ACTOR1_NPCTRADEOK then
        questAdvance(player, 2, { BL = 1 }) -- receive ITEM1
    elseif scene == SEQ_2_ACTOR2_NPCTRADEOK then
        questAdvance(player, SEQ_FINISH, { CH = 1 }) -- receive ITEM2
    elseif scene == SEQ_3_ACTOR3_NPCTRADEOK then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
        end
    end

    return false
end

onYield = questOnYield
onReturn = questOnReturn
