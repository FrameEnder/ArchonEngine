-- Quest: The Company You Keep (Immortal Flames) (main scenario)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-27) and Bob's Chronofoil capture c99fe374
-- (2026-10-28, the whole quest: its quest battle won; its fixture is tools/questkit/fixtures/682.json). The Twin
-- Adders' 680 with the Flames' people.
-- Sequence 0:   ACTOR0 (the flame officer, in the Waking Sands) offers it (scene 0, a yield: the client asks first and
--               accepts it there, returning 1): sequence 1.
-- Sequence 1:   ACTOR1 (the flame personnel officer, in Ul'dah - Steps of Nald, scene 1, retail's flags 0xF8482EFB):
--               notice 0, sequence 2.
-- Sequence 2:   ACTOR4 (the airship crewman, in Western Thanalan, scene 5) or EVENTRANGE0 (scene 13): YesNoQuestBattle;
--               1 enters QUESTBATTLE0 (47) in TERRITORYTYPE0 (278), and it can be tried again after a loss. Winning it
--               moves to sequence 255 (content/questbattle/47.lua.)
-- Sequence 255: ACTOR1 rewards it (scene 18, a yield returning 1 when the reward is taken; retail completed it at once.)
-- The others' lines go by the client's IsAcceptEvent order: ACTOR2, ACTOR3 and ACTOR0 (scenes 2-4) in sequence 1;
-- ACTOR1, ACTOR0, ACTOR2 and ACTOR3 (9-12) in sequence 2, after ACTOR5-7's empty 6-8; ACTOR0, ACTOR2, ACTOR3 (19-21) in
-- sequence 255. Its other scenes (EOBJECT0's and EOBJECT1's 14, 15; 16, 17) are empty.

SCENE_OFFER = 0
SCENE_PERSONNEL = 1
SCENE_CREWMAN_BATTLE = 5
SCENE_RANGE_BATTLE = 13
SCENE_REWARD = 18

-- Retail's flags (Bob's Chronofoil capture c99fe374.)
PERSONNEL_FLAGS = 0xF8482EFB

-- The others' lines in each sequence: { [sequence] = { [base ID] = scene } }.
LINES = {
    [1] = { [ACTOR2] = 2, [ACTOR3] = 3, [ACTOR0] = 4 },
    [2] = { [ACTOR1] = 9, [ACTOR0] = 10, [ACTOR2] = 11, [ACTOR3] = 12 },
    [255] = { [ACTOR0] = 19, [ACTOR2] = 20, [ACTOR3] = 21 },
}

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags, params)
    current_scene = scene
    player:play_scene(scene, flags, params or {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local line = sequence ~= nil and LINES[sequence] ~= nil and LINES[sequence][BASE_ID] or nil

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_PERSONNEL, PERSONNEL_FLAGS)
    elseif sequence == 2 and BASE_ID == ACTOR4 then
        play(player, SCENE_CREWMAN_BATTLE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    elseif line ~= nil then
        play(player, line, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- Walking into EVENTRANGE0 offers the battle too.
function onEnterTrigger(player, arg)
    if player:get_quest_sequence(EVENT_ID) == 2 then
        play(player, SCENE_RANGE_BATTLE, HIDE_HOTBAR)
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
            player:accept_quest(EVENT_ID)
            player:quest_sequence(EVENT_ID, 1)
        end
    elseif scene == SCENE_PERSONNEL then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_CREWMAN_BATTLE or scene == SCENE_RANGE_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
            return
        end
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
