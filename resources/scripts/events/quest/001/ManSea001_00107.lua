-- Quest: Coming to Limsa
-- NOTE: These openings are so similar, ensure changes are synced between all of them!
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-19) and Bob's Chronofoil capture of the Limsa
-- opening on retail (2026-10-20):
-- Sequence 0:   ACTOR0 (Ryssfloh) offers it (scene 0, a yield: 1 accepts, else the decline, scene 4), then the
--               cutscene (scene 1), "you have begun a quest" (scene 2, a yield) and the client's empty scene 3; when
--               scene 3 returns retail accepts the quest, sets sequence 1 and calls the opening (its scene 30 with 2,
--               nested), then finishes both events.
-- Sequence 1:   ride the Crow's Lift: ACTOR1 (Grehfarr) asks (scene 5, a yield: 1 rides), scene 6, then the event
--               finishes and the player is moved to the lift's top (POPRANGE0, PrepareZoning within 181), with
--               sequence 255.
-- Sequence 255: ACTOR2 (Baderon): cutscene (scene 11), then the reward (scene 12, a yield: 1 completes it). ACTOR1
--               asks again (scene 13, as 5).

SCENE_OFFER = 0
SCENE_CUTSCENE = 1
SCENE_BEGUN = 2
SCENE_ACCEPTED = 3 -- empty in the client's script; retail accepts when it returns
SCENE_DECLINE = 4
SCENE_LIFT = 5
SCENE_LIFT_RIDE = 6
SCENE_BADERON = 11
SCENE_REWARD = 12
SCENE_LIFT_AGAIN = 13

-- The opening's scene that updates the play area's borders, nested in the quest's event (retail: flags 0x2001, 2.)
SCENE_OPENING_BORDERS = 30

-- The opening's NPCs, by their layout IDs (the objects you talk to.)
ENPC_RYSSFLOH = 4102039
ENPC_BADERON = 4102072
ENPC_GREHFARR = 4107186

-- Destination for the Crow's Lift
-- This located in the EVT_OP_ONLY_ENPC layer.
POS_INN_WARP = 4127803

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags, params)
    current_scene = scene
    player:play_scene(scene, flags, params or {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if target.object_id == ENPC_RYSSFLOH and sequence == nil then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif target.object_id == ENPC_GREHFARR and sequence == 1 then
        play(player, SCENE_LIFT, HIDE_HOTBAR)
    elseif target.object_id == ENPC_GREHFARR and sequence == 255 then
        play(player, SCENE_LIFT_AGAIN, HIDE_HOTBAR)
    elseif target.object_id == ENPC_BADERON and sequence == 255 then
        play(player, SCENE_BADERON, SET_BASE | HIDE_HOTBAR)
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
            play(player, SCENE_CUTSCENE, SET_BASE | HIDE_HOTBAR | DISABLE_SKIP)
        else
            play(player, SCENE_DECLINE, HIDE_HOTBAR)
        end
        return
    elseif scene == SCENE_CUTSCENE then
        play(player, SCENE_BEGUN, HIDE_HOTBAR)
        return
    elseif scene == SCENE_BEGUN then
        play(player, SCENE_ACCEPTED, HIDE_HOTBAR)
        return
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)

        -- call back into the opening, presumably to update the borders of the play area
        player:start_event(OPENING_EVENT_HANDLER, EVENT_TYPE_NEST, 0)
        play(player, SCENE_OPENING_BORDERS, HIDE_HOTBAR | NO_DEFAULT_CAMERA, { 2 })
        return
    elseif scene == SCENE_LIFT or scene == SCENE_LIFT_AGAIN then
        if results[1] == 1 then
            play(player, SCENE_LIFT_RIDE, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_LIFT_RIDE then
        player:finish_event()
        -- Move the player into the destination position
        player:move_to_pop_range(POS_INN_WARP, true)
        if player:get_quest_sequence(EVENT_ID) == 1 then
            player:quest_sequence(EVENT_ID, 255)
        end
        return
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
        end
    elseif scene == SCENE_BADERON then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
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
