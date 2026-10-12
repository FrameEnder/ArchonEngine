-- Quest: Leia's Legacy (Lv. 14)
-- Written from the client's quest script (scenes, key items, objectives.)
-- Sequence 0: accept from ACTOR0 (scene 0)
-- Sequence 1: speak with ACTOR1, ACTOR2 and ACTOR3 (scenes 1-3), once each (BitFlag8 1-3), counted in UI8AL
-- Sequence 2: report to ACTOR0 (cutscene 4)
-- Sequence 3: speak with ACTOR4, which leads into the solo battle QUESTBATTLE0 (scene 5)
-- Sequence 4: you come back from the battle with ITEM0 (held in UI8BH from here on.) Speak with ACTOR4 (cutscene 11)
-- Sequence 5: hand ITEM0 to ACTOR0 (trade scene 15, then cutscene 16)
-- Sequence 255: speak with ACTOR0 (reward scene 18)
-- NOTE: Quest battles aren't implemented in Kawari yet, so agreeing to start it skips straight past it.

SCENE_ACCEPT = 0
SCENE_ACTOR0_REPORT = 4
SCENE_START_BATTLE = 5
SCENE_ACTOR4_AFTER_BATTLE = 11
SCENE_ACTOR0_TRADE = 15
SCENE_ACTOR0_TRADED = 16
SCENE_ACTOR0_REWARD = 18
PEOPLE_TO_ASK = 3

-- Base ID -> its scene and flag
WITNESSES = {
    [ACTOR1] = { scene = 1, flag = 1 },
    [ACTOR2] = { scene = 2, flag = 2 },
    [ACTOR3] = { scene = 3, flag = 3 },
}

function onTalk(target, player)
    local sequence = questSequence(player)
    local witness = WITNESSES[BASE_ID]

    if sequence == nil and BASE_ID == ACTOR0 then
        questPlay(player, SCENE_ACCEPT)
    elseif sequence == 1 and witness ~= nil and not questBitFlag8(player, witness.flag) then
        questPlay(player, witness.scene)
    elseif sequence == 2 and BASE_ID == ACTOR0 then
        questPlay(player, SCENE_ACTOR0_REPORT, QUEST_CUTSCENE_FLAGS)
    elseif sequence == 3 and BASE_ID == ACTOR4 then
        questPlay(player, SCENE_START_BATTLE)
    elseif sequence == 4 and BASE_ID == ACTOR4 then
        questPlay(player, SCENE_ACTOR4_AFTER_BATTLE, QUEST_CUTSCENE_FLAGS)
    elseif sequence == 5 and BASE_ID == ACTOR0 then
        questPlay(player, SCENE_ACTOR0_TRADE)
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR0 then
        questPlay(player, SCENE_ACTOR0_REWARD)
    else
        player:finish_event()
    end
end

function onSceneFinished(scene, results, player)
    if scene == SCENE_ACCEPT then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID, 1)
        end
    elseif scene == SCENE_ACTOR0_REPORT then
        questAdvance(player, 3)
    elseif scene == SCENE_START_BATTLE then
        if results[1] == 1 then
            questSkipBattle(player)
            questAdvance(player, 4, { BH = 1 }) -- came back with ITEM0
        end
    elseif scene == SCENE_ACTOR4_AFTER_BATTLE then
        questAdvance(player, 5, { BH = 1 })
    elseif scene == SCENE_ACTOR0_TRADE then
        if results[1] == 1 then
            questPlay(player, SCENE_ACTOR0_TRADED, QUEST_CUTSCENE_FLAGS)
            return true
        end
    elseif scene == SCENE_ACTOR0_TRADED then
        questAdvance(player, SEQ_FINISH, { BH = 1 })
    elseif scene == SCENE_ACTOR0_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
        end
    else
        for _, witness in pairs(WITNESSES) do
            if scene == witness.scene and not questBitFlag8(player, witness.flag) then
                setQuestBitFlag8(player, witness.flag, true)
                if questIncrement(player, "AL") >= PEOPLE_TO_ASK then
                    questAdvance(player, 2)
                end
            end
        end
    end

    return false
end

onYield = questOnYield
onReturn = questOnReturn
