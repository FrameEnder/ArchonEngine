-- Quest: Migrant Marauders (Gridania)
--
-- Checked against the client's quest script (kawari-questdump), Sapphire's script and a live recording of the quest:
-- Sequence 0:   ACTOR0 (Eylgar) offers the quest (scene 0, returns 1 when accepted.) The client itself refuses it while
--               QST_ACCEPT_CHECK isn't complete.
-- Sequence 1:   search EOBJECT0, the vulture egg (EVENT_ACTION_SEARCH, then scene 100, the "vultures appear" message),
--               which brings out ENEMY0 and ENEMY1, two northern vultures (layout IDs, in the Central Shroud drop-ins.)
--               Defeating them is counted in UI8AL (and UI8BH as its high part, as the client reads it.) Retail sends no
--               notice for the first; the second sends notice 0 (no arguments) and moves straight to sequence 255 with
--               the count cleared. Retail doesn't mark the egg searched, so it can be searched again (to bring out
--               vultures that got away); only one of each is out at a time.
-- Sequence 255: report to ACTOR1 (Lothaire) (scene 2, with the reward.)

SCENE_OFFER = 0
SCENE_EGG = 100
SCENE_FINISH = 2

VULTURES_NEEDED = 2

NOTICE_DONE = 0

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
-- Whether the egg is being searched, while its progress bar runs.
searching = false

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == EOBJECT0 then
        searching = true
        player:event_action(EVENT_ACTION_SEARCH)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_FINISH, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    if not searching or player:get_quest_sequence(EVENT_ID) ~= 1 then
        player:finish_event()
        return
    end
    searching = false
    play(player, SCENE_EGG, HIDE_HOTBAR)
end

-- Each vulture you (or your party) defeat counts.
function onBNpcKill(base_id, layout_id, player)
    if (layout_id ~= ENEMY0 and layout_id ~= ENEMY1) or player:get_quest_sequence(EVENT_ID) ~= 1 then
        return
    end

    local defeated = getVar(player, 0, false) + getVar(player, 1, true) * 16 + 1 -- UI8AL, UI8BH
    if defeated >= VULTURES_NEEDED then
        player:event_notice(EVENT_ID, NOTICE_DONE)
        player:quest_sequence(EVENT_ID, 255)
        for byte = 0, 5 do
            player:set_quest_var(EVENT_ID, byte, 0)
        end
    else
        setVar(player, 0, false, defeated)
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
    elseif scene == SCENE_EGG then
        player:spawn_quest_bnpc(ENEMY0)
        player:spawn_quest_bnpc(ENEMY1)
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
