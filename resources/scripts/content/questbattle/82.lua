-- Quest battle 82: the Summoner's level 50 quest's battle (quest 1096), in territory 326, whose level isn't known, so
-- its NPCs live in a drop-in of its own (`territory_326.json`, layer QB_82). Layout IDs, levels, HP, positions, gear
-- and flags are retail's: Bob's Chronofoil capture a75e1390 (2026-11-17: one run, won in 132 s, a Summoner of 50,
-- `questkit chronofoil --battle 0.4`), ContentFinderCondition 375.
--
-- A master (4302960, BNpcName 1651, 11391 HP) and its creature (4302979, 12945 HP) against you and an ally (4302956,
-- BNpcName 1640, 13592 HP). The creature goes for you, the master for the ally (and never moves), the ally for the
-- master, and it Cures you (four times, at 39 to 55 %).
--
-- Timed against the run (seconds from the fight starting, the director's scene 60000):
--   0      sequence 2; the master's talk 103 5.0 s in, the ally's 104 at 10.0
--   the creature at 80 % (22.6 s in, 1.7 s after its health passed it): sequence 3, the master's 105 5.1 s later
--   the creature at 50 % (58.2 s in, 1.8 s after): sequence 4 and the master's 106; 1.1 s later the master casts
--          1348 (4.2 s) on
--          its creature, which took all its health and gave the master back its own
--   6.5 s after sequence 4: sequence 5, and the creature fades out (ActorControl 607). The master's 107 7.1 s later,
--          the ally's 108 at 15.1, the master's 109 at 20.1. From then the master casts 1349 at you (5.1 s after the
--          sequence, then every 28.8 s: 1319 and 1543) and 1163 on itself (13.7 s after, then every 28.9 s), which
--          brings an add (4328234, level 48, 714 HP; it begins Fire III on you) 4.5 s after the cast begins
--   the master's health at 0: at once sequence 6 and "Duty Complete", scene 6 (0xF8402EFB) 3.1 s later; the quest
--          moves to 4
-- Read from one run, so guesses: that the creature's health (and not a clock) moves sequences 3 and 4 on.

-- Where retail puts the player and which way they face: their arrival in the capture.
START_POSITION = { x = 479.62, y = -0.87, z = -794.01 }
START_ROTATION = 1.119
-- Retail's opening scene 1 here.
OPENING_SCENE_FLAGS = 0xF841AEFB

BNPC_ALLY = 4302956
BNPC_MASTER = 4302960
BNPC_CREATURE = 4302979
BNPC_ADD = 4328234

QUEST_SEQUENCE_AFTER_WIN = 4
SEQUENCE_START = 1
SEQUENCE_FIGHTING = 2
SEQUENCE_CREATURE_HURT = 3
SEQUENCE_CREATURE_SPENT = 4
SEQUENCE_MASTER_ALONE = 5
SEQUENCE_WON = 6
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 6
FINISH_SCENE_FLAGS = 0xF8402EFB
FAIL_DELAY_UPDATES = 12

CREATURE_HURT_AT = 80
CREATURE_SPENT_AT = 50

-- Timers, in updates (half a second each): a timer of n fires n / 2 s after it's set in onUpdate or by another
-- timer, and (n - 1) / 2 s after it's set in onCommence.
-- Retail's sequences 3 and 4 each came 1.7 to 1.8 s after the creature's health had passed the mark.
HEALTH_NOTICED_AFTER = 3
CONSUME_AFTER = 2
ALONE_AFTER = 13
BLAST_FIRST = 10
BLAST_EVERY = 58
CALL_FIRST = 27
CALL_EVERY = 58
ADD_AFTER_CALL = 9

ACTION_CONSUME = 1348
ACTION_BLAST = 1349
ACTION_CALL = 1163
ACTION_CURE = 120

-- The battle talk (the content's text IDs and the speakers' BNpcNames, retail's; no portrait.)
SPEAKER_MASTER = { layout = BNPC_MASTER, name = 1651, icon = 0 }
SPEAKER_ALLY = { layout = BNPC_ALLY, name = 1640, icon = 0 }

MASTER = "master"
CREATURE = "creature"
ADDS = "adds"

BATTLE = Battle.new()
BATTLE:ally(BNPC_ALLY, {
    role = "support",
    focus = { MASTER, "any" },
    range = 6,
    heals = { { action = ACTION_CURE, targets = { Battle.PLAYER }, below = 55, every = 14 } },
})
BATTLE:enemy(BNPC_MASTER, { group = MASTER, attacks = BNPC_ALLY })
BATTLE:enemy(BNPC_CREATURE, { group = CREATURE })
BATTLE:enemy(BNPC_ADD, { group = ADDS, stands_for = 0 })

local over = false
local fail_in = nil
local phase = SEQUENCE_START
local timers = {}
-- How many of the master's adds stand (they share a layout ID.)
local adds = 0
-- A mark of the creature's health has been passed and its sequence is on its way.
local noticing = false

function after(updates, fn)
    table.insert(timers, { updates, fn })
end

function runTimers(director)
    local current = timers
    timers = {}
    for _, timer in ipairs(current) do
        timer[1] = timer[1] - 1
        if timer[1] <= 0 then
            timer[2](director)
        else
            table.insert(timers, timer)
        end
    end
end

local function sequence(director, n)
    phase = n
    director:set_sequence(n)
end

function talk(director, speaker, text)
    director:battle_talk(speaker.layout, speaker.name, text, speaker.icon)
end

function onSetup(director)
    director:set_sequence(SEQUENCE_START)
end

function onCommence(director)
    sequence(director, SEQUENCE_FIGHTING)
    BATTLE:spawn(director, { BNPC_ALLY })
    BATTLE:spawn(director, { BNPC_MASTER, BNPC_CREATURE })
    after(11, function(d) talk(d, SPEAKER_MASTER, 103) end)
    after(21, function(d) talk(d, SPEAKER_ALLY, 104) end)
end

local function bringAdd(director)
    if adds == 0 then
        BATTLE:spawn(director, { BNPC_ADD })
    else
        director:spawn_bnpc(BNPC_ADD)
    end
    adds = adds + 1
end

-- The master's two casts once it's alone, each on its own clock.
local function blast(director)
    if over or not BATTLE.alive[BNPC_MASTER] then
        return
    end
    director:use_action(BNPC_MASTER, ACTION_BLAST) -- at the player
    after(BLAST_EVERY, blast)
end

local function call(director)
    if over or not BATTLE.alive[BNPC_MASTER] then
        return
    end
    director:use_action(BNPC_MASTER, ACTION_CALL, BNPC_MASTER)
    after(ADD_AFTER_CALL, function(d)
        if not over and BATTLE.alive[BNPC_MASTER] then
            bringAdd(d)
        end
    end)
    after(CALL_EVERY, call)
end

local function creatureSpent(director)
    sequence(director, SEQUENCE_CREATURE_SPENT)
    talk(director, SPEAKER_MASTER, 106)
    after(CONSUME_AFTER, function(d)
        d:use_action(BNPC_MASTER, ACTION_CONSUME, BNPC_CREATURE)
    end)
    after(ALONE_AFTER, function(d)
        sequence(d, SEQUENCE_MASTER_ALONE)
        d:fade_out_bnpc(BNPC_CREATURE)
        after(2, function(d2) BATTLE:remove(d2, { BNPC_CREATURE }) end)
        after(14, function(d2) talk(d2, SPEAKER_MASTER, 107) end)
        after(30, function(d2) talk(d2, SPEAKER_ALLY, 108) end)
        after(40, function(d2) talk(d2, SPEAKER_MASTER, 109) end)
        after(BLAST_FIRST, blast)
        after(CALL_FIRST, call)
    end)
end

function onUpdate(director, health, player_health, info)
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
    runTimers(director)
    if over then
        return
    end
    BATTLE:update(director, health, player_health, info)
    local creature = (health or {})[BNPC_CREATURE]
    if creature ~= nil then
        if phase == SEQUENCE_FIGHTING and not noticing and creature <= CREATURE_HURT_AT then
            noticing = true
            after(HEALTH_NOTICED_AFTER, function(d)
                noticing = false
                sequence(d, SEQUENCE_CREATURE_HURT)
                after(10, function(d2) talk(d2, SPEAKER_MASTER, 105) end)
            end)
        elseif phase == SEQUENCE_CREATURE_HURT and not noticing and creature <= CREATURE_SPENT_AT then
            noticing = true
            after(HEALTH_NOTICED_AFTER, function(d)
                noticing = false
                if phase == SEQUENCE_CREATURE_HURT then
                    creatureSpent(d)
                end
            end)
        end
    end
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    if layout_id == BNPC_ADD then
        adds = math.max(adds - 1, 0)
        BATTLE.alive[BNPC_ADD] = adds > 0 or nil
    elseif layout_id == BNPC_CREATURE and phase < SEQUENCE_CREATURE_SPENT then
        -- Killed outright before the master could take it back: on to the master alone.
        creatureSpent(director)
    elseif layout_id == BNPC_MASTER then
        -- Retail's sequence and "Duty Complete" came as its health reached 0.
        over = true
        timers = {}
        director:set_sequence(SEQUENCE_WON)
        director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS)
    end
end

-- Losing sends you back to where you entered, still on the same step of the quest, so you can try again.
function onPlayerDeath(director)
    if over then
        return
    end
    over = true
    timers = {}
    director:set_sequence(SEQUENCE_FAILED)
    fail_in = FAIL_DELAY_UPDATES
end
