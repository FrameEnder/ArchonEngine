-- Quest battle 25: Axe in the Stone (ClsExc050_00313, the Marauders' level 5 quest): four waves of megalocrabs.
-- It takes place in territory 262, taken to be a copy of Lower La Noscea (the quest's megalocrab nest and its event
-- range are Lower La Noscea's, and retail put the player by them), so its NPCs live in the sea_s1f2 drop-in (layer
-- QB_ClsExc050_001): layout IDs, levels, HP, positions, gear and flags are retail's (Bob's Chronofoil capture
-- 0cafbf61 of 2026-11-17, `questkit chronofoil --battle 0.15`, ContentFinderCondition 328: one run, won in 110 s, a
-- Marauder of 8). A conjurer (4107368, base 713, level 35, 1220 HP) fights beside you from the start, a marauder
-- (4107369, base 715, level 40, 1601 HP) from the third wave; the small crabs (base 714, level 5, 65 HP) go for you,
-- the big ones (base 957, level 15, 797 HP) for the allies: the first two waves' for the conjurer, the later ones'
-- for the marauder.
--
-- Timed against the run (seconds from the fight starting, the director's scene 60000):
--   0      sequence 2; the conjurer, a big crab (4107432) and two small ones (4107370, 4107372) are there from the
--          zone-in, and nobody moves yet
--   3.1    sequence 3: they set on each other; the conjurer's 84 5.1 s later and 85 5.0 s after that
--   two small crabs down (36.6): sequence 4 and the second wave 0.5 s later (a big one, three small)
--   four down (57.7): sequence 5 and the third wave (two big, three small) with the marauder, who uses Defiance at
--          once; its 86 10.0 s after the sequence
--   eight down (84.6): sequence 6 and the fourth wave (a big one, three small); the marauder's 87 5.1 s later
--   everyone down (109.5): sequence 7 and "Duty Complete", then the battle's scene 7 (0xF8402EFB) 3.6 s later; the
--          quest moves to 255
-- What moves it on is read from this one run: each sequence came as a small crab fell (the second, fourth and
-- eighth), with big ones still up at the fourth and the eighth and one small one still up at the fourth, so the count
-- of small crabs fits and "the wave is down" doesn't; a count of all enemies (three, five, ten) fits as well.
-- Waves that come stand 5 s before they set off (Battle's arrivals.)

-- Where retail puts the player and which way they face: their arrival in the capture.
START_POSITION = { x = 158.82, y = 33.73, z = 89.79 }
START_ROTATION = 0.436
-- Retail's opening scene 1 here.
OPENING_SCENE_FLAGS = 0xF841AEFB

BNPC_CONJURER = 4107368
BNPC_MARAUDER = 4107369
SMALL = "small"
BIG = "big"
-- Each wave: its big crabs (and whom they go for), its small ones, and how many small ones down bring the next.
WAVES = {
    { big = { 4107432 }, small = { 4107370, 4107372 }, attacks = BNPC_CONJURER, next_at = 2 },
    { big = { 4107433 }, small = { 4107418, 4107419, 4107420 }, attacks = BNPC_CONJURER, next_at = 4, sequence = 4 },
    { big = { 4107434, 4107435 }, small = { 4107421, 4107422, 4107423 }, attacks = BNPC_MARAUDER, next_at = 8,
        sequence = 5 },
    { big = { 4107437 }, small = { 4107426, 4107427, 4107428 }, attacks = BNPC_MARAUDER, sequence = 6 },
}
MARAUDER_COMES_WITH = 3
ACTION_DEFIANCE = 48

QUEST_SEQUENCE_AFTER_WIN = 255
SEQUENCE_START = 1
SEQUENCE_FIGHTING = 2
SEQUENCE_ENGAGED = 3
SEQUENCE_WON = 7
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 7
FINISH_SCENE_FLAGS = 0xF8402EFB
FAIL_DELAY_UPDATES = 12

-- Timers, in updates (half a second each.)
ENGAGE_AFTER = 7
WAVE_AFTER = 1
DEATH_NOTICED_AFTER = 4

-- The battle talk (the content's text IDs and the speakers' BNpcNames, retail's; no portrait.)
SPEAKER_CONJURER = { layout = BNPC_CONJURER, name = 824, icon = 0 }
SPEAKER_MARAUDER = { layout = BNPC_MARAUDER, name = 826, icon = 0 }

ACTION_CURE = 120
-- Its one Cure on the player came at 60 %.
HEAL_BELOW_HEALTH = 60
HEAL_EVERY_SECONDS = 12

BATTLE = Battle.new()
BATTLE:ally(BNPC_CONJURER, {
    role = "support",
    focus = { BIG, SMALL },
    range = 14,
    heals = {
        { action = ACTION_CURE, targets = { Battle.PLAYER }, below = HEAL_BELOW_HEALTH, every = HEAL_EVERY_SECONDS },
    },
})
BATTLE:ally(BNPC_MARAUDER, { role = "tank", focus = { BIG, SMALL } })
for n, wave in ipairs(WAVES) do
    -- The first wave stands there from the start and waits for sequence 3.
    local waits = n == 1
    for _, layout in ipairs(wave.big) do
        BATTLE:enemy(layout, { group = BIG, attacks = wave.attacks, passive = waits, unaware = waits })
    end
    for _, layout in ipairs(wave.small) do
        BATTLE:enemy(layout, { group = SMALL, passive = waits, unaware = waits })
    end
end

local over = false
local fail_in = nil
local small_down = 0
local waves_out = 1
-- Whether the last wave is out (it comes a moment after its sequence.)
local waves_came = false
local timers = {}
local is_small = {}
for _, wave in ipairs(WAVES) do
    for _, layout in ipairs(wave.small) do
        is_small[layout] = true
    end
end

-- Timers of their own (not Battle:steps, which queues each after the last.)
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

function talk(director, speaker, text)
    director:battle_talk(speaker.layout, speaker.name, text, speaker.icon)
end

local function bringWave(director, n)
    local wave = WAVES[n]
    if n == MARAUDER_COMES_WITH then
        BATTLE:spawn(director, { BNPC_MARAUDER })
        director:use_action(BNPC_MARAUDER, ACTION_DEFIANCE, BNPC_MARAUDER)
    end
    BATTLE:spawn(director, wave.big)
    BATTLE:spawn(director, wave.small)
    waves_came = n == #WAVES
end

function onSetup(director)
    director:set_sequence(SEQUENCE_START)
end

function onCommence(director)
    director:set_sequence(SEQUENCE_FIGHTING)
    BATTLE:holdBack({ BNPC_CONJURER })
    BATTLE:spawn(director, { BNPC_CONJURER })
    bringWave(director, 1)
    after(ENGAGE_AFTER, function(d)
        d:set_sequence(SEQUENCE_ENGAGED)
        BATTLE:engage(d, WAVES[1].big, BNPC_CONJURER)
        BATTLE:engage(d, WAVES[1].small, nil)
        BATTLE:release(d, { BNPC_CONJURER })
        after(10, function(d2) talk(d2, SPEAKER_CONJURER, 84) end)
        after(20, function(d2) talk(d2, SPEAKER_CONJURER, 85) end)
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
end

-- A small crab down counts, and the last enemy's fall wins, 2 s after its health reaches 0 (its death animation, as
-- retail waits in the other battles: each sequence here came 1.6 to 2.1 s after the health that brought it.)
local function smallDown(director)
    small_down = small_down + 1
    local wave = WAVES[waves_out]
    if wave.next_at ~= nil and small_down >= wave.next_at then
        local n = waves_out + 1
        waves_out = n
        director:set_sequence(WAVES[n].sequence)
        after(WAVE_AFTER, function(d) bringWave(d, n) end)
        if n == 3 then
            after(20, function(d) talk(d, SPEAKER_MARAUDER, 86) end)
        elseif n == 4 then
            after(10, function(d) talk(d, SPEAKER_MARAUDER, 87) end)
        end
    end
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    if is_small[layout_id] then
        after(DEATH_NOTICED_AFTER, smallDown)
    end
    if waves_out == #WAVES and waves_came and BATTLE:enemiesLeft() == 0 then
        after(DEATH_NOTICED_AFTER, function(d)
            over = true
            timers = {}
            d:set_sequence(SEQUENCE_WON)
            d:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS)
        end)
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
