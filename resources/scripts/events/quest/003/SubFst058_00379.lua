-- Quest: In the Grim Darkness of the Forest (Lv. 10)
-- Sequence 0: accept from ACTOR0
-- Sequence 1: speak with ACTOR1
-- Sequence 2: search EOBJECT1, finding ITEM0 (held in UI8BH, unverified)
-- Sequence 255: hand ITEM0 to ACTOR1
-- EOBJECT0 can be examined during every sequence, but doesn't advance anything.
-- NOTE: The "searching" cast bar isn't played, the scene after it plays straight away.

function onTalk(target, player)
    local sequence = questSequence(player)

    if sequence == nil and BASE_ID == ACTOR0 then
        questPlay(player, SEQ_0_ACTOR0, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        questPlay(player, SEQ_1_ACTOR1, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == EOBJECT0 then
        questPlay(player, SEQ_1_EOBJECT0, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == EOBJECT1 then
        questPlay(player, SEQ_2_EOBJECT1, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == EOBJECT0 then
        questPlay(player, SEQ_2_EOBJECT0, HIDE_HOTBAR)
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR1 then
        questPlay(player, SEQ_3_ACTOR1, HIDE_HOTBAR)
    elseif sequence == SEQ_FINISH and BASE_ID == EOBJECT0 then
        questPlay(player, SEQ_3_EOBJECT0, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- Prompt scene -> the scenes played when the player agrees or declines.
FOLLOW_UPS = {
    [SEQ_1_EOBJECT0] = { SEQ_1_EOBJECT0_EVENTACTIONOK, SEQ_1_EOBJECT0_EVENTACTIONNO },
    [SEQ_2_EOBJECT1] = { SEQ_2_EOBJECT1_EVENTACTIONOK, SEQ_2_EOBJECT1_EVENTACTIONNO },
    [SEQ_2_EOBJECT0] = { SEQ_2_EOBJECT0_EVENTACTIONOK, SEQ_2_EOBJECT0_EVENTACTIONNO },
    [SEQ_3_ACTOR1] = { SEQ_3_ACTOR1_NPCTRADEOK, SEQ_3_ACTOR1_NPCTRADENO },
    [SEQ_3_EOBJECT0] = { SEQ_3_EOBJECT0_EVENTACTIONOK, SEQ_3_EOBJECT0_EVENTACTIONNO },
}

function onSceneFinished(scene, results, player)
    if scene == SEQ_0_ACTOR0 then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID, 1)
        end
    elseif scene == SEQ_1_ACTOR1 then
        setQuestSequence(player, 2)
    elseif FOLLOW_UPS[scene] ~= nil then
        local agreed = results[1] ~= 0
        if scene == SEQ_3_ACTOR1 then
            agreed = results[1] == 1 -- trades must be explicitly accepted
        end
        if agreed then
            questPlay(player, FOLLOW_UPS[scene][1], HIDE_HOTBAR)
        else
            questPlay(player, FOLLOW_UPS[scene][2], HIDE_HOTBAR)
        end
        return true
    elseif scene == SEQ_2_EOBJECT1_EVENTACTIONOK then
        setQuestVar(player, "BH", 1) -- found ITEM0
        setQuestSequence(player, SEQ_FINISH)
    elseif scene == SEQ_3_ACTOR1_NPCTRADEOK then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
        end
    end

    return false
end

onYield = questOnYield
onReturn = questOnReturn
