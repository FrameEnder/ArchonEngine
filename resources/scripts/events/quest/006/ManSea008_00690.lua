-- Quest: High Society (Limsa Lominsa)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-21) and Bob's Chronofoil capture of the Limsa
-- story on retail (2026-10-21):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: 1 accepts), then scene 1 (empty): sequence 1.
-- Sequence 1:   ACTOR1 (scene 2): notice 0, sequence 2.
-- Sequence 2:   ACTOR2 (scene 3, she hands ITEM0 over): notice 1 [21002], UI8BH = 1, sequence 255.
-- Sequence 255: hand ITEM0 to ACTOR1 (scene 4, an NpcTrade: 1 if given), then the reward (scene 5, a yield with the
--               reward picked.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_ACTOR1 = 2
SCENE_ACTOR2 = 3
SCENE_TRADE = 4
SCENE_FINISH = 5

-- Notice 1's argument, as retail sent it (probably ITEM0's icon.)
ITEM_ICON = 21002

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
        play(player, SCENE_ACTOR1, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR2 then
        play(player, SCENE_ACTOR2, HIDE_HOTBAR)
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
            play(player, SCENE_ACCEPTED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_ACTOR1 then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_ACTOR2 then
        player:event_notice(EVENT_ID, 1, { ITEM_ICON })
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH = 1: ITEM0
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_FINISH, HIDE_HOTBAR)
            return
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
