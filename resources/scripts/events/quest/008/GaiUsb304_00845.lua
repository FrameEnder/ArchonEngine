-- Quest: Give a Man a Drink
--
-- Written from the client's quest script (kawari-questdump, 2026-11-05) and checked against Bob's Chronofoil captures
-- b766138c (2026-11-05, the whole quest on a Gladiator: its fixture is tools/questkit/fixtures/845.json), 811b6f1b
-- (an Arcanist, `845-811b6f1b.json`) and 62c1dfe4 (a White Mage, `845-62c1dfe4.json`):
-- Sequence 0:   ACTOR0 (Shamani Lohmani) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1,
--               where the client accepts it: sequence 1, with ITEM0 (UI8BH = 1; the client lists it from there in
--               sequences 1 and 255.)
-- Sequence 1:   ACTOR1 (Rhitskylt) talks (scene 2): notice 0, sequence 255, ITEM0 kept.
-- Sequence 255: ACTOR2 (Drest) takes ITEM0 (scene 3, NpcTrade in event mode: a yield, 1 when handed over) and
--               rewards it (scene 4, a yield: QuestReward, 1 and the optional item chosen.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_RHITSKYLT = 2
SCENE_TRADE = 3
SCENE_REWARD = 4

NOTICE_RHITSKYLT = 0

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
        play(player, SCENE_RHITSKYLT, HIDE_HOTBAR)
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
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH: ITEM0
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_RHITSKYLT then
        player:event_notice(EVENT_ID, NOTICE_RHITSKYLT, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_TRADE then
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
