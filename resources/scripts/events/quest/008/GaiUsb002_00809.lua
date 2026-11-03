-- Quest: Tales from the Tidus Slayer
--
-- Written from the client's quest script (kawari-questdump, 2026-11-03) and checked against Bob's Chronofoil captures
-- 44df02fc and c9145fdc (2026-11-03, the whole quest each: its fixtures are tools/questkit/fixtures/809.json and
-- 809-c9145fdc.json):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: QuestOffer, 1 when accepted), then the cutscene where the client
--               accepts it (scene 1, 0xF8402EFB): sequence 1.
-- Sequence 1:   EOBJECT0 (the nest): scene 2 (0x2001, empty), EVENT_ACTION_SEARCH's bar (retail's 2 s), then scene 3
--               (0x2001, the "they appear" message): ENEMY0-ENEMY3 come out (layer QB_GaiUsb002_001 of the Lower La
--               Noscea drop-in, retail's positions, levels and HP); each defeated counts in UI8AL (1-4, no notice). As
--               the fourth falls ENEMY4 comes out (retail: 0.3 s later); defeating it clears the variables: notice 0,
--               sequence 255.
-- Sequence 255: ACTOR0 rewards it (scene 4, a yield: QuestReward, 1.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_NEST = 2
SCENE_APPEAR = 3
SCENE_REWARD = 4

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB on retail
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

NOTICE_DEFEATED = 0

-- The first four, counted in UI8AL; the last one comes after them.
FIRST_FOUR = { ENEMY0, ENEMY1, ENEMY2, ENEMY3 }

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function counted(player)
    return player:get_quest_var(EVENT_ID, 0) & 0x0F
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == EOBJECT0 then
        play(player, SCENE_NEST, OBJECT_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- The search bar at the nest finished: the scene after it.
function onEventActionCast(player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 then
        player:finish_event()
        return
    end
    play(player, SCENE_APPEAR, OBJECT_FLAGS)
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 then
        return
    end
    for _, enemy in ipairs(FIRST_FOUR) do
        if layout_id == enemy then
            local count = counted(player) + 1
            player:set_quest_var(EVENT_ID, 0, (player:get_quest_var(EVENT_ID, 0) & 0xF0) | count)
            if count == #FIRST_FOUR then
                player:spawn_quest_bnpc(ENEMY4)
            end
            return
        end
    end
    if layout_id == ENEMY4 then
        player:set_quest_var(EVENT_ID, 0, 0)
        player:event_notice(EVENT_ID, NOTICE_DEFEATED, {})
        player:quest_sequence(EVENT_ID, 255)
    end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            play(player, SCENE_ACCEPTED, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_NEST then
        -- Searching the nest: the bar, then scene 3 (see onEventActionCast.)
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif scene == SCENE_APPEAR then
        for _, enemy in ipairs(FIRST_FOUR) do
            player:spawn_quest_bnpc(enemy)
        end
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
