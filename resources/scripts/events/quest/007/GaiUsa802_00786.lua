-- Quest: Seeing Eye to Winged Eye
--
-- Written from the client's quest script (kawari-questdump, 2026-11-01) and checked against Bob's Chronofoil capture
-- 412ba91d (2026-11-01, the whole quest, turned down once first: its fixture is tools/questkit/fixtures/786.json):
-- Sequence 0:   ACTOR0 (Ivaurault) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1, where the
--               client accepts it: sequence 1.
-- Sequence 1:   EOBJECT0 (the destination): scene 2 (0x2001, empty), EVENT_ACTION_LOOKOUT_LONG's bar (10 s on
--               retail), then scene 3 (0x2001, the client's pop message), which brings out ENEMY0 (layer
--               QB_GaiUsa802_001 of the North Shroud drop-in, at retail's position, level and HP; timeline
--               GaiUsa802Enemy_1899), the player's own. Its death: notice 0, sequence 255. The look-out was broken
--               off in two of three runs (moving 8.7 s in, capture 412ba91d; paralysis 0.9 s in, b166797f): the server
--               ends the event then, with no scene 3, and it's done again.
-- Sequence 255: ACTOR0 rewards it (scene 4, a yield: QuestReward, 1 and the optional item chosen.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_LOOKOUT = 2
SCENE_SPOTTED = 3
SCENE_REWARD = 4

OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

NOTICE_DEFEATED = 0

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
    elseif sequence == 1 and BASE_ID == EOBJECT0 then
        play(player, SCENE_LOOKOUT, OBJECT_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- The look-out bar finished: the scene after it.
function onEventActionCast(player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 then
        player:finish_event()
        return
    end
    play(player, SCENE_SPOTTED, OBJECT_FLAGS)
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 or layout_id ~= ENEMY0 then
        return
    end
    player:event_notice(EVENT_ID, NOTICE_DEFEATED, {})
    player:quest_sequence(EVENT_ID, 255)
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
    elseif scene == SCENE_LOOKOUT then
        -- The look-out: the bar, then scene 3 (see onEventActionCast.)
        player:event_action(EVENT_ACTION_LOOKOUT_LONG)
        return
    elseif scene == SCENE_SPOTTED then
        -- One still out isn't brought out again.
        player:spawn_quest_bnpc(ENEMY0)
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
