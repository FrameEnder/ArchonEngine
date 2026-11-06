-- Quest: Influencing Inquisitors (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-06) and checked against Bob's Chronofoil captures
-- 130088ec and 3c56e09f (2026-11-06, the whole quest on each of his two characters: fixtures
-- tools/questkit/fixtures/938.json and `938-3c56e09f.json`):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1 (QuestAccepted):
--               sequence 1.
-- Sequence 1:   ACTOR1, 2, 3 and 4, in any order, each once (its counter UI8CH, UI8AL, UI8BH or UI8BL and BitFlag8
--               1-4): its scene (2-5), UI8AH (the count) up one, notice 0 [count, 4]; the fourth: sequence 255 with
--               every variable 0. Both characters went ACTOR1, 2, 3, 4.
-- Sequence 255: ACTOR5 rewards it (scene 6, a cutscene, 0xF8402EFB, a yield: QuestReward, 1; closing the window
--               leaves UI8A 0x11.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_REWARD = 6

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB on retail

-- The four: their scene, counter (byte, high nibble) and BitFlag8.
TALKS = {
    [ACTOR1] = { scene = 2, byte = 2, high = true, flag = 1 },
    [ACTOR2] = { scene = 3, byte = 0, high = false, flag = 2 },
    [ACTOR3] = { scene = 4, byte = 1, high = true, flag = 3 },
    [ACTOR4] = { scene = 5, byte = 1, high = false, flag = 4 },
}
NEEDED = 4
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
    elseif sequence == 1 and talk ~= nil and getVar(player, talk.byte, talk.high) == 0
        and not hasFlag(player, talk.flag) then
        talking = talk
        play(player, talk.scene, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR5 then
        play(player, SCENE_REWARD, CUTSCENE_FLAGS)
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
        player:event_notice(EVENT_ID, 0, { count, NEEDED })
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
