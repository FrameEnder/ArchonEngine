-- Quest: Paladin's Pledge (the Paladin's first quest, level 30 Gladiator)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-06) and checked against Bob's Chronofoil capture
-- accc33a9 (2026-11-06, the whole quest on a level 34 Gladiator): its fixture is tools/questkit/fixtures/1055.json.
-- Sequence 0:   ACTOR0 (Lulutsu, in Ul'dah - Steps of Thal) offers it (scene 0, a yield: QuestOffer, 1 when
--               accepted), then talks (scene 1, which ends with QuestAccepted): sequence 1.
-- Sequence 1:   ACTOR1 (Jenlyns, in Ul'dah - Steps of Thal, scene 2, a cutscene: retail's 0xF8402EFB): notice 0,
--               sequence 2.
-- Sequence 2:   EOBJECT0 (the iron brazier in Southern Thanalan; offered while BitFlag8 1 is unset): its empty scene 3
--               (0x2001), EVENT_ACTION_PROCESS_UPPER_MIDDLE's bar (5 s on retail), then scene 4 (the pop-up
--               message); as the event ends ENEMY0 and ENEMY1 (two of BNpcBase 18) come out for the player (layer
--               QB_JobPld300_001 of the wil_w1f4 drop-in, retail's positions, level 28 and 714 HP), counted in UI8AL;
--               the second down brings out ENEMY2 (BNpcBase 19, level 30, 780 HP; 0.24 s later on retail), and it
--               down: notice 1 ([1, 1, ITEM0's icon] on retail), sequence 3 with UI8BH = 1 (ITEM0, the key item the
--               client lists then.)
-- Sequence 3:   EOBJECT1 (the other brazier): talked to, scenes 5 and 6 (both 0x2001: the key item window); ITEM0 used
--               on it (an EventItem action: EventStart type 8 with the item, onEventItem): scene 7 (a cutscene,
--               CUT_JOBPLD00010: retail's 0xF8406EFB): notice 2 [ITEM2's icon], sequence 4 with UI8BH = 1 and UI8BL =
--               1 (ITEM2 and ITEM1, as the client reads them from sequence 4.)
-- Sequence 4:   ACTOR1 is handed ITEM1 (scene 10, NpcTrade in event mode: a yield, 1 when handed over) and talks
--               (scene 11): notice 3, sequence 255 with UI8BH = 1 (ITEM2 kept.)
-- Sequence 255: ACTOR1 is handed ITEM2 (scene 14, NpcTrade) and rewards it (scene 15, a yield: QuestReward, 1 when
--               the reward is taken), then, in the same event, the soul crystal's cutscene (scene 16, 0xF8482EFB,
--               parameter 0, where the client shows LOC_WS being learnt); the quest completes as it ends. Completing
--               it gives the Paladin's soul crystal (the ClassJob sheet's ItemSoulCrystal for the job whose
--               UnlockQuest it is: `zone_connection/quest.rs`), as Seer Folly's does.
-- EOBJECT0's later scenes (8, 9, 12, 13, 17, 18) are empty.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_JENLYNS = 2
SCENE_BRAZIER = 3
SCENE_BRAZIER_DONE = 4
SCENE_SECOND_BRAZIER = 5
SCENE_SECOND_BRAZIER_ITEMS = 6
SCENE_ITEM_USED = 7
SCENE_TRADE = 10
SCENE_TRADED = 11
SCENE_TRADE_LAST = 14
SCENE_REWARD = 15
SCENE_SOUL_CRYSTAL = 16

-- Retail's flags and parameters (Bob's Chronofoil capture accc33a9.)
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB
EVENT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001
ITEM_CUTSCENE_FLAGS = 0xF8406EFB
SOUL_CRYSTAL_FLAGS = 0xF8482EFB
SOUL_CRYSTAL_PARAMS = { 0 }

