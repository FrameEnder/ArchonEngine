-- Quest: Drowning Out the Voices
--
-- Written from the client's quest script (kawari-questdump, 2026-11-09) and checked against Connor's Chronofoil
-- capture e834e436 (2026-11-07/09; fixture tools/questkit/fixtures/1001.json):
-- Sequence 0:   ACTOR0 (Cid) offers it (scene 0, a yield: 1 when accepted), then scene 1: sequence 1, and the key
--               item ITEM0 (UI8DH 1, the client's GetEventItems in sequence 1).
-- Sequence 1:   ITEM0 used on the five prime locations, in any order (IsEventItemUsable takes each in sequence 1).
--               Three take it at once (PlayQuestGimmickReaction): EOBJECT0 scene 3, EOBJECT1 scene 5 (UI8BH 1),
--               EOBJECT4 scene 11 (UI8BL 1). Two bring enemies out (IsBattleNpcTriggerOwner, ScenarioMessage):
--               EOBJECT2 scene 7 brings ENEMY0-2, counted in UI8CH, and EOBJECT3 scene 9 brings ENEMY3 (UI8CL 1 when
--               it falls). Each done location sets its BitFlag8 (the EOBJECT's number + 1: retail set 2, 3, 4 and 5
--               for EOBJECT1-4), counts in UI8AH (IsTodoChecked: done at 5) and sends notice 0 [count, 5]. The fifth
--               (EOBJECT0 on retail) clears the variables, puts the item in UI8BH (GetEventItems at 255) and moves to
--               sequence 255.
-- Sequence 255: ACTOR0's scene 12 (a yield), then the reward (scene 13, a yield: QuestReward, 1 and the item picked.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_BEFORE_REWARD = 12
SCENE_REWARD = 13

NEEDED = 5
NOTICE_DONE = 0
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855, 960.)
REWARD_CLOSED = 0x11

-- The locations: their scene, their BitFlag8, the nibble they set (byte, high) and the enemies they bring out.
LOCATIONS = {
    [EOBJECT0] = { scene = 3, flag = 1 },
    [EOBJECT1] = { scene = 5, flag = 2, nibble = { 1, true } },
    [EOBJECT2] = { scene = 7, flag = 3, enemies = { ENEMY2, ENEMY1, ENEMY0 } },
    [EOBJECT3] = { scene = 9, flag = 4, enemies = { ENEMY3 } },
    [EOBJECT4] = { scene = 11, flag = 5, nibble = { 1, false } },
}

-- Quest variables: UI8A..F are bytes 0..5, H is the high nibble and L the low one.
function getVar(player, byte, high)
    local value = player:get_quest_var(EVENT_ID, byte)
    if high then
        return value >> 4
    end
    return value & 0x0F
end

function setVar(player, byte, high, nibble)
    local value = player:get_quest_var(EVENT_ID, byte)
    if high then
        value = (value & 0x0F) | (nibble << 4)
    else
        value = (value & 0xF0) | nibble
    end
    player:set_quest_var(EVENT_ID, byte, value)
end

-- BitFlag8: flag 1 is the top bit of UI8F, as retail sets it.
function hasFlag(player, flag)
    return (player:get_quest_var(EVENT_ID, 5) & (0x80 >> (flag - 1))) ~= 0
end

function setFlag(player, flag)
    player:set_quest_var(EVENT_ID, 5, player:get_quest_var(EVENT_ID, 5) | (0x80 >> (flag - 1)))
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
current_location = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

-- A location done: its flag, the count, notice 0, and the fifth moves the quest on.
function locationDone(player, location)
    local done = getVar(player, 0, true) + 1
    player:event_notice(EVENT_ID, NOTICE_DONE, { done, NEEDED })
    if done >= NEEDED then
        for byte = 0, 5 do
            player:set_quest_var(EVENT_ID, byte, 0)
        end
        setVar(player, 1, true, 1) -- the key item, where GetEventItems reads it at 255
        player:quest_sequence(EVENT_ID, 255)
        return
    end
    setVar(player, 0, true, done)
    if location.nibble ~= nil then
        setVar(player, location.nibble[1], location.nibble[2], 1)
    end
    setFlag(player, location.flag)
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    current_location = nil
    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_BEFORE_REWARD, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEventItem(item, target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local location = LOCATIONS[BASE_ID]
    current_location = nil
    if sequence == 1 and item == ITEM0 and location ~= nil and not hasFlag(player, location.flag) then
        current_location = location
        play(player, location.scene, HIDE_HOTBAR | NO_DEFAULT_CAMERA)
        return
    end
    player:finish_event()
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 then
        return
    end
    if layout_id == ENEMY3 then
        setVar(player, 2, false, 1) -- UI8CL
        locationDone(player, LOCATIONS[EOBJECT3])
    elseif layout_id == ENEMY0 or layout_id == ENEMY1 or layout_id == ENEMY2 then
        local count = getVar(player, 2, true) + 1 -- UI8CH counts them
        setVar(player, 2, true, count)
        if count >= #LOCATIONS[EOBJECT2].enemies then
            locationDone(player, LOCATIONS[EOBJECT2])
        end
    end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if current_location ~= nil and scene == current_location.scene then
        local location = current_location
        current_location = nil
        if location.enemies ~= nil then
            for _, enemy in ipairs(location.enemies) do
                player:spawn_quest_bnpc(enemy)
            end
        else
            locationDone(player, location)
        end
    elseif scene == SCENE_OFFER then
        if results[1] == 1 then
            play(player, SCENE_ACCEPTED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        setVar(player, 3, true, 1) -- the key item, UI8DH
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_BEFORE_REWARD then
        if results[1] == 1 then
            play(player, SCENE_REWARD, HIDE_HOTBAR)
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
