-- Quest: Every Little Thing She Does Is Magitek
--
-- Transcribed by questkit from the client's quest script (kawari-questdump) and Connor's Chronofoil capture e834e436 (2026-11-07/09) (fixture tools/questkit/fixtures/1005.json):
-- Sequence 0:
--   ACTOR0 (Cid): scene 0 (HIDE_HOTBAR, a yield), 1 (HIDE_HOTBAR) -> accepted, sequence 1
-- Sequence 1:
--   ACTOR2 (Wedge): scene 3 (HIDE_HOTBAR) -> sequence 2, notices [[0, [1, 1]]]
-- Sequence 2:
--   ACTOR3 (Serendipity): scene 5 (HIDE_HOTBAR) -> sequence 3, vars 00 10 00 00 00 00, notices [[1, [25919]]]
-- Sequence 3:
--   ACTOR1 (Slafborn): scene 8 (HIDE_HOTBAR, a yield), zone 156
--   ACTOR2 (Wedge): scene 6 (HIDE_HOTBAR, a yield), 7 (SET_BASE | INVIS_ENPC | INVIS_AETHERYTE | HIDE_HOTBAR) -> sequence 4, vars 00 00 00 00 00 00, notices [[2, [1, 1]]], zone 156
-- Sequence 4:
--   ACTOR4 (Wedge): scene 9 (HIDE_HOTBAR) -> mounted on the reaper (MOUNT0), sequence 5, notices [[3, []]]
-- Sequences 5-7 (the ride: the client hides the reaper, ACTOR7, in these):
--   ACTOR4 (Wedge): scenes 17, 23, 30 (HIDE_HOTBAR) -> back on the reaper after getting off (the client's script; unseen
--   on retail: scene 9's mount without the quest moving on)
-- Sequence 5:
--   walking into EVENTRANGE0: scene 14 (NO_DEFAULT_CAMERA | HIDE_HOTBAR), 15 (HIDE_HOTBAR) -> sequence 6, notices [[4, []]]
-- Sequence 6:
--   walking into EVENTRANGE1: scene 21 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 7, notices [[5, []]]
-- Sequence 7:
--   walking into EVENTRANGE2: scene 27 (NO_DEFAULT_CAMERA | HIDE_HOTBAR; its yield 232 is the server's: off the reaper,
--   resumed) -> sequence 8, notices [[6, []]]
-- The mount is retail's (Bob's Chronofoil captures da424662 and d1c46dff): the Mount packet as scene 9 ends, before its
-- notice (`player:mount`; Bob's bundle 20261114-033457: not put on the reaper, nor by Wedge after). Getting off at scene
-- 27 is the server's answer to the client's yield 232 (`events/lua.rs`), as for any scene that asks it.
-- Sequence 8:
--   ACTOR4 (Wedge): scene 34 (HIDE_HOTBAR) -> sequence 9, notices [[7, []]]
-- Sequence 9:
--   emote 41 on ACTOR7 (magitek reaper): scene 42 (NO_DEFAULT_CAMERA | HIDE_HOTBAR) -> sequence 10, notices [[8, []]]
-- Sequence 10:
--   ACTOR6 (Biggs): scene 44 (HIDE_HOTBAR, a yield), into the quest battle, zone 309
-- Sequence 255:
--   ACTOR0 (Cid): scene 50 (HIDE_HOTBAR, a yield) -> completed

-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855, 960.)
REWARD_CLOSED = 0x11

