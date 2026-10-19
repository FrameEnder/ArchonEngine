-- Quest: Close to Home (Ul'dah)
-- NOTE: ManWil003_00569 and ManWil004_00570 are the same quest for the other Ul'dah classes (the same client script
-- apart from ITEM0 and ACTOR1); keep the three in step.
--
-- Checked against the client's quest script (kawari-questdump) and retail (a Kawari Recorder export and Bob's
-- Chronofoil session of 2026-10-17, questkit replay 568):
-- Sequence 0:   ACTOR0 offers the quest (scene 0, a yield), then explains it (scene 50.) Accepting moves you from the
--               opening's Ul'dah into the real one (TERRITORYTYPE0), and gives you ITEM0 to deliver (UI8CH = 1.)
-- Sequence 1:   three objectives, in any order (IsTodoChecked and IsAcceptEvent):
--                 * attune to the aetheryte (AETHERYTE0, scene 1 then 2)  -> UI8BL = 1, BitFlag8 1, notice 0
--                 * talk to ACTOR1 (scene 3)                              -> UI8AL = 1, BitFlag8 2, notice 1
--                 * hand ITEM0 to ACTOR2 (trade scene 4, then scene 5)    -> UI8BH = 1, BitFlag8 3, notice 2
--               UI8AH counts the objectives done (retail: 1 after the guild, 2 after the aetheryte.) The last one
--               clears the variables and moves to sequence 255.
-- Sequence 255: report to ACTOR3 (scene 8) to finish.

SCENE_OFFER = 0
SCENE_EXPLAIN = 50
SCENE_AETHERYTE_WAIT = 1 -- the client's scene only waits, while the aetheryte attunes
SCENE_AETHERYTE = 2
SCENE_ACTOR1 = 3
SCENE_TRADE = 4
SCENE_GUILD = 5
SCENE_FINISH = 8

-- One notice per objective, with no arguments (retail, Bob's Chronofoil session of 2026-10-17.)
NOTICE_ATTUNED = 0
NOTICE_ACTOR1 = 1
NOTICE_GUILD = 2

-- Attuning, as retail did it (Bob's Chronofoil capture of 2026-10-18, game 2026.10.06): the aetheryte's own event is
-- started inside the quest's (EventStart, type Nest), with its condition instead of the quest's, and EventAction 19
-- (the attunement animation and 5 s bar) on the aetheryte; when the bar ends, LearnTeleport, that event's notice 0
-- (with one 0), its EventFinish, and only then scene 1 with the quest's condition back.
AETHERYTE_HANDLER = 0x50000 -- | the Aetheryte row
EVENT_ACTION_ATTUNE = 19
CONDITION_OCCUPIED_IN_EVENT = 6
CONDITION_OCCUPIED_IN_QUEST_EVENT = 7

UNLOCK_RETURN = 1 -- UnlockLink for Return, which retail sends after the attunement cutscene
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB, what retail plays the cutscenes with

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

function setBitFlag8(player, n)
    player:set_quest_var(EVENT_ID, 5, player:get_quest_var(EVENT_ID, 5) | (1 << (8 - n)))
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == AETHERYTE0 and getVar(player, 1, false) == 0 then
        if not player:has_aetheryte(AETHERYTE0) then
            player:start_nested_event(AETHERYTE_HANDLER | AETHERYTE0)
            player:set_condition(CONDITION_OCCUPIED_IN_QUEST_EVENT, false)
            player:set_condition(CONDITION_OCCUPIED_IN_EVENT, true)
            player:event_action(EVENT_ACTION_ATTUNE) -- continues in onEventActionCast
            return
        end
        play(player, SCENE_AETHERYTE_WAIT, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 and getVar(player, 0, false) == 0 then
        play(player, SCENE_ACTOR1, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR2 and getVar(player, 1, true) == 0 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR3 then
        play(player, SCENE_FINISH, CUTSCENE_FLAGS)
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    player:unlock_aetheryte(1, AETHERYTE0)
    player:event_notice(AETHERYTE_HANDLER | AETHERYTE0, 0, {0})
    player:finish_nested_event(AETHERYTE_HANDLER | AETHERYTE0)
    player:set_condition(CONDITION_OCCUPIED_IN_EVENT, false)
    player:set_condition(CONDITION_OCCUPIED_IN_QUEST_EVENT, true)
    play(player, SCENE_AETHERYTE_WAIT, HIDE_HOTBAR)
end

-- An objective is done: its variable, its flag, the count, its notice; then turn-in once all three are.
function objectiveDone(player, byte, high, flag, notice)
    setVar(player, byte, high, 1)
    setBitFlag8(player, flag)
    setVar(player, 0, true, getVar(player, 0, true) + 1)
    player:event_notice(EVENT_ID, notice)
    if getVar(player, 0, true) >= 3 then
        -- Each sequence has its own variables, so they're cleared (retail: all zero in sequence 255.)
        for b = 0, 5 do
            player:set_quest_var(EVENT_ID, b, 0)
        end
        player:quest_sequence(EVENT_ID, 255)
    end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            play(player, SCENE_EXPLAIN, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_EXPLAIN then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
        player:set_quest_var(EVENT_ID, 2, 0x10) -- UI8CH = 1: ITEM0, the delivery

        -- Just like in retail, "seamlessly" transition them to the real zone:
        local old_position = player.position
        local old_rotation = player.rotation
        player:change_territory(TERRITORYTYPE0, { x = old_position.x, y = old_position.y, z = old_position.z }, old_rotation)
    elseif scene == SCENE_AETHERYTE_WAIT then
        play(player, SCENE_AETHERYTE, CUTSCENE_FLAGS)
        return
    elseif scene == SCENE_AETHERYTE then
        player:unlock(UNLOCK_RETURN)
        objectiveDone(player, 1, false, 1, NOTICE_ATTUNED) -- UI8BL, BitFlag8 1
    elseif scene == SCENE_ACTOR1 then
        objectiveDone(player, 0, false, 2, NOTICE_ACTOR1) -- UI8AL, BitFlag8 2
    elseif scene == SCENE_TRADE then
        -- 1 means ITEM0 was handed over.
        if results[1] == 1 then
            play(player, SCENE_GUILD, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_GUILD then
        setVar(player, 2, true, 0) -- UI8CH = 0: ITEM0 is gone
        objectiveDone(player, 1, true, 3, NOTICE_GUILD) -- UI8BH, BitFlag8 3
    elseif scene == SCENE_FINISH then
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
