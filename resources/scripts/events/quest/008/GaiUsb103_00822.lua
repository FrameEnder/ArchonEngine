-- Quest: When the Worm Turns
--
-- Written from the client's quest script (kawari-questdump, 2026-11-03) and checked against Bob's Chronofoil captures
-- 44df02fc and c9145fdc (2026-11-03, the whole quest each: its fixtures are tools/questkit/fixtures/822.json and
-- 822-c9145fdc.json):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1, where the client
--               accepts it: sequence 1.
-- Sequence 1:   any ENEMY0 (a BNpcBase, the zone's own) defeated: ITEM0 (UI8BH = 1), notice 0 [ITEM0's icon],
--               sequence 2.
-- Sequence 2:   EOBJECT0 (the worms' territory): talked to, it only opens the inventory (scene 2, 0x2001); ITEM0 used
--               on it (`onEventItem`) plays scene 3 (0x2001): notice 1, the variables cleared, sequence 3.
-- Sequence 3:   EOBJECT1: scene 4 (0x2001, empty), EVENT_ACTION_WAITING's bar, then scene 5 (0x2001, the "it
--               appears" message): ENEMY1 comes out (layer QB_GaiUsb103_001 of the Southern Thanalan drop-in).
--               Defeating it: ITEM1 (UI8BH = 1), notice 2 [ITEM1's icon], sequence 255.
-- Sequence 255: ACTOR0 takes it (scene 8, NpcTrade in event mode: a yield, 1 when handed over) and rewards it
--               (scene 9, a yield: QuestReward, 1 and the optional item chosen.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_TERRITORY = 2
SCENE_ITEM_USED = 3
SCENE_WAIT = 4
SCENE_APPEAR = 5
SCENE_TRADE = 8
SCENE_REWARD = 9

OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

NOTICE_ITEM0 = 0
NOTICE_ITEM_USED = 1
NOTICE_DEFEATED = 2
-- The items' icons, as retail's notices carried them (Bob's Chronofoil capture 44df02fc.)
ITEM0_ICON = 25919
ITEM1_ICON = 29001

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
    elseif sequence == 2 and BASE_ID == EOBJECT0 then
        play(player, SCENE_TERRITORY, OBJECT_FLAGS)
    elseif sequence == 3 and BASE_ID == EOBJECT1 then
        play(player, SCENE_WAIT, OBJECT_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEventItem(item_id, target, player)
    if item_id == ITEM0 and BASE_ID == EOBJECT0 and player:get_quest_sequence(EVENT_ID) == 2 then
        play(player, SCENE_ITEM_USED, OBJECT_FLAGS)
        return
    end
    player:finish_event()
end

-- The bar at EOBJECT1 finished: the scene after it.
function onEventActionCast(player)
    if player:get_quest_sequence(EVENT_ID) ~= 3 then
        player:finish_event()
        return
    end
    play(player, SCENE_APPEAR, OBJECT_FLAGS)
end

function onBNpcKill(base_id, layout_id, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    if sequence == 1 and base_id == ENEMY0 then
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH: ITEM0
        player:event_notice(EVENT_ID, NOTICE_ITEM0, { ITEM0_ICON })
        player:quest_sequence(EVENT_ID, 2)
    elseif sequence == 3 and layout_id == ENEMY1 then
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH: ITEM1
        player:event_notice(EVENT_ID, NOTICE_DEFEATED, { ITEM1_ICON })
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
    elseif scene == SCENE_ITEM_USED then
        player:set_quest_var(EVENT_ID, 1, 0)
        player:event_notice(EVENT_ID, NOTICE_ITEM_USED, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_WAIT then
        -- Waiting there: the bar, then scene 5 (see onEventActionCast.)
        player:event_action(EVENT_ACTION_WAITING)
        return
    elseif scene == SCENE_APPEAR then
        player:spawn_quest_bnpc(ENEMY1)
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_REWARD, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID, results[2])
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
