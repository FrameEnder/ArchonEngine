-- Quest: Bringing out the Dead (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-06) and checked against Bob's Chronofoil captures
-- 130088ec and 3c56e09f (2026-11-06, the whole quest on each of his two characters: fixtures
-- tools/questkit/fixtures/876.json and `876-3c56e09f.json`):
-- Sequence 0:   ACTOR0 (in Eastern Thanalan) offers it (scene 0, a yield: QuestOffer, 1 when accepted; the client turns
--               it down itself while QST_ACCEPT_CHECK is in the journal), then scene 1 (QuestAccepted): sequence 1.
-- Sequence 1:   ACTOR1 (in Western Thanalan, offered while UI8AL is 0 and BitFlag16 1 unset; scene 2, 0x2001, the NPC
--               walking off): notice 0, sequence 2.
-- Sequence 2:   the four of ACTOR10-13, each offered until it's done (BitFlag16 1-4) and four are: its empty scene
--               (0x2001), an EventAction bar (retail's for each: ACTOR10 17 PROCESS_LONG, ACTOR11 and 12 16
--               PROCESS_MIDDLE, ACTOR13 15 PROCESS_SHORT), its second scene (0x2001): UI8AL (the count) and UI8BH
--               (ITEM0) up one, its BitFlag16 set, notice 1 [count, 4, ITEM0's icon]; the fourth: sequence 3 with
--               only UI8BH (4) left.
-- Sequence 3:   ACTOR17 takes the four ITEM0 (scene 27, NpcTrade, a yield: 1 when handed over) and talks (scene 28):
--               notice 2, sequence 4 with every variable 0.
-- Sequence 4:   ACTOR18, 19, 24 and 25, the same way (retail's bars: 16 for 18, 24 and 25, 15 for 19): UI8AH (the
--               count) up one, its own counter to 1 (18 UI8AL, 19 UI8BH, 24 UI8BL, 25 UI8CH) and its BitFlag16
--               (1, 2, 11, 12), an ITEM1 (UI8DH) from 18, 24 and 25 and the ITEM2 (UI8CL) from 19, notice 3
--               [count, 4, the icon]; the fourth: sequence 5 with UI8BH the ITEM2s and UI8BL the ITEM1s (retail's
--               00 13.)
-- Sequence 5:   ACTOR17 takes them (scene 52, NpcTrade) and talks (scene 53): notice 4, sequence 255, every variable 0.
-- Sequence 255: ACTOR0 rewards it (scene 61, a yield: QuestReward, 1; closing the window leaves UI8A 0x11.) Retail
--               completed The Warden Works in Mysterious Ways (877) with it, without rewards (FinishQuest 876 then
--               877, both characters.)
-- The others the client offers meanwhile have lines (LINES, numbered by its IsAcceptEvent order; only ACTOR17's,
-- 26, 47 and 73, say anything.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_ACTOR1 = 2
SCENE_TRADE_ITEM0 = 27
SCENE_AFTER_ITEM0 = 28
SCENE_TRADE_ITEMS = 52
SCENE_AFTER_ITEMS = 53
SCENE_REWARD = 61

BAR_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

-- Sequence 2's four: their scenes (the empty one, then the second), BitFlag16 and retail's bar.
SEQUENCE2 = {
    [ACTOR10] = { scene = 11, flag = 1, bar = EVENT_ACTION_PROCESS_LONG },
    [ACTOR11] = { scene = 13, flag = 2, bar = EVENT_ACTION_PROCESS_MIDDLE },
    [ACTOR12] = { scene = 15, flag = 3, bar = EVENT_ACTION_PROCESS_MIDDLE },
    [ACTOR13] = { scene = 17, flag = 4, bar = EVENT_ACTION_PROCESS_SHOR },
}
-- Sequence 4's four: their scenes, BitFlag16, counter (byte, high nibble), bar, and which item they give.
SEQUENCE4 = {
    [ACTOR18] = { scene = 36, flag = 1, byte = 0, high = false, bar = EVENT_ACTION_PROCESS_MIDDLE, item = 1 },
    [ACTOR19] = { scene = 38, flag = 2, byte = 1, high = true, bar = EVENT_ACTION_PROCESS_SHOR, item = 2 },
    [ACTOR24] = { scene = 48, flag = 11, byte = 1, high = false, bar = EVENT_ACTION_PROCESS_MIDDLE, item = 1 },
    [ACTOR25] = { scene = 50, flag = 12, byte = 2, high = true, bar = EVENT_ACTION_PROCESS_MIDDLE, item = 1 },
}
NEEDED = 4
-- The icon on the progress notices (Bob's Chronofoil captures of 2026-11-06.)
PROGRESS_ICON = 26024
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
REWARD_CLOSED = 0x11
-- Completed with this one, as retail did.
QUEST_ALSO_COMPLETED = 877

LINES = {
    [1] = { [ACTOR2] = 3, [ACTOR3] = 4, [ACTOR4] = 5, [ACTOR5] = 6, [ACTOR6] = 7, [ACTOR7] = 8, [ACTOR8] = 9,
            [ACTOR9] = 10 },
    [2] = { [ACTOR2] = 19, [ACTOR7] = 20, [ACTOR8] = 21, [ACTOR9] = 22, [ACTOR14] = 23, [ACTOR15] = 24,
            [ACTOR16] = 25, [ACTOR17] = 26 },
    [3] = { [ACTOR2] = 29, [ACTOR7] = 30, [ACTOR8] = 31, [ACTOR9] = 32, [ACTOR14] = 33, [ACTOR15] = 34,
            [ACTOR16] = 35 },
    [4] = { [ACTOR20] = 40, [ACTOR21] = 41, [ACTOR22] = 42, [ACTOR23] = 43, [ACTOR14] = 44, [ACTOR15] = 45,
            [ACTOR16] = 46, [ACTOR17] = 47 },
    [5] = { [ACTOR20] = 54, [ACTOR21] = 55, [ACTOR22] = 56, [ACTOR23] = 57, [ACTOR14] = 58, [ACTOR15] = 59,
            [ACTOR16] = 60 },
    [255] = { [ACTOR26] = 62, [ACTOR20] = 63, [ACTOR21] = 64, [ACTOR22] = 65, [ACTOR23] = 66, [ACTOR27] = 67,
              [ACTOR28] = 68, [ACTOR29] = 69, [ACTOR14] = 70, [ACTOR15] = 71, [ACTOR16] = 72, [ACTOR17] = 73 },
}

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

-- BitFlag16: flags 1-8 are UI8E from its top bit, 9-16 UI8F, as retail sets them.
function flagSpot(flag)
    if flag <= 8 then
        return 4, 0x80 >> (flag - 1)
    end
    return 5, 0x80 >> (flag - 9)
end

function hasFlag16(player, flag)
    local byte, bit = flagSpot(flag)
    return (player:get_quest_var(EVENT_ID, byte) & bit) ~= 0
end

function setFlag16(player, flag)
    local byte, bit = flagSpot(flag)
    player:set_quest_var(EVENT_ID, byte, player:get_quest_var(EVENT_ID, byte) | bit)
end

function clearVars(player)
    for byte = 0, 5 do
        player:set_quest_var(EVENT_ID, byte, 0)
    end
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- The one being seen to while its bar runs.
tending = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local tend = nil
    if sequence == 2 then
        tend = SEQUENCE2[BASE_ID]
    elseif sequence == 4 then
        tend = SEQUENCE4[BASE_ID]
    end

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_ACTOR1, BAR_FLAGS)
    elseif tend ~= nil and not hasFlag16(player, tend.flag) then
        tending = tend
        play(player, tend.scene, BAR_FLAGS)
    elseif sequence == 3 and BASE_ID == ACTOR17 then
        play(player, SCENE_TRADE_ITEM0, HIDE_HOTBAR)
    elseif sequence == 5 and BASE_ID == ACTOR17 then
        play(player, SCENE_TRADE_ITEMS, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    elseif sequence ~= nil and LINES[sequence] ~= nil and LINES[sequence][BASE_ID] ~= nil then
        play(player, LINES[sequence][BASE_ID], HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- The bar finished: the second scene.
function onEventActionCast(player)
    if tending == nil or current_scene ~= nil then
        player:finish_event()
        return
    end
    play(player, tending.scene + 1, BAR_FLAGS)
end

-- One of sequence 2's done.
function tendedSequence2(player, tend)
    local count = getVar(player, 0, false) + 1 -- UI8AL
    local items = getVar(player, 1, true) + 1 -- UI8BH: ITEM0
    player:event_notice(EVENT_ID, 1, { count, NEEDED, PROGRESS_ICON })
    if count >= NEEDED then
        clearVars(player)
        setVar(player, 1, true, items)
        player:quest_sequence(EVENT_ID, 3)
    else
        setVar(player, 0, false, count)
        setVar(player, 1, true, items)
        setFlag16(player, tend.flag)
    end
end

-- One of sequence 4's done.
function tendedSequence4(player, tend)
    local count = getVar(player, 0, true) + 1 -- UI8AH
    local item1 = getVar(player, 3, true) -- UI8DH: ITEM1
    local item2 = getVar(player, 2, false) -- UI8CL: ITEM2
    if tend.item == 1 then
        item1 = item1 + 1
    else
        item2 = item2 + 1
    end
    player:event_notice(EVENT_ID, 3, { count, NEEDED, PROGRESS_ICON })
    if count >= NEEDED then
        clearVars(player)
        player:set_quest_var(EVENT_ID, 1, (item2 << 4) | item1) -- UI8BH ITEM2, UI8BL ITEM1
        player:quest_sequence(EVENT_ID, 5)
    else
        setVar(player, 0, true, count)
        setVar(player, tend.byte, tend.high, 1)
        setVar(player, 3, true, item1)
        setVar(player, 2, false, item2)
        setFlag16(player, tend.flag)
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
    elseif scene == SCENE_ACTOR1 then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif tending ~= nil and scene == tending.scene then
        -- The bar, then the second scene (see onEventActionCast.)
        player:event_action(tending.bar)
        return
    elseif tending ~= nil and scene == tending.scene + 1 then
        local tend = tending
        tending = nil
        if player:get_quest_sequence(EVENT_ID) == 2 then
            tendedSequence2(player, tend)
        else
            tendedSequence4(player, tend)
        end
    elseif scene == SCENE_TRADE_ITEM0 or scene == SCENE_TRADE_ITEMS then
        if results[1] == 1 then
            play(player, scene + 1, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_AFTER_ITEM0 then
        clearVars(player)
        player:event_notice(EVENT_ID, 2, {})
        player:quest_sequence(EVENT_ID, 4)
    elseif scene == SCENE_AFTER_ITEMS then
        clearVars(player)
        player:event_notice(EVENT_ID, 4, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_event()
            player:finish_quest(EVENT_ID, results[2] or 0)
            player:mark_quest_completed(QUEST_ALSO_COMPLETED)
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
