-- Quest: By the Lights of Ishgard (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-06) and checked against Bob's Chronofoil captures
-- 130088ec and 3c56e09f (2026-11-06, the whole quest on each of his two characters: fixtures
-- tools/questkit/fixtures/939.json and `939-3c56e09f.json`):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1 (QuestAccepted):
--               sequence 1.
-- Sequence 1:   walking into EVENTRANGE0 (while UI8AL is under 3) plays scene 2 (its pop-up message, 0x2001), and as it
--               ends ENEMY0-2 (BNpcBase 46, layouts 4297103, 4297104, 4297106; drop-in layer QB_GaiUsc002_001 of
--               roc_r1f1, retail's positions, level 34, 1142 HP) come out for the player. Each one down counts in
--               UI8AL. EOBJECT0 (offered while BitFlag8 1 is unset) plays scene 3 (0x2001; the client's own warning
--               while UI8AL is under 3): with all three down, EVENT_ACTION_SEARCH's bar and scene 4: notice 0,
--               sequence 2 with every variable 0; otherwise nothing. EOBJECT1's scenes (5, 6) are empty: a search
--               with nothing to show.
-- Sequence 2:   EOBJECT2 (offered while UI8AL is 0 and BitFlag8 1 unset): scene 7, the bar, scene 8 (0x2001): notice 1
--               [ITEM0's icon], sequence 255 with ITEM0 (UI8BH = 1.) ACTOR1 has a line meanwhile (scene 9.)
-- Sequence 255: ACTOR0 takes it (scene 10, NpcTrade, a yield: 1 when handed over) and rewards it (scene 11, a yield:
--               QuestReward, 1; closing the window leaves UI8A 0x11.) ACTOR1 has a line meanwhile (scene 12.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_AMBUSH = 2
SCENE_EOBJECT0 = 3
SCENE_EOBJECT1 = 5
SCENE_EOBJECT2 = 7
SCENE_TRADE = 10
SCENE_REWARD = 11

QUIET_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail
ENEMIES = { ENEMY0, ENEMY1, ENEMY2 }
-- ITEM0's icon, on notice 1 (Bob's Chronofoil captures of 2026-11-06.)
ITEM0_ICON = 26151
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
REWARD_CLOSED = 0x11

LINES = {
    [2] = { [ACTOR1] = 9 },
    [255] = { [ACTOR1] = 12 },
}

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- The first scene of what's being searched, while its bar runs.
searching = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function beaten(player)
    return player:get_quest_var(EVENT_ID, 0) & 0x0F -- UI8AL
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == EOBJECT0 then
        searching = SCENE_EOBJECT0
        play(player, SCENE_EOBJECT0, QUIET_FLAGS)
    elseif sequence == 1 and BASE_ID == EOBJECT1 then
        searching = SCENE_EOBJECT1
        play(player, SCENE_EOBJECT1, QUIET_FLAGS)
    elseif sequence == 2 and BASE_ID == EOBJECT2 then
        searching = SCENE_EOBJECT2
        play(player, SCENE_EOBJECT2, QUIET_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    elseif sequence ~= nil and LINES[sequence] ~= nil and LINES[sequence][BASE_ID] ~= nil then
        play(player, LINES[sequence][BASE_ID], HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- Walking into EVENTRANGE0 springs the ambush.
function onEnterTrigger(player, arg)
    if player:get_quest_sequence(EVENT_ID) ~= 1 or beaten(player) >= #ENEMIES then
        player:finish_event()
        return
    end
    play(player, SCENE_AMBUSH, QUIET_FLAGS)
end

function onEventActionCast(player)
    if searching == nil or current_scene ~= nil then
        player:finish_event()
        return
    end
    play(player, searching + 1, QUIET_FLAGS)
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 then
        return
    end
    for _, enemy in ipairs(ENEMIES) do
        if layout_id == enemy then
            player:set_quest_var(EVENT_ID, 0, beaten(player) + 1)
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
    elseif scene == SCENE_AMBUSH then
        player:finish_event()
        -- Ones still out aren't brought out again.
        for _, enemy in ipairs(ENEMIES) do
            player:spawn_quest_bnpc(enemy)
        end
        return
    elseif searching ~= nil and scene == searching then
        if scene ~= SCENE_EOBJECT0 or beaten(player) >= #ENEMIES then
            player:event_action(EVENT_ACTION_SEARCH)
            return
        end
        searching = nil
    elseif searching ~= nil and scene == searching + 1 then
        searching = nil
        if scene == SCENE_EOBJECT0 + 1 then
            player:event_notice(EVENT_ID, 0, {})
            player:set_quest_var(EVENT_ID, 0, 0)
            player:quest_sequence(EVENT_ID, 2)
        elseif scene == SCENE_EOBJECT2 + 1 then
            player:event_notice(EVENT_ID, 1, { ITEM0_ICON })
            player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH = 1: ITEM0
            player:quest_sequence(EVENT_ID, 255)
        end
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
