-- Quest battle 61 (GaiUsc004_00941, the main scenario's level 41 quest in Coerthas.)
-- It takes place in territory 302, whose TerritoryType has Coerthas Central Highlands' level (r1f1), so its NPCs live
-- in the roc_r1f1 drop-in (layer QB_GaiUsc004_001). Layout IDs, levels, HP, positions, flags, looks and the start spot
-- are retail's (Bob's Chronofoil captures 130088ec and 3c56e09f of 2026-11-06, `questkit chronofoil --battle 0.11`,
-- and `--battle 0.13`, `0.15`, `0.17`: four runs, the first and last won, the second lost when the gladiator fell, the
-- third when the player did; level 41.) On your side: a gladiator you must keep alive (4256801, base 1750, 2401 HP), a
-- conjurer (4256802, base 1751) and an archer (4256803, base 1752). Against you: the one you fight (4256791, base 1746,
-- 17030 HP), three adds (base 1748, 4256796-4256798), then what he turns into (4291248, base 1747, 12772 HP), a wyvern
-- (4256804, base 1749) and one more add (4292887).
--
-- Retail's flow (seconds from the fight starting, the director's scene 60000), the same in every run:
--   0       sequence 2; battle talk 75 at 5.1, 76 at 10.1
--   17.4-17.5 sequence 3, the three adds 0.2-0.5 s later
--   the boss under 50 % (49.6, 49.2, 49.6, 49.7): sequence 4, talks 77, 78, 79 0.1, 5.1 and 10.1 s later; 11.1-13.0 s
--           after sequence 4 he casts Dragon's Blood (1162, 3.7 s) on himself and 4.4-4.5 s after the cast began
--           sequence 5, whatever his health (31-46 %); he fades (607) 1.1 s later, what he turns into appears 1.3 s
--           later (0.6 once), sequence 6 2.0-2.1 s after sequence 5 as he's taken away; talks 80 and 81 4.0 and 8.0 s
--           after sequence 6
--   what he turned into under 75 % (74-75 %, four runs): sequence 7, the wyvern 0.2-0.5 s later, talks 82 and 83 5.1
--           and 10.0 s after sequence 7
--   it under 40 % (40 %, three runs): sequence 8, the last add 0.2-0.4 s later; under 20 % (19.4, 19.3): talk 84
--   it, the wyvern and the last add down: "Duty Complete" (60001) and sequence 9 2.0-2.1 s after the last one's
--           health reached 0 (its death animation), the battle's scene 9 (0xF8402EFB) after it; the quest moves to
--           sequence 255
--   the archer's and the conjurer's health at 0: their last words, talk 87 and 86 (the archer fell in every run)
--   lost: the gladiator's health at 0 (talk 85 and sequence 0xFE at once) or yours (sequence 0xFE)
-- The opening scene 1 is a cutscene (0xF8412EFB.) Who fights whom and how hard is in their timelines (QB61*.)

-- Where retail puts the player and which way they face: their SpawnPlayer in every run.
START_POSITION = { x = -865.98, y = 226.1, z = 14.44 }
START_ROTATION = -1.832
OPENING_SCENE_FLAGS = 0xF8412EFB

BNPC_GLADIATOR = 4256801
BNPC_CONJURER = 4256802
BNPC_ARCHER = 4256803
BNPC_BOSS = 4256791
BNPC_TURNED = 4291248
BNPC_WYVERN = 4256804
BNPC_LAST_ADD = 4292887
ALLIES = { BNPC_GLADIATOR, BNPC_CONJURER, BNPC_ARCHER }
ADDS = { 4256798, 4256797, 4256796 }
-- Who has to be down to win (the boss is gone by then.)
LAST_ONES = { [BNPC_TURNED] = true, [BNPC_WYVERN] = true, [BNPC_LAST_ADD] = true }

QUEST_SEQUENCE_AFTER_WIN = 255
SEQUENCE_FIGHTING = 2
SEQUENCE_ADDS = 3
SEQUENCE_BOSS_HALF = 4
SEQUENCE_TURNING = 5
SEQUENCE_TURNED = 6
SEQUENCE_WYVERN = 7
SEQUENCE_LAST_ADD = 8
SEQUENCE_WON = 9
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 9
FINISH_SCENE_FLAGS = 0xF8402EFB
FAIL_DELAY_UPDATES = 10

-- The health marks (percent): the boss's for sequence 4, what he turns into for 7, 8 and talk 84.
BOSS_HALF = 50
TURNED_WYVERN = 75
TURNED_LAST_ADD = 40
TURNED_LOW = 20

