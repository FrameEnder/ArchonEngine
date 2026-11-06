-- Quest battle 20: That Old Familiar Feeling (ClsGla150_00262, the Gladiators' level 15 quest.)
-- It takes place in territory 254, a copy of Ul'dah - Steps of Nald (retail's positions in it stand on Steps of
-- Nald's ground, to the hundredth of a yalm, against the same capture's visits to 130), so its NPCs live in the
-- wil_w1t1 drop-in (layer QB_ClsGla150_002). Levels, HP, positions, gear, display and character data flags and the
-- start spot are retail's (Bob's Chronofoil capture 941895d2 of 2026-11-06, `questkit chronofoil --battle 0.7`: one
-- run, won; a gladiator synced to 19): the ally (3993254, base 629, BNpcName 690, level 50, 5178 HP, the battle
-- talk's speaker); five archers out from the start (3993614-3993618, base 669, 1845 HP), all on him; then four more
-- on him (3993218-3993221, base 630) and two on you (3993250-3993251, base 631); then three more on him
-- (3993388-3993390, base 630) and four on you (3993399 and 3993528, base 632; 3993529 and 3993544, base 670.)
--
-- Timed against the run (seconds from the fight starting, the director's scene 60000):
--   0       sequence 2: the ally and the first five
--   5.1     battle talk 122 (the ally, no portrait)
--   the second of the first five down (32.7; the director heard of it 2.0 s later, its death animation): sequence 3
--           and talk 123 at once
--   30.1 s after sequence 3 (64.8): sequence 4, the second wave 0.45 s later
--   10.1 s after sequence 4 (74.9): sequence 10 and talk 124; talk 125 15.1 s after that
--   the fifth of the second wave down (106.8, at once, as Big Trouble in Little Ala Mhigo's second wave): sequence 5,
--           the third wave 0.44 s later, talk 126 5.1 s after sequence 5
--   every enemy down (162.0, heard 2.0 s later): "Duty Complete" (60001) and sequence 6, then the battle's scene 6
--           (a cutscene, 0xF8402EFB) 3.6 s later; the quest moves to 255

-- Where retail puts the player and which way they face: their SpawnPlayer in the capture.
START_POSITION = { x = 28.64, y = 7.21, z = -100.73 }
START_ROTATION = 2.876
-- Retail's opening scene 1 here (Kawari's default too, written out.)
OPENING_SCENE_FLAGS = 0xF841AEFB

BNPC_ALLY = 3993254
WAVE_1 = { 3993614, 3993615, 3993616, 3993617, 3993618 }
WAVE_2 = { 3993218, 3993219, 3993220, 3993221, 3993250, 3993251 }
WAVE_3 = { 3993388, 3993389, 3993390, 3993399, 3993528, 3993529, 3993544 }
-- Whom each of the later waves goes for first, as retail's did (the rest: the player.)
ON_ALLY = {
    [3993218] = true, [3993219] = true, [3993220] = true, [3993221] = true,
    [3993388] = true, [3993389] = true, [3993390] = true,
}

QUEST_SEQUENCE_AFTER_WIN = 255
SEQUENCE_FIGHTING = 2
SEQUENCE_FIRST_DOWN = 3
SEQUENCE_SECOND_WAVE = 4
SEQUENCE_SECOND_WAVE_TALK = 10
SEQUENCE_THIRD_WAVE = 5
SEQUENCE_WON = 6
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 6
FINISH_SCENE_FLAGS = 0xF8402EFB
FAIL_DELAY_UPDATES = 12

-- How many of the first five down bring sequence 3, and of the second wave the third.
FIRST_WAVE_DOWN = 2
SECOND_WAVE_DOWN = 5

-- Timers, in updates (half a second each.)
DEATH_NOTICED_AFTER = 4
SECOND_WAVE_AFTER_SEQUENCE_3 = 60
WAVE_OUT_AFTER = 1
SEQUENCE_10_AFTER = 20
TALK_START_AFTER = 10
TALK_125_AFTER = 30
TALK_126_AFTER = 10
-- A timer started from onActorDeath counts the update it was started in, so one more.
FROM_DEATH = 1

-- The battle talk (the content's text IDs, retail's.)
SPEAKER_ALLY = { layout = BNPC_ALLY, name = 690, icon = 0 }
TALK_START = 122
TALK_FIRST_DOWN = 123
TALK_SECOND_WAVE = 124
TALK_SECOND_WAVE_LATER = 125
TALK_THIRD_WAVE = 126

BATTLE = Battle.new()
-- On retail he took on whichever enemy last hit him (21 of his 23 changes), up close, the whole fight.
BATTLE:ally(BNPC_ALLY, { role = "dps", split = false, focus = { "adds" }, order = "nearest" })
for _, layout in ipairs(WAVE_1) do
    BATTLE:enemy(layout, { group = { "wave1", "adds" }, attacks = BNPC_ALLY })
end
for _, layout in ipairs(WAVE_2) do
    BATTLE:enemy(layout, { group = { "wave2", "adds" }, attacks = ON_ALLY[layout] and BNPC_ALLY or nil })
end
for _, layout in ipairs(WAVE_3) do
    BATTLE:enemy(layout, { group = { "wave3", "adds" }, attacks = ON_ALLY[layout] and BNPC_ALLY or nil })
end

local over = false
local fail_in = nil
local phase = 1
local first_down = 0
local second_down = 0
local third_wave_out = false
-- The enemies out and not yet heard to be down.
local standing = {}
local timers = {}

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
    director:battle_talk(speaker.layout, speaker.name, text, speaker.icon, nil, speaker.kind)
end

function bringOut(director, layouts)
    for _, layout in ipairs(layouts) do
        standing[layout] = true
    end
    BATTLE:spawn(director, layouts)
end

function inList(list, layout)
    for _, other in ipairs(list) do
        if other == layout then
            return true
        end
    end
    return false
end

function onSetup(director)
    director:set_sequence(1)
end

function onCommence(director)
    phase = SEQUENCE_FIGHTING
    director:set_sequence(SEQUENCE_FIGHTING)
    BATTLE:spawn(director, { BNPC_ALLY })
    bringOut(director, WAVE_1)
    after(TALK_START_AFTER, function(d) talk(d, SPEAKER_ALLY, TALK_START) end)
end

-- Sequence 3, then the second wave on a timer (it came 30 s later, whatever the first five were doing.)
function firstDown(director)
    phase = SEQUENCE_FIRST_DOWN
    director:set_sequence(SEQUENCE_FIRST_DOWN)
    talk(director, SPEAKER_ALLY, TALK_FIRST_DOWN)
    after(SECOND_WAVE_AFTER_SEQUENCE_3, function(d)
        phase = SEQUENCE_SECOND_WAVE
        d:set_sequence(SEQUENCE_SECOND_WAVE)
        after(WAVE_OUT_AFTER, function(d2) bringOut(d2, WAVE_2) end)
        after(SEQUENCE_10_AFTER, function(d2)
            if phase ~= SEQUENCE_SECOND_WAVE then
                return
            end
            phase = SEQUENCE_SECOND_WAVE_TALK
            d2:set_sequence(SEQUENCE_SECOND_WAVE_TALK)
            talk(d2, SPEAKER_ALLY, TALK_SECOND_WAVE)
            after(TALK_125_AFTER, function(d3)
                if phase == SEQUENCE_SECOND_WAVE_TALK then
                    talk(d3, SPEAKER_ALLY, TALK_SECOND_WAVE_LATER)
                end
            end)
        end)
    end)
end

function thirdWave(director)
    phase = SEQUENCE_THIRD_WAVE
    director:set_sequence(SEQUENCE_THIRD_WAVE)
    after(WAVE_OUT_AFTER + FROM_DEATH, function(d)
        third_wave_out = true
        bringOut(d, WAVE_3)
    end)
    after(TALK_126_AFTER + FROM_DEATH, function(d) talk(d, SPEAKER_ALLY, TALK_THIRD_WAVE) end)
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
    BATTLE:update(director, health, player_health, info)
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    -- The second wave's fifth brought the third at once on retail, not 2 s later.
    if inList(WAVE_2, layout_id) then
        second_down = second_down + 1
        if second_down >= SECOND_WAVE_DOWN and (phase == SEQUENCE_SECOND_WAVE or phase == SEQUENCE_SECOND_WAVE_TALK) then
            thirdWave(director)
        end
    end
    after(DEATH_NOTICED_AFTER, function(d) deathNoticed(d, layout_id) end)
end

-- The director hears of a death once its animation has played (2.0 s after the health reached 0, both times it acted
-- on one here.)
function deathNoticed(director, layout_id)
    if over then
        return
    end
    standing[layout_id] = nil
    if inList(WAVE_1, layout_id) then
        first_down = first_down + 1
        if phase == SEQUENCE_FIGHTING and first_down >= FIRST_WAVE_DOWN then
            firstDown(director)
        end
    end
    if third_wave_out and next(standing) == nil then
        over = true
        timers = {}
        phase = SEQUENCE_WON
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
