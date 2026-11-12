-- Quest battle 50: Tactical Planning (ClsAcn100_00456, the Arcanists' level 10 quest.)
-- It takes place in territory 285, whose level is Middle La Noscea's (s1f1), so its NPCs are layer QB_ClsAcn100_001 of
-- sea_s1f1: Foreseer K'lyhia on your side (4255768, base 2048, 364 HP) and later her Topaz Carbuncle (4255788, 2049);
-- against you goblin picaroons (2047, 192 HP) and goblin traders (2046, 249 HP) in three waves. Layout IDs, levels,
-- HP, positions, display and character data flags, her looks and the start spot are retail's (Bob's Kawari Recorder
-- run of 2026-11-12, `questkit battle`: one run, won, a level 14 arcanist.)
--
-- Retail's flow (seconds from sequence 2, which came 0.5 s after the director's scene 60000):
--   0       sequence 2: three picaroons (4255772-4); K'lyhia's battle talk 86 and 87 at 4.6 and 9.6
--   two of them down: sequence 3 at once and the second wave 0.3 s later (a trader 4255783, picaroons 4255781-2);
--           talks 88 and 89 4.6 and 5.1 s after sequence 3, sequence 4 10.4 s after it (nothing else then)
--   K'lyhia's first Summon II (1307, a 5.7 s cast 6.7 s in) ended: her Topaz Carbuncle 1.3 s later
--   both of the second wave's picaroons down: sequence 5 and talk 92 at once, the third wave 0.6 s later (a trader
--           4255785, a picaroon 4255787), talk 90 0.8 s after, 93 and 91 5.1 and 5.8 s after, 94 10.0 s after;
--           sequence 6 15.1 s after sequence 5
--   everything down: "Duty Complete" (60001) and sequence 7 2.0 s after the last fell, the battle's scene 7
--           (0xF8402EFB) 3.6 s after that; the quest moves to sequence 5
-- Talks come every 5 s or so of a sequence (each is 4 s on screen: the client queues them.) What each one does is in
-- their timelines (QB50*).

-- Where retail put the player and which way they faced.
START_POSITION = { x = -110.38, y = 45.38, z = -245.45 }
START_ROTATION = 2.43
-- Retail's opening scene 1 here.
OPENING_SCENE_FLAGS = 0xF8412FF7

BNPC_KLYHIA = 4255768
BNPC_CARBUNCLE = 4255788
WAVE_1 = { 4255774, 4255773, 4255772 }
WAVE_2_PICAROONS = { 4255781, 4255782 }
WAVE_2_TRADER = 4255783
WAVE_3 = { 4255785, 4255787 }
-- How many of the first wave bring the second.
WAVE_1_DOWN_FOR_WAVE_2 = 2

QUEST_SEQUENCE_AFTER_WIN = 5
SEQUENCE_FIGHTING = 2
SEQUENCE_SECOND_WAVE = 3
SEQUENCE_PRECONDITION = 4
SEQUENCE_THIRD_WAVE = 5
SEQUENCE_ON_YOUR_OWN = 6
SEQUENCE_WON = 7
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 7
FINISH_SCENE_FLAGS = 0xF8402EFB
FAIL_DELAY_UPDATES = 12

ACTION_SUMMON_II = 1307

-- K'lyhia's battle talk (the content's text IDs, retail's) and when, in updates (half a second each) after what brings
-- it. A timer of n set in onUpdate fires n / 2 s later, one set before the next update (onCommence, onActorDeath)
-- (n - 1) / 2 s later.
SPEAKER_KLYHIA = { layout = BNPC_KLYHIA, name = 1493, icon = 0 }
TALKS_FIGHTING = { { 11, 86 }, { 21, 87 } }
TALKS_SECOND_WAVE = { { 10, 88 }, { 11, 89 } }
TALKS_THIRD_WAVE = { { 1, 92 }, { 3, 90 }, { 11, 93 }, { 13, 91 }, { 21, 94 } }
SEQUENCE_4_AFTER = 21
SEQUENCE_6_AFTER = 31
WAVE_2_AFTER = 1
WAVE_3_AFTER = 2
-- Her carbuncle came out 1.3 s after her first Summon II's 5.7 s cast ended.
CARBUNCLE_AFTER_CAST = 15
WON_AFTER_DEATH = 5

