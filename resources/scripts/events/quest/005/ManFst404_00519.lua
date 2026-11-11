-- Quest: Lady of the Vortex (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-08) and checked against Bob's three Chronofoil
-- captures of 2026-11-08, each the whole quest: 6a5244ec (the Howling Eye with Duty Support; fixture
-- tools/questkit/fixtures/519.json), 09ecf8e7 and 6c495fc4 (the Howling Eye by the Duty Finder: `519-09ecf8e7.json`,
-- `519-6c495fc4.json`):
-- Sequence 0:   ACTOR0 (Cid, at New Gridania's airship landing) offers it (scene 0, a yield: QuestOffer, 1 when
--               accepted), then the cutscene (scene 1, 0xF8402FFF, where the client accepts it): sequence 1, and it
--               ends at POPRANGE1 in TERRITORYTYPE1 (331: retail's zone change as it ended, all three runs, ZoneInit's
--               pop range 4313619.)
-- Sequence 1:   ACTOR2 (Alphinaud, there; offered while UI8AL is 0 and BitFlag8 1 unset) talks (scene 3, the Howling
--               Eye's unlock image and log): the Howling Eye (INSTANCEDUNGEON0, InstanceContent 20003) unlocked,
--               notice 0 [1, 1], sequence 2. Back and forth meanwhile: ACTOR1 (Lionnellais, in New Gridania) asks
--               whether to go up (scene 5, a yield: YesNo, 1 for yes: POPRANGE0 in TERRITORYTYPE0, the landing), and
--               ACTOR0 whether to go to the ship (scene 4: POPRANGE1 in 331, ZoneInit's pop range again; 6a5244ec.)
-- Sequence 2:   clearing the Howling Eye: sequence 3 (retail sent it in the duty, as Garuda fell; Kawari as the
--               player leaves it, onDutyCleared), and leaving it puts them at Ul'dah's airship landing (TERRITORYTYPE2,
--               130; ZoneInit's pop range was POPRANGE2 in all three runs.) ACTOR2 has a line meanwhile (scene 6.)
-- Sequence 3:   arriving in Ul'dah (the quest's enter-territory event): the cutscene (scene 7, 0xF8402FFF,
--               CUT_SCENE_AFTER_01/02): notice 2, sequence 255.
-- Sequence 255: ACTOR4 (Alphinaud, in Ul'dah) rewards it (scene 8, a yield: QuestReward, 1 and the item picked.)
--               ACTOR5 (Elyenora) asks whether to go down (scene 9), unseen on retail: "yes" is taken to send the
--               player to POPRANGE3 in TERRITORYTYPE2 (the same spot as Reclamation's POPRANGE0; a guess.)
-- Before it's accepted, ACTOR1 (Lionnellais, Gridania's airship ticketer, at the gate below the landing) asks whether
-- to go up (scene 2, the client's sequence 0 question): "yes" takes the player to POPRANGE0 in TERRITORYTYPE0, the
-- landing where Cid offers it, as his scene 5 does once it's accepted. Unseen on retail (every run accepted it at
-- the landing straight after Into the Eye of the Storm left them there); without it, someone who'd left the landing
-- couldn't get back to Cid (Bob's bundle 20261111-101422.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_UNLOCK = 3
SCENE_TO_THE_SHIP = 4
SCENE_LIONNELLAIS = 5
SCENE_LIONNELLAIS_BEFORE = 2
SCENE_ALPHINAUD_LINE = 6
SCENE_ARRIVAL = 7
SCENE_REWARD = 8
SCENE_ELYENORA = 9

-- Retail's flags (Bob's Chronofoil captures of 2026-11-08.)
CUTSCENE_FLAGS = 0xF8402FFF

-- Where retail put the player as they left the cleared Howling Eye (each capture's next visit: Ul'dah's airship
-- landing, POPRANGE2: first positions within 1.3 yalms of this, facing -1.91 to -2.24.)
ULDAH = TERRITORYTYPE2 or 130
AFTER_THE_HOWLING_EYE = { x = -40.55, y = 84.0, z = -1.47 }
AFTER_THE_HOWLING_EYE_ROTATION = -2.04
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855, 960.)
REWARD_CLOSED = 0x11

LINES = {
    [2] = { [ACTOR2] = SCENE_ALPHINAUD_LINE },
}

-- The yes/no questions and where "yes" goes: { scene, territory, pop range }; before it's accepted, and by sequence.
RIDES_BEFORE = {
    [ACTOR1] = { SCENE_LIONNELLAIS_BEFORE, TERRITORYTYPE0, POPRANGE0 },
}
RIDES = {
    [1] = {
        [ACTOR1] = { SCENE_LIONNELLAIS, TERRITORYTYPE0, POPRANGE0 },
        [ACTOR0] = { SCENE_TO_THE_SHIP, TERRITORYTYPE1, POPRANGE1 },
    },
    [255] = { [ACTOR5] = { SCENE_ELYENORA, TERRITORYTYPE2, POPRANGE3 } },
}

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
current_ride = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    current_ride = nil

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
        return
    elseif sequence == nil and RIDES_BEFORE[BASE_ID] ~= nil then
        current_ride = RIDES_BEFORE[BASE_ID]
        play(player, current_ride[1], HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == ACTOR2 then
        play(player, SCENE_UNLOCK, HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR4 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
        return
    elseif sequence ~= nil and RIDES[sequence] ~= nil and RIDES[sequence][BASE_ID] ~= nil then
        current_ride = RIDES[sequence][BASE_ID]
        play(player, current_ride[1], HIDE_HOTBAR)
        return
    elseif sequence ~= nil and LINES[sequence] ~= nil and LINES[sequence][BASE_ID] ~= nil then
        play(player, LINES[sequence][BASE_ID], HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

-- The Howling Eye cleared.
function onDutyCleared(content_id, player)
    if player:get_quest_sequence(EVENT_ID) == 2 and content_id == INSTANCEDUNGEON0 then
        player:quest_sequence(EVENT_ID, 3)
        player:set_duty_exit(ULDAH, AFTER_THE_HOWLING_EYE, AFTER_THE_HOWLING_EYE_ROTATION)
    end
end

-- Arriving in Ul'dah after it.
function onEnterTerritory(player)
    if player:get_quest_sequence(EVENT_ID) == 3 then
        play(player, SCENE_ARRIVAL, CUTSCENE_FLAGS)
    else
        player:finish_event()
    end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if current_ride ~= nil and scene == current_ride[1] then
        local ride = current_ride
        current_ride = nil
        if results[1] == 1 then
            player:finish_event()
            player:change_territory_pop_range(ride[2], ride[3])
            return
        end
    elseif scene == SCENE_OFFER then
        if results[1] == 1 then
            play(player, SCENE_ACCEPTED, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        -- The cutscene ends on the ship (see "Cutscenes that end somewhere else" in docs/src/fork/quests.md.)
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
        player:finish_event()
        player:change_territory_pop_range(TERRITORYTYPE1, POPRANGE1)
        return
    elseif scene == SCENE_UNLOCK then
        player:unlock_content(INSTANCEDUNGEON0)
        player:event_notice(EVENT_ID, 0, { 1, 1 })
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_ARRIVAL then
        player:event_notice(EVENT_ID, 2, {})
        player:quest_sequence(EVENT_ID, 255)
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
