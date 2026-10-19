-- Quest: Prudence at This Junction (Ul'dah, between Step Nine and Out of House and Home)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-19) and retail (Bob's Chronofoil capture of
-- 2026-10-18, questkit replay 3852):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield returning 1 when accepted), then scene 1 (how to equip gear);
--               accepted after it. The client's scene 0 refuses on its own while Disorderly Conduct, Until a Quieter
--               Time or the other Prudence quest (QST_ACCEPT_CHECK_*) is accepted.
-- Sequence 1:   ACTOR1 (scene 2): notice 0, sequence 255. No variables change on retail.
-- Sequence 255: ACTOR1 rewards it (scene 3, a yield returning 1 and the chosen item). The client only lets you talk to
--               him, and only rewards, with item level 5 or more in head, body, hands, legs and feet
--               (checkEquippedItemLevel); otherwise the scene ends without a reward.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_WARIN = 2
SCENE_FINISH = 3

NOTICE_WARIN = 0

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
        play(player, SCENE_WARIN, HIDE_HOTBAR)
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
            play(player, SCENE_ACCEPTED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_WARIN then
        player:event_notice(EVENT_ID, NOTICE_WARIN)
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
