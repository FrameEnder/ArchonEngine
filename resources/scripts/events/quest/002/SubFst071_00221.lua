-- Quest: Leves of Hawthorne
--
-- Checked against the client's quest script (kawari-questdump) and Bob's Chronofoil capture b9af1bb2 (2026-11-24)
-- (fixture tools/questkit/fixtures/221.json, from sequence 1: the quest was accepted before the capture began).
-- Sequence 0:   ACTOR0 (the Hawthorne Hut's levemete) offers it: scene 0 (QuestOffer: returns 1 when accepted, 0 when
--               not), then 1 and 2 (2 shows it accepted: QuestAccepted), sequence 1. Unverified: no capture has the
--               offer; the scenes' order is the listing's (its lines run 1-3, 4, then 5 and the system line 6), the
--               flags those of every talk retail sent (HIDE_HOTBAR), scene 0 a yield as other quests' offers.
-- Sequence 1:   complete a levequest. Talking to ACTOR0 meanwhile is scene 5 (the client picks its line by whether
--               the player is in a battle or gathering leve's content). As the leve was completed retail moved the
--               quest to sequence 255, no notice, in the packets that ended it (Leve 573, the quest's LEVE, its
--               director 0x8001054e): the levequest system calls onLeveCompleted.
-- Sequence 255: ACTOR0: scene 3, then scene 4 (QuestReward, QuestCompleted, the leve unlock image) completes it.
--               Retail ran ACTOR0's levemete event (GuildLeveAssignment 2, GL_ASSIGNMENT: scene 5 with [1, 573], the
--               leve's turn-in and its reward) between them; that's the levequest system's (known issues).

SCENE_OFFER = 0
SCENE_EXPLAIN = 1
SCENE_ACCEPTED = 2
SCENE_GREETING = 3
SCENE_REWARD = 4
SCENE_REMINDER = 5

-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855, 960.)
REWARD_CLOSED = 0x11

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
step = nil

function play(player, scene, flags, params)
    current_scene = scene
    player:play_scene(scene, flags, params or {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    step = nil
    if BASE_ID ~= ACTOR0 then
        player:finish_event()
    elseif sequence == nil then
        step = 0
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 then
        step = 1
        play(player, SCENE_REMINDER, HIDE_HOTBAR)
    elseif sequence == 255 then
        step = 255
        play(player, SCENE_GREETING, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- A levequest completed (its Leve row): the quest moves on, as retail's did with Leve 573.
function onLeveCompleted(leve, player)
    if player:get_quest_sequence(EVENT_ID) == 1 then
        player:quest_sequence(EVENT_ID, 255)
    end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if step == 0 and scene == SCENE_OFFER then
        if results[1] ~= 1 then
            player:finish_event()
            return
        end
        play(player, SCENE_EXPLAIN, HIDE_HOTBAR)
        return
    elseif step == 0 and scene == SCENE_EXPLAIN then
        play(player, SCENE_ACCEPTED, HIDE_HOTBAR)
        return
    elseif step == 0 and scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif step == 255 and scene == SCENE_GREETING then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
        return
    elseif step == 255 and scene == SCENE_REWARD then
        if results[1] ~= 1 then
            -- The reward window closed: the quest stays open.
            if player:get_quest_var(EVENT_ID, 0) ~= REWARD_CLOSED then
                player:set_quest_var(EVENT_ID, 0, REWARD_CLOSED)
            end
            player:finish_event()
            return
        end
        player:finish_event()
        player:finish_quest(EVENT_ID, results[2] or 0)
        return
    end

    player:finish_event()
end

function onYield(scene, id, results, player)
    sceneFinished(scene, results, player)
end

function onReturn(scene, results, player)
    sceneFinished(scene, results, player)
end
