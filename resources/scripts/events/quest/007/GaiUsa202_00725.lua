-- Quest: Spirited Away
--
-- Written from the client's quest script (kawari-questdump, 2026-10-30) and checked against Bob's Chronofoil capture
-- caa5e908 (2026-10-30, the whole quest: its fixture is tools/questkit/fixtures/725.json):
-- Sequence 0:   ACTOR0 (Komuxio) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1, where the
--               client accepts it: sequence 1.
-- Sequence 1:   ACTOR1 (Vorsaile Heuloix) talks (scene 2): notice 0, sequence 2.
-- Sequence 2:   ACTOR2 (Giah Molkoh) talks (scene 3): notice 1, sequence 255.
-- Sequence 255: ACTOR3 (Buscarron) rewards it (scene 4, a cutscene and a yield: QuestReward, 1; gil only.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_VORSAILE = 2
SCENE_GIAH = 3
SCENE_REWARD = 4

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB on retail

NOTICE_VORSAILE = 0
NOTICE_GIAH = 1

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
        play(player, SCENE_VORSAILE, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR2 then
        play(player, SCENE_GIAH, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR3 then
        play(player, SCENE_REWARD, CUTSCENE_FLAGS)
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
    elseif scene == SCENE_VORSAILE then
        player:event_notice(EVENT_ID, NOTICE_VORSAILE, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_GIAH then
        player:event_notice(EVENT_ID, NOTICE_GIAH, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_REWARD then
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
