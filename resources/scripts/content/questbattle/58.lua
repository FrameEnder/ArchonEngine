-- Quest battle: Big Trouble in Little Ala Mhigo (GaiUsa709_00782.)
-- It takes place in territory 306, whose TerritoryType has Southern Thanalan's level (w1f4), so its NPCs live in the
-- wil_w1f4 drop-in (layer QB_GaiUsa709_001): on your side Wilred (base 1984, 4251863), two young men (1985, 4251864;
-- 1986, 4251867), Gundobald (1982, 4251873) and a conjurer (1983, 4251874); against you the boss (1977, 4252035,
-- 5712 HP) and three waves of adds (the QuestBattle sheet's INIT_POP_1ST/2ST/3ST: three, three and four of them came
-- out; the sheet names a few more retail never spawned). Layout IDs, levels, HP, positions, display and character
-- data flags, looks and the start spot are retail's (Bob's Chronofoil capture 20ef0e90 of 2026-10-31,
-- `questkit chronofoil --battle 0.1`, won; a level 31 player), checked against two more runs of other characters'
-- (captures 9faa9bac and bd51ca23 of 2026-10-31, `--battle 0.1`, both won). Retail had them all out from the moment
-- you arrived; here they come out as the fight starts, as in the other battles (the opening cutscene makes its own.)
--
-- Timed against the three runs (seconds from the fight starting, the director's scene 60000):
--   0       sequence 2: the boss on Gundobald, the first wave's lancer on the first young man and its two pugilists on
--           the conjurer; your side takes the nearest add, Gundobald the boss throughout
--   5, 10   battle talk 98 (Wilred) and 99 (Gundobald); 15, 20 the young men's yells (NpcYell YELL_QIB_01, 02)
--   the first wave down, as the last one's death animation ended (2.0 s after its health reached 0 in every run;
--           31.7, 35.2 and 28.4 s in): sequence 3; 0.2-0.5 s later the second wave (the first young man, Wilred,
--           Wilred); the boss's 100 at + 5.1, Gundobald's 101 and sequence 4 at + 10.1 (10.0 in the third)
--   sequence 4 + 5.0  Gundobald's 102
--   sequence 5, whichever comes first, each seen once: the whole second wave down (its last one's health at 0, at
--           once: the third run, 28.9 s after 4), the boss at 40 % (40.9 -> 39.7 in the second run, 23.0 s after 4)
--           or 50.0 s after sequence 4 (the first run: one of the second wave still up, the boss at 53 %). With it the
--           boss's 103 and the third wave (Wilred, the conjurer, the first young man or Wilred if he has fallen,
--           Gundobald); + 10.1 sequence 6
--   the third wave's third down (its health at 0, at once in all three runs: 148.9, 151.1 and 112.6 s in), after
--           sequence 6: sequence 7; Gundobald's 104 1.0-1.1 s later
--   a young man's health at 0: his last words (107 the first, 108 the second), whatever else; the conjurer fell in the
--           second run with no words
--   the boss's health at 0 (175.0, 170.8 and 139.2 s in): 2.0 s later, as its death animation ended, "Duty Complete"
--           (60001) and sequence 8, then the battle's scene 8 (0xF8402EFB); the quest moves to sequence 255
-- The three runs are, in order, captures 20ef0e90, 9faa9bac and bd51ca23.
--   you down: sequence 0xFE
-- How hard each hits is in their timelines (QB58*).

-- Where retail puts the player and which way they face: their SpawnPlayer in the capture.
START_POSITION = { x = 39.49, y = 3.84, z = -364.19 }
START_ROTATION = -0.14
-- Retail's opening scene 1 here (Bob's Chronofoil capture 20ef0e90.)
OPENING_SCENE_FLAGS = 0xF8412EFB

