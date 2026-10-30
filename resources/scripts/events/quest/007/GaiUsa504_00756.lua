-- Quest: Where There Is Smoke
--
-- Written from the client's quest script (kawari-questdump, 2026-10-30) and checked against Bob's Chronofoil capture
-- 9df4ff01 (2026-10-30, the whole quest: its fixture is tools/questkit/fixtures/756.json):
-- Sequence 0:   ACTOR0 (Hihibaru) offers it (scene 0, a yield: QuestOffer, 1 when accepted; the client turns it down
--               itself until QST_ACCEPT_CHECK, Ratting It Out (752), is complete), then scene 1, where the client
--               accepts it: sequence 1, with UI8BH = 1 (ITEM0 held; the client lists it from there.)
-- Sequence 1:   EOBJECT0: talking to it plays scene 2 (0x2001), which only opens the inventory. Using ITEM0 on it (the
--               server's EventItem event) plays scene 3 (0x2001, a pop-up message) and ENEMY0, a dusk bat, comes out
--               (layer QB_GaiUsa504_001 of the Eastern Thanalan drop-in, at retail's position, level and HP).
--               Defeating it: notice 0 [ITEM1's icon], sequence 255 (UI8BH stays 1: now ITEM1 held.)
-- Sequence 255: ACTOR0 takes ITEM1 (scene 4, NpcTrade in event mode: a yield, 1 when handed over) and rewards it
--               (scene 5, a yield: QuestReward, 1.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_DESTINATION = 2
SCENE_ITEM_USED = 3
SCENE_TRADE = 4
SCENE_REWARD = 5

FLAGS_OBJECT = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

NOTICE_BAT = 0
-- ITEM1's icon, as retail's notice carried it (Bob's Chronofoil capture 9df4ff01.)
ITEM1_ICON = 26151

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
        play(player, SCENE_DESTINATION, FLAGS_OBJECT)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEventItem(item_id, target, player)
    if item_id == ITEM0 and BASE_ID == EOBJECT0 and player:get_quest_sequence(EVENT_ID) == 1 then
        play(player, SCENE_ITEM_USED, FLAGS_OBJECT)
    else
        player:finish_event()
    end
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 or layout_id ~= ENEMY0 then
        return
    end
    player:event_notice(EVENT_ID, NOTICE_BAT, { ITEM1_ICON })
    player:quest_sequence(EVENT_ID, 255)
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
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH: ITEM0 held
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_ITEM_USED then
        player:spawn_quest_bnpc(ENEMY0)
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_REWARD, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
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
