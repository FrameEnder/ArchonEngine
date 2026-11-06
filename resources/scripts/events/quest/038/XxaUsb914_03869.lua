-- Quest: The Best Inventions (main scenario; the quest that stands in for 937)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-06) and checked against Bob's Chronofoil captures
-- 130088ec and 3c56e09f (2026-11-06, the whole quest on each of his two characters: fixtures
-- tools/questkit/fixtures/3869.json and `3869-3c56e09f.json`):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: QuestOffer, 1 when accepted; the client turns it down itself while
--               QST_ACCEPT_CHECK_01, _02 or _03 is in the journal), then scene 1 (QuestAccepted): sequence 1.
-- Sequence 1:   any three ENEMY0 (a BNpcBase, 132, the zone's own) defeated: each an ITEM0 (UI8BH, the count) and
--               notice 0 [count, 3, ITEM0's icon]; the third: sequence 255 (UI8BH 3.)
-- Sequence 255: ACTOR0 takes them (scene 2, NpcTrade, a yield: 1 when handed over), then the cutscene and reward
--               (scene 3, retail's 0xF8402FFF, a yield: QuestReward, 1; closing the window leaves UI8A 0x11.) Retail
--               then completed 937 too, with no rewards (FinishQuest 3869 then 937, both characters.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_TRADE = 2
SCENE_REWARD = 3

CUTSCENE_FLAGS = 0xF8402FFF -- retail's (Bob's Chronofoil captures of 2026-11-06)
NEEDED = 3
-- ITEM0's icon, on the notices (Bob's Chronofoil captures of 2026-11-06.)
ITEM0_ICON = 21266
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
REWARD_CLOSED = 0x11
-- Completed with this one, as retail did.
QUEST_ALSO_COMPLETED = 937

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
    if player:get_quest_sequence(EVENT_ID) ~= 1 or base_id ~= ENEMY0 then
        return
    end
    local count = (player:get_quest_var(EVENT_ID, 1) >> 4) + 1 -- UI8BH
    player:set_quest_var(EVENT_ID, 1, count << 4)
    player:event_notice(EVENT_ID, 0, { count, NEEDED, ITEM0_ICON })
    if count >= NEEDED then
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
            play(player, SCENE_REWARD, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_event()
            player:finish_quest(EVENT_ID, results[2] or 0)
            player:mark_quest_completed(QUEST_ALSO_COMPLETED)
            return
        elseif player:get_quest_var(EVENT_ID, 0) ~= REWARD_CLOSED then
            player:set_quest_var(EVENT_ID, 0, REWARD_CLOSED)
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
