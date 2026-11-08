-- Quest: The Curious Case of Giggity (main scenario; it took the place of 968, 971 and 974, which it completes)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-08) and checked against Bob's three Chronofoil
-- captures of 2026-11-08 (d7993cf0, 195176f8 and 0d10bf45, each the whole quest; fixtures
-- tools/questkit/fixtures/3870.json, 3870-195176f8.json, 3870-0d10bf45.json):
-- Sequence 0:   ACTOR0 (Ceana) offers it (scene 0, a yield: QuestOffer, 1 when accepted; the client turns it down
--               itself while any of QST_ACCEPT_CHECK_01-03, the three quests it replaces, is underway), then scene 1,
--               where the client accepts it: sequence 1.
-- Sequence 1:   ACTOR1 (Hedyn, in Old Gridania) talks (scene 2): ITEM0 (UI8BH = 1), notice 0 with its icon, sequence 2.
-- Sequence 2:   EOBJECT0 (the destination, in the Central Shroud): talked to, it only opens the inventory (scene 3,
--               0x2001); ITEM0 used on it (`onEventItem`) plays scene 4 (0x2001, the item its parameter; the client's
--               pop-up message) and
--               ENEMY0 comes out as the event ends (layer QB_XxaUsc307_001 of the Central Shroud drop-in). Its fall:
--               notice 1 with ITEM1's icon, sequence 255 (UI8BH stays 1: ITEM1 there.)
-- Sequence 255: ACTOR1 takes ITEM1 (scene 7, NpcTrade in event mode: a yield, 1 when handed over) and rewards it
--               (scene 8, a yield: QuestReward, 1); then 968, 971 and 974 are completed too, as retail's FinishQuest
--               for each right after this one's.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_HEDYN = 2
SCENE_DESTINATION = 3
SCENE_ITEM_USED = 4
SCENE_TRADE = 7
SCENE_REWARD = 8

OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

NOTICE_ITEM0 = 0
NOTICE_DEFEATED = 1
-- The items' icons, as retail's notices carried them (Bob's Chronofoil captures of 2026-11-08.)
ITEM0_ICON = 21253
ITEM1_ICON = 20649

-- The quests this one replaced (QST_ACCEPT_CHECK_01-03 are their event IDs), completed with it.
QUESTS_ALSO_COMPLETED = { 968, 971, 974 }

-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855, 960.)
REWARD_CLOSED = 0x11

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags, params)
    current_scene = scene
    player:play_scene(scene, flags, params or {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_HEDYN, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == EOBJECT0 then
        play(player, SCENE_DESTINATION, OBJECT_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        -- The trade first, every time (ITEM1 stays held until the quest completes.)
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEventItem(item_id, target, player)
    if item_id == ITEM0 and BASE_ID == EOBJECT0 and player:get_quest_sequence(EVENT_ID) == 2 then
        -- Retail's scene had the item as its parameter.
        play(player, SCENE_ITEM_USED, OBJECT_FLAGS, { item_id })
        return
    end
    player:finish_event()
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 2 or layout_id ~= ENEMY0 then
        return
    end
    player:event_notice(EVENT_ID, NOTICE_DEFEATED, { ITEM1_ICON })
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
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_HEDYN then
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH: ITEM0
        player:event_notice(EVENT_ID, NOTICE_ITEM0, { ITEM0_ICON })
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_ITEM_USED then
        player:spawn_quest_bnpc(ENEMY0)
    elseif scene == SCENE_TRADE then
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
