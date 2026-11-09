-- Quest: Moving On
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Connor's Chronofoil capture af05916c (2026-11-07/09) (fixture tools/questkit/fixtures/3874.json):
-- Sequence 0:
--   ACTOR0 (Minfilia): scene 0 (HIDE_HOTBAR, a yield), 1 (HIDE_HOTBAR) -> accepted, sequence 1
-- Sequence 1:
--   ACTOR1 (Alphinaud): scene 2 (SET_BASE | HIDE_HOTBAR) -> sequence 2, notices [[0, []]]
-- Sequence 2:
--   ACTOR2 (Gegeruju): scene 3 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 3, notices [[1, []]]
-- Sequence 3:
--   ACTOR5 (Byrglaent): scene 6 (HIDE_HOTBAR), vars 01 00 00 00 00 40, notices [[2, [1, 2]]]
--   ACTOR4 (Junghbhar): scene 5 (HIDE_HOTBAR) -> sequence 4, vars 00 00 00 00 00 00, notices [[2, [2, 2]]]
-- Sequence 4:
--   ACTOR6 (Shamani Lohmani): scene 7 (HIDE_HOTBAR) -> sequence 5, notices [[3, []]]
-- Sequence 5:
--   walking into EVENTRANGE0: scene 8 (NO_DEFAULT_CAMERA | HIDE_HOTBAR)
--   defeating 320 -> sequence 6, notices [[4, []]]
-- Sequence 6:
--   ACTOR7 (F'lhaminn): scene 13 (SET_BASE | HIDE_HOTBAR | INVINCIBLE) -> sequence 255, notices [[5, []]]
-- Sequence 255:
--   ACTOR8 (Alphinaud): scene 14 (HIDE_HOTBAR, a yield) -> completed
-- The 2 talks of sequence 3 work in any order: UI8AL counts them and each sets its BitFlag8 (retail's), the last moves the quest on.

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
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        step = 2
        play(player, 2, SET_BASE | HIDE_HOTBAR)
        return
    elseif sequence == 2 and BASE_ID == ACTOR2 then
        step = 3
        play(player, 3, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 3 and BASE_ID == ACTOR5 and not hasFlag(player, 2) then
        step = 4
        play(player, 6, HIDE_HOTBAR)
        return
    elseif sequence == 3 and BASE_ID == ACTOR4 and not hasFlag(player, 1) then
        step = 5
        play(player, 5, HIDE_HOTBAR)
        return
    elseif sequence == 4 and BASE_ID == ACTOR6 then
        step = 6
        play(player, 7, HIDE_HOTBAR)
        return
    elseif sequence == 6 and BASE_ID == ACTOR7 then
        step = 9
        play(player, 13, SET_BASE | HIDE_HOTBAR | INVINCIBLE)
        return
    elseif sequence == 255 and BASE_ID == ACTOR8 then
        step = 10
        play(player, 14, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEnterTrigger(player, arg)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == 5 and arg == EVENTRANGE0 then
        step = 7
        play(player, 8, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onBNpcKill(base_id, layout_id, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    if sequence == 5 and layout_id == ENEMY0 then
        player:event_notice(EVENT_ID, 4, {})
        player:quest_sequence(EVENT_ID, 6)
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
    elseif step == 2 and scene == 2 then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif step == 3 and scene == 3 then
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif step == 4 and scene == 6 then
        local count = getVar(player, 0, false) + 1
        player:event_notice(EVENT_ID, 2, { count, 2 })
        if count >= 2 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x00)
            player:set_quest_var(EVENT_ID, 2, 0x00)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 4)
        else
            setVar(player, 0, false, count)
            setFlag(player, 2)
        end
    elseif step == 5 and scene == 5 then
        local count = getVar(player, 0, false) + 1
        player:event_notice(EVENT_ID, 2, { count, 2 })
        if count >= 2 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x00)
            player:set_quest_var(EVENT_ID, 2, 0x00)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 4)
        else
            setVar(player, 0, false, count)
            setFlag(player, 1)  -- not seen on retail (its step was the last): the next free one
        end
    elseif step == 6 and scene == 7 then
        player:event_notice(EVENT_ID, 3, {})
        player:quest_sequence(EVENT_ID, 5)
    elseif step == 7 and scene == 8 then
        player:spawn_quest_bnpc(ENEMY0)
    elseif step == 9 and scene == 13 then
        player:event_notice(EVENT_ID, 5, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif step == 10 and scene == 14 then
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
