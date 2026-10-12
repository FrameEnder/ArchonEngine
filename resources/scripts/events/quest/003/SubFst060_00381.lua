-- Quest: Migrant Marauders (Lv. 10)
-- Sequence 0: accept from ACTOR0
-- Sequence 1: search EOBJECT0, which brings out ENEMY0 and ENEMY1 (layout IDs.) Defeat both, counted in UI8AL.
-- Sequence 255: report to ACTOR1
-- NOTE: The "searching" cast bar isn't played, the scene after it plays straight away.

ENEMIES_TO_DEFEAT = 2

function onTalk(target, player)
    local sequence = questSequence(player)

    if sequence == nil and BASE_ID == ACTOR0 then
        questPlay(player, SEQ_0_ACTOR0, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == EOBJECT0 then
        questPlay(player, SEQ_1_EOBJECT0, HIDE_HOTBAR)
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR1 then
        questPlay(player, SEQ_2_ACTOR1, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onBNpcKill(name_id, layout_id, player)
    if questSequence(player) ~= 1 or (layout_id ~= ENEMY0 and layout_id ~= ENEMY1) then
        return
    end

    local defeated = questVar(player, "AL") + 1
    if defeated >= ENEMIES_TO_DEFEAT then
        setQuestVar(player, "AL", 0)
        setQuestSequence(player, SEQ_FINISH)
    else
        setQuestVar(player, "AL", defeated)
    end
end

function onSceneFinished(scene, results, player)
    if scene == SEQ_0_ACTOR0 then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID, 1)
        end
    elseif scene == SEQ_1_EOBJECT0 then
        if results[1] ~= 0 then
            questPlay(player, SEQ_1_EOBJECT0_EVENTACTIONOK, HIDE_HOTBAR)
        else
            questPlay(player, SEQ_1_EOBJECT0_EVENTACTIONNO, HIDE_HOTBAR)
        end
        return true
    elseif scene == SEQ_1_EOBJECT0_EVENTACTIONOK then
        -- Only bring the marauders out once
        if not questBitFlag8(player, 1) then
            setQuestBitFlag8(player, 1, true)
            player:spawn_quest_bnpc(ENEMY0)
            player:spawn_quest_bnpc(ENEMY1)
        end
    elseif scene == SEQ_2_ACTOR1 then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
        end
    end

    return false
end

onYield = questOnYield
onReturn = questOnReturn
