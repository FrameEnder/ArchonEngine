-- Quest battle 21: The Face of Thal (ClsGla200_00263, the Gladiators' level 20 quest.)
-- It takes place in territory 255, a copy of Western Thanalan (retail's positions in it stand on Western Thanalan's
-- ground, to the hundredth of a yalm, against the same capture's visits to 140), so its NPCs live in the wil_w1f1
-- drop-in (layer QB_ClsGla200_001). Levels, HP, positions, gear, display and character data flags and the start spot
-- are retail's (Bob's Chronofoil capture 941895d2 of 2026-11-06, `questkit chronofoil --battle 0.14`: one run, won; a
-- gladiator synced to 24). On your side: the one who speaks (4000991, base 638, BNpcName 689, level 50, 1941 HP) and
-- four gladiators (4001010-4001013, bases 868-871). Against you, three waves: four lancers (base 633), three archers
-- (634) and two gladiators (635); then three lancers, three archers and two conjurers (636); then three lancers,
-- three thaumaturges (637) and their leader (4001144, base 872, level 25, 2554 HP.)
--
-- Timed against the run (seconds from the fight starting, the director's scene 60000):
--   0       sequence 2: everyone out
--   5.1     battle talk 165 (the one who speaks, no portrait)
--   the seventh of the first nine down (42.5; the director heard of it 2.0 s later, its death animation): sequence 3,
--           the second wave 0.45 s later
--   10.05 s after sequence 3 (54.6): sequence 4; talks 166 and 167 5.1 and 10.1 s after it
--   the sixth of the second wave down (84.6, at once, as That Old Familiar Feeling's second wave): sequence 5, the
--           third wave 0.17 s later, talk 168 10.1 s after sequence 5
--   their leader at half health (124.7): sequence 6 and talk 169 at once
--   their leader down, the rest already (136.0, 0.3 s later, before its death animation): "Duty Complete" (60001) and
--           sequence 7, then the battle's scene 7 (a cutscene, 0xF8402EFB) 3.7 s later; the quest moves to 255
-- The one who speaks Cures the others on your side below 70 % (retail's at 3-66 % after the heal, every 6.3 s or so;
-- never himself or you), the second gladiator him, the others and himself below 65 % (45-65 % after, every 5.9 s),
-- and the conjurers their side below 60 % (5-59 % after, every 6.2 s.)

-- Where retail puts the player and which way they face: their SpawnPlayer in the capture.
START_POSITION = { x = -239.7, y = 32.44, z = 395.6 }
START_ROTATION = 2.354
-- Retail's opening scene 1 here: no cutscene (over in 0.2 s.)
OPENING_SCENE_FLAGS = 0x2001

BNPC_LEADER_ALLY = 4000991
BNPC_GLADIATOR_1 = 4001010
BNPC_GLADIATOR_2 = 4001011
BNPC_GLADIATOR_3 = 4001012
BNPC_GLADIATOR_4 = 4001013
ALLIES = { BNPC_LEADER_ALLY, BNPC_GLADIATOR_1, BNPC_GLADIATOR_2, BNPC_GLADIATOR_3, BNPC_GLADIATOR_4 }
BNPC_BOSS = 4001144

-- Each wave: its enemies, their group (for whom the allies go for) and whom each went for first on retail (nil: the
-- player.)
WAVE_1 = {
    { 4001017, "lancers", BNPC_LEADER_ALLY }, { 4001018, "lancers", BNPC_LEADER_ALLY },
    { 4001019, "lancers", BNPC_LEADER_ALLY }, { 4001020, "lancers", BNPC_LEADER_ALLY },
    { 4001025, "archers", BNPC_GLADIATOR_1 }, { 4001026, "archers", BNPC_GLADIATOR_2 },
    { 4001073, "archers", BNPC_GLADIATOR_2 },
    { 4001028, "gladiators", BNPC_GLADIATOR_3 }, { 4001029, "gladiators", BNPC_GLADIATOR_4 },
}
WAVE_2 = {
    { 4001081, "lancers", BNPC_LEADER_ALLY }, { 4001082, "lancers", BNPC_LEADER_ALLY },
    { 4001083, "lancers", BNPC_LEADER_ALLY },
    { 4001089, "archers", BNPC_GLADIATOR_2 }, { 4001091, "archers", BNPC_GLADIATOR_2 }, { 4001229, "archers", nil },
    { 4001094, "healers", nil }, { 4001095, "healers", BNPC_LEADER_ALLY },
}
WAVE_3 = {
    { 4001104, "lancers", BNPC_LEADER_ALLY }, { 4001105, "lancers", BNPC_LEADER_ALLY },
    { 4001106, "lancers", BNPC_LEADER_ALLY },
    { 4001113, "casters", BNPC_GLADIATOR_1 }, { 4001116, "casters", BNPC_GLADIATOR_2 }, { 4001118, "casters", nil },
    { BNPC_BOSS, "boss", BNPC_LEADER_ALLY },
}

QUEST_SEQUENCE_AFTER_WIN = 255
SEQUENCE_FIGHTING = 2
SEQUENCE_SECOND_WAVE = 3
SEQUENCE_SECOND_WAVE_TALK = 4
SEQUENCE_THIRD_WAVE = 5
SEQUENCE_BOSS_HALF = 6
SEQUENCE_WON = 7
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 7
FINISH_SCENE_FLAGS = 0xF8402EFB
FAIL_DELAY_UPDATES = 12

-- How many of the first wave down bring the second, and of the second the third; the boss's health for sequence 6.
FIRST_WAVE_DOWN = 7
SECOND_WAVE_DOWN = 6
BOSS_HALF = 50

-- Timers, in updates (half a second each.)
DEATH_NOTICED_AFTER = 4
WAVE_OUT_AFTER = 1
SEQUENCE_4_AFTER = 20
TALK_START_AFTER = 10
TALK_166_AFTER = 10
TALK_167_AFTER = 20
TALK_168_AFTER = 20
WON_AFTER = 0
-- A timer started from onActorDeath counts the update it was started in, so one more.
FROM_DEATH = 1

-- The battle talk (the content's text IDs, retail's.)
SPEAKER = { layout = BNPC_LEADER_ALLY, name = 689, icon = 0 }
TALK_START = 165
TALK_SEQUENCE_4 = 166
TALK_SEQUENCE_4_LATER = 167
TALK_THIRD_WAVE = 168
TALK_BOSS_HALF = 169

ACTION_CURE = 120

BATTLE = Battle.new()
BATTLE:ally(BNPC_LEADER_ALLY, {
    role = "dps",
    split = false,
    focus = { "lancers", "boss", "any" },
    order = "nearest",
    heals = { { action = ACTION_CURE, targets = { BNPC_GLADIATOR_1, BNPC_GLADIATOR_2, BNPC_GLADIATOR_3, BNPC_GLADIATOR_4 },
        below = 70, every = 6.3 } },
})
BATTLE:ally(BNPC_GLADIATOR_1, { role = "dps", split = false, focus = { "archers", "boss", "any" }, order = "nearest" })
BATTLE:ally(BNPC_GLADIATOR_2, {
    role = "dps",
    split = false,
    focus = { "archers", "boss", "any" },
    order = "nearest",
    heals = { { action = ACTION_CURE,
        targets = { BNPC_LEADER_ALLY, BNPC_GLADIATOR_1, BNPC_GLADIATOR_3, BNPC_GLADIATOR_4, "self" },
        below = 65, every = 5.9 } },
})
for _, ally in ipairs({ BNPC_GLADIATOR_3, BNPC_GLADIATOR_4 }) do
    BATTLE:ally(ally, { role = "dps", split = false, focus = { "gladiators", "lancers", "boss", "any" }, order = "nearest" })
end
for wave, enemies in ipairs({ WAVE_1, WAVE_2, WAVE_3 }) do
    for _, enemy in ipairs(enemies) do
        local role = { group = { enemy[2], "wave" .. wave, "adds" }, attacks = enemy[3] }
        if enemy[2] == "healers" then
            role.heals = { { action = ACTION_CURE, targets = "allies", below = 60, every = 6.2 } }
        end
        BATTLE:enemy(enemy[1], role)
    end
end

local over = false
local fail_in = nil
local phase = 1
local first_down = 0
local second_down = 0
-- The enemies out and not yet down (by their health.)
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

function layoutsOf(wave)
    local layouts = {}
    for _, enemy in ipairs(wave) do
        table.insert(layouts, enemy[1])
    end
    return layouts
end

function inWave(wave, layout)
    for _, enemy in ipairs(wave) do
        if enemy[1] == layout then
            return true
        end
    end
    return false
end

function bringOut(director, wave)
    local layouts = layoutsOf(wave)
    for _, layout in ipairs(layouts) do
        standing[layout] = true
    end
    BATTLE:spawn(director, layouts)
end

function onSetup(director)
    director:set_sequence(1)
end

function onCommence(director)
    phase = SEQUENCE_FIGHTING
    director:set_sequence(SEQUENCE_FIGHTING)
    BATTLE:spawn(director, ALLIES)
    bringOut(director, WAVE_1)
    after(TALK_START_AFTER, function(d) talk(d, SPEAKER, TALK_START) end)
end

-- Sequence 3 and the second wave, then sequence 4 on a timer.
function secondWave(director)
    phase = SEQUENCE_SECOND_WAVE
    director:set_sequence(SEQUENCE_SECOND_WAVE)
    after(WAVE_OUT_AFTER, function(d) bringOut(d, WAVE_2) end)
    after(SEQUENCE_4_AFTER, function(d)
        phase = SEQUENCE_SECOND_WAVE_TALK
        d:set_sequence(SEQUENCE_SECOND_WAVE_TALK)
        after(TALK_166_AFTER, function(d2) talk(d2, SPEAKER, TALK_SEQUENCE_4) end)
        after(TALK_167_AFTER, function(d2) talk(d2, SPEAKER, TALK_SEQUENCE_4_LATER) end)
        -- The second wave fell fast enough: the third comes now.
        if second_down >= SECOND_WAVE_DOWN then
            thirdWave(d, 0)
        end
    end)
end

function thirdWave(director, from_death)
    phase = SEQUENCE_THIRD_WAVE
    director:set_sequence(SEQUENCE_THIRD_WAVE)
    bringOut(director, WAVE_3)
    after(TALK_168_AFTER + from_death, function(d) talk(d, SPEAKER, TALK_THIRD_WAVE) end)
end

function won(director)
    over = true
    timers = {}
    phase = SEQUENCE_WON
    director:set_sequence(SEQUENCE_WON)
    director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS)
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
    if phase == SEQUENCE_THIRD_WAVE and health[BNPC_BOSS] ~= nil and health[BNPC_BOSS] <= BOSS_HALF then
        phase = SEQUENCE_BOSS_HALF
        director:set_sequence(SEQUENCE_BOSS_HALF)
        talk(director, SPEAKER, TALK_BOSS_HALF)
    end
    BATTLE:update(director, health, player_health, info)
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    standing[layout_id] = nil
    -- The second wave's sixth brought the third at once on retail, not 2 s later.
    if inWave(WAVE_2, layout_id) then
        second_down = second_down + 1
        if second_down >= SECOND_WAVE_DOWN and phase == SEQUENCE_SECOND_WAVE_TALK then
            thirdWave(director, FROM_DEATH)
        end
    end
    if inWave(WAVE_1, layout_id) then
        after(DEATH_NOTICED_AFTER, function(d)
            first_down = first_down + 1
            if phase == SEQUENCE_FIGHTING and first_down >= FIRST_WAVE_DOWN then
                secondWave(d)
            end
        end)
    end
    -- The win came with the last one's health at 0 on retail (0.3 s after), not after its death animation.
    if phase >= SEQUENCE_THIRD_WAVE and next(standing) == nil then
        after(WON_AFTER + FROM_DEATH, won)
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
