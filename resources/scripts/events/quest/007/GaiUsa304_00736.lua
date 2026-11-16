-- Quest: Buried Truth
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and scenes and effects from Bob's Chronofoil capture of 2026-11-16 (fixture tools/questkit/fixtures/736.json):
-- Sequence 0:
--   ACTOR0 (Tanga Tonga): scene 0 (HIDE_HOTBAR, a yield), 1 (HIDE_HOTBAR) -> accepted, sequence 1
-- Sequence 1:
--   EOBJECT1 (round fossil): scene 4 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_SEARCH), 5 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), vars 11 00 10 00 00 40, notices [[0, [22256]]]
--   EOBJECT3 (oblong fossil): scene 8 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_SEARCH), 9 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), vars 11 11 10 00 00 50, notices [[1, [1, 2, 22202]]]
--   EOBJECT4 (oblong fossil): scene 10 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), the bar (EVENT_ACTION_SEARCH), 11 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 255, vars 00 21 00 00 00 00, notices [[1, [2, 2, 22202]]]
-- Sequence 255:
--   ACTOR0 (Tanga Tonga): scene 14 (HIDE_HOTBAR, a yield), 15 (HIDE_HOTBAR, a yield) -> completed
--
-- By hand, from the client's script (IsAcceptEvent, GetEventItems, GetTodoArgs): any of the six fossils, in any
-- order. EOBJECT0 and 1 are round ones (one wanted, counted in UI8AL; ITEM0, shown from UI8CH), EOBJECT2 to 5 oblong
-- (two wanted, counted in UI8BH; ITEM1, shown from UI8BL); the client lets an object be searched while its kind is
-- still wanted and its bit (BitFlag8 its number + 1, UI8F from 0x80 down) isn't set. Its scenes are 2 and 3 plus
-- twice its number (retail's 4/5, 8/9 and 10/11 for EOBJECT1, 3 and 4). The variables after each are retail's
-- (UI8A 0x11 and UI8C 0x10 with the round one; UI8B 0x11 with the first oblong one; at sequence 255 UI8B 0x21, the
-- items' counts there, and the rest zero). Not seen on retail: EOBJECT0, 2 and 5, the round one found after an
-- oblong one, and UI8B with two oblong ones while the round one is still wanted (0x22 here).

-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855, 960.)
REWARD_CLOSED = 0x11

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- Which step of the quest this event is (the sequence notes above), the scene after a progress bar, and the reward
-- the player picked (the scenes after it show it.)
step = nil
after_bar = nil
chosen = 0
-- The fossil being searched (0 to 5.)
fossil = nil

ROUND_WANTED = 1
OBLONG_WANTED = 2
-- The notices' last numbers, as retail sent them: the round fossil's and the oblong ones'.
ROUND_NOTICE = 22256
OBLONG_NOTICE = 22202

function fossil_of(base)
    local objects = { EOBJECT0, EOBJECT1, EOBJECT2, EOBJECT3, EOBJECT4, EOBJECT5 }
    for number, object in ipairs(objects) do
        if base == object then
            return number - 1
        end
    end
    return nil
end

function fossil_bit(number)
    return 0x80 >> number
end

function rounds_found(player)
    return (player:get_quest_var(EVENT_ID, 0) or 0) & 0x0F
end

function oblongs_found(player)
    return (player:get_quest_var(EVENT_ID, 1) or 0) >> 4
end

-- As the client's IsAcceptEvent: its kind still wanted, and not searched yet.
function fossil_wanted(player, number)
    if ((player:get_quest_var(EVENT_ID, 5) or 0) & fossil_bit(number)) ~= 0 then
        return false
    end
    if number < 2 then
        return rounds_found(player) < ROUND_WANTED
    end
    return oblongs_found(player) < OBLONG_WANTED
end

function fossil_found(player, number)
    local flags = (player:get_quest_var(EVENT_ID, 5) or 0) | fossil_bit(number)
    if number < 2 then
        player:event_notice(EVENT_ID, 0, { ROUND_NOTICE })
        player:set_quest_var(EVENT_ID, 0, 0x11) -- UI8A
        player:set_quest_var(EVENT_ID, 2, 0x10) -- UI8C
    else
        local found = oblongs_found(player) + 1
        player:event_notice(EVENT_ID, 1, { found, OBLONG_WANTED, OBLONG_NOTICE })
        player:set_quest_var(EVENT_ID, 1, found * 0x11) -- UI8B
    end
    if rounds_found(player) >= ROUND_WANTED and oblongs_found(player) >= OBLONG_WANTED then
        player:set_quest_var(EVENT_ID, 0, 0x00) -- UI8A
        player:set_quest_var(EVENT_ID, 1, 0x21) -- UI8B
        player:set_quest_var(EVENT_ID, 2, 0x00) -- UI8C
        player:set_quest_var(EVENT_ID, 5, 0x00) -- UI8F
        player:quest_sequence(EVENT_ID, 255)
    else
        player:set_quest_var(EVENT_ID, 5, flags) -- UI8F
    end
end

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
    elseif sequence == 1 and fossil_of(BASE_ID) ~= nil then
        fossil = fossil_of(BASE_ID)
        if fossil_wanted(player, fossil) then
            step = 2
            play(player, 2 + fossil * 2, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
            return
        end
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        step = 3
        play(player, 14, HIDE_HOTBAR)
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
        play(player, 1, HIDE_HOTBAR)
        return
    elseif step == 1 and scene == 1 then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif step == 2 and scene == 2 + fossil * 2 then
        after_bar = { scene = 3 + fossil * 2, flags = NO_DEFAULT_CAMERA | HIDE_HOTBAR }
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif step == 2 and scene == 3 + fossil * 2 then
        fossil_found(player, fossil)
    elseif step == 3 and scene == 14 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        play(player, 15, HIDE_HOTBAR)
        return
    elseif step == 3 and scene == 15 then
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
