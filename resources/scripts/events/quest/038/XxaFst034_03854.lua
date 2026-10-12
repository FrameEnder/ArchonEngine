-- Quest: Eggs over Queasy (Lv. 4)
-- NOTE: This quest was added in patch 6.1. Neither the Quest sheet nor Sapphire names its scenes, so the numbers below
-- follow the usual order (accept = 0, then each NPC in turn) and are a best guess. Watch the server log for the real ones.
-- Sequence 0: accept from ACTOR0
-- Sequence 1: defeat 3 ENEMY0 (a BNpcName), collecting ITEM0, counted in UI8AL
-- Sequence 255: report to ACTOR1

SCENE_ACCEPT = 0
SCENE_ACTOR1_FINISH = 1
ENEMIES_TO_DEFEAT = 3

function onTalk(target, player)
    local sequence = questSequence(player)

    if sequence == nil and BASE_ID == ACTOR0 then
        questPlay(player, SCENE_ACCEPT, HIDE_HOTBAR)
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR1 then
        questPlay(player, SCENE_ACTOR1_FINISH, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onBNpcKill(name_id, layout_id, player)
    if questSequence(player) ~= 1 or name_id ~= ENEMY0 then
        return
    end

    local defeated = questVar(player, "AL") + 1
    setQuestVar(player, "AL", defeated)
    if defeated >= ENEMIES_TO_DEFEAT then
        setQuestSequence(player, SEQ_FINISH)
    end
end

function onSceneFinished(scene, results, player)
    if scene == SCENE_ACCEPT then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID, 1)
        end
    elseif scene == SCENE_ACTOR1_FINISH then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
        end
    end

    return false
end

onYield = questOnYield
onReturn = questOnReturn