-- Dragon's Blood: the boss's turning (the director's, on himself.)
ACTION_DRAGONS_BLOOD = 1162

-- Timers, in updates (half a second each). A timer of n fires n / 2 s after it's set by another timer or in onUpdate
-- (after that update's timers have run), and (n - 1) / 2 s after it's set before the next update (onCommence,
-- onActorDeath, onEventActionCast.)
TALK_75_AFTER = 11
TALK_76_AFTER = 21
SEQUENCE_3_AFTER = 36
ADDS_AFTER = 1
TALK_77_AFTER = 1
TALK_78_AFTER = 10
TALK_79_AFTER = 20
DRAGONS_BLOOD_AFTER = 24
SEQUENCE_5_AFTER = 9
BOSS_FADES_AFTER = 2
TURNED_AFTER = 3
SEQUENCE_6_AFTER = 4
TALK_80_AFTER = 8
TALK_81_AFTER = 16
WYVERN_AFTER = 1
TALK_82_AFTER = 10
TALK_83_AFTER = 20
LAST_ADD_AFTER = 1
-- The win came with the last one's death animation, 2.0 s after its health reached 0.
WON_AFTER_DEATH = 5

-- Who's talking (DirectorPopupMessage, by whom it came from), with retail's names and portraits.
SPEAKER_GLADIATOR = { layout = BNPC_GLADIATOR, name = 2085, icon = 73189 }
SPEAKER_BOSS = { layout = BNPC_BOSS, name = 2083, icon = 73193 }
SPEAKER_TURNED = { layout = BNPC_TURNED, name = 2084, icon = 73190 }
SPEAKER_ARCHER_FALLS = { layout = BNPC_ARCHER, name = 2086, icon = 73192 }
SPEAKER_CONJURER_FALLS = { layout = BNPC_CONJURER, name = 2086, icon = 73191 }

ACTION_CURE = 120
ACTION_CURE_II = 135

BATTLE = Battle.new()
-- On retail the gladiator stayed on the boss, then what he turned into, then the last add; the conjurer the same,
-- taking the wyvern a while; the archer the boss, then the adds as they came, until he fell (every run.)
-- He never took an enemy off you or the others (no target but those three in any run), so no `protect`: pulling one
-- every 10 s, as Battle's tanks do, drew the adds onto him.
BATTLE:ally(BNPC_GLADIATOR, { role = "tank", focus = { "boss", "turned", "last" }, order = "oldest", protect = false })
BATTLE:ally(BNPC_CONJURER, {
    role = "dps",
    split = false,
    focus = { "boss", "turned", "wyvern", "last", "adds" },
    order = "oldest",
    range = 15,
    -- Retail's: Cure on the gladiator, you and itself at 25-47 % (median 40), Cure II at 12-28 % (four runs.)
    heals = {
        { action = ACTION_CURE_II, targets = { BNPC_GLADIATOR, Battle.PLAYER, "self" }, below = 25, every = 19.0 },
        { action = ACTION_CURE, targets = { BNPC_GLADIATOR, Battle.PLAYER, "self" }, below = 42, every = 12.0 },
    },
})
-- The archer went for 4256796 as the adds came (all five runs, 0.4-1.2 s after they appeared), then 4256797.
-- Each keeps to its own focus: as Battle's DPS they split the enemies between them, and the archer took the second
-- in the conjurer's list (an add while the conjurer was on the boss), never the add retail's took.
BATTLE:ally(BNPC_ARCHER, {
    role = "dps", focus = { "add96", "add97", "adds", "boss" }, order = "oldest", range = 15, split = false,
})
-- Whom each went for first on retail (nil: you; the White Mage's run, 21f967d2, and three of the other four.)
-- The boss stayed on the gladiator the whole fight in every run but the Paladin's (130088ec, whose tank stance took it
-- at 31 s): a healer's or a damage dealer's enmity never got it off him. Kawari's started him at Battle's usual 300 and
-- turned to the White Mage (Bob's bundle 20261110-131148: five Ruins on her.) A Paladin's stance still takes it.
BOSS_ON_GLADIATOR = 10000
BATTLE:enemy(BNPC_BOSS, { group = "boss", attacks = BNPC_GLADIATOR, enmity = BOSS_ON_GLADIATOR })
BATTLE:enemy(4256798, { group = "adds", attacks = nil })
BATTLE:enemy(4256797, { group = { "add97", "adds" }, attacks = BNPC_ARCHER })
BATTLE:enemy(4256796, { group = { "add96", "adds" }, attacks = BNPC_CONJURER })
BATTLE:enemy(BNPC_TURNED, { group = "turned", attacks = nil })
BATTLE:enemy(BNPC_WYVERN, { group = "wyvern", attacks = BNPC_CONJURER })
BATTLE:enemy(BNPC_LAST_ADD, { group = "last", attacks = BNPC_CONJURER })

-- The adds' way down (all five runs, to a yalm; Kawari's walked into the cliff below their ledge and stayed there,
-- Bob's bundle 20261110-122816): each stood 5.0-5.4 s where it appeared, then 4256796, 4256798 and the last add
-- (4292887) walked to the ledge's edge (2.7-2.8 yalms a second) and dropped 15 yalms to the arena in 1.56 s, landing
-- 8.4-10.8 s after they appeared. 4256797 walked (2.0 a second) to the front of the ledge by 9.1 s and fought from
-- there, 14 yalms above and away from the archer it hit, then took the edge at 34-39 s (36.4, median) and dropped.
-- The wyvern stood 5.2-5.6 s where it appeared (5.4, median) before it set off for the arena. Times from the
-- appearance; spots and heights retail's.
ADD_ROUTES = {
    [BNPC_WYVERN] = { legs = { { wait = 5.4 } } },
    [4256796] = {
        from = { x = -909.83, y = 247.17, z = -26.83 },
        legs = {
            { wait = 5.2 },
            { to = { x = -907.38, y = 246.2, z = -18.2 }, speed = 2.8 },
            { to = { x = -906.6, y = 231.72, z = -17.2 }, speed = 0.81 },
        },
    },
    [4256798] = {
        from = { x = -915.29, y = 248.38, z = -25.96 },
        legs = {
            { wait = 5.05 },
            { to = { x = -907.38, y = 246.17, z = -18.2 }, speed = 2.7 },
            { to = { x = -906.77, y = 231.6, z = -17.0 }, speed = 0.86 },
        },
    },
    [BNPC_LAST_ADD] = {
        from = { x = -911.8, y = 248.04, z = -23.6 },
        legs = {
            { wait = 5.1 },
            { to = { x = -907.35, y = 246.17, z = -18.2 }, speed = 2.7 },
            { to = { x = -906.74, y = 231.71, z = -16.95 }, speed = 0.89 },
        },
    },
    [4256797] = {
        from = { x = -907.2, y = 246.76, z = -24.85 },
        legs = {
            { wait = 5.0 },
            { to = { x = -899.11, y = 243.85, z = -25.89 }, speed = 2.0 },
        },
    },
}
ADD97_LEDGE = { x = -899.11, y = 243.85, z = -25.89 }
ADD97_DROPS = {
    { to = { x = -898.95, y = 243.64, z = -19.97 }, speed = 2.7 },
    { to = { x = -898.41, y = 231.1, z = -18.45 }, speed = 1.28 },
}
-- From its appearance to setting off for the edge (36.4 s at the edge, less the 2.2 s walk there), in updates.
ADD97_DROPS_AFTER = 68
-- How far it reaches from the ledge (it hit the archer 13.9 yalms off), and its usual reach once down (melee.)
ADD97_LEDGE_REACH = 15
MELEE_REACH = 5

local over = false
local fail_in = nil
local phase = 1
local left = {}
local turned_low = false
-- Where the boss last stood: what he turns into appears there (retail's, five runs: 0-5.5 yalms from its layout's
-- spot, wherever he was.)
local boss_at = nil
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

function sequence(director, number)
    phase = number
    director:set_sequence(number)
end

function bringOut(director, layouts, at)
    for _, layout in ipairs(layouts) do
        if LAST_ONES[layout] then
            left[layout] = true
        end
    end
    -- Away from the start, so the allies don't pick them as they appear (Battle:walk then says until when.)
    for _, layout in ipairs(layouts) do
        if ADD_ROUTES[layout] ~= nil then
            BATTLE.away[layout] = math.huge
        end
    end
    BATTLE:spawn(director, layouts, at)
    for _, layout in ipairs(layouts) do
        local route = ADD_ROUTES[layout]
        if route ~= nil then
            local took = BATTLE:walk(director, layout, route.from, route.legs)
            if layout == 4256797 then
                -- It fights from the ledge until it drops, and the allies leave it be until then (the archer went for
                -- it only after it dropped, every run.)
                BATTLE.away[layout] = math.huge
                director:delay(took)
                director:set_attack_range(layout, ADD97_LEDGE_REACH)
                director:delay(0)
                after(ADD97_DROPS_AFTER, add97Drops)
            end
        end
    end
end

function add97Drops(director)
    if not BATTLE.alive[4256797] then
        return
    end
    local took = BATTLE:walk(director, 4256797, ADD97_LEDGE, ADD97_DROPS)
    director:delay(took)
    director:set_attack_range(4256797, MELEE_REACH)
    director:delay(0)
end

function onSetup(director)
    director:set_sequence(1)
end

function onCommence(director)
    sequence(director, SEQUENCE_FIGHTING)
    BATTLE:spawn(director, { BNPC_GLADIATOR, BNPC_CONJURER, BNPC_ARCHER, BNPC_BOSS })
    after(TALK_75_AFTER, function(d) talk(d, SPEAKER_GLADIATOR, 75) end)
    after(TALK_76_AFTER, function(d) talk(d, SPEAKER_BOSS, 76) end)
    after(SEQUENCE_3_AFTER, function(d)
        sequence(d, SEQUENCE_ADDS)
        after(ADDS_AFTER, function(d2) bringOut(d2, ADDS) end)
    end)
end

-- The boss at half: the talk, then he turns (Dragon's Blood) into what you fight next.
function bossHalf(director)
    sequence(director, SEQUENCE_BOSS_HALF)
    after(TALK_77_AFTER, function(d) talk(d, SPEAKER_GLADIATOR, 77) end)
    after(TALK_78_AFTER, function(d) talk(d, SPEAKER_BOSS, 78) end)
    after(TALK_79_AFTER, function(d) talk(d, SPEAKER_BOSS, 79) end)
    after(DRAGONS_BLOOD_AFTER, function(d)
        d:use_action(BNPC_BOSS, ACTION_DRAGONS_BLOOD, BNPC_BOSS)
        after(SEQUENCE_5_AFTER, function(d2)
            sequence(d2, SEQUENCE_TURNING)
            after(BOSS_FADES_AFTER, function(d3) d3:fade_out_bnpc(BNPC_BOSS) end)
            after(TURNED_AFTER, function(d3) bringOut(d3, { BNPC_TURNED }, boss_at) end)
            after(SEQUENCE_6_AFTER, function(d3)
                BATTLE:remove(d3, { BNPC_BOSS })
                sequence(d3, SEQUENCE_TURNED)
                after(TALK_80_AFTER, function(d4) talk(d4, SPEAKER_TURNED, 80) end)
                after(TALK_81_AFTER, function(d4) talk(d4, SPEAKER_GLADIATOR, 81) end)
            end)
        end)
    end)
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
    local turned = health[BNPC_TURNED]
    if info ~= nil and info[BNPC_BOSS] ~= nil and info[BNPC_BOSS].x ~= nil then
        boss_at = { x = info[BNPC_BOSS].x, y = info[BNPC_BOSS].y, z = info[BNPC_BOSS].z }
    end
    if phase == SEQUENCE_ADDS and boss ~= nil and boss < BOSS_HALF then
        bossHalf(director)
    elseif phase == SEQUENCE_TURNED and turned ~= nil and turned < TURNED_WYVERN then
        sequence(director, SEQUENCE_WYVERN)
        after(WYVERN_AFTER, function(d) bringOut(d, { BNPC_WYVERN }) end)
        after(TALK_82_AFTER, function(d) talk(d, SPEAKER_TURNED, 82) end)
        after(TALK_83_AFTER, function(d) talk(d, SPEAKER_GLADIATOR, 83) end)
    elseif phase == SEQUENCE_WYVERN and turned ~= nil and turned < TURNED_LAST_ADD then
        sequence(director, SEQUENCE_LAST_ADD)
        after(LAST_ADD_AFTER, function(d) bringOut(d, { BNPC_LAST_ADD }) end)
    end
    if phase >= SEQUENCE_LAST_ADD and not turned_low and turned ~= nil and turned < TURNED_LOW then
        turned_low = true
        talk(director, SPEAKER_TURNED, 84)
    end
    BATTLE:update(director, health, player_health, info)
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    if layout_id == BNPC_ARCHER then
        talk(director, SPEAKER_ARCHER_FALLS, 87)
    elseif layout_id == BNPC_CONJURER then
        talk(director, SPEAKER_CONJURER_FALLS, 86)
    elseif layout_id == BNPC_GLADIATOR then
        talk(director, SPEAKER_GLADIATOR, 85)
        lost(director)
        return
    end
    if LAST_ONES[layout_id] then
        left[layout_id] = nil
        if phase >= SEQUENCE_LAST_ADD and next(left) == nil then
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
