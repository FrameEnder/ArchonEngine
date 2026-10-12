-- Quest: Don't Look Down (Lv. 10)
-- Sequence 0: accept from ACTOR0
-- Sequence 1: search EOBJECT0-3, once each (BitFlag8 1-4), counted in UI8AL
-- Sequence 255: report to ACTOR1
-- NOTE: The "searching" cast bar isn't played, the scene after it plays straight away.

OBJECTS_TO_SEARCH = 4

-- For each searchable object: its flag, prompt scene, and the scenes played afterwards.
function searchable(player)
    return {
        [EOBJECT0] = { flag = 1, scene = SEQ_1_EOBJECT0, ok = SEQ_1_EOBJECT0_EVENTACTIONOK, no = SEQ_1_EOBJECT0_EVENTACTIONNO },
        [EOBJECT1] = { flag = 2, scene = SEQ_1_EOBJECT1, ok = SEQ_1_EOBJECT1_EVENTACTIONOK, no = SEQ_1_EOBJECT1_EVENTACTIONNO },
        [EOBJECT2] = { flag = 3, scene = SEQ_1_EOBJECT2, ok = SEQ_1_EOBJECT2_EVENTACTIONOK, no = SEQ_1_EOBJECT2_EVENTACTIONNO },
        [EOBJECT3] = { flag = 4, scene = SEQ_1_EOBJECT3, ok = SEQ_1_EOBJECT3_EVENTACTIONOK, no = SEQ_1_EOBJECT3_EVENTACTIONNO },
    }
end

function onTalk(target, player)
    local sequence = questSequence(player)
    local object = searchable(player)[BASE_ID]

    if sequence == nil and BASE_ID == ACTOR0 then
        questPlay(player, SEQ_0_ACTOR0, HIDE_HOTBAR)
    elseif sequence == 1 and object ~= nil and not questBitFlag8(player, object.flag) then
        questPlay(player, object.scene, HIDE_HOTBAR)
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR1 then
        questPlay(player, SEQ_2_ACTOR1, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onSceneFinished(scene, results, player)
    if scene == SEQ_0_ACTOR0 then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID, 1)
        end
        return false
    end

    for _, object in pairs(searchable(player)) do
        if scene == object.scene then
            if results[1] ~= 0 then
                questPlay(player, object.ok, HIDE_HOTBAR)
            else
                questPlay(player, object.no, HIDE_HOTBAR)
            end
            return true
        elseif scene == object.ok then
            if not questBitFlag8(player, object.flag) then
                setQuestBitFlag8(player, object.flag, true)
                local searched = questVar(player, "AL") + 1
                if searched >= OBJECTS_TO_SEARCH then
                    setQuestVar(player, "AL", 0)
                    setQuestSequence(player, SEQ_FINISH)
                else
                    setQuestVar(player, "AL", searched)
                end
            end
            return false
        end
    end

    if scene == SEQ_2_ACTOR1 then
        -- Hand over what was found
        if results[1] == 1 then
            questPlay(player, SEQ_2_ACTOR1_NPCTRADEOK, HIDE_HOTBAR)
        else
            questPlay(player, SEQ_2_ACTOR1_NPCTRADENO, HIDE_HOTBAR)
        end
        return true
    elseif scene == SEQ_2_ACTOR1_NPCTRADEOK then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
        end
    end

    return false
end

onYield = questOnYield
onReturn = questOnReturn
