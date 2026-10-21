-- Quest: Lurkers in the Grotto (Limsa Lominsa)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-21), Sapphire's ManSea005 and Bob's Chronofoil
-- capture of the Limsa story on retail (2026-10-21, two runs of the quest battle: lost, then won):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: 1 accepts), then the cutscene (scene 1): sequence 1.
-- Sequence 1:   EOBJECT0, the stone monument (scene 2, empty), then EVENT_ACTION_PROCESS_MIDDLE (the progress bar),
--               then the quest battle's yes/no (scene 3, YesNoQuestBattle: 1 enters QUESTBATTLE0 in TERRITORYTYPE0),
--               all with HIDE_HOTBAR | NO_DEFAULT_CAMERA; after a loss it's all of it again. Winning the battle sets
--               sequence 2 and UI8BH = 1 (ITEM0) in one update (the battle's script, content/questbattle/35.lua.)
-- Sequence 2:   back in TERRITORYTYPE1, the aftermath cutscenes (scene 4, SET_BASE | HIDE_HOTBAR): sequence 255.
-- Sequence 255: hand ITEM0 to ACTOR0 (scene 5, an NpcTrade: 1 if given), then the reward (scene 6, the reward picked);
--               retail then marks quest 464 completed too, with no rewards of its own.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_MONUMENT = 2 -- empty in the client's script: the progress bar follows
SCENE_ENTER_BATTLE = 3
SCENE_AFTERMATH = 4
SCENE_TRADE = 5
SCENE_FINISH = 6

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB, as retail played the cutscenes
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001, as retail played the monument's scenes

-- Retail sent FinishQuest for this one too, straight after this quest's (Bob's Chronofoil capture of 2026-10-21).
-- Washed Up (465) checks it (QST_ACCEPT_CHECK).
QUEST_ALSO_COMPLETED = 464

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == EOBJECT0 then
        play(player, SCENE_MONUMENT, OBJECT_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- Back in Middle La Noscea after winning the battle.
function onEnterTerritory(player)
    if player:get_quest_sequence(EVENT_ID) == 2 then
        play(player, SCENE_AFTERMATH, CUTSCENE_FLAGS)
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    if player:get_quest_sequence(EVENT_ID) == 1 then
        play(player, SCENE_ENTER_BATTLE, OBJECT_FLAGS)
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
            play(player, SCENE_ACCEPTED, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_MONUMENT then
        -- The event carries on once the progress bar finishes, see onEventActionCast.
        player:event_action(EVENT_ACTION_PROCESS_MIDDLE)
        return
    elseif scene == SCENE_ENTER_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
            return
        end
    elseif scene == SCENE_AFTERMATH then
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_FINISH, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_FINISH then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID, results[2])
            player:mark_quest_completed(QUEST_ALSO_COMPLETED)
        end
    end

    player:finish_event()
end

function onYield(scene, id, results, player)
    sceneFinished(scene, results, player)
end

function onReturn(scene, results, player)
    sceneFinished(scene, results, player)
end
