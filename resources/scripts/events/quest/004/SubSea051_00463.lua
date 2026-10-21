-- Quest: Dressed to Call (Limsa Lominsa)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-21) and Bob's Chronofoil capture of the Limsa
-- story on retail (2026-10-21); laid out like Gridania's Passing Muster (SubFst045_00201):
-- Sequence 0:   ACTOR0 offers the quest (scene 0), which returns 1 when it's accepted (straight into sequence 255.)
-- Sequence 255: talk to ACTOR0 again (scene 1). The client checks your head, body, hands, legs and feet are all of
--               the item level it wants itself (checkEquippedItemLevel). If they are it shows the reward window and
--               returns 1 and the reward picked (an item ID), otherwise ACTOR0 tells you to gear up and it returns 0.

SCENE_OFFER = 0
SCENE_FINISH = 1

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
    elseif sequence == 255 and BASE_ID == ACTOR0 then
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
            -- Accepting puts the quest straight into sequence 255, all that's left is talking to ACTOR0 again.
            player:accept_quest(EVENT_ID)
        end
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
