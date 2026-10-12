-- Quest: To Guard a Guardian (Lv. 14)
-- Written from the client's quest script (scenes, key items, objectives.)
-- Sequence 0: the quest offer from ACTOR0 (scene 0), then a cutscene (scene 1.) You receive ITEM0 (held in UI8BH.)
-- Sequence 1: hand ITEM0 to ACTOR1 (trade scene 2, then scene 3.) ACTOR2 takes you into TERRITORYTYPE0 (scene 5.)
-- Sequence 2: speak with ACTOR3, which leads into the solo battle QUESTBATTLE0 (scene 7)
-- Sequence 3: return from the battle (cutscene 20)
-- Sequence 255: report to ACTOR1 (reward scene 21.) ACTOR2 takes you into TERRITORYTYPE0 (scene 22.)
-- NOTE: Quest battles aren't implemented in Kawari yet, so agreeing to start it skips straight past it.
-- NOTE: On retail, walking into EVENTRANGE0 can also offer the battle (scene 6.) Kawari doesn't have quest event ranges.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_ACTOR1_TRADE = 2
SCENE_ACTOR1_TRADED = 3
SCENE_ACTOR2_ENTER = 5
SCENE_START_BATTLE = 7
SCENE_AFTER_BATTLE = 20
SCENE_ACTOR1_REWARD = 21
SCENE_ACTOR2_ENTER_FINISH = 22

function onTalk(target, player)
    local sequence = questSequence(player)

    if sequence == nil and BASE_ID == ACTOR0 then
        questPlay(player, SCENE_OFFER)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        questPlay(player, SCENE_ACTOR1_TRADE)
    elseif sequence == 1 and BASE_ID == ACTOR2 then
        questPlay(player, SCENE_ACTOR2_ENTER)
    elseif sequence == 2 and BASE_ID == ACTOR3 then
        questPlay(player, SCENE_START_BATTLE)
    elseif sequence == 3 and BASE_ID == ACTOR3 then
        -- In case the cutscene after the battle was missed
        questPlay(player, SCENE_AFTER_BATTLE, QUEST_CUTSCENE_FLAGS)
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR1 then
        questPlay(player, SCENE_ACTOR1_REWARD)
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR2 then
        questPlay(player, SCENE_ACTOR2_ENTER_FINISH)
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
        setQuestVar(player, "BH", 1) -- receive ITEM0
    elseif scene == SCENE_ACTOR1_TRADE then
        if results[1] == 1 then
            questPlay(player, SCENE_ACTOR1_TRADED, QUEST_CUTSCENE_FLAGS)
            return true
        end
    elseif scene == SCENE_ACTOR1_TRADED then
        questAdvance(player, 2)
    elseif scene == SCENE_ACTOR2_ENTER or scene == SCENE_ACTOR2_ENTER_FINISH then
        if results[1] == 1 then
            player:change_territory_pop_range(TERRITORYTYPE0, POPRANGE0)
        end
    elseif scene == SCENE_START_BATTLE then
        if results[1] == 1 then
            questSkipBattle(player)
            questAdvance(player, 3)
            questPlay(player, SCENE_AFTER_BATTLE, QUEST_CUTSCENE_FLAGS)
            return true
        end
    elseif scene == SCENE_AFTER_BATTLE then
        questAdvance(player, SEQ_FINISH)
    elseif scene == SCENE_ACTOR1_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
        end
    end

    return false
end

onYield = questOnYield
onReturn = questOnReturn
