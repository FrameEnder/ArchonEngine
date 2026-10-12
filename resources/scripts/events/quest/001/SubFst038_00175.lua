-- Quest: Surveying the Damage (Lv. 4)
-- Checked against the client's quest script (scenes, key items, objectives.)
-- Sequence 0: accept from ACTOR0 (scene 0)
-- Sequence 1, in any order. Each object hides once searched (BitFlag8):
--   * Search EOBJECT0 (scene 1), finding ITEM0                 -> UI8AL = 1, item in UI8CH, flag 1
--   * Search EOBJECT1 and EOBJECT2 (scenes 2/3) for two ITEM1  -> UI8BH counts to 2, items in UI8CL, flags 2/3
--   * Search EOBJECT3 (scene 4), finding ITEM2                 -> UI8BL = 1, item in UI8DH, flag 4
-- Sequence 255: hand everything to ACTOR1 (trade scene 5, then reward scene 92). Items are held in UI8BH/BL/CH.
-- NOTE: The "searching" cast bar isn't played, the object's scene plays straight away.

SCENE_ACCEPT = 0
SCENE_ACTOR1_TRADE = 5
SCENE_ACTOR1_REWARD = 92

-- Base ID -> its scene, flag, objective variable and item variable.
SEARCHABLE = {
    [EOBJECT0] = { scene = 1, flag = 1, objective = "AL", item = "CH" },
    [EOBJECT1] = { scene = 2, flag = 2, objective = "BH", item = "CL" },
    [EOBJECT2] = { scene = 3, flag = 3, objective = "BH", item = "CL" },
    [EOBJECT3] = { scene = 4, flag = 4, objective = "BL", item = "DH" },
}

function onTalk(target, player)
    local sequence = questSequence(player)
    local object = SEARCHABLE[BASE_ID]

    if sequence == nil and BASE_ID == ACTOR0 then
        questPlay(player, SCENE_ACCEPT)
    elseif sequence == 1 and object ~= nil and not questBitFlag8(player, object.flag) then
        questPlay(player, object.scene)
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR1 then
        questPlay(player, SCENE_ACTOR1_TRADE)
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
    end

    for _, object in pairs(SEARCHABLE) do
        if scene == object.scene and not questBitFlag8(player, object.flag) then
            setQuestBitFlag8(player, object.flag, true)
            questIncrement(player, object.objective)
            questIncrement(player, object.item)

            if questVar(player, "AL") >= 1 and questVar(player, "BH") >= 2 and questVar(player, "BL") >= 1 then
                questAdvance(player, SEQ_FINISH, { BH = 1, BL = 2, CH = 1 })
            end
            return false
        end
    end

    if scene == SCENE_ACTOR1_TRADE then
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
