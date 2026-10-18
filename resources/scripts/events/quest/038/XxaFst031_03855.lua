-- Quest: A Soldier's Breakfast (Gridania)
--
-- Checked against the client's quest script (kawari-questdump) and a live recording of the quest:
-- Sequence 0:   ACTOR0 offers the quest (scene 0, returns 1 when accepted), then explains it (scene 1.)
-- Sequence 1:   two todos, in any order:
--                 todo 0: take the anole egg, EOBJECT0 (scene 2, EVENTACTION0, then scene 3, both HIDE_HOTBAR |
--                         NO_DEFAULT_CAMERA as Bob's Chronofoil capture of 2026-10-14 shows): UI8AL = 1, ITEM0 count in
--                         UI8BL
--                 todo 1: defeat three ENEMY0 (anoles): UI8BH counts them
--               UI8AH counts the todos done. With both done, retail moves to sequence 2 and keeps only the egg, in
--               UI8BH (where the client looks for ITEM0 from then on.)
-- Sequence 2:   report to ACTOR0 (scene 4), which moves to sequence 255.
-- Notices (retail): 0 with the egg's icon when it's taken, 1 [count, 3] for each anole, 2 on reporting to ACTOR0.
-- Sequence 255: hand the egg to ACTOR1 (trade scene 5, returns 1 when handed over), then the reward (scene 6.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_EGG = 2
SCENE_TAKE_EGG = 3
EGG_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA
SCENE_REPORT = 4
SCENE_TRADE = 5
SCENE_FINISH = 6

ANOLES_NEEDED = 3

-- The notices retail sends, and the egg's icon (Bob's Chronofoil capture of 2026-10-14.)
NOTICE_EGG = 0
NOTICE_ANOLES = 1
NOTICE_REPORTED = 2
ITEM0_ICON = 25251

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

-- UI8AH: how many of the two todos are done. Once both are, it's on to Pauline with only the egg kept.
function todoDone(player)
    local done = getVar(player, 0, true) + 1
    setVar(player, 0, true, done)
    if done >= 2 then
        player:quest_sequence(EVENT_ID, 2)
        for byte = 0, 5 do
            player:set_quest_var(EVENT_ID, byte, 0)
        end
        setVar(player, 1, true, 1) -- UI8BH: ITEM0
    end
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == EOBJECT0 and getVar(player, 0, false) == 0 then
        -- The egg's scene, the progress bar (see sceneFinished), then its scene once that finishes.
        play(player, SCENE_EGG, EGG_FLAGS)
    elseif sequence == 2 and BASE_ID == ACTOR0 then
        play(player, SCENE_REPORT, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 then
        player:finish_event()
        return
    end
    play(player, SCENE_TAKE_EGG, EGG_FLAGS)
end

-- Each ENEMY0 you (or your party) defeat counts, up to three.
function onBNpcKill(base_id, layout_id, player)
    if base_id ~= ENEMY0 or player:get_quest_sequence(EVENT_ID) ~= 1 then
        return
    end

    local defeated = getVar(player, 1, true) -- UI8BH
    if defeated >= ANOLES_NEEDED then
        return
    end
    defeated = defeated + 1
    setVar(player, 1, true, defeated)
    player:event_notice(EVENT_ID, NOTICE_ANOLES, { defeated, ANOLES_NEEDED })
    if defeated >= ANOLES_NEEDED then
        todoDone(player)
    end
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
    elseif scene == SCENE_EGG then
        -- Taking the egg shows a progress bar; its scene plays once it finishes (see onEventActionCast.)
        player:event_action(EVENTACTION0)
        return
    elseif scene == SCENE_TAKE_EGG then
        setVar(player, 0, false, 1) -- UI8AL: the egg
        setVar(player, 1, false, 1) -- UI8BL: ITEM0
        player:event_notice(EVENT_ID, NOTICE_EGG, { ITEM0_ICON })
        todoDone(player)
    elseif scene == SCENE_REPORT then
        player:event_notice(EVENT_ID, NOTICE_REPORTED)
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_FINISH, HIDE_HOTBAR)
            return
        end
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
