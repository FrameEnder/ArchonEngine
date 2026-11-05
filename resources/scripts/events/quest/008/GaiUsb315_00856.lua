-- Quest: As You Wish
--
-- Written from the client's quest script (kawari-questdump, 2026-11-05) and checked against Bob's Chronofoil captures
-- b4ce5e3f (2026-11-05, the whole quest on a Gladiator: its fixture is tools/questkit/fixtures/856.json), d65126c9
-- (an Arcanist, `856-d65126c9.json`) and 811eb9ff (a White Mage, `856-811eb9ff.json`):
-- Sequence 0:   ACTOR0 (Wheiskaet, in Eastern La Noscea) offers it (scene 0, a yield: QuestOffer, 1 when accepted),
--               then scene 1, where the client accepts it: sequence 1.
-- Sequence 1:   the three destinations in Upper La Noscea, in any order (put out by the client itself: no
--               SpawnObject on retail). Each is offered while its own nibble is 0 and its BitFlag8 unset (the
--               client's IsAcceptEvent: EOBJECT0 UI8AL and flag 1, EOBJECT1 UI8BH and flag 2, EOBJECT2 UI8BL and flag
--               3): its scene (2, 4 or 6, 0x2001, which only waits), the bar of the quest's EventAction (row 30),
--               then its scene after (3, 5 or 7, 0x2001.) Then that nibble = 1, its flag set, UI8AH (the count, the
--               todo's) one more, notice 0 [count, 3]; the third: the variables cleared, sequence 255.
-- Sequence 255: ACTOR1 (Riol) rewards it (scene 8, a yield: QuestReward, 1.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_REWARD = 8
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

NOTICE_VISITED = 0
DESTINATIONS_NEEDED = 3
-- What retail set UI8A to when a reward window was closed (Bob's capture b766138c, quest 840; 414 and 680.)
REWARD_CLOSED = 0x11

-- The destinations: each one's scenes, its "visited" nibble (the byte and whether it's the high one) and its BitFlag8.
DESTINATIONS = {
    { object = EOBJECT0, scene = 2, done = 3, byte = 0, high = false, flag = 1 },
    { object = EOBJECT1, scene = 4, done = 5, byte = 1, high = true, flag = 2 },
    { object = EOBJECT2, scene = 6, done = 7, byte = 1, high = false, flag = 3 },
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

function destinationOf(base_id)
    for _, destination in ipairs(DESTINATIONS) do
        if destination.object == base_id then
            return destination
        end
    end
    return nil
end

-- Whether the client offers this destination (its IsAcceptEvent.)
function offered(player, destination)
    return getVar(player, destination.byte, destination.high) == 0 and not hasFlag(player, destination.flag)
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- The destination being looked at in this event.
visiting = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local destination = destinationOf(BASE_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and destination ~= nil and offered(player, destination) then
        visiting = destination
        play(player, destination.scene, OBJECT_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- The bar finished: the destination's scene after it.
function onEventActionCast(player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 or visiting == nil then
        player:finish_event()
        return
    end
    play(player, visiting.done, OBJECT_FLAGS)
end

-- One more destination visited.
function visited(player, destination)
    local count = getVar(player, 0, true) + 1
    player:event_notice(EVENT_ID, NOTICE_VISITED, { count, DESTINATIONS_NEEDED })
    if count >= DESTINATIONS_NEEDED then
        for byte = 0, 5 do
            player:set_quest_var(EVENT_ID, byte, 0)
        end
        player:quest_sequence(EVENT_ID, 255)
        return
    end
    setVar(player, destination.byte, destination.high, 1)
    setVar(player, 0, true, count) -- UI8AH: the count
    setFlag(player, destination.flag)
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
    elseif visiting ~= nil and scene == visiting.scene then
        -- The bar, then the scene after it (onEventActionCast.)
        player:event_action(EVENT_ACTION_)
        return
    elseif visiting ~= nil and scene == visiting.done then
        local destination = visiting
        visiting = nil
        visited(player, destination)
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID, results[2])
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
