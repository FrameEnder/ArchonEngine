-- Quest: Out of House and Home (Ul'dah)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-19) and retail (Bob's Chronofoil capture of
-- 2026-10-18, questkit replay 307):
-- Sequence 0:   ACTOR0 (Warin) offers it (scene 0, a yield: the client's QuestOffer, 1 when accepted) and gives ITEM0
--               (UI8BH = 1; the client lists ITEM0 from UI8BH in every sequence.)
-- Sequence 1:   EOBJECT0, the fissure (the client offers it while BitFlag8 1 is unset, which retail never sets):
--               talking to it plays scene 1 (0x2001), which only opens the inventory. Using ITEM0 on it (the client
--               allows ITEM0 in sequence 1; the key item's 3 s cast, then the server's EventItem event) plays scene 2
--               (0x2001, a pop-up message), and ENEMY0, a coblyn, comes out of it. ENEMY0..2 are layout IDs; the
--               client counts them while UI8AL < 3 and ticks todo 0 at UI8AL >= 3. Defeating ENEMY0 counts UI8AL = 1
--               and brings out ENEMY1 and ENEMY2; the second kill counts UI8AL = 2, the third sends notice 0, clears
--               the variables (ITEM0 is gone) and moves to 255. (Sapphire counted to 4; the client and retail to 3.)
-- Sequence 255: ACTOR0 rewards it (scene 13, a yield: QuestReward, 1 and the chosen item.)
-- Scenes 3-12 are empty or pop-up messages in the client's script and retail never played them.
-- The coblyns are layer QB_SubWil064_001 of the Central Thanalan drop-in, at retail's positions, levels and HP.

SCENE_OFFER = 0
SCENE_FISSURE = 1
SCENE_ITEM_USED = 2
SCENE_FINISH = 13
FLAGS_FISSURE = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

NOTICE_CLEARED = 0
KILLS_NEEDED = 3

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
    elseif sequence == 1 and BASE_ID == EOBJECT0 then
        play(player, SCENE_FISSURE, FLAGS_FISSURE)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_FINISH, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEventItem(item_id, target, player)
    if item_id == ITEM0 and BASE_ID == EOBJECT0 and player:get_quest_sequence(EVENT_ID) == 1 then
        play(player, SCENE_ITEM_USED, FLAGS_FISSURE)
    else
        player:finish_event()
    end
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 then
        return
    end
    if layout_id ~= ENEMY0 and layout_id ~= ENEMY1 and layout_id ~= ENEMY2 then
        return
    end

    local kills = getVar(player, 0, false) + 1 -- UI8AL
    if kills >= KILLS_NEEDED then
        player:event_notice(EVENT_ID, NOTICE_CLEARED)
        for byte = 0, 5 do
            player:set_quest_var(EVENT_ID, byte, 0)
        end
        player:quest_sequence(EVENT_ID, 255)
        return
    end
    setVar(player, 0, false, kills)
    if layout_id == ENEMY0 then
        player:spawn_quest_bnpc(ENEMY1)
        player:spawn_quest_bnpc(ENEMY2)
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
    elseif scene == SCENE_ITEM_USED then
        -- The first coblyn, or (using the item again) the two after it if they got away.
        if getVar(player, 0, false) == 0 then
            player:spawn_quest_bnpc(ENEMY0)
        else
            player:spawn_quest_bnpc(ENEMY1)
            player:spawn_quest_bnpc(ENEMY2)
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
