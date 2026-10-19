-- Quest: Heir Today, Gone Tomorrow (Ul'dah, Central Thanalan)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-19) and retail (Bob's Chronofoil capture of
-- 2026-10-18, questkit replay 3853):
-- Sequence 0:   ACTOR0 (Leofric) offers it (scene 0, a yield: the client's QuestOffer, 1 when accepted), then scene 1;
--               accepted after it, sequence 1. The client's scene 0 refuses on its own while QST_ACCEPT_CHECK is
--               accepted.
-- Sequence 1:   walking into EVENTRANGE0 (offered while BitFlag8 1 is unset, which retail never sets) plays scene 2
--               (0x2001; the client's script only shows a pop-up message if someone else triggered the enemy), and
--               when it returns ENEMY0, a mildewed goobbue (a layout ID), comes out after the player. Its death sends
--               notice 0 (no arguments) and moves to 255; no variables change on retail. The client also offers
--               ENEMY0 while UI8AL < 1 and EOBJECT0 (never touched on retail), but scenes 3 and 4 are empty: the
--               event just ends.
-- Sequence 255: ACTOR0 rewards it (scene 5, a yield: QuestReward, 1 and the chosen item).
-- The goobbue is layer QB_XxaWil083_001 of the Central Thanalan drop-in, at retail's position, level 12 and HP 243.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_AMBUSH = 2
SCENE_FINISH = 5
FLAGS_AMBUSH = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

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
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_FINISH, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEnterTrigger(player, arg)
    if player:get_quest_sequence(EVENT_ID) == 1 then
        play(player, SCENE_AMBUSH, FLAGS_AMBUSH)
    else
        player:finish_event()
    end
end

function onBNpcKill(base_id, layout_id, player)
    if layout_id == ENEMY0 and player:get_quest_sequence(EVENT_ID) == 1 then
        player:event_notice(EVENT_ID, NOTICE_DEFEATED)
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
            play(player, SCENE_ACCEPTED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_AMBUSH then
        -- Retail brought it out after the event ended (0.4 s after its EventFinish).
        player:finish_event()
        player:spawn_quest_bnpc(ENEMY0)
        return
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
