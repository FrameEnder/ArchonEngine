-- Quest: The Talk of Coerthas (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-06) and checked against Bob's Chronofoil captures
-- 130088ec and 3c56e09f (2026-11-06, the whole quest on each of his two characters: fixtures
-- tools/questkit/fixtures/897.json and `897-3c56e09f.json`):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: QuestOffer, 1 when accepted; the client turns it down itself while
--               QST_ACCEPT_CHECK is in the journal), then scene 1 (QuestAccepted): sequence 1.
-- Sequence 1:   ACTOR1, ACTOR2 and ACTOR3, in any order, each once (its counter, UI8AL, UI8BH or UI8BL, and its
--               BitFlag8, 1-3): its scene (2, 3, 4), its own notice (0, 1, 2: the journal has a step for each), UI8AH
--               (the count) up one; the third: sequence 255 with every variable 0. Both characters went ACTOR1, 2, 3.
-- Sequence 255: ACTOR0 rewards it (scene 5, a yield: QuestReward, 1; closing the window leaves UI8A 0x11.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_REWARD = 5

-- The three: their scene, notice, BitFlag8, and their counter (byte, high nibble.)
TALKS = {
    [ACTOR1] = { scene = 2, notice = 0, flag = 1, byte = 0, high = false },
    [ACTOR2] = { scene = 3, notice = 1, flag = 2, byte = 1, high = true },
    [ACTOR3] = { scene = 4, notice = 2, flag = 3, byte = 1, high = false },
}
NEEDED = 3
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
REWARD_CLOSED = 0x11

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
talking = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local talk = TALKS[BASE_ID]

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and talk ~= nil and not hasFlag(player, talk.flag) then
        talking = talk
        play(player, talk.scene, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
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
            play(player, SCENE_ACCEPTED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif talking ~= nil and scene == talking.scene then
        local talk = talking
        talking = nil
        local count = getVar(player, 0, true) + 1 -- UI8AH
        player:event_notice(EVENT_ID, talk.notice, {})
        if count >= NEEDED then
            for byte = 0, 5 do
                player:set_quest_var(EVENT_ID, byte, 0)
            end
            player:quest_sequence(EVENT_ID, 255)
        else
            setVar(player, 0, true, count)
            setVar(player, talk.byte, talk.high, 1)
            setFlag(player, talk.flag)
        end
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_event()
            player:finish_quest(EVENT_ID, results[2] or 0)
            return
        elseif player:get_quest_var(EVENT_ID, 0) ~= REWARD_CLOSED then
            player:set_quest_var(EVENT_ID, 0, REWARD_CLOSED)
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
