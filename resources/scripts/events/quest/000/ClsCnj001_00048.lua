-- Quest: Trial by Earth (Gridania)
--
-- Checked against the client's quest script (kawari-questdump) and Bob's Chronofoil capture b23b91ef (2026-10-26,
-- the whole quest; its fixture is tools/questkit/fixtures/48.json).
-- Sequence 0:   ACTOR0 (E-Sumi-Yan) offers the quest (scene 0, a yield returning 1 when accepted.)
-- Sequence 1:   ACTOR1 (in the North Shroud) explains (scene 1): sequence 2 and notice 0.
-- Sequence 2:   search EOBJECT0 (scene 2, then EVENT_ACTION_SEARCH's bar, then SEQ_2_EOBJECT0_EVENTACTIONOK, scene
--               100's message; all 0x2001, as retail). It brings out ENEMY0 (a layout ID: the North Shroud drop-in's
--               QB_ClsCnj001_001 layer, from retail's spawn), which retail sent just after the event finished, owned by
--               the player and naming EOBJECT0 as its trigger. Retail sets no variable for the search, so it can be
--               searched again while the enemy is around (only one comes out.) Killing it sends notice 1 and moves to
--               sequence 3.
-- Sequence 3:   ACTOR1's cutscene (scene 3, NQ_POINT_001; retail's flags 0xF8402FFF): notice 2 and sequence 255.
-- Sequence 255: ACTOR0's reward (scene 4, a yield returning 1 and the item chosen.)

SCENE_OFFER = 0
SCENE_EXPLAINED = 1
SCENE_SEARCH = 2
SCENE_CUTSCENE = 3
SCENE_REWARD = 4

-- Retail's flags (Bob's Chronofoil capture b23b91ef.)
SEARCH_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA
CUTSCENE_FLAGS = 0xF8402FFF

NOTICE_EXPLAINED = 0
NOTICE_DEFEATED = 1
NOTICE_CUTSCENE = 2

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
        play(player, SCENE_EXPLAINED, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == EOBJECT0 then
        -- The object's scene, the progress bar (see sceneFinished), then the "found it" scene once that finishes.
        play(player, SCENE_SEARCH, SEARCH_FLAGS)
    elseif sequence == 3 and BASE_ID == ACTOR1 then
        play(player, SCENE_CUTSCENE, CUTSCENE_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    if player:get_quest_sequence(EVENT_ID) ~= 2 then
        player:finish_event()
        return
    end
    play(player, SEQ_2_EOBJECT0_EVENTACTIONOK, SEARCH_FLAGS)
end

function onBNpcKill(base_id, layout_id, player)
    if layout_id ~= ENEMY0 or player:get_quest_sequence(EVENT_ID) ~= 2 then
        return
    end
    player:event_notice(EVENT_ID, NOTICE_DEFEATED, {})
    player:quest_sequence(EVENT_ID, 3)
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
    elseif scene == SCENE_EXPLAINED then
        player:event_notice(EVENT_ID, NOTICE_EXPLAINED, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_SEARCH then
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif scene == SEQ_2_EOBJECT0_EVENTACTIONOK then
        player:spawn_quest_bnpc(ENEMY0)
    elseif scene == SCENE_CUTSCENE then
        player:event_notice(EVENT_ID, NOTICE_CUTSCENE, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_REWARD then
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
