-- Quest: Sealed with Science (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-08) and checked against Bob's three Chronofoil
-- captures of 2026-11-08 (18a89a32, 4bcbd425 and de5beba7, each the whole quest; fixtures
-- tools/questkit/fixtures/955.json, 955-18a89a32.json, 955-de5beba7.json):
-- Sequence 0:   ACTOR0 (Lamberteint) offers it (scene 0, a yield: QuestOffer, 1 when accepted; the client turns it down
--               itself while QST_ACCEPT_CHECK, All Due Precautions (954), is underway), then scene 1, where the client
--               accepts it: sequence 255 at once, with UI8BH = 1 (ITEM0 held; the client lists it from there.)
-- Sequence 255: ACTOR1 (Hahasako) takes ITEM0 (scene 2, NpcTrade in event mode: a yield, 1 when handed over) and
--               rewards it (scene 3, a yield: QuestReward, 1.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_TRADE = 2
SCENE_REWARD = 3

-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
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
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        -- The trade first, every time (ITEM0 stays held until the quest completes.)
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
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH: ITEM0 held
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_REWARD, HIDE_HOTBAR)
            return
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
