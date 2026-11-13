-- Quest: Poisoned Hearts
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Bob's Chronofoil capture da424662 (2026-11-12) (fixture tools/questkit/fixtures/1058.json):
-- Sequence 0:
--   ACTOR0 (Jenlyns): scene 0 (HIDE_HOTBAR, a yield), 1 (HIDE_HOTBAR) -> accepted, sequence 1
-- Sequence 1:
--   walking into EVENTRANGE1: scene 5 (NO_DEFAULT_CAMERA | HIDE_HOTBAR)
--   defeating 1924, vars 00 10 00 00 00 00
--   defeating 1923, vars 00 20 00 00 00 00
--   EOBJECT1 (Sultansworn coffer): scene 6 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_SEARCH), 7 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), vars 10 30 00 00 00 40, notices [[0, [1, 4, 48340]]]
--   walking into EVENTRANGE0: scene 2 (NO_DEFAULT_CAMERA | HIDE_HOTBAR)
--   defeating 1923, vars 11 30 00 00 00 40
--   defeating 1924, vars 12 30 00 00 00 40
--   EOBJECT0 (Sultansworn coffer): scene 3 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_SEARCH), 4 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), vars 23 30 00 00 00 c0, notices [[0, [2, 4, 45192]]]
--   walking into EVENTRANGE3: scene 11 (NO_DEFAULT_CAMERA | HIDE_HOTBAR)
--   defeating 1923, vars 23 30 10 00 00 c0
--   defeating 1924, vars 23 30 20 00 00 c0
--   EOBJECT3 (Sultansworn coffer): scene 12 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_SEARCH), 13 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), vars 33 30 30 00 00 d0, notices [[0, [3, 4, 40617]]]
--   walking into EVENTRANGE2: scene 8 (NO_DEFAULT_CAMERA | HIDE_HOTBAR)
--   defeating 1923, vars 33 31 30 00 00 d0
--   defeating 1924, vars 33 32 30 00 00 d0
--   EOBJECT2 (Sultansworn coffer): scene 9 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_SEARCH), 10 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 255, vars 00 00 00 00 00 00, notices [[0, [4, 4, 46845]]]
-- Sequence 255:
--   ACTOR0 (Jenlyns): scene 14 (HIDE_HOTBAR, a yield) -> completed
-- Each search (scenes 7, 4, 13 and 10) gives its armour piece (RITEM1, RITEM0, RITEM3, RITEM2: `obtain_item`), then the
-- quest's LOGMESSAGE0 and the notice, as Bob's Chronofoil capture da424662 did.
-- The 4 talks of sequence 1 work in any order: UI8AH counts them and each sets its BitFlag8 (retail's), the last moves the quest on.

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
    elseif sequence == 1 and BASE_ID == EOBJECT1 and not hasFlag(player, 2) then
        step = 5
        play(player, 6, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == EOBJECT0 and not hasFlag(player, 1) then
        step = 9
        play(player, 3, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == EOBJECT3 and not hasFlag(player, 4) then
        step = 13
        play(player, 12, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == EOBJECT2 and not hasFlag(player, 3) then
        step = 17
        play(player, 9, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        step = 18
        play(player, 14, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEnterTrigger(player, arg)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == 1 and arg == EVENTRANGE1 then
        step = 2
        play(player, 5, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 1 and arg == EVENTRANGE0 then
        step = 6
        play(player, 2, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 1 and arg == EVENTRANGE3 then
        step = 10
        play(player, 11, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 1 and arg == EVENTRANGE2 then
        step = 14
        play(player, 8, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onBNpcKill(base_id, layout_id, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    if sequence == 1 and layout_id == ENEMY3 then
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8B
    elseif sequence == 1 and layout_id == ENEMY2 then
        player:set_quest_var(EVENT_ID, 1, 0x20) -- UI8B
    elseif sequence == 1 and layout_id == ENEMY0 then
        player:set_quest_var(EVENT_ID, 0, 0x11) -- UI8A
    elseif sequence == 1 and layout_id == ENEMY1 then
        player:set_quest_var(EVENT_ID, 0, 0x12) -- UI8A
    elseif sequence == 1 and layout_id == ENEMY6 then
        player:set_quest_var(EVENT_ID, 2, 0x10) -- UI8C
    elseif sequence == 1 and layout_id == ENEMY7 then
        player:set_quest_var(EVENT_ID, 2, 0x20) -- UI8C
    elseif sequence == 1 and layout_id == ENEMY4 then
        player:set_quest_var(EVENT_ID, 1, 0x31) -- UI8B
    elseif sequence == 1 and layout_id == ENEMY5 then
        player:set_quest_var(EVENT_ID, 1, 0x32) -- UI8B
    end
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
        play(player, 1, HIDE_HOTBAR)
        return
    elseif step == 1 and scene == 1 then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif step == 2 and scene == 5 then
        player:spawn_quest_bnpc(ENEMY3)
        player:spawn_quest_bnpc(ENEMY2)
    elseif step == 5 and scene == 6 then
        after_bar = { scene = 7, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif step == 5 and scene == 7 then
        local count = getVar(player, 0, true) + 1
        player:obtain_item(RITEM1, 1)
        player:event_log_message(EVENT_ID, LOGMESSAGE0, {})
        player:event_notice(EVENT_ID, 0, { 1, 4, 48340 })
        if count >= 4 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x00)
            player:set_quest_var(EVENT_ID, 2, 0x00)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 255)
        else
            setVar(player, 0, true, count)
            setVar(player, 1, true, 3) -- UI8BH
            setFlag(player, 2)
        end
    elseif step == 6 and scene == 2 then
        player:spawn_quest_bnpc(ENEMY1)
        player:spawn_quest_bnpc(ENEMY0)
    elseif step == 9 and scene == 3 then
        after_bar = { scene = 4, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif step == 9 and scene == 4 then
        local count = getVar(player, 0, true) + 1
        player:obtain_item(RITEM0, 1)
        player:event_log_message(EVENT_ID, LOGMESSAGE0, {})
        player:event_notice(EVENT_ID, 0, { 2, 4, 45192 })
        if count >= 4 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x00)
            player:set_quest_var(EVENT_ID, 2, 0x00)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 255)
        else
            setVar(player, 0, true, count)
            setVar(player, 0, false, 3) -- UI8AL
            setFlag(player, 1)
        end
    elseif step == 10 and scene == 11 then
        player:spawn_quest_bnpc(ENEMY7)
        player:spawn_quest_bnpc(ENEMY6)
    elseif step == 13 and scene == 12 then
        after_bar = { scene = 13, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif step == 13 and scene == 13 then
        local count = getVar(player, 0, true) + 1
        player:obtain_item(RITEM3, 1)
        player:event_log_message(EVENT_ID, LOGMESSAGE0, {})
        player:event_notice(EVENT_ID, 0, { 3, 4, 40617 })
        if count >= 4 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x00)
            player:set_quest_var(EVENT_ID, 2, 0x00)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 255)
        else
            setVar(player, 0, true, count)
            setVar(player, 2, true, 3) -- UI8CH
            setFlag(player, 4)
        end
    elseif step == 14 and scene == 8 then
        player:spawn_quest_bnpc(ENEMY5)
        player:spawn_quest_bnpc(ENEMY4)
    elseif step == 17 and scene == 9 then
        after_bar = { scene = 10, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif step == 17 and scene == 10 then
        local count = getVar(player, 0, true) + 1
        player:obtain_item(RITEM2, 1)
        player:event_log_message(EVENT_ID, LOGMESSAGE0, {})
        player:event_notice(EVENT_ID, 0, { 4, 4, 46845 })
        if count >= 4 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x00)
            player:set_quest_var(EVENT_ID, 2, 0x00)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 255)
        else
            setVar(player, 0, true, count)
            setFlag(player, 3)  -- not seen on retail (its step was the last): the next free one
        end
    elseif step == 18 and scene == 14 then
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
