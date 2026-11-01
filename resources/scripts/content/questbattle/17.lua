-- Quest battle 17 (ClsCnj006_00441, the conjurers' quest 441.)
-- It takes place in territory 235, whose TerritoryType has the South Shroud's level (f1f3), so its NPCs live in the
-- fst_f1f3 drop-in (layer QB_ClsCnj006_002): Sylphie on your side (base 558, 3873175, the QuestBattle sheet's
-- INIT_P_POP_SYLPHIE), the boss (301, 3796294, INIT_POP_BOSS, 2106 HP), two yodomi (1710, INIT_POP_YODOMI_01/02) and
-- two imps (1709, INIT_POP_IMP_01/02). Layout IDs, levels, HP, positions, display and character data flags and the
-- start spot are retail's (Bob's Chronofoil capture df6f5af9 of 2026-11-01, `questkit chronofoil --battle 0.4`, `0.6`
-- and `0.8`: lost, lost and won; a level 33 conjurer synced to 34). The layout IDs, yells and the drain's ID come from
-- the QuestBattle sheet's variables.
--
-- Timed against the three runs (seconds from the fight starting, the director's scene 60000):
--   0       sequence 2: the boss on you, Sylphie on the boss
--   5, 10   Sylphie's battle talk 48 and 49 (BNpcName 628, no portrait); 15.1 her yell QIB_YELL_01 (1003)
--   the boss at 70 % (75.5 -> 66.2, 73.1 -> 66.5, 71.4 -> 63.2): sequence 3, her yell QIB_YELL_02 (1004) and the first
--           yodomi 0.2-0.5 s later
--   in sequence 3, the boss's first drain (1127, on the yodomi): sequence 4 3.6 s after its cast began, her 50 5.0 s
--           after that; only in the first run (in the other two the boss was at 40 % first)
--   the boss at 40 % (42.1 -> 39.4, 44.4 -> 36.5, 46.6 -> 38.8): sequence 5 (from 3 or 4) and the imps 0.3-0.5 s later,
--           her 51 10.0 s after; 20.0 s after 5 sequence 6 and the second yodomi, her 52 5.0 s after
--   the boss and everything that came out down (the boss's health at 0, the last of them in the won run): "Duty
--           Complete" (60001) and sequence 7 at once, then the battle's scene 7 (0xF8402FFF); the quest moves to 255
--   you down: sequence 0xFE (at your health's 0, both losses), out 6 s later
-- The yodomi never act or move; the boss drains one about every 26 s, which heals it and gives it Regen for 21 s (its
-- rotation's `on_ally`, QuestBattleNpc_01127), so they're worth killing. Its 513 is a gaze: look away
-- (QuestBattleNpc_00513). How hard each hits is in their timelines (QB17*).

-- Where retail puts the player and which way they face: their SpawnPlayer in the capture (all three runs.)
START_POSITION = { x = 318.21, y = 4.36, z = -82.8 }
START_ROTATION = 1.933
-- Retail's opening scene 1 here (all three runs.)
OPENING_SCENE_FLAGS = 0xF841AFFF

BNPC_BOSS = INIT_POP_BOSS or 3796294
BNPC_SYLPHIE = INIT_P_POP_SYLPHIE or 3873175
BNPC_YODOMI_1 = INIT_POP_YODOMI_01 or 4273807
BNPC_YODOMI_2 = INIT_POP_YODOMI_02 or 4273810
IMPS = { INIT_POP_IMP_01 or 4273820, INIT_POP_IMP_02 or 4273821 }
YELL_START = QIB_YELL_01 or 1003
YELL_YODOMI = QIB_YELL_02 or 1004
ACTION_DRAIN = DEF_YODOMI_DORAIN or 1127

QUEST_SEQUENCE_AFTER_WIN = 255
SEQUENCE_FIGHTING = 2
SEQUENCE_WON = 7
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 7
FINISH_SCENE_FLAGS = 0xF8402FFF
FAIL_DELAY_UPDATES = 12

-- The boss's health (percent) that brings sequences 3 and 5.
FIRST_YODOMI_HEALTH = 70
IMPS_HEALTH = 40
-- Timers, in updates (half a second each).
SEQUENCE_4_AFTER_DRAIN = 8
SEQUENCE_6_AFTER = 40

SPEAKER_SYLPHIE = { layout = BNPC_SYLPHIE, name = 628, icon = 0 }

BATTLE = Battle.new()
ACTION_CURE = 120
-- Sylphie on the boss, then the imps when they come; she Cures you below 50 % (retail's at 7-50 %, never anyone else.)
BATTLE:ally(BNPC_SYLPHIE, {
    role = "dps",
    split = false,
    focus = { "adds", "boss" },
    order = "nearest",
    range = 15,
    heals = { { action = ACTION_CURE, targets = { Battle.PLAYER }, below = 50, every = 12 } },
})
BATTLE:enemy(BNPC_BOSS, { group = "boss" })
for _, imp in ipairs(IMPS) do
    BATTLE:enemy(imp, { group = "adds" })
end
for _, yodomi in ipairs({ BNPC_YODOMI_1, BNPC_YODOMI_2 }) do
    BATTLE:enemy(yodomi, { group = "yodomi", passive = true })
end

local over = false
local fail_in = nil
local phase = 1
local boss_health = 100
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

-- A yodomi: it stands where it came out and does nothing (retail's never moved or acted.)
function yodomi(director, layout)
    bringOut(director, { layout })
    director:hold_bnpc(layout)
end

function onSetup(director)
    director:set_sequence(1)
end

function onCommence(director)
    phase = SEQUENCE_FIGHTING
    director:set_sequence(SEQUENCE_FIGHTING)
    BATTLE:spawn(director, { BNPC_SYLPHIE })
    bringOut(director, { BNPC_BOSS })
    after(11, function(d) talk(d, SPEAKER_SYLPHIE, 48) end)
    after(21, function(d) talk(d, SPEAKER_SYLPHIE, 49) end)
    after(31, function(d) d:yell(BNPC_SYLPHIE, YELL_START) end)
end

function sequence5(director)
    phase = 5
    director:set_sequence(5)
    after(1, function(d) bringOut(d, IMPS) end)
    after(20, function(d) talk(d, SPEAKER_SYLPHIE, 51) end)
    after(SEQUENCE_6_AFTER, function(d)
        if phase >= SEQUENCE_WON then
            return
        end
        phase = 6
        d:set_sequence(6)
        yodomi(d, BNPC_YODOMI_2)
        after(10, function(d2) talk(d2, SPEAKER_SYLPHIE, 52) end)
    end)
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
    boss_health = health[BNPC_BOSS] or boss_health
    runTimers(director)
    local boss = health[BNPC_BOSS]
    if boss ~= nil then
        if phase == 2 and boss <= FIRST_YODOMI_HEALTH then
            phase = 3
            director:set_sequence(3)
            director:yell(BNPC_SYLPHIE, YELL_YODOMI)
            after(1, function(d) yodomi(d, BNPC_YODOMI_1) end)
        end
        if (phase == 3 or phase == 4) and boss <= IMPS_HEALTH then
            sequence5(director)
        end
    end
    BATTLE:update(director, health, player_health, info)
end

-- The boss's first drain in sequence 3 brings sequence 4 (retail's 3.6 s after its cast began.)
function onActorCast(director, layout_id, action_id, actor_id)
    if over or phase ~= 3 or layout_id ~= BNPC_BOSS or action_id ~= ACTION_DRAIN then
        return
    end
    after(SEQUENCE_4_AFTER_DRAIN, function(d)
        -- Not once the boss is at 40 %: sequence 5 comes instead (the second run's drain began at 25.0 s, sequence 5
        -- came at 28.1 and there was no 4.)
        if phase ~= 3 or boss_health <= IMPS_HEALTH then
            return
        end
        phase = 4
        d:set_sequence(4)
        after(10, function(d2) talk(d2, SPEAKER_SYLPHIE, 50) end)
    end)
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    standing[layout_id] = nil
    -- Won with the boss down and nothing else left standing (Bob: the fight doesn't end until every enemy is down.)
    if standing[BNPC_BOSS] == nil and next(standing) == nil then
        over = true
        timers = {}
        phase = SEQUENCE_WON
        director:set_sequence(SEQUENCE_WON)
        director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS)
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
