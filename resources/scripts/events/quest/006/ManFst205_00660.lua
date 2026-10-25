-- Quest: Into a Copper Hell (Ul'dah)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-25), Sapphire's script and retail (Bob's
-- Chronofoil capture of 2026-10-25: Ul'dah, the Copperbell Mines three times, quest battle 44 four times, back to
-- Momodi; questkit replay 660):
-- Sequence 0:   ACTOR0 (Momodi) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then the cutscene
--               (scene 1, CUTSCENE_FLAGS, which ends with QuestAccepted): sequence 1.
-- Sequence 1:   ACTOR1 (Painted Mesa) (scene 2): notice 0, sequence 2.
-- Sequence 2:   ACTOR2 (Stone Torch) (scene 3, which shows the Copperbell Mines' unlock image and the "new duty" log
--               line): the Copperbell Mines (INSTANCEDUNGEON0, InstanceContent 3) are unlocked (ActorControlSelf 131
--               before the notice), notice 1, sequence 3. Meanwhile ACTOR1 (scene 6), ACTOR3 (scene 4) and ACTOR4
--               (scene 5) have a line each.
-- Sequence 3:   clearing the Copperbell Mines: retail moved it on as the player left the cleared duty (notice 2,
--               no variables, sequence 4), which the server tells the script with onDutyCleared. Meanwhile ACTOR2
--               (scene 9), ACTOR3 (scene 7) and ACTOR4 (scene 8) have a line each.
-- Sequence 4:   ACTOR1 asks whether to enter QUESTBATTLE0 (scene 10: YesNoQuestBattle, a yield, 1 for yes), which
--               takes the player to TERRITORYTYPE0 (274). Retail declined once and entered four times. The client
--               also offers EOBJECT0 here, whose scene (11) is empty. Winning the battle sets sequence 5: that's the
--               battle's director (content script), not this script.
-- Sequence 5:   entering TERRITORYTYPE1 (130) after the battle (onEnterTerritory) plays the cutscene (scene 12,
--               CUTSCENE_FLAGS): sequence 255.
-- Sequence 255: ACTOR0 rewards it (scene 13, a cutscene and a yield: QuestReward, 1 and the chosen item.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_PAINTED_MESA = 2
SCENE_STONE_TORCH = 3
SCENE_ISILDAURE = 4
SCENE_ALIANNE = 5
SCENE_PAINTED_MESA_WAITING = 6
SCENE_ISILDAURE_DUTY = 7
SCENE_ALIANNE_DUTY = 8
SCENE_STONE_TORCH_DUTY = 9
SCENE_ENTER_BATTLE = 10
SCENE_EOBJECT = 11
SCENE_AFTER_BATTLE = 12
SCENE_FINISH = 13

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB, as retail played the cutscenes (Bob's Chronofoil capture)

-- Notices (Bob's Chronofoil capture of 2026-10-25): one per todo, none with arguments.
NOTICE_PAINTED_MESA = 0
NOTICE_STONE_TORCH = 1
NOTICE_COPPERBELL = 2

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
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_PAINTED_MESA, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR2 then
        play(player, SCENE_STONE_TORCH, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR1 then
        play(player, SCENE_PAINTED_MESA_WAITING, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR3 then
        play(player, SCENE_ISILDAURE, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR4 then
        play(player, SCENE_ALIANNE, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR2 then
        play(player, SCENE_STONE_TORCH_DUTY, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR3 then
        play(player, SCENE_ISILDAURE_DUTY, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR4 then
        play(player, SCENE_ALIANNE_DUTY, HIDE_HOTBAR)
    elseif sequence == 4 and BASE_ID == ACTOR1 then
        play(player, SCENE_ENTER_BATTLE, HIDE_HOTBAR)
    elseif sequence == 4 and BASE_ID == EOBJECT0 then
        play(player, SCENE_EOBJECT, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_FINISH, CUTSCENE_FLAGS)
    else
        player:finish_event()
    end
end

-- Leaving a duty the player cleared (its InstanceContent ID). Not an event of the quest's: nothing to finish.
function onDutyCleared(content_id, player)
    if player:get_quest_sequence(EVENT_ID) == 3 and content_id == INSTANCEDUNGEON0 then
        player:event_notice(EVENT_ID, NOTICE_COPPERBELL, {})
        player:quest_sequence(EVENT_ID, 4)
    end
end

-- Back in Ul'dah after winning the battle.
function onEnterTerritory(player)
    if player:get_quest_sequence(EVENT_ID) == 5 then
        play(player, SCENE_AFTER_BATTLE, CUTSCENE_FLAGS)
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
    elseif scene == SCENE_PAINTED_MESA then
        player:event_notice(EVENT_ID, NOTICE_PAINTED_MESA, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_STONE_TORCH then
        player:unlock_content(INSTANCEDUNGEON0)
        player:event_notice(EVENT_ID, NOTICE_STONE_TORCH, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_ENTER_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
            return
        end
    elseif scene == SCENE_AFTER_BATTLE then
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_FINISH then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID, results[2])
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
