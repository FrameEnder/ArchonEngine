-- Quest battle 53: Sinking Doesmaga (ClsAcn300_00461, the Arcanists' level 30 quest.)
-- It takes place in territory 288, aboard the Morningstar again (level ocn_o1/evt/o1e2), so its NPCs are layer
-- QB_ClsAcn300_001 of the ocn_o1e2 drop-in: on your side Foreseer K'lyhia (4256368, base 2075, 5460 HP), her Topaz
-- Carbuncle later (4256369, 2076) and the glowing grimoire (4263912, 2077), a magicked circle that never acts; against
-- you the crew in two waves (boatswains 2071, gunners 2073, a falmadair 2072, Barber Hirskskrat 2074, a topman 2078, a
-- cosier 2079), then Doesmaga Poisonheart (4256716, 2080, 2661 HP) with three more. Layout IDs, levels, HP, positions,
-- display and character data flags, looks and the start spot are retail's (Bob's Kawari Recorder run of 2026-11-12,
-- `questkit battle`: one run, won, a level 34 arcanist.)
--
-- Retail's flow (seconds from the director's scene 60000):
--   0       sequence 2 (0.2 s after) and the grimoire; the player gains Vulnerability Down (status 63, 30 s, from no
--           one) at 0.2 and the director's "The magicked circle grants you..." (LogMessage 3225) comes at 5.1; the
--           first six on K'lyhia; her battle talk 121 at 10.1
--   K'lyhia's Summon II (1307, a 5.7 s cast 5.1 s in): her Topaz Carbuncle 7.3 s after it began
--   four of the first six down (the gunners and boatswains, 20.7): sequence 3 at once, the second wave 0.5 s later
--           (a cosier, two boatswains, two gunners, a topman); Doesmaga's talk 122 (the event NPC) 15.1 s after
--   everything down (61.7): sequence 4 and the battle's scene 4 (0xF8412FFF, Doesmaga's arrival) 1.9 s later; as it
--           ends, sequence 5 and the third wave 0.5 s after (Doesmaga, two topmen, a boatswain); talks 123
--           (K'lyhia) and 124 (Doesmaga) 7.1 and 12.1 s after sequence 5
--   Doesmaga under 50 %: sequence 6
--   everything down: "Duty Complete" (60001) and sequence 7 0.3 s after the last fell, the battle's scene 7
--           (0xF8402EFB) 3.1 s after that; the quest moves to sequence 8
-- What each one does is in their timelines (QB53*). Doesmaga and his lackeys are event NPCs at the bow until scene 4;
-- Kawari leaves those as the zone has them.

-- Where retail put the player and which way they faced.
START_POSITION = { x = 7.69, y = 16.35, z = -1.7 }
START_ROTATION = -0.03
-- Retail's opening scene 1 here.
OPENING_SCENE_FLAGS = 0xF841AEFB

BNPC_KLYHIA = 4256368
BNPC_CARBUNCLE = 4256369
BNPC_GRIMOIRE = 4263912
BNPC_DOESMAGA = 4256716
BNPC_BARBER = 4256387
ENPC_DOESMAGA = 4262403
WAVE_1 = { 4256381, 4256382, 4256383, 4256387, 4256530, 4256531 }
WAVE_2 = { 4256540, 4256706, 4256707, 4256708, 4256711, 4256712 }
WAVE_3 = { BNPC_DOESMAGA, 4256717, 4256718, 4256719 }
WAVE_1_DOWN_FOR_WAVE_2 = 4

QUEST_SEQUENCE_AFTER_WIN = 8
SEQUENCE_FIGHTING = 2
SEQUENCE_SECOND_WAVE = 3
SEQUENCE_ARRIVAL = 4
SEQUENCE_DOESMAGA = 5
SEQUENCE_DOESMAGA_HALF = 6
SEQUENCE_WON = 7
SEQUENCE_FAILED = 0xFE
ARRIVAL_SCENE = 4
ARRIVAL_SCENE_FLAGS = 0xF8412FFF
FINISH_SCENE = 7
FINISH_SCENE_FLAGS = 0xF8402EFB
FAIL_DELAY_UPDATES = 12
DOESMAGA_HALF = 50

ACTION_SUMMON_II = 1307
ACTION_CURE = 120
-- The magicked circle's gift: Vulnerability Down for 30 s, and the director's line about it.
STATUS_VULNERABILITY_DOWN = 63
VULNERABILITY_DOWN_SECONDS = 30.0
LOG_CIRCLE = 3225

SPEAKER_KLYHIA = { layout = BNPC_KLYHIA, name = 1493, icon = 0 }
SPEAKER_DOESMAGA_NPC = { layout = ENPC_DOESMAGA, name = 1005186, icon = 0, kind = 3 }
SPEAKER_DOESMAGA = { layout = BNPC_DOESMAGA, name = 1500, icon = 0 }

-- Timers, in updates (half a second each). A timer of n fires (n - 1) / 2 s after it's set before the next update
-- (onCommence, onActorDeath, onSceneFinished), n / 2 s after it's set in onUpdate.
TALKS_FIGHTING = { { 21, SPEAKER_KLYHIA, 121 } }
TALKS_SECOND_WAVE = { { 31, SPEAKER_DOESMAGA_NPC, 122 } }
TALKS_DOESMAGA = { { 15, SPEAKER_KLYHIA, 123 }, { 25, SPEAKER_DOESMAGA, 124 } }
LOG_CIRCLE_AFTER = 11
WAVE_2_AFTER = 2
ARRIVAL_AFTER = 5
WAVE_3_AFTER = 2
CARBUNCLE_AFTER_CAST = 15
WON_AFTER_DEATH = 2

BATTLE = Battle.new()
BATTLE:ally(BNPC_KLYHIA, { role = "tank", focus = { "crew" }, range = 10 })
BATTLE:ally(BNPC_CARBUNCLE, { role = "dps", focus = { "crew" }, order = "oldest", range = 3 })
for _, wave in ipairs({ WAVE_1, WAVE_2, WAVE_3 }) do
    for _, layout in ipairs(wave) do
        local role = { group = "crew", attacks = BNPC_KLYHIA }
        if layout == BNPC_BARBER then
            role.heals = { { action = ACTION_CURE, targets = { "allies" }, below = 20, every = 6 } }
        end
        BATTLE:enemy(layout, role)
    end
end

local over = false
local winning = false
local fail_in = nil
local phase = 1
local wave_1_down = 0
local carbuncle_out = false
local circle_given = false
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
        local speaker, text = line[2], line[3]
        after(line[1], function(d)
            d:battle_talk(speaker.layout, speaker.name, text, speaker.icon, nil, speaker.kind)
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

function onSetup(director)
    director:set_sequence(1)
end

function onCommence(director)
    sequence(director, SEQUENCE_FIGHTING)
    BATTLE:spawn(director, { BNPC_KLYHIA })
    -- The grimoire stands where it is and never acts.
    director:spawn_bnpc(BNPC_GRIMOIRE)
    director:hold_bnpc(BNPC_GRIMOIRE)
    bringOut(director, WAVE_1)
    talks(TALKS_FIGHTING)
    after(LOG_CIRCLE_AFTER, function(d) d:log_message(LOG_CIRCLE, {}) end)
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
    -- The circle's Vulnerability Down, from no one, as the fight begins.
    if not circle_given and info ~= nil and info.player ~= nil and info.player.id ~= nil then
        circle_given = true
        director:gain_effect(info.player.id, STATUS_VULNERABILITY_DOWN, 0, VULNERABILITY_DOWN_SECONDS)
    end
    runTimers(director)
    BATTLE:update(director, health, player_health, info)
    if phase == SEQUENCE_DOESMAGA and health[BNPC_DOESMAGA] ~= nil and health[BNPC_DOESMAGA] < DOESMAGA_HALF then
        sequence(director, SEQUENCE_DOESMAGA_HALF)
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
    for _, layout in ipairs(WAVE_1) do
        if layout == layout_id then
            wave_1_down = wave_1_down + 1
            if wave_1_down == WAVE_1_DOWN_FOR_WAVE_2 and phase == SEQUENCE_FIGHTING then
                sequence(director, SEQUENCE_SECOND_WAVE)
                after(WAVE_2_AFTER, function(d) bringOut(d, WAVE_2) end)
                talks(TALKS_SECOND_WAVE)
            end
        end
    end
    if next(standing) ~= nil then
        return
    end
    if phase == SEQUENCE_SECOND_WAVE then
        -- The crew down: Doesmaga arrives (the battle's scene 4.)
        timers = {}
        after(ARRIVAL_AFTER, function(d)
            sequence(d, SEQUENCE_ARRIVAL)
            d:play_scene(ARRIVAL_SCENE, ARRIVAL_SCENE_FLAGS)
        end)
    elseif phase >= SEQUENCE_DOESMAGA and not winning then
        winning = true
        timers = {}
        after(WON_AFTER_DEATH, won)
    end
end

function onSceneFinished(director, scene)
    if scene ~= ARRIVAL_SCENE or phase ~= SEQUENCE_ARRIVAL or over then
        return
    end
    sequence(director, SEQUENCE_DOESMAGA)
    after(WAVE_3_AFTER, function(d) bringOut(d, WAVE_3) end)
    talks(TALKS_DOESMAGA)
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
