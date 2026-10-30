-- Quest battle: Nouveau Riche (XxaUsa308_03862; the old GaiUsa308_00740's.)
-- It takes place in territory 232, a copy of the South Shroud (its TerritoryType has the South Shroud's level), so its
-- NPCs live in the fst_f1f3 drop-in (layer QB_XxaUsa308_002): the boss (4293096, BNpcBase 1639, BNpcName 2064, level
-- 23, 4522 HP), three lancers with it (4293092, 4293093, 4293095, base 1642), a lancer (4293098, base 1640) and a
-- gladiator (4293097, base 1646) on your side, the second wave (an archer 4293111, a thaumaturge 4293110 and a lancer
-- 4293109, bases 1644, 1643, 1642; a gladiator 4293099, base 1645, BNpcName 57), and two archers who come to help
-- (4293125, 4293126, base 1641, BNpcName 139). Layout IDs, levels, HP, positions, display flags, looks and the start spot
-- are retail's (Bob's Chronofoil capture caa5e908 of 2026-10-30, `questkit chronofoil --battle 0.13`, one run, won; a
-- level 27 player.) The client's battle script (GaiUsa308Btl_00740, Bob's dump of 2026-10-30) has scenes 1 (the opening
-- cutscene) and 7 (the closing one), and OnSequence001/002 (the objective and its message.)
--
-- Timed against that run (seconds from the fight starting, the director's scene 60000):
--   0       sequence 2: the boss and its three lancers, each on someone: the boss and one on your lancer, one on your
--           gladiator, one on you. Your lancer takes the one on it, then the boss; your gladiator the one on it, then
--           the newest arrivals, healing your lancer (Cure) when it's low
--   5, 10   battle talk 86 (the boss) and 87 (your gladiator, BNpcName 80)
--   29.0    the three lancers down, as the last one's death animation ended: sequence 3 and the second wave (the archer
--           and the gladiator on your gladiator, the thaumaturge and the lancer on you); the gladiator's 88 10 s later
--           and the boss's 89 15 s later
--   +30.0   sequence 4 and the two archers (they take the adds, then the boss); the first archer's 90 5.1 s later and
--           the boss's 91 10.1 s later
--   +30.1   sequence 5: the archer, the thaumaturge and the lancer of the second wave again (the thaumaturge and the
--           lancer on the first archer, the archer on your gladiator)
--   the second wave's gladiator down: its last words (92) as its health reaches 0
--   the boss down: its 93, sequence 6, "Duty Complete" and sequence 7 at once, then the battle's scene 7 (0xF8402EFB);
--           the quest moves to sequence 255
--   you down: sequence 0xFE
-- How hard each hits is in their timelines (QB13*).

-- Where retail puts the player and which way they face: their SpawnPlayer in the capture.
START_POSITION = { x = 40.19, y = 7.81, z = -9.13 }
START_ROTATION = -1.175
-- Retail's opening scene 1 here (Bob's Chronofoil capture caa5e908.)
OPENING_SCENE_FLAGS = 0xF8412EFB

BNPC_BOSS = 4293096
BNPC_LANCER = 4293098
BNPC_GLADIATOR = 4293097
BNPC_ARCHER_1 = 4293125
BNPC_ARCHER_2 = 4293126
BNPC_ENEMY_GLADIATOR = 4293099
-- Each wave: { [enemy] = whom it goes for (false: you) }.
FIRST_WAVE = { [4293092] = false, [4293093] = BNPC_GLADIATOR, [4293095] = BNPC_LANCER }
SECOND_WAVE = { [4293109] = false, [4293110] = false, [4293111] = BNPC_GLADIATOR, [BNPC_ENEMY_GLADIATOR] = BNPC_GLADIATOR }
THIRD_WAVE = { [4293109] = BNPC_ARCHER_1, [4293110] = BNPC_ARCHER_1, [4293111] = BNPC_GLADIATOR }

QUEST_SEQUENCE_AFTER_WIN = 255
SEQUENCE_FIGHTING = 2
SEQUENCE_ENDING = 6
SEQUENCE_WON = 7
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 7
FINISH_SCENE_FLAGS = 0xF8402EFB
FAIL_DELAY_UPDATES = 12

-- Timers, in updates (half a second each): a timer of n updates fires n / 2 s after it's set by another timer, and
-- (n - 1) / 2 s after it's set outside them (onCommence, a death: counted down from that same update.)
-- Retail's director answered the first wave's end as the last lancer's death animation ended (2 s after its health
-- reached 0.)
DEATH_ANIMATION_UPDATES = 4
NEXT_SEQUENCE_AFTER = 60