-- The QuestBattle sheet's names for them (INIT_P_POP_*: your side; INIT_POP_*ST_*: the waves.)
BNPC_WILRED = INIT_P_POP_01 or 4251863
BNPC_YOUNG_MAN_A = INIT_P_POP_02 or 4251864
BNPC_YOUNG_MAN_B = INIT_P_POP_03 or 4251867
BNPC_GUNDOBALD = INIT_P_POP_04 or 4251873
BNPC_CONJURER = INIT_P_POP_05 or 4251874
BNPC_BOSS = INIT_POP_1ST_01 or 4252035
-- The waves retail brought out: { [enemy] = the one it went for first, or a list of them in order (whoever's first
-- still standing; none, or false: you) }.
FIRST_WAVE = {
    [INIT_POP_1ST_05 or 4252775] = BNPC_YOUNG_MAN_A,
    [INIT_POP_1ST_03 or 4252773] = BNPC_CONJURER,
    [INIT_POP_1ST_02 or 4252772] = BNPC_CONJURER,
}
SECOND_WAVE = {
    [INIT_POP_2ST_05 or 4252785] = BNPC_YOUNG_MAN_A,
    [INIT_POP_2ST_03 or 4252783] = BNPC_WILRED,
    [INIT_POP_2ST_01 or 4252781] = BNPC_WILRED,
}
THIRD_WAVE = {
    [INIT_POP_3ST_04 or 4252789] = BNPC_WILRED,
    [INIT_POP_3ST_03 or 4252788] = BNPC_CONJURER,
    -- The first young man in two runs, Wilred in the one where he had fallen (captures 9faa9bac, bd51ca23, 20ef0e90.)
    [INIT_POP_3ST_02 or 4252787] = { BNPC_YOUNG_MAN_A, BNPC_WILRED },
    [INIT_POP_3ST_01 or 4252786] = BNPC_GUNDOBALD,
}
-- The boss's health (percent) that brings sequence 5 in sequence 4 (retail's at 40.9 -> 39.7, capture 9faa9bac.)
BOSS_HEALTH_FOR_SEQUENCE_5 = 40
-- The third wave's fall that brings sequence 7 (retail's third of four.)
THIRD_WAVE_DOWN_FOR_SEQUENCE_7 = 3

QUEST_SEQUENCE_AFTER_WIN = 255
SEQUENCE_FIGHTING = 2
SEQUENCE_WON = 8
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 8
FINISH_SCENE_FLAGS = 0xF8402EFB
FAIL_DELAY_UPDATES = 12

-- Timers, in updates (half a second each): a timer of n updates fires n / 2 s after it's set by another timer, and
-- (n - 1) / 2 s after it's set outside them (onCommence, a death: counted down from that same update.)
SECOND_WAVE_AFTER = 1
SEQUENCE_4_AFTER = 20
-- At the latest: sequence 5 comes sooner with the second wave down or the boss at BOSS_HEALTH_FOR_SEQUENCE_5.
SEQUENCE_5_LATEST = 100
SEQUENCE_6_AFTER = 20
-- Retail's director answered the first wave's last death and the boss's 2 s after their health reached 0, as the
-- death animation ended; the third wave's third at once.
DEATH_ANIMATION_UPDATES = 4

-- Who's talking (DirectorPopupMessage), with retail's BNpcNames; the portraits are the sheet's FACE_GRAPHIC_*.
SPEAKER_WILRED = { layout = BNPC_WILRED, name = 2071, icon = FACE_GRAPHIC_WILRED or 73198 }
SPEAKER_GUNDOBALD = { layout = BNPC_GUNDOBALD, name = 2069, icon = FACE_GRAPHIC_GUNDOBALD or 73199 }
SPEAKER_BOSS = { layout = BNPC_BOSS, name = 2067, icon = FACE_GRAPHIC_AMALJABOSS or 73200 }
SPEAKER_YOUNG_MAN_A = { layout = BNPC_YOUNG_MAN_A, name = 2072, icon = FACE_GRAPHIC_YOUNGMAN_A or 73201 }
SPEAKER_YOUNG_MAN_B = { layout = BNPC_YOUNG_MAN_B, name = 2072, icon = FACE_GRAPHIC_YOUNGMAN_B or 73202 }
-- Their last words, as their health reaches 0.
LAST_WORDS = {
    [BNPC_YOUNG_MAN_A] = { speaker = SPEAKER_YOUNG_MAN_A, text = 107 },
    [BNPC_YOUNG_MAN_B] = { speaker = SPEAKER_YOUNG_MAN_B, text = 108 },
}

