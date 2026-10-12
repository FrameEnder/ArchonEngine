-- Quest: Renewing the Covenant (Lv. 14)
-- NOTE: The Quest sheet doesn't name this quest's scenes, so these come from Sapphire's script and are unverified for 7.56.
-- Sequence 0: accept from ACTOR0 (plus a follow-up scene)
-- Sequence 1: ACTOR2 gives you RITEM0 (a hat), then speak with ACTOR1 while wearing it
-- Sequence 255: report to ACTOR3. ACTOR4 can take you to TERRITORYTYPE0.
-- NOTE: Kawari can't check equipment from scripts yet, so ACTOR1 assumes you're wearing the hat once you've been given it.

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
        setQuestBitFlag8(player, 1, true) -- remember that we were given the hat
    elseif scene == SCENE_ACTOR1 then
        if questBitFlag8(player, 1) then
            questPlay(player, SCENE_ACTOR1_WEARING, HIDE_HOTBAR)
        else
            questPlay(player, SCENE_ACTOR1_NOT_WEARING, HIDE_HOTBAR)
        end
        return true
    elseif scene == SCENE_ACTOR1_WEARING then
        questPlay(player, SCENE_ACTOR1_CUTSCENE, FADE_OUT | CONDITION_CUTSCENE | HIDE_UI)
        return true
    elseif scene == SCENE_ACTOR1_CUTSCENE then
        setQuestSequence(player, SEQ_FINISH)
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
