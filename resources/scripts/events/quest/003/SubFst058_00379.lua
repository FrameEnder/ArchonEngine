-- Quest: In the Grim Darkness of the Forest (Gridania)
--
-- Checked against the client's quest script (kawari-questdump), Sapphire's script and a live recording of the quest:
-- Sequence 0:   ACTOR0 (Theodore) offers the quest (scene 0, returns 1 when accepted.)
-- Sequence 1:   talk to ACTOR1 (Roseline) (scene 1.) Retail sends notice 0 (no arguments) as the quest moves on to
--               sequence 2.
-- Sequence 2:   search EOBJECT1, the worn leather bag (scene 3, EVENT_ACTION_SEARCH, then scene 98, as retail plays
--               searches: Bob's Chronofoil capture of 2026-10-14.) It holds ITEM0, counted in
--               UI8BH (the client lists it as a key item from that.) Retail sends notice 1 with the item's icon as the
--               quest moves on to sequence 255.
-- Sequence 255: hand ITEM0 to ACTOR1 (scene 5, the trade window; scene 94 when it's handed over, with the reward.)
-- EOBJECT0, the gutted campfire, can be searched in every sequence (scenes 2, 4 and 6, then 100, 96 and 92), but it
-- doesn't advance anything.

SCENE_OFFER = 0
SCENE_ROSELINE = 1
SCENE_BAG = 3
SCENE_BAG_FOUND = 98
SCENE_TRADE = 5
SCENE_TRADE_OK = 94

-- The campfire's scenes (before and after the progress bar) in each sequence.
CAMPFIRE_SCENES = {
    [1] = { 2, 100 },
    [2] = { 4, 96 },
    [255] = { 6, 92 },
}

-- The quest's notices, as retail sends them. The second carries the bag's icon (ITEM0's), as retail sends it.
NOTICE_SEQUENCE = 0
NOTICE_ITEM = 1
ITEM0_ICON = 25919

SEARCH_SCENE_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA

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
-- The search under way: { its scene, the scene to play once the progress bar finishes } (see onEventActionCast.)
searching = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

-- A search: the object's scene, the progress bar when it ends (sceneFinished), then the "found it" scene.
function search(player, scene, found)
    searching = { scene, found }
    play(player, scene, SEARCH_SCENE_FLAGS)
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_ROSELINE, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == EOBJECT1 then
        search(player, SCENE_BAG, SCENE_BAG_FOUND)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    elseif sequence ~= nil and BASE_ID == EOBJECT0 and CAMPFIRE_SCENES[sequence] ~= nil then
        search(player, CAMPFIRE_SCENES[sequence][1], CAMPFIRE_SCENES[sequence][2])
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    if searching == nil then
        player:finish_event()
        return
    end
    local found = searching[2]
    searching = nil
    play(player, found, SEARCH_SCENE_FLAGS)
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil
    local sequence = player:get_quest_sequence(EVENT_ID)

    if searching ~= nil and scene == searching[1] then
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif scene == SCENE_OFFER then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID)
            player:quest_sequence(EVENT_ID, 1)
        end
    elseif scene == SCENE_ROSELINE and sequence == 1 then
        player:event_notice(EVENT_ID, NOTICE_SEQUENCE)
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_BAG_FOUND and sequence == 2 then
        player:event_notice(EVENT_ID, NOTICE_ITEM, { ITEM0_ICON })
        player:quest_sequence(EVENT_ID, 255)
        setVar(player, 1, true, 1) -- UI8BH: ITEM0
    elseif scene == SCENE_TRADE then
        -- 1 when ITEM0 was handed over, nothing when the trade window was closed.
        if results[1] == 1 then
            play(player, SCENE_TRADE_OK, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_TRADE_OK then
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
