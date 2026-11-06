-- Quest: The Final Flight of the Enterprise (main scenario; the quest that stands in for 917)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-06) and checked against Bob's Chronofoil captures
-- 130088ec and 3c56e09f (2026-11-06, the whole quest on each of his two characters: fixtures
-- tools/questkit/fixtures/3868.json and `3868-3c56e09f.json`):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: QuestOffer, 1 when accepted; the client turns it down itself while
--               QST_ACCEPT_CHECK, the old 917, is in the journal), then scene 1 (QuestAccepted): sequence 1.
-- Sequence 1:   ACTOR1 (scene 2, 0x2001, the NPC walking off): notice 0, sequence 2.
-- Sequence 2:   ACTOR0 (scene 3): notice 1 [the icon retail sent], sequence 3 with ITEM0 (UI8BH = 1.)
-- Sequence 3:   ACTOR2 (scene 4): notice 2 [the icon], sequence 255 with ITEM1 too (UI8BL = 1.)
-- Sequence 255: ACTOR3 takes them (scene 5, NpcTrade, a yield: 1 when handed over) and rewards it (scene 6, a yield:
--               QuestReward, 1; closing the window leaves UI8A 0x11.) Retail then completed 917 too, with no rewards
--               (FinishQuest 3868 then 917, both characters.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_ACTOR1 = 2
SCENE_ACTOR0 = 3
SCENE_ACTOR2 = 4
SCENE_TRADE = 5
SCENE_REWARD = 6

QUIET_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail
-- The icon on notices 1 and 2 (Bob's Chronofoil captures of 2026-11-06.)
NOTICE_ICON = 25918
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
REWARD_CLOSED = 0x11
-- Completed with this one, as retail did.
QUEST_ALSO_COMPLETED = 917

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
        play(player, SCENE_ACTOR1, QUIET_FLAGS)
    elseif sequence == 2 and BASE_ID == ACTOR0 then
        play(player, SCENE_ACTOR0, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR2 then
        play(player, SCENE_ACTOR2, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR3 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
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
    elseif scene == SCENE_ACTOR1 then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_ACTOR0 then
        player:event_notice(EVENT_ID, 1, { NOTICE_ICON })
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH = 1: ITEM0
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_ACTOR2 then
        player:event_notice(EVENT_ID, 2, { NOTICE_ICON })
        player:set_quest_var(EVENT_ID, 1, 0x11) -- and UI8BL = 1: ITEM1
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_REWARD, HIDE_HOTBAR)
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
