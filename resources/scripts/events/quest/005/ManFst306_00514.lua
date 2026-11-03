-- Quest: Into the Beast's Maw
--
-- Written from the client's quest script (kawari-questdump, 2026-10-30) and checked against Bob's Chronofoil capture
-- caa5e908 (2026-10-30, the whole quest and the Thousand Maws of Toto-Rak: its fixture is
-- tools/questkit/fixtures/514.json):
-- Sequence 0:   ACTOR0 (Buscarron) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then the cutscene
--               (scene 1, 0xF8402EFB, which ends with QuestAccepted): sequence 1.
-- Sequence 1:   ACTOR1 (Bloisirant) talks (scene 2, which shows the dungeon's unlock image and the "new duty" log
--               line): the Thousand Maws of Toto-Rak (INSTANCEDUNGEON0) are unlocked, notice 0, sequence 2.
-- Sequence 2:   clearing it: retail moved it on as the player left the cleared duty (no notice, no variables,
--               sequence 3), which the server tells the script with onDutyCleared, and sent them to the dungeon's
--               entrance in TERRITORYTYPE0 (POPRANGE0, pop range 4332862), not back where they entered from: both
--               runs went in from the Quarrymill (40, 8, -10) and came out there (Bob's Chronofoil captures caa5e908
--               and 1fac2416, visit 0.16's ZoneInit), as Skeletons in Her Closet's clear does from anywhere.
-- Sequence 3:   back in TERRITORYTYPE0 (153, the South Shroud; onEnterTerritory) the cutscene plays (scene 4,
--               0xF8402EFB, its zone-entry event): notice 2, sequence 255.
-- Sequence 255: ACTOR0 rewards it (scene 5, a yield: QuestReward, 1 and the chosen item.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_BLOISIRANT = 2
SCENE_AFTER_DUTY = 4
SCENE_REWARD = 5

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB on retail

NOTICE_BLOISIRANT = 0
NOTICE_AFTER_DUTY = 2

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
        play(player, SCENE_BLOISIRANT, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- Leaving a duty the player cleared (its InstanceContent ID). Not an event of the quest's: nothing to finish.
-- Where leaving the cleared duty puts the player (POPRANGE0): the two captures' ZoneInit and the player's own spawn
-- there (rotation 14 of 65535.)
MAWS_ENTRANCE = { x = -87.64, y = 3.30, z = -167.38 }
MAWS_ENTRANCE_ROTATION = -3.1403

function onDutyCleared(content_id, player)
    if player:get_quest_sequence(EVENT_ID) == 2 and content_id == INSTANCEDUNGEON0 then
        player:quest_sequence(EVENT_ID, 3)
        -- At the entrance, where the client asks for the cutscene (onEnterTerritory.)
        player:set_duty_exit(TERRITORYTYPE0, MAWS_ENTRANCE, MAWS_ENTRANCE_ROTATION)
    end
end

-- Back in the South Shroud after the duty.
function onEnterTerritory(player)
    if player:get_quest_sequence(EVENT_ID) == 3 then
        play(player, SCENE_AFTER_DUTY, CUTSCENE_FLAGS)
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
            play(player, SCENE_ACCEPTED, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_BLOISIRANT then
        player:unlock_content(INSTANCEDUNGEON0)
        player:event_notice(EVENT_ID, NOTICE_BLOISIRANT, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_AFTER_DUTY then
        player:event_notice(EVENT_ID, NOTICE_AFTER_DUTY, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID, results[2])
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
