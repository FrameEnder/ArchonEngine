-- Quest: Surveying the Damage (Gridania)
--
-- Checked against the client's quest script (kawari-questdump) and a live recording of the quest:
-- Sequence 0:   ACTOR0 offers the quest (scene 0, returns 1 when accepted.)
-- Sequence 1:   search the four objects in the cave (EVENT_ACTION_SEARCH, then their scene):
--                 EOBJECT0 survey records (scene 1):           todo 0, UI8AL = 1, ITEM0 count in UI8CH
--                 EOBJECT1/2 surveyor's instruments (2 and 3): todo 1, UI8BH counts to 2, ITEM1 count in UI8CL
--                 EOBJECT3 surveyor's rope (scene 4):          todo 2, UI8BL = 1, ITEM2 count in UI8DH
--               Each object searched sets its BitFlag8 (1-4, kept in UI8F), which stops the client offering it
--               again, and UI8AH counts the todos completed. Once all three are done, retail moves straight to
--               sequence 255 and keeps only the item counts, where the client looks for them then: ITEM0 in
--               UI8BH, ITEM1 in UI8BL, ITEM2 in UI8CH.
-- Sequence 255: hand the items to ACTOR1 (trade scene 5, returns 1 when handed over), then the reward (scene 92.)

SCENE_OFFER = 0
SCENE_TRADE = 5
SCENE_FINISH = 92

-- Each object: its scene and BitFlag8 index.
OBJECTS = {
    [EOBJECT0] = { scene = 1, flag = 1 },
    [EOBJECT1] = { scene = 2, flag = 2 },
    [EOBJECT2] = { scene = 3, flag = 3 },
    [EOBJECT3] = { scene = 4, flag = 4 },
}

INSTRUMENTS_NEEDED = 2

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

-- UI8AH: how many of the three todos are done.
function todoDone(player)
    setVar(player, 0, true, getVar(player, 0, true) + 1)
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- The object being searched, while its progress bar runs.
searching = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local object = OBJECTS[BASE_ID]

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and object ~= nil and not hasFlag(player, object.flag) then
        -- Searching shows a progress bar; the scene plays once it finishes (see onEventActionCast.)
        searching = object
        player:event_action(EVENT_ACTION_SEARCH)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    if searching == nil or player:get_quest_sequence(EVENT_ID) ~= 1 then
        player:finish_event()
        return
    end
    play(player, searching.scene, HIDE_HOTBAR)
end

-- What searching each object gives, as retail records it.
function searched(player, scene)
    if scene == 1 then
        setVar(player, 0, false, 1) -- UI8AL: the records
        setVar(player, 2, true, 1) -- UI8CH: ITEM0
        setFlag(player, 1)
        todoDone(player)
    elseif scene == 2 or scene == 3 then
        local instruments = getVar(player, 1, true) + 1 -- UI8BH
        setVar(player, 1, true, instruments)
        setVar(player, 2, false, getVar(player, 2, false) + 1) -- UI8CL: ITEM1
        setFlag(player, scene) -- flag 2 or 3
        if instruments >= INSTRUMENTS_NEEDED then
            todoDone(player)
        end
    elseif scene == 4 then
        setVar(player, 1, false, 1) -- UI8BL: the rope
        setVar(player, 3, true, 1) -- UI8DH: ITEM2
        setFlag(player, 4)
        todoDone(player)
    end

    if getVar(player, 0, false) >= 1 and getVar(player, 1, true) >= INSTRUMENTS_NEEDED and getVar(player, 1, false) >= 1 then
        -- Retail keeps only the item counts for the hand-in: ITEM0 in UI8BH, ITEM1 in UI8BL, ITEM2 in UI8CH.
        player:quest_sequence(EVENT_ID, 255)
        player:set_quest_var(EVENT_ID, 0, 0)
        player:set_quest_var(EVENT_ID, 1, (1 << 4) | INSTRUMENTS_NEEDED)
        player:set_quest_var(EVENT_ID, 2, 1 << 4)
        player:set_quest_var(EVENT_ID, 3, 0)
        player:set_quest_var(EVENT_ID, 4, 0)
        player:set_quest_var(EVENT_ID, 5, 0)
    end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID)
            player:quest_sequence(EVENT_ID, 1)
        end
    elseif scene >= 1 and scene <= 4 then
        searching = nil
        searched(player, scene)
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_FINISH, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_FINISH then
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
