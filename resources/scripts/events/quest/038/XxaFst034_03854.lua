-- Quest: Eggs over Queasy (Gridania)
--
-- Checked against the client's quest script (kawari-questdump):
-- Sequence 0:   ACTOR0 offers the quest (scene 0, returns 1 when accepted), then explains it (scene 1.)
-- Sequence 1:   defeat ENEMY0 to collect three of ITEM0. The client counts ITEM0 in UI8BH, both for the journal's
--               todo (it's done at 3) and for the items you hold. Each egg sends notice 0 [count, 3, icon].
-- Sequence 255: hand the ITEM0s to ACTOR1 (trade scene 2, returns 1 when handed over), then the reward (scene 3.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_TRADE = 2
SCENE_FINISH = 3

ITEMS_NEEDED = 3

-- Retail's notice for each egg, with ITEM0's icon (Bob's Chronofoil capture of 2026-10-14.)
NOTICE_EGGS = 0
ITEM0_ICON = 22623

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

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- Each ENEMY0 you (or your party) defeat gives you one ITEM0.
function onBNpcKill(base_id, layout_id, player)
    if base_id ~= ENEMY0 or player:get_quest_sequence(EVENT_ID) ~= 1 then
        return
    end

    local collected = getVar(player, 1, true) + 1 -- UI8BH
    setVar(player, 1, true, collected)
    player:event_notice(EVENT_ID, NOTICE_EGGS, { collected, ITEMS_NEEDED, ITEM0_ICON })
    if collected >= ITEMS_NEEDED then
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
            play(player, SCENE_ACCEPTED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_FINISH, HIDE_HOTBAR)
            return
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
