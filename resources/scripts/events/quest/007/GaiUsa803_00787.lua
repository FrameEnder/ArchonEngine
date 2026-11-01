-- Quest: Rock of Rancor
--
-- Written from the client's quest script (kawari-questdump, 2026-11-01) and checked against Bob's Chronofoil capture
-- 412ba91d (2026-11-01, the whole quest, turned down twice first: its fixture is tools/questkit/fixtures/787.json):
-- Sequence 0:   ACTOR0 (Aideen) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1, where the
--               client accepts it: sequence 1, with ITEM0 (UI8BH = 1; the client lists it from there.)
-- Sequence 1:   EOBJECT0 (the destination): talked to, it only opens the inventory (scene 2); ITEM0 used on it
--               (`onEventItem`) plays scene 3 (0x2001): notice 0, the variables cleared (ITEM0 gone), sequence 2.
-- Sequence 2:   five humming crystals, in any order (the client's IsAcceptEvent offers each while its nibble and flag
--               are unset): its scene, EVENT_ACTION_SEARCH_MIDDLE's bar, then the next scene, all 0x2001:
--                 EOBJECT1, scenes 4 and 5: UI8AL = 1, BitFlag8 1
--                 EOBJECT2, scenes 6 and 7: UI8BH = 1, BitFlag8 2
--                 EOBJECT3, scenes 8 and 9: UI8BL = 1, BitFlag8 3
--                 EOBJECT4, scenes 10 and 11: UI8CH = 1, BitFlag8 4
--                 EOBJECT5, scenes 12 and 13: UI8CL = 1, BitFlag8 5
--               UI8AH counts them and UI8DL the ITEM1 each gives; each sends notice 1 [count, 5, ITEM1's icon].
--               After the fifth retail cleared the variables, kept the five ITEM1 in UI8BL (the client lists them
--               from there in sequence 255) and moved to sequence 255.
-- Sequence 255: ACTOR0 takes them (scene 14, NpcTrade in event mode: a yield, 1 when handed over) and rewards it
--               (scene 15, a yield: QuestReward, 1 and the optional item chosen.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_DESTINATION = 2
SCENE_ITEM_USED = 3
SCENE_TRADE = 14
SCENE_REWARD = 15

OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

NOTICE_ITEM_USED = 0
NOTICE_CRYSTAL = 1
-- ITEM1's icon, as retail's notice carried it (Bob's Chronofoil capture 412ba91d.)
ITEM1_ICON = 20005

-- The five crystals: their scene, the scene after the search bar, the nibble that marks them and their flag.
CRYSTALS = {
    { object = EOBJECT1, scene = 4, searched = 5, byte = 0, high = false, flag = 1 },
    { object = EOBJECT2, scene = 6, searched = 7, byte = 1, high = true, flag = 2 },
    { object = EOBJECT3, scene = 8, searched = 9, byte = 1, high = false, flag = 3 },
    { object = EOBJECT4, scene = 10, searched = 11, byte = 2, high = true, flag = 4 },
    { object = EOBJECT5, scene = 12, searched = 13, byte = 2, high = false, flag = 5 },
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

-- BitFlag8: flag 1 is the top bit of UI8F, as retail sets it.
function hasFlag(player, flag)
    return (player:get_quest_var(EVENT_ID, 5) & (0x80 >> (flag - 1))) ~= 0
end

function setFlag(player, flag)
    player:set_quest_var(EVENT_ID, 5, player:get_quest_var(EVENT_ID, 5) | (0x80 >> (flag - 1)))
end

function clearVars(player)
    for byte = 0, 5 do
        player:set_quest_var(EVENT_ID, byte, 0)
    end
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
current_crystal = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    current_crystal = nil

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == EOBJECT0 then
        play(player, SCENE_DESTINATION, OBJECT_FLAGS)
        return
    elseif sequence == 2 then
        for _, crystal in ipairs(CRYSTALS) do
            if BASE_ID == crystal.object and not hasFlag(player, crystal.flag) then
                current_crystal = crystal
                play(player, crystal.scene, OBJECT_FLAGS)
                return
            end
        end
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEventItem(item_id, target, player)
    if item_id == ITEM0 and BASE_ID == EOBJECT0 and player:get_quest_sequence(EVENT_ID) == 1 then
        play(player, SCENE_ITEM_USED, OBJECT_FLAGS)
    else
        player:finish_event()
    end
end

-- The search bar on a crystal finished: the scene after it.
function onEventActionCast(player)
    if player:get_quest_sequence(EVENT_ID) ~= 2 or current_crystal == nil then
        current_crystal = nil
        player:finish_event()
        return
    end
    play(player, current_crystal.searched, OBJECT_FLAGS)
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil
    local crystal = current_crystal

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            play(player, SCENE_ACCEPTED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        setVar(player, 1, true, 1) -- UI8BH = 1: ITEM0
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_ITEM_USED then
        player:event_notice(EVENT_ID, NOTICE_ITEM_USED, {})
        clearVars(player)
        player:quest_sequence(EVENT_ID, 2)
    elseif crystal ~= nil and scene == crystal.scene then
        -- Searching the crystal: the bar, then the scene after it (see onEventActionCast.)
        player:event_action(EVENT_ACTION_SEARCH_MIDDLE)
        return
    elseif crystal ~= nil and scene == crystal.searched then
        current_crystal = nil
        local done = getVar(player, 0, true) + 1
        player:event_notice(EVENT_ID, NOTICE_CRYSTAL, { done, #CRYSTALS, ITEM1_ICON })
        if done >= #CRYSTALS then
            clearVars(player)
            setVar(player, 1, false, done) -- UI8BL: the ITEM1 kept for the trade
            player:quest_sequence(EVENT_ID, 255)
        else
            setVar(player, 0, true, done)
            setVar(player, crystal.byte, crystal.high, 1)
            setVar(player, 3, false, done) -- UI8DL: ITEM1 held
            setFlag(player, crystal.flag)
        end
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
