-- Quest battle 65: Escape from Castrum Centri (ManFst407_00521, the main scenario's level 47 quest.)
-- It takes place in territory 305, whose level is Mor Dhona's (l1f1), so its NPCs are layer QB_ManFst407_001 of
-- lak_l1f1. Two parts with a cutscene between. First, by the bound Scions: Wedge (4278227, base 2173) and Biggs
-- (4278226, 2172) on your side against a 5th Cohort signifer (4278235, 2171), an eques (4278234, 2170) and an optio
-- (4278232, 2169, 8793 HP). Then, on the way out: Urianger (4278521, 2045) and Papalymo (4278513, 2044) on your side,
-- Urianger's Amber Carbuncle (4308235, 2174) later, against three waves: the first with a Magitek Vanguard H-1
-- (4278569, 2177) shielded by a Magitek Field Generator (4286888, 2179) a signifer guards; the second with two vanguards
-- (4285260-1) shielded by two generators (4286889, 4286891) and their signifers; the last a Magitek Colossus (4287242,
-- 2178) and an optio (4287243, 2180.) Layout IDs, levels, HP, positions, display and character data flags, looks and
-- the start spot are retail's (Bob's Kawari Recorder runs of 2026-11-12, `questkit battle`: one lost in the first part,
-- one won; a level 47 arcanist.)
--
-- Retail's flow (seconds from the director's scene 60000):
--   0       sequence 2 (0.4 s after); the eques on Wedge, the optio on Biggs; the bound Scions' battle talk (event
--           NPCs, with portraits): Minfilia's 261, Urianger's 262, Papalymo's 263 at 5.0, 15.0 and 20.0
--   the optio down (41.5): sequence 8 and the battle's scene 8 (0xF8402EFB) at once; the first part's NPCs go
--   scene 8 over (3.8 s): sequence 9 (the client sets the new objective), the second part 0.4 s later; the
--           hoplomachus's talk 265 5.1 s after, Urianger's Amber Carbuncle 12.6 s after, Urianger's 266 and
--           Papalymo's 267 25.1 and 30.1 s after
--   the first generator down (107.7): sequence 10 at once and "The magitek field surrounding the vanguard is
--           deactivated." (LogMessage 3216) from the director, as with every generator; the vanguard takes damage only
--           once its generators are down
--   15.0 s after sequence 10: sequence 11 and the second wave 0.4 s later; Papalymo's 268 and Urianger's 269 10.1 and
--           15.1 s after
--   the second wave down (177.6): sequence 12 at once, the colossus and the optio 0.5 s later; the optio's 270, Biggs's
--           271 (an event NPC), Minfilia's 272, Urianger's 273 and Papalymo's 274 10.1, 15.1, 20.1, 20.3 and 31.2 s
--           after, and sequence 13 15.2 s after
--   everyone down: "Duty Complete" (60001) and sequence 14 1.1 s after the last fell, the battle's scene 14 (0x2001)
--           3.1 s after that; the quest moves to sequence 6 and the player goes on to Ul'dah (the quest's scene 40
--           there.)
-- The generators and the signifers by them wait until someone comes at them. The retry began with "The Echo grants
-- you strength beyond your limits!" (LogMessage 3212); the Echo isn't played. What each one does is in their
-- timelines (QB65*).

-- Where retail put the player and which way they faced.
START_POSITION = { x = -408.14, y = -1.4, z = -276.02 }
START_ROTATION = 1.1
-- Retail's opening scene 1 here.
OPENING_SCENE_FLAGS = 0xF841AEFB
-- A win sends the player on to Ul'dah - Steps of Nald, where the quest goes on (where Bob's recording found him there
-- 7.6 s later; which way he faced isn't recorded.)
EXIT_AFTER_WIN = { territory = 130, x = -40.46, y = 84.0, z = 1.33, rotation = 0.0 }

BNPC_WEDGE = 4278227
BNPC_BIGGS = 4278226
BNPC_FIRST_OPTIO = 4278232
FIRST_PART = { 4278235, 4278234, BNPC_FIRST_OPTIO }
BNPC_URIANGER = 4278521
BNPC_PAPALYMO = 4278513
BNPC_CARBUNCLE = 4308235
WAVE_1 = { 4278558, 4286888, 4278577, 4278538, 4278569, 4278529 }
WAVE_2 = { 4285261, 4285274, 4285317, 4286891, 4285260, 4286889, 4285278 }
WAVE_3 = { 4287242, 4287243 }
-- Each vanguard and the generators that shield it.
SHIELDS = {
    [4278569] = { 4286888 },
    [4285260] = { 4286889, 4286891 },
    [4285261] = { 4286889, 4286891 },
}
GENERATORS = { [4286888] = true, [4286889] = true, [4286891] = true }
-- Who each one goes for first (nil: you), and who waits until someone comes at them.
ATTACKS = {
    [4278234] = BNPC_WEDGE, [4278232] = BNPC_BIGGS,
    [4278569] = BNPC_PAPALYMO, [4278558] = BNPC_PAPALYMO, [4278538] = BNPC_URIANGER, [4278529] = BNPC_URIANGER,
    [4285274] = BNPC_URIANGER, [4285261] = BNPC_URIANGER, [4285260] = BNPC_PAPALYMO,
    [4287242] = BNPC_PAPALYMO,
}
WAITING = { [4286888] = true, [4278577] = true, [4286889] = true, [4286891] = true, [4285317] = true,
    [4285278] = true }
