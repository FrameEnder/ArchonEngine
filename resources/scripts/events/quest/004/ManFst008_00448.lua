-- Quest: Festive Endeavors (Gridania)
--
-- Checked against the client's quest script (kawari-questdump), Sapphire's script and a live recording of the quest
-- (questkit replay):
-- Sequence 0:   ACTOR0 (Lewin) offers the quest (scene 0, returns 1 when accepted), then explains it (scene 1.)
-- Sequence 1:   ACTOR2 (Mother Miounne) (scene 3): notice 0, sequence 2.
-- Sequence 2:   ACTOR3 (Beatin) (scene 4) gives ITEM0, the Monoa mask (UI8BH = 1), with notice 1 [ITEM0's icon], and
--               moves to sequence 255.
-- Sequence 255: hand ITEM0 to ACTOR2 (trade scene 5, 1 when handed over; then scene 6 with the reward.)
-- ACTOR1 (the Gods' Quiver bow) lets you into the Seat of the First Bow at any point (scene 2, a yes/no; TERRITORYTYPE0
-- at POPRANGE0.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_DOOR = 2
SCENE_MIOUNNE = 3
SCENE_BEATIN = 4
SCENE_TRADE = 5
SCENE_TRADE_OK = 6

NOTICE_MIOUNNE = 0
NOTICE_MASK = 1
-- ITEM0's icon, as retail sends it with notice 1.
ITEM0_ICON = 21002

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
    elseif BASE_ID == ACTOR1 then
        play(player, SCENE_DOOR, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR2 then
        play(player, SCENE_MIOUNNE, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR3 then
        play(player, SCENE_BEATIN, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
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
    elseif scene == SCENE_DOOR then
        if results[1] == 1 then
            player:finish_event()
            player:change_territory_pop_range(TERRITORYTYPE0, POPRANGE0)
            return
        end
    elseif scene == SCENE_MIOUNNE then
        player:event_notice(EVENT_ID, NOTICE_MIOUNNE)
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_BEATIN then
        setVar(player, 1, true, 1) -- UI8BH: ITEM0, the mask
        player:event_notice(EVENT_ID, NOTICE_MASK, { ITEM0_ICON })
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_TRADE then
        -- 1 when the mask was handed over.
        if results[1] == 1 then
            play(player, SCENE_TRADE_OK, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_TRADE_OK then
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
