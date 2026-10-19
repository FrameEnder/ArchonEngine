-- Quest: The Perfect Swarm (Ul'dah)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-19) and retail (Bob's Chronofoil capture of
-- 2026-10-18, questkit replay 329):
-- Sequence 0:   ACTOR0 (the Drunken Stag) offers it (scene 0, a yield: QuestOffer, 1 when accepted): sequence 1.
-- Sequence 1:   defeat four ENEMY0 (tainted water sprites, a BNpcBase) in Western Thanalan: UI8AL counts them (the
--               client ticks todo 0 at 4) with notice 0 [count, 4]; the fourth clears it and moves to sequence 255.
-- Sequence 255: ACTOR1 (Fufulupa) rewards it (scene 1, a yield: QuestReward, 1 and the chosen item.)

SCENE_OFFER = 0
SCENE_FINISH = 1

NOTICE_PROGRESS = 0
SPRITES_NEEDED = 4

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
        play(player, SCENE_FINISH, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onBNpcKill(base_id, layout_id, player)
    if base_id ~= ENEMY0 or player:get_quest_sequence(EVENT_ID) ~= 1 then
        return
    end

    local defeated = getVar(player, 0, false) + 1 -- UI8AL
    if defeated > SPRITES_NEEDED then
        return
    end
    player:event_notice(EVENT_ID, NOTICE_PROGRESS, { defeated, SPRITES_NEEDED })
    if defeated >= SPRITES_NEEDED then
        player:set_quest_var(EVENT_ID, 0, 0)
        player:quest_sequence(EVENT_ID, 255)
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
