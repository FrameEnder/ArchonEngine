-- Quest battle: Just Deserts, for the quest of the same name (SubSea057_00469.)
-- It takes place in territory 272, taken to be a copy of Middle La Noscea as 252 is (retail's positions in it are
-- Middle La Noscea's), so the clay golem lives in the sea_s1f1 drop-in (layer QB_SubSea057_001), at retail's position,
-- level (10), HP (844) and display flags (0x40000).
--
-- The same golem fight as Spirithold Broken and Way Down in the Hole (questbattle/15.lua, 18.lua) with Limsa
-- Lominsa's lines, timed against Bob's Chronofoil capture of 2026-10-21 (`questkit chronofoil --battle 0.11`,
-- territory 272, duty 333; seconds from entering, the fight starts at 34.0, the director's scene 60000, which is
-- Kawari's onCommence):
--   0.8   director sequence 1 and the opening cutscene (scene 1)
--   34.0  the fight starts: sequence 2
--   39.1  sequence 3, and the golem rumbles (NpcYell 122)
--   67.0  the golem at 70% (589 of 844 after the hit): the masked mage's first line (battle talk 82), sequence 4; Sand
--         Breath (445) 3.2 s later; sequence 5 at 75.2; the second line (83) and sequence 6 at 80.3
--   91.6  the golem below 45% (47.6% -> 43.2%): sequence 7; a second later it rumbles (NpcYell 123) and casts
--         Plaincracker (482), then every 16.3 s
--   155.9 the golem below 30%: sequence 8; Sand Breath 2.0 s later; Plaincracker 6.4 s after that, then every 16.3 s
--   163.4 the golem below 15%: the third line (84), sequence 9
--   211.9 the golem falls (its health reached 0 at 209.8): sequence 10 and "Duty Complete", then the battle's scene 10 (a cutscene) and out when it
--         ends; the quest moves to sequence 5
-- The lines come through the battle talk window under BNpcName 750 (the masked mage) with no portrait. The player
-- was an arcanist (its carbuncle, BNpcBase 13498, is in the capture too.)

-- Where retail puts the player and which way they face (the capture.)
START_POSITION = { x = -25.52, y = 10.87, z = -439.72 }
START_ROTATION = 2.49

BNPC_GOLEM = INIT_POP_01 or 4082418
-- Who's talking: the masked mage, by BNpcName (BNPC_NAME_01), for all three lines; retail sends no portrait.
SPEAKER_MASKED_MAGE = BNPC_NAME_01 or 750
SPEAKER_ICON = 0
-- The battle talk lines, by the content's text ID (from the capture.)
TEXT_FIRST = 82
TEXT_SECOND = 83
TEXT_THIRD = 84

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
-- Plaincracker's interval once the golem is below 45% (16.3 s on retail), in updates.
PLAINCRACKER_EVERY = 33

-- As the golem's health drops to each of these (in percent), these steps run, each this many updates after the last.
PHASES = {
    {
        health = 70,
        steps = {
            { 0, "talk", TEXT_FIRST },
            { 0, "sequence", 4 },
            { 6, "action", ACTION_SAND_BREATH },
            { 10, "sequence", 5 },
            { 10, "talk", TEXT_SECOND },
            { 0, "sequence", 6 },
        },
    },
    {
        health = 45,
        steps = {
            { 0, "sequence", 7 },
            { 2, "yell", YELL_RUMBLE },
            { 0, "action", ACTION_PLAINCRACKER },
            { 0, "repeat_plaincracker" },
        },
    },
    {
        health = 30,
        steps = {
            { 0, "stop_plaincracker" },
            { 0, "sequence", 8 },
            { 4, "action", ACTION_SAND_BREATH },
            -- On a timer rather than a step, so the 15% line isn't queued behind it (retail's came before it.)
            { 0, "plaincracker_in", 13 },
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

-- Once it falls: "Duty Complete" 2 s later (its health reached 0 at 209.8, sequence 10 came at 211.9 as its death
-- animation ended), then the closing cutscene, then they're sent back.
WON_STEPS = {
    { 4, "sequence", 10 },
    { 0, "complete" },
}

local next_phase = 1
-- Updates until the golem's next Plaincracker, once it's below 45% (nil before.) Kept apart from the steps so the 15%
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
    elseif kind == "plaincracker_in" then
        plaincracker_in = step[3]
    elseif kind == "stop_plaincracker" then
        -- At 30% Sand Breath comes first; Plaincracker starts over after it.
        plaincracker_in = nil
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

-- The fight starts as the opening cutscene ends; five seconds later the golem rumbles and comes for the player.
START_STEPS = {
    { 10, "sequence", 3 },
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
