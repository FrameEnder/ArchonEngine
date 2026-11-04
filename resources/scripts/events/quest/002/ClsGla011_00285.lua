-- Quest: Way of the Gladiator (for the other classes)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-04) and checked against Bob's Chronofoil capture
-- d3379099 (2026-11-04, on a Conjurer: its fixture is tools/questkit/fixtures/285.json). It follows So You Want to Be
-- a Gladiator (177); Gladiators who began as one take ClsGla020_00253 instead.
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: QuestOffer, 1 when accepted): sequence 255.
-- Sequence 255: ACTOR1 (the guildmaster): her cutscene (scene 1, retail's flags 0xF8482EFB, a yield: QuestReward, 1)
--               completes it. Its CLASSJOB parameter (1, Gladiator) is unlocked by the server's finish_quest, after its
--               EXP, as retail (UnlockClass, SetLevel: `unlock_quest_classjob`), with the weathered gladius it rewards.

SCENE_OFFER = 0
SCENE_GUILDMASTER = 1

CUTSCENE_FLAGS = 0xF8482EFB -- retail's (Bob's Chronofoil capture d3379099)

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
        play(player, SCENE_GUILDMASTER, CUTSCENE_FLAGS)
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
        end
    elseif scene == SCENE_GUILDMASTER then
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
