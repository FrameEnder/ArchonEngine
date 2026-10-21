-- Quest 111: the Limsa Lominsa opening's way out, into the real Lower Decks
--
-- Written from Bob's Chronofoil capture of the Limsa opening on retail (2026-10-20) alone: the client's script hasn't
-- been dumped, so its name (any file ending _00111.lua is found) and parameter names aren't known, and NPCs are
-- compared by their layout IDs (the objects you talk to), as retail had them. Offered alongside Close to Home once
-- Coming to Limsa Lominsa is done (the opening's offer markers list both).
-- Sequence 0:   OFFERER offers it (scene 0, a yield: 1 accepts), then scene 1; when it returns, sequence 1 (retail sent
--               no AcceptQuest for this one, only the journal update.)
-- Sequence 1:   the opening's exit (EXIT, an event object) asks (scene 2, a yield: 1 goes): sequence 2, and the
--               player goes through a loading screen into the real Lower Decks (129) at pop range 4161445.
-- Sequence 2:   talk to the six people (PEOPLE, in any order; retail went in this order): each plays two scenes
--               (2n + 1, then 2n + 2 for the n-th, both returning) with HIDE_HOTBAR | NO_DEFAULT_CAMERA, then notice 1
--               with [count, 6, 25002]. UI8AL and UI8BH count them, BitFlag8 n marks the n-th; the sixth clears all but
--               UI8BH (6) and moves to sequence 255.
-- Sequence 255: FINISHER: scene 16, then the reward (scene 17, a yield: 1 completes it.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_EXIT = 2
SCENE_FINISH = 16
SCENE_REWARD = 17

-- Layout IDs, from the capture.
OFFERER = 4102073
EXIT = 2367400
PEOPLE = { 4082215, 4082216, 4082217, 4082218, 4082219, 4082220 }
FINISHER = 4069046

TERRITORY_LOWER_DECKS = 129
POP_RANGE_ARRIVAL = 4161445 -- retail's ZoneInit into the Lower Decks

NOTICE_PERSON = 1
NOTICE_ARG = 25002 -- retail's third notice argument
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001, as retail played the six people's scenes

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

function personIndex(object_id)
    for i, id in ipairs(PEOPLE) do
        if id == object_id then
            return i
        end
    end
    return nil
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- Which of the six this event is for (a new Lua state per event.)
current_person = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local person = personIndex(target.object_id)

    if sequence == nil and target.object_id == OFFERER then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and target.object_id == EXIT then
        play(player, SCENE_EXIT, HIDE_HOTBAR)
    elseif sequence == 2 and person ~= nil and not hasBitFlag8(player, person) then
        current_person = person
        play(player, 2 * person + 1, OBJECT_FLAGS)
    elseif sequence == 255 and target.object_id == FINISHER then
        play(player, SCENE_FINISH, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function personDone(player, person)
    local count = getVar(player, 0, false) + 1
    if count >= #PEOPLE then
        for b = 0, 5 do
            player:set_quest_var(EVENT_ID, b, 0)
        end
        setVar(player, 1, true, count) -- retail keeps UI8BH (6) in sequence 255
        player:event_notice(EVENT_ID, NOTICE_PERSON, { count, #PEOPLE, NOTICE_ARG })
        player:quest_sequence(EVENT_ID, 255)
    else
        setVar(player, 0, false, count)
        setVar(player, 1, true, count)
        setBitFlag8(player, person)
        player:event_notice(EVENT_ID, NOTICE_PERSON, { count, #PEOPLE, NOTICE_ARG })
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
    elseif scene == SCENE_EXIT then
        if results[1] == 1 then
            player:quest_sequence(EVENT_ID, 2)
            player:finish_event()
            player:change_territory_pop_range(TERRITORY_LOWER_DECKS, POP_RANGE_ARRIVAL)
            return
        end
    elseif current_person ~= nil and scene == 2 * current_person + 1 then
        play(player, scene + 1, OBJECT_FLAGS)
        return
    elseif current_person ~= nil and scene == 2 * current_person + 2 then
        personDone(player, current_person)
    elseif scene == SCENE_FINISH then
        if results[1] == 1 then
            play(player, SCENE_REWARD, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
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
