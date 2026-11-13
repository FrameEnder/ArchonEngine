-- Quest: Only You Can Prevent Forest Ire
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Bob's Chronofoil capture 898dd0b9 (2026-11-12) (fixture tools/questkit/fixtures/1080.json):
-- Sequence 0:
--   ACTOR0 (Raya-O-Senna): scene 0 (HIDE_HOTBAR, a yield), 1 (HIDE_HOTBAR) -> accepted, sequence 1
-- Sequence 1:
--   walking into EVENTRANGE0: scene 2 (NO_DEFAULT_CAMERA | HIDE_HOTBAR)
--   defeating 1937, vars 01 00 00 00 00 00
--   defeating 1937, vars 02 00 00 00 00 00
--   defeating 1937, vars 03 00 00 00 00 00
--   EOBJECT0 (destination): scene 3 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_QUIET_UNDER_LONG), 4 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 255, vars 00 00 00 00 00 00, notices [[0, []]]
-- Sequence 255:
--   ACTOR0 (Raya-O-Senna): scene 7 (HIDE_HOTBAR, a yield), 8 (SET_BASE | HIDE_HOTBAR | DISABLE_SKIP, a yield) -> completed
-- The 3 enemies of sequence 1 count in any order (UI8AL).

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

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == nil and BASE_ID == ACTOR0 then
        step = 1
        play(player, 0, HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == EOBJECT0 then
        step = 6
        play(player, 3, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        step = 7
        play(player, 7, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEnterTrigger(player, arg)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == 1 and arg == EVENTRANGE0 then
        step = 2
        play(player, 2, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

-- The enemies of sequence 1, in any order: UI8AL counts them.
KILLS_1 = { [ENEMY0] = true, [ENEMY1] = true, [ENEMY2] = true }

function onBNpcKill(base_id, layout_id, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    if sequence == 1 and KILLS_1[layout_id] then
        local count = getVar(player, 0, false) + 1
        if count <= 3 then
            setVar(player, 0, false, count)
        end
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
    elseif step == 2 and scene == 2 then
        player:spawn_quest_bnpc(ENEMY2)
        player:spawn_quest_bnpc(ENEMY1)
        player:spawn_quest_bnpc(ENEMY0)
    elseif step == 6 and scene == 3 then
        after_bar = { scene = 4, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_QUIET_UNDER_LONG)
        return
    elseif step == 6 and scene == 4 then
        player:event_notice(EVENT_ID, 0, {})
        player:event_log_message(EVENT_ID, LOGMESSAGE0, {})
        player:set_quest_var(EVENT_ID, 0, 0x00) -- UI8A
        player:quest_sequence(EVENT_ID, 255)
    elseif step == 7 and scene == 7 then
        if results[1] ~= 1 then
            -- The reward window closed: the quest stays open.
            if player:get_quest_var(EVENT_ID, 0) ~= REWARD_CLOSED then
                player:set_quest_var(EVENT_ID, 0, REWARD_CLOSED)
            end
            player:finish_event()
            return
        end
        chosen = results[2] or 0
        play(player, 8, SET_BASE | HIDE_HOTBAR | DISABLE_SKIP, { chosen })
        return
    elseif step == 7 and scene == 8 then
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
