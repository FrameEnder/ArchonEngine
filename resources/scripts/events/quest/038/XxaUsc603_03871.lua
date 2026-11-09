-- Quest: Dressed for Conquest
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Connor's Chronofoil capture e834e436 (2026-11-07/09) (fixture tools/questkit/fixtures/3871.json):
-- Sequence 0:
--   ACTOR0 (Sark Malark): scene 0 (HIDE_HOTBAR, a yield), 1 (HIDE_HOTBAR) -> accepted, sequence 1
-- Sequence 1:
--   defeating ENEMY3, vars 00 00 10 00 00 00, notices [[1, [1, 3, 40128]]]
--   defeating ENEMY2, vars 00 01 10 00 00 00, notices [[0, [1, 3, 42345]]]
--   defeating ENEMY4, vars 00 01 20 00 00 00, notices [[1, [2, 3, 40128]]]
--   defeating ENEMY0, vars 00 02 20 00 00 00, notices [[0, [2, 3, 42345]]]
--   defeating ENEMY1, vars 10 03 20 00 00 00, notices [[0, [3, 3, 42345]]]
--   defeating ENEMY3 -> sequence 2, vars 00 33 00 00 00 00, notices [[1, [3, 3, 40128]]]
--   (the kills are two hauls, any order, every kill counting: HAULS below, from the client's GetEventItems and
--   IsTodoChecked; the lines above are retail's order)
-- Sequence 2:
--   ACTOR0 (Sark Malark): scene 2 (HIDE_HOTBAR, a yield), 3 (HIDE_HOTBAR) -> sequence 3, notices [[2, []]]
-- Sequence 3:
--   ACTOR1 (Eginolf): scene 4 (HIDE_HOTBAR, a yield), 5 (SET_BASE | INVIS_ENPC | INVIS_AETHERYTE | HIDE_HOTBAR) -> sequence 255, notices [[3, [42345]]]
-- Sequence 255:
--   ACTOR0 (Sark Malark): scene 6 (HIDE_HOTBAR, a yield), 7 (HIDE_HOTBAR, a yield) -> completed

-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855, 960.)
REWARD_CLOSED = 0x11

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- Which step of the quest this event is (the sequence notes above), the scene after a progress bar, and the reward
-- the player picked (the scenes after it show it.)
step = nil
after_bar = nil
chosen = 0

function play(player, scene, flags, params)
    current_scene = scene
    player:play_scene(scene, flags, params or {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == nil and BASE_ID == ACTOR0 then
        step = 1
        play(player, 0, HIDE_HOTBAR)
        return
    elseif sequence == 2 and BASE_ID == ACTOR0 then
        step = 8
        play(player, 2, HIDE_HOTBAR)
        return
    elseif sequence == 3 and BASE_ID == ACTOR1 then
        step = 9
        play(player, 4, HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        step = 10
        play(player, 6, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

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

-- The two hauls of sequence 1 (the client's GetEventItems and IsTodoChecked): ITEM0 from ENEMY0-2, counted in UI8BL,
-- and ITEM1 from ENEMY3-4, counted in UI8CH, three of each, every kill one (retail's run: all eight kills counted).
-- Each sends its notice with the item's icon (0 [n, 3, 42345] and 1 [n, 3, 40128]); retail set UI8AH when the first
-- haul was full, and moved both counts to UI8BH and UI8BL with sequence 2 when the second was.
HAULS = {
    { enemies = { [ENEMY0] = true, [ENEMY1] = true, [ENEMY2] = true }, byte = 1, high = false, notice = 0,
      icon = 42345 },
    { enemies = { [ENEMY3] = true, [ENEMY4] = true }, byte = 2, high = true, notice = 1, icon = 40128 },
}
NEEDED = 3

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 then
        return
    end
    for index, haul in ipairs(HAULS) do
        local count = getVar(player, haul.byte, haul.high)
        if haul.enemies[base_id] and count < NEEDED then
            count = count + 1
            player:event_notice(EVENT_ID, haul.notice, { count, NEEDED, haul.icon })
            local other = HAULS[3 - index]
            if count >= NEEDED and getVar(player, other.byte, other.high) >= NEEDED then
                for byte = 0, 5 do
                    player:set_quest_var(EVENT_ID, byte, 0)
                end
                player:set_quest_var(EVENT_ID, 1, 0x33) -- ITEM0 in UI8BH, ITEM1 in UI8BL (GetEventItems at 2)
                player:quest_sequence(EVENT_ID, 2)
                return
            end
            setVar(player, haul.byte, haul.high, count)
            if count >= NEEDED and index == 1 then
                setVar(player, 0, true, 1) -- UI8AH, as retail when ITEM0's haul was full
            end
            return
        end
    end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if step == 1 and scene == 0 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        play(player, 1, HIDE_HOTBAR)
        return
    elseif step == 1 and scene == 1 then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif step == 8 and scene == 2 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        play(player, 3, HIDE_HOTBAR)
        return
    elseif step == 8 and scene == 3 then
        player:event_notice(EVENT_ID, 2, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif step == 9 and scene == 4 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        play(player, 5, SET_BASE | INVIS_ENPC | INVIS_AETHERYTE | HIDE_HOTBAR)
        return
    elseif step == 9 and scene == 5 then
        player:event_notice(EVENT_ID, 3, { 42345 })
        player:quest_sequence(EVENT_ID, 255)
    elseif step == 10 and scene == 6 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        play(player, 7, HIDE_HOTBAR)
        return
    elseif step == 10 and scene == 7 then
        if results[1] ~= 1 then
            -- The reward window closed: the quest stays open.
            if player:get_quest_var(EVENT_ID, 0) ~= REWARD_CLOSED then
                player:set_quest_var(EVENT_ID, 0, REWARD_CLOSED)
            end
            player:finish_event()
            return
        end
        chosen = results[2] or 0
        player:finish_event()
        player:finish_quest(EVENT_ID, chosen)
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
