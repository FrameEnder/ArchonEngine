-- Quest: In the Grim Darkness of the Forest (Lv. 10)
-- Checked against the client's quest script (scenes, key items, objectives.)
-- Sequence 0: accept from ACTOR0 (scene 0)
-- Sequence 1: speak with ACTOR1 (scene 1)
-- Sequence 2: search EOBJECT1 (scene 3), finding ITEM0 (held in UI8BH)
-- Sequence 255: hand ITEM0 to ACTOR1 (trade scene 5, then reward scene 94)
-- EOBJECT0 (the campfire) can be examined throughout, but its scenes are empty and it doesn't advance anything.
-- NOTE: The "searching" cast bar isn't played, the object's scene plays straight away.

function onTalk(target, player)
    local sequence = questSequence(player)

    if sequence == nil and BASE_ID == ACTOR0 then
        questPlay(player, SEQ_0_ACTOR0)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        questPlay(player, SEQ_1_ACTOR1)
    elseif sequence == 2 and BASE_ID == EOBJECT1 then
        questPlay(player, SEQ_2_EOBJECT1)
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR1 then
        questPlay(player, SEQ_3_ACTOR1)
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
        questAdvance(player, 2)
    elseif scene == SEQ_2_EOBJECT1 then
        questAdvance(player, SEQ_FINISH, { BH = 1 }) -- found ITEM0
    elseif scene == SEQ_3_ACTOR1 then
        if results[1] == 1 then
            questPlay(player, SEQ_3_ACTOR1_NPCTRADEOK)
            return true
        end
    elseif scene == SEQ_3_ACTOR1_NPCTRADEOK then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
        end
    end

    return false
end

onYield = questOnYield
onReturn = questOnReturn
