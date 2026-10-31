-- Quest: Wilred Wants You
--
-- Written from the client's quest script (kawari-questdump, 2026-10-31) and checked against Bob's Chronofoil capture
-- 20ef0e90 (2026-10-31, the whole quest: its fixture is tools/questkit/fixtures/778.json):
-- Sequence 0:   ACTOR0 (Hremfing) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1, where the
--               client accepts it: sequence 1.
-- Sequence 1:   ACTOR1 (Wilred) talks (scene 2, a pop-up message) and brings out ENEMY0 and ENEMY1 (layer
--               QB_GaiUsa705_001 of the Southern Thanalan drop-in, at retail's positions, levels and HP; timelines
--               GaiUsa705Enemy_2257 and _2258), the player's own (the client's scene 2 is shorter while they're out:
--               IsBattleNpcTriggerOwner). UI8AL counts them down; the first: UI8AL = 1, the second: the variables
--               cleared, notice 0, sequence 2. Retail's ENEMY0 fell first, 20 s after they came out, ENEMY1 20 s later.
-- Sequence 2:   ACTOR1 talks (scene 3, retail's flags 0x2001) and runs off: notice 1, sequence 255.
-- Sequence 255: ACTOR2 (Gundobald) rewards it (scene 4, a yield: QuestReward, 1 and the optional item chosen.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_WILRED = 2
SCENE_WILRED_AFTER = 3
SCENE_REWARD = 4

-- HIDE_HOTBAR | NO_DEFAULT_CAMERA, retail's for scene 3 (Bob's Chronofoil capture 20ef0e90.)
SCENE_3_FLAGS = 0x2001

NOTICE_DEFEATED = 0
NOTICE_WILRED = 1

-- The enemies to defeat (UI8AL counts them.)
ENEMIES_TO_DEFEAT = 2

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
        play(player, SCENE_WILRED, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR1 then
        play(player, SCENE_WILRED_AFTER, SCENE_3_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 or (layout_id ~= ENEMY0 and layout_id ~= ENEMY1) then
        return
    end
    local defeated = (player:get_quest_var(EVENT_ID, 0) & 0x0F) + 1
    if defeated >= ENEMIES_TO_DEFEAT then
        for byte = 0, 5 do
            player:set_quest_var(EVENT_ID, byte, 0)
        end
        player:event_notice(EVENT_ID, NOTICE_DEFEATED, {})
        player:quest_sequence(EVENT_ID, 2)
    else
        player:set_quest_var(EVENT_ID, 0, (player:get_quest_var(EVENT_ID, 0) & 0xF0) | defeated)
    end
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
    elseif scene == SCENE_WILRED then
        -- One still out isn't brought out again (either counts: the client accepts both while UI8AL < 2.)
        player:spawn_quest_bnpc(ENEMY0)
        player:spawn_quest_bnpc(ENEMY1)
    elseif scene == SCENE_WILRED_AFTER then
        player:event_notice(EVENT_ID, NOTICE_WILRED, {})
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
