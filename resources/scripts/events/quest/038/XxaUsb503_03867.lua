-- Quest: You Can't Take It with You (main scenario; the quest that stands in for 871, 872 and 881)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-06) and checked against Bob's Chronofoil captures
-- 650dbfd8, 0fd8bce0 and 48aaf6fe (2026-11-06, the whole quest each: fixtures tools/questkit/fixtures/3867.json,
-- `3867-0fd8bce0.json`, `3867-48aaf6fe.json`):
-- Sequence 0:   ACTOR0 (Marques, in Eastern Thanalan) offers it (scene 0, a yield: QuestOffer, 1 when accepted; the
--               client turns it down itself while QST_ACCEPT_CHECK_01 or _02, the old 871 and 872, is in the
--               journal), then talks (scene 1, which ends with QuestAccepted): sequence 1 with ITEM0 (UI8CH = 1.)
-- Sequence 1:   ACTOR1 (the traveling goldsmith) takes ITEM0 (scene 2, NpcTrade in event mode: a yield, 1 when handed
--               over) and talks (scene 3): notice 0 [the icon retail sent], sequence 2 with UI8BH = 1 and UI8BL = 1
--               (the two key items the client lists then.)
-- Sequence 2:   ACTOR0 takes one (scene 4, NpcTrade) and the cutscene (scene 5, 0xF8402EFB): notice 1 [its icon],
--               sequence 255 with UI8BH = 1. ACTOR1 has a line meanwhile (scene 6.)
-- Sequence 255: ACTOR2 (Eluned) takes the last (scene 7, NpcTrade) and rewards it (scene 8, a yield: QuestReward, 1.)
--               Retail then completed 872 and 881 too, with no rewards (FinishQuest 3867, 872, 881, all three runs),
--               as the newer quest stands in for them.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_GOLDSMITH_TRADE = 2
SCENE_GOLDSMITH = 3
SCENE_MARQUES_TRADE = 4
SCENE_MARQUES = 5
SCENE_ELUNED_TRADE = 7
SCENE_REWARD = 8

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB on retail

-- The icons the notices carried on retail (Bob's Chronofoil captures of 2026-11-06.)
NOTICE_GOLDSMITH_ICON = 23103
NOTICE_MARQUES_ICON = 21002
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
REWARD_CLOSED = 0x11
-- Completed with this one, as retail did.
QUESTS_ALSO_COMPLETED = { 872, 881 }

LINES = {
    [2] = { [ACTOR1] = 6 },
}

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
        play(player, SCENE_GOLDSMITH_TRADE, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR0 then
        play(player, SCENE_MARQUES_TRADE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_ELUNED_TRADE, HIDE_HOTBAR)
    elseif sequence ~= nil and LINES[sequence] ~= nil and LINES[sequence][BASE_ID] ~= nil then
        play(player, LINES[sequence][BASE_ID], HIDE_HOTBAR)
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
        player:set_quest_var(EVENT_ID, 2, 0x10) -- UI8CH = 1: ITEM0
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_GOLDSMITH_TRADE then
        if results[1] == 1 then
            play(player, SCENE_GOLDSMITH, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_GOLDSMITH then
        player:event_notice(EVENT_ID, 0, { NOTICE_GOLDSMITH_ICON })
        player:set_quest_var(EVENT_ID, 2, 0)
        player:set_quest_var(EVENT_ID, 1, 0x11) -- UI8BH = 1, UI8BL = 1
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_MARQUES_TRADE then
        if results[1] == 1 then
            play(player, SCENE_MARQUES, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_MARQUES then
        player:event_notice(EVENT_ID, 1, { NOTICE_MARQUES_ICON })
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH = 1: the one left
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_ELUNED_TRADE then
        if results[1] == 1 then
            play(player, SCENE_REWARD, HIDE_HOTBAR)
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