-- Who's talking (DirectorPopupMessage): by BNpcName, with retail's portraits.
SPEAKER_BOSS = { layout = BNPC_BOSS, name = 2064, icon = 73203 }
SPEAKER_GLADIATOR = { layout = BNPC_GLADIATOR, name = 80, icon = 73204 }
SPEAKER_ENEMY_GLADIATOR = { layout = BNPC_ENEMY_GLADIATOR, name = 57, icon = 73205 }
SPEAKER_ARCHER = { layout = BNPC_ARCHER_1, name = 139, icon = 73206 }

BATTLE = Battle.new()
ACTION_CURE = 120
BATTLE:ally(BNPC_LANCER, { role = "dps", split = false, focus = { "mine:" .. BNPC_LANCER, "boss" }, order = "nearest" })
BATTLE:ally(BNPC_GLADIATOR, {
    role = "dps",
    split = false,
    focus = { "mine:" .. BNPC_GLADIATOR, "adds", "boss" },
    order = "newest",
    heals = { { action = ACTION_CURE, targets = "allies", below = 63, every = 5.5 } },
})
for _, archer in ipairs({ BNPC_ARCHER_1, BNPC_ARCHER_2 }) do
    BATTLE:ally(archer, { role = "dps", split = false, focus = { "adds", "boss" }, order = "oldest", range = 15 })
end
BATTLE:enemy(BNPC_BOSS, {
    group = "boss",
    attacks = BNPC_LANCER,
    heals = { { action = ACTION_CURE, targets = { BNPC_BOSS }, below = 68, every = 11 } },
})

local over = false
local fail_in = nil
local phase = 1
local first_wave_left = 3
-- Timers of their own, side by side: { updates left, fn }.
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

-- Enemies sent after someone each ({ [enemy] = ally, or false for you }), that ally going for them first. Only those
-- not already standing (the third wave is the second's again.)
function arrive(director, wave)
    local ids = {}
    for enemy, target in pairs(wave) do
        if not BATTLE.alive[enemy] then
            local group = target and { "mine:" .. target, "adds" } or "adds"
            BATTLE:enemy(enemy, { group = group, attacks = target or nil })
            table.insert(ids, enemy)
        end
    end
    table.sort(ids)
    BATTLE:spawn(director, ids)
end

function sortedKeys(t)
    local keys = {}
    for key in pairs(t) do
        table.insert(keys, key)
    end
    table.sort(keys)
    return keys
end

function secondWave(director)
    phase = 3
    director:set_sequence(3)
    arrive(director, SECOND_WAVE)
    after(20, function(d) talk(d, SPEAKER_ENEMY_GLADIATOR, 88) end)
    after(30, function(d) talk(d, SPEAKER_BOSS, 89) end)
    after(NEXT_SEQUENCE_AFTER, function(d)
        phase = 4
        d:set_sequence(4)
        BATTLE:spawn(d, { BNPC_ARCHER_1, BNPC_ARCHER_2 })
        after(10, function(d2) talk(d2, SPEAKER_ARCHER, 90) end)
        after(20, function(d2) talk(d2, SPEAKER_BOSS, 91) end)
        after(NEXT_SEQUENCE_AFTER, function(d2)
            phase = 5
            d2:set_sequence(5)
            arrive(d2, THIRD_WAVE)
        end)
    end)
end

function onSetup(director)
    director:set_sequence(1)
end

function onCommence(director)
    director:set_sequence(SEQUENCE_FIGHTING)
    BATTLE:spawn(director, { BNPC_LANCER, BNPC_GLADIATOR })
    BATTLE:spawn(director, { BNPC_BOSS })
    arrive(director, FIRST_WAVE)
    after(11, function(d) talk(d, SPEAKER_BOSS, 86) end)
    after(21, function(d) talk(d, SPEAKER_GLADIATOR, 87) end)
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
    if layout_id == BNPC_BOSS then
        over = true
        timers = {}
        talk(director, SPEAKER_BOSS, 93)
        director:set_sequence(SEQUENCE_ENDING)
        director:set_sequence(SEQUENCE_WON)
        director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS)
        return
    end
    if layout_id == BNPC_ENEMY_GLADIATOR then
        talk(director, SPEAKER_ENEMY_GLADIATOR, 92)
    end
    if phase == 1 and FIRST_WAVE[layout_id] ~= nil then
        first_wave_left = first_wave_left - 1
        if first_wave_left <= 0 then
            phase = 2
            after(DEATH_ANIMATION_UPDATES, secondWave)
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
