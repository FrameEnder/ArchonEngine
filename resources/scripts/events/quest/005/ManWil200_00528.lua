-- Quest: The Ul'dahn Envoy (Ul'dah)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-20) and retail (Bob's Chronofoil capture of
-- 2026-10-18, questkit replay 528):
-- Sequence 0:   ACTOR0 (Raubahn) offers it (scene 0, a yield: the client's QuestOffer, 1 when accepted), then the
--               cutscene (scene 1, where the client accepts it): sequence 1 with his two letters, ITEM0 and ITEM1
--               (UI8BH and UI8BL.)
-- Sequence 1:   ACTOR1 (Momodi) (cutscene 2): notice 0, sequence 2. ACTOR0 only has a line (scene 3.)
-- Sequence 2:   ACTOR2 (Elyenora, Ul'dah's airship ticketer) (scene 4, a yes/no) takes you up to the landing (POPRANGE0,
--               through a loading screen as on retail): sequence 3. "No" is the scene's error code 50 (the client's
--               script cancels the scene) and changes nothing.
-- Sequence 3:   board at EOBJECT1, the gangplank (scene 6, which only waits, then EVENT_ACTION_WAITING_SHOR, then scene 7,
--               a yes/no, then the flight, cutscene 8): you land in Limsa Lominsa (TERRITORYTYPE1, POPRANGE1) at
--               sequence 4. ACTOR2 takes you up again (scene 9, a yes/no.)
-- Sequence 4:   ACTOR3 (the storm honor guard) (scene 11): notice 3, sequence 5. ACTOR4 (the arrivals attendant) takes
--               you down (scene 12, a yes/no; POPRANGE2.)
-- Sequence 5:   ACTOR5 (Zanthael, Limsa Lominsa Lower Decks) takes ITEM0 (hand-over scene 16, a yield: 1 when handed
--               over, nothing when the window was closed; then 17 and the audience, cutscene 18): UI8BH 0, notice 4,
--               sequence 6. ACTOR3 only has a line (scene 20.)
-- Sequence 6:   ACTOR6 (L'nophlo, Limsa's ticketer) (scene 24, a yes/no) takes you up to the landing (POPRANGE3):
--               sequence 7.
-- Sequence 7:   board at EOBJECT3 (scene 28, the bar, scene 29, then the flight, cutscene 30): you land in Gridania
--               (TERRITORYTYPE2, POPRANGE4) at sequence 8. ACTOR6 takes you up again (scene 31, a yes/no.)
-- Sequence 8:   ACTOR7 (the serpent honor guard) (scene 35): notice 7, sequence 255. ACTOR8 (the arrivals attendant)
--               takes you down (scene 37, a yes/no; POPRANGE5.)
-- Sequence 255: ACTOR9 (the silent conjurer, Old Gridania) takes ITEM1 (hand-over scene 42, then 43 and the cutscene 44)
--               and rewards it (scene 45, a yield: QuestReward, 1 and the chosen item; the client cancels the trade
--               otherwise.) ACTOR7 only has a line (scene 47.)
-- Not taken on retail (client script only): once you've flown on, the ticketers' yes/no is followed by an airship
-- cutscene: ACTOR2 in sequences 4-7 (scenes 14/15, 22/23, 26/27, 33/34) and 8-255 (38/39, 49/50, with another line),
-- ACTOR6 in 8-255 (40/41, 51/52). This script takes them to the quest's next city's arrival (Limsa Lominsa's
-- POPRANGE1, then Gridania's POPRANGE4), the pop ranges retail's flights landed on; unseen on retail. EOBJECT0
-- (sequences 2-3), EOBJECT2 (4-7) and EOBJECT4 (8-255) are offered too, but their scenes (5, 10, 13, 21, 25, 32, 36,
-- 48) are empty: the event just ends. Scenes 19 and 46 are empty and played by nothing. The client offers each step's
-- main NPC or object only while UI8AL < 1 and BitFlag8 1 is unset, which this quest never sets.
-- Notices: 0, 3, 4 and 7 (those todos done), none with arguments; the variables are only the letters.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_MOMODI = 2
SCENE_RAUBAHN = 3
SCENE_ULDAH_TICKETS = 4
SCENE_ULDAH_CHECK = 6
SCENE_ULDAH_BOARD = 7
SCENE_FLIGHT_TO_LIMSA = 8
SCENE_ULDAH_TICKETS_AGAIN = 9
SCENE_STORM_GUARD = 11
SCENE_LIMSA_ATTENDANT = 12
SCENE_ZANTHAEL = 16
SCENE_ZANTHAEL_2 = 17
SCENE_ADMIRAL = 18
SCENE_STORM_GUARD_AGAIN = 20
SCENE_LIMSA_TICKETS = 24
SCENE_LIMSA_CHECK = 28
SCENE_LIMSA_BOARD = 29
SCENE_FLIGHT_TO_GRIDANIA = 30
SCENE_LIMSA_TICKETS_AGAIN = 31
SCENE_SERPENT_GUARD = 35
SCENE_GRIDANIA_ATTENDANT = 37
SCENE_CONJURER = 42
SCENE_CONJURER_2 = 43
SCENE_AUDIENCE = 44
SCENE_REWARD = 45
SCENE_SERPENT_GUARD_AGAIN = 47

NOTICE_MOMODI = 0
NOTICE_LIMSA = 3
NOTICE_ADMIRAL = 4
NOTICE_GRIDANIA = 7

ULDAH = TERRITORYTYPE0 or 130
LIMSA = TERRITORYTYPE1 or 128
GRIDANIA = TERRITORYTYPE2 or 132
-- Where each move leaves you (retail's ZoneInit pop ranges for the flights, 4102790 and 4102803; the others by the
-- order of the Quest sheet's parameters and the ActorSetPos positions in Bob's Chronofoil capture of 2026-10-18.)
ULDAH_LANDING = POPRANGE0 or 4103351
LIMSA_ARRIVAL = POPRANGE1 or 4102790
LIMSA_BELOW = POPRANGE2 or 4103399
LIMSA_LANDING = POPRANGE3 or 4103402
GRIDANIA_ARRIVAL = POPRANGE4 or 4102803
GRIDANIA_BELOW = POPRANGE5 or 4103382

-- The progress bar before boarding (retail: EventAction 11, about 2 s.)
EVENT_ACTION_BOARD = EVENT_ACTION_WAITING_SHOR or 11
-- A yes/no answered No (its error code: the client's script cancels the scene.)
SCENE_DECLINED = 50
-- How retail plays the gangplanks' check and yes/no scenes: HIDE_HOTBAR | NO_DEFAULT_CAMERA (Bob's Chronofoil capture,
-- 2026-10-18.)
GANGPLANK_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB, as retail plays its cutscenes (Bob's Chronofoil capture of 2026-10-18)

-- Each gangplank's yes/no, after its check scene and the progress bar.
BOARDING_AFTER_CHECK = {
    [SCENE_ULDAH_CHECK] = SCENE_ULDAH_BOARD,
    [SCENE_LIMSA_CHECK] = SCENE_LIMSA_BOARD,
}

-- The yes/nos that only move you about (unless answered No), and where they leave you.
MOVES = {
    [SCENE_ULDAH_TICKETS_AGAIN] = { ULDAH, ULDAH_LANDING },
    [SCENE_LIMSA_ATTENDANT] = { LIMSA, LIMSA_BELOW },
    [SCENE_LIMSA_TICKETS_AGAIN] = { LIMSA, LIMSA_LANDING },
    [SCENE_GRIDANIA_ATTENDANT] = { GRIDANIA, GRIDANIA_BELOW },
}

-- The ticketers' scenes by sequence (client script.)
ELYENORA_SCENES = { [2] = SCENE_ULDAH_TICKETS, [3] = SCENE_ULDAH_TICKETS_AGAIN, [4] = 14, [5] = 22, [6] = 26, [7] = 33,
    [8] = 38, [255] = 49 }
LNOPHLO_SCENES = { [6] = SCENE_LIMSA_TICKETS, [7] = SCENE_LIMSA_TICKETS_AGAIN, [8] = 40, [255] = 51 }

-- After you've flown on, the ticketers' yes/no and its airship cutscene, and where it leaves you (not seen on retail.)
FLIGHTS = {
    [14] = { 15, LIMSA, LIMSA_ARRIVAL },
    [22] = { 23, LIMSA, LIMSA_ARRIVAL },
    [26] = { 27, LIMSA, LIMSA_ARRIVAL },
    [33] = { 34, LIMSA, LIMSA_ARRIVAL },
    [38] = { 39, GRIDANIA, GRIDANIA_ARRIVAL },
    [49] = { 50, GRIDANIA, GRIDANIA_ARRIVAL },
    [40] = { 41, GRIDANIA, GRIDANIA_ARRIVAL },
    [51] = { 52, GRIDANIA, GRIDANIA_ARRIVAL },
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
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_MOMODI, CUTSCENE_FLAGS)
    elseif sequence == 1 and BASE_ID == ACTOR0 then
        play(player, SCENE_RAUBAHN, HIDE_HOTBAR)
    elseif sequence ~= nil and BASE_ID == ACTOR2 and ELYENORA_SCENES[sequence] ~= nil then
        play(player, ELYENORA_SCENES[sequence], HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == EOBJECT1 then
        play(player, SCENE_ULDAH_CHECK, GANGPLANK_FLAGS)
    elseif sequence == 4 and BASE_ID == ACTOR3 then
        play(player, SCENE_STORM_GUARD, HIDE_HOTBAR)
    elseif sequence == 4 and BASE_ID == ACTOR4 then
        play(player, SCENE_LIMSA_ATTENDANT, HIDE_HOTBAR)
    elseif sequence == 5 and BASE_ID == ACTOR5 then
        play(player, SCENE_ZANTHAEL, HIDE_HOTBAR)
    elseif sequence == 5 and BASE_ID == ACTOR3 then
        play(player, SCENE_STORM_GUARD_AGAIN, HIDE_HOTBAR)
    elseif sequence ~= nil and BASE_ID == ACTOR6 and LNOPHLO_SCENES[sequence] ~= nil then
        play(player, LNOPHLO_SCENES[sequence], HIDE_HOTBAR)
    elseif sequence == 7 and BASE_ID == EOBJECT3 then
        play(player, SCENE_LIMSA_CHECK, GANGPLANK_FLAGS)
    elseif sequence == 8 and BASE_ID == ACTOR7 then
        play(player, SCENE_SERPENT_GUARD, HIDE_HOTBAR)
    elseif sequence == 8 and BASE_ID == ACTOR8 then
        play(player, SCENE_GRIDANIA_ATTENDANT, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR9 then
        play(player, SCENE_CONJURER, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR7 then
        play(player, SCENE_SERPENT_GUARD_AGAIN, HIDE_HOTBAR)
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

-- Ends the event and moves the player, to another zone or within this one. Retail moves them within a zone through a
-- loading screen too (PrepareZoning to the same territory, then ActorSetPos; Bob's Chronofoil capture of 2026-10-18.)
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
    elseif scene == SCENE_MOMODI then
        player:event_notice(EVENT_ID, NOTICE_MOMODI)
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_ULDAH_TICKETS then
        if player.scene_error_code ~= SCENE_DECLINED then
            player:quest_sequence(EVENT_ID, 3)
            move(player, ULDAH, ULDAH_LANDING)
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
    elseif scene == SCENE_ULDAH_BOARD then
        if player.scene_error_code ~= SCENE_DECLINED then
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
        setVar(player, 1, true, 0) -- UI8BH: ITEM0 was handed over
        player:event_notice(EVENT_ID, NOTICE_ADMIRAL)
        player:quest_sequence(EVENT_ID, 6)
    elseif scene == SCENE_LIMSA_TICKETS then
        if player.scene_error_code ~= SCENE_DECLINED then
            player:quest_sequence(EVENT_ID, 7)
            move(player, LIMSA, LIMSA_LANDING)
            return
        end
    elseif scene == SCENE_LIMSA_BOARD then
        if player.scene_error_code ~= SCENE_DECLINED then
            play(player, SCENE_FLIGHT_TO_GRIDANIA, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_FLIGHT_TO_GRIDANIA then
        player:quest_sequence(EVENT_ID, 8)
        move(player, GRIDANIA, GRIDANIA_ARRIVAL)
        return
    elseif scene == SCENE_SERPENT_GUARD then
        player:event_notice(EVENT_ID, NOTICE_GRIDANIA)
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_CONJURER then
        if results[1] == 1 then
            play(player, SCENE_CONJURER_2, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_CONJURER_2 then
        play(player, SCENE_AUDIENCE, CUTSCENE_FLAGS)
        return
    elseif scene == SCENE_AUDIENCE then
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