-- Notice 1's arguments and notice 2's icon, as retail sent them (Bob's Chronofoil capture accc33a9.)
NOTICE_BEATEN_ARGS = { 1, 1, 20406 }
ITEM2_ICON = 22210

-- The two the brazier brings out, counted in UI8AL, and the one that comes after them.
FIRST_ENEMIES = { ENEMY0, ENEMY1 }

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
        play(player, SCENE_JENLYNS, CUTSCENE_FLAGS)
    elseif sequence == 2 and BASE_ID == EOBJECT0 and (player:get_quest_var(EVENT_ID, 5) & 0x80) == 0 then
        play(player, SCENE_BRAZIER, EVENT_FLAGS)
    elseif sequence == 3 and BASE_ID == EOBJECT1 then
        play(player, SCENE_SECOND_BRAZIER, EVENT_FLAGS)
    elseif sequence == 4 and BASE_ID == ACTOR1 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_TRADE_LAST, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- ITEM0 used on the second brazier (the client allows it only there, in sequence 3: its IsEventItemUsable.)
function onEventItem(item_id, target, player)
    if player:get_quest_sequence(EVENT_ID) == 3 and BASE_ID == EOBJECT1 and item_id == ITEM0 then
        play(player, SCENE_ITEM_USED, ITEM_CUTSCENE_FLAGS)
    else
        player:finish_event()
    end
end

-- The brazier's bar finished: the scene after it.
function onEventActionCast(player)
    if current_scene ~= nil or player:get_quest_sequence(EVENT_ID) ~= 2 then
        player:finish_event()
        return
    end
    play(player, SCENE_BRAZIER_DONE, EVENT_FLAGS)
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 2 then
        return
    end
    if layout_id == ENEMY2 then
        player:event_notice(EVENT_ID, 1, NOTICE_BEATEN_ARGS)
        for byte = 0, 5 do
            player:set_quest_var(EVENT_ID, byte, 0)
        end
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH = 1: ITEM0
        player:quest_sequence(EVENT_ID, 3)
        return
    end
    for _, enemy in ipairs(FIRST_ENEMIES) do
        if layout_id == enemy then
            local beaten = (player:get_quest_var(EVENT_ID, 0) & 0x0F) + 1
            player:set_quest_var(EVENT_ID, 0, beaten)
            if beaten >= #FIRST_ENEMIES then
                player:spawn_quest_bnpc(ENEMY2)
            end
            return
        end
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
    elseif scene == SCENE_JENLYNS then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_BRAZIER then
        -- The bar, then scene 4 (see onEventActionCast.)
        player:event_action(EVENT_ACTION_PROCESS_UPPER_MIDDLE)
        return
    elseif scene == SCENE_BRAZIER_DONE then
        player:finish_event()
        -- Ones still out aren't brought out again; the third waits for these two.
        for _, enemy in ipairs(FIRST_ENEMIES) do
            player:spawn_quest_bnpc(enemy)
        end
        return
    elseif scene == SCENE_SECOND_BRAZIER then
        play(player, SCENE_SECOND_BRAZIER_ITEMS, EVENT_FLAGS)
        return
    elseif scene == SCENE_ITEM_USED then
        player:event_notice(EVENT_ID, 2, { ITEM2_ICON })
        player:set_quest_var(EVENT_ID, 1, 0x11) -- UI8BH = 1 (ITEM2), UI8BL = 1 (ITEM1)
        player:quest_sequence(EVENT_ID, 4)
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_TRADED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_TRADED then
        player:event_notice(EVENT_ID, 3, {})
        player:set_quest_var(EVENT_ID, 1, 0x10) -- ITEM1 handed over, ITEM2 kept
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_TRADE_LAST then
        if results[1] == 1 then
            play(player, SCENE_REWARD, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            play(player, SCENE_SOUL_CRYSTAL, SOUL_CRYSTAL_FLAGS, SOUL_CRYSTAL_PARAMS)
            return
        end
    elseif scene == SCENE_SOUL_CRYSTAL then
        player:finish_event()
        player:finish_quest(EVENT_ID, 0)
        return
    end

    player:finish_event()
end

function onYield(scene, id, results, player)
    sceneFinished(scene, results, player)
end

function onReturn(scene, results, player)
    sceneFinished(scene, results, player)
end
