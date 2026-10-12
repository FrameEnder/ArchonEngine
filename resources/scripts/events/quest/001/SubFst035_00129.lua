-- Quest: Spirithold Broken (Lv. 9)
-- Written from the client's quest script (scenes and objectives.)
-- Sequence 0: accept from ACTOR0 (offer scene 0, then the accepting cutscene 1)
-- Sequence 1: speak with ACTOR1 (scene 2)
-- Sequence 2: calm ACTOR2-ACTOR6 (scenes 4, 7, 10, 13, 16), once each (BitFlag8 1-5), counted in UI8AL.
--             ACTOR7 has a line here too (scene 18.)
-- Sequence 3: speak with ACTOR7, which leads into the solo battle QUESTBATTLE0 (scene 19)
-- Sequence 4: return from the battle (cutscene 23)
-- Sequence 5: speak with ACTOR1 (scene 24)
-- Sequence 255: report to ACTOR8 (reward cutscene 25)
-- NOTE: Quest battles aren't implemented in Kawari yet, so agreeing to start it skips straight past it.
-- NOTE: The "calming" cast bar (EVENT_ACTION_) isn't played, the NPC's scene plays straight away.
-- NOTE: Retail unlocks the inn and levequests with this quest's reward. Kawari doesn't do that from here yet.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_ACTOR1 = 2
SCENE_ACTOR7 = 18
SCENE_START_BATTLE = 19
SCENE_AFTER_BATTLE = 23
SCENE_ACTOR1_REPORT = 24
SCENE_ACTOR8_REWARD = 25
PEOPLE_TO_CALM = 5

-- Base ID -> its scene and flag
PANICKING = {
    [ACTOR2] = { scene = 4, flag = 1 },
    [ACTOR3] = { scene = 7, flag = 2 },
    [ACTOR4] = { scene = 10, flag = 3 },
    [ACTOR5] = { scene = 13, flag = 4 },
    [ACTOR6] = { scene = 16, flag = 5 },
}

function onTalk(target, player)
    local sequence = questSequence(player)
    local panicking = PANICKING[BASE_ID]

    if sequence == nil and BASE_ID == ACTOR0 then
        questPlay(player, SCENE_OFFER)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        questPlay(player, SCENE_ACTOR1)
    elseif sequence == 2 and panicking ~= nil and not questBitFlag8(player, panicking.flag) then
        questPlay(player, panicking.scene)
    elseif sequence == 2 and BASE_ID == ACTOR7 then
        questPlay(player, SCENE_ACTOR7)
    elseif sequence == 3 and BASE_ID == ACTOR7 then
        questPlay(player, SCENE_START_BATTLE)
    elseif sequence == 4 and BASE_ID == ACTOR7 then
        -- In case the cutscene after the battle was missed
        questPlay(player, SCENE_AFTER_BATTLE, QUEST_CUTSCENE_FLAGS)
    elseif sequence == 5 and BASE_ID == ACTOR1 then
        questPlay(player, SCENE_ACTOR1_REPORT)
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR8 then
        questPlay(player, SCENE_ACTOR8_REWARD, QUEST_CUTSCENE_FLAGS)
    else
        player:finish_event()
    end
end

function onSceneFinished(scene, results, player)
    if scene == SCENE_OFFER then
        if results[1] == 1 then
            questPlay(player, SCENE_ACCEPTED, QUEST_CUTSCENE_FLAGS)
            return true
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID, 1)
    elseif scene == SCENE_ACTOR1 then
        questAdvance(player, 2)
    elseif scene == SCENE_START_BATTLE then
        if results[1] == 1 then
            questSkipBattle(player)
            questAdvance(player, 4)
            questPlay(player, SCENE_AFTER_BATTLE, QUEST_CUTSCENE_FLAGS)
            return true
        end
    elseif scene == SCENE_AFTER_BATTLE then
        questAdvance(player, 5)
    elseif scene == SCENE_ACTOR1_REPORT then
        questAdvance(player, SEQ_FINISH)
    elseif scene == SCENE_ACTOR8_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
        end
    else
        for _, panicking in pairs(PANICKING) do
            if scene == panicking.scene and not questBitFlag8(player, panicking.flag) then
                setQuestBitFlag8(player, panicking.flag, true)
                if questIncrement(player, "AL") >= PEOPLE_TO_CALM then
                    questAdvance(player, 3)
                end
            end
        end
    end

    return false
end

onYield = questOnYield
onReturn = questOnReturn
