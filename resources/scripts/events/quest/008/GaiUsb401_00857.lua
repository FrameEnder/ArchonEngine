-- Quest: Lord of Crags (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-06) and checked against Bob's Chronofoil captures
-- 650dbfd8 (2026-11-06, the whole quest, the Navel with Duty Support: its fixture is tools/questkit/fixtures/857.json),
-- 0fd8bce0 and 48aaf6fe (the Navel by the Duty Finder: `857-0fd8bce0.json`, `857-48aaf6fe.json`):
-- Sequence 0:   ACTOR0 (in Upper La Noscea) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then the
--               cutscene (scene 1, 0xF8402EFB, where the client accepts it): sequence 1.
-- Sequence 1:   EOBJECT0 (the beastman aetheryte): its empty scene 2 (0x2001), EVENT_ACTION_PROCESS_UPPER_MIDDLE's bar
--               (5 s on retail), then the cutscene (scene 3, 0xF8406EFB, the Navel's unlock image): the Navel
--               (INSTANCEDUNGEON0, InstanceContent 20002) unlocked, notice 0, sequence 2.
-- Sequence 2:   clearing the Navel: notice 1, sequence 255 (retail sent it in the duty, as Titan fell; Kawari as the
--               player leaves it, onDutyCleared), and leaving it puts them back by the aetheryte, all three runs alike
--               (POPRANGE0's spot; one run had queued from elsewhere in the zone.) ACTOR3 (Y'shtola) has a line
--               meanwhile (scene 5; ACTOR2's, 4, is empty.)
-- Sequence 255: ACTOR4 (Y'shtola, at the aetheryte) rewards it (scene 6, a yield: QuestReward, 1.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_AETHERYTE = 2
SCENE_UNLOCK = 3
SCENE_REWARD = 6

-- Retail's flags (Bob's Chronofoil captures of 2026-11-06.)
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001
UNLOCK_FLAGS = 0xF8406EFB

-- Where retail put the player as they left the cleared Navel (the next visit of each capture: Upper La Noscea, at
-- (377.8-379.0, 23.6, -137.1 to -137.4), facing 2.023), whatever part of the zone they had queued from.
UPPER_LA_NOSCEA = 139
AFTER_THE_NAVEL = { x = 378.3, y = 23.61, z = -137.25 }
AFTER_THE_NAVEL_ROTATION = 2.023
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
REWARD_CLOSED = 0x11

LINES = {
    [2] = { [ACTOR3] = 5 },
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
    elseif sequence == 1 and BASE_ID == EOBJECT0 then
        play(player, SCENE_AETHERYTE, OBJECT_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR4 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    elseif sequence ~= nil and LINES[sequence] ~= nil and LINES[sequence][BASE_ID] ~= nil then
        play(player, LINES[sequence][BASE_ID], HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- The bar at the aetheryte finished: the cutscene.
function onEventActionCast(player)
    if current_scene ~= nil or player:get_quest_sequence(EVENT_ID) ~= 1 then
        player:finish_event()
        return
    end
    play(player, SCENE_UNLOCK, UNLOCK_FLAGS)
end

-- The Navel cleared.
function onDutyCleared(content_id, player)
    if player:get_quest_sequence(EVENT_ID) == 2 and content_id == INSTANCEDUNGEON0 then
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 255)
        player:set_duty_exit(UPPER_LA_NOSCEA, AFTER_THE_NAVEL, AFTER_THE_NAVEL_ROTATION)
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
    elseif scene == SCENE_AETHERYTE then
        -- The bar, then the cutscene (see onEventActionCast.)
        player:event_action(EVENT_ACTION_PROCESS_UPPER_MIDDLE)
        return
    elseif scene == SCENE_UNLOCK then
        player:unlock_content(INSTANCEDUNGEON0)
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
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
