-- Quest: It Won't Work
--
-- Written from the client's quest script (kawari-questdump, 2026-11-05) and checked against Bob's Chronofoil captures
-- b766138c (2026-11-05, the whole quest on a Gladiator: its fixture is tools/questkit/fixtures/843.json), 811b6f1b
-- (an Arcanist, the two talks the other way round: `843-811b6f1b.json`) and 62c1dfe4 (a White Mage,
-- `843-62c1dfe4.json`):
-- Sequence 0:   ACTOR0 (Shamani Lohmani) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1,
--               where the client accepts it: sequence 1.
-- Sequence 1:   ACTOR1 (Durim Falurim, scene 2) and ACTOR2 (Junghbhar, scene 3), in either order: the client offers
--               each while UI8AL < 2 and its BitFlag8 (1 and 2) is unset. The first: UI8AL = 1, its flag set, notice 0
--               [1, 2]; the second: notice 0 [2, 2], the variables cleared, sequence 255.
-- Sequence 255: ACTOR0 rewards it (scene 4, a yield: QuestReward, 1 and the optional item chosen; closing the window
--               leaves it open, and retail then sets UI8A to 0x11, as in An Offer You Can Refuse.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_REWARD = 4

NOTICE_TALKED = 0
TALKS_NEEDED = 2
-- What retail set UI8A to when a reward window was closed (Bob's capture b766138c, quest 840; 414 and 680.)
REWARD_CLOSED = 0x11

-- The two to talk to: each one's scene and BitFlag8.
TALKS = {
    { actor = ACTOR1, scene = 2, flag = 1 },
    { actor = ACTOR2, scene = 3, flag = 2 },
}

-- BitFlag8: flag 1 is the top bit of UI8F, as retail sets it.
function hasFlag(player, flag)
    return (player:get_quest_var(EVENT_ID, 5) & (0x80 >> (flag - 1))) ~= 0
end

function setFlag(player, flag)
    player:set_quest_var(EVENT_ID, 5, player:get_quest_var(EVENT_ID, 5) | (0x80 >> (flag - 1)))
end

-- UI8AL: how many of them have been talked to.
function talked(player)
    return player:get_quest_var(EVENT_ID, 0) & 0x0F
end

function talkOf(base_id)
    for _, talk in ipairs(TALKS) do
        if talk.actor == base_id then
            return talk
        end
    end
    return nil
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- The one being talked to in this event.
talking = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local talk = talkOf(BASE_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and talk ~= nil and talked(player) < TALKS_NEEDED and not hasFlag(player, talk.flag) then
        talking = talk
        play(player, talk.scene, HIDE_HOTBAR)
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
    elseif talking ~= nil and scene == talking.scene then
        local count = talked(player) + 1
        player:event_notice(EVENT_ID, NOTICE_TALKED, { count, TALKS_NEEDED })
        if count >= TALKS_NEEDED then
            player:set_quest_var(EVENT_ID, 0, 0)
            player:set_quest_var(EVENT_ID, 5, 0)
            player:quest_sequence(EVENT_ID, 255)
        else
            player:set_quest_var(EVENT_ID, 0, count)
            setFlag(player, talking.flag)
        end
        talking = nil
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
