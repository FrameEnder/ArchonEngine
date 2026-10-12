-- Quest: Salvaging the Scene (Lv. 10)
-- Written from the client's quest script (scenes, key items, objectives.)
-- Sequence 0: the quest offer (scene 0), then ACTOR0 talks you through it (scene 1)
-- Sequence 1: search EOBJECT0 and EOBJECT1 (scenes 2 and 3), once each (BitFlag8 1-2), counted in UI8AL. Each one
--             holds an ITEM0, counted in UI8BH.
-- Sequence 255: hand the 2 ITEM0 to ACTOR1 (trade scene 23, then reward scene 24)
-- On retail a gigantoad (ENEMY0, a layout ID) ambushes you when you walk into EVENTRANGE0 near the wreck, announced
-- by scene 22. Kawari doesn't have quest event ranges, so it appears when you search the first piece of cargo instead.
-- Defeating it isn't required.
-- NOTE: The "searching" cast bar isn't played, the object's scene plays straight away.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_AMBUSH = 22
SCENE_ACTOR1_TRADE = 23
SCENE_ACTOR1_REWARD = 24
CARGO_TO_SEARCH = 2

-- Base ID -> its scene and flag
CARGO = {
    [EOBJECT0] = { scene = 2, flag = 1 },
    [EOBJECT1] = { scene = 3, flag = 2 },
}

function onTalk(target, player)
    local sequence = questSequence(player)
    local cargo = CARGO[BASE_ID]

    if sequence == nil and BASE_ID == ACTOR0 then
        questPlay(player, SCENE_OFFER)
    elseif sequence == 1 and cargo ~= nil and not questBitFlag8(player, cargo.flag) then
        questPlay(player, cargo.scene)
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR1 then
        questPlay(player, SCENE_ACTOR1_TRADE)
    else
        player:finish_event()
    end
end

function onSceneFinished(scene, results, player)
    if scene == SCENE_OFFER then
        if results[1] == 1 then
            questPlay(player, SCENE_ACCEPTED)
            return true
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID, 1)
    elseif scene == SCENE_ACTOR1_TRADE then
        if results[1] == 1 then
            questPlay(player, SCENE_ACTOR1_REWARD)
            return true
        end
    elseif scene == SCENE_ACTOR1_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
        end
    else
        for _, cargo in pairs(CARGO) do
            if scene == cargo.scene and not questBitFlag8(player, cargo.flag) then
                setQuestBitFlag8(player, cargo.flag, true)
                local found = questIncrement(player, "BH")
                local searched = questIncrement(player, "AL")

                if searched >= CARGO_TO_SEARCH then
                    questAdvance(player, SEQ_FINISH, { BH = found })
                elseif searched == 1 then
                    player:spawn_quest_bnpc(ENEMY0)
                    questPlay(player, SCENE_AMBUSH)
                    return true
                end
            end
        end
    end

    return false
end

onYield = questOnYield
onReturn = questOnReturn
