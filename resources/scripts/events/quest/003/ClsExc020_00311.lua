-- Quest: Way of the Marauder
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Bob's Chronofoil capture 0cafbf61 (2026-11-17): a new Marauder (fixture tools/questkit/fixtures/311.json):
-- Sequence 0:
--   ACTOR0 (Blauthota): scene 0 (HIDE_HOTBAR, a yield), 1 (HIDE_HOTBAR) -> accepted, sequence 1
-- Sequence 1:
--   ACTOR1 (Wyrnzoen): scene 3 (SET_BASE | HIDE_HOTBAR | DISABLE_SKIP, a yield) -> sequence 2, notices [[0, []]]
-- Sequence 2:
--   defeating ENEMY0, ENEMY1 and ENEMY2, three of each in any order (`KINDS`): notices 1, 2 and 3 with [count, 3];
--   retail's order was ENEMY0 (vars 01, 02, 13 00), ENEMY2 (13 01, 13 02, 23 03), ENEMY1 (23 13, 23 23), the last
--   -> sequence 255, vars 00 00
-- Sequence 255:
--   ACTOR1 (Wyrnzoen): scene 4 (HIDE_HOTBAR, a yield) -> completed

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
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        step = 2
        play(player, 3, SET_BASE | HIDE_HOTBAR | DISABLE_SKIP)
        return
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        step = 12
        play(player, 4, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

-- Sequence 2's three kinds of enemy, three of each, in any order: each kind has its own count (the client's
-- IsTodoChecked: ENEMY0 in UI8AL, ENEMY1 in UI8BH, ENEMY2 in UI8BL) and its own notice, and UI8AH counts the kinds
-- that are done; the third done clears the variables and moves the quest on (retail: 13 00, 23 03, then 00 00 and
-- sequence 255.)
KILLS_NEEDED = 3
KINDS = {
    { enemy = ENEMY0, byte = 0, high = false, notice = 1 },
    { enemy = ENEMY1, byte = 1, high = true, notice = 2 },
    { enemy = ENEMY2, byte = 1, high = false, notice = 3 },
}

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 2 then
        return
    end
    for _, kind in ipairs(KINDS) do
        if base_id == kind.enemy then
            local vars = { player:get_quest_var(EVENT_ID, 0), player:get_quest_var(EVENT_ID, 1) }
            local value = vars[kind.byte + 1]
            local count = kind.high and (value >> 4) or (value & 0x0F)
            if count >= KILLS_NEEDED then
                return
            end
            count = count + 1
            vars[kind.byte + 1] = kind.high and ((value & 0x0F) | (count << 4)) or ((value & 0xF0) | count)
            player:event_notice(EVENT_ID, kind.notice, { count, KILLS_NEEDED })
            local done = vars[1] >> 4
            if count == KILLS_NEEDED then
                done = done + 1
                vars[1] = (vars[1] & 0x0F) | (done << 4)
            end
            if done == #KINDS then
                vars = { 0, 0 }
            end
            for byte = 0, 1 do
                if vars[byte + 1] ~= player:get_quest_var(EVENT_ID, byte) then
                    player:set_quest_var(EVENT_ID, byte, vars[byte + 1])
                end
            end
            if done == #KINDS then
                player:quest_sequence(EVENT_ID, 255)
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
    elseif step == 2 and scene == 3 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif step == 12 and scene == 4 then
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
