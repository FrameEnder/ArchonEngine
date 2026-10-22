-- Quest: Righting the Shipwright (Limsa Lominsa)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-21) and Bob's Chronofoil capture of the Limsa
-- story on retail (2026-10-21):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: 1 accepts): sequence 1.
-- Sequence 1:   ACTOR1 (scene 1): notice 0, sequence 2.
-- Sequence 2:   EOBJECT0..EOBJECT2, in any order: EOBJECTn plays scene 6n + 2 (empty), EVENT_ACTION_GATHER_SHORT (the
--               progress bar), then scene 6n + 3 (the client's message that something comes out), all with
--               HIDE_HOTBAR | NO_DEFAULT_CAMERA; it gives ITEMn (notice n + 1 with ITEMn's icon), and then ENEMYn comes
--               out (a layout ID, drop-in layer QB_SubSea118_001; killing it changes nothing). UI8AH counts them, the
--               object's own nibble (UI8AL, UI8BH, UI8BL for EOBJECT0..2) and BitFlag8 n + 1 mark it, and ITEMn is held
--               in UI8CH, UI8CL, UI8DH. The third moves the items to UI8BH, UI8BL and UI8CH, with sequence 3.
--               ACTOR1 has a line meanwhile (scene 20).
-- Sequence 3:   hand the items to ACTOR1 (scene 21, an NpcTrade: 1 if given), then scene 22: notice 4, sequence 255.
-- Sequence 255: ACTOR2 rewards it (scene 24, a yield: 1 completes it.)

SCENE_OFFER = 0
SCENE_ACTOR1 = 1
SCENE_ACTOR1_WAITING = 20
SCENE_TRADE = 21
SCENE_TRADED = 22
SCENE_FINISH = 24

OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001
NEEDED = 3

-- Each object: its done nibble, where its item is held, the notice's icon (retail's) and its enemy.
OBJECTS = {
    [0] = { done = { 0, false }, held = { 2, true }, icon = 21602 },
    [1] = { done = { 1, true }, held = { 2, false }, icon = 21003 },
    [2] = { done = { 1, false }, held = { 3, true }, icon = 20910 },
}

-- Quest variables: UI8A..F are bytes 0..5, H is the high nibble and L the low one. BitFlag8 n is bit 8 - n of byte 5.
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

function hasBitFlag8(player, n)
    return (player:get_quest_var(EVENT_ID, 5) & (1 << (8 - n))) ~= 0
end

function setBitFlag8(player, n)
    player:set_quest_var(EVENT_ID, 5, player:get_quest_var(EVENT_ID, 5) | (1 << (8 - n)))
end

-- Which of EOBJECT0..EOBJECT2 BASE_ID is (0..2), if it's one.
function objectIndex()
    for n, id in pairs({ [0] = EOBJECT0, [1] = EOBJECT1, [2] = EOBJECT2 }) do
        if id == BASE_ID then
            return n
        end
    end
    return nil
end

function enemyOf(n)
    return ({ [0] = ENEMY0, [1] = ENEMY1, [2] = ENEMY2 })[n]
end

-- The object whose bar is running, and the one whose enemy comes out once its event ends.
gathering = nil

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local n = objectIndex()

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_ACTOR1, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR1 then
        play(player, SCENE_ACTOR1_WAITING, HIDE_HOTBAR)
    elseif sequence == 2 and n ~= nil and not hasBitFlag8(player, n + 1) then
        gathering = n
        play(player, 6 * n + 2, OBJECT_FLAGS)
    elseif sequence == 3 and BASE_ID == ACTOR1 then
        play(player, SCENE_TRADE, OBJECT_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_FINISH, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    if player:get_quest_sequence(EVENT_ID) == 2 and gathering ~= nil then
        play(player, 6 * gathering + 3, OBJECT_FLAGS)
    else
        player:finish_event()
    end
end

function gathered(player, n)
    local object = OBJECTS[n]
    local count = getVar(player, 0, true) + 1
    player:event_notice(EVENT_ID, n + 1, { object.icon })
    if count >= NEEDED then
        for b = 0, 5 do
            player:set_quest_var(EVENT_ID, b, 0)
        end
        setVar(player, 1, true, 1) -- ITEM0
        setVar(player, 1, false, 1) -- ITEM1
        setVar(player, 2, true, 1) -- ITEM2
        player:quest_sequence(EVENT_ID, 3)
    else
        setVar(player, 0, true, count)
        setVar(player, object.done[1], object.done[2], 1)
        setVar(player, object.held[1], object.held[2], 1)
        setBitFlag8(player, n + 1)
    end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID)
            player:quest_sequence(EVENT_ID, 1)
        end
    elseif scene == SCENE_ACTOR1 then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif gathering ~= nil and scene == 6 * gathering + 2 then
        -- The event carries on once the progress bar finishes, see onEventActionCast.
        player:event_action(EVENT_ACTION_GATHER_SHORT)
        return
    elseif gathering ~= nil and scene == 6 * gathering + 3 then
        local n = gathering
        gathering = nil
        gathered(player, n)
        -- Retail's order: the scene, its end, then the enemy (brought out once at a time.)
        player:finish_event()
        player:spawn_quest_bnpc(enemyOf(n))
        return
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_TRADED, OBJECT_FLAGS)
            return
        end
    elseif scene == SCENE_TRADED then
        for b = 0, 5 do
            player:set_quest_var(EVENT_ID, b, 0)
        end
        player:event_notice(EVENT_ID, 4, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_FINISH then
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
