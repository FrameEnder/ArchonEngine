-- Quest: World of Wonders
--
-- Written from the client's quest script (kawari-questdump, 2026-10-29) and its listeners, and checked against Bob's
-- Chronofoil capture deaf7e28 (2026-10-29, the whole quest: its fixture is tools/questkit/fixtures/435.json). A tour of
-- the Gold Saucer, one attraction a sequence; every scene after the offer is a cutscene (0xF8402EF3) ending in a
-- notice and the next sequence, the variables all 0.
-- Sequence 0:   ACTOR0 (the airship ticketer) offers it (scene 0, a yield returning 1 when accepted), then scene 1
--               (QuestAccepted): sequence 1.
-- Sequence 1:   ACTOR1 (a Gold Saucer attendant) (scene 2): notice 0, sequence 2.
-- Sequence 2:   ACTOR2 (another attendant) (scene 4, 0x2000): notice 1, sequence 3.
-- Sequence 3:   ACTOR3 (the Triple Triad trader) (scene 5): notice 2, sequence 4.
-- Sequence 4:   ACTOR4 (a GATE Keeper) (scene 6): notice 3, sequence 5.
-- Sequence 5:   ACTOR5 (Veleda) (scene 7): notice 4, sequence 6.
-- Sequence 6:   ACTOR6 (a GATE Keeper) (scene 8): notice 5, sequence 255.
-- Sequence 255: ACTOR1 rewards it (scene 9, a yield returning 1: five of item 10085.)
-- ACTOR0's line after accepting is scene 3.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_ATTENDANT = 2
SCENE_ACTOR0_LINE = 3
SCENE_SECOND_ATTENDANT = 4
SCENE_CARD_TRADER = 5
SCENE_GATE_KEEPER = 6
SCENE_VELEDA = 7
SCENE_SECOND_GATE_KEEPER = 8
SCENE_REWARD = 9

TOUR_FLAGS = 0xF8402EF3 -- retail's

-- Each sequence's stop: who, the scene, its flags and the notice; the next sequence follows.
STOPS = {
    [1] = { actor = ACTOR1, scene = SCENE_ATTENDANT, flags = TOUR_FLAGS, notice = 0, next = 2 },
    [2] = { actor = ACTOR2, scene = SCENE_SECOND_ATTENDANT, flags = HIDE_HOTBAR, notice = 1, next = 3 },
    [3] = { actor = ACTOR3, scene = SCENE_CARD_TRADER, flags = TOUR_FLAGS, notice = 2, next = 4 },
    [4] = { actor = ACTOR4, scene = SCENE_GATE_KEEPER, flags = TOUR_FLAGS, notice = 3, next = 5 },
    [5] = { actor = ACTOR5, scene = SCENE_VELEDA, flags = TOUR_FLAGS, notice = 4, next = 6 },
    [6] = { actor = ACTOR6, scene = SCENE_SECOND_GATE_KEEPER, flags = TOUR_FLAGS, notice = 5, next = 255 },
}

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
current_stop = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags or HIDE_HOTBAR, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local stop = sequence ~= nil and STOPS[sequence] or nil
    current_stop = nil

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER)
    elseif stop ~= nil and BASE_ID == stop.actor then
        current_stop = stop
        play(player, stop.scene, stop.flags)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_REWARD, TOUR_FLAGS)
    elseif sequence ~= nil and BASE_ID == ACTOR0 then
        play(player, SCENE_ACTOR0_LINE)
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
            play(player, SCENE_ACCEPTED)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif current_stop ~= nil and scene == current_stop.scene then
        player:event_notice(EVENT_ID, current_stop.notice, {})
        player:quest_sequence(EVENT_ID, current_stop.next)
        current_stop = nil
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID, 0)
            player:finish_event()
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
