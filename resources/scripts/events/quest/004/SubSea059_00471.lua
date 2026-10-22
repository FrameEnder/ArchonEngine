-- Quest: Rocks in a Hard Place (Limsa Lominsa)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-22) and Bob's Chronofoil capture of the Limsa
-- story on retail (2026-10-21):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: 1 accepts): sequence 1.
-- Sequence 1:   EOBJECT0, the shovel: scene 1 (empty), EVENT_ACTION_SEARCH_SHORT (the progress bar), scene 100 (empty):
--               ITEM0 (UI8BL = 3, its uses), notice 0 [21003], sequence 2.
-- Sequence 2:   in any order:
--               three rocks, EOBJECT1..3: talking plays scene n + 1 (the client's item window); using ITEM0 on one (a
--               key item's event, onEventItem) plays scene 100 - 2n: UI8BH counts them, UI8BL goes down by one,
--               BitFlag8 n, notice 1 [count, 3];
--               three piles of debris, EOBJECT4..6: scene n + 1 (empty), EVENT_ACTION_SEARCH, scene 100 - 2n (empty):
--               UI8AL counts them, BitFlag8 n, notice 2 [count, 3]; the third also sets UI8AH = 1, as retail did.
--               The client offers each only while its count is under 3 and its flag unset. With all six done the
--               variables are cleared and the quest moves to 255 (retail's last was a rock.)
-- Sequence 255: ACTOR0 rewards it (scene 8, a yield with the reward picked.)
-- The objects' scenes with HIDE_HOTBAR | NO_DEFAULT_CAMERA (0x2001), as on retail. Scenes 99 - 2n (empty) aren't
-- played, by retail or by this script.

SCENE_OFFER = 0
SCENE_SHOVEL = 1
SCENE_SHOVEL_TAKEN = 100
SCENE_FINISH = 8

OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001
ITEM0_ICON = 21003 -- what retail's notice 0 carried
ITEM0_USES = 3
NEEDED = 3

NOTICE_SHOVEL = 0
NOTICE_ROCK = 1
NOTICE_DEBRIS = 2

-- The objects' retail layout IDs (Bob's Chronofoil capture of 2026-10-21; matched to EOBJECTn by the scenes and flags
-- retail set), in case their base IDs aren't known.
LAYOUTS = {
    [0] = 3929316,
    [1] = 3929669, [2] = 3929670, [3] = 3929672,
    [4] = 3929673, [5] = 3929674, [6] = 3929676,
}

function objectIndex(target)
    local bases = { [0] = EOBJECT0, [1] = EOBJECT1, [2] = EOBJECT2, [3] = EOBJECT3, [4] = EOBJECT4, [5] = EOBJECT5,
        [6] = EOBJECT6 }
    for n = 0, 6 do
        if bases[n] ~= nil and bases[n] == BASE_ID then
            return n
        end
    end
    if target ~= nil then
        for n = 0, 6 do
            if target.object_id == LAYOUTS[n] then
                return n
            end
        end
    end
    return nil
end

function isRock(n)
    return n ~= nil and n >= 1 and n <= 3
end

function isDebris(n)
    return n ~= nil and n >= 4 and n <= 6
end

-- Quest variables: UI8A..F are bytes 0..5, H the high nibble and L the low one; BitFlag8 n is bit 0x80 >> (n - 1) of
-- UI8F.
function nibble(player, byte, high)
    local value = player:get_quest_var(EVENT_ID, byte)
    if high then
        return value >> 4
    end
    return value & 0x0F
end

function setNibble(player, byte, high, n)
    local value = player:get_quest_var(EVENT_ID, byte)
    if high then
        value = (value & 0x0F) | (n << 4)
    else
        value = (value & 0xF0) | n
    end
    player:set_quest_var(EVENT_ID, byte, value)
end

function flagSet(player, n)
    return (player:get_quest_var(EVENT_ID, 5) & (0x80 >> (n - 1))) ~= 0
end

function setFlag(player, n)
    player:set_quest_var(EVENT_ID, 5, player:get_quest_var(EVENT_ID, 5) | (0x80 >> (n - 1)))
end

function clearVars(player)
    for b = 0, 5 do
        player:set_quest_var(EVENT_ID, b, 0)
    end
end

-- Whether object n can still be done (the client's IsAcceptEvent.)
function open(player, n)
    if isRock(n) then
        return nibble(player, 1, true) < NEEDED and not flagSet(player, n)
    end
    return nibble(player, 0, false) < NEEDED and not flagSet(player, n)
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- The object whose scenes are playing.
working = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local n = objectIndex(target)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and n == 0 then
        working = 0
        play(player, SCENE_SHOVEL, OBJECT_FLAGS)
    elseif sequence == 2 and (isRock(n) or isDebris(n)) and open(player, n) then
        working = n
        play(player, n + 1, OBJECT_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_FINISH, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEventItem(item_id, target, player)
    local n = objectIndex(target)
    if player:get_quest_sequence(EVENT_ID) == 2 and item_id == ITEM0 and isRock(n) and open(player, n) then
        working = n
        play(player, 100 - 2 * n, OBJECT_FLAGS)
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    if working == 0 then
        play(player, SCENE_SHOVEL_TAKEN, OBJECT_FLAGS)
    elseif isDebris(working) then
        play(player, 100 - 2 * working, OBJECT_FLAGS)
    else
        player:finish_event()
    end
end

-- All six done: on to the reward.
function checkDone(player)
    if nibble(player, 1, true) >= NEEDED and nibble(player, 0, false) >= NEEDED then
        clearVars(player)
        player:quest_sequence(EVENT_ID, 255)
    end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil
    local n = working

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID)
            player:quest_sequence(EVENT_ID, 1)
        end
    elseif scene == SCENE_SHOVEL and n == 0 then
        -- The event carries on once the progress bar finishes, see onEventActionCast.
        player:event_action(EVENT_ACTION_SEARCH_SHORT)
        return
    elseif scene == SCENE_SHOVEL_TAKEN and n == 0 then
        working = nil
        setNibble(player, 1, false, ITEM0_USES)
        player:event_notice(EVENT_ID, NOTICE_SHOVEL, { ITEM0_ICON })
        player:quest_sequence(EVENT_ID, 2)
    elseif isDebris(n) and scene == n + 1 then
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif isDebris(n) and scene == 100 - 2 * n then
        working = nil
        local count = nibble(player, 0, false) + 1
        setNibble(player, 0, false, count)
        if count >= NEEDED then
            setNibble(player, 0, true, 1)
        end
        setFlag(player, n)
        player:event_notice(EVENT_ID, NOTICE_DEBRIS, { count, NEEDED })
        checkDone(player)
    elseif isRock(n) and scene == 100 - 2 * n then
        working = nil
        local count = nibble(player, 1, true) + 1
        setNibble(player, 1, true, count)
        setNibble(player, 1, false, math.max(nibble(player, 1, false) - 1, 0))
        setFlag(player, n)
        player:event_notice(EVENT_ID, NOTICE_ROCK, { count, NEEDED })
        checkDone(player)
    elseif scene == SCENE_FINISH then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID, results[2])
        end
    else
        -- A rock's item window (scenes 2-4) closing: the item's own event follows if one was used.
        working = nil
    end

    player:finish_event()
end

function onYield(scene, id, results, player)
    sceneFinished(scene, results, player)
end

function onReturn(scene, results, player)
    sceneFinished(scene, results, player)
end
