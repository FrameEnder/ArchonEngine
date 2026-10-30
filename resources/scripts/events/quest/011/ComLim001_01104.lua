-- Quest: A Pup No Longer (Maelstrom)
--
-- Written from the client's quest script (kawari-questdump, 2026-10-30) and checked against Bob's Chronofoil capture
-- 9df4ff01 (2026-10-30, the whole quest: its fixture is tools/questkit/fixtures/1104.json):
-- Sequence 0:   ACTOR0 (R'ashaht Rhiki) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1, where
--               the client accepts it: sequence 255.
-- Sequence 255: ACTOR1 (Berkoeya Loetahlsyn): scene 2 (empty in the client's script), then scene 3 (her talk), then
--               the reward (scene 4, a yield: QuestReward, 1; it shows SCREENIMAGE0 and 1 and the HowTo), where
--               retail also unlocked a duty (ActorControlSelf 131, InstanceContent 40001) with the reward.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_EMPTY = 2
SCENE_TALK = 3
SCENE_REWARD = 4

-- The duty retail unlocked as the quest was completed (ActorControlSelf 131's InstanceContent in Bob's Chronofoil
-- capture 9df4ff01; the client's scene 4 says so with UNLOCK_ADD_NEW_CONTENT_TO_CF.)
UNLOCKED_CONTENT = 40001

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
        play(player, SCENE_EMPTY, HIDE_HOTBAR)
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
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_EMPTY then
        play(player, SCENE_TALK, HIDE_HOTBAR)
        return
    elseif scene == SCENE_TALK then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
        return
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
            player:unlock_content(UNLOCKED_CONTENT)
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
