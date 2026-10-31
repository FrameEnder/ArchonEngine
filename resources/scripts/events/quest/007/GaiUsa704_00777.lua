-- Quest: The Bear and the Young'uns' Cares
--
-- Written from the client's quest script (kawari-questdump, 2026-10-31) and checked against Bob's Chronofoil capture
-- 20ef0e90 (2026-10-31, the whole quest: its fixture is tools/questkit/fixtures/777.json):
-- Sequence 0:   ACTOR0 (Gundobald) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1, where the
--               client accepts it: sequence 1.
-- Sequence 1:   four to talk to, in any order (the client's IsAcceptEvent offers each while its nibble and flag are
--               unset):
--                 ACTOR1, scene 2: UI8AL = 1, BitFlag8 1
--                 ACTOR2, scene 3: UI8BH = 1, BitFlag8 2
--                 ACTOR3, scene 4: UI8BL = 1, BitFlag8 3
--                 ACTOR4, scene 5: UI8CH = 1, BitFlag8 4
--               UI8AH counts them, and each sends notice 0 [count, 4]. After the fourth retail cleared the variables
--               and moved to sequence 255.
-- Sequence 255: ACTOR0 rewards it (scene 6, a yield: QuestReward, 1 and the optional item chosen.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_REWARD = 6

NOTICE_TALKED = 0

-- The four in sequence 1: their scene, the nibble that counts them and their flag.
TALKS = {
    { actor = ACTOR1, scene = 2, byte = 0, high = false, flag = 1 },
    { actor = ACTOR2, scene = 3, byte = 1, high = true, flag = 2 },
    { actor = ACTOR3, scene = 4, byte = 1, high = false, flag = 3 },
    { actor = ACTOR4, scene = 5, byte = 2, high = true, flag = 4 },
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
current_talk = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    current_talk = nil

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
        return
    elseif sequence == 1 then
        for _, talk in ipairs(TALKS) do
            if BASE_ID == talk.actor and not hasFlag(player, talk.flag) then
                current_talk = talk
                play(player, talk.scene, HIDE_HOTBAR)
                return
            end
        end
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
        return
    end
    player:finish_event()
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
    elseif current_talk ~= nil and scene == current_talk.scene then
        local talk = current_talk
        current_talk = nil
        local done = getVar(player, 0, true) + 1
        player:event_notice(EVENT_ID, NOTICE_TALKED, { done, #TALKS })
        if done >= #TALKS then
            for byte = 0, 5 do
                player:set_quest_var(EVENT_ID, byte, 0)
            end
            player:quest_sequence(EVENT_ID, 255)
        else
            setVar(player, 0, true, done)
            setVar(player, talk.byte, talk.high, 1)
            setFlag(player, talk.flag)
        end
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
