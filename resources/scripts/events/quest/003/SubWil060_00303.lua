-- Quest: Step Nine (Ul'dah)
--
-- Written from retail (Bob's Chronofoil capture of 2026-10-18, questkit replay 303) and Sapphire's SubWil060; not yet
-- checked against the client's script (no kawari-questdump of it yet):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield returning 1 when accepted.) Accepting gives you ITEM0 to deliver
--               (UI8BH = 1.)
-- Sequence 1:   ACTOR1 takes ITEM0 (scene 1, a hand-over returning 1 and the item) and gives you ITEM1 (scene 2):
--               UI8BH = 0, UI8BL = 1, notice 0 with ITEM1's icon, then sequence 255.
-- Sequence 255: ACTOR2 takes ITEM1 (scene 4, a hand-over) and gives the reward (scene 5, a yield returning 1.)

SCENE_OFFER = 0
SCENE_DELIVER = 1
SCENE_DELIVERED = 2
SCENE_HAND_IN = 4
SCENE_FINISH = 5

NOTICE_DELIVERED = 0
ITEM1_ICON = 25210 -- what retail's notice carried (Sapphire's Item1Icon)

-- Quest variables: UI8A..F are bytes 0..5, H is the high nibble and L the low one.
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
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_DELIVER, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_HAND_IN, HIDE_HOTBAR)
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
            player:accept_quest(EVENT_ID)
            player:quest_sequence(EVENT_ID, 1)
            setVar(player, 1, true, 1) -- UI8BH = 1: ITEM0
        end
    elseif scene == SCENE_DELIVER then
        -- 1 means ITEM0 was handed over.
        if results[1] == 1 then
            play(player, SCENE_DELIVERED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_DELIVERED then
        setVar(player, 1, true, 0) -- UI8BH = 0: ITEM0 is gone
        setVar(player, 1, false, 1) -- UI8BL = 1: ITEM1
        player:event_notice(EVENT_ID, NOTICE_DELIVERED, { ITEM1_ICON })
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_HAND_IN then
        if results[1] == 1 then
            play(player, SCENE_FINISH, HIDE_HOTBAR)
            return
        end
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
