-- Quest: Sylph-management (main scenario)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-27) and Bob's Chronofoil capture c99fe374
-- (2026-10-28, Dags: the whole quest, after The Company You Keep (Immortal Flames); its fixture is
-- tools/questkit/fixtures/513.json). It follows any of the three companies' next steps (683, 684, 685.)
-- Sequence 0:   ACTOR0 (Minfilia, in the Waking Sands) offers it (scene 0, a yield returning 1 when accepted), then
--               the cutscene (scene 1, retail's flags 0xF8402EFB; the client accepts it there): sequence 255 at once,
--               and the player is put back where they stood to talk to her (retail's PrepareZoning to the Waking Sands
--               and ActorSetPos, at the very spot.)
-- Sequence 255: ACTOR1 (Vorsaile Heuloix, in New Gridania) rewards it (scene 2, a yield returning 1 when the reward is
--               taken): the quest is completed.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_REWARD = 2

-- Retail's flags (Bob's Chronofoil capture c99fe374.)
CUTSCENE_FLAGS = 0xF8402EFB

-- Where the cutscene leaves the player: the Waking Sands, where they stood.
WAKING_SANDS = 212

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags, params)
    current_scene = scene
    player:play_scene(scene, flags, params or {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
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
            play(player, SCENE_ACCEPTED, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 255)
        player:finish_event()
        local stood = player.position
        player:change_territory(WAKING_SANDS, { x = stood.x, y = stood.y, z = stood.z }, player.rotation)
        return
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_event()
            player:finish_quest(EVENT_ID, results[2] or 0)
            return
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
