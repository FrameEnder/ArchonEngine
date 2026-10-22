-- Quest: The Lominsan Envoy (Limsa Lominsa)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-21) and Bob's Chronofoil capture of the Limsa
-- story on retail (2026-10-21). It's The Ul'dahn Envoy (ManWil200_00528.lua) from Limsa Lominsa: Gridania, then Ul'dah.
-- Sequence 0:   ACTOR0 (Merlwyb) offers it (scene 0, a yield: 1 accepts), then the cutscene (scene 1, where the client
--               accepts it): sequence 1 with her two letters, ITEM0 and ITEM1 (UI8BH and UI8BL.)
-- Sequence 1:   ACTOR2 (Baderon) (cutscene 3): notice 0, sequence 2.
-- Sequence 2:   ACTOR3 (L'nophlo, Limsa's airship ticketer) (scene 4, a yes/no) takes you up to the landing
--               (POPRANGE1, through a loading screen as on retail): sequence 3. "No" is the scene's error code 50 (the
--               client's script cancels the scene) and changes nothing.
-- Sequence 3:   board at EOBJECT1, the gangplank (scene 6, then EVENT_ACTION_WAITING_SHOR, then scene 7, a yes/no, then
--               the flight, cutscene 8): you land in Gridania (TERRITORYTYPE2, POPRANGE2) at sequence 4. ACTOR3 takes
--               you up again (scene 9, a yes/no.)
-- Sequence 4:   ACTOR4 (the serpent honor guard) (scene 11): notice 3, sequence 5. ACTOR5 (the arrivals attendant) takes
--               you down (scene 12, a yes/no; POPRANGE3.)
-- Sequence 5:   ACTOR6 (Old Gridania) takes ITEM0 (hand-over scene 16, a yield: 1 when handed over; then 17 and the
--               audience, cutscene 18): UI8BL 0 (as retail cleared it), notice 4, sequence 6. ACTOR4 only has a line
--               (scene 20.)
-- Sequence 6:   ACTOR7 (Lionnellais, Gridania's ticketer) (scene 24, a yes/no) takes you up to the landing (POPRANGE4):
--               sequence 7.
-- Sequence 7:   board at EOBJECT3 (scene 28, the bar, scene 29, then the flight, cutscene 30): you land in Ul'dah
--               (TERRITORYTYPE3, POPRANGE5) at sequence 8. ACTOR7 takes you up again (scene 31, a yes/no.)
-- Sequence 8:   ACTOR8 (the flame honor guard) (scene 35): notice 7, sequence 255. ACTOR9 (the arrivals attendant)
--               takes you down (scene 36, a yes/no; POPRANGE6.)
-- Sequence 255: ACTOR10 (the Steps of Thal) takes ITEM1 (hand-over scene 42, then 43 and the cutscene 44) and rewards it
--               (scene 45, a yield with the reward picked.) ACTOR8 only has a line (scene 47.)
-- Not taken on retail (client script only): ACTOR1 (Zanthael) offers a yes/no in sequences 0 and 1 (scene 2), taken
-- here as the way back into the Command room (TERRITORYTYPE0, POPRANGE0, where A Mizzenmast Repast's yes/no leaves you).
-- Once you've flown on, the ticketers' yes/no is followed by an airship cutscene: ACTOR3 in sequences 4-7 (scenes
-- 14/15, 22/23, 26/27, 33/34) and 8-255 (38/39, 49/50), ACTOR7 in 8-255 (40/41, 51/52). This script takes them to the
-- quest's next city's arrival (Gridania's POPRANGE2, then Ul'dah's POPRANGE5), the pop ranges retail's flights landed
-- on; unseen on retail. EOBJECT0, EOBJECT2 and EOBJECT4 are offered too, but their scenes are empty: the event ends.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_ZANTHAEL = 2
SCENE_BADERON = 3
SCENE_LIMSA_TICKETS = 4
SCENE_LIMSA_CHECK = 6
SCENE_LIMSA_BOARD = 7
SCENE_FLIGHT_TO_GRIDANIA = 8
SCENE_LIMSA_TICKETS_AGAIN = 9
SCENE_SERPENT_GUARD = 11
SCENE_GRIDANIA_ATTENDANT = 12
SCENE_HAND_OVER = 16
SCENE_HAND_OVER_2 = 17
SCENE_AUDIENCE = 18
SCENE_SERPENT_GUARD_AGAIN = 20
SCENE_GRIDANIA_TICKETS = 24
SCENE_GRIDANIA_CHECK = 28
SCENE_GRIDANIA_BOARD = 29
SCENE_FLIGHT_TO_ULDAH = 30
SCENE_GRIDANIA_TICKETS_AGAIN = 31
SCENE_FLAME_GUARD = 35
SCENE_ULDAH_ATTENDANT = 36
SCENE_THAL = 42
SCENE_THAL_2 = 43
SCENE_ULDAH_AUDIENCE = 44
SCENE_REWARD = 45
SCENE_FLAME_GUARD_AGAIN = 47

NOTICE_BADERON = 0
NOTICE_GRIDANIA = 3
NOTICE_HANDED_OVER = 4
NOTICE_ULDAH = 7

COMMAND_ROOM = TERRITORYTYPE0 or 198
LIMSA = TERRITORYTYPE1 or 128
GRIDANIA = TERRITORYTYPE2 or 132
ULDAH = TERRITORYTYPE3 or 130
-- Where each move leaves you (the Quest sheet's parameters; retail's ZoneInit pop ranges for the moves it made, Bob's
-- Chronofoil capture of 2026-10-21.)
COMMAND_ROOM_ENTRANCE = POPRANGE0 or 4146949
LIMSA_LANDING = POPRANGE1 or 4103402
GRIDANIA_ARRIVAL = POPRANGE2 or 4102803
GRIDANIA_BELOW = POPRANGE3 or 4103382
GRIDANIA_LANDING = POPRANGE4 or 4103392
ULDAH_ARRIVAL = POPRANGE5 or 4102780
ULDAH_BELOW = POPRANGE6 or 4103339

-- The progress bar before boarding (retail: EventAction 11.)
EVENT_ACTION_BOARD = EVENT_ACTION_WAITING_SHOR or 11
-- A yes/no answered No (its error code: the client's script cancels the scene.)
SCENE_DECLINED = 50
-- How retail plays the gangplanks' check and yes/no scenes: HIDE_HOTBAR | NO_DEFAULT_CAMERA.
GANGPLANK_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB, as retail plays its cutscenes

-- Each gangplank's yes/no, after its check scene and the progress bar.
BOARDING_AFTER_CHECK = {
    [SCENE_LIMSA_CHECK] = SCENE_LIMSA_BOARD,
    [SCENE_GRIDANIA_CHECK] = SCENE_GRIDANIA_BOARD,
}

-- The yes/nos that only move you about (unless answered No), and where they leave you.
MOVES = {
    [SCENE_ZANTHAEL] = { COMMAND_ROOM, COMMAND_ROOM_ENTRANCE },
    [SCENE_LIMSA_TICKETS_AGAIN] = { LIMSA, LIMSA_LANDING },
    [SCENE_GRIDANIA_ATTENDANT] = { GRIDANIA, GRIDANIA_BELOW },
    [SCENE_GRIDANIA_TICKETS_AGAIN] = { GRIDANIA, GRIDANIA_LANDING },
    [SCENE_ULDAH_ATTENDANT] = { ULDAH, ULDAH_BELOW },
}

-- The ticketers' scenes by sequence (client script.)
LNOPHLO_SCENES = { [2] = SCENE_LIMSA_TICKETS, [3] = SCENE_LIMSA_TICKETS_AGAIN, [4] = 14, [5] = 22, [6] = 26, [7] = 33,
    [8] = 38, [255] = 49 }
LIONNELLAIS_SCENES = { [6] = SCENE_GRIDANIA_TICKETS, [7] = SCENE_GRIDANIA_TICKETS_AGAIN, [8] = 40, [255] = 51 }

-- After you've flown on, the ticketers' yes/no and its airship cutscene, and where it leaves you (not seen on retail.)
FLIGHTS = {
    [14] = { 15, GRIDANIA, GRIDANIA_ARRIVAL },
    [22] = { 23, GRIDANIA, GRIDANIA_ARRIVAL },
    [26] = { 27, GRIDANIA, GRIDANIA_ARRIVAL },
    [33] = { 34, GRIDANIA, GRIDANIA_ARRIVAL },
    [38] = { 39, ULDAH, ULDAH_ARRIVAL },
    [49] = { 50, ULDAH, ULDAH_ARRIVAL },
    [40] = { 41, ULDAH, ULDAH_ARRIVAL },
    [51] = { 52, ULDAH, ULDAH_ARRIVAL },
}
-- The cutscene that's playing, and where it leaves you.
flight = nil

-- Quest variables: UI8A..F are bytes 0..5, H is the high nibble and L the low one.
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

-- The scene the boarding progress bar leads to.
boarding = nil

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif (sequence == nil or sequence == 1) and BASE_ID == ACTOR1 then
        play(player, SCENE_ZANTHAEL, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR2 then
        play(player, SCENE_BADERON, CUTSCENE_FLAGS)
    elseif sequence ~= nil and BASE_ID == ACTOR3 and LNOPHLO_SCENES[sequence] ~= nil then
        play(player, LNOPHLO_SCENES[sequence], HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == EOBJECT1 then
        play(player, SCENE_LIMSA_CHECK, GANGPLANK_FLAGS)
    elseif sequence == 4 and BASE_ID == ACTOR4 then
        play(player, SCENE_SERPENT_GUARD, HIDE_HOTBAR)
    elseif sequence == 4 and BASE_ID == ACTOR5 then
        play(player, SCENE_GRIDANIA_ATTENDANT, HIDE_HOTBAR)
    elseif sequence == 5 and BASE_ID == ACTOR6 then
        play(player, SCENE_HAND_OVER, HIDE_HOTBAR)
    elseif sequence == 5 and BASE_ID == ACTOR4 then
        play(player, SCENE_SERPENT_GUARD_AGAIN, HIDE_HOTBAR)
    elseif sequence ~= nil and BASE_ID == ACTOR7 and LIONNELLAIS_SCENES[sequence] ~= nil then
        play(player, LIONNELLAIS_SCENES[sequence], HIDE_HOTBAR)
    elseif sequence == 7 and BASE_ID == EOBJECT3 then
        play(player, SCENE_GRIDANIA_CHECK, GANGPLANK_FLAGS)
    elseif sequence == 8 and BASE_ID == ACTOR8 then
        play(player, SCENE_FLAME_GUARD, HIDE_HOTBAR)
    elseif sequence == 8 and BASE_ID == ACTOR9 then
        play(player, SCENE_ULDAH_ATTENDANT, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR10 then
        play(player, SCENE_THAL, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR8 then
        play(player, SCENE_FLAME_GUARD_AGAIN, HIDE_HOTBAR)
    else
        -- EOBJECT0, EOBJECT2 and EOBJECT4's scenes are empty.
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
    play(player, scene, GANGPLANK_FLAGS)
end

-- Ends the event and moves the player, to another zone or within this one (retail moves them within a zone through a
-- loading screen too.)
function move(player, territory, pop_range)
    player:finish_event()
    player:change_territory_pop_range(territory, pop_range)
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
        player:set_quest_var(EVENT_ID, 1, 0x11) -- UI8BH and UI8BL: ITEM0 and ITEM1, the letters
    elseif scene == SCENE_BADERON then
        player:event_notice(EVENT_ID, NOTICE_BADERON)
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_LIMSA_TICKETS then
        if player.scene_error_code ~= SCENE_DECLINED then
            player:quest_sequence(EVENT_ID, 3)
            move(player, LIMSA, LIMSA_LANDING)
            return
        end
    elseif MOVES[scene] ~= nil then
        if player.scene_error_code ~= SCENE_DECLINED then
            move(player, MOVES[scene][1], MOVES[scene][2])
            return
        end
    elseif FLIGHTS[scene] ~= nil then
        if player.scene_error_code ~= SCENE_DECLINED then
            flight = FLIGHTS[scene]
            play(player, flight[1], CUTSCENE_FLAGS)
            return
        end
    elseif flight ~= nil and scene == flight[1] then
        local territory, pop_range = flight[2], flight[3]
        flight = nil
        move(player, territory, pop_range)
        return
    elseif BOARDING_AFTER_CHECK[scene] ~= nil then
        -- The check passed (retail goes straight on): the progress bar, then the yes/no (see onEventActionCast.)
        boarding = BOARDING_AFTER_CHECK[scene]
        player:event_action(EVENT_ACTION_BOARD)
        return
    elseif scene == SCENE_LIMSA_BOARD then
        if player.scene_error_code ~= SCENE_DECLINED then
            play(player, SCENE_FLIGHT_TO_GRIDANIA, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_FLIGHT_TO_GRIDANIA then
        player:quest_sequence(EVENT_ID, 4)
        move(player, GRIDANIA, GRIDANIA_ARRIVAL)
        return
    elseif scene == SCENE_SERPENT_GUARD then
        player:event_notice(EVENT_ID, NOTICE_GRIDANIA)
        player:quest_sequence(EVENT_ID, 5)
    elseif scene == SCENE_HAND_OVER then
        if results[1] == 1 then
            play(player, SCENE_HAND_OVER_2, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_HAND_OVER_2 then
        play(player, SCENE_AUDIENCE, CUTSCENE_FLAGS)
        return
    elseif scene == SCENE_AUDIENCE then
        setVar(player, 1, false, 0) -- UI8BL: retail cleared it here (vars 0x11 -> 0x10)
        player:event_notice(EVENT_ID, NOTICE_HANDED_OVER)
        player:quest_sequence(EVENT_ID, 6)
    elseif scene == SCENE_GRIDANIA_TICKETS then
        if player.scene_error_code ~= SCENE_DECLINED then
            player:quest_sequence(EVENT_ID, 7)
            move(player, GRIDANIA, GRIDANIA_LANDING)
            return
        end
    elseif scene == SCENE_GRIDANIA_BOARD then
        if player.scene_error_code ~= SCENE_DECLINED then
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
    elseif scene == SCENE_THAL then
        if results[1] == 1 then
            play(player, SCENE_THAL_2, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_THAL_2 then
        play(player, SCENE_ULDAH_AUDIENCE, CUTSCENE_FLAGS)
        return
    elseif scene == SCENE_ULDAH_AUDIENCE then
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
