-- Quest: You Shall Not Trespass (Gridania)
--
-- Checked against the client's quest script (kawari-questdump) and a live recording of the quest:
-- Sequence 0:   ACTOR0 (Keitha) offers the quest (scene 0, returns 1 when accepted.) The client itself refuses it while
--               QST_ACCEPT_CHECK is accepted.
-- Sequence 1:   talk to ACTOR1 (Roseline) at the Shroud's edge (scene 1.) Retail sends notice 0 (no arguments) as the
--               quest moves on to sequence 2.
-- Sequence 2:   defeat four ENEMY0 (qiqirn scramblers): UI8AL counts them, and each one sends notice 1 with [count, 4],
--               the progress pop-up. The fourth moves straight to sequence 255 with the count cleared.
-- Sequence 255: report to ACTOR1 for the reward (scene 3.)

SCENE_OFFER = 0
SCENE_ROSELINE = 1
SCENE_FINISH = 3

SCRAMBLERS_NEEDED = 4

-- The quest's notices, as retail sends them.
NOTICE_SEQUENCE = 0
NOTICE_PROGRESS = 1

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
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_ROSELINE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_FINISH, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- Each ENEMY0 you (or your party) defeat counts, up to four.
function onBNpcKill(base_id, layout_id, player)
    if base_id ~= ENEMY0 or player:get_quest_sequence(EVENT_ID) ~= 2 then
        return
    end

    local defeated = getVar(player, 0, false) + 1 -- UI8AL
    if defeated > SCRAMBLERS_NEEDED then
        return
    end
    player:event_notice(EVENT_ID, NOTICE_PROGRESS, { defeated, SCRAMBLERS_NEEDED })
    if defeated >= SCRAMBLERS_NEEDED then
        player:quest_sequence(EVENT_ID, 255)
        player:set_quest_var(EVENT_ID, 0, 0)
    else
        setVar(player, 0, false, defeated)
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
    elseif scene == SCENE_ROSELINE then
        player:event_notice(EVENT_ID, NOTICE_SEQUENCE)
        player:quest_sequence(EVENT_ID, 2)
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
