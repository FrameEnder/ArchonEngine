-- Quest: My Little Chocobo (Twin Adder)
--
-- Written from the client's quest script (kawari-questdump, 2026-10-29) and checked against Bob's Chronofoil capture 0845b9a2 (2026-10-29, the whole quest: its fixture is tools/questkit/fixtures/700.json).
-- The three companies' My Little Chocobo (700 the Twin Adders', 701 the Maelstrom's, 702 the Immortal Flames') are the
-- same script with each company's people, Chocobo Issuance (RITEM0), chocobo (ACTOR2) and cutscene (NCUT0).
-- Sequence 0:   ACTOR0 (Vorsaile Heuloix, in New Gridania) offers it (scene 0, a yield returning 1 when accepted): sequence 1.
-- Sequence 1:   ACTOR1 (Cingur, at the Gridanian stables) takes RITEM0 (scene 1: NpcTrade, a yield returning 1 once it's handed
--               over; the Issuance costs 200 of the company's seals at its quartermaster), then the cutscene (scene
--               2, 0xF8402FFF); as it ends RITEM0 is taken, notice 0, sequence 2.
-- Sequence 2:   ACTOR2 (the chocobo) asks for its name (scene 4: NameMyChocobo; the client yields with the text, yield
--               231, EventYieldText16/24), which the server takes (`name_companion`: CompanionName, then the yield
--               resumed with 1); as the scene ends, notice 1, sequence 255.
-- Sequence 255: ACTOR1 rewards it (scene 6, a yield returning 1: the Chocobo Whistle, 6001, whose use teaches the
--               company chocobo mount.)
-- ACTOR0's line after it's accepted is scene 3 (sequences 1, 2 and 255), ACTOR1's in sequence 2 scene 5.

SCENE_OFFER = 0
SCENE_TRADE = 1
SCENE_CUTSCENE = 2
SCENE_ACTOR0_LINE = 3
SCENE_NAME = 4
SCENE_ACTOR1_LINE = 5
SCENE_REWARD = 6

-- Retail's flags: every scene 0x2000 but the cutscene's.
CUTSCENE_FLAGS = 0xF8402FFF

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags or HIDE_HOTBAR, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_TRADE)
    elseif sequence == 2 and BASE_ID == ACTOR2 then
        play(player, SCENE_NAME)
    elseif sequence == 2 and BASE_ID == ACTOR1 then
        play(player, SCENE_ACTOR1_LINE)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_REWARD)
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
            player:accept_quest(EVENT_ID)
            player:quest_sequence(EVENT_ID, 1)
        end
    elseif scene == SCENE_TRADE then
        -- 1 once the Issuance was handed over, nothing when the trade window was closed.
        if results[1] == 1 then
            play(player, SCENE_CUTSCENE, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_CUTSCENE then
        player:take_handed_in_items()
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_NAME then
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_event()
            player:finish_quest(EVENT_ID, 0)
            return
        end
    end

    player:finish_event()
end

function onYield(scene, id, results, player)
    -- The chocobo's name, typed into the prompt (an event text answer): taken, and the prompt answered.
    if scene == SCENE_NAME and scene == current_scene then
        player:name_companion(scene, id)
        return
    end
    sceneFinished(scene, results, player)
end

function onReturn(scene, results, player)
    sceneFinished(scene, results, player)
end
