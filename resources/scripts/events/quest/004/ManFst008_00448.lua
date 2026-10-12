-- Quest: Festive Endeavors (Lv. 14)
-- NOTE: The Quest sheet doesn't name this quest's scenes, so these come from Sapphire's script and are unverified for 7.56.
-- Sequence 0: accept from ACTOR0 (plus a follow-up scene)
-- Sequence 1: speak with ACTOR2
-- Sequence 2: speak with ACTOR3, receiving ITEM0 (UI8BH)
-- Sequence 255: hand ITEM0 to ACTOR2
-- ACTOR1 can take you into TERRITORYTYPE0 at any point.

SCENE_ACCEPT = 0
SCENE_ACCEPTED = 1
SCENE_ACTOR1_ENTER = 2
SCENE_ACTOR2 = 3
SCENE_ACTOR3 = 4
SCENE_ACTOR2_TRADE = 5
SCENE_ACTOR2_TRADE_OK = 6
SCENE_ACTOR2_TRADE_NO = 7

function onTalk(target, player)
    local sequence = questSequence(player)

    if sequence == nil and BASE_ID == ACTOR0 then
        questPlay(player, SCENE_ACCEPT, HIDE_HOTBAR)
    elseif BASE_ID == ACTOR1 then
        questPlay(player, SCENE_ACTOR1_ENTER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR2 then
        questPlay(player, SCENE_ACTOR2, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR3 then
        questPlay(player, SCENE_ACTOR3, HIDE_HOTBAR)
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR2 then
        questPlay(player, SCENE_ACTOR2_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onSceneFinished(scene, results, player)
    if scene == SCENE_ACCEPT then
        if results[1] == 1 then
            questPlay(player, SCENE_ACCEPTED, HIDE_HOTBAR)
            return true
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID, 1)
    elseif scene == SCENE_ACTOR1_ENTER then
        if results[1] == 1 then
            player:change_territory_pop_range(TERRITORYTYPE0, POPRANGE0)
        end
    elseif scene == SCENE_ACTOR2 then
        setQuestSequence(player, 2)
    elseif scene == SCENE_ACTOR3 then
        setQuestVar(player, "BH", 1) -- receive ITEM0
        setQuestSequence(player, SEQ_FINISH)
    elseif scene == SCENE_ACTOR2_TRADE then
        if results[1] == 1 then
            questPlay(player, SCENE_ACTOR2_TRADE_OK, HIDE_HOTBAR)
        else
            questPlay(player, SCENE_ACTOR2_TRADE_NO, HIDE_HOTBAR)
        end
        return true
    elseif scene == SCENE_ACTOR2_TRADE_OK then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
        end
    end

    return false
end

onYield = questOnYield
onReturn = questOnReturn
