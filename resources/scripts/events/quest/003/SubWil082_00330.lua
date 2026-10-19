-- Quest: Last Letter to Lost Hope (Ul'dah)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-19) and retail (Bob's Chronofoil capture of
-- 2026-10-18, questkit replay 330):
-- Sequence 0:   ACTOR0 (Fufulupa, Western Thanalan) offers it (scene 0, a yield: QuestOffer, 1 when accepted):
--               sequence 1.
-- Sequence 1:   in Central Thanalan, ACTOR1 (Seseli) gives you ITEM0 (scene 1, talk; the client offers her while UI8AL
--               and BitFlag8 1 are unset): UI8BH = 1 (the client lists ITEM0 from UI8BH), notice 0 with ITEM0's icon
--               (25918 on retail), sequence 255. ACTOR2 only talks (scene 2).
-- Sequence 255: ACTOR3 (Leofric) takes ITEM0 (scene 3, a hand-over window returning 1 and the item), then the reward
--               (scene 4, a yield: QuestReward, 1; anything else cancels the trade; gil only.) ACTOR1 and ACTOR2 still
--               talk (scenes 6 and 7: which is whose is a guess from the order; retail didn't play them.)

SCENE_OFFER = 0
SCENE_LETTER = 1
SCENE_ACTOR2 = 2
SCENE_DELIVER = 3
SCENE_FINISH = 4
SCENE_ACTOR1_AFTER = 6
SCENE_ACTOR2_AFTER = 7

NOTICE_LETTER = 0
ITEM0_ICON = 25918 -- what retail's notice carried

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
        play(player, SCENE_LETTER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR2 then
        play(player, SCENE_ACTOR2, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR3 then
        play(player, SCENE_DELIVER, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_ACTOR1_AFTER, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_ACTOR2_AFTER, HIDE_HOTBAR)
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
        end
    elseif scene == SCENE_LETTER then
        setVar(player, 1, true, 1) -- UI8BH = 1: ITEM0
        player:event_notice(EVENT_ID, NOTICE_LETTER, { ITEM0_ICON })
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_DELIVER then
        -- 1 means ITEM0 was handed over.
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
