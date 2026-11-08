-- Quest: Sweet Somethings
--
-- Written from the client's quest script (kawari-questdump, 2026-11-08) and checked against Bob's three Chronofoil
-- captures of 2026-11-08 (d7993cf0, 195176f8 and 0d10bf45, each the whole quest; fixtures
-- tools/questkit/fixtures/963.json, 963-195176f8.json, 963-0d10bf45.json):
-- Sequence 0:   ACTOR0 (Davyd) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1, where the
--               client accepts it: sequence 1.
-- Sequence 1:   three to ask, in any order (the client's IsAcceptEvent offers each while UI8AL is under 3 and its flag
--               is unset):
--                 ACTOR1 (Erapi Taropi), scene 2: BitFlag8 1
--                 ACTOR2 (Louis), scene 3: BitFlag8 2
--                 ACTOR3 (Ahlduwil), scene 4: BitFlag8 3
--               UI8AL counts them, and each sends notice 0 [count, 3]. After the third retail cleared the variables
--               and moved to sequence 255 (all three runs asked Ahlduwil, Erapi Taropi, then Louis.)
-- Sequence 255: ACTOR0 rewards it (scene 5, a yield: QuestReward, 1.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_REWARD = 5

NOTICE_ASKED = 0
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855, 960.)
REWARD_CLOSED = 0x11

-- Those to ask in sequence 1: their scene and their flag.
TALKS = {
    { actor = ACTOR1, scene = 2, flag = 1 },
    { actor = ACTOR2, scene = 3, flag = 2 },
    { actor = ACTOR3, scene = 4, flag = 3 },
}

-- BitFlag8: flag 1 is the top bit of UI8F, as retail sets it.
function hasFlag(player, flag)
    return (player:get_quest_var(EVENT_ID, 5) & (0x80 >> (flag - 1))) ~= 0
end

function setFlag(player, flag)
    player:set_quest_var(EVENT_ID, 5, player:get_quest_var(EVENT_ID, 5) | (0x80 >> (flag - 1)))
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
current_talk = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    current_talk = nil

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
        return
    elseif sequence == 1 then
        for _, talk in ipairs(TALKS) do
            if BASE_ID == talk.actor and not hasFlag(player, talk.flag) then
                current_talk = talk
                play(player, talk.scene, HIDE_HOTBAR)
                return
            end
        end
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
        return
    end
    player:finish_event()
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
    elseif current_talk ~= nil and scene == current_talk.scene then
        local talk = current_talk
        current_talk = nil
        local done = (player:get_quest_var(EVENT_ID, 0) & 0x0F) + 1
        player:event_notice(EVENT_ID, NOTICE_ASKED, { done, #TALKS })
        if done >= #TALKS then
            for byte = 0, 5 do
                player:set_quest_var(EVENT_ID, byte, 0)
            end
            player:quest_sequence(EVENT_ID, 255)
        else
            player:set_quest_var(EVENT_ID, 0, (player:get_quest_var(EVENT_ID, 0) & 0xF0) | done) -- UI8AL
            setFlag(player, talk.flag)
        end
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_event()
            player:finish_quest(EVENT_ID, results[2] or 0)
            return
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
