-- Quest: Seer Folly (the White Mage's first quest, level 30 Conjurer)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-05) and checked against Bob's Chronofoil capture
-- 8d550e33 (2026-11-05, the whole quest, its quest battle won; a level 35 conjurer): its fixture is
-- tools/questkit/fixtures/1079.json.
-- Sequence 0:   ACTOR0 (E-Sumi-Yan, in Old Gridania) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then
--               talks (scene 1, which ends with QuestAccepted): sequence 1.
-- Sequence 1:   ACTOR1 (Raya-O-Senna, in the Central Shroud, scene 2): YesNoQuestBattle; 1 enters QUESTBATTLE0 (75) in
--               TERRITORYTYPE0 (319), and it can be tried again after a loss. Winning it moves to sequence 255 with
--               UI8BH 1 (ITEM0, the key item the client lists then; content/questbattle/75.lua.)
-- Sequence 255: ACTOR3 (Raya-O-Senna, in the South Shroud) rewards it (scene 6, a yield: QuestReward, 1 when the reward
--               is taken). Retail then, in the same event, set UI8AL and UI8AH to 1 and played scene 7 (flags
--               0xF8482EFB, parameter 0: the soul crystal's cutscene, where the client shows LOC_WS, 136, being
--               learnt), and completed the quest as scene 7 ended. Completing it gives the White Mage's soul crystal
--               (the ClassJob sheet's ItemSoulCrystal for the job whose UnlockQuest it is: `zone_connection/quest.rs`).
-- EOBJECT0 (the search) and ACTOR2 (A-Ruhn-Senna), listed in sequence 1, have only empty scenes now (3, 4 and 5: what
-- was left of the quest's older steps); retail's run never touched them. Which of 3-5 is whose is a guess: EOBJECT0
-- 3, ACTOR2 4.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_BATTLE = 2
SCENE_SEARCH = 3
SCENE_ARUHN = 4
SCENE_REWARD = 6
SCENE_SOUL_CRYSTAL = 7

-- Retail's flags and parameter for the soul crystal's cutscene (Bob's Chronofoil capture 8d550e33.)
SOUL_CRYSTAL_FLAGS = 0xF8482EFB
SOUL_CRYSTAL_PARAMS = { 0 }

-- UI8AL and UI8AH, both 1 as the reward was taken (byte 0 0x11, retail's UpdateQuest as scene 7 began.)
VARS_AFTER_REWARD = 0x11

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags, params)
    current_scene = scene
    player:play_scene(scene, flags, params or {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_BATTLE, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == EOBJECT0 then
        play(player, SCENE_SEARCH, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR2 then
        play(player, SCENE_ARUHN, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR3 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            play(player, SCENE_ACCEPTED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
            return
        end
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:set_quest_var(EVENT_ID, 0, VARS_AFTER_REWARD)
            play(player, SCENE_SOUL_CRYSTAL, SOUL_CRYSTAL_FLAGS, SOUL_CRYSTAL_PARAMS)
            return
        end
    elseif scene == SCENE_SOUL_CRYSTAL then
        player:finish_event()
        player:finish_quest(EVENT_ID, 0)
        return
    end

    player:finish_event()
end

function onYield(scene, id, results, player)
    sceneFinished(scene, results, player)
end

function onReturn(scene, results, player)
    sceneFinished(scene, results, player)
end
