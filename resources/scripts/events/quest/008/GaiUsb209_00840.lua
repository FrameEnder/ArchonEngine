-- Quest: An Offer You Can Refuse
--
-- Written from the client's quest script (kawari-questdump, 2026-11-05) and checked against Bob's Chronofoil captures
-- b766138c (2026-11-05, the whole quest on a Gladiator: its fixture is tools/questkit/fixtures/840.json), 811b6f1b
-- (an Arcanist, `840-811b6f1b.json`) and 62c1dfe4 (a White Mage, `840-62c1dfe4.json`):
-- Sequence 0:   ACTOR0 (Shamani Lohmani) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1,
--               where the client accepts it: sequence 1.
-- Sequence 1:   ACTOR1 (Byrglaent) talks (scene 2): notice 0, sequence 255.
-- Sequence 255: ACTOR0 rewards it (scene 3, a yield: QuestReward, 1 and the optional item chosen.) Closing the reward
--               window instead (0) leaves the quest open, and retail then set UI8AH and UI8AL to 1 (0x11), as it
--               did in Victory in Peril (414) and The Company You Keep (680); talking to her again plays the reward
--               again.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_BYRGLAENT = 2
SCENE_REWARD = 3

NOTICE_BYRGLAENT = 0
-- What retail set UI8A to when the reward window was closed (Bob's capture b766138c, as in 414 and 680.)
REWARD_CLOSED = 0x11

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
        play(player, SCENE_BYRGLAENT, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
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
            play(player, SCENE_ACCEPTED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_BYRGLAENT then
        player:event_notice(EVENT_ID, NOTICE_BYRGLAENT, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID, results[2])
        elseif player:get_quest_var(EVENT_ID, 0) ~= REWARD_CLOSED then
            player:set_quest_var(EVENT_ID, 0, REWARD_CLOSED)
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
