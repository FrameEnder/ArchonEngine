-- Quest: A Hero in the Making (main scenario)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-27) and Bob's Chronofoil capture b8432613
-- (2026-10-27, the whole quest; its fixture is tools/questkit/fixtures/511.json).
-- Sequence 0:   ACTOR0 (Minfilia) offers it (scene 0, a yield returning 1 when accepted), then the cutscene (scene 1,
--               retail's flags 0xF8402EFB; the client accepts it there): sequence 1, and the player is put back where
--               they stood to talk to her (retail's PrepareZoning to the Waking Sands and ActorSetPos: Dags' run, Bob's
--               Chronofoil capture c99fe374, at exactly the spot he talked from; Bob's own another spot.)
-- Sequence 1:   ACTOR1 (Tataru, scene 2): notice 0, sequence 2.
-- Sequence 2:   the three Grand Companies' places, in any order, each its todo (the client's IsAcceptEvent):
--               * EOBJECT0, in Gridania (todo 1): scene 6 (0x2001), EVENT_ACTION_WAITING's bar, then the cutscene
--                 (scene 7): UI8BH 1 and BitFlag24 1 (0x80 in UI8D);
--               * EOBJECT1, in Ul'dah (todo 2): scenes 8 and 9 the same way: UI8BL 1 and BitFlag24 2 (0x40);
--               * ACTOR5 (Zanthael, in Limsa Lominsa, todo 3): scene 10 asks (a yield returning 1 for yes, the client
--                 cancels the scene otherwise), then the cutscene (scene 11): UI8AL 1 and BitFlag24 3 (0x20).
--               UI8AH counts them (retail: 101000800000 after Gridania, 201100C00000 after Ul'dah). Each one's notice
--               is its todo's number (no arguments); the last moves to sequence 255 with every variable cleared.
-- Sequence 255: ACTOR24 (Minfilia) rewards it (scene 33, a yield returning 1), then the cutscene (scene 34, parameter 0;
--               retail set UI8AH and UI8AL to 1 as it began), and once it ends the quest is completed.
-- The officers' lines (ACTOR2-4, the flame, serpent and storm officers) go by the client's IsAcceptEvent order: scenes
-- 3-5 in sequence 1, 30-32 in sequence 2, 35-37 in sequence 255. Its other scenes (12-29, ACTOR6-23) are empty.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_TATARU = 2
SCENE_GRIDANIA = 6
SCENE_GRIDANIA_CUTSCENE = 7
SCENE_ULDAH = 8
SCENE_ULDAH_CUTSCENE = 9
SCENE_ZANTHAEL = 10
SCENE_LIMSA_CUTSCENE = 11
SCENE_REWARD = 33
SCENE_CUTSCENE = 34

-- Retail's flags (Bob's Chronofoil capture b8432613.)
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001
CUTSCENE_FLAGS = 0xF8402EFB

-- Each place: its object (or NPC), its scenes, and what it sets (`done`.)
--   byte, value: the UI8 it sets (UI8BH, UI8BL or UI8AL), as `byte` and the bits to OR in; flag: its BitFlag24 bit.
PLACES = {
    [EOBJECT0] = { scene = SCENE_GRIDANIA, cutscene = SCENE_GRIDANIA_CUTSCENE, todo = 1, byte = 1, value = 0x10, flag = 0x80 },
    [EOBJECT1] = { scene = SCENE_ULDAH, cutscene = SCENE_ULDAH_CUTSCENE, todo = 2, byte = 1, value = 0x01, flag = 0x40 },
    [ACTOR5] = { scene = SCENE_ZANTHAEL, cutscene = SCENE_LIMSA_CUTSCENE, todo = 3, byte = 0, value = 0x01, flag = 0x20 },
}
-- UI8D, BitFlag24's byte.
FLAG_BYTE = 3
PLACES_IN_ALL = 3

-- Where the opening cutscene leaves the player: the Waking Sands, where they stood (PrepareZoning, then ActorSetPos.)
WAKING_SANDS = 212

-- The others' lines in each sequence: { [sequence] = { [base ID] = scene } }.
LINES = {
    [1] = { [ACTOR2] = 3, [ACTOR3] = 4, [ACTOR4] = 5 },
    [2] = { [ACTOR2] = 30, [ACTOR3] = 31, [ACTOR4] = 32 },
    [255] = { [ACTOR2] = 35, [ACTOR3] = 36, [ACTOR4] = 37 },
}

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- The place whose cutscene is playing.
current_place = nil
-- The reward chosen in scene 33 (none: the Quest sheet's are fixed), kept until the cutscene after it ends.
reward = 0

function play(player, scene, flags, params)
    current_scene = scene
    player:play_scene(scene, flags, params or {})
end

-- Whether a place's todo is done (its BitFlag24 bit.)
function visited(player, place)
    return (player:get_quest_var(EVENT_ID, FLAG_BYTE) & place.flag) ~= 0
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local line = sequence ~= nil and LINES[sequence] ~= nil and LINES[sequence][BASE_ID] or nil
    local place = PLACES[BASE_ID]

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_TATARU, HIDE_HOTBAR)
    elseif sequence == 2 and place ~= nil and not visited(player, place) then
        current_place = place
        local flags = BASE_ID == ACTOR5 and HIDE_HOTBAR or OBJECT_FLAGS
        play(player, place.scene, flags)
    elseif sequence == 255 and BASE_ID == ACTOR24 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    elseif line ~= nil then
        play(player, line, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- The bar after a place's object (EVENT_ACTION_WAITING, 2 s on retail) finished: its cutscene.
function onEventActionCast(player)
    if player:get_quest_sequence(EVENT_ID) ~= 2 or current_place == nil then
        player:finish_event()
        return
    end
    play(player, current_place.cutscene, CUTSCENE_FLAGS)
end

-- A place visited: its variables, its todo's notice, and after the third, sequence 255 with the variables cleared.
function placeVisited(player, place)
    local count = (player:get_quest_var(EVENT_ID, 0) >> 4) + 1
    player:event_notice(EVENT_ID, place.todo, {})
    if count >= PLACES_IN_ALL then
        for byte = 0, 5 do
            player:set_quest_var(EVENT_ID, byte, 0)
        end
        player:quest_sequence(EVENT_ID, 255)
        return
    end
    player:set_quest_var(EVENT_ID, place.byte, player:get_quest_var(EVENT_ID, place.byte) | place.value)
    player:set_quest_var(EVENT_ID, 0, (count << 4) | (player:get_quest_var(EVENT_ID, 0) & 0x0F))
    player:set_quest_var(EVENT_ID, FLAG_BYTE, player:get_quest_var(EVENT_ID, FLAG_BYTE) | place.flag)
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
        player:finish_event()
        local stood = player.position
        player:change_territory(WAKING_SANDS, { x = stood.x, y = stood.y, z = stood.z }, player.rotation)
        return
    elseif scene == SCENE_TATARU then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_GRIDANIA or scene == SCENE_ULDAH then
        -- A short wait shows a progress bar; the cutscene plays once it finishes (see onEventActionCast.)
        player:event_action(EVENT_ACTION_WAITING)
        return
    elseif scene == SCENE_ZANTHAEL then
        if results[1] == 1 and current_place ~= nil then
            play(player, current_place.cutscene, CUTSCENE_FLAGS)
            return
        end
        current_place = nil
    elseif scene == SCENE_GRIDANIA_CUTSCENE or scene == SCENE_ULDAH_CUTSCENE or scene == SCENE_LIMSA_CUTSCENE then
        if current_place ~= nil then
            placeVisited(player, current_place)
        end
        current_place = nil
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            reward = results[2] or 0
            play(player, SCENE_CUTSCENE, CUTSCENE_FLAGS, { 0 })
            player:set_quest_var(EVENT_ID, 0, 0x11) -- UI8AH and UI8AL, as retail set them
            return
        end
    elseif scene == SCENE_CUTSCENE then
        player:finish_event()
        player:finish_quest(EVENT_ID, reward)
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
