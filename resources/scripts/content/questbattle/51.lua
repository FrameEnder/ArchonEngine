-- Quest battle 51: Over the Rails (ClsAcn150_00458, the Arcanists' level 15 quest.)
-- It takes place in territory 286, aboard the Morningstar, whose level is ocn_o1/evt/o1e2, so its NPCs are layer
-- QB_ClsAcn150_001 of the ocn_o1e2 drop-in: on your side Foreseer K'lyhia (4256134, base 2055, 590 HP), Geissfryn
-- Eveningcalm (4256136, 2053, a conjurer) and Aersthota Morningcalm (4256137, 2054, a marauder), later K'lyhia's Topaz
-- Carbuncle (4256201, 2056); against you the crew: three boatswains (2050) and four falmadairs (2051, gladiators) on
-- deck and the navigator (4256168, 2052, a marauder, 1032 HP) above, then two waves. Layout IDs, levels, HP,
-- positions, display and character data flags, looks and the start spot are retail's (Bob's Kawari Recorder run of
-- 2026-11-12, `questkit battle`: one run, won, a level 19 arcanist.)
--
-- Retail's flow (seconds from the director's scene 60000):
--   0       sequence 2 (0.2 s after); the boatswains on K'lyhia, two falmadairs each on Aersthota and Geissfryn; the
--           navigator waits until someone hits him; K'lyhia's battle talk 99 and 100 at 3.1 and 8.1
--   K'lyhia's Summon II (1307, a 5.7 s cast 7.1 s in): her Topaz Carbuncle 7.35 s after it began
--   six of the first seven down (the boatswains, then three falmadairs; 32.6): sequence 3 at once, the second wave
--           0.7 s later (gunners 4256183-4, falmadairs 4256170-1); talk 103 15.1 s after sequence 3
--   the navigator down (63.6): sequence 4 at once, the third wave 0.3 s later (falmadairs 4256187-8, a gromet
--           4256194)
--   everything down: "Duty Complete" (60001) and sequence 5 1.0 s after the last fell, the battle's scene 5
--           (0xF8402EFB) 3.6 s after that; the quest moves to 255
-- Talks 101 and 102 weren't said in the run. What each one does is in their timelines (QB51*).

-- Where retail put the player and which way they faced.
START_POSITION = { x = 6.78, y = 16.35, z = 0.91 }
START_ROTATION = -2.64
-- Retail's opening scene 1 here.
OPENING_SCENE_FLAGS = 0xF841AEFB

BNPC_KLYHIA = 4256134
BNPC_GEISSFRYN = 4256136
BNPC_AERSTHOTA = 4256137
BNPC_CARBUNCLE = 4256201
BNPC_NAVIGATOR = 4256168
BOATSWAINS = { 4256144, 4256145, 4256146 }
FALMADAIRS_ON_AERSTHOTA = { 4256163, 4256167 }
FALMADAIRS_ON_GEISSFRYN = { 4256162, 4256164 }
WAVE_2 = { 4256183, 4256184, 4256170, 4256171 }
WAVE_2_TARGETS = {
    [4256183] = BNPC_GEISSFRYN,
    [4256184] = BNPC_KLYHIA,
    [4256170] = BNPC_AERSTHOTA,
    [4256171] = BNPC_AERSTHOTA,
}
WAVE_3 = { 4256187, 4256188, 4256194 }
WAVE_3_TARGETS = { [4256187] = BNPC_GEISSFRYN, [4256188] = BNPC_AERSTHOTA, [4256194] = BNPC_GEISSFRYN }
-- How many of the first seven (the deck's crew) bring the second wave.
FIRST_DOWN_FOR_WAVE_2 = 6

QUEST_SEQUENCE_AFTER_WIN = 255
SEQUENCE_FIGHTING = 2
SEQUENCE_SECOND_WAVE = 3
SEQUENCE_THIRD_WAVE = 4
SEQUENCE_WON = 5
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 5
FINISH_SCENE_FLAGS = 0xF8402EFB
FAIL_DELAY_UPDATES = 12

ACTION_SUMMON_II = 1307
ACTION_PHYSICK = 190
ACTION_CURE = 120

-- K'lyhia's battle talk (the content's text IDs, retail's), in updates (half a second each) after what brings it. A
-- timer of n set before the next update (onCommence, onActorDeath) fires (n - 1) / 2 s later.
SPEAKER_KLYHIA = { layout = BNPC_KLYHIA, name = 1493, icon = 0 }
TALKS_FIGHTING = { { 7, 99 }, { 17, 100 } }
TALKS_SECOND_WAVE = { { 31, 103 } }
WAVE_2_AFTER = 2
WAVE_3_AFTER = 2
CARBUNCLE_AFTER_CAST = 15
WON_AFTER_DEATH = 3

BATTLE = Battle.new()
-- Aersthota holds the crew off the others; Geissfryn and K'lyhia fight from range and heal her (retail's Physick at
-- 43-66 %, Cure at 49 %); the carbuncle takes the oldest one up close.
BATTLE:ally(BNPC_AERSTHOTA, { role = "tank", focus = { "adds", "boss" }, range = 3 })
BATTLE:ally(BNPC_GEISSFRYN, {
    role = "dps",
    focus = { "adds", "boss" },
    range = 20,
    keep_near = { within = 10 },
    heals = { { action = ACTION_CURE, targets = { BNPC_AERSTHOTA }, below = 54, every = 12 } },
})
BATTLE:ally(BNPC_KLYHIA, {
    role = "dps",
    focus = { "adds", "boss" },
    range = 10,
    heals = { { action = ACTION_PHYSICK, targets = { BNPC_AERSTHOTA }, below = 60, every = 12 } },
})
BATTLE:ally(BNPC_CARBUNCLE, { role = "dps", focus = { "adds", "boss" }, order = "oldest", range = 3 })
for _, layout in ipairs(BOATSWAINS) do
    BATTLE:enemy(layout, { group = "adds", attacks = BNPC_KLYHIA })
