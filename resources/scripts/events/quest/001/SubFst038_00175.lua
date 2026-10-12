-- Quest: Surveying the Damage (Lv. 4)
-- NOTE: The Quest sheet doesn't name this quest's scenes, so these come from Sapphire's script and are unverified for 7.56.
-- Sequence 0: accept from ACTOR0
-- Sequence 1, in any order:
--   * Search EOBJECT0, finding ITEM0          -> UI8AL = 1, UI8CH = 1
--   * Search EOBJECT1 and EOBJECT2 for ITEM1  -> UI8BH counts to 2, UI8CL holds the items, BitFlag8 2/3 hide them
--   * Search EOBJECT3, finding ITEM2          -> UI8BL = 1, UI8DH = 1
-- Sequence 255: hand everything to ACTOR1
-- NOTE: The "searching" cast bar isn't played, the scene after it plays straight away.

SCENE_ACCEPT = 0
SCENE_EOBJECT0 = 1
SCENE_EOBJECT1 = 2
SCENE_EOBJECT2 = 3
SCENE_EOBJECT3 = 4
SCENE_ACTOR1_TRADE = 5
SCENE_ACTOR1_TRADE_NO = 91
SCENE_ACTOR1_TRADE_OK = 92

function onTalk(target, player)
    local sequence = questSequence(player)

    if sequence == nil and BASE_ID == ACTOR0 then
        questPlay(player, SCENE_ACCEPT, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == EOBJECT0 and questVar(player, "AL") == 0 then
        questPlay(player, SCENE_EOBJECT0, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == EOBJECT1 and not questBitFlag8(player, 2) then
        questPlay(player, SCENE_EOBJECT1, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == EOBJECT2 and not questBitFlag8(player, 3) then
        questPlay(player, SCENE_EOBJECT2, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == EOBJECT3 and questVar(player, "BL") == 0 then
        questPlay(player, SCENE_EOBJECT3, HIDE_HOTBAR)
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR1 then
        questPlay(player, SCENE_ACTOR1_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function checkCompletion(player)
    if questVar(player, "AL") >= 1 and questVar(player, "BH") >= 2 and questVar(player, "BL") >= 1 then
        setQuestSequence(player, SEQ_FINISH)
    end
end

function foundItem1(player, flag)
    setQuestVar(player, "BH", questVar(player, "BH") + 1)
    setQuestVar(player, "CL", questVar(player, "CL") + 1)
    setQuestBitFlag8(player, flag, true) -- makes the object fade out
    checkCompletion(player)
end

function onSceneFinished(scene, results, player)
    if scene == SCENE_ACCEPT then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID, 1)
        end
    elseif scene == SCENE_EOBJECT0 then
        setQuestVar(player, "AL", 1)
        setQuestVar(player, "CH", 1)
        checkCompletion(player)
    elseif scene == SCENE_EOBJECT1 then
        foundItem1(player, 2)
    elseif scene == SCENE_EOBJECT2 then
        foundItem1(player, 3)
    elseif scene == SCENE_EOBJECT3 then
        setQuestVar(player, "BL", 1)
        setQuestVar(player, "DH", 1)
        checkCompletion(player)
    elseif scene == SCENE_ACTOR1_TRADE then
        if results[1] == 1 then
            questPlay(player, SCENE_ACTOR1_TRADE_OK, HIDE_HOTBAR)
        else
            questPlay(player, SCENE_ACTOR1_TRADE_NO, HIDE_HOTBAR)
        end
        return true
    elseif scene == SCENE_ACTOR1_TRADE_OK then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
        end
    end

    return false
end

onYield = questOnYield
onReturn = questOnReturn