BATTLE = Battle.new()
ACTION_CURE = 120
-- Gundobald on the boss from start to end, Curing the others below 64 % (retail's at 9-64 % across the three runs,
-- every 6.2 s or so; once you, never himself); the conjurer stays where it is, Curing its side and itself below 50 %
-- (retail's at 2-50 %, every 5.6 s at the most often; never you, Gundobald once); the rest take the nearest add.
BATTLE:ally(BNPC_GUNDOBALD, {
    role = "dps",
    split = false,
    focus = { "boss", "adds" },
    heals = { { action = ACTION_CURE,
        targets = { Battle.PLAYER, BNPC_WILRED, BNPC_CONJURER, BNPC_YOUNG_MAN_A, BNPC_YOUNG_MAN_B },
        below = 64, every = 6.2 } },
})
BATTLE:ally(BNPC_CONJURER, {
    role = "dps",
    split = false,
    focus = { "adds", "boss" },
    order = "nearest",
    range = 25,
    heals = { { action = ACTION_CURE, targets = { BNPC_WILRED, BNPC_YOUNG_MAN_A, BNPC_YOUNG_MAN_B, "self" },
        below = 50, every = 5.6 } },
})
for _, ally in ipairs({ BNPC_WILRED, BNPC_YOUNG_MAN_A, BNPC_YOUNG_MAN_B }) do
    BATTLE:ally(ally, { role = "dps", split = false, focus = { "adds", "boss" }, order = "nearest" })
end
BATTLE:enemy(BNPC_BOSS, { group = "boss", attacks = BNPC_GUNDOBALD })

local over = false
local fail_in = nil
local phase = 1
local first_wave_left = 0
local second_wave_left = nil
local third_wave_down = 0
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

-- A wave comes out, each after the one retail's went for (the next on its list, or you, if that one has fallen.)
function arrive(director, wave)
    local ids = {}
    for enemy, targets in pairs(wave) do
        local target = nil
        for _, candidate in ipairs(type(targets) == "table" and targets or { targets }) do
            if candidate and BATTLE.alive[candidate] then
                target = candidate
                break
            end
        end
        BATTLE:enemy(enemy, { group = "adds", attacks = target })
        table.insert(ids, enemy)
    end
    table.sort(ids)
    BATTLE:spawn(director, ids)
    return #ids
end

function sequence3(director)
    phase = 3
    director:set_sequence(3)
    after(SECOND_WAVE_AFTER, function(d) second_wave_left = arrive(d, SECOND_WAVE) end)
    after(10, function(d) talk(d, SPEAKER_BOSS, 100) end)
    after(SEQUENCE_4_AFTER, function(d)
        phase = 4
        talk(d, SPEAKER_GUNDOBALD, 101)
        d:set_sequence(4)
        after(10, function(d2) talk(d2, SPEAKER_GUNDOBALD, 102) end)
        after(SEQUENCE_5_LATEST, sequence5)
        if second_wave_left ~= nil and second_wave_left <= 0 then
            sequence5(d)
        end
    end)
end

-- Once, in sequence 4: the second wave down, the boss at BOSS_HEALTH_FOR_SEQUENCE_5 or SEQUENCE_5_LATEST.
function sequence5(director)
    if phase ~= 4 then
        return
    end
    phase = 5
    director:set_sequence(5)
    talk(director, SPEAKER_BOSS, 103)
    arrive(director, THIRD_WAVE)
    after(SEQUENCE_6_AFTER, function(d)
        phase = 6
        d:set_sequence(6)
        checkSequence7(d)
    end)
end

-- Sequence 7: the third wave's third down, once sequence 6 has come.
function checkSequence7(director)
    if phase == 6 and third_wave_down >= THIRD_WAVE_DOWN_FOR_SEQUENCE_7 then
        phase = 7
        director:set_sequence(7)
        after(3, function(d) talk(d, SPEAKER_GUNDOBALD, 104) end)
    end
end

function onSetup(director)
    director:set_sequence(1)
end

function onCommence(director)
    director:set_sequence(SEQUENCE_FIGHTING)
    BATTLE:spawn(director, { BNPC_WILRED, BNPC_YOUNG_MAN_A, BNPC_YOUNG_MAN_B, BNPC_GUNDOBALD, BNPC_CONJURER })
    BATTLE:spawn(director, { BNPC_BOSS })
    first_wave_left = arrive(director, FIRST_WAVE)
    after(11, function(d) talk(d, SPEAKER_WILRED, 98) end)
    after(21, function(d) talk(d, SPEAKER_GUNDOBALD, 99) end)
    after(31, function(d) d:yell(BNPC_YOUNG_MAN_A, YELL_QIB_01 or 1127) end)
    after(41, function(d) d:yell(BNPC_YOUNG_MAN_B, YELL_QIB_02 or 1128) end)
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
    if phase == 4 and health[BNPC_BOSS] ~= nil and health[BNPC_BOSS] <= BOSS_HEALTH_FOR_SEQUENCE_5 then
        sequence5(director)
    end
    BATTLE:update(director, health, player_health, info)
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    local last_words = LAST_WORDS[layout_id]
    if last_words ~= nil then
        talk(director, last_words.speaker, last_words.text)
        return
    end
    if layout_id == BNPC_BOSS then
        -- Nothing else happens now: the win, as its death animation ends.
        phase = SEQUENCE_WON
        timers = {}
        after(DEATH_ANIMATION_UPDATES, won)
        return
    end
    if FIRST_WAVE[layout_id] ~= nil and phase == 1 then
        first_wave_left = first_wave_left - 1
        if first_wave_left <= 0 then
            phase = 2
            after(DEATH_ANIMATION_UPDATES, sequence3)
        end
    elseif SECOND_WAVE[layout_id] ~= nil and second_wave_left ~= nil then
        second_wave_left = second_wave_left - 1
        if second_wave_left <= 0 then
            sequence5(director)
        end
    elseif THIRD_WAVE[layout_id] ~= nil then
        third_wave_down = third_wave_down + 1
        checkSequence7(director)
    end
end

function won(director)
    over = true
    timers = {}
    director:set_sequence(SEQUENCE_WON)
    director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS)
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
