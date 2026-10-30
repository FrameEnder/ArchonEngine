-- Quest: Microbrewing
--
-- Written from the client's quest script (kawari-questdump, 2026-10-30) and checked against Bob's Chronofoil capture
-- caa5e908 (2026-10-30, the whole quest: its fixture is tools/questkit/fixtures/3861.json):
-- Sequence 0:   ACTOR0 (Teteroon) offers it (scene 0, a yield: QuestOffer, 1 when accepted; the client turns it down
--               itself while QST_ACCEPT_CHECK, the old Microbrewing (734), is accepted), then scene 1, where the client
--               accepts it: sequence 1.
-- Sequence 1:   coeurl pups (ENEMY0, BNpcBase 28, any of the open world's): each one defeated gives ITEM0, counted in
--               UI8BH (the client lists ITEM0 from it), notice 0 [count, 3, ITEM0's icon]; retail's three kills all
--               counted. At three, sequence 255 (UI8BH stays 3.)
-- Sequence 255: ACTOR0 takes them (scene 2, NpcTrade in event mode: a yield, 1 when handed over) and rewards it (scene 3,
--               a yield: QuestReward, 1.) Retail then completed the old Microbrewing (734) too: FinishQuest 3861, then
--               734.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_TRADE = 2
SCENE_REWARD = 3

NOTICE_PUP = 0
PUPS_NEEDED = 3
-- ITEM0's icon, as retail's notices carried it (Bob's Chronofoil capture caa5e908.)
ITEM0_ICON = 25919
-- The quest this one replaced, completed with it (retail's FinishQuest 734 right after 3861's.)
QUEST_ALSO_COMPLETED = 734

-- UI8BH: the pups defeated (and ITEM0 held.)
function pups(player)
    return player:get_quest_var(EVENT_ID, 1) >> 4
end

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
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onBNpcKill(base_id, layout_id, player)
    if base_id ~= ENEMY0 or player:get_quest_sequence(EVENT_ID) ~= 1 then
        return
    end
    local count = pups(player) + 1
    player:event_notice(EVENT_ID, NOTICE_PUP, { count, PUPS_NEEDED, ITEM0_ICON })
    player:set_quest_var(EVENT_ID, 1, (player:get_quest_var(EVENT_ID, 1) & 0x0F) | (count << 4))
    if count >= PUPS_NEEDED then
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
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_REWARD, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
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
