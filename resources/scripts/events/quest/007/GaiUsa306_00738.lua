-- Quest: Sylphish Concerns
--
-- Written from the client's quest script (kawari-questdump, 2026-10-30) and checked against Bob's Chronofoil capture
-- caa5e908 (2026-10-30, the whole quest: its fixture is tools/questkit/fixtures/738.json):
-- Sequence 0:   ACTOR0 (Buscarron) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1, where the
--               client accepts it: sequence 1.
-- Sequence 1:   five destinations (EOBJECT0-4), in any order (the client offers each while its flag, and for the last
--               three its nibble, are unset). Each: its first scene (0x2001, empty), then EVENT_ACTION_WAITING2_MIDDLE's
--               bar (ActorControlTarget EventAction 1, 12; five seconds), then its second scene (0x2001): UI8AH counts
--               them, notice 0 [count, 5], BitFlag8 n + 1 and a nibble of its own:
--                 EOBJECT0 scenes 2, 3: UI8CH, and ENEMY0 comes out (scene 3's pop-up message)
--                 EOBJECT1 scenes 4, 5: UI8CL (a guess: retail's was the last, which clears them), ENEMY1 and ENEMY2
--                          come out
--                 EOBJECT2 scenes 6, 7: UI8AL
--                 EOBJECT3 scenes 8, 9: UI8BH
--                 EOBJECT4 scenes 10, 11: UI8BL
--               The enemies (layout IDs) needn't be fought. With all five retail cleared the variables and moved to
--               sequence 255.
-- Sequence 255: ACTOR0 rewards it (scene 12, a yield: QuestReward, 1 and the chosen item.)
-- The enemies are layer QB_GaiUsa306_001 of the South Shroud drop-in, at retail's positions, levels and HP.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_REWARD = 12
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

NOTICE_DESTINATION = 0
DESTINATIONS_NEEDED = 5

-- The destinations: their scenes (before and after the bar), flag, nibble (byte, high) and who comes out.
DESTINATIONS = {
    { eobj = EOBJECT0, scene = 2, found = 3, flag = 1, byte = 2, high = true, enemies = { ENEMY0 } },
    { eobj = EOBJECT1, scene = 4, found = 5, flag = 2, byte = 2, high = false, enemies = { ENEMY1, ENEMY2 } },
    { eobj = EOBJECT2, scene = 6, found = 7, flag = 3, byte = 0, high = false, enemies = {} },
    { eobj = EOBJECT3, scene = 8, found = 9, flag = 4, byte = 1, high = true, enemies = {} },
    { eobj = EOBJECT4, scene = 10, found = 11, flag = 5, byte = 1, high = false, enemies = {} },
}

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

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
searching = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    searching = nil

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
        return
    elseif sequence == 1 then
        for _, destination in ipairs(DESTINATIONS) do
            if BASE_ID == destination.eobj and not hasFlag(player, destination.flag) then
                searching = destination
                play(player, destination.scene, OBJECT_FLAGS)
                return
            end
        end
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

-- The bar is over: whoever lies in wait comes out, and the second scene says so.
function onEventActionCast(player)
    if searching == nil then
        player:finish_event()
        return
    end
    for _, enemy in ipairs(searching.enemies) do
        player:spawn_quest_bnpc(enemy)
    end
    play(player, searching.found, OBJECT_FLAGS)
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            play(player, SCENE_ACCEPTED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif searching ~= nil and scene == searching.scene then
        -- The first scene only waits: then the bar (see onEventActionCast.)
        player:event_action(EVENT_ACTION_WAITING2_MIDDLE)
        return
    elseif searching ~= nil and scene == searching.found then
        local found = getVar(player, 0, true) + 1
        player:event_notice(EVENT_ID, NOTICE_DESTINATION, { found, DESTINATIONS_NEEDED })
        if found >= DESTINATIONS_NEEDED then
            for byte = 0, 5 do
                player:set_quest_var(EVENT_ID, byte, 0)
            end
            player:quest_sequence(EVENT_ID, 255)
        else
            setVar(player, 0, true, found)
            setVar(player, searching.byte, searching.high, 1)
            setFlag(player, searching.flag)
        end
        searching = nil
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
