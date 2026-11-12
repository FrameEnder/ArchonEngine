-- Quest battle 62 (GaiUsc408_00983, Notorious Biggs, the main scenario's level 43 quest in Coerthas.)
-- It takes place in territory 304, taken to be a copy of Coerthas Central Highlands (r1f1): retail's start spot is
-- where it let the player out in the Central Highlands afterwards (-499.6, 287.1, 166.9), so its NPCs are layer
-- QB_GaiUsc408_001 of roc_r1f1. Layout IDs, levels, HP, positions, display and character data flags, looks and the
-- start spot are retail's (Connor's Chronofoil capture e834e436 of 2026-11-07, `questkit chronofoil --battle 9.53`,
-- won, a level 47 player; Bob's 689b15da of 2026-11-09, `--battle 0.2` lost and `0.4` won, level 45.) On your side:
-- one to keep alive (4264070, base 1656, 5109 HP), a level 50 fighter (4264069, base 1650) and a level 50 conjurer
-- (4264062, base 1651). Against you: the boss (4264130, base 1652, 12038 HP) and three archers (4264135, 4264138,
-- 4264140), out from the start; the second wave (4264377, 4264378, 4264380, 4264381, 4264382, 4264384) and the third
-- (4264403, 4264404, 4264406, 4264408) come later.
--
-- Retail's flow (seconds from the fight starting, the director's scene 60000), the same in the three runs:
--   0       sequence 2; battle talks 97 (the one to keep alive), 98 (the boss), 99 (the fighter), 100 (the conjurer)
--           at 5.1, 10.1, 15.1 and 20.1
--   the three archers' health at 0, or the boss under 50 % before that (49 %, 9.53): sequence 3 at once, the second
--           wave 0.4 s later; talk 101 (the boss) 5.1 s after sequence 3, 102 (the conjurer) 15.1 s after it
--   five of the boss and the second wave down (which five differed: 9.53 the boss, 4264377, 4264378, 4264380 and
--           4264381; 0.4 4264384 in place of 4264378): sequence 4 at once, the third wave 0.2 s later, talks 103 (the
--           fighter) and 104 (the conjurer) 5.1 and 10.1 s after it
--   seven of the boss and the second and third waves down: sequence 5 and talk 105 (the one to keep alive) at once
--   everyone down: "Duty Complete" (60001) and sequence 6 2.1 s after the last one's health reached 0, the battle's
--           scene 6 (0xF8402EFB) 3.1-3.7 s after that; the quest moves to sequence 255
--   the one to keep alive under 20 % (19 %, 0.2): NpcYell 1530 from it
--   lost: its health at 0 (talk 108 and sequence 0xFE at once: 0.2, 90.8 s in) or yours
-- The opening scene 1 is a cutscene (0xF8412FFF). How hard each fights is in their timelines (QB62*), with how near
-- each gets (`attack_range`) and the boss's 1068 walk (`reach`); the later waves stand 4.9 s where they appear first
-- (Battle.ARRIVAL_STAND_SECONDS: 4.8-5.0 s on retail, both waves, four runs.)

-- Where retail puts the player and which way they face: their spawn in the three runs.
START_POSITION = { x = -498.89, y = 286.89, z = 167.44 }
START_ROTATION = -2.512
OPENING_SCENE_FLAGS = 0xF8412FFF

BNPC_ESCORT = 4264070
BNPC_FIGHTER = 4264069
BNPC_CONJURER = 4264062
BNPC_BOSS = 4264130
ARCHERS = { 4264135, 4264138, 4264140 }
SECOND_WAVE = { 4264384, 4264382, 4264381, 4264380, 4264378, 4264377 }
THIRD_WAVE = { 4264408, 4264406, 4264404, 4264403 }

QUEST_SEQUENCE_AFTER_WIN = 255
SEQUENCE_FIGHTING = 2
SEQUENCE_SECOND_WAVE = 3
SEQUENCE_THIRD_WAVE = 4
SEQUENCE_LAST = 5
SEQUENCE_WON = 6
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 6
FINISH_SCENE_FLAGS = 0xF8402EFB
FAIL_DELAY_UPDATES = 10

