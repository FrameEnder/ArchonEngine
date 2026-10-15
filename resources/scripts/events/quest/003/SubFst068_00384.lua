-- Quest: A Hearer Is Often Late (Gridania)
--
-- Checked against the client's quest script (kawari-questdump), Sapphire's script and a live recording of the quest:
-- Sequence 0:   ACTOR0 (Lothaire) offers the quest (scene 0, returns 1 when accepted.) The client itself refuses it while
--               QST_ACCEPT_CHECK isn't complete.
-- Sequence 1:   talk to ACTOR1 (Leonnie) (scene 1.) Retail sends notice 0 (no arguments) as the quest moves on to
--               sequence 255.
-- Sequence 255: report to ACTOR2 (Armelle) (scene 2, with the reward.) ACTOR1 has a line of her own (scene 3.)

SCENE_OFFER = 0
SCENE_LEONNIE = 1
SCENE_FINISH = 2
SCENE_LEONNIE_AFTER = 3

NOTICE_SEQUENCE = 0

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
        play(player, SCENE_LEONNIE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_FINISH, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_LEONNIE_AFTER, HIDE_HOTBAR)
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
    elseif scene == SCENE_LEONNIE and player:get_quest_sequence(EVENT_ID) == 1 then
        player:event_notice(EVENT_ID, NOTICE_SEQUENCE)
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
