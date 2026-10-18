-- Quest: Dread Is in the Air (Gridania)
--
-- Checked against the client's quest script (kawari-questdump), Sapphire's script and a live recording of the quest
-- (questkit replay):
-- Sequence 0:   ACTOR0 (Luquelot) offers and accepts the quest in one scene (scene 0, SEQ_0_ACTOR0.) Retail goes straight
--               to sequence 255 with ITEM0, Luquelot's letter, in UI8BH.
-- Sequence 255: hand the letter to ACTOR1 (Mother Miounne) (trade scene 1, SEQ_1_ACTOR1; then SEQ_1_ACTOR1_NPCTRADEOK,
--               100, with the reward, or SEQ_1_ACTOR1_NPCTRADENO, 99, when the window was closed.)

SCENE_OFFER = SEQ_0_ACTOR0 or 0
SCENE_TRADE = SEQ_1_ACTOR1 or 1
SCENE_TRADE_OK = SEQ_1_ACTOR1_NPCTRADEOK or 100
SCENE_TRADE_NO = SEQ_1_ACTOR1_NPCTRADENO or 99

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB, as retail plays its cutscenes (Bob's Chronofoil capture of 2026-10-14)

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

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID)
            player:quest_sequence(EVENT_ID, 255)
            setVar(player, 1, true, 1) -- UI8BH: ITEM0, the letter
        end
    elseif scene == SCENE_TRADE then
        -- 1 when the letter was handed over.
        if results[1] == 1 then
            play(player, SCENE_TRADE_OK, CUTSCENE_FLAGS)
        else
            play(player, SCENE_TRADE_NO, HIDE_HOTBAR)
        end
        return
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
