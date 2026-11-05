-- Quest battle 19: Ul'dah's Most Wanted (ClsGla100_00261, the Gladiators' level 10 quest.)
-- It takes place in territory 253, whose TerritoryType has Central Thanalan's level (w1f2), so its NPCs live in the
-- wil_w1 drop-in (layer QB_ClsGla100_002), with the layout IDs the QuestBattle sheet names: the gladiator on your side
-- (INIT_P_POP_01, base 625, BNpcName 924, level 10, 211 HP), three thugs (INIT_POP_01_01-03, base 626), then two more
-- and a tougher one (INIT_POP_02_01-03, 626 and 627), then their leader (INIT_POP_03_01, base 628, 672 HP). Levels,
-- HP, positions, display and character data flags and the start spot are retail's (Bob's Chronofoil capture a763f0a0
-- of 2026-11-05, `questkit chronofoil --battle 0.5`: one run, won; a level 14 gladiator synced to 14.)
--
-- Timed against the run (seconds from the fight starting, the director's scene 60000; deaths as the director heard
-- of them, 2 s after the health reached 0, `DEATH_NOTICED_AFTER`):
--   0       sequence 2: the three thugs on the gladiator
--   5.9     battle talk 107 (the gladiator, no portrait)
--   the second of the three down (18.4): sequence 3, the next three 0.6 s later, on the gladiator; talk 108 10.1 s
--           after sequence 3
--   all six down (47.3): sequence 4 and talk 103 at once; 5.0 s later sequence 5 and their leader (0.2 s after it),
--           on the gladiator, who holds back until sequence 6 5.1 s later; talk 104 2.0 s after that
--   the leader down (77.6): "Duty Complete" (60001) and sequence 7, then the battle's scene 7 (0x2001) 3.6 s later;
--           the quest moves to 5
-- The gladiator Cures himself at 25-35 % (retail's at 52 and 74 of his 211.)

-- Where retail puts the player and which way they face: their SpawnPlayer in the capture.
START_POSITION = { x = 27.35, y = 13.0, z = 51.79 }
START_ROTATION = -2.571
-- Retail's opening scene 1 here: no cutscene (over in 0.3 s.)
OPENING_SCENE_FLAGS = 0x2001

BNPC_GLADIATOR = INIT_P_POP_01 or 3977801
WAVE_1 = { INIT_POP_01_01 or 3977803, INIT_POP_01_02 or 3977804, INIT_POP_01_03 or 3977805 }
WAVE_2 = { INIT_POP_02_01 or 3977807, INIT_POP_02_02 or 3977808, INIT_POP_02_03 or 3977809 }
BNPC_LEADER = INIT_POP_03_01 or 3983981

QUEST_SEQUENCE_AFTER_WIN = 5
SEQUENCE_FIGHTING = 2
SEQUENCE_SECOND_WAVE = 3
SEQUENCE_WAVES_DOWN = 4
SEQUENCE_LEADER = 5
SEQUENCE_LEADER_FIGHT = 6
SEQUENCE_WON = 7
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 7
FINISH_SCENE_FLAGS = 0x2001
FAIL_DELAY_UPDATES = 12

-- How many of the first three down bring the second wave.
SECOND_WAVE_AFTER_DOWN = 2

-- Timers, in updates (half a second each.)
DEATH_NOTICED_AFTER = 4
SECOND_WAVE_AFTER = 1
LEADER_AFTER = 10
LEADER_FIGHT_AFTER = 10
TALK_START_AFTER = 12
TALK_SECOND_WAVE_AFTER = 20
TALK_LEADER_FIGHT_AFTER = 4

-- The battle talk (the content's text IDs, retail's.)
SPEAKER_GLADIATOR = { layout = BNPC_GLADIATOR, name = 924, icon = 0 }
TALK_START = 107
TALK_SECOND_WAVE = 108
TALK_WAVES_DOWN = 103
TALK_LEADER_FIGHT = 104

ACTION_CURE = 120

BATTLE = Battle.new()
BATTLE:ally(BNPC_GLADIATOR, {
    role = "dps",
    split = false,
    focus = { "adds", "boss" },
    order = "nearest",
    heals = { { action = ACTION_CURE, targets = { "self" }, below = 35, every = 5 } },
})
for _, layout in ipairs(WAVE_1) do
    BATTLE:enemy(layout, { group = { "wave1", "adds" }, attacks = BNPC_GLADIATOR })
end
for _, layout in ipairs(WAVE_2) do
    BATTLE:enemy(layout, { group = { "wave2", "adds" }, attacks = BNPC_GLADIATOR })
end
BATTLE:enemy(BNPC_LEADER, { group = "boss", attacks = BNPC_GLADIATOR })

local over = false
local fail_in = nil
local phase = 1
local first_down = 0
-- The second wave is out (it comes a moment after sequence 3.)
local second_wave_out = false
-- The enemies out and not yet down.
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
    BATTLE:spawn(director, { BNPC_GLADIATOR })
    bringOut(director, WAVE_1)
    after(TALK_START_AFTER, function(d) talk(d, SPEAKER_GLADIATOR, TALK_START) end)
end

-- Both waves down: the gladiator stands back, then their leader comes and he takes him on.
function wavesDown(director)
    phase = SEQUENCE_WAVES_DOWN
    director:set_sequence(SEQUENCE_WAVES_DOWN)
    talk(director, SPEAKER_GLADIATOR, TALK_WAVES_DOWN)
    BATTLE:holdBack({ BNPC_GLADIATOR })
    after(LEADER_AFTER, function(d)
        phase = SEQUENCE_LEADER
        d:set_sequence(SEQUENCE_LEADER)
        bringOut(d, { BNPC_LEADER })
        after(LEADER_FIGHT_AFTER, function(d2)
            phase = SEQUENCE_LEADER_FIGHT
            d2:set_sequence(SEQUENCE_LEADER_FIGHT)
            BATTLE:release(d2, { BNPC_GLADIATOR })
            after(TALK_LEADER_FIGHT_AFTER, function(d3) talk(d3, SPEAKER_GLADIATOR, TALK_LEADER_FIGHT) end)
        end)
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
    BATTLE:update(director, health, player_health, info)
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    after(DEATH_NOTICED_AFTER, function(d) deathNoticed(d, layout_id) end)
end

-- The director hears of a death once its animation has played (retail: 1.8-2.1 s after the health reached 0, all
-- four times it acted on one, as Lord of the Inferno's 2.2 s.)
function deathNoticed(director, layout_id)
    if over then
        return
    end
    standing[layout_id] = nil
    if layout_id == BNPC_LEADER then
        over = true
        timers = {}
        phase = SEQUENCE_WON
        director:set_sequence(SEQUENCE_WON)
        director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS)
        return
    end
    if inList(WAVE_1, layout_id) then
        first_down = first_down + 1
        if phase == SEQUENCE_FIGHTING and first_down >= SECOND_WAVE_AFTER_DOWN then
            phase = SEQUENCE_SECOND_WAVE
            director:set_sequence(SEQUENCE_SECOND_WAVE)
            after(SECOND_WAVE_AFTER, function(d)
                second_wave_out = true
                bringOut(d, WAVE_2)
            end)
            after(TALK_SECOND_WAVE_AFTER, function(d) talk(d, SPEAKER_GLADIATOR, TALK_SECOND_WAVE) end)
        end
    end
    if phase == SEQUENCE_SECOND_WAVE and second_wave_out and next(standing) == nil then
        wavesDown(director)
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
