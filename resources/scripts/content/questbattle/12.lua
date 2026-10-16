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

-- Who goes after whom, as the recording shows (who took whose hits): Quzal after Galfrid, the lostwings after
-- Miraudont, the slowbeaks after the two guards, the sincallers after Galfrid and Miraudont; the blanga, the gargoyle
-- and the masked mage after the player. Anyone not listed goes after the player. Hitting them pulls them over, as
-- enmity does. How hard each hits and heals is in their timelines' recorded_amounts.
LOSTWINGS = { [4089296] = true, [4089297] = true, [4089298] = true, [3803269] = true, [4089325] = true }
SLOWBEAKS = { [4089292] = true, [4089293] = true, [4089294] = true, [3803267] = true, [3803268] = true }
ENEMY_TARGETS = {
    [BNPC_QUZAL] = BNPC_GALFRID,
    [4089296] = BNPC_MIRAUDONT, [4089297] = BNPC_MIRAUDONT, [4089298] = BNPC_MIRAUDONT,
    [3803269] = BNPC_MIRAUDONT, [4089325] = BNPC_MIRAUDONT,
    [4089292] = BNPC_BOW, [4089293] = BNPC_LANCE, [4089294] = BNPC_BOW, [3803267] = BNPC_LANCE, [3803268] = BNPC_BOW,
    [3803263] = BNPC_GALFRID, [3803264] = BNPC_MIRAUDONT,
}

-- Who casts Cure on whom, below what health (in percent), at most this often (in updates), from the recording: the
-- bow, Miraudont and Galfrid keep the two guards (and themselves) up, the sincallers heal Quzal, and Papalymo the
-- player in the second half.
PLAYER = "player"
HEALERS = {
    { healer = BNPC_BOW, targets = { BNPC_BOW, BNPC_LANCE }, below = 60, every = 10 },
    { healer = BNPC_MIRAUDONT, targets = { BNPC_BOW, BNPC_LANCE, BNPC_MIRAUDONT }, below = 50, every = 12 },
    { healer = BNPC_GALFRID, targets = { BNPC_BOW, BNPC_LANCE, BNPC_GALFRID, BNPC_MIRAUDONT }, below = 45, every = 20 },
    { healer = 3803263, targets = { BNPC_QUZAL }, below = 60, every = 10 },
    { healer = 3803264, targets = { BNPC_QUZAL }, below = 60, every = 10 },
    { healer = BNPC_PAPALYMO, targets = { PLAYER }, below = 50, every = 8 },
}

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
ACTION_CURE = 120

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
        director:set_hate(id, ENEMY_TARGETS[id], STARTING_ENMITY) -- nil is the player
    end
    alliesAttack(director)
end

-- Each healer Cures whoever of theirs is lowest, if they're below its threshold.
function runHeals(director, health, player_health)
    for _, h in ipairs(HEALERS) do
        h.wait = (h.wait or 0) - 1
        if h.wait <= 0 and health[h.healer] ~= nil then
            local best, best_health = nil, h.below
            for _, target in ipairs(h.targets) do
                local current = target == PLAYER and player_health or health[target]
                if current ~= nil and current > 0 and current < best_health then
                    best, best_health = target, current
                end
            end
            if best ~= nil then
                if best == PLAYER then
                    director:use_action(h.healer, ACTION_CURE)
                else
                    director:use_action(h.healer, ACTION_CURE, best)
                end
                h.wait = h.every
            end
        end
    end
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

-- The oldest enemy still standing, of a kind if asked.
function oldest(kind)
    for _, id in ipairs(arrived) do
        if remaining[id] and (kind == nil or kind[id]) then
            return id
        end
    end
    return nil
end

-- As on retail: Galfrid, Yda and Papalymo take the boss (Quzal, then the mage) while it's there; Miraudont the
-- lostwings and the guards the slowbeaks that are after them; then whoever's oldest.
function alliesAttack(director)
    local any = oldest(nil)
    local boss = nil
    for _, id in ipairs({ BNPC_QUZAL, BNPC_MAGE }) do
        if remaining[id] then
            boss = id
        end
    end
    ally_priority = ally_priority + ALLY_ENMITY
    for id, _ in pairs(allies) do
        local target = any
        if boss ~= nil and (id == BNPC_GALFRID or id == BNPC_YDA or id == BNPC_PAPALYMO) then
            target = boss
        elseif id == BNPC_MIRAUDONT then
            target = oldest(LOSTWINGS) or any
        elseif id == BNPC_BOW or id == BNPC_LANCE then
            target = oldest(SLOWBEAKS) or any
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

    runHeals(director, health, player_health)

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
