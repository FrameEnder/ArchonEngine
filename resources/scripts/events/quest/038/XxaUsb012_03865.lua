-- Quest: Trial by Turtle
--
-- Written from the client's quest script (kawari-questdump, 2026-11-03) and checked against Bob's Chronofoil captures
-- 44df02fc and c9145fdc (2026-11-03, the whole quest each: its fixtures are tools/questkit/fixtures/3865.json and
-- 3865-c9145fdc.json):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: QuestOffer, 1 when accepted), then the cutscene where the client
--               accepts it (scene 1, retail's flags 0xF8406EFB): sequence 1.
-- Sequence 1:   EOBJECT0 (an egg; the client offers it while BitFlag8 1 is unset): scene 2 (0x2001, empty),
--               EVENTACTION0's bar (retail's 2 s), then scene 3 (0x2001: its "it appears" line for the one it
--               brings out): ENEMY0 comes out (layer QB_XxaUsb012_001 of the South Shroud drop-in). Defeating it:
--               notice 0, sequence 2.
-- Sequence 2:   EOBJECT2 (another egg): scene 7 (0x2001, empty), EVENTACTION0's bar again (retail's, 2 s), scene 8
--               (0x2001, empty): ITEM0 (UI8BH = 1), notice 1 [ITEM0's icon], sequence 3.
-- Sequence 3:   ACTOR0 takes it (scene 10, NpcTrade in event mode: a yield, 1 when handed over; closing the trade
--               returns 0 and nothing happens), then scene 11: the variables cleared, notice 2, sequence 255.
-- Sequence 255: ACTOR1 (in Southern Thanalan) rewards it (scene 12, a yield: QuestReward, 1 and the optional item.)
-- EOBJECT1 (its scenes 4, 5 and 6) wasn't used on retail: talking to it does nothing here.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_EGG = 2
SCENE_APPEAR = 3
SCENE_SECOND_EGG = 7
SCENE_SECOND_EGG_SEARCHED = 8
SCENE_TRADE = 10
SCENE_TRADED = 11
SCENE_REWARD = 12

-- SET_BASE | HIDE_HOTBAR | INVINCIBLE, retail's for the accepting cutscene (Bob's Chronofoil capture 44df02fc.)
CUTSCENE_FLAGS = 0xF8406EFB
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

NOTICE_DEFEATED = 0
NOTICE_ITEM0 = 1
NOTICE_TRADED = 2
-- ITEM0's icon, as retail's notice carried it (Bob's Chronofoil capture 44df02fc.)
ITEM0_ICON = 25251

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
        play(player, SCENE_EGG, OBJECT_FLAGS)
    elseif sequence == 2 and BASE_ID == EOBJECT2 then
        play(player, SCENE_SECOND_EGG, OBJECT_FLAGS)
    elseif sequence == 3 and BASE_ID == ACTOR0 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- A search bar finished: the scene after it.
function onEventActionCast(player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    if sequence == 1 then
        play(player, SCENE_APPEAR, OBJECT_FLAGS)
    elseif sequence == 2 then
        play(player, SCENE_SECOND_EGG_SEARCHED, OBJECT_FLAGS)
    else
        player:finish_event()
    end
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 or layout_id ~= ENEMY0 then
        return
    end
    player:event_notice(EVENT_ID, NOTICE_DEFEATED, {})
    player:quest_sequence(EVENT_ID, 2)
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            play(player, SCENE_ACCEPTED, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_EGG or scene == SCENE_SECOND_EGG then
        -- Searching the egg: the bar, then the next scene (see onEventActionCast.)
        player:event_action(EVENTACTION0)
        return
    elseif scene == SCENE_APPEAR then
        player:spawn_quest_bnpc(ENEMY0)
    elseif scene == SCENE_SECOND_EGG_SEARCHED then
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH: ITEM0
        player:event_notice(EVENT_ID, NOTICE_ITEM0, { ITEM0_ICON })
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_TRADED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_TRADED then
        player:set_quest_var(EVENT_ID, 1, 0)
        player:event_notice(EVENT_ID, NOTICE_TRADED, {})
        player:quest_sequence(EVENT_ID, 255)
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
