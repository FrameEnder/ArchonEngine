-- Quest: He Ain't Heavy
--
-- Written from the client's quest script (kawari-questdump, 2026-10-31) and checked against Bob's Chronofoil capture
-- a0149e5e (2026-10-31, the whole quest: its fixture is tools/questkit/fixtures/775.json):
-- Sequence 0:   ACTOR0 offers it and the client accepts it in the same scene (scene 0, a yield: QuestOffer, 1 when
--               accepted): sequence 1.
-- Sequence 1:   ACTOR1 talks (scene 1): notice 0, sequence 2.
-- Sequence 2:   ACTOR0 talks (scene 2): notice 1, sequence 3.
-- Sequence 3:   ACTOR2 (scene 3, a pop-up message) brings out ENEMY0 (layer QB_GaiUsa702_001 of the South Shroud
--               drop-in, at retail's position, level and HP; timeline GaiUsa702Enemy_1909). Defeating it: notice 2,
--               sequence 4.
-- Sequence 4:   ACTOR2's cutscene (scene 4, retail's flags 0xF8406EFB): notice 3, sequence 255.
-- Sequence 255: ACTOR0 rewards it (scene 5, a yield: QuestReward, 1 and the optional item chosen.)

SCENE_OFFER = 0
SCENE_ACTOR1 = 1
SCENE_ACTOR0 = 2
SCENE_ACTOR2 = 3
SCENE_CUTSCENE = 4
SCENE_REWARD = 5

-- SET_BASE | HIDE_HOTBAR | INVINCIBLE, retail's for the cutscene (Bob's Chronofoil capture a0149e5e.)
CUTSCENE_FLAGS = 0xF8406EFB

NOTICE_ACTOR1 = 0
NOTICE_ACTOR0 = 1
NOTICE_DEFEATED = 2
NOTICE_CUTSCENE = 3

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
        play(player, SCENE_ACTOR1, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR0 then
        play(player, SCENE_ACTOR0, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR2 then
        play(player, SCENE_ACTOR2, HIDE_HOTBAR)
    elseif sequence == 4 and BASE_ID == ACTOR2 then
        play(player, SCENE_CUTSCENE, CUTSCENE_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 3 or layout_id ~= ENEMY0 then
        return
    end
    player:event_notice(EVENT_ID, NOTICE_DEFEATED, {})
    player:quest_sequence(EVENT_ID, 4)
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
    elseif scene == SCENE_ACTOR1 then
        player:event_notice(EVENT_ID, NOTICE_ACTOR1, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_ACTOR0 then
        player:event_notice(EVENT_ID, NOTICE_ACTOR0, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_ACTOR2 then
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
