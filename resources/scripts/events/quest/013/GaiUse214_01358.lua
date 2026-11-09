-- Quest: Scouts in Distress
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Connor's Chronofoil capture af05916c (2026-11-07/09) (fixture tools/questkit/fixtures/1358.json):
-- Sequence 0:
--   ACTOR0 (Falkbryda): scene 0 (HIDE_HOTBAR, a yield), 1 (HIDE_HOTBAR) -> accepted, sequence 1, vars 00 10 00 00 00 00
-- Sequence 1:
--   key item 2001214 on ACTOR1 (storm private): scene 3 (NO_DEFAULT_CAMERA | HIDE_HOTBAR | INVINCIBLE) -> sequence 2, vars 00 00 00 00 00 00, notices [[0, []]]
-- Sequence 2:
--   walking into EVENTRANGE0: scene 5 (NO_DEFAULT_CAMERA | HIDE_HOTBAR)
--   defeating 2861, vars 01 00 00 00 00 00
--   defeating 2862, vars 02 00 00 00 00 00
--   defeating 2863 -> sequence 3, vars 00 00 00 00 00 00, notices [[1, []]]
-- Sequence 3:
--   ACTOR2 (soldier's corpse): scene 12 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_PROCESS_MIDDLE), 13 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), vars 01 10 00 00 00 80, notices [[2, [1, 2, 26024]]]
--   ACTOR3 (soldier's corpse): scene 14 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_PROCESS_LONG), 15 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 255, vars 00 20 00 00 00 00, notices [[2, [2, 2, 26024]]]
-- Sequence 255:
--   ACTOR0 (Falkbryda): scene 16 (HIDE_HOTBAR, a yield), 17 (HIDE_HOTBAR, a yield) -> completed
-- The 2 talks of sequence 3 work in any order: UI8AL counts them and each sets its BitFlag8 (retail's), the last moves the quest on.
-- The 3 enemies of sequence 2 count in any order (UI8AL).

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
    elseif sequence == 3 and BASE_ID == ACTOR2 and not hasFlag(player, 1) then
        step = 7
        play(player, 12, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 3 and BASE_ID == ACTOR3 and not hasFlag(player, 2) then
        step = 8
        play(player, 14, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        step = 9
        play(player, 16, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEventItem(item, target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == 1 and item == ITEM0 and BASE_ID == ACTOR1 then
        step = 2
        play(player, 3, NO_DEFAULT_CAMERA | HIDE_HOTBAR | INVINCIBLE)
        return
    end
    player:finish_event()
end

function onEnterTrigger(player, arg)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == 2 and arg == EVENTRANGE0 then
        step = 3
        play(player, 5, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

-- The enemies of sequence 2, in any order: UI8AL counts them.
KILLS_2 = { [ENEMY0] = true, [ENEMY1] = true, [ENEMY2] = true }

function onBNpcKill(base_id, layout_id, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    if sequence == 2 and KILLS_2[layout_id] then
        local count = getVar(player, 0, false) + 1
        if count >= 3 then
            player:event_notice(EVENT_ID, 1, {})
            player:set_quest_var(EVENT_ID, 0, 0x00) -- UI8A
            player:quest_sequence(EVENT_ID, 3)
        else
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
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8B
        player:quest_sequence(EVENT_ID, 1)
    elseif step == 2 and scene == 3 then
        player:spawn_quest_bnpc(ENEMY2)
        player:spawn_quest_bnpc(ENEMY1)
        player:spawn_quest_bnpc(ENEMY0)
        player:event_notice(EVENT_ID, 0, {})
        player:set_quest_var(EVENT_ID, 1, 0x00) -- UI8B
        player:quest_sequence(EVENT_ID, 2)
    elseif step == 3 and scene == 5 then
        player:spawn_quest_bnpc(ENEMY2)
        player:spawn_quest_bnpc(ENEMY1)
        player:spawn_quest_bnpc(ENEMY0)
    elseif step == 7 and scene == 12 then
        after_bar = { scene = 13, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_PROCESS_MIDDLE)
        return
    elseif step == 7 and scene == 13 then
        local count = getVar(player, 0, false) + 1
        player:event_notice(EVENT_ID, 2, { count, 2, 26024 })
        if count >= 2 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x20)
            player:set_quest_var(EVENT_ID, 2, 0x00)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 255)
        else
            setVar(player, 0, false, count)
            setVar(player, 1, true, 1) -- UI8BH
            setFlag(player, 1)
        end
    elseif step == 8 and scene == 14 then
        after_bar = { scene = 15, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_PROCESS_LONG)
        return
    elseif step == 8 and scene == 15 then
        local count = getVar(player, 0, false) + 1
        player:event_notice(EVENT_ID, 2, { count, 2, 26024 })
        if count >= 2 then
            player:set_quest_var(EVENT_ID, 0, 0x00)
            player:set_quest_var(EVENT_ID, 1, 0x20)
            player:set_quest_var(EVENT_ID, 2, 0x00)
            player:set_quest_var(EVENT_ID, 3, 0x00)
            player:set_quest_var(EVENT_ID, 4, 0x00)
            player:set_quest_var(EVENT_ID, 5, 0x00)
            player:quest_sequence(EVENT_ID, 255)
        else
            setVar(player, 0, false, count)
            setFlag(player, 2)  -- not seen on retail (its step was the last): the next free one
        end
    elseif step == 9 and scene == 16 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        play(player, 17, HIDE_HOTBAR)
        return
    elseif step == 9 and scene == 17 then
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
