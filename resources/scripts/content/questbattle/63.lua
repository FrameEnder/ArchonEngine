-- Quest battle 63 (GaiUsc604_01004, Fool Me Twice, the main scenario's level 46 quest in Mor Dhona.)
-- It takes place in territory 308, taken to be a copy of Mor Dhona (l1f1): its spots stand a few yalms from where
-- the quest's EOBJECT0 is used (-129.6, -2.7, -629.2 in Mor Dhona, where retail let the player out), so its NPCs are
-- layer QB_GaiUsc604_001 of lak_l1f1. Layout IDs, levels, HP, positions, display and character data flags, looks and
-- the start spot are retail's (Connor's Chronofoil capture e834e436 of 2026-11-07, `questkit chronofoil --battle
-- 9.195` and `9.197`: two runs, the first lost when the ally fell, the second won; a level 47 player.) On your side:
-- one ally you must keep alive (4270049, base 1987, 10560 HP). Against you: the boss (4270036, base 1988, 6540 HP)
-- and a first wave beside him (a gladiator 4270011, two lancers 4270007-8, two archers 4270002-3), all out from the
-- start; then a second wave (an archer 4270061, a lancer 4270060, a conjurer 4270059.)
--
-- Retail's flow (seconds from the fight starting, the director's scene 60000):
--   0       sequence 2; battle talks 101 and 102 (the ally) at 5.1 and 10.1
--   the first wave down (all five; the boss may live): sequence 3 4.3 s after the last one's health reached 0, the
--           second wave 0.2 s later; talk 103 (the second wave's lancer) 5.1 s after sequence 3, 104 (the ally) 10.1 s
--           after
--   the boss under 40 % (39.3 %, the won run): sequence 4; he casts 1286 on himself 1.5 s later (2.7 s, status 61)
--           and sequence 5 comes 5.5 s after sequence 4; talks 105 and 106 (the ally) 6.1 and 11.9 s after it
--   the boss's health at 0: "Duty Complete" (60001) and sequence 6 2.1 s later, the battle's scene 6 (0xF8402EFB)
--           3.7 s after that; the quest moves to sequence 255 with ITEM0 (UI8BH) gone
--   lost: the ally's health at 0 (its last words, talk 107, and sequence 0xFE at once: the lost run, 69.6 s in) or
--           yours
-- The opening scene 1 is a cutscene (0xF8412EF3). Who fights whom and how hard is in their timelines (QB63*.)

-- Where retail puts the player and which way they face: their spawn in both runs.
START_POSITION = { x = -127.58, y = -2.66, z = -630.68 }
START_ROTATION = -0.719
OPENING_SCENE_FLAGS = 0xF8412EF3

BNPC_ALLY = 4270049
BNPC_BOSS = 4270036
BNPC_GLADIATOR = 4270011
FIRST_WAVE = { BNPC_GLADIATOR, 4270007, 4270008, 4270002, 4270003 }
SECOND_WAVE = { 4270061, 4270060, 4270059 }
BNPC_SECOND_LANCER = 4270060
BNPC_CONJURER = 4270059

QUEST_SEQUENCE_AFTER_WIN = 255
-- The win takes ITEM0 (UI8BH) with it, as retail's UpdateQuest did.
QUEST_VARS_AFTER_WIN = { [1] = 0 }
SEQUENCE_FIGHTING = 2
SEQUENCE_SECOND_WAVE = 3
SEQUENCE_BOSS_LOW = 4
SEQUENCE_BOSS_GUARDED = 5
SEQUENCE_WON = 6
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 6
FINISH_SCENE_FLAGS = 0xF8402EFB
FAIL_DELAY_UPDATES = 10

-- The boss's health mark (percent) for sequence 4.
BOSS_LOW = 40

-- Timers, in updates (half a second each). A timer of n fires n / 2 s after it's set by another timer or in onUpdate
-- (after that update's timers have run), and (n - 1) / 2 s after it's set before the next update (onCommence,
-- onActorDeath.)
TALK_101_AFTER = 11
TALK_102_AFTER = 21
SEQUENCE_3_AFTER = 10
TALK_103_AFTER = 10
TALK_104_AFTER = 20
SEQUENCE_5_AFTER = 11
TALK_105_AFTER = 12
TALK_106_AFTER = 24
-- The win came 2.1 s after the boss's health reached 0 (his death animation.)
WON_AFTER_DEATH = 5

-- Who's talking (DirectorPopupMessage, by whom it came from), with retail's names and portraits.
SPEAKER_ALLY = { layout = BNPC_ALLY, name = 1486, icon = 73184 }
SPEAKER_LANCER = { layout = BNPC_SECOND_LANCER, name = 1811, icon = 73185 }

ACTION_ALLY_HEAL = 964
ACTION_CURE = 120

BATTLE = Battle.new()
-- On retail the ally went for whoever was after you, the adds first (a lancer, the gladiator), then the boss, the
-- second wave's lancer as it came, and the boss again; it healed you or itself at 15-48 % (964, every 5.7-7.2 s.)
BATTLE:ally(BNPC_ALLY, {
    role = "tank",
    focus = { "adds", "boss" },
    order = "oldest",
    heals = { { action = ACTION_ALLY_HEAL, targets = { Battle.PLAYER, "self" }, below = 48, every = 7.0 } },
})
-- Whom each went for first on retail (nil: you): everyone you, but the archers, who took the ally on at once.
BATTLE:enemy(BNPC_BOSS, { group = "boss", attacks = nil })
BATTLE:enemy(BNPC_GLADIATOR, { group = "adds", attacks = nil })
BATTLE:enemy(4270007, { group = "adds", attacks = nil })
BATTLE:enemy(4270008, { group = "adds", attacks = nil })
BATTLE:enemy(4270002, { group = "adds", attacks = BNPC_ALLY })
BATTLE:enemy(4270003, { group = "adds", attacks = BNPC_ALLY })
BATTLE:enemy(4270061, { group = "adds", attacks = nil })
BATTLE:enemy(BNPC_SECOND_LANCER, { group = "adds", attacks = nil })
-- The conjurer cured the boss at 39 % (706, the won run.)
BATTLE:enemy(BNPC_CONJURER, {
    group = "adds",
    attacks = nil,
    heals = { { action = ACTION_CURE, targets = "allies", below = 45, every = 10.0 } },
})

local over = false
local fail_in = nil
local phase = 1
local first_left = {}
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
    director:battle_talk(speaker.layout, speaker.name, text, speaker.icon)
end

function sequence(director, number)
    phase = number
    director:set_sequence(number)
end

function onSetup(director)
    director:set_sequence(1)
    -- Everyone is out before the opening, as on retail.
    for _, layout in ipairs(FIRST_WAVE) do
        first_left[layout] = true
    end
    BATTLE:spawn(director, { BNPC_ALLY, BNPC_BOSS })
    BATTLE:spawn(director, FIRST_WAVE)
end

function onCommence(director)
    sequence(director, SEQUENCE_FIGHTING)
    after(TALK_101_AFTER, function(d) talk(d, SPEAKER_ALLY, 101) end)
    after(TALK_102_AFTER, function(d) talk(d, SPEAKER_ALLY, 102) end)
end

function secondWave(director)
    sequence(director, SEQUENCE_SECOND_WAVE)
    BATTLE:spawn(director, SECOND_WAVE)
    after(TALK_103_AFTER, function(d) talk(d, SPEAKER_LANCER, 103) end)
    after(TALK_104_AFTER, function(d) talk(d, SPEAKER_ALLY, 104) end)
end

function won(director)
    over = true
    timers = {}
    sequence(director, SEQUENCE_WON)
    director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS,
        QUEST_VARS_AFTER_WIN)
end

function lost(director)
    over = true
    timers = {}
    director:set_sequence(SEQUENCE_FAILED)
    fail_in = FAIL_DELAY_UPDATES
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
    local boss = health[BNPC_BOSS]
    if phase == SEQUENCE_SECOND_WAVE and boss ~= nil and boss < BOSS_LOW then
        sequence(director, SEQUENCE_BOSS_LOW)
        after(SEQUENCE_5_AFTER, function(d)
            sequence(d, SEQUENCE_BOSS_GUARDED)
            after(TALK_105_AFTER, function(d2) talk(d2, SPEAKER_ALLY, 105) end)
            after(TALK_106_AFTER, function(d2) talk(d2, SPEAKER_ALLY, 106) end)
        end)
    end
    BATTLE:update(director, health, player_health, info)
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    if layout_id == BNPC_ALLY then
        talk(director, SPEAKER_ALLY, 107)
        lost(director)
        return
    end
    if layout_id == BNPC_BOSS then
        -- Nothing else comes once he's down: only the win, with his death animation.
        timers = {}
        phase = SEQUENCE_WON
        after(WON_AFTER_DEATH, won)
        return
    end
    if first_left[layout_id] then
        first_left[layout_id] = nil
        if next(first_left) == nil and phase == SEQUENCE_FIGHTING then
            after(SEQUENCE_3_AFTER, secondWave)
        end
    end
end

-- Losing sends you back to where you entered, still on the same step of the quest, so you can try again.
function onPlayerDeath(director)
    if over then
        return
    end
    lost(director)
end
