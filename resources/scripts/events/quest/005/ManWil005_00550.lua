-- Quest: Underneath the Sultantree (Ul'dah)
--
-- Checked against the client's quest script (kawari-questdump) and retail (a Kawari Recorder export and Bob's
-- Chronofoil session of 2026-10-17, questkit replay 550):
-- Sequence 0:   ACTOR0 offers the quest (scene 0, a yield returning 1 when accepted), then explains it in a cutscene
--               (scene 1.)
-- Sequence 1:   ACTOR1 asks whether to enter the quest battle (scene 2, YesNoQuestBattle, a yield returning 1.)
--               QUESTBATTLE0 takes place in TERRITORYTYPE0, and winning it moves the quest to sequence 2. Every
--               variable stays 0 through the quest.
-- Sequence 2:   coming back into Central Thanalan (TERRITORYTYPE1) plays the aftermath cutscenes (scene 5.)
-- Sequence 255: report to ACTOR0, and the reward (scene 6, a yield returning 1.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_ENTER_BATTLE = 2
SCENE_AFTERMATH = 5
SCENE_FINISH = 6

-- Scene flags as retail sent them: 0xF8402EFB for the explanation, 0xF8402FFF for the aftermath and the reward.
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR
AFTERMATH_FLAGS = SET_BASE | HIDE_HOTBAR | INVIS_ENPC | INVIS_AETHERYTE

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
        play(player, SCENE_ENTER_BATTLE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_FINISH, AFTERMATH_FLAGS)
    else
        player:finish_event()
    end
end

-- Back in Central Thanalan after winning the battle.
function onEnterTerritory(player)
    if player:get_quest_sequence(EVENT_ID) == 2 then
        play(player, SCENE_AFTERMATH, AFTERMATH_FLAGS)
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
    elseif scene == SCENE_ENTER_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
            return
        end
    elseif scene == SCENE_AFTERMATH then
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
