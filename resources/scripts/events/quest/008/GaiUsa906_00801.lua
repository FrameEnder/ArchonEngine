-- Quest: Skeletons in Her Closet
--
-- Written from the client's quest script (kawari-questdump, 2026-11-02) and checked against Bob's Chronofoil capture
-- 5d9c7d7f (2026-11-02, the whole quest and Haukke Manor with Duty Support: its fixture is
-- tools/questkit/fixtures/801.json):
-- Sequence 0:   ACTOR0 (Ursandel) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then the talk where the
--               client accepts it (scene 1, 0xF8402EFB): sequence 1.
-- Sequence 1:   EOBJECT0 (the manor gate in TERRITORYTYPE0, the Central Shroud): scene 2 (0x2001, empty), then
--               EVENT_ACTION_TOUCH_MIDDLE's bar (retail's 5 s), then scene 3 (0x2001: the dungeon's unlock image and
--               the "new duty" log line): Haukke Manor (INSTANCEDUNGEON0) unlocked, notice 0, sequence 2.
-- Sequence 2:   clearing it: retail moved it on as the player left the cleared duty (no notice, no variables,
--               sequence 3), which the server tells the script with onDutyCleared.
-- Sequence 3:   back in TERRITORYTYPE0 (onEnterTerritory) CUT_SCENE_01 plays (scene 4, 0xF8402EFB, its zone-entry
--               event): notice 2, sequence 4.
-- Sequence 4:   ACTOR0 talks (scene 5): notice 3, sequence 255.
-- Sequence 255: ACTOR2 (Minfilia, in the Waking Sands) rewards it (scene 6, a cutscene and a yield: QuestReward, 1.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_GATE = 2
SCENE_UNLOCKED = 3
SCENE_AFTER_DUTY = 4
SCENE_URSANDEL = 5
SCENE_REWARD = 6

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB on retail
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

NOTICE_UNLOCKED = 0
NOTICE_AFTER_DUTY = 2
NOTICE_URSANDEL = 3

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
        play(player, SCENE_GATE, OBJECT_FLAGS)
    elseif sequence == 4 and BASE_ID == ACTOR0 then
        play(player, SCENE_URSANDEL, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_REWARD, CUTSCENE_FLAGS)
    else
        player:finish_event()
    end
end

-- The bar at the gate finished: the scene after it.
function onEventActionCast(player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 then
        player:finish_event()
        return
    end
    play(player, SCENE_UNLOCKED, OBJECT_FLAGS)
end

-- Leaving a duty the player cleared (its InstanceContent ID). Not an event of the quest's: nothing to finish.
function onDutyCleared(content_id, player)
    if player:get_quest_sequence(EVENT_ID) == 2 and content_id == INSTANCEDUNGEON0 then
        player:quest_sequence(EVENT_ID, 3)
    end
end

-- Back in the Central Shroud after the duty.
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
    elseif scene == SCENE_GATE then
        -- Touching the gate: the bar, then scene 3 (see onEventActionCast.)
        player:event_action(EVENT_ACTION_TOUCH_MIDDLE)
        return
    elseif scene == SCENE_UNLOCKED then
        player:unlock_content(INSTANCEDUNGEON0)
        player:event_notice(EVENT_ID, NOTICE_UNLOCKED, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_AFTER_DUTY then
        player:event_notice(EVENT_ID, NOTICE_AFTER_DUTY, {})
        player:quest_sequence(EVENT_ID, 4)
    elseif scene == SCENE_URSANDEL then
        player:event_notice(EVENT_ID, NOTICE_URSANDEL, {})
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
