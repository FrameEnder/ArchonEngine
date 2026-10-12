-- Quest: Renewing the Covenant (Lv. 14)
-- Checked against the client's quest script (scenes and objectives.)
-- Sequence 0: accept from ACTOR0 (offer scene 0, then scene 1)
-- Sequence 1: ACTOR2 gives you RITEM0 (a hat, scene 6), then speak with ACTOR1 while wearing it (scene 2, then 3 and the
--             ceremony cutscene 4). Without the hat on, ACTOR1 plays scene 5 instead.
-- Sequence 255: report to ACTOR3 (reward scene 7). ACTOR4 can take you into TERRITORYTYPE0 (scene 8).

SCENE_ACCEPT = 0
SCENE_ACCEPTED = 1
SCENE_ACTOR1 = 2
SCENE_ACTOR1_WEARING = 3
SCENE_ACTOR1_CUTSCENE = 4
SCENE_ACTOR1_NOT_WEARING = 5
SCENE_ACTOR2 = 6
SCENE_ACTOR3_FINISH = 7
SCENE_ACTOR4 = 8

function onTalk(target, player)
    local sequence = questSequence(player)

    if sequence == nil and BASE_ID == ACTOR0 then
        questPlay(player, SCENE_ACCEPT, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        questPlay(player, SCENE_ACTOR1, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR2 then
        questPlay(player, SCENE_ACTOR2, HIDE_HOTBAR)
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR3 then
        questPlay(player, SCENE_ACTOR3_FINISH, HIDE_HOTBAR)
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR4 then
        questPlay(player, SCENE_ACTOR4, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- Whether RITEM0 is equipped. If the equipment can't be read, assume it is so the quest can't get stuck.
function isWearingHat(player)
    local ok, head = pcall(function()
        return player.inventory.equipped.head.id
    end)
    return not ok or head == RITEM0
end

function onSceneFinished(scene, results, player)
    if scene == SCENE_ACCEPT then
        if results[1] == 1 then
            questPlay(player, SCENE_ACCEPTED, HIDE_HOTBAR)
            return true
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID, 1)
    elseif scene == SCENE_ACTOR2 then
        player:add_item(RITEM0, 1)
    elseif scene == SCENE_ACTOR1 then
        if isWearingHat(player) then
            questPlay(player, SCENE_ACTOR1_WEARING, HIDE_HOTBAR)
        else
            questPlay(player, SCENE_ACTOR1_NOT_WEARING, HIDE_HOTBAR)
        end
        return true
    elseif scene == SCENE_ACTOR1_WEARING then
        questPlay(player, SCENE_ACTOR1_CUTSCENE, QUEST_CUTSCENE_FLAGS)
        return true
    elseif scene == SCENE_ACTOR1_CUTSCENE then
        questAdvance(player, SEQ_FINISH)
    elseif scene == SCENE_ACTOR3_FINISH then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
        end
    elseif scene == SCENE_ACTOR4 then
        if results[1] == 1 then
            player:change_territory_pop_range(TERRITORYTYPE0, POPRANGE0)
        end
    end

    return false
end

onYield = questOnYield
onReturn = questOnReturn
