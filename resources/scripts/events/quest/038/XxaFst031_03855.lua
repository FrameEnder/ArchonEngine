-- Quest: A Soldier's Breakfast (Lv. 4)
-- Checked against the client's quest script (scenes, key items, objectives.)
-- Sequence 0: the quest offer (scene 0), then ACTOR0 talks you through it (scene 1)
-- Sequence 1, in any order:
--   * Search EOBJECT0 (scene 2), finding ITEM0  -> UI8AL = 1, item in UI8BL, BitFlag8 1 hides the egg
--   * Defeat 3 ENEMY0 (a BNpcName)              -> counted in UI8BH
-- Sequence 2: report back to ACTOR0 (scene 4). ITEM0 is now held in UI8BH.
-- Sequence 255: hand ITEM0 to ACTOR1 (trade scene 5, then reward scene 6)
-- NOTE: The "searching" cast bar isn't played, the object's scene plays straight away.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_EOBJECT0 = 2
SCENE_ACTOR0_REPORT = 4
SCENE_ACTOR1_TRADE = 5
SCENE_ACTOR1_REWARD = 6
ENEMIES_TO_DEFEAT = 3

function onTalk(target, player)
    local sequence = questSequence(player)

    if sequence == nil and BASE_ID == ACTOR0 then
        questPlay(player, SCENE_OFFER)
    elseif sequence == 1 and BASE_ID == EOBJECT0 and not questBitFlag8(player, 1) then
        questPlay(player, SCENE_EOBJECT0)
    elseif sequence == 2 and BASE_ID == ACTOR0 then
        questPlay(player, SCENE_ACTOR0_REPORT)
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR1 then
        questPlay(player, SCENE_ACTOR1_TRADE)
    else
        player:finish_event()
    end
end

function checkCompletion(player)
    if questVar(player, "AL") >= 1 and questVar(player, "BH") >= ENEMIES_TO_DEFEAT then
        questAdvance(player, 2, { BH = 1 }) -- ITEM0 moves to UI8BH
    end
end

function onBNpcKill(name_id, layout_id, player)
    if questSequence(player) ~= 1 or name_id ~= ENEMY0 or questVar(player, "BH") >= ENEMIES_TO_DEFEAT then
        return
    end

    questIncrement(player, "BH")
    checkCompletion(player)
end

function onSceneFinished(scene, results, player)
    if scene == SCENE_OFFER then
        if results[1] == 1 then
            questPlay(player, SCENE_ACCEPTED)
            return true
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID, 1)
    elseif scene == SCENE_EOBJECT0 then
        if not questBitFlag8(player, 1) then
            setQuestBitFlag8(player, 1, true)
            setQuestVar(player, "AL", 1)
            setQuestVar(player, "BL", 1) -- found ITEM0
            checkCompletion(player)
        end
    elseif scene == SCENE_ACTOR0_REPORT then
        questAdvance(player, SEQ_FINISH, { BH = 1 })
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