BATTLE = Battle.new()
-- K'lyhia and her carbuncle go for the oldest goblin out, a summoner from range and her carbuncle up close.
BATTLE:ally(BNPC_KLYHIA, { role = "dps", focus = { "goblins" }, order = "oldest", range = 10 })
BATTLE:ally(BNPC_CARBUNCLE, { role = "dps", focus = { "goblins" }, order = "oldest", range = 3 })
for _, layout in ipairs(WAVE_1) do
    BATTLE:enemy(layout, { group = { "goblins", "wave1" } })
end
for _, layout in ipairs(WAVE_2_PICAROONS) do
    BATTLE:enemy(layout, { group = { "goblins", "wave2_picaroons" } })
end
BATTLE:enemy(WAVE_2_TRADER, { group = "goblins" })
for _, layout in ipairs(WAVE_3) do
    BATTLE:enemy(layout, { group = "goblins" })
end

local over = false
local winning = false
local fail_in = nil
local phase = 1
local wave_1_down = 0
local carbuncle_out = false
-- The goblins out and not yet down.
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

function talk(director, text)
    director:battle_talk(SPEAKER_KLYHIA.layout, SPEAKER_KLYHIA.name, text, SPEAKER_KLYHIA.icon)
end

function talks(lines)
    for _, line in ipairs(lines) do
        local text = line[2]
        after(line[1], function(d) talk(d, text) end)
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

-- Sequences 4 and 6 come with time, if the fight hasn't moved past them.
function sequenceAfter(updates, value, before)
    after(updates, function(d)
        if phase < value and phase >= before then
            sequence(d, value)
        end
    end)
end

-- Two of the first wave down: the second.
function waveOneDown(director)
    wave_1_down = wave_1_down + 1
    if wave_1_down ~= WAVE_1_DOWN_FOR_WAVE_2 or phase ~= SEQUENCE_FIGHTING then
        return
    end
    sequence(director, SEQUENCE_SECOND_WAVE)
    after(WAVE_2_AFTER, function(d) bringOut(d, { WAVE_2_TRADER, WAVE_2_PICAROONS[1], WAVE_2_PICAROONS[2] }) end)
    talks(TALKS_SECOND_WAVE)
    sequenceAfter(SEQUENCE_4_AFTER, SEQUENCE_PRECONDITION, SEQUENCE_SECOND_WAVE)
end

-- The second wave's picaroons down: the third.
BATTLE:onCleared("wave2_picaroons", function(director)
    if over or phase < SEQUENCE_SECOND_WAVE or phase >= SEQUENCE_THIRD_WAVE then
        return
    end
    sequence(director, SEQUENCE_THIRD_WAVE)
    after(WAVE_3_AFTER, function(d) bringOut(d, WAVE_3) end)
    talks(TALKS_THIRD_WAVE)
    sequenceAfter(SEQUENCE_6_AFTER, SEQUENCE_ON_YOUR_OWN, SEQUENCE_THIRD_WAVE)
end)

function onSetup(director)
    director:set_sequence(1)
end

function onCommence(director)
    sequence(director, SEQUENCE_FIGHTING)
    BATTLE:spawn(director, { BNPC_KLYHIA })
    bringOut(director, WAVE_1)
    talks(TALKS_FIGHTING)
end

-- Her first Summon II brings her Topaz Carbuncle.
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
    for _, layout in ipairs(WAVE_1) do
        if layout == layout_id then
            waveOneDown(director)
        end
    end
    -- Won once the third wave has come and every goblin is down (its talks, if any are left, go unsaid.)
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
