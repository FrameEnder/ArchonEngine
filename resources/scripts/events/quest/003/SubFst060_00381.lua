-- Quest: Migrant Marauders (Lv. 10)
-- Checked against the client's quest script (scenes and objectives.)
-- Sequence 0: accept from ACTOR0 (scene 0)
-- Sequence 1: search EOBJECT0 (scene 1, BitFlag8 1), which brings out ENEMY0 and ENEMY1 (layout IDs, announced by
--             scene 100.) Defeating ENEMY0 sets UI8AL and ENEMY1 sets UI8BH; the objective is their sum reaching 2.
-- Sequence 255: report to ACTOR1 (reward scene 2)
-- NOTE: The "searching" cast bar isn't played, the object's scene plays straight away.

SCENE_ANNOUNCE = 100

function onTalk(target, player)
    local sequence = questSequence(player)

    if sequence == nil and BASE_ID == ACTOR0 then
        questPlay(player, SEQ_0_ACTOR0)
    elseif sequence == 1 and BASE_ID == EOBJECT0 and not questBitFlag8(player, 1) then
        questPlay(player, SEQ_1_EOBJECT0)
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR1 then
        questPlay(player, SEQ_2_ACTOR1)
    else
        player:finish_event()
    end
end

function onBNpcKill(name_id, layout_id, player)
    if questSequence(player) ~= 1 then
        return
    end

    if layout_id == ENEMY0 then
        setQuestVar(player, "AL", 1)
    elseif layout_id == ENEMY1 then
        setQuestVar(player, "BH", 1)
    else
        return
    end

    if questVar(player, "AL") + questVar(player, "BH") >= 2 then
        questAdvance(player, SEQ_FINISH)
    end
end

function onSceneFinished(scene, results, player)
    if scene == SEQ_0_ACTOR0 then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID, 1)
        end
    elseif scene == SEQ_1_EOBJECT0 then
        -- Only bring the marauders out once
        if not questBitFlag8(player, 1) then
            setQuestBitFlag8(player, 1, true)
            player:spawn_quest_bnpc(ENEMY0)
            player:spawn_quest_bnpc(ENEMY1)
            questPlay(player, SCENE_ANNOUNCE)
            return true
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
