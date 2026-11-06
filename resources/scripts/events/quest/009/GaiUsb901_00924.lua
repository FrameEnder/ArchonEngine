-- Quest: Ye of Little Faith (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-06) and checked against Bob's Chronofoil captures
-- 130088ec and 3c56e09f (2026-11-06, the whole quest on each of his two characters: fixtures
-- tools/questkit/fixtures/924.json and `924-3c56e09f.json`):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1 (QuestAccepted):
--               sequence 1 with ITEM0 and ITEM1 (UI8BH = UI8BL = 1.)
-- Sequence 1:   ACTOR1 (scene 2): notice 0, sequence 2.
-- Sequence 2:   ACTOR2 (scene 3): notice 1, sequence 3.
-- Sequence 3:   ACTOR3 (scene 4): notice 2, sequence 255. The items stay throughout.
-- Sequence 255: ACTOR4 takes them (scene 5, NpcTrade, a yield: 1 when handed over), then the cutscene and reward
--               (scene 6, 0xF8402EFB, a yield: QuestReward, 1; closing the window leaves UI8A 0x11.) Retail completed
--               Opportunity Knocks (925) and All by Ourselves (926) with it, without rewards (FinishQuest 924, 925,
--               926, both characters.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_TRADE = 5
SCENE_REWARD = 6

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB on retail
-- The talks of sequences 1-3: who, their scene and notice.
TALKS = {
    [1] = { actor = ACTOR1, scene = 2, notice = 0 },
    [2] = { actor = ACTOR2, scene = 3, notice = 1 },
    [3] = { actor = ACTOR3, scene = 4, notice = 2 },
}
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
REWARD_CLOSED = 0x11
-- Completed with this one, as retail did.
QUESTS_ALSO_COMPLETED = { 925, 926 }

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local talk = sequence ~= nil and TALKS[sequence] or nil

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif talk ~= nil and BASE_ID == talk.actor then
        play(player, talk.scene, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR4 then
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
    local sequence = player:get_quest_sequence(EVENT_ID)
    local talk = sequence ~= nil and TALKS[sequence] or nil

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            play(player, SCENE_ACCEPTED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:set_quest_var(EVENT_ID, 1, 0x11) -- UI8BH = UI8BL = 1: ITEM0, ITEM1
        player:quest_sequence(EVENT_ID, 1)
    elseif talk ~= nil and scene == talk.scene then
        player:event_notice(EVENT_ID, talk.notice, {})
        player:quest_sequence(EVENT_ID, sequence == 3 and 255 or sequence + 1)
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_REWARD, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_event()
            player:finish_quest(EVENT_ID, results[2] or 0)
            for _, quest in ipairs(QUESTS_ALSO_COMPLETED) do
                player:mark_quest_completed(quest)
            end
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
