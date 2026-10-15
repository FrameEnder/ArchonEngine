-- Quest: Threat Level Elevated (Gridania)
--
-- Checked against the client's quest script (kawari-questdump), Sapphire's script and a live recording of the quest.
-- A letter is passed along: each NPC takes one key item (the trade window) and hands back the next. The client lists
-- the key items from the quest variables: ITEM0 in UI8BH, ITEM1 in UI8BL, ITEM2 in UI8CH.
-- Sequence 0:   ACTOR0 (Roseline) offers the quest (scene 0, returns 1 when accepted) and gives ITEM0.
-- Sequence 1:   hand ITEM0 to ACTOR1 (Elmar) (scene 1, the trade; scene 100 once it's handed over), who gives ITEM1.
--               Retail sends notice 0 with ITEM1's icon as the quest moves on to sequence 2.
-- Sequence 2:   hand ITEM1 to ACTOR2 (Bernard) (scene 2; scene 98), who gives ITEM2. Retail sends notice 1 with ITEM2's
--               icon as the quest moves on to sequence 255.
-- Sequence 255: hand ITEM2 to ACTOR3 (Eylgar) (scene 3; scene 96, with the reward.)

SCENE_OFFER = 0

-- Each trade: who takes it, in which sequence, and the scene once it's handed over.
TRADES = {
    [1] = { actor = ACTOR1, scene = 1, handed = 100 },
    [2] = { actor = ACTOR2, scene = 2, handed = 98 },
    [255] = { actor = ACTOR3, scene = 3, handed = 96 },
}

-- The quest's notices, with the icon of the item just given, as retail sends them.
NOTICE_ITEM1 = 0
NOTICE_ITEM2 = 1
ITEM1_ICON = 26151
ITEM2_ICON = 26151

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
    elseif sequence ~= nil and TRADES[sequence] ~= nil and BASE_ID == TRADES[sequence].actor then
        play(player, TRADES[sequence].scene, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil
    local sequence = player:get_quest_sequence(EVENT_ID)
    local trade = sequence ~= nil and TRADES[sequence] or nil

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID)
            player:quest_sequence(EVENT_ID, 1)
            setVar(player, 1, true, 1) -- UI8BH: ITEM0
        end
    elseif trade ~= nil and scene == trade.scene then
        -- 1 when the item was handed over, nothing when the trade window was closed.
        if results[1] == 1 then
            play(player, trade.handed, HIDE_HOTBAR)
            return
        end
    elseif trade ~= nil and scene == trade.handed then
        if sequence == 1 then
            player:event_notice(EVENT_ID, NOTICE_ITEM1, { ITEM1_ICON })
            player:quest_sequence(EVENT_ID, 2)
            setVar(player, 1, true, 0) -- UI8BH: ITEM0 handed over
            setVar(player, 1, false, 1) -- UI8BL: ITEM1
        elseif sequence == 2 then
            player:event_notice(EVENT_ID, NOTICE_ITEM2, { ITEM2_ICON })
            player:quest_sequence(EVENT_ID, 255)
            setVar(player, 1, false, 0) -- UI8BL: ITEM1 handed over
            setVar(player, 2, true, 1) -- UI8CH: ITEM2
        elseif results[1] == 1 then
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