end
for _, layout in ipairs(FALMADAIRS_ON_AERSTHOTA) do
    BATTLE:enemy(layout, { group = "adds", attacks = BNPC_AERSTHOTA })
end
for _, layout in ipairs(FALMADAIRS_ON_GEISSFRYN) do
    BATTLE:enemy(layout, { group = "adds", attacks = BNPC_GEISSFRYN })
end
BATTLE:enemy(BNPC_NAVIGATOR, { group = "boss", passive = true })
for layout, ally in pairs(WAVE_2_TARGETS) do
    BATTLE:enemy(layout, { group = "adds", attacks = ally })
end
for layout, ally in pairs(WAVE_3_TARGETS) do
    BATTLE:enemy(layout, { group = "adds", attacks = ally })
end

local over = false
local winning = false
local fail_in = nil
local phase = 1
local first_down = 0
local navigator_down = false
local carbuncle_out = false
-- The first seven, and every enemy out and not yet down.
local first_seven = {}
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

function talks(lines)
    for _, line in ipairs(lines) do
        local text = line[2]
        after(line[1], function(d)
            d:battle_talk(SPEAKER_KLYHIA.layout, SPEAKER_KLYHIA.name, text, SPEAKER_KLYHIA.icon)
        end)
    end
end

function bringOut(director, layouts)
    for _, layout in ipairs(layouts) do
        standing[layout] = true
    end
    BATTLE:spawn(director, layouts)
end

function sequence(director, value)
    phase = value
    director:set_sequence(value)
end

function secondWave(director)
    sequence(director, SEQUENCE_SECOND_WAVE)
    after(WAVE_2_AFTER, function(d) bringOut(d, WAVE_2) end)
    talks(TALKS_SECOND_WAVE)
end

function thirdWave(director)
    sequence(director, SEQUENCE_THIRD_WAVE)
    after(WAVE_3_AFTER, function(d) bringOut(d, WAVE_3) end)
end

function onSetup(director)
    director:set_sequence(1)
end

function onCommence(director)
    sequence(director, SEQUENCE_FIGHTING)
    BATTLE:spawn(director, { BNPC_KLYHIA, BNPC_GEISSFRYN, BNPC_AERSTHOTA })
    for _, group in ipairs({ BOATSWAINS, FALMADAIRS_ON_AERSTHOTA, FALMADAIRS_ON_GEISSFRYN }) do
        for _, layout in ipairs(group) do
            first_seven[layout] = true
        end
        bringOut(director, group)
    end
    bringOut(director, { BNPC_NAVIGATOR })
    talks(TALKS_FIGHTING)
end

-- Her Summon II brings her Topaz Carbuncle.
function onActorCast(director, layout_id, action_id)
    if over or layout_id ~= BNPC_KLYHIA or action_id ~= ACTION_SUMMON_II or carbuncle_out then
        return
    end
    carbuncle_out = true
    after(CARBUNCLE_AFTER_CAST, function(d) BATTLE:spawn(d, { BNPC_CARBUNCLE }) end)
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
    if not standing[layout_id] then
        return
    end
    standing[layout_id] = nil
    if first_seven[layout_id] then
        first_seven[layout_id] = nil
        first_down = first_down + 1
        if first_down == FIRST_DOWN_FOR_WAVE_2 and phase == SEQUENCE_FIGHTING then
            secondWave(director)
            -- The navigator fell first: the third wave right behind the second.
            if navigator_down then
                thirdWave(director)
            end
        end
    end
    if layout_id == BNPC_NAVIGATOR then
        navigator_down = true
        if phase == SEQUENCE_SECOND_WAVE then
            thirdWave(director)
        end
    end
    -- Won once the third wave has come and every one of the crew is down.
    if phase >= SEQUENCE_THIRD_WAVE and next(standing) == nil and not winning then
        winning = true
        timers = {}
        after(WON_AFTER_DEATH, won)
    end
end

function won(director)
    over = true
    timers = {}
    sequence(director, SEQUENCE_WON)
    director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS)
end

-- Losing sends you back to where you entered, still on the same step of the quest, so you can try again.
function onPlayerDeath(director)
    if over or winning then
        return
    end
    over = true
    timers = {}
    director:set_sequence(SEQUENCE_FAILED)
    fail_in = FAIL_DELAY_UPDATES
end
