-- Quest battle 64 (GaiUsc605_01005, Every Little Thing She Does Is Magitek, the main scenario's level 46 quest in Mor
-- Dhona.)
-- It takes place in territory 309, taken to be a copy of Mor Dhona (l1f1) as 63's: its spots are within 50 yalms of
-- quest battle 63's, which stand beside where retail let the player out in Mor Dhona, so its NPCs are layer
-- QB_GaiUsc605_001 of lak_l1f1. Layout IDs, levels, HP, positions,
-- display and character data flags, looks and the start spot are retail's (Connor's Chronofoil capture e834e436 of
-- 2026-11-07, `questkit chronofoil --battle 9.202`: one run, won; a level 48 player.) On your side: an ally who fights
-- (4277164, base 1876) and one the enemies wear down (4277157, base 1877, 11193 HP). Against you: the boss (4277127,
-- base 1958, 5450 HP) and three others (4277134-4277136, bases 1873-1875), all out from the start. Two event NPCs talk
-- (1007075 and 1007076, as EventNpcs: kind 3, no actor.)
--
-- Retail's flow (seconds from the fight starting, the director's scene 60000):
--   0       sequence 2; battle talks 122 (4277136) and 123 (the ally who fights) at 5.1 and 10.1
--   the worn-down ally under 90 % (89.6 %): talk 124 (1007075)
--   4277136's health at 0: sequence 3 at once
--   another down after it: sequence 4 at once, talk 128 (1007076) 5.0 s later. Connor's run had the boss next, Bob's
--           Chronofoil run of 2026-11-12 (`64-da424662.json`) 4277135, with the boss last.
--   the boss's health at 0 with others still up: talk 125 (1007075) 1.2 s later (not in Bob's run, where he was last
--           and the win came 1.4 s after him)
--   all four down: "Duty Complete" (60001) and sequence 5 2.1 s after the last one's health reached 0, the battle's
--           scene 5 (0xF8402EFB) 3.2 s after that; the quest moves to sequence 255
--   lost (no lost run recorded): your health at 0, sequence 0xFE and out 5 s later
-- The opening scene 1 is a cutscene (0xF841AEFB). Who fights whom and how hard is in their timelines (QB64*.)

-- Where retail puts the player and which way they face: their spawn in the run.
START_POSITION = { x = -78.43, y = 1.89, z = -627.73 }
START_ROTATION = -1.159
OPENING_SCENE_FLAGS = 0xF841AEFB

BNPC_FIGHTER = 4277164
BNPC_WORN = 4277157
BNPC_BOSS = 4277127
BNPC_FIRST = 4277136
ENEMIES = { BNPC_BOSS, 4277134, 4277135, BNPC_FIRST }

QUEST_SEQUENCE_AFTER_WIN = 255
SEQUENCE_FIGHTING = 2
SEQUENCE_FIRST_DOWN = 3
SEQUENCE_SECOND_DOWN = 4
SEQUENCE_WON = 5
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 5
FINISH_SCENE_FLAGS = 0xF8402EFB
FAIL_DELAY_UPDATES = 10

-- The worn-down ally's health mark (percent) for talk 124.
WORN_TALK = 90

-- Timers, in updates (half a second each). A timer of n fires n / 2 s after it's set by another timer or in onUpdate
-- (after that update's timers have run), and (n - 1) / 2 s after it's set before the next update (onCommence,
-- onActorDeath.)
TALK_122_AFTER = 11
TALK_123_AFTER = 21
TALK_125_AFTER = 3
TALK_128_AFTER = 11
-- The win came 2.1 s after the last one's health reached 0 (its death animation.)
WON_AFTER_DEATH = 5

-- Who's talking (DirectorPopupMessage, by whom it came from), with retail's names and portraits. The two event NPCs
-- aren't battle NPCs: kind 3 and no actor, as Victory in Peril's watcher.
SPEAKER_FIRST = { layout = BNPC_FIRST, name = 1812, icon = 73183 }
SPEAKER_FIGHTER = { layout = BNPC_FIGHTER, name = 1486, icon = 73184 }
SPEAKER_ENPC_A = { layout = 0, name = 1007075, icon = 73181, kind = 3 }
SPEAKER_ENPC_B = { layout = 0, name = 1007076, icon = 73182, kind = 3 }

ACTION_SELF_HEAL = 57

BATTLE = Battle.new()
-- On retail the one who fights took 4277136 first, then the boss, then the other two; the worn-down one held the boss,
-- then 4277136, then 4277135, without being seen to act.
BATTLE:ally(BNPC_FIGHTER, { role = "dps", focus = { "first", "boss", "adds" }, order = "oldest" })
BATTLE:ally(BNPC_WORN, { role = "tank", focus = { "boss", "first", "adds" }, order = "oldest", protect = false })
-- Whom each went for first on retail (nil: you.)
BATTLE:enemy(BNPC_BOSS, { group = "boss", attacks = nil })
BATTLE:enemy(BNPC_FIRST, {
    group = "first",
    attacks = BNPC_FIGHTER,
    -- It cured itself at 6 % (805, once.)
    heals = { { action = ACTION_SELF_HEAL, targets = { "self" }, below = 10, every = 30.0 } },
})
BATTLE:enemy(4277134, { group = "adds", attacks = BNPC_FIGHTER })
BATTLE:enemy(4277135, { group = "adds", attacks = BNPC_WORN })

local over = false
local fail_in = nil
local phase = SEQUENCE_FIGHTING
local left = {}
local worn_talked = false
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

function onSetup(director)
    director:set_sequence(1)
    for _, layout in ipairs(ENEMIES) do
        left[layout] = true
    end
    BATTLE:spawn(director, { BNPC_FIGHTER, BNPC_WORN })
    BATTLE:spawn(director, ENEMIES)
end

function onCommence(director)
    director:set_sequence(SEQUENCE_FIGHTING)
    after(TALK_122_AFTER, function(d) talk(d, SPEAKER_FIRST, 122) end)
    after(TALK_123_AFTER, function(d) talk(d, SPEAKER_FIGHTER, 123) end)
end

function won(director)
    over = true
    timers = {}
    director:set_sequence(SEQUENCE_WON)
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
    local worn = health[BNPC_WORN]
    if not worn_talked and worn ~= nil and worn < WORN_TALK then
        worn_talked = true
        talk(director, SPEAKER_ENPC_A, 124)
    end
    BATTLE:update(director, health, player_health, info)
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    -- 4277136 first (both runs), then whoever's next; the sequence only goes forwards.
    if layout_id == BNPC_FIRST and phase < SEQUENCE_FIRST_DOWN then
        phase = SEQUENCE_FIRST_DOWN
        director:set_sequence(SEQUENCE_FIRST_DOWN)
    elseif phase == SEQUENCE_FIRST_DOWN and left[layout_id] then
        phase = SEQUENCE_SECOND_DOWN
        director:set_sequence(SEQUENCE_SECOND_DOWN)
        after(TALK_128_AFTER, function(d) talk(d, SPEAKER_ENPC_B, 128) end)
    end
    if left[layout_id] then
        left[layout_id] = nil
        if next(left) == nil then
            after(WON_AFTER_DEATH, won)
        elseif layout_id == BNPC_BOSS then
            after(TALK_125_AFTER, function(d) talk(d, SPEAKER_ENPC_A, 125) end)
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
