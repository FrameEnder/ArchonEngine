-- Quest: Power of Deduction
--
-- Written from the client's quest script (kawari-questdump, 2026-11-01) and checked against Bob's Chronofoil capture
-- 412ba91d (2026-11-01, the whole quest: its fixture is tools/questkit/fixtures/799.json):
-- Sequence 0:   ACTOR0 (Medrod) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then the cutscene (scene 1,
--               0xF8402EFB), where the client accepts it: sequence 1.
-- Sequence 1:   walking into EVENTRANGE0 (the client allows it while UI8AL < 1) brings out ENEMY0, the player's own
--               (layer QB_GaiUsa904_001 of the North Shroud drop-in, a floating eye at retail's position, level and HP),
--               and plays scene 2 (0x2001, the client's pop message). Its death: UI8AL = 1, no notice. ACTOR1 (the
--               mangled corpse): scene 3 (0x2001; the client's message that it can't be reached while UI8AL < 1, and
--               nothing more then); with UI8AL = 1 EVENT_ACTION_SEARCH's bar, then scene 4 (0x2001, empty): notice 0
--               [ITEM0's icon], the variables cleared and ITEM0 given (UI8BH = 1), sequence 255.
-- Sequence 255: ACTOR2 (Aethelmaer) takes ITEM0 (scene 7, NpcTrade in event mode: a yield, 1 when handed over) and
--               rewards it (scene 8, a cutscene and a yield: QuestReward, 1 and the optional item chosen.)
-- EOBJECT0 (the violated corpse) has empty scenes (5, 6), not seen on retail and not played.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_AMBUSH = 2
SCENE_CORPSE = 3
SCENE_SEARCHED = 4
SCENE_TRADE = 7
SCENE_REWARD = 8

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB on retail
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

NOTICE_SEARCHED = 0
-- ITEM0's icon, as retail's notice carried it (Bob's Chronofoil capture 412ba91d.)
ITEM0_ICON = 26024

-- UI8AL: ENEMY0 defeated.
function defeated(player)
    return (player:get_quest_var(EVENT_ID, 0) & 0x0F) >= 1
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
        play(player, SCENE_CORPSE, OBJECT_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- Walking into EVENTRANGE0 brings the floating eye out (nothing new while it's out.)
function onEnterTrigger(player, arg)
    if player:get_quest_sequence(EVENT_ID) ~= 1 or defeated(player) then
        player:finish_event()
        return
    end
    player:spawn_quest_bnpc(ENEMY0)
    play(player, SCENE_AMBUSH, OBJECT_FLAGS)
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 or layout_id ~= ENEMY0 then
        return
    end
    player:set_quest_var(EVENT_ID, 0, (player:get_quest_var(EVENT_ID, 0) & 0xF0) | 1)
end

-- The search bar on the corpse finished: the scene after it.
function onEventActionCast(player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 then
        player:finish_event()
        return
    end
    play(player, SCENE_SEARCHED, OBJECT_FLAGS)
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
    elseif scene == SCENE_CORPSE then
        if defeated(player) then
            -- Searching the corpse: the bar, then scene 4 (see onEventActionCast.)
            player:event_action(EVENT_ACTION_SEARCH)
            return
        end
    elseif scene == SCENE_SEARCHED then
        player:event_notice(EVENT_ID, NOTICE_SEARCHED, { ITEM0_ICON })
        for byte = 0, 5 do
            player:set_quest_var(EVENT_ID, byte, 0)
        end
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH = 1: ITEM0
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_REWARD, CUTSCENE_FLAGS)
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