-- The enemy conjurers' Cures.
HEALERS = { [4278558] = { 120, 135 }, [4287243] = { 135 } }

QUEST_SEQUENCE_AFTER_WIN = 6
SEQUENCE_FIRST_PART = 2
SEQUENCE_CUTSCENE = 8
SEQUENCE_SECOND_PART = 9
SEQUENCE_GENERATOR = 10
SEQUENCE_SECOND_WAVE = 11
SEQUENCE_LAST_WAVE = 12
SEQUENCE_LAST_WAVE_ON = 13
SEQUENCE_WON = 14
SEQUENCE_FAILED = 0xFE
CUTSCENE = 8
CUTSCENE_FLAGS = 0xF8402EFB
FINISH_SCENE = 14
FINISH_SCENE_FLAGS = 0x2001
FAIL_DELAY_UPDATES = 12

ACTION_SUMMON_II = 1307
ACTION_CURE = 120
LOG_FIELD_DOWN = 3216

-- Who's talking (DirectorPopupMessage), with retail's names and portraits; the Scions bound and Biggs at the end as
-- event NPCs (kind 3.)
SPEAKER_MINFILIA_BOUND = { layout = 4278275, name = 1007077, icon = 73177, kind = 3 }
SPEAKER_URIANGER_BOUND = { layout = 4307929, name = 1007084, icon = 73178, kind = 3 }
SPEAKER_PAPALYMO_BOUND = { layout = 4307930, name = 1007083, icon = 73164, kind = 3 }
SPEAKER_HOPLOMACHUS = { layout = 4278538, name = 1809, icon = 73179 }
SPEAKER_URIANGER = { layout = BNPC_URIANGER, name = 1492, icon = 73178 }
SPEAKER_PAPALYMO = { layout = BNPC_PAPALYMO, name = 620, icon = 73164 }
SPEAKER_OPTIO = { layout = 4287243, name = 2093, icon = 73180 }
SPEAKER_BIGGS = { layout = 4288515, name = 1007079, icon = 73182, kind = 3 }
SPEAKER_MINFILIA = { layout = 4288517, name = 1007080, icon = 73177, kind = 3 }

-- Timers, in updates (half a second each). A timer of n fires n / 2 s after it's set in onUpdate or another timer,
-- (n - 1) / 2 s after it's set before the next update (onCommence, onActorDeath, onSceneFinished.)
TALKS_FIRST_PART = { { 11, SPEAKER_MINFILIA_BOUND, 261 }, { 31, SPEAKER_URIANGER_BOUND, 262 },
    { 41, SPEAKER_PAPALYMO_BOUND, 263 } }
TALKS_SECOND_PART = { { 11, SPEAKER_HOPLOMACHUS, 265 }, { 51, SPEAKER_URIANGER, 266 }, { 61, SPEAKER_PAPALYMO, 267 } }
TALKS_SECOND_WAVE = { { 20, SPEAKER_PAPALYMO, 268 }, { 30, SPEAKER_URIANGER, 269 } }
TALKS_LAST_WAVE = { { 21, SPEAKER_OPTIO, 270 }, { 31, SPEAKER_BIGGS, 271 }, { 41, SPEAKER_MINFILIA, 272 },
    { 41, SPEAKER_URIANGER, 273 }, { 63, SPEAKER_PAPALYMO, 274 } }
SECOND_PART_AFTER = 1
CARBUNCLE_AFTER_CAST = 15
SECOND_WAVE_AFTER = 31
LAST_WAVE_AFTER = 2
SEQUENCE_13_AFTER = 31
WON_AFTER_DEATH = 3

BATTLE = Battle.new()
BATTLE:ally(BNPC_WEDGE, { role = "dps", focus = { "first" }, order = "oldest", range = 10 })
BATTLE:ally(BNPC_BIGGS, { role = "tank", focus = { "first" }, range = 3 })
BATTLE:ally(BNPC_URIANGER, {
    role = "dps",
    focus = { "second" },
    range = 15,
    heals = { { action = ACTION_CURE, targets = { "player", BNPC_PAPALYMO, BNPC_URIANGER }, below = 50, every = 12 } },
})
BATTLE:ally(BNPC_PAPALYMO, {
    role = "dps",
    focus = { "second" },
    range = 15,
    heals = { { action = ACTION_CURE, targets = { "player", BNPC_PAPALYMO, BNPC_URIANGER, BNPC_CARBUNCLE },
        below = 60, every = 8 } },
})
BATTLE:ally(BNPC_CARBUNCLE, { role = "dps", focus = { "second" }, order = "oldest", range = 3 })
for _, layout in ipairs(FIRST_PART) do
    BATTLE:enemy(layout, { group = "first", attacks = ATTACKS[layout] })
