-- Quest battle: Way Down in the Hole, for the quest of the same name (SubWil066_00320.)
-- It takes place in a copy of Central Thanalan (territory 248), so the clay golem lives in the wil_w1 drop-in (layer
-- QB_SubWil066_001), at retail's position, level (10) and HP (844).
--
-- The same golem fight as Spirithold Broken (questbattle/15.lua) with Ul'dah's sequence numbers and lines, timed
-- against Bob's Chronofoil capture of 2026-10-18 (`questkit chronofoil --battle 0.11`; seconds from entering, the
-- fight starts at 15.5, the director's scene 60000, which is Kawari's onCommence):
--   0.6   director sequence 1 and the opening cutscene (scene 1)
--   15.5  the fight starts: sequence 2
--   18.5  sequence 3, and the golem rumbles (NpcYell 122)
--   77.4  the golem at 70% (589 of 844 after the hit): the masked mage's first line (battle talk 95), sequence 4; Sand
--         Breath (445, a 3.7 s cast) 2.4 s later; sequence 5 at 84.8; the second line (96) and sequence 6 at 89.9
--   112.0 the golem below 45% (45.5% -> 41.4%): sequence 7; it rumbles (NpcYell 123) and casts Plaincracker (482)
--   125.4 the golem at 30%: sequence 8; Sand Breath 1.2 s later; Plaincracker 7.6 s after that, then every 16.3 s
--   151.4 the golem at 15%: the third line (97), sequence 9
--   173.8 the golem falls: sequence 10 and "Duty Complete", then the battle's scene 10 (a cutscene) and out when it
--         ends; the quest moves to sequence 5
-- The lines come through the battle talk window under BNpcName 750 (the masked mage) with no portrait.

-- Where retail puts the player and which way they face (the capture.)
START_POSITION = { x = -224.76, y = -31.75, z = 111.42 }
START_ROTATION = -0.537

BNPC_GOLEM = INIT_POP_01 or 3872242
-- Who's talking: the masked mage, by BNpcName (BNPC_NAME_01), for all three lines; retail sends no portrait.
SPEAKER_MASKED_MAGE = BNPC_NAME_01 or 750
SPEAKER_ICON = 0
-- The battle talk lines, by the content's text ID (from the capture.)
TEXT_FIRST = 95
TEXT_SECOND = 96
TEXT_THIRD = 97

ACTION_SAND_BREATH = DEF_ACTION_SAND_BREATH or 445
ACTION_PLAINCRACKER = 482
YELL_RUMBLE_START = 122
YELL_RUMBLE = 123

-- The quest moves to this sequence when the battle is won.
QUEST_SEQUENCE_AFTER_WIN = 5
-- The scene retail plays after "Duty Complete", and its flags (a cutscene); the player leaves when it ends.
FINISH_SCENE = 10
FINISH_SCENE_FLAGS = 0xF8402EFB
SEQUENCE_FAILED = 0xFE
-- How many updates (half a second each) after the player falls before sending them out.
FAIL_DELAY_UPDATES = 12
-- Plaincracker's interval once the golem is below 30% (16.3 s on retail), in updates.
PLAINCRACKER_EVERY = 33

-- As the golem's health drops to each of these (in percent), these steps run, each this many updates after the last.
PHASES = {
    {
        health = 70,
        steps = {
            { 0, "talk", TEXT_FIRST },
            { 0, "sequence", 4 },
            { 5, "action", ACTION_SAND_BREATH },
            { 10, "sequence", 5 },
            { 10, "talk", TEXT_SECOND },
            { 0, "sequence", 6 },
        },
    },
    {
        health = 45,
        steps = {
            { 0, "sequence", 7 },
            { 0, "yell", YELL_RUMBLE },
            { 0, "action", ACTION_PLAINCRACKER },
        },
    },
    {
        health = 30,
        steps = {
            { 0, "sequence", 8 },
            { 2, "action", ACTION_SAND_BREATH },
            { 13, "action", ACTION_PLAINCRACKER },
            { 0, "repeat_plaincracker" },
        },
    },
    {
        health = 15,
        steps = {
            { 0, "talk", TEXT_THIRD },
            { 0, "sequence", 9 },
        },
    },
}

-- Once it falls: "Duty Complete" straight away, then the closing cutscene, then they're sent back.
WON_STEPS = {
    { 0, "sequence", 10 },
    { 0, "complete" },
}

local next_phase = 1
-- Updates until the golem's next Plaincracker, once it's below 30% (nil before.) Kept apart from the steps so the 15%
-- line isn't queued behind it.
local plaincracker_in = nil
-- Steps waiting to run: { updates left, kind, ... }.
local pending = {}
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
        director:battle_talk(0, SPEAKER_MASKED_MAGE, step[3], SPEAKER_ICON)
    elseif kind == "yell" then
        director:yell(BNPC_GOLEM, step[3])
    elseif kind == "action" then
        director:use_action(BNPC_GOLEM, step[3]) -- at the player
    elseif kind == "repeat_plaincracker" then
        plaincracker_in = PLAINCRACKER_EVERY
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

-- The fight starts as the opening cutscene ends; three seconds later the golem rumbles and comes for the player.
START_STEPS = {
    { 6, "sequence", 3 },
    { 0, "yell", YELL_RUMBLE_START },
}

function onCommence(director)
    director:spawn_bnpc(BNPC_GOLEM)
    director:set_sequence(2)
    director:set_hate(BNPC_GOLEM, nil, 1) -- the player
    queue(START_STEPS)
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

    if plaincracker_in ~= nil then
        plaincracker_in = plaincracker_in - 1
        if plaincracker_in <= 0 then
            director:use_action(BNPC_GOLEM, ACTION_PLAINCRACKER)
            plaincracker_in = PLAINCRACKER_EVERY
        end
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
    plaincracker_in = nil
    queue(WON_STEPS)
end

-- Losing sends you back to where you entered, still on the same step of the quest, so you can try again.
function onPlayerDeath(director)
    if over then
        return
    end
    over = true
    pending = {}
    plaincracker_in = nil
    director:set_sequence(SEQUENCE_FAILED)
    fail_in = FAIL_DELAY_UPDATES
end
