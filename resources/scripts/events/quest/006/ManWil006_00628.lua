-- Quest: Secrets and Lies (Ul'dah)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-20) and retail (Bob's Chronofoil capture of
-- 2026-10-18, questkit replay 628):
-- Sequence 0:   ACTOR0 (Fufulupa, Western Thanalan) offers it (scene 0, a yield: QuestOffer, 1 when accepted) and gives ITEM0: sequence
--               255 with UI8BH = 1 (the client lists ITEM0 from UI8BH.)
-- Sequence 255: ACTOR1 (Momodi, Ul'dah) takes ITEM0 (scene 1, a hand-over window returning 1 and the item), then
--               the reward in a cutscene (scene 2, 0xF8402EFB, a yield: QuestReward, 1; anything else cancels the
--               trade; gil only.)

SCENE_OFFER = 0
SCENE_DELIVER = 1
SCENE_FINISH = 2

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB on retail

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
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_DELIVER, HIDE_HOTBAR)
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
            setVar(player, 1, true, 1) -- UI8BH = 1: ITEM0
        end
    elseif scene == SCENE_DELIVER then
        -- 1 means ITEM0 was handed over.
        if results[1] == 1 then
            play(player, SCENE_FINISH, CUTSCENE_FLAGS)
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
