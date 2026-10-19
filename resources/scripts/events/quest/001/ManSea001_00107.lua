-- Quest: Coming to Limsa
-- NOTE: These openings are so similar, ensure changes are synced between all of them!

-- scene 0: show quest prompt
-- scene 4: decline
-- scene 1: after quest acceptance
-- scene 2: you have begun a quest!
-- scene 5: take the crow's lift (13 is the same scene, for sequence 255)
-- scene 11: cutscene with grehfarr
-- scene 12: quest completed prompt
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-19), not yet against retail: the decline is
-- scene 4 (the client's scene 3 is empty; this script played 3, then 4 before the lift's yes/no), scenes 3, 6-10 are
-- empty. The client has a sequence 1 (ride the lift: ACTOR1 while UI8AL is 0 and BitFlag8 1 unset, with EOBJECT0/1),
-- then 255 (ACTOR2); this script still goes straight to 255 on accepting, as it always has, until a recording shows
-- what retail sets. The reward scene only completes the quest when it returns 1 (QuestReward).

ENPC_RYSSFLOH = 4102039
ENPC_BADERON = 4102072
ENPC_GREHFARR = 4107186

-- Destination for the Crow's Lift
-- This located in the EVT_OP_ONLY_ENPC layer.
POS_INN_WARP = 4127803

function onTalk(target, player)
    originating_npc = target

    if target.object_id == ENPC_RYSSFLOH then
        player:play_scene(00000, HIDE_HOTBAR, {})
    elseif target.object_id == ENPC_GREHFARR then
        player:play_scene(00005, HIDE_HOTBAR, {})
    elseif target.object_id == ENPC_BADERON then
        player:play_scene(00011, SET_BASE | HIDE_HOTBAR, {})
    else
        player:finish_event()
    end
end

function onReturn(scene, results, player)
    -- Note that the scene yields, not returns - unlike the other openings.
    if scene == 1 then
        player:play_scene(00002, HIDE_HOTBAR, {})
        return
    elseif scene == 6 then
        -- Move the player into the destination position
        player:move_to_pop_range(POS_INN_WARP, true)
    elseif scene == 11 then
        player:play_scene(00012, HIDE_HOTBAR, {})
        return
    end

    player:finish_event()
end

function onYield(scene, id, results, player)
    if scene == 0 then
        -- first param: whether the quest was accepted
        local accepted = results[1] == 1
        if accepted then
            player:play_scene(00001, SET_BASE | HIDE_HOTBAR | DISABLE_SKIP, {})
            return
        else
            player:play_scene(00004, HIDE_HOTBAR, {})
            return
        end

    elseif scene == 2 then
        player:accept_quest(EVENT_ID)

        -- call back into the opening, presumably to update the borders of the play area
        player:start_event(OPENING_EVENT_HANDLER, EVENT_TYPE_NEST, 0)
        player:play_scene(30, HIDE_HOTBAR | NO_DEFAULT_CAMERA, {2})
        return
    elseif scene == 5 then
        if results[1] == 1 then
            -- take the warp
            player:play_scene(6, HIDE_HOTBAR, {})
            return
        end
    elseif scene == 12 then
        local completed = results[1] == 1
        if completed then
            player:finish_quest(EVENT_ID)
        end
    end

    player:finish_event()
end
