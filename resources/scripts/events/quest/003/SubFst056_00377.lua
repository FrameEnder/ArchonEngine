-- Quest: Don't Look Down (Lv. 10)
-- Checked against the client's quest script (scenes and objectives.)
-- Sequence 0: accept from ACTOR0 (scene 0)
-- Sequence 1: search EOBJECT0-3 (scenes 1-4), once each (BitFlag8 1-4), counted in UI8AL
-- Sequence 255: report to ACTOR1 (reward scene 5)
-- NOTE: The "searching" cast bar isn't played, the object's scene plays straight away.

SCENE_ACCEPT = 0
SCENE_ACTOR1_REWARD = 5
OBJECTS_TO_SEARCH = 4

-- Base ID -> its scene and flag
SEARCHABLE = {
    [EOBJECT0] = { scene = SEQ_1_EOBJECT0, flag = 1 },
    [EOBJECT1] = { scene = SEQ_1_EOBJECT1, flag = 2 },
    [EOBJECT2] = { scene = SEQ_1_EOBJECT2, flag = 3 },
    [EOBJECT3] = { scene = SEQ_1_EOBJECT3, flag = 4 },
}

function onTalk(target, player)
    local sequence = questSequence(player)
    local object = SEARCHABLE[BASE_ID]

    if sequence == nil and BASE_ID == ACTOR0 then
        questPlay(player, SCENE_ACCEPT)
    elseif sequence == 1 and object ~= nil and not questBitFlag8(player, object.flag) then
        questPlay(player, object.scene)
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR1 then
        questPlay(player, SCENE_ACTOR1_REWARD)
    else
        player:finish_event()
    end
end

function onSceneFinished(scene, results, player)
    if scene == SCENE_ACCEPT then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID, 1)
        end
        return false
    elseif scene == SCENE_ACTOR1_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
        end
        return false
    end

    for _, object in pairs(SEARCHABLE) do
        if scene == object.scene and not questBitFlag8(player, object.flag) then
            setQuestBitFlag8(player, object.flag, true)
            if questIncrement(player, "AL") >= OBJECTS_TO_SEARCH then
                questAdvance(player, SEQ_FINISH)
            end
        end
    end

    return false
end

onYield = questOnYield
onReturn = questOnReturn
