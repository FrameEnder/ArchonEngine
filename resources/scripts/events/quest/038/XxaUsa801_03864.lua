-- Quest: Ziz Is So Ridiculous (today's)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-01) and checked against Bob's Chronofoil capture
-- 412ba91d (2026-11-01, the whole quest: its fixture is tools/questkit/fixtures/3864.json):
-- Sequence 0:   ACTOR0 (Aideen) offers it (scene 0, a yield: QuestOffer, 1 when accepted; the client turns it down
--               itself while QST_ACCEPT_CHECK, the old Ziz Is So Ridiculous (785), is accepted), then scene 1, where
--               the client accepts it: sequence 1.
-- Sequence 1:   three of BNpcBase ENEMY0 (any of the open world's): each counts in UI8AL with notice 0 [n, 3]; the
--               third clears it and moves to sequence 255.
-- Sequence 255: ACTOR0 rewards it (scene 2, a yield: QuestReward, 1 and the optional item chosen). Retail then
--               completed the old Ziz Is So Ridiculous (785) too, with no rewards: FinishQuest 785 with 3864's.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_REWARD = 2

NOTICE_DEFEATED = 0
KILLS_NEEDED = 3
-- The quest this one replaced, completed with it (retail's FinishQuest 785 with 3864's.)
QUEST_ALSO_COMPLETED = 785

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
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 or base_id ~= ENEMY0 then
        return
    end
    local defeated = (player:get_quest_var(EVENT_ID, 0) & 0x0F) + 1
    player:event_notice(EVENT_ID, NOTICE_DEFEATED, { defeated, KILLS_NEEDED })
    if defeated >= KILLS_NEEDED then
        player:set_quest_var(EVENT_ID, 0, 0)
        player:quest_sequence(EVENT_ID, 255)
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
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID, results[2])
            player:mark_quest_completed(QUEST_ALSO_COMPLETED)
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
