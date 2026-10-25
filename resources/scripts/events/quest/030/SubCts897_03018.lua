-- Quest: Absolutely Glamourous (Vesper Bay)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-25). Retail (Bob's Chronofoil capture of
-- 2026-10-25, questkit replay 3018) only has the offer, declined (scene 0 answered 0); everything after it follows
-- the client's script and If I Had a Glamour (3017), whose steps retail confirmed, and is unverified.
-- Sequence 0:   ACTOR0 (Swyrgeim) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then talks (scene 1,
--               which ends with QuestAccepted): sequence 1.
-- Sequence 1:   ACTOR1 (Folclind) gives you ITEM0 (scene 2): notice 0 and sequence 2. The client's GetEventItems
--               counts ITEM0 in UI8BH at sequence 1 but in UI8BL at sequence 2 (ITEM1 in UI8BH), so UI8BL = 1.
--               ACTOR0 has a line (scene 3.)
-- Sequence 2:   ACTOR0 asks for ITEM0 (scene 4: NpcTrade in event-item mode, a yield, 1 once handed over) and hands
--               you ITEM1 (scene 5): UI8BL = 0, UI8BH = 1 (ITEM1, where sequence 255 counts it), notice 1, sequence
--               255. ACTOR1 has a line (scene 6.)
-- Sequence 255: ACTOR2 (Goberin) asks for ITEM1 (scene 7: NpcTrade in event-item mode) and rewards it (scene 8, a
--               yield: QuestReward, 1 and the chosen item.) ACTOR0 has a line (scene 9.)
-- Both items are key items, held in the quest's variables. The notices' item icons (3017's carried one) aren't
-- known for these items, so they're sent without.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_FOLCLIND = 2
SCENE_SWYRGEIM_WAITING = 3
SCENE_SWYRGEIM_TRADE = 4
SCENE_SWYRGEIM_GIVES = 5
SCENE_FOLCLIND_AFTER = 6
SCENE_GOBERIN_TRADE = 7
SCENE_REWARD = 8
SCENE_SWYRGEIM_AFTER = 9

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
    elseif sequence == 2 and BASE_ID == ACTOR0 then
        play(player, SCENE_SWYRGEIM_TRADE, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR1 then
        play(player, SCENE_FOLCLIND_AFTER, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_GOBERIN_TRADE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_SWYRGEIM_AFTER, HIDE_HOTBAR)
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
        player:event_notice(EVENT_ID, 0, {})
        player:set_quest_var(EVENT_ID, 1, 0x01) -- UI8BL = 1: ITEM0, where sequence 2 counts it
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_SWYRGEIM_TRADE then
        -- 1 when the item was handed over, nothing when the trade window was closed.
        if results[1] == 1 then
            play(player, SCENE_SWYRGEIM_GIVES, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_SWYRGEIM_GIVES then
        player:event_notice(EVENT_ID, 1, {})
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH = 1: ITEM1; ITEM0 handed over
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_GOBERIN_TRADE then
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
