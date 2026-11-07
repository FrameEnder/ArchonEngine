-- Quest: In Pursuit of the Past (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-07) and checked against Bob's two Chronofoil captures
-- of 2026-11-07, both the whole quest: 1cefedf4 (the Stone Vigil with Duty Support; fixture
-- tools/questkit/fixtures/952.json) and 341808b0 (a tank, through the Duty Finder; 952-341808b0.json):
-- Sequence 0:   ACTOR0 (Alphinaud, in Coerthas Central Highlands) offers it (scene 0, a yield: QuestOffer, 1 when
--               accepted), then his line (scene 1, where the client accepts it): sequence 1.
-- Sequence 1:   ACTOR1 (Drillemont)'s cutscene (scene 2, 0xF8402EFB): notice 0, sequence 2.
-- Sequence 2:   ACTOR2 (Nathelain) (scene 3, which shows the Stone Vigil's unlock image and log): the Stone Vigil
--               (INSTANCEDUNGEON0, InstanceContent 11) unlocked, notice 1, sequence 3.
-- Sequence 3:   clearing the Stone Vigil: sequence 4 with no notice. Retail sent it as the player used the exit after the
--               clear (the director's log 1534 `[11]`, then the update, then the zone change), which is when Kawari's
--               onDutyCleared comes; and that exit took the player to New Gridania (TERRITORYTYPE0) by the quest, not
--               back where they had queued: both runs' first position there was within 0.4 yalms of (10.33, -21.97,
--               120.2), facing -1.51 and -1.54 (POPRANGE0's spot, taken to be). ACTOR2 has a line meanwhile (6);
--               the client's scenes 4 and 5 are empty.
-- Sequence 4:   arriving in New Gridania (TERRITORYTYPE0, the quest's enter-territory event while BitFlag8 1 is unset):
--               the cutscene (scene 7, 0xF8402EFB, CUT_SCENE_01): notice 3, sequence 255.
-- Sequence 255: ACTOR4 (Alphinaud, in New Gridania) rewards it (scene 8, a yield: QuestReward, 1.) ACTOR5
--               (Lionnellais) asks a yes/no (scene 9), unseen on retail: "yes" is taken to send the player to
--               POPRANGE1 in New Gridania (a guess.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_DRILLEMONT = 2
SCENE_UNLOCK = 3
SCENE_NATHELAIN_LINE = 6
SCENE_ARRIVAL = 7
SCENE_REWARD = 8
SCENE_LIONNELLAIS = 9

-- Retail's flags (Bob's Chronofoil capture 1cefedf4.)
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB

-- Where retail put the player as they left the cleared Stone Vigil (both captures' next visit: New Gridania; the
-- facing is the pop range's, taken to be, within 0.07 of both runs' first position updates.)
NEW_GRIDANIA = TERRITORYTYPE0 or 132
AFTER_THE_STONE_VIGIL = { x = 10.33, y = -21.97, z = 120.2 }
AFTER_THE_STONE_VIGIL_ROTATION = -1.578
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
REWARD_CLOSED = 0x11

LINES = {
    [3] = { [ACTOR2] = SCENE_NATHELAIN_LINE },
}

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
        play(player, SCENE_DRILLEMONT, CUTSCENE_FLAGS)
    elseif sequence == 2 and BASE_ID == ACTOR2 then
        play(player, SCENE_UNLOCK, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR4 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR5 then
        play(player, SCENE_LIONNELLAIS, HIDE_HOTBAR)
    elseif sequence ~= nil and LINES[sequence] ~= nil and LINES[sequence][BASE_ID] ~= nil then
        play(player, LINES[sequence][BASE_ID], HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- The Stone Vigil cleared.
function onDutyCleared(content_id, player)
    if player:get_quest_sequence(EVENT_ID) == 3 and content_id == INSTANCEDUNGEON0 then
        player:quest_sequence(EVENT_ID, 4)
        player:set_duty_exit(NEW_GRIDANIA, AFTER_THE_STONE_VIGIL, AFTER_THE_STONE_VIGIL_ROTATION)
    end
end

-- Arriving in New Gridania after it.
function onEnterTerritory(player)
    if player:get_quest_sequence(EVENT_ID) == 4 then
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

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            play(player, SCENE_ACCEPTED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_DRILLEMONT then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_UNLOCK then
        player:unlock_content(INSTANCEDUNGEON0)
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_ARRIVAL then
        player:event_notice(EVENT_ID, 3, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_LIONNELLAIS then
        if results[1] == 1 and POPRANGE1 ~= nil then
            player:finish_event()
            player:change_territory_pop_range(NEW_GRIDANIA, POPRANGE1)
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
