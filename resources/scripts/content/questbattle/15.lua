-- Quest battle: Spirithold Broken, for the quest of the same name (SubFst035_00129.)
-- It takes place in a copy of the Central Shroud, so the clay golem lives in the fst_f1 drop-in (layer QB_SubFst035_001.)
--
-- Timed against Kawari Recorder's recording of retail and Bob's Chronofoil capture of 2026-10-14 (seconds from
-- entering; the fight starts at 10.5, the director's scene 60000, which is Kawari's onCommence):
--   0.8   director sequence 1, "Defeat the clay golem!", and the opening cutscene
--   10.5  the fight starts: sequence 2, and the golem rumbles (NpcYell 122)
--   39.7  the golem at 70%: the masked mage's first line (battle talk 152), sequence 3; True Grit 2.6 s later
--   47.4  sequence 4
--   52.5  his second (153), sequence 5
--   75.1  the golem at 45%: sequence 6; it rumbles (NpcYell 123) and casts Plaincracker
--   91.8  sequence 7; True Grit again, then Plaincracker
--   106.9 the golem at 15%: his third (154), sequence 8
--   118.9 the golem falls: sequence 9 and "Duty Complete", then the battle's scene 9 (a cutscene with two more
--         lines, played by the client) and out when it ends, at 142.5
-- The health marks are the capture's (questkit battle-sim: it crossed 70% at the first line, 45% at sequence 6 and
-- 15% at the third line); Sapphire's 65% and 30% came 7 and 16 s late against it.
--
-- Bob's Chronofoil capture of the same battle (2026-10-14) gave the battle talk's text IDs and the scenes: the three
-- lines come through the battle talk window (DirectorPopupMessage) under BNpcName 750 with no portrait and no
-- speaker actor (he isn't in the battle), and the closing scene is 9 with cutscene flags.

-- Where retail puts the player and which way they face: their SpawnPlayer in Bob's Chronofoil capture (2026-10-14).
START_POSITION = { x = 255.77, y = 24.0, z = 191.24 }
START_ROTATION = -1.544

BNPC_GOLEM = INIT_POP_01 or 3869528
-- Who's talking: the masked mage, by BNpcName (BNPC_NAME_01), for all three lines; retail sends no portrait.
SPEAKER_MASKED_MAGE = BNPC_NAME_01 or 750
SPEAKER_ICON = 0
-- The battle talk lines, by the content's text ID (from the capture.)
TEXT_STRONG = 152
TEXT_WRITHE = 153
TEXT_IMPOSSIBLE = 154

ACTION_TRUE_GRIT = DEF_ACTION_SAND_BREATH or 445
ACTION_PLAINCRACKER = 482
YELL_RUMBLE_START = 122
YELL_RUMBLE = 123

-- The quest moves to this sequence when the battle is won.
QUEST_SEQUENCE_AFTER_WIN = 4
-- The scene retail plays after "Duty Complete", and its flags (a cutscene); the player leaves when it ends.
FINISH_SCENE = 9
FINISH_SCENE_FLAGS = 0xF8402EFB
SEQUENCE_FAILED = 0xFE
-- How many updates (half a second each) after the player falls before sending them out.
FAIL_DELAY_UPDATES = 12

-- As the golem's health drops to each of these (in percent), these steps run, each this many updates after the last.
PHASES = {
    {
        health = 70,
        steps = {
            { 0, "talk", TEXT_STRONG },
            { 0, "sequence", 3 },
            { 6, "action", ACTION_TRUE_GRIT },
            { 10, "sequence", 4 },
            { 10, "talk", TEXT_WRITHE },
            { 0, "sequence", 5 },
        },
    },
    {
        health = 45,
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
            { 0, "talk", TEXT_IMPOSSIBLE },
            { 0, "sequence", 8 },
        },
    },
}

-- Once it falls: "Duty Complete" straight away, then the closing cutscene, then they're sent back.
WON_STEPS = {
    { 0, "sequence", 9 },
    { 0, "complete" },
}

local next_phase = 1
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

-- The fight starts as the opening cutscene ends: the golem rumbles and comes for the player.
function onCommence(director)
    director:spawn_bnpc(BNPC_GOLEM)
    director:yell(BNPC_GOLEM, YELL_RUMBLE_START)
    director:set_sequence(2)
    director:set_hate(BNPC_GOLEM, nil, 1) -- the player
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
