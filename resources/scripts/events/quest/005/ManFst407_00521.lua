-- Quest: Escape from Castrum Centri
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Connor's Chronofoil capture e834e436 (2026-11-07/09) (fixture tools/questkit/fixtures/521.json):
-- Sequence 0:
--   ACTOR0 (Cid): scene 0 (HIDE_HOTBAR, a yield), 1 (SET_BASE | HIDE_HOTBAR) -> accepted, sequence 1
-- Sequence 1:
--   ACTOR5 (Cid): scene 7 (HIDE_HOTBAR), 8 (SET_BASE | HIDE_HOTBAR | INVINCIBLE) -> sequence 2, notices [[0, []]], zone 335
-- Sequence 2:
--   emote 59 on ACTOR9 (imperial courier): scene 18 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), vars 11 00 00 00 00 08, notices [[1, [1, 3]]]
--   emote 59 on ACTOR7 (imperial decurion): scene 12 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), vars 21 01 00 00 00 88, notices [[1, [2, 3]]]
--   emote 59 on ACTOR10 (imperial guard): scene 21 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 3, vars 00 00 00 00 00 00, notices [[1, [3, 3]]]
-- Sequence 3:
--   emote 59 on ACTOR11 (imperial centurion): scene 24 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 4, vars 00 10 00 00 00 00, notices [[2, [1, 1, 26014]]]
-- Sequence 4:
--   ACTOR12 (Biggs): scene 29 (HIDE_HOTBAR, a yield), 30 (0xF8402EF3) -> sequence 5, vars 00 00 00 00 00 00, notices [[3, [1, 1]]], zone 335
-- Sequence 5:
--   EOBJECT0 (steel door): scene 34 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_TOUCH_MIDDLE), 35 (NO_DEFAULT_CAMERA | HIDE_HOTBAR, a yield), into the quest battle, zone 305
-- Sequence 6:
--   entering the zone: scene 40 (SET_BASE | HIDE_HOTBAR) -> sequence 255, notices [[5, []]]
-- Sequence 255:
--   ACTOR15 (Minfilia): scene 41 (HIDE_HOTBAR, a yield) -> completed
-- The 3 emotes of sequence 2 work in any order: UI8AH counts them and each sets its BitFlag8 (retail's), the last moves the quest on.

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

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == nil and BASE_ID == ACTOR0 then
        step = 1
        play(player, 0, HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == ACTOR5 then
        step = 2
        play(player, 7, HIDE_HOTBAR)
        return
    elseif sequence == 4 and BASE_ID == ACTOR12 then
        step = 7
        play(player, 29, HIDE_HOTBAR)
        return
    elseif sequence == 5 and BASE_ID == EOBJECT0 then
        step = 8
        play(player, 34, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR15 then
        step = 10
        play(player, 41, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEmote(emote, target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == 2 and emote == 59 and BASE_ID == ACTOR9 and not hasFlag(player, 5) then
        step = 3
        play(player, 18, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 2 and emote == 59 and BASE_ID == ACTOR7 and not hasFlag(player, 1) then
        step = 4
        play(player, 12, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 2 and emote == 59 and BASE_ID == ACTOR10 and not hasFlag(player, 2) then
        step = 5
        play(player, 21, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 3 and emote == 59 and BASE_ID == ACTOR11 then
        step = 6
        play(player, 24, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEnterTerritory(player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == 6 then
        step = 9
        play(player, 40, SET_BASE | HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEventActionCast(player)
    if after_bar ~= nil then
        local scene = after_bar
        after_bar = nil
        play(player, scene.scene, scene.flags)
        return
    end
    player:finish_event()
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
        play(player, 1, SET_BASE | HIDE_HOTBAR)
        return
    elseif step == 1 and scene == 1 then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif step == 2 and scene == 7 then
        play(player, 8, SET_BASE | HIDE_HOTBAR | INVINCIBLE)
        return
    elseif step == 2 and scene == 8 then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
        player:finish_event()
        player:change_territory_pop_range(TERRITORYTYPE1, POPRANGE1)
        return
    elseif step == 3 and scene == 18 then
        local count = getVar(player, 0, true) + 1
        player:event_notice(EVENT_ID, 1, { count, 3 })
        if count >= 3 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x00)
            player:set_quest_var(EVENT_ID, 2, 0x00)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 3)
        else
            setVar(player, 0, true, count)
            setVar(player, 0, false, 1) -- UI8AL
            setFlag(player, 5)
        end
    elseif step == 4 and scene == 12 then
        local count = getVar(player, 0, true) + 1
        player:event_notice(EVENT_ID, 1, { count, 3 })
        if count >= 3 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x00)
            player:set_quest_var(EVENT_ID, 2, 0x00)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 3)
        else
            setVar(player, 0, true, count)
            setVar(player, 1, false, 1) -- UI8BL
            setFlag(player, 1)
        end
    elseif step == 5 and scene == 21 then
        local count = getVar(player, 0, true) + 1
        player:event_notice(EVENT_ID, 1, { count, 3 })
        if count >= 3 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x00)
            player:set_quest_var(EVENT_ID, 2, 0x00)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 3)
        else
            setVar(player, 0, true, count)
            setFlag(player, 2)  -- not seen on retail (its step was the last): the next free one
        end
    elseif step == 6 and scene == 24 then
        player:event_notice(EVENT_ID, 2, { 1, 1, 26014 })
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8B
        player:quest_sequence(EVENT_ID, 4)
    elseif step == 7 and scene == 29 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        play(player, 30, 0xF8402EF3)
        return
    elseif step == 7 and scene == 30 then
        player:event_notice(EVENT_ID, 3, { 1, 1 })
        player:set_quest_var(EVENT_ID, 1, 0x00) -- UI8B
        player:quest_sequence(EVENT_ID, 5)
        player:finish_event()
        -- Where retail put the player (ActorSetPos; no pop range of the quest's.)
        player:change_territory(TERRITORYTYPE1, { x = -494.72, y = -3.22, z = -273.3 }, 1.57)
        return
    elseif step == 8 and scene == 34 then
        after_bar = { scene = 35, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_TOUCH_MIDDLE)
        return
    elseif step == 8 and scene == 35 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        player:finish_event()
        player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE2)
        return
    elseif step == 9 and scene == 40 then
        player:event_notice(EVENT_ID, 5, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif step == 10 and scene == 41 then
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
