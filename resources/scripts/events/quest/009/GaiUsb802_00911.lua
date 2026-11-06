-- Quest: Following the Evidence (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-06) and checked against Bob's Chronofoil captures
-- 130088ec and 3c56e09f (2026-11-06, the whole quest on each of his two characters: fixtures
-- tools/questkit/fixtures/911.json and `911-3c56e09f.json`; questkit had layouts 4293142 and 4293143 the wrong way
-- round, from the order of the Quest sheet's listeners: the variables retail set, the client's IsAcceptEvent and the
-- scenes put EOBJECT4 at 4293142 and EOBJECT3 at 4293143):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1 (QuestAccepted):
--               sequence 1.
-- Sequence 1:   ACTOR1 (scene 2): notice 0, sequence 2.
-- Sequence 2:   ACTOR2 (offered while UI8AL is 0 and BitFlag8 1 unset; scene 3): notice 1, sequence 3.
-- Sequence 3:   EOBJECT2, 3 and 4, in any order, each once (its counter UI8BH, UI8AL or UI8BL, its BitFlag16 1, 2
--               or 3): its first scene, EVENT_ACTION_SEARCH's bar, its second scene (all 0x2001; EOBJECT2 9 and 10,
--               EOBJECT3 11 and 12, EOBJECT4 13 and 14), UI8AH (the count) up one and notice 2 [count, 3, the icon];
--               EOBJECT2 and 4 give an ITEM0 (UI8CH; their second scenes say so), EOBJECT3 nothing (its notice has no
--               icon). The third: sequence 4 with UI8BH the ITEM0s (2) and the rest 0.
-- Sequence 4:   ACTOR2 takes them (scene 27, NpcTrade, a yield: 1 when handed over) and talks (scene 28): notice 3,
--               sequence 255 (UI8BH stays: ACTOR0 takes them at the end.)
-- Sequence 255: ACTOR0 takes them (scene 40, NpcTrade) and rewards it (scene 41, a yield: QuestReward, 1; closing the
--               window leaves UI8A 0x11.)
-- The others offered meanwhile have lines (numbered by the client's IsAcceptEvent order; an object's is two empty
-- scenes with the search bar between.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_ACTOR1 = 2
SCENE_ACTOR2 = 3
SCENE_TRADE = 27
SCENE_TRADED = 28
SCENE_TRADE_FINAL = 40
SCENE_REWARD = 41

QUIET_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

-- Sequence 3's three: their first scene, counter (byte, high nibble), BitFlag16 and whether they give an ITEM0.
SEARCHES = {
    [EOBJECT2] = { scene = 9, byte = 1, high = true, flag = 1, item = true },
    [EOBJECT3] = { scene = 11, byte = 0, high = false, flag = 2, item = false },
    [EOBJECT4] = { scene = 13, byte = 1, high = false, flag = 3, item = true },
}
NEEDED = 3
-- ITEM0's icon, on the notice for one found (Bob's Chronofoil captures of 2026-11-06.)
ITEM0_ICON = 21002
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
REWARD_CLOSED = 0x11

-- The lines: an actor's scene, or an object's first scene (its search bar and second scene follow.)
LINES = {
    [2] = { [ACTOR3] = 4, [EOBJECT0] = 5, [EOBJECT1] = 7 },
    [3] = { [EOBJECT0] = 15, [EOBJECT1] = 17, [ACTOR2] = 19, [ACTOR3] = 20, [EOBJECT5] = 21, [EOBJECT6] = 23,
            [EOBJECT7] = 25 },
    [4] = { [EOBJECT0] = 29, [EOBJECT1] = 31, [ACTOR3] = 33, [EOBJECT5] = 34, [EOBJECT6] = 36, [EOBJECT7] = 38 },
    [255] = { [EOBJECT1] = 42, [EOBJECT0] = 44, [ACTOR2] = 46, [ACTOR3] = 47, [EOBJECT5] = 48, [EOBJECT6] = 50,
              [EOBJECT7] = 52 },
}
OBJECTS = {
    [EOBJECT0] = true, [EOBJECT1] = true, [EOBJECT5] = true, [EOBJECT6] = true, [EOBJECT7] = true,
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

-- BitFlag16: flags 1-8 are UI8E from its top bit, as retail sets them.
function hasFlag16(player, flag)
    return (player:get_quest_var(EVENT_ID, 4) & (0x80 >> (flag - 1))) ~= 0
end

function setFlag16(player, flag)
    player:set_quest_var(EVENT_ID, 4, player:get_quest_var(EVENT_ID, 4) | (0x80 >> (flag - 1)))
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- What's being searched (a SEARCHES entry, or a line's object), while its bar runs.
searching = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local search = SEARCHES[BASE_ID]

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_ACTOR1, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR2 then
        play(player, SCENE_ACTOR2, HIDE_HOTBAR)
    elseif sequence == 3 and search ~= nil and getVar(player, search.byte, search.high) == 0
        and not hasFlag16(player, search.flag) then
        searching = search
        play(player, search.scene, QUIET_FLAGS)
    elseif sequence == 4 and BASE_ID == ACTOR2 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_TRADE_FINAL, HIDE_HOTBAR)
    elseif sequence ~= nil and LINES[sequence] ~= nil and LINES[sequence][BASE_ID] ~= nil then
        local scene = LINES[sequence][BASE_ID]
        if OBJECTS[BASE_ID] then
            searching = { scene = scene }
            play(player, scene, QUIET_FLAGS)
        else
            play(player, scene, HIDE_HOTBAR)
        end
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    if searching == nil or current_scene ~= nil then
        player:finish_event()
        return
    end
    play(player, searching.scene + 1, QUIET_FLAGS)
end

-- One of sequence 3's searched.
function searched(player, search)
    local count = getVar(player, 0, true) + 1 -- UI8AH
    local items = getVar(player, 2, true) -- UI8CH
    if search.item then
        items = items + 1
        player:event_notice(EVENT_ID, 2, { count, NEEDED, ITEM0_ICON })
    else
        player:event_notice(EVENT_ID, 2, { count, NEEDED })
    end
    if count >= NEEDED then
        for byte = 0, 5 do
            player:set_quest_var(EVENT_ID, byte, 0)
        end
        setVar(player, 1, true, items) -- UI8BH: the ITEM0s
        player:quest_sequence(EVENT_ID, 4)
    else
        setVar(player, 0, true, count)
        setVar(player, search.byte, search.high, 1)
        setVar(player, 2, true, items)
        setFlag16(player, search.flag)
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
    elseif searching ~= nil and scene == searching.scene then
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif searching ~= nil and scene == searching.scene + 1 then
        local search = searching
        searching = nil
        if search.flag ~= nil then
            searched(player, search)
        end
    elseif scene == SCENE_ACTOR1 then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_ACTOR2 then
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_TRADE or scene == SCENE_TRADE_FINAL then
        if results[1] == 1 then
            play(player, scene + 1, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_TRADED then
        player:event_notice(EVENT_ID, 3, {})
        player:quest_sequence(EVENT_ID, 255)
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
