-- Quest battle 79: Austerities of Flame (JobSmn300_01091, the summoners' level 30 quest.)
-- It takes place in territory 323, whose level is Southern Thanalan's (w1f4), so its NPCs are layer QB_JobSmn300_001
-- of wil_w1f4: Y'mhitra the Learned on your side (4300437, base 1514, 2340 HP, a conjurer), Ifrit-Egi against you
-- (4300450, 1513, 4435 HP), later a flame shiver (4300454, 1515) and three crimson nails (4300459, 4300461-2, 1516).
-- Layout IDs, levels, HP, positions, display and character data flags, her looks and the start spot are retail's
-- (Bob's Kawari Recorder run of 2026-11-12, `questkit battle`: one run, won, a level 34 arcanist.)
--
-- Retail's flow (seconds from the director's scene 60000):
--   0       sequence 2; Ifrit-Egi on Y'mhitra (then you, 6 s in); her battle talk 84 at 3.1
--   Ifrit-Egi under 80 % (10.7): sequence 3, the flame shiver 0.4 s later (on Y'mhitra), talk 85 10.1 s after
--   Ifrit-Egi under 50 % (22.5): sequence 4, the crimson nails 0.4 s later, Ifrit-Egi's Radiant Shield (1323) as they
--           come, "Ifrit-Egi invokes crimson nails." (LogMessage 3222) from the director 3.1 s after, talk 86
--           8.1 s after; sequence 5 10.1 s after sequence 4 if the nails still stand
--   the nails down (39.2): sequence 7 at once, talk 87 2.0 s after (sequence 6 never came)
--   Ifrit-Egi down: "Duty Complete" (60001) and sequence 8 at once, the battle's scene 8 (0xF8402EFB) 3.1 s after;
--           the quest moves to sequence 3
-- The nails never moved nor acted: they stand there to be broken. What the others do is in their timelines (QB79*).

-- Where retail put the player and which way they faced.
START_POSITION = { x = -346.98, y = 1.08, z = 745.41 }
START_ROTATION = -1.8
-- Retail's opening scene 1 here.
OPENING_SCENE_FLAGS = 0xF841AEFB

BNPC_YMHITRA = 4300437
BNPC_EGI = 4300450
BNPC_SHIVER = 4300454
NAILS = { 4300459, 4300461, 4300462 }

QUEST_SEQUENCE_AFTER_WIN = 3
SEQUENCE_FIGHTING = 2
SEQUENCE_SHIVER = 3
SEQUENCE_NAILS = 4
SEQUENCE_NAILS_STAND = 5
SEQUENCE_NAILS_DOWN = 7
SEQUENCE_WON = 8
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 8
FINISH_SCENE_FLAGS = 0xF8402EFB
FAIL_DELAY_UPDATES = 12
SHIVER_HEALTH = 80
NAILS_HEALTH = 50

ACTION_RADIANT_SHIELD = 1323
LOG_NAILS = 3222

SPEAKER_YMHITRA = { layout = BNPC_YMHITRA, name = 1640, icon = 0 }

-- Timers, in updates (half a second each). A timer of n fires n / 2 s after it's set in onUpdate, (n - 1) / 2 s after
-- it's set before the next update (onCommence, onActorDeath.)
TALK_84_AFTER = 7
TALK_85_AFTER = 20
TALK_86_AFTER = 16
TALK_87_AFTER = 5
SHIVER_AFTER = 1
NAILS_AFTER = 1
LOG_NAILS_AFTER = 6
SEQUENCE_5_AFTER = 20

BATTLE = Battle.new()
BATTLE:ally(BNPC_YMHITRA, { role = "dps", focus = { "adds", "boss" }, range = 15 })
BATTLE:enemy(BNPC_EGI, { group = "boss", attacks = BNPC_YMHITRA })
BATTLE:enemy(BNPC_SHIVER, { group = "adds", attacks = BNPC_YMHITRA })
for _, layout in ipairs(NAILS) do
    BATTLE:enemy(layout, { group = "adds" })
end

local over = false
local fail_in = nil
local phase = 1
local nails_standing = 0
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
    director:battle_talk(SPEAKER_YMHITRA.layout, SPEAKER_YMHITRA.name, text, SPEAKER_YMHITRA.icon)
end

function sequence(director, value)
    phase = value
    director:set_sequence(value)
end

function onSetup(director)
    director:set_sequence(1)
end

function onCommence(director)
    sequence(director, SEQUENCE_FIGHTING)
    BATTLE:spawn(director, { BNPC_YMHITRA, BNPC_EGI })
    after(TALK_84_AFTER, function(d) talk(d, 84) end)
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
    local egi = health[BNPC_EGI]
    if egi == nil then
        return
    end
    if phase == SEQUENCE_FIGHTING and egi < SHIVER_HEALTH then
        sequence(director, SEQUENCE_SHIVER)
        after(SHIVER_AFTER, function(d) BATTLE:spawn(d, { BNPC_SHIVER }) end)
        after(TALK_85_AFTER, function(d) talk(d, 85) end)
    end
    if phase == SEQUENCE_SHIVER and egi < NAILS_HEALTH then
        sequence(director, SEQUENCE_NAILS)
        after(NAILS_AFTER, function(d)
            nails_standing = #NAILS
            BATTLE:spawn(d, NAILS)
            -- They stand where they come out and never act.
            for _, layout in ipairs(NAILS) do
                d:hold_bnpc(layout)
            end
            d:use_action(BNPC_EGI, ACTION_RADIANT_SHIELD)
        end)
        after(LOG_NAILS_AFTER, function(d) d:log_message(LOG_NAILS, {}) end)
        after(TALK_86_AFTER, function(d) talk(d, 86) end)
        after(SEQUENCE_5_AFTER, function(d)
            if phase == SEQUENCE_NAILS then
                sequence(d, SEQUENCE_NAILS_STAND)
            end
        end)
    end
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    if layout_id == BNPC_EGI then
        over = true
        timers = {}
        sequence(director, SEQUENCE_WON)
        director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS)
        return
    end
    for _, layout in ipairs(NAILS) do
        if layout == layout_id then
            nails_standing = nails_standing - 1
            if nails_standing == 0 and (phase == SEQUENCE_NAILS or phase == SEQUENCE_NAILS_STAND) then
                sequence(director, SEQUENCE_NAILS_DOWN)
                after(TALK_87_AFTER, function(d) talk(d, 87) end)
            end
        end
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
