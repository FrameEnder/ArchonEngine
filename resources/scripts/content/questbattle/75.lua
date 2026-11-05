-- Quest battle 75: Seer Folly (JobWhm300_01079, the White Mage's first quest.)
-- It takes place in territory 319, whose TerritoryType has the Central Shroud's level (f1f1), so its NPCs live in the
-- fst_f1 drop-in (layer QB_JobWhm300_001), with the layout IDs the QuestBattle sheet names: Raya-O-Senna and A-Ruhn-Senna
-- on your side (INIT_P_POP_RAYAO_01, base 1675, BNpcName 1422; INIT_P_POP_ARUN_01, 1676, 1423; both white mages), the
-- boss (INIT_POP_1_BOSS_TRANT_01, base 1674, 6209 HP) with a raptor and two bats (INIT_POP_1_RPT_01, _FLY_01/02), then
-- two waves of three. Levels, HP, positions, display and character data flags and the start spot are retail's (Bob's
-- Chronofoil capture 8d550e33 of 2026-11-05, `questkit chronofoil --battle 0.4`: one run, won; a level 35 conjurer
-- synced to 34). The sheet's other layout IDs (_RPT_02/03, _FLY_03/04, _SLUG_02/03, _FLY_02 of wave 2, _WATER_ERMNT_02/03,
-- _FLY_02 of wave 3) didn't come in that run, so they aren't spawned.
--
-- Timed against the run (seconds from the fight starting, the director's scene 60000):
--   0       sequence 2: the boss on Raya-O-Senna, its raptor and bats on A-Ruhn-Senna
--   5.1, 10.1  battle talk 86 (Raya-O-Senna) and 87 (A-Ruhn-Senna), no portrait
--   the boss's raptor and bats down (20.8, as the last of them fell): sequence 3 and the second wave at once, a raptor
--           on Raya-O-Senna, a bat on A-Ruhn-Senna, a slug on you (the boss at 85.6 %: not its health)
--   the boss at 40 % (44.1 -> 38.3, 56.7): sequence 4 and the third wave at once, a raptor on A-Ruhn-Senna, a bat on you,
--           a water sprite on Raya-O-Senna; her 88 3.1 s later
--   everything down (111.7, the last bat 1.4 s before): "Duty Complete" (60001) and sequence 5, then the battle's scene 5
--           (0xF8402EFB) 3.1 s later; the quest moves to 255 with UI8BH 1 (the key item)
-- What each one does is in their timelines (QB75*): the boss's Silkscreen (701) knocks back whoever it hits, the slug's
-- Viscous Discharge (1166) silences.

-- Where retail puts the player and which way they face: their SpawnPlayer in the capture.
START_POSITION = { x = -284.58, y = 19.84, z = 119.85 }
START_ROTATION = -0.149
-- Retail's opening scene 1 here.
OPENING_SCENE_FLAGS = 0xF8412FFF

BNPC_RAYAO = INIT_P_POP_RAYAO_01 or 4293115
BNPC_ARUHN = INIT_P_POP_ARUN_01 or 4293116
BNPC_BOSS = INIT_POP_1_BOSS_TRANT_01 or 4293124
WAVE_1 = { INIT_POP_1_RPT_01 or 4293129, INIT_POP_1_FLY_01 or 4293133, INIT_POP_1_FLY_02 or 4293134 }
WAVE_2_RAPTOR = INIT_POP_2_RPT_01 or 4293148
WAVE_2_BAT = INIT_POP_2_FLY_01 or 4293146
WAVE_2_SLUG = INIT_POP_2_SLUG_01 or 4293144
WAVE_3_RAPTOR = INIT_POP_3_RPT_01 or 4293155
WAVE_3_BAT = INIT_POP_3_FLY_01 or 4293153
WAVE_3_SPRITE = INIT_POP_3_WATER_ERMNT_01 or 4293151

QUEST_SEQUENCE_AFTER_WIN = 255
-- UI8BH 1: ITEM0, which the client lists in the next sequence (retail's UpdateQuest as the battle was won.)
QUEST_VARS_AFTER_WIN = { [1] = 0x10 }
SEQUENCE_FIGHTING = 2
SEQUENCE_SECOND_WAVE = 3
SEQUENCE_THIRD_WAVE = 4
SEQUENCE_WON = 5
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 5
FINISH_SCENE_FLAGS = 0xF8402EFB
FAIL_DELAY_UPDATES = 12

-- The boss's health (percent) that brings the third wave.
THIRD_WAVE_HEALTH = 40

-- The battle talk (the content's text IDs, retail's), in updates (half a second each) after what brings it.
SPEAKER_RAYAO = { layout = BNPC_RAYAO, name = 1422, icon = 0 }
SPEAKER_ARUHN = { layout = BNPC_ARUHN, name = 1423, icon = 0 }
TALK_RAYAO_START = 86
TALK_ARUHN_START = 87
TALK_RAYAO_THIRD_WAVE = 88
TALK_RAYAO_AFTER = 11
TALK_ARUHN_AFTER = 21
TALK_THIRD_WAVE_AFTER = 6

ACTION_CURE = 120

BATTLE = Battle.new()
-- Both stay by you, casting from 5 yalms, and Cure each other below 60 % (retail's at 40-59 % and 59-80 %, never
-- you.) A-Ruhn-Senna takes the boss's adds first; Raya-O-Senna the boss.
BATTLE:ally(BNPC_ARUHN, {
    role = "dps",
    split = false,
    focus = { "wave1", "boss", "adds" },
    order = "nearest",
    range = 5,
    heals = { { action = ACTION_CURE, targets = { BNPC_RAYAO, BNPC_ARUHN }, below = 60, every = 12 } },
})
BATTLE:ally(BNPC_RAYAO, {
    role = "dps",
    split = false,
    focus = { "boss", "adds" },
    order = "nearest",
    range = 5,
    heals = { { action = ACTION_CURE, targets = { BNPC_ARUHN, BNPC_RAYAO }, below = 60, every = 12 } },
})
BATTLE:enemy(BNPC_BOSS, { group = "boss", attacks = BNPC_RAYAO })
for _, layout in ipairs(WAVE_1) do
    BATTLE:enemy(layout, { group = { "wave1", "adds" }, attacks = BNPC_ARUHN })
end
BATTLE:enemy(WAVE_2_RAPTOR, { group = "adds", attacks = BNPC_RAYAO })
BATTLE:enemy(WAVE_2_BAT, { group = "adds", attacks = BNPC_ARUHN })
BATTLE:enemy(WAVE_2_SLUG, { group = "adds" })
BATTLE:enemy(WAVE_3_RAPTOR, { group = "adds", attacks = BNPC_ARUHN })
BATTLE:enemy(WAVE_3_BAT, { group = "adds" })
BATTLE:enemy(WAVE_3_SPRITE, { group = "adds", attacks = BNPC_RAYAO })

local over = false
local fail_in = nil
local phase = 1
-- The enemies out and not yet down (the boss among them.)
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

function bringOut(director, layouts)
    for _, layout in ipairs(layouts) do
        standing[layout] = true
    end
    BATTLE:spawn(director, layouts)
end

-- The boss's adds down: the second wave.
BATTLE:onCleared("wave1", function(director)
    if over or phase ~= SEQUENCE_FIGHTING then
        return
    end
    phase = SEQUENCE_SECOND_WAVE
    director:set_sequence(SEQUENCE_SECOND_WAVE)
    bringOut(director, { WAVE_2_RAPTOR, WAVE_2_BAT, WAVE_2_SLUG })
end)

-- The boss at 40 %: the third wave (from the second or, if the boss got there first, the first.)
BATTLE:onHealth(BNPC_BOSS, THIRD_WAVE_HEALTH, function(director)
    if over or phase >= SEQUENCE_THIRD_WAVE then
        return
    end
    phase = SEQUENCE_THIRD_WAVE
    director:set_sequence(SEQUENCE_THIRD_WAVE)
    bringOut(director, { WAVE_3_RAPTOR, WAVE_3_BAT, WAVE_3_SPRITE })
    after(TALK_THIRD_WAVE_AFTER, function(d) talk(d, SPEAKER_RAYAO, TALK_RAYAO_THIRD_WAVE) end)
end)

function onSetup(director)
    director:set_sequence(1)
end

function onCommence(director)
    phase = SEQUENCE_FIGHTING
    director:set_sequence(SEQUENCE_FIGHTING)
    BATTLE:spawn(director, { BNPC_RAYAO, BNPC_ARUHN })
    bringOut(director, { BNPC_BOSS })
    bringOut(director, WAVE_1)
    after(TALK_RAYAO_AFTER, function(d) talk(d, SPEAKER_RAYAO, TALK_RAYAO_START) end)
    after(TALK_ARUHN_AFTER, function(d) talk(d, SPEAKER_ARUHN, TALK_ARUHN_START) end)
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
    standing[layout_id] = nil
    BATTLE:died(director, layout_id)
    -- Won once the third wave has come and nothing is left standing (retail: the last of them, then at once.)
    if phase >= SEQUENCE_THIRD_WAVE and next(standing) == nil then
        over = true
        timers = {}
        phase = SEQUENCE_WON
        director:set_sequence(SEQUENCE_WON)
        director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS,
            QUEST_VARS_AFTER_WIN)
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
