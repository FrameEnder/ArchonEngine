-- Quest: Don't Look Down (Gridania)
--
-- Checked against the client's quest script (kawari-questdump), Sapphire's script and a live recording of the quest:
-- Sequence 0:   ACTOR0 (Osha Jaab) offers the quest (scene 0, returns 1 when accepted.)
-- Sequence 1:   search the four blue trumpets, EOBJECT0 to EOBJECT3, in any order (EVENT_ACTION_SEARCH, a progress bar,
--               then the trumpet's scene, 1 to 4.) UI8AL counts them, each sets its BitFlag8 (1 to 4, in UI8F) so the
--               client stops offering it, and each sends notice 0 with [count, 4], the progress pop-up. The fourth moves
--               straight to sequence 255 with the variables cleared.
-- Sequence 255: ACTOR1 (Theodore) and the reward (scene 5.)

SCENE_OFFER = 0
SCENE_FINISH = 5

-- Each trumpet: its scene and BitFlag8 index.
TRUMPETS = {
    [EOBJECT0] = { scene = 1, flag = 1 },
    [EOBJECT1] = { scene = 2, flag = 2 },
    [EOBJECT2] = { scene = 3, flag = 3 },
    [EOBJECT3] = { scene = 4, flag = 4 },
}
TRUMPETS_NEEDED = 4

NOTICE_PROGRESS = 0

-- The scene after a search has nothing to show, so it plays without hiding the hotbar (as Sapphire does), which
-- would only make it flicker once the progress bar ends.
SEARCH_SCENE_FLAGS = 0

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
-- The trumpet being searched, while its progress bar runs.
searching = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local trumpet = TRUMPETS[BASE_ID]

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and trumpet ~= nil and not hasFlag(player, trumpet.flag) then
        -- Searching shows a progress bar; the trumpet's scene plays once it finishes (see onEventActionCast.)
        searching = trumpet
        player:event_action(EVENT_ACTION_SEARCH)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_FINISH, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    if searching == nil or player:get_quest_sequence(EVENT_ID) ~= 1 then
        player:finish_event()
        return
    end
    play(player, searching.scene, SEARCH_SCENE_FLAGS)
end

-- One trumpet searched.
function searched(player, flag)
    local count = getVar(player, 0, false) + 1 -- UI8AL
    player:event_notice(EVENT_ID, NOTICE_PROGRESS, { count, TRUMPETS_NEEDED })
    if count >= TRUMPETS_NEEDED then
        player:quest_sequence(EVENT_ID, 255)
        for byte = 0, 5 do
            player:set_quest_var(EVENT_ID, byte, 0)
        end
    else
        setVar(player, 0, false, count)
        setFlag(player, flag)
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
    elseif searching ~= nil and scene == searching.scene then
        local flag = searching.flag
        searching = nil
        searched(player, flag)
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