-- The boss's health mark (percent) that brings the second wave before the archers are down, the escort's for the
-- yell, and how many of the boss and the later waves have to fall for sequences 4 and 5.
BOSS_HALF = 50
ESCORT_YELL = 20
YELL_ESCORT = 1530
DOWN_FOR_THIRD_WAVE = 5
DOWN_FOR_LAST = 7

-- Timers, in updates (half a second each). A timer of n fires n / 2 s after it's set by another timer or in onUpdate
-- (after that update's timers have run), and (n - 1) / 2 s after it's set before the next update (onCommence,
-- onActorDeath.)
TALK_97_AFTER = 11
TALK_98_AFTER = 21
TALK_99_AFTER = 31
TALK_100_AFTER = 41
TALK_101_AFTER = 10
TALK_102_AFTER = 30
TALK_103_AFTER = 10
TALK_104_AFTER = 20
-- The win came 2.1 s after the last one's health reached 0 (its death animation.)
WON_AFTER_DEATH = 5

-- Who's talking (DirectorPopupMessage, by whom it came from), with retail's names and portraits.
SPEAKER_ESCORT = { layout = BNPC_ESCORT, name = 1418, icon = 73182 }
SPEAKER_BOSS = { layout = BNPC_BOSS, name = 2087, icon = 73186 }
SPEAKER_FIGHTER = { layout = BNPC_FIGHTER, name = 619, icon = 73156 }
SPEAKER_CONJURER = { layout = BNPC_CONJURER, name = 729, icon = 73187 }

ACTION_SELF_HEAL = 57
ACTION_CURE = 120
ACTION_PROTECT = 129

BATTLE = Battle.new()
-- Who went for whom on retail (four runs: Bob's Chronofoil captures 689b15da 0.2 and 0.4, da424662 0.22, 898dd0b9 0.12,
-- `questkit battle`), and from how near (centre to centre, where they stopped walking):
-- * the fighter held the boss until it fell (3.0-3.7 yalms), then took the ones after it, the weakest first (the
--   archer 4264378 before the 1655), then the rest; it never pulled anyone off the others (every enemy kept whom it
--   was sent after unless the player drew it), so it doesn't `protect`;
-- * the conjurer and the escort met whoever came for them: each wave's enemies after them, the moment they appeared
--   (they set off 0.2-1.6 s after, and ran 25-35 yalms to meet them), the weakest first (the conjurer's archer before
--   its marauders), and the boss or the other adds when no one was after them. The conjurer fought from 6.0-7.1 yalms,
--   like the rest, not from afar; Cure at 40-57 %, mostly on the most hurt, and 129 on someone every 20 s.
BATTLE:ally(BNPC_FIGHTER, {
    role = "tank",
    protect = false,
    focus = { "boss", "mine", "adds" },
    order = "lowest_hp",
    heals = { { action = ACTION_SELF_HEAL, targets = { "self" }, below = 45, every = 30.0 } },
})
BATTLE:ally(BNPC_CONJURER, {
    focus = { "mine", "adds", "boss" },
    order = "lowest_hp",
    range = 6.5,
    heals = { { action = ACTION_CURE, targets = "party", below = 50, every = 10.0 } },
    buffs = { { action = ACTION_PROTECT, every = 20.0 } },
})
BATTLE:ally(BNPC_ESCORT, { role = "dps", focus = { "mine", "boss", "adds" }, order = "lowest_hp", range = 3.5 })
-- Whom each went for first on retail (nil: you), the same in every run.
BATTLE:enemy(BNPC_BOSS, {
    group = "boss",
    attacks = BNPC_FIGHTER,
    heals = { { action = ACTION_SELF_HEAL, targets = { "self" }, below = 49, every = 20.0 } },
})
BATTLE:enemy(4264140, { group = "adds", attacks = nil })
BATTLE:enemy(4264138, { group = "adds", attacks = BNPC_FIGHTER })
BATTLE:enemy(4264135, { group = "adds", attacks = BNPC_CONJURER })
BATTLE:enemy(4264384, { group = "adds", attacks = BNPC_ESCORT })
BATTLE:enemy(4264382, { group = "adds", attacks = BNPC_FIGHTER })
BATTLE:enemy(4264381, { group = "adds", attacks = BNPC_CONJURER })
BATTLE:enemy(4264380, { group = "adds", attacks = BNPC_CONJURER })
BATTLE:enemy(4264378, { group = "adds", attacks = BNPC_FIGHTER })
BATTLE:enemy(4264377, { group = "adds", attacks = BNPC_CONJURER })
BATTLE:enemy(4264408, { group = "adds", attacks = BNPC_ESCORT })
BATTLE:enemy(4264406, { group = "adds", attacks = BNPC_ESCORT })
BATTLE:enemy(4264404, { group = "adds", attacks = BNPC_FIGHTER })
BATTLE:enemy(4264403, { group = "adds", attacks = BNPC_CONJURER })

local over = false
local fail_in = nil
local phase = 1
local archers_left = {}
local left = {}
local counted = {}
local down = 0
local yelled = false
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

function bringOut(director, layouts, count)
    for _, layout in ipairs(layouts) do
        left[layout] = true
        if count then
            counted[layout] = true
        end
    end
    BATTLE:spawn(director, layouts)
end

function onSetup(director)
    director:set_sequence(1)
    for _, layout in ipairs(ARCHERS) do
        archers_left[layout] = true
    end
    BATTLE:spawn(director, { BNPC_FIGHTER, BNPC_CONJURER, BNPC_ESCORT })
    bringOut(director, { BNPC_BOSS }, true)
    bringOut(director, ARCHERS, false)
end

function onCommence(director)
    sequence(director, SEQUENCE_FIGHTING)
    after(TALK_97_AFTER, function(d) talk(d, SPEAKER_ESCORT, 97) end)
    after(TALK_98_AFTER, function(d) talk(d, SPEAKER_BOSS, 98) end)
    after(TALK_99_AFTER, function(d) talk(d, SPEAKER_FIGHTER, 99) end)
    after(TALK_100_AFTER, function(d) talk(d, SPEAKER_CONJURER, 100) end)
end

function secondWave(director)
    sequence(director, SEQUENCE_SECOND_WAVE)
    bringOut(director, SECOND_WAVE, true)
    after(TALK_101_AFTER, function(d) talk(d, SPEAKER_BOSS, 101) end)
    after(TALK_102_AFTER, function(d) talk(d, SPEAKER_CONJURER, 102) end)
end

function won(director)
    over = true
    timers = {}
    sequence(director, SEQUENCE_WON)
    director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS)
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
    if phase == SEQUENCE_FIGHTING and boss ~= nil and boss < BOSS_HALF then
        secondWave(director)
    end
    local escort = health[BNPC_ESCORT]
    if not yelled and escort ~= nil and escort < ESCORT_YELL then
        yelled = true
        director:yell(BNPC_ESCORT, YELL_ESCORT)
    end
    BATTLE:update(director, health, player_health, info)
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    if layout_id == BNPC_ESCORT then
        talk(director, SPEAKER_ESCORT, 108)
        lost(director)
        return
    end
    if archers_left[layout_id] then
        archers_left[layout_id] = nil
        if next(archers_left) == nil and phase == SEQUENCE_FIGHTING then
            secondWave(director)
        end
    end
    -- The boss and the later waves count towards sequences 4 and 5.
    if counted[layout_id] then
        down = down + 1
        if down >= DOWN_FOR_THIRD_WAVE and phase == SEQUENCE_SECOND_WAVE then
            sequence(director, SEQUENCE_THIRD_WAVE)
            bringOut(director, THIRD_WAVE, true)
            after(TALK_103_AFTER, function(d) talk(d, SPEAKER_FIGHTER, 103) end)
            after(TALK_104_AFTER, function(d) talk(d, SPEAKER_CONJURER, 104) end)
        elseif down >= DOWN_FOR_LAST and phase == SEQUENCE_THIRD_WAVE then
            sequence(director, SEQUENCE_LAST)
            talk(director, SPEAKER_ESCORT, 105)
        end
    end
    if left[layout_id] then
        left[layout_id] = nil
        if next(left) == nil and phase >= SEQUENCE_THIRD_WAVE then
            timers = {}
            after(WON_AFTER_DEATH, won)
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
