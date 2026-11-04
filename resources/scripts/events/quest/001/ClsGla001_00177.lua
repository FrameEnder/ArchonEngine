-- Quest: So You Want to Be a Gladiator
--
-- Written from the client's quest script (kawari-questdump, 2026-11-04) and checked against Bob's Chronofoil capture
-- d3379099 (2026-11-04, on a Conjurer: its fixture is tools/questkit/fixtures/177.json): the guild's door NPC (its
-- issuer; the quest has no parameters) talks and asks (scene 0, a yield: its YesNo, 1 for yes). Yes completes it at
-- once, never accepted (retail: FinishQuest and no AcceptQuest), which lets the player take Way of the Gladiator for
-- the other classes (285). The client's scene also checks the player is a Gladiator already (GetClassLevel) and
-- whether they qualify (IsQuestAcceptQualified), and tells them itself; retail's answer was 1 when it went on.

SCENE_TALK = 0

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    if player:get_quest_sequence(EVENT_ID) == nil then
        play(player, SCENE_TALK, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if scene == SCENE_TALK and results[1] == 1 then
        player:finish_quest(EVENT_ID, 0)
    end

    player:finish_event()
end

function onYield(scene, id, results, player)
    sceneFinished(scene, results, player)
end

function onReturn(scene, results, player)
    sceneFinished(scene, results, player)
end
