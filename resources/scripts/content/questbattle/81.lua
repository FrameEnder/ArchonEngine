-- Quest battle 81: Austerities of Wind (JobSmn451_01095, the summoners' level 45 quest.)
-- It takes place in territory 325, whose level is Outer La Noscea's (s1f6), so its NPCs are layer QB_JobSmn451_001 of
-- sea_s1f6: Y'mhitra the Learned on your side (4302425, base 2211, 3168 HP, a conjurer); against you Garuda-Egi
-- (4302448, 2209, 20056 HP), the Egi plumes she calls (one of 4302452, 4302454, 4302455 each time: 2210, 80 HP), then
-- Chirada-Egi (4321859, 2212) and Suparna-Egi (4321860, 2213). Layout IDs, levels, HP, positions, display and character
-- data flags, her looks and the start spot are retail's (Bob's Kawari Recorder runs of 2026-11-12, `questkit battle`:
-- one lost, one won; a level 45 arcanist.)
--
-- Retail's flow (seconds from the director's scene 60000):
--   0       sequence 2 (0.5 s after); Garuda-Egi on Y'mhitra, then you (7 s in); Y'mhitra's battle talk 68 at 3.1
--   Garuda-Egi under 80 % (22.9, 24.2): sequence 3; a plume 7.4 s later and every 6.8-7.5 s after, each standing where
--           it came out, waiting; talk 69 20.0 s after sequence 3
--   Garuda-Egi under 40 % (45.1, 64.0): sequence 7 (4-6 never came), Chirada-Egi and Suparna-Egi 0.6 s later, no more
--           plumes; every plume left dives (Featherlance, 665, round itself: 300-314 on the player, 402-424 on
--           Y'mhitra) 2.1-5.8 s later and falls 2.3 s after: five of them killed the player in the lost run; talk 70
--           10.1 s after sequence 7
--   Garuda-Egi and both of hers down: "Duty Complete" (60001) and sequence 9 1.5 s after the last fell, the battle's
--           scene 9 (0xF8402EFB) 3.6 s after that; the quest moves to sequence 2, and as scene 9 ends the quest's own
--           "LOGMESSAGE0" (LogMessage 3675, from quest 1095's event)
-- The retry began with "The Echo grants you strength beyond your limits!" (LogMessage 3212) from the director; the
-- Echo isn't played. What each one does is in their timelines (QB81*).

-- Where retail put the player and which way they faced.
START_POSITION = { x = -366.71, y = 51.5, z = -289.81 }
START_ROTATION = 2.21
-- Retail's opening scene 1 here.
OPENING_SCENE_FLAGS = 0xF841AEFB

BNPC_YMHITRA = 4302425
BNPC_GARUDA = 4302448
BNPC_CHIRADA = 4321859
BNPC_SUPARNA = 4321860
PLUMES = { 4302452, 4302454, 4302455 }

QUEST_SEQUENCE_AFTER_WIN = 2
-- Austerities of Wind's LOGMESSAGE0, the quest's line as the battle's closing scene ends.
QUEST_LOG = 3675
SEQUENCE_FIGHTING = 2
SEQUENCE_PLUMES = 3
SEQUENCE_SISTERS = 7
SEQUENCE_WON = 9
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 9
FINISH_SCENE_FLAGS = 0xF8402EFB
FAIL_DELAY_UPDATES = 12
PLUMES_HEALTH = 80
SISTERS_HEALTH = 40

ACTION_FEATHERLANCE = 665
ACTION_CURE = 120

SPEAKER_YMHITRA = { layout = BNPC_YMHITRA, name = 1640, icon = 0 }

-- Timers, in updates (half a second each). A timer of n fires n / 2 s after it's set in onUpdate or another timer,
-- (n - 1) / 2 s after it's set before the next update (onCommence, onActorDeath.)
TALK_68_AFTER = 7
TALK_69_AFTER = 40
TALK_70_AFTER = 20
FIRST_PLUME_AFTER = 15
PLUME_EVERY = 14
SISTERS_AFTER = 1
-- The plumes dive one after another from 2.1 s after sequence 7, and fall 2.3 s after they dive.
DIVE_AFTER = 4
DIVE_EVERY = 2
FALLS_AFTER_DIVE = 5
WON_AFTER_DEATH = 4

BATTLE = Battle.new()
BATTLE:ally(BNPC_YMHITRA, {
    role = "dps",
    focus = { "boss", "adds" },
    range = 15,
    heals = { { action = ACTION_CURE, targets = { "player", BNPC_YMHITRA }, below = 40, every = 12 } },
})
BATTLE:enemy(BNPC_GARUDA, { group = "boss", attacks = BNPC_YMHITRA })
BATTLE:enemy(BNPC_CHIRADA, { group = "adds", attacks = BNPC_YMHITRA })
BATTLE:enemy(BNPC_SUPARNA, { group = "adds" })

local over = false
local winning = false
local fail_in = nil
local phase = 1
-- The plumes waiting (actor IDs), and how many of Garuda-Egi and hers are still up.
local plumes = {}
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
    director:battle_talk(SPEAKER_YMHITRA.layout, SPEAKER_YMHITRA.name, text, SPEAKER_YMHITRA.icon)
end

function sequence(director, value)
    phase = value
    director:set_sequence(value)
end

-- A plume, one of the three at random, waits where it comes out; another comes every 7 s until sequence 7.
function plume(director)
    if over or phase ~= SEQUENCE_PLUMES then
        return
    end
    local id = director:spawn_bnpc(PLUMES[math.random(#PLUMES)])
    director:hold_bnpc(id)
    table.insert(plumes, id)
    after(PLUME_EVERY, plume)
end

function onSetup(director)
    director:set_sequence(1)
end

function onCommence(director)
    sequence(director, SEQUENCE_FIGHTING)
    standing[BNPC_GARUDA] = true
    BATTLE:spawn(director, { BNPC_YMHITRA, BNPC_GARUDA })
    after(TALK_68_AFTER, function(d) talk(d, 68) end)
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
    local garuda = health[BNPC_GARUDA]
    if garuda == nil then
        return
    end
    if phase == SEQUENCE_FIGHTING and garuda < PLUMES_HEALTH then
        sequence(director, SEQUENCE_PLUMES)
        after(FIRST_PLUME_AFTER, plume)
        after(TALK_69_AFTER, function(d) talk(d, 69) end)
    end
    if phase == SEQUENCE_PLUMES and garuda < SISTERS_HEALTH then
        sequence(director, SEQUENCE_SISTERS)
        after(SISTERS_AFTER, function(d)
            standing[BNPC_CHIRADA] = true
            standing[BNPC_SUPARNA] = true
            BATTLE:spawn(d, { BNPC_CHIRADA, BNPC_SUPARNA })
        end)
        -- Every plume left dives, one after another, and falls.
        for i, id in ipairs(plumes) do
            after(DIVE_AFTER + DIVE_EVERY * (i - 1), function(d)
                d:hold_bnpc(id, false)
                d:actor_use_action(id, ACTION_FEATHERLANCE)
                after(FALLS_AFTER_DIVE, function(dd) dd:kill_bnpc(id) end)
            end)
        end
        plumes = {}
        after(TALK_70_AFTER, function(d) talk(d, 70) end)
    end
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
    if phase >= SEQUENCE_SISTERS and next(standing) == nil and not winning then
        winning = true
        timers = {}
        after(WON_AFTER_DEATH, function(d)
            over = true
            timers = {}
            sequence(d, SEQUENCE_WON)
            d:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS, nil, true,
                QUEST_LOG)
        end)
    end
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
