-- Quest: Washed Up (Limsa Lominsa)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-21) and Bob's Chronofoil capture of the Limsa
-- story on retail (2026-10-21):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: 1 accepts; the client refuses it itself while QST_ACCEPT_CHECK is
--               accepted): sequence 1.
-- Sequence 1:   talk to ACTOR1..ACTOR5, in any order: ACTORn plays scene n (HIDE_HOTBAR | NO_DEFAULT_CAMERA; the
--               client's menu in it changes nothing here), then notice 0 with [count, 5]. UI8AL counts them, BitFlag8 n
--               marks ACTORn; the fifth clears the variables and moves to sequence 255.
-- Sequence 255: ACTOR0: the reward (scene 6, a yield: 1 completes it.)

SCENE_OFFER = 0
SCENE_FINISH = 6

NOTICE_TALKED = 0
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001, as retail played the five talks

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

-- Which of ACTOR1..ACTOR5 BASE_ID is (1..5), if it's one.
function talkerIndex()
    for i, id in ipairs({ ACTOR1, ACTOR2, ACTOR3, ACTOR4, ACTOR5 }) do
        if id == BASE_ID then
            return i
        end
    end
    return nil
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local talker = talkerIndex()

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and talker ~= nil and getVar(player, 0, false) < 5 and not hasBitFlag8(player, talker) then
        play(player, talker, OBJECT_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_FINISH, HIDE_HOTBAR)
    else
        player:finish_event()
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
    elseif scene >= 1 and scene <= 5 then
        local count = getVar(player, 0, false) + 1
        player:event_notice(EVENT_ID, NOTICE_TALKED, { count, 5 })
        if count >= 5 then
            for b = 0, 5 do
                player:set_quest_var(EVENT_ID, b, 0)
            end
            player:quest_sequence(EVENT_ID, 255)
        else
            setVar(player, 0, false, count)
            setBitFlag8(player, scene)
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
