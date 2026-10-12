-- Quest: Chasing Shadows (Lv. 5)
-- NOTE: The Quest sheet doesn't name most of this quest's scenes, so these come from Sapphire's script and are unverified for 7.56.
-- Sequence 0: accept from ACTOR0 (followed by a cutscene)
-- Sequence 1: examine EOBJECT0, which leads into the solo battle QUESTBATTLE0
-- Sequence 2: return from the battle (a cutscene plays)
-- Sequence 3: examine EOBJECT1
-- Sequence 255: report to ACTOR0
-- NOTE: Quest battles aren't implemented in Kawari yet, so agreeing to start it skips straight past it.
-- NOTE: The "searching" cast bar isn't played, the scene after it plays straight away.

SCENE_ACCEPT = 0
SCENE_ACCEPTED_CUTSCENE = 1
SCENE_EOBJECT0 = 2
SCENE_START_BATTLE = 3
SCENE_AFTER_BATTLE = 5
SCENE_ACTOR0_REPORT = 9
SCENE_ACTOR0_FINISH = 10

AFTER_BATTLE_FLAGS = NO_DEFAULT_CAMERA | CONDITION_CUTSCENE | SILENT_ENTER_TERRI_ENV | HIDE_HOTBAR | SILENT_ENTER_TERRI_BGM | SILENT_ENTER_TERRI_SE | DISABLE_STEALTH

function onTalk(target, player)
    local sequence = questSequence(player)

    if sequence == nil and BASE_ID == ACTOR0 then
        questPlay(player, SCENE_ACCEPT, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == EOBJECT0 then
        questPlay(player, SCENE_EOBJECT0, HIDE_HOTBAR)
    elseif sequence == 2 then
        -- In case the cutscene after the battle was missed
        questPlay(player, SCENE_AFTER_BATTLE, AFTER_BATTLE_FLAGS)
    elseif sequence == 3 and BASE_ID == EOBJECT1 then
        setQuestSequence(player, SEQ_FINISH)
        player:finish_event()
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR0 then
        questPlay(player, SCENE_ACTOR0_REPORT, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onSceneFinished(scene, results, player)
    if scene == SCENE_ACCEPT then
        if results[1] == 1 then
            questPlay(player, SCENE_ACCEPTED_CUTSCENE, FADE_OUT | CONDITION_CUTSCENE | HIDE_UI)
            return true
        end
    elseif scene == SCENE_ACCEPTED_CUTSCENE then
        player:accept_quest(EVENT_ID, 1)
    elseif scene == SCENE_EOBJECT0 then
        questPlay(player, SCENE_START_BATTLE, HIDE_HOTBAR)
        return true
    elseif scene == SCENE_START_BATTLE then
        if results[1] == 1 then
            player:send_message("Quest battles aren't implemented yet, so this one was skipped.")
            setQuestSequence(player, 2)
            questPlay(player, SCENE_AFTER_BATTLE, AFTER_BATTLE_FLAGS)
            return true
        end
    elseif scene == SCENE_AFTER_BATTLE then
        setQuestSequence(player, 3)
    elseif scene == SCENE_ACTOR0_REPORT then
        questPlay(player, SCENE_ACTOR0_FINISH, FADE_OUT | HIDE_HOTBAR | CONDITION_CUTSCENE | HIDE_UI)
        return true
    elseif scene == SCENE_ACTOR0_FINISH then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
        end
    end

    return false
end

onYield = questOnYield
onReturn = questOnReturn
