-- Quest: Nothing to See Here (Ul'dah)
--
-- Checked against the client's quest script (kawari-questdump) and retail (a Kawari Recorder export and Bob's
-- Chronofoil session of 2026-10-17, questkit replay 671):
-- Sequence 0:   ACTOR0 offers and accepts the quest (scene 0, a yield returning 1), giving you three of ITEM0
--               (UI8CH = 3.)
-- Sequence 1:   hand one ITEM0 to each of ACTOR1, ACTOR2 and ACTOR3, in any order: a trade scene (1, 4, 7; returns 1
--               when handed over), then the NPC's reaction (2, 5, 8.) Each one sets its variable and flag (IsAnnounce):
--                 * ACTOR1 -> UI8AL = 1, BitFlag8 1
--                 * ACTOR2 -> UI8BH = 1, BitFlag8 2
--                 * ACTOR3 -> UI8BL = 1, BitFlag8 3
--               UI8AH counts them and UI8CH the items left; each sends notice 0 with { count, 3 } (GetTodoArgs.) The
--               third clears the variables and moves to sequence 255. Talking to one already done plays its short
--               line (3, 6, 9.)
-- Sequence 255: report to ACTOR0, and the reward (scene 10, a yield returning 1 and the chosen item.) ACTOR1-3 have
--               their short lines (11, 12, 13.)

SCENE_OFFER = 0
SCENE_FINISH = 10
NEEDED = 3
NOTICE_HANDED = 0

-- Per NPC: trade scene, reaction, line when already done (sequence 1 and 255), its variable (byte, high nibble) and
-- its BitFlag8.
NPCS = {
    [ACTOR1] = { trade = 1, reaction = 2, done = 3, after = 11, byte = 0, high = false, flag = 1 },
    [ACTOR2] = { trade = 4, reaction = 5, done = 6, after = 12, byte = 1, high = true, flag = 2 },
    [ACTOR3] = { trade = 7, reaction = 8, done = 9, after = 13, byte = 1, high = false, flag = 3 },
}

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
current_npc = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local npc = NPCS[BASE_ID]
    current_npc = npc

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and npc ~= nil then
        if getVar(player, npc.byte, npc.high) == 0 then
            play(player, npc.trade, HIDE_HOTBAR)
        else
            play(player, npc.done, HIDE_HOTBAR)
        end
    elseif sequence == 255 and npc ~= nil then
        play(player, npc.after, HIDE_HOTBAR)
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
    local npc = current_npc

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID)
            player:quest_sequence(EVENT_ID, 1)
            setVar(player, 2, true, NEEDED) -- UI8CH: the ITEM0s to hand out
        end
    elseif npc ~= nil and scene == npc.trade then
        if results[1] == 1 then
            play(player, npc.reaction, HIDE_HOTBAR)
            return
        end
    elseif npc ~= nil and scene == npc.reaction then
        local count = getVar(player, 0, true) + 1
        setVar(player, npc.byte, npc.high, 1)
        setBitFlag8(player, npc.flag)
        setVar(player, 0, true, count) -- UI8AH
        setVar(player, 2, true, NEEDED - count) -- UI8CH
        player:event_notice(EVENT_ID, NOTICE_HANDED, { count, NEEDED })
        if count >= NEEDED then
            for byte = 0, 5 do
                player:set_quest_var(EVENT_ID, byte, 0)
            end
            player:quest_sequence(EVENT_ID, 255)
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
