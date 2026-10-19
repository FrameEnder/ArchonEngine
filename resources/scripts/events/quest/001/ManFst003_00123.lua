-- Quest: Close to Home (Gridania), for Archers
-- NOTE: The same quest as ManFst004_00124 (Conjurers) and ManFst002_00085 (Lancers), with a different guild
-- (ACTOR2) and ITEM0. The client's scripts differ only in ACTOR2's dialogue, so keep them in sync.
--
-- Checked against the client's quest script (kawari-questdump); no retail recording of this version, so what retail
-- does (flags, variables, notices) is ManFst004_00124's, whose client script it matches scene for scene:
-- Sequence 0:   ACTOR0 offers the quest (scene 0), then explains it (scene 50.) Accepting moves you from the opening's
--               New Gridania into the real one (TERRITORYTYPE0), and gives you ITEM0 to deliver.
-- Sequence 1:   three objectives, in any order:
--                 * attune to the aetheryte (ACTOR1, scenes 51 then 1)     -> UI8AL = 1
--                 * visit the guild (ACTOR2, scene 2)                      -> UI8BH = 1
--                 * hand ITEM0 to ACTOR3 (trade scene 3, then scene 100)  -> UI8BL = 1
--               ITEM0 is held in UI8CH during this sequence (GetEventItems). UI8AH counts the objectives done, and
--               attuning also sets BitFlag8 1 (UI8F 0x80), as retail does in 124. Each objective sends its own notice,
--               numbered by objective (0 attune, 1 guild, 2 delivery) whatever the order, the last one included.
-- Sequence 255: report to ACTOR4 (scene 5) to finish.

SCENE_OFFER = 0
SCENE_EXPLAIN = 50
SCENE_AETHERYTE_WAIT = 51
SCENE_AETHERYTE = 1
SCENE_GUILD = 2
SCENE_TRADE = 3
SCENE_TRADE_OK = 100
SCENE_TRADE_NO = 99
SCENE_FINISH = 5

-- One notice per objective, with no arguments (as retail sends them in ManFst004_00124.)
NOTICE_ATTUNED = 0
NOTICE_GUILD = 1
NOTICE_DELIVERED = 2

UNLOCK_RETURN = 1 -- UnlockLink for Return, learned when attuning here
SET_EOBJ_BASE = 0xF8400EF3 -- scene flags the attunement and hand-over scenes use on retail (in 124)
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB, as retail plays 124's cutscenes (Chronofoil, 2026-10-14)

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

    if sequence == nil then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR2 then
        play(player, SCENE_GUILD, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR3 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID ~= ACTOR0 then
        -- The aetheryte is the only other thing the client sends to this quest during sequence 1.
        if not player:has_aetheryte(ACTOR1) then
            player:unlock_aetheryte(1, ACTOR1)
        end
        player:unlock(UNLOCK_RETURN)
        play(player, SCENE_AETHERYTE_WAIT, HIDE_HOTBAR | NO_DEFAULT_CAMERA)
    elseif sequence == 255 and BASE_ID == ACTOR4 then
        play(player, SCENE_FINISH, CUTSCENE_FLAGS)
    else
        player:finish_event()
    end
end

-- UI8AH counts the objectives done, as retail does in 124 (its recording: 1 after attuning, 2 after the delivery.)
function objectiveDone(player)
    setVar(player, 0, true, getVar(player, 0, true) + 1)
end

-- Moves to turn-in once all three objectives are done. Each sequence has its own variables, so they're cleared.
function checkCompletion(player)
    if getVar(player, 0, false) == 1 and getVar(player, 1, true) == 1 and getVar(player, 1, false) == 1 then
        for byte = 0, 5 do
            player:set_quest_var(EVENT_ID, byte, 0)
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
        play(player, SCENE_AETHERYTE, SET_EOBJ_BASE | HIDE_HOTBAR | INVIS_EOBJ)
        return
    elseif scene == SCENE_AETHERYTE then
        setVar(player, 0, false, 1) -- UI8AL
        objectiveDone(player)
        -- BitFlag8 1, as retail sets it when attuning (the recording shows UI8F 0x80 from here on.)
        player:set_quest_var(EVENT_ID, 5, player:get_quest_var(EVENT_ID, 5) | 0x80)
        player:event_notice(EVENT_ID, NOTICE_ATTUNED)
        checkCompletion(player)
    elseif scene == SCENE_GUILD then
        setVar(player, 1, true, 1) -- UI8BH
        objectiveDone(player)
        player:event_notice(EVENT_ID, NOTICE_GUILD)
        checkCompletion(player)
    elseif scene == SCENE_TRADE then
        -- 1 means ITEM0 was handed over.
        if results[1] == 1 then
            play(player, SCENE_TRADE_OK, SET_EOBJ_BASE | HIDE_HOTBAR | INVIS_EOBJ)
            return
        end
    elseif scene == SCENE_TRADE_OK then
        setVar(player, 2, true, 0) -- UI8CH = 0: ITEM0 is gone
        setVar(player, 1, false, 1) -- UI8BL
        objectiveDone(player)
        player:event_notice(EVENT_ID, NOTICE_DELIVERED)
        checkCompletion(player)
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
