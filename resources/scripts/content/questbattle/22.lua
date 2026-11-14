-- Quest battle 22: On Holy Ground (ClsGla250_00264, the Gladiators' level 25 quest): a duel.
-- It takes place in territory 256, a copy of Eastern Thanalan (retail's positions in it stand on Eastern Thanalan's
-- ground, within 0.02 yalms, against the same capture's visits to 145), so its one NPC lives in the wil_w1f3 drop-in
-- (layer QB_ClsGla250_002). Level, HP, position, gear, display and character data flags and the start spot are
-- retail's (Bob's Chronofoil capture accc33a9 of 2026-11-06, `questkit chronofoil --battle 0.6`: one run, won; a
-- gladiator synced to 29): the pugilist (4004491, base 640, BNpcName 698, level 25, 2079 HP), on you alone. The one
-- watching (ENpcResident 1004411) isn't in the fight, so its lines are sent as an EventNpc's (kind 3), as Victory in
-- Peril's watcher's.
--
-- Timed against the run (seconds from the fight starting, the director's scene 60000):
--   0       sequence 2
--   5.1     battle talk 124 (the watcher)
--   the pugilist at 80 % (12.5): sequence 3; 127 (him) 2.5 s and 126 (the watcher) 10.1 s after it
--   the pugilist at 40 % (24.6): sequence 4; 128 (him) 3.1 s, 125 (him) and 129 (the watcher) 8.0 s after it
--   (his Second Wind took him from 33 % back to 47 % at 29.6 s: the thresholds go once each)
--   him down (48.7, heard 2.1 s later, his death animation): "Duty Complete" (60001) and sequence 5, then the battle's
--           scene 5 (a cutscene, 0xF8482EFB, with its menu) 3.5 s later; the quest moves to 255

-- Where retail puts the player and which way they face: their SpawnPlayer in the capture.
START_POSITION = { x = -518.39, y = -21.6, z = 120.48 }
START_ROTATION = 2.334
-- Retail's opening scene 1 here.
OPENING_SCENE_FLAGS = 0xF8412EFB

BNPC_PUGILIST = 4004491

QUEST_SEQUENCE_AFTER_WIN = 255
SEQUENCE_FIGHTING = 2
SEQUENCE_EIGHTY = 3
SEQUENCE_FORTY = 4
SEQUENCE_WON = 5
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 5
FINISH_SCENE_FLAGS = 0xF8482EFB
FAIL_DELAY_UPDATES = 12

-- His health for sequences 3 and 4.
FIRST_THRESHOLD = 80
SECOND_THRESHOLD = 40

-- Timers, in updates (half a second each.)
DEATH_NOTICED_AFTER = 4
TALK_START_AFTER = 10

-- The battle talk (the content's text IDs, retail's.)
-- The one watching stands there (layout 4065824, retail's spawn at the start of Bob's run; no one stood for it
-- until 2026-11-14.)
ENPC_WATCHER = 4065824
SPEAKER_WATCHER = { layout = ENPC_WATCHER, name = 1004411, icon = 0, kind = 3 }
SPEAKER_PUGILIST = { layout = BNPC_PUGILIST, name = 698, icon = 0 }
TALK_START = 124

BATTLE = Battle.new()
BATTLE:enemy(BNPC_PUGILIST, { group = "boss" })

local over = false
local fail_in = nil
local phase = 1
local timers = {}

-- Timers of their own (not Battle:steps, which queues each after the last: a quick kill would wait for the talk.)
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

function onSetup(director)
    director:set_sequence(1)
    director:spawn_bnpc(ENPC_WATCHER)
end

function onCommence(director)
    phase = SEQUENCE_FIGHTING
    director:set_sequence(SEQUENCE_FIGHTING)
    BATTLE:spawn(director, { BNPC_PUGILIST })
    after(TALK_START_AFTER, function(d) talk(d, SPEAKER_WATCHER, TALK_START) end)
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
    local pugilist = health[BNPC_PUGILIST]
    if phase == SEQUENCE_FIGHTING and pugilist ~= nil and pugilist <= FIRST_THRESHOLD then
        phase = SEQUENCE_EIGHTY
        director:set_sequence(SEQUENCE_EIGHTY)
        after(5, function(d) talk(d, SPEAKER_PUGILIST, 127) end)
        after(20, function(d) talk(d, SPEAKER_WATCHER, 126) end)
    end
    if phase == SEQUENCE_EIGHTY and pugilist ~= nil and pugilist <= SECOND_THRESHOLD then
        phase = SEQUENCE_FORTY
        director:set_sequence(SEQUENCE_FORTY)
        after(6, function(d) talk(d, SPEAKER_PUGILIST, 128) end)
        after(16, function(d)
            talk(d, SPEAKER_PUGILIST, 125)
            talk(d, SPEAKER_WATCHER, 129)
        end)
    end
    BATTLE:update(director, health, player_health, info)
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    if layout_id == BNPC_PUGILIST then
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
