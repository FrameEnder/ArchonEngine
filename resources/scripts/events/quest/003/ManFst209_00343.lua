-- Quest: Lord of the Inferno (main scenario)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-27) and Bob's Chronofoil capture b8432613
-- (2026-10-27, the whole quest: four runs of its quest battle, three lost, and the Bowl of Embers; its fixture is
-- tools/questkit/fixtures/343.json).
-- Sequence 0:   ACTOR0 (Minfilia) offers it (scene 0, a yield returning 1 when accepted), then scene 1 (the client
--               accepts it there): sequence 1.
-- Sequence 1:   ACTOR1 (the flame sergeant, in Eastern Thanalan, scene 2): notice 0 [1, 1], sequence 2.
-- Sequence 2:   EOBJECT0, the destination (scene 7, then EVENT_ACTION_WAITING_SHOR's bar, then scene 8:
--               YesNoQuestBattle, all 0x2001; 1 enters QUESTBATTLE0 in TERRITORYTYPE0, and it can be tried again after
--               a loss). Winning it moves to sequence 3 (content/questbattle/45.lua.)
-- Sequence 3:   ACTOR6 (the flame sergeant, in Southern Thanalan, scene 14: the Bowl of Embers' unlock image): the Bowl
--               of Embers (INSTANCEDUNGEON0, InstanceContent 20001) is unlocked, notice 2 [1, 1], sequence 4.
-- Sequence 4:   clearing the Bowl of Embers: notice 3 [1, 1], sequence 5 (retail sent it in the duty, as it was won;
--               Kawari as the player leaves it, onDutyCleared.)
-- Sequence 5:   ACTOR20 (Thancred, scene 40, 0x6001): notice 4 [1, 1], sequence 255.
-- Sequence 255: ACTOR31 (the Scion of the Seventh Dawn, in the Waking Sands) rewards it (scene 51, a yield returning
--               1), then the cutscene (scene 52, CUT_MANFST20950, retail's flags 0xF8402EFB, parameter 0; retail set
--               UI8AH and UI8AL to 1 as it began), and once it ends the quest is completed and the player put where
--               the cutscene left them in the Waking Sands (retail's ActorSetPos; no pop range for it.)
-- The others' lines are matched by the order in which the client's IsAcceptEvent offers each sequence's NPCs (retail
-- played none of them); its other scenes are empty.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_SERGEANT = 2
SCENE_DESTINATION = 7
SCENE_ENTER_BATTLE = 8
SCENE_SERGEANT_AFTER = 14
SCENE_THANCRED = 40
SCENE_REWARD = 51
SCENE_CUTSCENE = 52

-- Retail's flags (Bob's Chronofoil capture b8432613.)
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001
THANCRED_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA | INVINCIBLE -- 0x6001
CUTSCENE_FLAGS = 0xF8402EFB

-- Each todo's notice carried [1, 1] on retail.
TODO_DONE = { 1, 1 }

-- Where retail put the player after the cutscene (its ActorSetPos, warp type 1), in the Waking Sands.
WAKING_SANDS = 212
AFTER_CUTSCENE = { x = 31.131683, y = 1.0000019, z = -0.13700001 }
AFTER_CUTSCENE_ROTATION = 1.5706

-- The others' lines in each sequence: { [sequence] = { [base ID] = scene } }.
LINES = {
    [2] = { [ACTOR1] = 9 },
    [3] = { [ACTOR7] = 15, [ACTOR9] = 17, [ACTOR10] = 18, [ACTOR18] = 26 },
    [4] = { [ACTOR7] = 27, [ACTOR9] = 29, [ACTOR10] = 30, [ACTOR6] = 32, [ACTOR18] = 39 },
}

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- The reward chosen in scene 51 (none: the Quest sheet's are fixed), kept until the cutscene after it ends.
reward = 0

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
        play(player, SCENE_SERGEANT, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == EOBJECT0 then
        -- The destination's scene, the bar (see sceneFinished), then the prompt once that finishes.
        play(player, SCENE_DESTINATION, OBJECT_FLAGS)
    elseif sequence == 3 and BASE_ID == ACTOR6 then
        play(player, SCENE_SERGEANT_AFTER, HIDE_HOTBAR)
    elseif sequence == 5 and BASE_ID == ACTOR20 then
        play(player, SCENE_THANCRED, THANCRED_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR31 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    elseif line ~= nil then
        play(player, line, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    if player:get_quest_sequence(EVENT_ID) ~= 2 then
        player:finish_event()
        return
    end
    play(player, SCENE_ENTER_BATTLE, OBJECT_FLAGS)
end

-- The Bowl of Embers cleared.
function onDutyCleared(content_id, player)
    if player:get_quest_sequence(EVENT_ID) == 4 and content_id == INSTANCEDUNGEON0 then
        player:event_notice(EVENT_ID, 3, TODO_DONE)
        player:quest_sequence(EVENT_ID, 5)
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
    elseif scene == SCENE_SERGEANT then
        player:event_notice(EVENT_ID, 0, TODO_DONE)
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_DESTINATION then
        -- A short wait shows a progress bar; the prompt plays once it finishes (see onEventActionCast.)
        player:event_action(EVENT_ACTION_WAITING_SHOR)
        return
    elseif scene == SCENE_ENTER_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
            return
        end
    elseif scene == SCENE_SERGEANT_AFTER then
        player:unlock_content(INSTANCEDUNGEON0)
        player:event_notice(EVENT_ID, 2, TODO_DONE)
        player:quest_sequence(EVENT_ID, 4)
    elseif scene == SCENE_THANCRED then
        player:event_notice(EVENT_ID, 4, TODO_DONE)
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            reward = results[2] or 0
            play(player, SCENE_CUTSCENE, CUTSCENE_FLAGS, { 0 })
            player:set_quest_var(EVENT_ID, 0, 0x11) -- UI8AH and UI8AL, as retail set them
            return
        end
    elseif scene == SCENE_CUTSCENE then
        player:finish_event()
        player:finish_quest(EVENT_ID, reward)
        player:change_territory(WAKING_SANDS, AFTER_CUTSCENE, AFTER_CUTSCENE_ROTATION)
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
