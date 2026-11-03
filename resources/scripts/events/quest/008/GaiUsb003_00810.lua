-- Quest: Hungry Hungry Goobbues
--
-- Written from the client's quest script (kawari-questdump, 2026-11-03) and checked against Bob's Chronofoil captures
-- 44df02fc and c9145fdc (2026-11-03, the whole quest each: its fixtures are tools/questkit/fixtures/810.json and
-- 810-c9145fdc.json):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1, where the client
--               accepts it: sequence 1.
-- Sequence 1:   EOBJECT0 (the destination): scene 2 (0x2001, empty), EVENT_ACTION_LOOKOUT_LONG's bar (retail's
--               10 s), then scene 3 (0x2001, the "it appears" message): ENEMY0 comes out (layer QB_GaiUsb003_001 of
--               the Lower La Noscea drop-in). Defeating it: notice 0, sequence 255.
-- Sequence 255: ACTOR0 rewards it (scene 14, a yield: QuestReward, 1 and the optional item chosen.)
-- EOBJECT1-EOBJECT5 (more of the client's listeners) and the empty scenes 4-13 and 15-24 weren't used on retail.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_LOOKOUT = 2
SCENE_APPEAR = 3
SCENE_REWARD = 14

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

-- The lookout's bar finished: the scene after it.
function onEventActionCast(player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 then
        player:finish_event()
        return
    end
    play(player, SCENE_APPEAR, OBJECT_FLAGS)
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
        -- Keeping a lookout: the bar, then scene 3 (see onEventActionCast.)
        player:event_action(EVENT_ACTION_LOOKOUT_LONG)
        return
    elseif scene == SCENE_APPEAR then
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
