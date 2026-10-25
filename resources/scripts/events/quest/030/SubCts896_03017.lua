-- Quest: If I Had a Glamour (Vesper Bay)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-25) and retail (Bob's Chronofoil capture of
-- 2026-10-25, questkit replay 3017):
-- Sequence 0:   ACTOR0 (Swyrgeim) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then talks (scene 1,
--               which ends with QuestAccepted): sequence 1.
-- Sequence 1:   ACTOR1 (Folclind) gives you ITEM0 (scene 2): notice 0 with the item's icon (24401, as retail sent
--               it), UI8BH = 1 (the client's GetEventItems: ITEM0 count in UI8BH) and sequence 255. ACTOR0 has a line
--               (scene 3; the client offers him in this sequence.)
-- Sequence 255: ACTOR0 asks for ITEM0 (scene 4: NpcTrade in event-item mode, a yield, 1 once it's handed over), then
--               rewards it (scene 5, a yield: QuestReward, 1 and the chosen item; it shows glamour's how-tos.) ITEM0
--               is a key item, held in the quest's variables, so nothing leaves the bags. ACTOR1 has a line (scene 6.)
-- Completing it unlocks glamours (UnlockLink 15, the Quest sheet's; the server's finish_quest sends it.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_FOLCLIND = 2
SCENE_SWYRGEIM_WAITING = 3
SCENE_TRADE = 4
SCENE_REWARD = 5
SCENE_FOLCLIND_AFTER = 6

ITEM_ICON = 24401 -- notice 0's argument on retail (Bob's Chronofoil capture of 2026-10-25)

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
        play(player, SCENE_FOLCLIND, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR0 then
        play(player, SCENE_SWYRGEIM_WAITING, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_FOLCLIND_AFTER, HIDE_HOTBAR)
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
    elseif scene == SCENE_FOLCLIND then
        player:event_notice(EVENT_ID, 0, { ITEM_ICON })
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH = 1: ITEM0
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_TRADE then
        -- 1 when the item was handed over, nothing when the trade window was closed.
        if results[1] == 1 then
            play(player, SCENE_REWARD, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_REWARD then
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
