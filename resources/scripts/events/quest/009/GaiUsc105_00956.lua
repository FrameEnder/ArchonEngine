-- Quest: With the Utmost Care (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-08) and checked against Bob's three Chronofoil
-- captures of 2026-11-08 (18a89a32, 4bcbd425 and de5beba7, each the whole quest; fixtures
-- tools/questkit/fixtures/956.json, 956-18a89a32.json, 956-de5beba7.json):
-- Sequence 0:   ACTOR0 (Hahasako) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1, where the
--               client accepts it: sequence 1, with UI8BH = 1 and UI8BL = 1 (ITEM0 and ITEM1 held.)
-- Sequence 1:   EOBJECT0 (layout 4299848): talking to it plays scene 2 (0x2001), which only opens the inventory.
--               Using ITEM0 on it (the server's EventItem event) plays scene 3 (0x2001): notice 0, sequence 2, UI8BL
--               cleared (UI8BH = 1 is now ITEM1, as the client's GetEventItems reads it in sequence 2.)
-- Sequence 2:   EOBJECT1 (layout 4335308): talking plays scene 4 (the inventory); using ITEM1 on it plays scene 5
--               (0x2001): notice 1 [ITEM2's icon], sequence 3 (UI8BH = 1 is now ITEM2.)
-- Sequence 3:   ACTOR0 takes ITEM2 (scene 6, NpcTrade in event mode: a yield, 1 when handed over), then scene 7:
--               notice 2, sequence 255 (UI8BH = 1 stays: ITEM2, by the client's GetEventItems in sequence 255.)
-- Sequence 255: ACTOR1 (Lamberteint) takes it (scene 8, NpcTrade: a yield, 1) and rewards it (scene 9, a yield:
--               QuestReward, 1.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_CLUSTER = 2
SCENE_CLUSTER_USED = 3
SCENE_CRYSTAL = 4
SCENE_CRYSTAL_USED = 5
SCENE_HAHASAKO_TRADE = 6
SCENE_HAHASAKO_TAKEN = 7
SCENE_TRADE = 8
SCENE_REWARD = 9

FLAGS_OBJECT = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

NOTICE_CLUSTER = 0
NOTICE_CRYSTAL = 1
NOTICE_HAHASAKO = 2
-- ITEM2's icon, as retail's notice carried it (all three captures.)
ITEM2_ICON = 22603
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
    elseif sequence == 1 and BASE_ID == EOBJECT0 then
        play(player, SCENE_CLUSTER, FLAGS_OBJECT)
    elseif sequence == 2 and BASE_ID == EOBJECT1 then
        play(player, SCENE_CRYSTAL, FLAGS_OBJECT)
    elseif sequence == 3 and BASE_ID == ACTOR0 then
        play(player, SCENE_HAHASAKO_TRADE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEventItem(item_id, target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    if sequence == 1 and item_id == ITEM0 and BASE_ID == EOBJECT0 then
        play(player, SCENE_CLUSTER_USED, FLAGS_OBJECT)
    elseif sequence == 2 and item_id == ITEM1 and BASE_ID == EOBJECT1 then
        play(player, SCENE_CRYSTAL_USED, FLAGS_OBJECT)
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
        player:set_quest_var(EVENT_ID, 1, 0x11) -- UI8BH and UI8BL: ITEM0 and ITEM1 held
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_CLUSTER_USED then
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH: ITEM1 now
        player:event_notice(EVENT_ID, NOTICE_CLUSTER, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_CRYSTAL_USED then
        player:event_notice(EVENT_ID, NOTICE_CRYSTAL, { ITEM2_ICON })
        player:quest_sequence(EVENT_ID, 3) -- UI8BH: ITEM2 now
    elseif scene == SCENE_HAHASAKO_TRADE then
        if results[1] == 1 then
            play(player, SCENE_HAHASAKO_TAKEN, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_HAHASAKO_TAKEN then
        player:event_notice(EVENT_ID, NOTICE_HAHASAKO, {})
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
