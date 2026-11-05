-- Quest: Coming to Ul'dah
-- NOTE: These openings are so similar, ensure changes are synced between all of them!
--
-- Checked against the client's quest script (kawari-questdump) and retail: a Kawari Recorder export (2026-10-17, a
-- fresh Ul'dah character) and Bob's Chronofoil captures of the same session (2026-10-17, re-checked 2026-10-18): its
-- fixture is tools/questkit/fixtures/594.json.

-- scene 0: show quest prompt
-- scene 3: decline
-- scene 1: after quest acceptance
-- scene 2: you have begun a quest!
-- scene 4: momodi's cutscene
-- scene 5: accept reward dialog
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-19) and retail (questkit replay 594): the scenes
-- above are the client's 0-5; its scenes 6-13 are empty. In sequence 255 the client also lets you talk to EOBJECT0-3
-- (IsAcceptEvent), which this script just ends. The reward scene only completes the quest when it returns 1
-- (QuestReward).

ENPC_WYMOND = 3969639
ENPC_MOMODI = 3969632

function onTalk(target, player)
    originating_npc = target

    if target.object_id == ENPC_WYMOND then
        player:play_scene(00000, HIDE_HOTBAR, {})
    elseif target.object_id == ENPC_MOMODI then
        player:play_scene(00004, SET_BASE | HIDE_HOTBAR, {})
    else
        player:finish_event()
    end
end

function onReturn(scene, results, player)
    if scene == 4 then
        player:play_scene(00005, HIDE_HOTBAR, {})
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
            player:play_scene(00003, HIDE_HOTBAR, {})
            return
        end
    elseif scene == 1 then
        player:play_scene(00002, HIDE_HOTBAR, {})
        return
    elseif scene == 2 then
        player:accept_quest(EVENT_ID)

        -- call back into the opening, presumably to update the borders of the play area
        player:start_event(OPENING_EVENT_HANDLER, EVENT_TYPE_NEST, 0)
        player:play_scene(30, HIDE_HOTBAR | NO_DEFAULT_CAMERA, {2})
        return
    elseif scene == 5 then
        local completed = results[1] == 1
        if completed then
            player:finish_quest(EVENT_ID)
        end
    end

    player:finish_event()
end
