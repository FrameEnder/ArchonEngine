-- Quest: The Reluctant Researcher (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-08) and checked against Bob's three Chronofoil
-- captures of 2026-11-08 (18a89a32, 4bcbd425 and de5beba7, each the whole quest; fixtures
-- tools/questkit/fixtures/962.json, 962-18a89a32.json, 962-de5beba7.json):
-- Sequence 0:   ACTOR0 (Ceana) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1, where the
--               client accepts it: sequence 1.
-- Sequence 1:   ACTOR1 (the ferry skipper) (scene 2) takes the player across: sequence 2 and POPRANGE0 in Western La
--               Noscea (TERRITORYTYPE0: retail's zone change to the same territory, as the talk finished.)
-- Sequence 2:   ACTOR2 (Ceana, across the water) (scene 3, 0x2001: she runs off and fades): notice 1 [ITEM0's icon],
--               sequence 255 with UI8BH = 1 (ITEM0 held.) ACTOR0 has a line meanwhile (4); the ferry skipper asks
--               whether to go across (scene 5, a yes/no, unseen on retail: "yes" is taken to take the player to
--               POPRANGE0 again, as scene 2 does.)
-- Sequence 255: ACTOR3 (Davyd) takes ITEM0 (scene 6, NpcTrade in event mode: a yield, 1 when handed over) and rewards
--               it (scene 7, a yield: QuestReward, 1.) The ferry skipper asks the same (scene 8.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_FERRY = 2
SCENE_CEANA = 3
SCENE_CEANA_LINE = 4
SCENE_FERRY_AGAIN = 5
SCENE_TRADE = 6
SCENE_REWARD = 7
SCENE_FERRY_AFTER = 8

FLAGS_CEANA = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

NOTICE_CEANA = 1
-- ITEM0's icon, as retail's notice carried it (all three captures.)
ITEM0_ICON = 22603
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
REWARD_CLOSED = 0x11

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
        play(player, SCENE_FERRY, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR2 then
        play(player, SCENE_CEANA, FLAGS_CEANA)
    elseif sequence == 2 and BASE_ID == ACTOR0 then
        play(player, SCENE_CEANA_LINE, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR1 then
        play(player, SCENE_FERRY_AGAIN, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR3 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_FERRY_AFTER, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function acrossTheWater(player)
    player:finish_event()
    player:change_territory_pop_range(TERRITORYTYPE0, POPRANGE0)
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
    elseif scene == SCENE_FERRY then
        player:quest_sequence(EVENT_ID, 2)
        acrossTheWater(player)
        return
    elseif scene == SCENE_FERRY_AGAIN or scene == SCENE_FERRY_AFTER then
        if results[1] == 1 then
            acrossTheWater(player)
            return
        end
    elseif scene == SCENE_CEANA then
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH: ITEM0 held
        player:event_notice(EVENT_ID, NOTICE_CEANA, { ITEM0_ICON })
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
