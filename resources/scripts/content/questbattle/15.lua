-- Quest battle: Spirithold Broken, for the quest of the same name (SubFst035_00129.)
-- It takes place in a copy of the Central Shroud, so the clay golem lives in the fst_f1 drop-in (layer QB_SubFst035_001.)
--
-- Timed against Kawari Recorder's recording of retail (seconds from entering):
--   0.5   director sequence 1, "Defeat the clay golem!"
--   10.6  the golem rumbles (NpcYell 122) and the fight starts: sequence 2
--   39.4  the masked mage: "You are strong..." (battle talk 1), sequence 3; True Grit lands at 46.2 (4 second cast)
--   47.2  sequence 4
--   52.2  "Yesss...writhe!..." (battle talk 2), sequence 5
--   75.1  sequence 6; it rumbles (NpcYell 123) and casts Plaincracker, which lands at 80.2
--   91.9  sequence 7; True Grit again, landing at 97.0, then Plaincracker landing at 103.2
--   106.6 the masked mage: "Impossible!..." (battle talk 3), sequence 8
--   118.8 the golem falls: sequence 9 and "Duty Complete"
--   131.2, 134.0 two more lines (battle talk 4 and 5), and out at 143.5
-- The health at which the lines and skills come is Sapphire's (65%, 60%, 30%, 15%.)
--
-- The battle talk is the quest's text whose keys contain BATTLETALK, in key order (the server logs which it found.)
-- On retail it's shown in the battle talk window, which Kawari can't show yet, so it's said in chat instead.

START_POSITION = { x = 258.321, y = 24.0, z = 192.406 }
START_ROTATION = -1.544

BNPC_GOLEM = INIT_POP_01 or 3869528
-- Who's talking: the masked mage, by BNpcName (BNPC_NAME_01.) 0 shows "???", as retail does for the other voice.
SPEAKER_MASKED_MAGE = BNPC_NAME_01 or 750
SPEAKER_UNKNOWN = 0

ACTION_TRUE_GRIT = DEF_ACTION_SAND_BREATH or 445
ACTION_PLAINCRACKER = 482
YELL_RUMBLE_START = 122
YELL_RUMBLE = 123

-- The quest moves to this sequence when the battle is won.
QUEST_SEQUENCE_AFTER_WIN = 4
SEQUENCE_FAILED = 0xFE
-- How many updates (half a second each) after the player falls before sending them out.
FAIL_DELAY_UPDATES = 12

-- Things that happen a number of updates (half a second each) after something else, in order.
FIGHT_STARTS_IN = 21

-- As the golem's health drops to each of these (in percent), these steps run, each this many updates after the last.
PHASES = {
    {
        health = 65,
        steps = {
            { 0, "talk", 1, SPEAKER_MASKED_MAGE },
            { 0, "sequence", 3 },
            { 6, "action", ACTION_TRUE_GRIT },
            { 10, "sequence", 4 },
            { 10, "talk", 2, SPEAKER_UNKNOWN },
            { 0, "sequence", 5 },
        },
    },
    {
        health = 30,
        steps = {
            { 0, "sequence", 6 },
            { 4, "yell", YELL_RUMBLE },
            { 0, "action", ACTION_PLAINCRACKER },
            { 30, "sequence", 7 },
            { 2, "action", ACTION_TRUE_GRIT },
            { 13, "action", ACTION_PLAINCRACKER },
        },
    },
    {
        health = 15,
        steps = {
            { 0, "talk", 3, SPEAKER_MASKED_MAGE },
            { 0, "sequence", 8 },
        },
    },
}

-- Once it falls: "Duty Complete" straight away (players stay), the last lines, then they're sent back.
WON_STEPS = {
    { 0, "sequence", 9 },
    { 0, "complete" },
    { 25, "talk", 4, SPEAKER_UNKNOWN },
    { 6, "talk", 5, SPEAKER_UNKNOWN },
    { 19, "leave" },
}

local next_phase = 1
-- Steps waiting to run: { updates left, kind, ... }.
local pending = {}
local fight_starts_in = nil
local over = false
local fail_in = nil

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
        director:say_battle_talk(BNPC_GOLEM, step[3], step[4])
    elseif kind == "yell" then
        director:yell(BNPC_GOLEM, step[3])
    elseif kind == "action" then
        director:use_action(BNPC_GOLEM, step[3]) -- at the player
    elseif kind == "complete" then
        director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, true) -- they stay for the last lines
    elseif kind == "leave" then
        director:leave_quest_battle()
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
end

function onCommence(director)
    director:set_sequence(1)
    director:spawn_bnpc(BNPC_GOLEM)
    fight_starts_in = FIGHT_STARTS_IN
end

-- health: the health (in percent) of every living NPC, by layout ID
-- player_health: the player's health in percent (nil if they're not here)
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

    if fight_starts_in ~= nil then
        fight_starts_in = fight_starts_in - 1
        if fight_starts_in <= 0 then
            fight_starts_in = nil
            director:yell(BNPC_GOLEM, YELL_RUMBLE_START)
            director:set_sequence(2)
            director:set_hate(BNPC_GOLEM, nil, 1) -- the player
        end
        return
    end

    local golem_health = health[BNPC_GOLEM]
    if golem_health == nil then
        return
    end
    while PHASES[next_phase] ~= nil and golem_health <= PHASES[next_phase].health do
        queue(PHASES[next_phase].steps)
        next_phase = next_phase + 1
    end
end

function onActorDeath(director, layout_id, position)
    if layout_id ~= BNPC_GOLEM or over then
        return
    end
    over = true
    -- Anything it was about to do won't happen now.
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