-- Wedge puts the player back on the reaper in the ride's sequences (the client's scenes for him there.)
WEDGE_REMOUNT = { [5] = 17, [6] = 23, [7] = 30 }

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
    elseif sequence == 1 and BASE_ID == ACTOR2 then
        step = 2
        play(player, 3, HIDE_HOTBAR)
        return
    elseif sequence == 2 and BASE_ID == ACTOR3 then
        step = 3
        play(player, 5, HIDE_HOTBAR)
        return
    elseif sequence == 3 and BASE_ID == ACTOR1 then
        step = 4
        play(player, 8, HIDE_HOTBAR)
        return
    elseif sequence == 3 and BASE_ID == ACTOR2 then
        step = 5
        play(player, 6, HIDE_HOTBAR)
        return
    elseif sequence == 4 and BASE_ID == ACTOR4 then
        step = 6
        play(player, 9, HIDE_HOTBAR)
        return
    elseif sequence ~= nil and sequence >= 5 and sequence <= 7 and BASE_ID == ACTOR4 then
        step = 14
        play(player, WEDGE_REMOUNT[sequence], HIDE_HOTBAR)
        return
    elseif sequence == 8 and BASE_ID == ACTOR4 then
        step = 10
        play(player, 34, HIDE_HOTBAR)
        return
    elseif sequence == 10 and BASE_ID == ACTOR6 then
        step = 12
        play(player, 44, HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        step = 13
        play(player, 50, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEmote(emote, target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == 9 and emote == 41 and BASE_ID == ACTOR7 then
        step = 11
        play(player, 42, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEnterTrigger(player, arg)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if sequence == 5 and arg == EVENTRANGE0 then
        step = 7
        play(player, 14, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 6 and arg == EVENTRANGE1 then
        step = 8
        play(player, 21, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
        return
    elseif sequence == 7 and arg == EVENTRANGE2 then
        step = 9
        play(player, 27, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
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
    elseif step == 2 and scene == 3 then
        player:event_notice(EVENT_ID, 0, { 1, 1 })
        player:quest_sequence(EVENT_ID, 2)
    elseif step == 3 and scene == 5 then
        player:event_notice(EVENT_ID, 1, { 25919 })
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8B
        player:quest_sequence(EVENT_ID, 3)
    elseif step == 4 and scene == 8 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        player:finish_event()
        -- Where retail put the player (ActorSetPos; no pop range of the quest's.)
        player:change_territory(TERRITORYTYPE0, { x = 1.36, y = -158.58, z = 11.22 }, 3.118)
        return
    elseif step == 5 and scene == 6 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        play(player, 7, SET_BASE | INVIS_ENPC | INVIS_AETHERYTE | HIDE_HOTBAR)
        return
    elseif step == 5 and scene == 7 then
        player:event_notice(EVENT_ID, 2, { 1, 1 })
        player:set_quest_var(EVENT_ID, 1, 0x00) -- UI8B
        player:quest_sequence(EVENT_ID, 4)
        player:finish_event()
        -- Where retail put the player (ActorSetPos; no pop range of the quest's.)
        player:change_territory(TERRITORYTYPE0, { x = -62.82, y = 3.38, z = -633.49 }, 2.524)
        return
    elseif step == 6 and scene == 9 then
        player:mount(MOUNT0)
        player:event_notice(EVENT_ID, 3, {})
        player:quest_sequence(EVENT_ID, 5)
    elseif step == 7 and scene == 14 then
        play(player, 15, HIDE_HOTBAR)
        return
    elseif step == 7 and scene == 15 then
        player:event_notice(EVENT_ID, 4, {})
        player:quest_sequence(EVENT_ID, 6)
    elseif step == 8 and scene == 21 then
        player:event_notice(EVENT_ID, 5, {})
        player:quest_sequence(EVENT_ID, 7)
    elseif step == 9 and scene == 27 then
        player:event_notice(EVENT_ID, 6, {})
        player:quest_sequence(EVENT_ID, 8)
    elseif step == 14 then
        -- As scene 9 does: the Mount, then the event's end.
        player:mount(MOUNT0)
    elseif step == 10 and scene == 34 then
        if player.scene_error_code ~= 0 then
            -- Broken off (retail's error code): nothing happens.
            player:finish_event()
            return
        end
        player:event_notice(EVENT_ID, 7, {})
        player:quest_sequence(EVENT_ID, 9)
    elseif step == 11 and scene == 42 then
        player:event_notice(EVENT_ID, 8, {})
        player:quest_sequence(EVENT_ID, 10)
    elseif step == 12 and scene == 44 then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        player:finish_event()
        player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE1)
        return
    elseif step == 13 and scene == 50 then
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