end
for _, wave in ipairs({ WAVE_1, WAVE_2, WAVE_3 }) do
    for _, layout in ipairs(wave) do
        local role = { group = "second", attacks = ATTACKS[layout], passive = WAITING[layout] or nil }
        if HEALERS[layout] ~= nil then
            role.heals = {}
            for _, action in ipairs(HEALERS[layout]) do
                table.insert(role.heals, { action = action, targets = { "allies" }, below = 50, every = 8 })
            end
        end
        BATTLE:enemy(layout, role)
    end
end

local over = false
local winning = false
local fail_in = nil
local phase = 1
local carbuncle_out = false
local standing = {}
local generators_up = {}
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
        if GENERATORS[layout] then
            generators_up[layout] = true
        end
    end
    BATTLE:spawn(director, layouts)
    -- A vanguard takes no damage while one of its generators stands.
    for _, layout in ipairs(layouts) do
        if SHIELDS[layout] ~= nil then
            director:set_invulnerable(layout, true)
        end
    end
end

function sequence(director, value)
    phase = value
    director:set_sequence(value)
end

function onSetup(director)
    director:set_sequence(1)
end

function onCommence(director)
    sequence(director, SEQUENCE_FIRST_PART)
    BATTLE:spawn(director, { BNPC_WEDGE, BNPC_BIGGS })
    bringOut(director, FIRST_PART)
    talks(TALKS_FIRST_PART)
end

-- Urianger's Summon II brings his Amber Carbuncle.
function onActorCast(director, layout_id, action_id)
    if over or layout_id ~= BNPC_URIANGER or action_id ~= ACTION_SUMMON_II or carbuncle_out then
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

function inWave(wave, layout_id)
    for _, layout in ipairs(wave) do
        if layout == layout_id then
            return true
        end
    end
    return false
end

function waveDown(wave)
    for _, layout in ipairs(wave) do
        if standing[layout] then
            return false
        end
    end
    return true
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

    -- The first part ends as its optio falls: scene 8, and everyone of it goes.
    if layout_id == BNPC_FIRST_OPTIO and phase == SEQUENCE_FIRST_PART then
        timers = {}
        sequence(director, SEQUENCE_CUTSCENE)
        for _, layout in ipairs({ BNPC_WEDGE, BNPC_BIGGS, 4278235, 4278234 }) do
            if standing[layout] or layout == BNPC_WEDGE or layout == BNPC_BIGGS then
                director:despawn_bnpc(layout)
                BATTLE:died(director, layout)
            end
            standing[layout] = nil
        end
        director:play_scene(CUTSCENE, CUTSCENE_FLAGS)
        return
    end

    if GENERATORS[layout_id] then
        generators_up[layout_id] = nil
        director:log_message(LOG_FIELD_DOWN, {})
        for vanguard, shields in pairs(SHIELDS) do
            local shielded = false
            for _, generator in ipairs(shields) do
                shielded = shielded or generators_up[generator] ~= nil
            end
            if not shielded and standing[vanguard] then
                director:set_invulnerable(vanguard, false)
            end
        end
        if phase == SEQUENCE_SECOND_PART then
            sequence(director, SEQUENCE_GENERATOR)
            after(SECOND_WAVE_AFTER, function(d)
                sequence(d, SEQUENCE_SECOND_WAVE)
                bringOut(d, WAVE_2)
                talks(TALKS_SECOND_WAVE)
            end)
        end
    end

    if phase == SEQUENCE_SECOND_WAVE and inWave(WAVE_2, layout_id) and waveDown(WAVE_2) then
        sequence(director, SEQUENCE_LAST_WAVE)
        after(LAST_WAVE_AFTER, function(d) bringOut(d, WAVE_3) end)
        talks(TALKS_LAST_WAVE)
        after(SEQUENCE_13_AFTER, function(d)
            if phase == SEQUENCE_LAST_WAVE then
                sequence(d, SEQUENCE_LAST_WAVE_ON)
            end
        end)
    end

    if phase >= SEQUENCE_LAST_WAVE and next(standing) == nil and not winning then
        winning = true
        timers = {}
        after(WON_AFTER_DEATH, function(d)
            over = true
            timers = {}
            sequence(d, SEQUENCE_WON)
            d:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS)
        end)
    end
end

-- Scene 8 over: the second part.
function onSceneFinished(director, scene)
    if scene ~= CUTSCENE or phase ~= SEQUENCE_CUTSCENE or over then
        return
    end
    sequence(director, SEQUENCE_SECOND_PART)
    after(SECOND_PART_AFTER, function(d)
        BATTLE:spawn(d, { BNPC_URIANGER, BNPC_PAPALYMO })
        bringOut(d, WAVE_1)
    end)
    talks(TALKS_SECOND_PART)
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
