-- Quest: Do Angry Pirates Dream (Limsa Lominsa)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-21) and Bob's Chronofoil capture of the Limsa
-- story on retail (2026-10-21):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: 1 accepts): sequence 1.
-- Sequence 1:   ACTOR1 (scene 1): notice 0, sequence 2.
-- Sequence 2:   ACTOR2 (scene 2): notice 1, sequence 3.
-- Sequence 3:   ACTOR3: the cutscene (scene 3, 0xF8402FFF, a yield): notice 2 [1, 1], sequence 255. ACTOR4 and ACTOR5
--               have a line each meanwhile (scenes 4 and 5, from the client's script).
-- Sequence 255: ACTOR1 rewards it (scene 12, a yield: 1 completes it.)

SCENE_OFFER = 0
SCENE_ACTOR1 = 1
SCENE_ACTOR2 = 2
SCENE_CUTSCENE = 3
SCENE_ACTOR4 = 4
SCENE_ACTOR5 = 5
SCENE_FINISH = 12

-- The cutscene's flags as retail sent them (0xF8402FFF.)
CUTSCENE_FLAGS = 0xF8402FFF

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
        play(player, SCENE_ACTOR1, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR2 then
        play(player, SCENE_ACTOR2, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR3 then
        play(player, SCENE_CUTSCENE, CUTSCENE_FLAGS)
    elseif sequence == 3 and BASE_ID == ACTOR4 then
        play(player, SCENE_ACTOR4, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR5 then
        play(player, SCENE_ACTOR5, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_FINISH, HIDE_HOTBAR)
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
            player:accept_quest(EVENT_ID)
            player:quest_sequence(EVENT_ID, 1)
        end
    elseif scene == SCENE_ACTOR1 then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_ACTOR2 then
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_CUTSCENE then
        player:event_notice(EVENT_ID, 2, { 1, 1 })
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
