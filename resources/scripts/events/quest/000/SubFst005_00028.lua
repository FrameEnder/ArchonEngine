-- Quest: To the Bannock (Gridania)
--
-- Checked against the client's quest script (kawari-questdump) and retail: Kawari Recorder exports (2026.09.15-
-- 2026.10.06, a fresh character from the opening to quest 387) and Bob's Chronofoil captures (notices 2026.10.06,
-- scenes and effects 2026-10-14 to 2026-10-16): its fixture is tools/questkit/fixtures/28.json.
-- Sequence 0:   ACTOR0 offers the quest (scene 0), which returns 1 when it's accepted.
-- Sequence 255: report to ACTOR1 at the Bannock (scene 1) to finish. It returns 1 and the reward you picked
--               (an item id). No variables or items are used.

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
            -- Accepting puts the quest straight into sequence 255, since all that's left is reporting to ACTOR1.
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
