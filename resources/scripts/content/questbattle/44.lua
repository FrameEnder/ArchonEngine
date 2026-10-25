-- Quest battle: Into a Copper Hell, for the quest of the same name (ManFst205_00660.)
-- Territory 274, taken to be a copy of Ul'dah - Steps of Nald (130): retail's positions in it stand on Steps of Nald's
-- ground (y 4.0, by the Quicksand), so the three adventurers are layer QB_ManFst205_001 of the wil_w1t1 drop-in, at
-- retail's spots, levels, HP and display flags (Sapphire's script has the same layout IDs).
--
-- Timed against Bob's Chronofoil capture of 2026-10-25 (4777fd0f, `questkit chronofoil --battle 0.1/0.3/0.5/0.7`,
-- territory 274, duty 338; three lost and one won, as a conjurer; seconds from the fight starting, the director's scene
-- 60000, which is Kawari's onCommence):
--   before the opening cutscene (scene 1, flags 0xF841AEFB): director sequence 1; all three stand there from the start
--   0     the fight starts: sequence 2; the pugilist (BNpcBase 960, 1844 HP) comes for the player, the gladiator (961)
--         and the thaumaturge (962) wait (they fought only once hit, or from sequence 4)
--   5.1   the pugilist's first line (battle talk 204; all its lines under BNpcName 991 with portrait 73230)
--   the pugilist below 65 % (67.0 -> 61.5, 68.8 -> 59.4, 65.2 -> 59.8, 70.0 -> 64.5): line 205
--   below 50 % (51.0 -> 45.6, 53.4 -> 47.1, 54.3 -> 49.0, 52.0 -> 43.9): line 206
--   below 35 % (38.0 -> 32.6, 35.0 -> 29.6, 38.0 -> 32.8, 40.1 -> 33.8): sequence 3 and line 207; 5 s later
--         sequence 4 and the other two come for the player
--   below 20 % (20.5 -> 15.0, 24.4 -> 18.9, 24.4 -> 18.8): line 208
--   all three down (the won run: the last one's health reached 0 at 76.7, retail's Defeated at 78.7): sequence 5 and
--         "Duty Complete", then the battle's scene 5 (a cutscene, flags 0xF8402EFB) and out; the quest moves to 5
--   the player down: sequence 0xFE at once, out 5 s later
-- The pugilist heals itself (Second Wind) now and then, so its marks can come late.

-- The opening cutscene's scene flags, as retail sent them (the capture.)
OPENING_SCENE_FLAGS = 0xF841AEFB

-- Where retail puts the player and which way they face (the capture.)
START_POSITION = { x = 37.01, y = 4.0, z = -137.54 }
START_ROTATION = 2.879

BNPC_PUGILIST = INIT_POP_A or 4094987
BNPC_GLADIATOR = INIT_POP_B or 4094990
BNPC_THAUMATURGE = INIT_POP_C or 4094992
ENEMIES = { BNPC_PUGILIST, BNPC_GLADIATOR, BNPC_THAUMATURGE }
-- The ones that wait until sequence 4 (or until hit)
WAITING = { BNPC_GLADIATOR, BNPC_THAUMATURGE }

-- Who's talking: the pugilist, by BNpcName, with its portrait (the capture.)
SPEAKER_NAME = 991
SPEAKER_ICON = 73230

-- The quest moves to this sequence when the battle is won (retail's UpdateQuest as the cutscene ended.)
QUEST_SEQUENCE_AFTER_WIN = 5
-- The scene retail plays after "Duty Complete", and its flags (a cutscene); the player leaves when it ends.
FINISH_SCENE = 5
FINISH_SCENE_FLAGS = 0xF8402EFB
SEQUENCE_FAILED = 0xFE
-- How many updates (half a second each) after the player falls before sending them out (retail: 5 s.)
FAIL_DELAY_UPDATES = 10

-- As the pugilist's health drops to each of these (in percent), these steps run, each this many updates after the
-- last.
PHASES = {
    { health = 65, steps = { { 0, "talk", 205 } } },
    { health = 50, steps = { { 0, "talk", 206 } } },
    {
        health = 35,
        steps = {
            { 0, "sequence", 3 },
            { 0, "talk", 207 },
            { 10, "sequence", 4 },
            { 0, "engage" },
        },
    },
    { health = 20, steps = { { 0, "talk", 208 } } },
}

START_STEPS = {
    { 10, "talk", 204 },
}

-- Once the last falls: "Duty Complete" 2 s later (retail's came as the last one's death animation ended), then the
-- closing cutscene, then they're sent back.
WON_STEPS = {
    { 4, "sequence", 5 },
    { 0, "complete" },
}

local next_phase = 1
-- Steps waiting to run: { updates left, kind, ... }.
local pending = {}
local over = false
local fail_in = nil
local standing = {}

-- Queues steps after whatever's already waiting, each `delay` updates after the one before.
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

function runStep(director, step)
    local kind = step[2]
    if kind == "sequence" then
        director:set_sequence(step[3])
    elseif kind == "talk" then
        director:battle_talk(BNPC_PUGILIST, SPEAKER_NAME, step[3], SPEAKER_ICON)
    elseif kind == "engage" then
        for _, id in ipairs(WAITING) do
            if standing[id] then
                director:set_notice(id, true)
                director:set_hate(id, nil, 1) -- the player
            end
        end
    elseif kind == "complete" then
        director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS)
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

function onSetup(director)
    director:set_sequence(1)
end

function onCommence(director)
    for _, id in ipairs(ENEMIES) do
        director:spawn_bnpc(id)
        standing[id] = true
    end
    -- The gladiator and the thaumaturge only fight once hit, until sequence 4
    for _, id in ipairs(WAITING) do
        director:set_notice(id, false)
    end
    director:set_sequence(2)
    director:set_hate(BNPC_PUGILIST, nil, 1) -- the player
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

    runPending(director)
    if over then
        return
    end

    local pugilist = health[BNPC_PUGILIST]
    if pugilist == nil then
        return
    end
    while PHASES[next_phase] ~= nil and pugilist < PHASES[next_phase].health do
        queue(PHASES[next_phase].steps)
        next_phase = next_phase + 1
    end
end

function onActorDeath(director, layout_id, position)
    if standing[layout_id] == nil or over then
        return
    end
    standing[layout_id] = nil
    if next(standing) ~= nil then
        return
    end
    over = true
    pending = {}
    queue(WON_STEPS)
end

-- Losing sends you back to where you entered, still on the same step of the quest, so you can try again.
function onPlayerDeath(director)
    if over then
        return
    end
    over = true
    pending = {}
    director:set_sequence(SEQUENCE_FAILED)
    fail_in = FAIL_DELAY_UPDATES
end
