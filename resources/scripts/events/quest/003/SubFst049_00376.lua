-- Quest: You Shall Not Trespass (Lv. 10)
-- NOTE: The Quest sheet doesn't name this quest's scenes, so these come from Sapphire's script and are unverified for 7.56.
-- Sequence 0: accept from ACTOR0
-- Sequence 1: speak with ACTOR1
-- Sequence 2: defeat 4 ENEMY0 (a BNpcName), counted in UI8AL
-- Sequence 255: report to ACTOR1

SCENE_ACCEPT = 0
SCENE_ACTOR1 = 1
SCENE_ACTOR1_REMINDER = 2
SCENE_ACTOR1_FINISH = 3
ENEMIES_TO_DEFEAT = 4

function onTalk(target, player)
    local sequence = questSequence(player)

    if sequence == nil and BASE_ID == ACTOR0 then
        questPlay(player, SCENE_ACCEPT, HIDE_HOTBAR)
    elseif BASE_ID == ACTOR1 and sequence == 1 then
        questPlay(player, SCENE_ACTOR1, HIDE_HOTBAR)
    elseif BASE_ID == ACTOR1 and sequence == 2 then
        questPlay(player, SCENE_ACTOR1_REMINDER, HIDE_HOTBAR)
    elseif BASE_ID == ACTOR1 and sequence == SEQ_FINISH then
        questPlay(player, SCENE_ACTOR1_FINISH, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onBNpcKill(name_id, layout_id, player)
    if questSequence(player) ~= 2 or name_id ~= ENEMY0 then
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
    if scene == SCENE_ACCEPT then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID, 1)
        end
    elseif scene == SCENE_ACTOR1 then
        setQuestSequence(player, 2)
    elseif scene == SCENE_ACTOR1_FINISH then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
        end
    end

    return false
end

onYield = questOnYield
onReturn = questOnReturn
