-- Quest: The Gridanian Envoy (Gridania)
--
-- Checked against Sapphire's script (its scene list is the client's) and a live recording of the quest (questkit
-- replay); the notices and variables are retail's, which differ from Sapphire's in places. The positions after each
-- flight and ticket counter are Sapphire's.
-- TODO: dump it with kawari-questdump (the Quest Lab) to check the scene numbers against the client's script.
-- Sequence 0:   ACTOR1 (the silent conjurer) lets you into the Lotus Stand (scene 2), where ACTOR0 (Kan-E-Senna) offers
--               the quest (scene 0, returns 1 when accepted) and explains it (cutscene 1.) You carry her two letters,
--               ITEM0 and ITEM1 (UI8BH and UI8BL.)
-- Sequence 1:   ACTOR2 (Mother Miounne) (cutscene 3): notice 0, sequence 2.
-- Sequence 2:   ACTOR3 (Lionnellais, Gridania's airship ticketer) (scene 4) takes you up to the landing: sequence 3.
-- Sequence 3:   board at EOBJECT1, the gangplank (EVENT_ACTION_WAITING_SHOR, then scene 7, a yes/no, then the flight,
--               cutscene 8): you land in Limsa Lominsa (TERRITORYTYPE2) at sequence 4. ACTOR3 takes you up again (9.)
-- Sequence 4:   ACTOR4 (the Storm Honor Guard) (scene 11): notice 3, sequence 5. ACTOR5 (the arrivals attendant) takes
--               you down (scene 12.)
-- Sequence 5:   ACTOR6 (Zanthael) (scene 16, a yes/no, then 17 and the audience, cutscene 18) takes ITEM1 (UI8BL = 0):
--               notice 4, sequence 6.
-- Sequence 6:   ACTOR7 (L'nophlo, Limsa's ticketer) (scene 24) takes you up to the landing: sequence 7.
-- Sequence 7:   board at EOBJECT3 (scene 29, then the flight, cutscene 30): you land in Ul'dah (TERRITORYTYPE3) at
--               sequence 8. ACTOR7 takes you up again (scene 31.)
-- Sequence 8:   ACTOR8 (the Flame Honor Guard) (scene 35): notice 7, sequence 255. ACTOR9 takes you down (scene 36.)
-- Sequence 255: ACTOR10 (Bartholomew) (scene 42, then 43, the audience cutscene 44 and the reward, 45.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_LOTUS_DOOR = 2
SCENE_MIOUNNE = 3
SCENE_GRIDANIA_TICKETS = 4
SCENE_GRIDANIA_BOARD = 7
SCENE_FLIGHT_TO_LIMSA = 8
SCENE_GRIDANIA_TICKETS_AGAIN = 9
SCENE_STORM_GUARD = 11
SCENE_LIMSA_ATTENDANT = 12
SCENE_ZANTHAEL = 16
SCENE_ZANTHAEL_2 = 17
SCENE_ADMIRAL = 18
SCENE_LIMSA_TICKETS = 24
SCENE_LIMSA_BOARD = 29
SCENE_FLIGHT_TO_ULDAH = 30
SCENE_LIMSA_TICKETS_AGAIN = 31
SCENE_FLAME_GUARD = 35
SCENE_ULDAH_ATTENDANT = 36
SCENE_BARTHOLOMEW = 42
SCENE_BARTHOLOMEW_2 = 43
SCENE_FLAME_GENERAL = 44
SCENE_REWARD = 45

NOTICE_MIOUNNE = 0
NOTICE_LIMSA = 3
NOTICE_ADMIRAL = 4
NOTICE_ULDAH = 7

GRIDANIA = TERRITORYTYPE1 or 132
LIMSA = TERRITORYTYPE2 or 128
ULDAH = TERRITORYTYPE3 or 130
LOTUS_STAND = TERRITORYTYPE0 or 205
LOTUS_STAND_ENTRANCE = POPRANGE0 or 3877982
-- The progress bar before boarding (Sapphire uses EventAction 10.)
EVENT_ACTION_BOARD = EVENT_ACTION_WAITING_SHOR or 10

-- Where the ticket counters, the attendants and the flights leave you (Sapphire's positions.)
GRIDANIA_LANDING = { position = { x = 22.55, y = -19.0, z = 114.2 }, rotation = 0.0 }
LIMSA_LANDING = { position = { x = -24.3, y = 92.0, z = 2.47 }, rotation = -0.37 }
LIMSA_ARRIVAL = { position = { x = -11.86, y = 92.0, z = 17.0 }, rotation = -1.95 }
LIMSA_BELOW = { position = { x = -13.95, y = 91.5, z = -5.88 }, rotation = 2.89 }
ULDAH_ARRIVAL = { position = { x = -44.14, y = 84.0, z = -0.47 }, rotation = 1.3 }
ULDAH_BELOW = { position = { x = -19.45, y = 83.2, z = 3.85 }, rotation = 1.8 }

CUTSCENE_FLAGS = FADE_OUT | CONDITION_CUTSCENE | HIDE_UI

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

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

-- Where the scenes that only move you about leave you.
MOVES = {
    [SCENE_GRIDANIA_TICKETS_AGAIN] = GRIDANIA_LANDING,
    [SCENE_LIMSA_ATTENDANT] = LIMSA_BELOW,
    [SCENE_LIMSA_TICKETS_AGAIN] = LIMSA_LANDING,
    [SCENE_ULDAH_ATTENDANT] = ULDAH_BELOW,
}

-- The scene the boarding progress bar leads to.
boarding = nil

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR1 then
        play(player, SCENE_LOTUS_DOOR, HIDE_HOTBAR)
    elseif sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR2 then
        play(player, SCENE_MIOUNNE, CUTSCENE_FLAGS)
    elseif sequence == 2 and BASE_ID == ACTOR3 then
        play(player, SCENE_GRIDANIA_TICKETS, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR3 then
        play(player, SCENE_GRIDANIA_TICKETS_AGAIN, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == EOBJECT1 then
        boarding = SCENE_GRIDANIA_BOARD
        player:event_action(EVENT_ACTION_BOARD)
    elseif sequence == 4 and BASE_ID == ACTOR4 then
        play(player, SCENE_STORM_GUARD, HIDE_HOTBAR)
    elseif sequence == 4 and BASE_ID == ACTOR5 then
        play(player, SCENE_LIMSA_ATTENDANT, HIDE_HOTBAR)
    elseif sequence == 5 and BASE_ID == ACTOR6 then
        play(player, SCENE_ZANTHAEL, HIDE_HOTBAR)
    elseif sequence == 6 and BASE_ID == ACTOR7 then
        play(player, SCENE_LIMSA_TICKETS, HIDE_HOTBAR)
    elseif sequence == 7 and BASE_ID == ACTOR7 then
        play(player, SCENE_LIMSA_TICKETS_AGAIN, HIDE_HOTBAR)
    elseif sequence == 7 and BASE_ID == EOBJECT3 then
        boarding = SCENE_LIMSA_BOARD
        player:event_action(EVENT_ACTION_BOARD)
    elseif sequence == 8 and BASE_ID == ACTOR8 then
        play(player, SCENE_FLAME_GUARD, HIDE_HOTBAR)
    elseif sequence == 8 and BASE_ID == ACTOR9 then
        play(player, SCENE_ULDAH_ATTENDANT, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR10 then
        play(player, SCENE_BARTHOLOMEW, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    if boarding == nil then
        player:finish_event()
        return
    end
    local scene = boarding
    boarding = nil
    play(player, scene, HIDE_HOTBAR)
end

-- Ends the event and moves the player, to another zone or within this one.
function move(player, territory, place)
    player:finish_event()
    if territory ~= nil then
        player:change_territory(territory, place.position, place.rotation)
    else
        player:set_position(place.position, place.rotation)
    end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if scene == SCENE_LOTUS_DOOR then
        if results[1] == 1 then
            player:finish_event()
            player:change_territory_pop_range(LOTUS_STAND, LOTUS_STAND_ENTRANCE)
            return
        end
    elseif scene == SCENE_OFFER then
        if results[1] == 1 then
            play(player, SCENE_ACCEPTED, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
        player:set_quest_var(EVENT_ID, 1, 0x11) -- UI8BH and UI8BL: ITEM0 and ITEM1, the letters
    elseif scene == SCENE_MIOUNNE then
        player:event_notice(EVENT_ID, NOTICE_MIOUNNE)
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_GRIDANIA_TICKETS then
        player:quest_sequence(EVENT_ID, 3)
        move(player, nil, GRIDANIA_LANDING)
        return
    elseif MOVES[scene] ~= nil then
        move(player, nil, MOVES[scene])
        return
    elseif scene == SCENE_GRIDANIA_BOARD then
        if results[1] == 1 then
            play(player, SCENE_FLIGHT_TO_LIMSA, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_FLIGHT_TO_LIMSA then
        player:quest_sequence(EVENT_ID, 4)
        move(player, LIMSA, LIMSA_ARRIVAL)
        return
    elseif scene == SCENE_STORM_GUARD then
        player:event_notice(EVENT_ID, NOTICE_LIMSA)
        player:quest_sequence(EVENT_ID, 5)
    elseif scene == SCENE_ZANTHAEL then
        if results[1] == 1 then
            play(player, SCENE_ZANTHAEL_2, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_ZANTHAEL_2 then
        play(player, SCENE_ADMIRAL, CUTSCENE_FLAGS)
        return
    elseif scene == SCENE_ADMIRAL then
        setVar(player, 1, false, 0) -- UI8BL: ITEM1 was handed over
        player:event_notice(EVENT_ID, NOTICE_ADMIRAL)
        player:quest_sequence(EVENT_ID, 6)
    elseif scene == SCENE_LIMSA_TICKETS then
        player:quest_sequence(EVENT_ID, 7)
        move(player, nil, LIMSA_LANDING)
        return
    elseif scene == SCENE_LIMSA_BOARD then
        if results[1] == 1 then
            play(player, SCENE_FLIGHT_TO_ULDAH, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_FLIGHT_TO_ULDAH then
        player:quest_sequence(EVENT_ID, 8)
        move(player, ULDAH, ULDAH_ARRIVAL)
        return
    elseif scene == SCENE_FLAME_GUARD then
        player:event_notice(EVENT_ID, NOTICE_ULDAH)
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_BARTHOLOMEW then
        if results[1] == 1 then
            play(player, SCENE_BARTHOLOMEW_2, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_BARTHOLOMEW_2 then
        play(player, SCENE_FLAME_GENERAL, CUTSCENE_FLAGS)
        return
    elseif scene == SCENE_FLAME_GENERAL then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
        return
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
