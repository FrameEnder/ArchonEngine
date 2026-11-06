-- Quest: Blood for Blood (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-06) and checked against Bob's Chronofoil captures
-- 130088ec and 3c56e09f (2026-11-06, the whole quest on each of his two characters: fixtures
-- tools/questkit/fixtures/940.json and `940-3c56e09f.json`):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1 (QuestAccepted):
--               sequence 1 with ITEM0 (UI8BH = 1; retail left it set to the end of sequence 4.)
-- Sequence 1:   ACTOR1 takes it (scene 2, NpcTrade, a yield: 1 when handed over) and talks (scene 3): notice 0,
--               sequence 2.
-- Sequence 2:   ACTOR2, the same (scenes 4, 5): notice 1, sequence 3.
-- Sequence 3:   ACTOR2 talked to (scene 6) asks for an emote; the right one on her (Emote 12, both characters) plays
--               scene 7: notice 2, sequence 4; another plays scene 8.
-- Sequence 4:   EOBJECT0: scene 9, EVENT_ACTION_SEARCH's bar, then notice 3, sequence 5 and scene 10 (both 0x2001;
--               retail moved the quest on as the bar ended, before scene 10.) ACTOR2 has a line (scene 11.)
-- Sequence 5:   EOBJECT1: scene 12, the bar, scene 13 (0x2001): notice 4 [ITEM1's icon], sequence 255 with ITEM1
--               (UI8BL = 1.) EOBJECT2's scenes (14, 15) are empty: a search with nothing to show.
-- Sequence 255: ACTOR3 takes it (scene 16, NpcTrade) and rewards it (scene 17, a yield: QuestReward, 1 and the
--               optional item picked; closing the window leaves UI8A 0x11.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_TRADE_ACTOR1 = 2
SCENE_AFTER_ACTOR1 = 3
SCENE_TRADE_ACTOR2 = 4
SCENE_AFTER_ACTOR2 = 5
SCENE_ASKS = 6
SCENE_EMOTE_RIGHT = 7
SCENE_EMOTE_WRONG = 8
SCENE_EOBJECT0 = 9
SCENE_ACTOR2_LINE = 11
SCENE_EOBJECT1 = 12
SCENE_EOBJECT2 = 14
SCENE_TRADE_FINAL = 16
SCENE_REWARD = 17

QUIET_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail
-- The emote ACTOR2 wants (Bob's Chronofoil captures of 2026-11-06: the Emote sheet's row.)
EMOTE_WANTED = 12
-- ITEM1's icon, on notice 4 (Bob's Chronofoil captures of 2026-11-06.)
ITEM1_ICON = 21002
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
REWARD_CLOSED = 0x11

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- The first scene of what's being searched, while its bar runs.
searching = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_TRADE_ACTOR1, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR2 then
        play(player, SCENE_TRADE_ACTOR2, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR2 then
        play(player, SCENE_ASKS, HIDE_HOTBAR)
    elseif sequence == 4 and BASE_ID == EOBJECT0 then
        searching = SCENE_EOBJECT0
        play(player, SCENE_EOBJECT0, QUIET_FLAGS)
    elseif sequence == 4 and BASE_ID == ACTOR2 then
        play(player, SCENE_ACTOR2_LINE, HIDE_HOTBAR)
    elseif sequence == 5 and BASE_ID == EOBJECT1 then
        searching = SCENE_EOBJECT1
        play(player, SCENE_EOBJECT1, QUIET_FLAGS)
    elseif sequence == 5 and BASE_ID == EOBJECT2 then
        searching = SCENE_EOBJECT2
        play(player, SCENE_EOBJECT2, QUIET_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR3 then
        play(player, SCENE_TRADE_FINAL, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEmote(emote, target, player)
    if player:get_quest_sequence(EVENT_ID) == 3 and BASE_ID == ACTOR2 then
        if emote == EMOTE_WANTED then
            play(player, SCENE_EMOTE_RIGHT, HIDE_HOTBAR)
        else
            play(player, SCENE_EMOTE_WRONG, HIDE_HOTBAR)
        end
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    if searching == nil or current_scene ~= nil then
        player:finish_event()
        return
    end
    if searching == SCENE_EOBJECT0 then
        player:event_notice(EVENT_ID, 3, {})
        player:quest_sequence(EVENT_ID, 5)
    end
    play(player, searching + 1, QUIET_FLAGS)
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
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH = 1: ITEM0
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_TRADE_ACTOR1 or scene == SCENE_TRADE_ACTOR2 then
        if results[1] == 1 then
            play(player, scene + 1, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_AFTER_ACTOR1 then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_AFTER_ACTOR2 then
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_EMOTE_RIGHT then
        player:event_notice(EVENT_ID, 2, {})
        player:quest_sequence(EVENT_ID, 4)
    elseif searching ~= nil and scene == searching then
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif searching ~= nil and scene == searching + 1 then
        local searched = searching
        searching = nil
        if searched == SCENE_EOBJECT1 then
            player:event_notice(EVENT_ID, 4, { ITEM1_ICON })
            player:set_quest_var(EVENT_ID, 1, 0x11) -- and UI8BL = 1: ITEM1
            player:quest_sequence(EVENT_ID, 255)
        end
    elseif scene == SCENE_TRADE_FINAL then
        if results[1] == 1 then
            play(player, SCENE_REWARD, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_event()
            player:finish_quest(EVENT_ID, results[2] or 0)
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
