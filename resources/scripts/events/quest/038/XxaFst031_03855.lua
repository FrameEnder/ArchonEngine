-- Quest: A Soldier's Breakfast (Lv. 4)
-- NOTE: This quest was added in patch 6.1. Neither the Quest sheet nor Sapphire names its scenes, so the numbers below
-- follow the usual order (accept = 0, then each NPC/object in turn) and are a best guess. Watch the server log for the real ones.
-- Sequence 0: accept from ACTOR0
-- Sequence 1, in any order:
--   * Search EOBJECT0, finding ITEM0       -> UI8AL = 1
--   * Defeat 3 ENEMY0 (a BNpcName)         -> counted in UI8BH
-- Sequence 2: report back to ACTOR0
-- Sequence 255: report to ACTOR1
-- NOTE: The "searching" cast bar isn't played, the scene after it plays straight away.

SCENE_ACCEPT = 0
SCENE_EOBJECT0 = 1
SCENE_ACTOR0_REPORT = 2
SCENE_ACTOR1_FINISH = 3
ENEMIES_TO_DEFEAT = 3

function onTalk(target, player)
    local sequence = questSequence(player)

    if sequence == nil and BASE_ID == ACTOR0 then
        questPlay(player, SCENE_ACCEPT, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == EOBJECT0 and questVar(player, "AL") == 0 then
        questPlay(player, SCENE_EOBJECT0, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR0 then
        questPlay(player, SCENE_ACTOR0_REPORT, HIDE_HOTBAR)
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR1 then
        questPlay(player, SCENE_ACTOR1_FINISH, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function checkCompletion(player)
    if questVar(player, "AL") >= 1 and questVar(player, "BH") >= ENEMIES_TO_DEFEAT then
        setQuestSequence(player, 2)
    end
end

function onBNpcKill(name_id, layout_id, player)
    if questSequence(player) ~= 1 or name_id ~= ENEMY0 or questVar(player, "BH") >= ENEMIES_TO_DEFEAT then
        return
    end

    setQuestVar(player, "BH", questVar(player, "BH") + 1)
    checkCompletion(player)
end

function onSceneFinished(scene, results, player)
    if scene == SCENE_ACCEPT then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID, 1)
        end
    elseif scene == SCENE_EOBJECT0 then
        setQuestVar(player, "AL", 1)
        checkCompletion(player)
    elseif scene == SCENE_ACTOR0_REPORT then
        setQuestSequence(player, SEQ_FINISH)
    elseif scene == SCENE_ACTOR1_FINISH then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
        end
    end

    return false
end

onYield = questOnYield
onReturn = questOnReturn
