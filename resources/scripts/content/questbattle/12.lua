-- Quest battle: To Guard a Guardian, for the quest of the same name (ManFst007_00447.)
-- It takes place in a copy of the Central Shroud, so its BNpcs live in the fst_f1 drop-in (layer QB_ManFst007_001.)
--
-- Timed against Kawari Recorder's recording of retail (seconds from the fight starting, after the opening cutscene):
--   0      director sequence 2: Galfrid, Miraudont and the guards with you; Quzal Huatotl and six Ixali; more Ixali
--          and guards fighting each other further off. Galfrid: battle talk 161 at 5, Miraudont 162 at 10
--   the first six Ixali down: sequence 3, six more arrive; Galfrid 164 eight seconds later
--   Quzal at 33%: sequence 4, Quzal 165 (and Void Call)
--   Quzal at 25%: sequence 5, Quzal 166, a blanga arrives; Galfrid 163 six seconds later, Miraudont 168 three after
--   everyone down: sequence 6, everyone else leaves and the director's scene 6 plays (a cutscene)
--   after it: sequence 7, a lesser gargoyle and the masked mage; the mage 173 five seconds in
--   22 seconds in: sequence 8, the mage 174 five seconds later
--   gargoyle at 50%: the mage 176
--   gargoyle at 25%: sequence 9, Yda and Papalymo arrive; ten seconds later sequence 10, Papalymo 177, Yda 178 five
--          seconds later, the mage 175 five after that
--   the mage at 40%: the mage 179, Papalymo 180 7.5 seconds later, Yda 181 4.5 after that
--   the mage down: sequence 11, "Duty Complete", the quest battle's scene 11 and back out (the quest moves to
--          sequence 3)
-- Sequence 1 is set before the opening cutscene, as retail does. The layout IDs, health, levels, where everyone stands
-- and the start position are the recording's too. Which six Ixali come first and which in the second wave is a guess
-- from where they stand (the recording doesn't say which layout ID arrived when.)
--
-- The battle talk window (DirectorPopupMessage) shows the content's own text by its ID, so these are the IDs retail
-- sent; the speaker's name and portrait are numbers from the same packets.

START_POSITION = { x = -299.5, y = 21.2, z = 161.24 }
START_ROTATION = 1.88

BNPC_GALFRID = 4089300
BNPC_MIRAUDONT = 4089305
-- The two guards fighting with you.
BNPC_BOW = 3854581
BNPC_LANCE = 3854582
BNPC_QUZAL = 4089289
BNPC_BLANGA = 4089330
BNPC_GARGOYLE = 4089338
BNPC_MAGE = 3854784
BNPC_YDA = 4018837
BNPC_PAPALYMO = 4018839

FIRST_IXALI = { 4089292, 4089293, 4089294, 4089296, 4089297, 4089298 }
SECOND_IXALI = { 3803267, 3803268, 3803269, 4089325, 3803263, 3803264 }
-- Guards and Ixali fighting each other in the background, as pairs. Both have far too much health to win.
BACKGROUND = {
    { 3854583, 3854584 }, { 3854673, 3854723 }, { 3854588, 3854715 }, { 3854674, 3854724 }, { 3854675, 3854719 },
}
-- Background Ixali with nobody of their own; they join in on a guard.
BACKGROUND_EXTRA = { { 3854721, 3854583 }, { 3854586, 3854588 }, { 3854720, 3854673 } }
ALLIES_FIRST_HALF = { BNPC_GALFRID, BNPC_MIRAUDONT, BNPC_BOW, BNPC_LANCE }
ALLIES_SECOND_HALF = { BNPC_YDA, BNPC_PAPALYMO }

-- Who's talking, by BNpcName, and their portrait in the battle talk window.
SPEAKER_GALFRID = { layout = BNPC_GALFRID, name = 948, icon = 0x11e1a }
SPEAKER_MIRAUDONT = { layout = BNPC_MIRAUDONT, name = 517, icon = 0x11e1b }
SPEAKER_QUZAL = { layout = BNPC_QUZAL, name = 946, icon = 0x11e1c }
SPEAKER_MAGE = { layout = BNPC_MAGE, name = 617, icon = 0x11e12 }
SPEAKER_YDA = { layout = BNPC_YDA, name = 619, icon = 0x11dc4 }
SPEAKER_PAPALYMO = { layout = BNPC_PAPALYMO, name = 620, icon = 0x11dcc }

-- The quest moves to this sequence when the battle is won; retail then plays the quest battle's scene 11.
QUEST_SEQUENCE_AFTER_WIN = 3
FINISH_SCENE = 11
-- The cutscene halfway through, and its scene flags as retail sent them.
CUTSCENE_SCENE = 6
CUTSCENE_FLAGS = 0xF8402EFB
SEQUENCE_START = 1
SEQUENCE_FAILED = 0xFE
FAIL_DELAY_UPDATES = 12

ACTION_VOID_CALL = 867
-- Papalymo heals the player with Cure when they drop below this much health (in percent), at most every few seconds.
ACTION_CURE = 120
HEAL_BELOW_HEALTH = 50
HEAL_COOLDOWN_UPDATES = 8

STARTING_ENMITY = 300
ALLY_ENMITY = 10000
-- How much the background fighters hate each other.
BACKGROUND_ENMITY = 100000

-- Steps: { updates after the step before, kind, ... }
START_STEPS = {
    { 0, "sequence", 2 },
    { 10, "talk", SPEAKER_GALFRID, 161 },
    { 10, "talk", SPEAKER_MIRAUDONT, 162 },
}
SECOND_WAVE_STEPS = {
    { 0, "sequence", 3 },
    { 0, "spawn", SECOND_IXALI },
    { 16, "talk", SPEAKER_GALFRID, 164 },
}
-- As Quzal's health drops to each of these (in percent).
QUZAL_PHASES = {
    { health = 33, steps = { { 0, "sequence", 4 }, { 0, "talk", SPEAKER_QUZAL, 165 }, { 0, "action", BNPC_QUZAL, ACTION_VOID_CALL } } },
    {
        health = 25,
        steps = {
            { 0, "sequence", 5 },
            { 0, "talk", SPEAKER_QUZAL, 166 },
            { 0, "spawn", { BNPC_BLANGA } },
            { 12, "talk", SPEAKER_GALFRID, 163 },
            { 5, "talk", SPEAKER_MIRAUDONT, 168 },
        },
    },
}
AFTER_CUTSCENE_STEPS = {
    { 0, "sequence", 7 },
    { 0, "spawn", { BNPC_GARGOYLE, BNPC_MAGE } },
    { 10, "talk", SPEAKER_MAGE, 173 },
    { 34, "sequence", 8 },
    { 10, "talk", SPEAKER_MAGE, 174 },
}
GARGOYLE_PHASES = {
    { health = 50, steps = { { 0, "talk", SPEAKER_MAGE, 176 } } },
    {
        health = 25,
        steps = {
            { 0, "sequence", 9 },
            { 0, "allies", ALLIES_SECOND_HALF },
            { 20, "sequence", 10 },
            { 0, "talk", SPEAKER_PAPALYMO, 177 },
            { 10, "talk", SPEAKER_YDA, 178 },
            { 10, "talk", SPEAKER_MAGE, 175 },
        },
    },
}
MAGE_PHASES = {
    {
        health = 40,
        steps = {
            { 0, "talk", SPEAKER_MAGE, 179 },
            { 15, "talk", SPEAKER_PAPALYMO, 180 },
            { 9, "talk", SPEAKER_YDA, 181 },
        },
    },
}

-- Enemies that have to fall, by layout ID, in the order they arrived.
local remaining = {}
local arrived = {}
-- Everyone who has arrived, so the first half only ends once the second wave and the blanga have come and gone.
local spawned = {}
local first_wave_left = 0
local allies = {}
local pending = {}
-- Which phases of each enemy's have run.
local phase_done = {}
local half = 1
local over = false
local fail_in = nil
local heal_cooldown = 0
local ally_priority = 0

function queue(steps)
    local at = 0
    for _, step in ipairs(pending) do
        at = math.max(at, step[1])
    end
    for _, step in ipairs(steps) do
        at = at + step[1]
        local queued = { at }
        for i = 2, #step do
            queued[i] = step[i]
        end
        table.insert(pending, queued)
    end
end

function spawnEnemies(director, ids)
    for _, id in ipairs(ids) do
        remaining[id] = true
        spawned[id] = true
        table.insert(arrived, id)
        director:spawn_bnpc(id)
        director:set_hate(id, nil, STARTING_ENMITY) -- the player
    end
    alliesAttack(director)
end

function spawnAllies(director, ids)
    for _, id in ipairs(ids) do
        allies[id] = true
        director:spawn_bnpc(id)
    end
    alliesAttack(director)
end

function runStep(director, step)
    local kind = step[2]
    if kind == "sequence" then
        director:set_sequence(step[3])
    elseif kind == "talk" then
        director:battle_talk(step[3].layout, step[3].name, step[4], step[3].icon)
    elseif kind == "spawn" then
        spawnEnemies(director, step[3])
    elseif kind == "allies" then
        spawnAllies(director, step[3])
    elseif kind == "action" then
        director:use_action(step[3], step[4]) -- at the player
    end
end

function runPending(director)
    local still_pending = {}
    for _, step in ipairs(pending) do
        if step[1] <= 0 then
            runStep(director, step)
        else
            step[1] = step[1] - 1
            table.insert(still_pending, step)
        end
    end
    pending = still_pending
end

-- Every ally goes for the oldest enemy still standing; Galfrid and Yda take the boss (Quzal, then the mage) while
-- it's there.
function alliesAttack(director)
    local oldest = nil
    for _, id in ipairs(arrived) do
        if remaining[id] then
            oldest = id
            break
        end
    end
    local boss = nil
    for _, id in ipairs({ BNPC_QUZAL, BNPC_MAGE }) do
        if remaining[id] then
            boss = id
        end
    end
    ally_priority = ally_priority + ALLY_ENMITY
    for id, _ in pairs(allies) do
        local target = oldest
        if boss ~= nil and (id == BNPC_GALFRID or id == BNPC_YDA) then
            target = boss
        end
        if target ~= nil then
            director:set_hate(id, target, ally_priority)
        end
    end
end

function runPhases(name, phases, health)
    if health == nil then
        return
    end
    local done = phase_done[name] or 0
    while phases[done + 1] ~= nil and health <= phases[done + 1].health do
        done = done + 1
        queue(phases[done].steps)
    end
    phase_done[name] = done
end

function onSetup(director)
    director:set_sequence(SEQUENCE_START)
end

function onCommence(director)
    for _, pair in ipairs(BACKGROUND) do
        director:spawn_bnpc(pair[1])
        director:spawn_bnpc(pair[2])
        director:set_hate(pair[1], pair[2], BACKGROUND_ENMITY)
        director:set_hate(pair[2], pair[1], BACKGROUND_ENMITY)
    end
    for _, pair in ipairs(BACKGROUND_EXTRA) do
        director:spawn_bnpc(pair[1])
        director:set_hate(pair[1], pair[2], BACKGROUND_ENMITY)
    end
    spawnAllies(director, ALLIES_FIRST_HALF)
    spawnEnemies(director, { BNPC_QUZAL })
    spawnEnemies(director, FIRST_IXALI)
    first_wave_left = #FIRST_IXALI
    queue(START_STEPS)
end

function onUpdate(director, health, player_health)
    if fail_in ~= nil then
        fail_in = fail_in - 1
        if fail_in <= 0 then
            fail_in = nil
            director:fail_quest_battle()
        end
        return
    end
    if over then
        return
    end

    runPending(director)

    if heal_cooldown > 0 then
        heal_cooldown = heal_cooldown - 1
    elseif player_health ~= nil and player_health > 0 and player_health < HEAL_BELOW_HEALTH
        and health[BNPC_PAPALYMO] ~= nil then
        director:use_action(BNPC_PAPALYMO, ACTION_CURE) -- on the player
        heal_cooldown = HEAL_COOLDOWN_UPDATES
    end

    if half == 1 then
        runPhases("quzal", QUZAL_PHASES, health[BNPC_QUZAL])
    elseif half == 2 then
        runPhases("gargoyle", GARGOYLE_PHASES, health[BNPC_GARGOYLE])
        runPhases("mage", MAGE_PHASES, health[BNPC_MAGE])
    end
end

-- Everyone from the first half leaves for the cutscene.
function clearFirstHalf(director)
    for _, pair in ipairs(BACKGROUND) do
        director:despawn_bnpc(pair[1])
        director:despawn_bnpc(pair[2])
    end
    for _, pair in ipairs(BACKGROUND_EXTRA) do
        director:despawn_bnpc(pair[1])
    end
    for _, id in ipairs(ALLIES_FIRST_HALF) do
        director:despawn_bnpc(id)
    end
    allies = {}
    arrived = {}
end

function onActorDeath(director, layout_id, position)
    if not remaining[layout_id] or over then
        return
    end
    remaining[layout_id] = nil

    for _, id in ipairs(FIRST_IXALI) do
        if id == layout_id then
            first_wave_left = first_wave_left - 1
            if first_wave_left == 0 then
                queue(SECOND_WAVE_STEPS)
            end
        end
    end

    -- Whatever their health would have brought still happens if they fall before it (e.g. the blanga, or Yda and
    -- Papalymo.)
    if layout_id == BNPC_QUZAL then
        runPhases("quzal", QUZAL_PHASES, 0)
    elseif layout_id == BNPC_GARGOYLE then
        runPhases("gargoyle", GARGOYLE_PHASES, 0)
    end

    if half == 1 and next(remaining) == nil and spawned[BNPC_BLANGA] and spawned[SECOND_IXALI[1]] then
        -- Quzal, both waves and the blanga are down.
        half = 0
        pending = {}
        director:set_sequence(6)
        clearFirstHalf(director)
        director:play_scene(CUTSCENE_SCENE, CUTSCENE_FLAGS)
        return
    end

    if layout_id == BNPC_MAGE then
        over = true
        pending = {}
        director:set_sequence(11)
        director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE)
        return
    end

    alliesAttack(director)
end

function onSceneFinished(director, scene)
    if scene ~= CUTSCENE_SCENE or half ~= 0 then
        return
    end
    half = 2
    queue(AFTER_CUTSCENE_STEPS)
end

-- Losing sends you back to where you entered, still on the same step of the quest, so you can try again.
function onPlayerDeath(director)
    if over or half == 0 then
        return
    end
    over = true
    pending = {}
    director:set_sequence(SEQUENCE_FAILED)
    fail_in = FAIL_DELAY_UPDATES
end
