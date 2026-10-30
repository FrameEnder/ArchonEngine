-- Quest: First Impressions
--
-- Written from the client's quest script (kawari-questdump, 2026-10-30) and checked against Bob's Chronofoil capture
-- 5fcc970c (2026-10-30, the whole quest: its fixture is tools/questkit/fixtures/710.json):
-- Sequence 0:   ACTOR0 (Rolfe Hawthorne) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1, where
--               the client accepts it: sequence 1.
-- Sequence 1:   ACTOR1 (Rosa Hawthorne) talks (scene 2) and gives ITEM0: notice 0 with its icon, UI8BH = 1 (the client
--               lists ITEM0 from UI8BH), sequence 2.
-- Sequence 2:   EOBJECT0, the curious tussock (offered while BitFlag8 1 is unset, which retail never set): talking to
--               it plays scene 3 (0x2001), which only opens the inventory. Using ITEM0 on it (the client allows ITEM0
--               in sequence 2) plays scene 4 (0x2001, a pop-up message), and ENEMY0, an ochu (a layout ID), comes out
--               of it. Defeating it (the client counts it while UI8AL < 1): notice 1 with ITEM1's icon, ITEM0 gone
--               and ITEM1 given (UI8BL = 1), sequence 255.
-- Sequence 255: ACTOR0 takes ITEM1 (scene 5, NpcTrade in event mode: a yield, 1 when handed over) and rewards it
--               (scene 6, a yield: QuestReward, 1.)
-- The ochu is layer QB_GaiUsa004_001 of the East Shroud drop-in, at retail's position, level and HP.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_ROSA = 2
SCENE_TUSSOCK = 3
SCENE_ITEM_USED = 4
SCENE_TRADE = 5
SCENE_REWARD = 6
FLAGS_TUSSOCK = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

NOTICE_ITEM0 = 0
NOTICE_OCHU = 1
-- The key items' icons the notices carried on retail (Bob's Chronofoil capture 5fcc970c.)
ITEM0_ICON = 20644
ITEM1_ICON = 25005

-- Quest variables: UI8A..F are bytes 0..5, H is the high nibble and L the low one.
function getVar(player, byte, high)
    local value = player:get_quest_var(EVENT_ID, byte)
    if high then
        return value >> 4
    end
    return value & 0x0F
end

function setVar(player, byte, high, nibble)
    local value = player:get_quest_var(EVENT_ID, byte)
    if high then
        value = (value & 0x0F) | (nibble << 4)
    else
        value = (value & 0xF0) | nibble
    end
    player:set_quest_var(EVENT_ID, byte, value)
end

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
        play(player, SCENE_ROSA, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == EOBJECT0 then
        play(player, SCENE_TUSSOCK, FLAGS_TUSSOCK)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEventItem(item_id, target, player)
    if item_id == ITEM0 and BASE_ID == EOBJECT0 and player:get_quest_sequence(EVENT_ID) == 2 then
        play(player, SCENE_ITEM_USED, FLAGS_TUSSOCK)
    else
        player:finish_event()
    end
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 2 or layout_id ~= ENEMY0 then
        return
    end
    player:event_notice(EVENT_ID, NOTICE_OCHU, { ITEM1_ICON })
    setVar(player, 1, true, 0) -- ITEM0 handed over
    setVar(player, 1, false, 1) -- UI8BL = 1: ITEM1
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
    elseif scene == SCENE_ROSA then
        player:event_notice(EVENT_ID, NOTICE_ITEM0, { ITEM0_ICON })
        setVar(player, 1, true, 1) -- UI8BH = 1: ITEM0
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
