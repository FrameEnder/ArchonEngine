-- Quest battle: Lord of the Inferno's, for the quest of the same name (ManFst209_00343.)
-- It takes place in territory 275, a copy of Eastern Thanalan (retail's positions in it stand on Eastern Thanalan's
-- ground, by its open-world zombies), so its NPCs are layer QB_ManFst209_001 of the wil_w1f3 drop-in: four Immortal
-- Flames on your side (the lead gladiator 4098749, BNpcBase 963; a second gladiator 4098743, 964; a pugilist
-- 4098744, 965; a thaumaturge who heals, 4098745, 966), the Amalj'aa leader (4098741, 967, a thaumaturge, 11360 HP),
-- three waves of lancers (968), soldiers (969) and javeliniers (970), and the scout who talks (an event NPC,
-- 4100860). Layout IDs, levels, HP, positions, looks, display flags and the start spot are retail's (Bob's Chronofoil
-- capture b8432613 of 2026-10-27, `questkit chronofoil --battle 0.8`, `0.10` and `0.12`, lost, and `0.14`, won; a
-- level 24 conjurer.) The client's battle script (ManFst209Btl_00343) has the opening cutscene (scene 1), the closing
-- one (scene 7) and the "won't talk" message (10001.)
--
-- Timed against the four runs (seconds from the fight starting, the director's scene 60000):
--   0     sequence 2: everyone there, the first wave each on its own ally (the lancers and two soldiers on the
--         pugilist, two soldiers and the leader on the lead gladiator)
--   5, 10 the scout's battle talk 202 (ENpcResident 1004668, portrait 73225), the lead's 203 (BNpcName 993, 73226)
--   five of the first wave's nine left (four down, in all four runs): sequence 3
--   30 s after that (30.0-30.1 s, three runs): sequence 4, the second wave (four soldiers on the thaumaturge, a
--         javelinier on you); the scout's 204 at once, the lead's 205 10 s later
--   two left (the won run: not at three, 8 s before; the lost ones never got there): sequence 5, the third wave (three lancers and three
--         soldiers on the second gladiator, a javelinier on you); the lead's 206 10 s later, the scout's 207 15 s
--   55 s after sequence 5 (55.1 s, the won run, with six enemies still standing and the leader at 84 %): sequence 7
--         and the battle's scene 7 (a cutscene, 0xF8402EFB); the second gladiator's last words (210); the quest moves
--         to sequence 3
--   an ally falling: its last words (the pugilist 209, BNpcName 1007, portrait 73229; the thaumaturge 208, 1005,
--         73227); the battle goes on without it (two fell in two of the lost runs)
--   you down: sequence 0xFE (all three losses)
-- The allies keep to their own: the lead on the leader, the others on the adds, the thaumaturge healing; Cure (120)
-- on whoever of the side is lowest below 40 % (retail's medians 31-41 %, about every 6 s), the pugilist's Second
-- Wind (57) on itself below 35 %. How hard each hits is in their timelines (QB45*).

-- Where retail puts the player and which way they face: their SpawnPlayer in the capture.
START_POSITION = { x = -334.77, y = -12.09, z = -238.7 }
START_ROTATION = 2.229
-- Retail's opening cutscene's flags here (all four runs.)
OPENING_SCENE_FLAGS = 0xF841AEFB
-- Won, retail sent the player on to Southern Thanalan (146), by the flame sergeant the quest goes to next, not back
-- to where they entered (the capture's next visit: ZoneInit pop range 4104257, the player at this spot.) Lost, back where
-- they came from.
EXIT_AFTER_WIN = { territory = 146, x = -56.7, y = -24.57, z = -565.88, rotation = -0.468 }

BNPC_LEAD = 4098749
BNPC_GLADIATOR = 4098743
BNPC_PUGILIST = 4098744
BNPC_THAUMATURGE = 4098745
BNPC_LEADER = 4098741
BNPC_SCOUT = 4100860
ALLIES = { BNPC_LEAD, BNPC_GLADIATOR, BNPC_PUGILIST, BNPC_THAUMATURGE }

-- Each wave: { [enemy] = whom it goes for (nil: you) }.
FIRST_WAVE = {
    [4098775] = BNPC_PUGILIST, [4098776] = BNPC_PUGILIST, [4098777] = BNPC_PUGILIST, [4098783] = BNPC_PUGILIST,
    [4098784] = BNPC_PUGILIST, [4098791] = BNPC_PUGILIST, [4098792] = BNPC_PUGILIST,
    [4098866] = BNPC_LEAD, [4098871] = BNPC_LEAD,
}
SECOND_WAVE = {
    [4098810] = BNPC_THAUMATURGE, [4098819] = BNPC_THAUMATURGE, [4098821] = BNPC_THAUMATURGE,
    [4098823] = BNPC_THAUMATURGE, [4098806] = false,
}
THIRD_WAVE = {
    [4098887] = BNPC_GLADIATOR, [4098888] = BNPC_GLADIATOR, [4098889] = BNPC_GLADIATOR,
    [4098839] = BNPC_GLADIATOR, [4098840] = BNPC_GLADIATOR, [4098842] = BNPC_GLADIATOR, [4098886] = false,
}

-- The quest moves to this sequence when the battle is won.
QUEST_SEQUENCE_AFTER_WIN = 3
SEQUENCE_FIGHTING = 2
SEQUENCE_WON = 7
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 7
FINISH_SCENE_FLAGS = 0xF8402EFB
FAIL_DELAY_UPDATES = 12

-- When things move on: enemies left (the adds), and the timers, in updates (half a second each.)
SEQUENCE_3_AT = 5
SEQUENCE_4_AFTER = 60
SEQUENCE_5_AT = 2
WON_AFTER = 110
-- Sequences 3 and 5 came 2 s after the enemy's health reached 0 (2.0 and 1.7 s, the won run), as its death animation
-- ended.
DEATH_COUNTS_AFTER = 4

-- Who's talking (DirectorPopupMessage): the scout by its ENpcResident (an event NPC: kind 3), the Flames by their
-- BNpcName, each with retail's portrait. The content's text IDs, as retail sent them.
SPEAKER_SCOUT = { layout = BNPC_SCOUT, name = 1004668, icon = 73225, kind = 3 }
SPEAKER_LEAD = { layout = BNPC_LEAD, name = 993, icon = 73226 }
SPEAKER_GLADIATOR = { layout = BNPC_GLADIATOR, name = 994, icon = 73228 }
SPEAKERS = {
    [BNPC_PUGILIST] = { layout = BNPC_PUGILIST, name = 1007, icon = 73229 },
    [BNPC_THAUMATURGE] = { layout = BNPC_THAUMATURGE, name = 1005, icon = 73227 },
}
LAST_WORDS = { [BNPC_PUGILIST] = 209, [BNPC_THAUMATURGE] = 208 }

BATTLE = Battle.new()
ACTION_CURE = 120
ACTION_SECOND_WIND = 57
HEAL_BELOW = 40
HEAL_EVERY_SECONDS = 6
BATTLE:ally(BNPC_LEAD, {
    role = "dps",
    split = false,
    focus = { "boss", "adds" },
    range = 3,
    heals = { { action = ACTION_CURE, targets = "allies", below = HEAL_BELOW, every = HEAL_EVERY_SECONDS } },
})
BATTLE:ally(BNPC_GLADIATOR, {
    role = "dps",
    split = false,
    focus = { "adds", "boss" },
    order = "nearest",
    range = 3,
    heals = { { action = ACTION_CURE, targets = "allies", below = HEAL_BELOW, every = HEAL_EVERY_SECONDS } },
})
BATTLE:ally(BNPC_PUGILIST, {
    role = "dps",
    split = false,
    focus = { "adds", "boss" },
    order = "nearest",
    range = 3,
    heals = { { action = ACTION_SECOND_WIND, targets = { "self" }, below = 35, every = 120 } },
})
BATTLE:ally(BNPC_THAUMATURGE, {
    role = "support",
    split = false,
    focus = { "adds", "boss" },
    order = "nearest",
    range = 25,
    heals = { { action = ACTION_CURE, targets = "allies", below = HEAL_BELOW, every = HEAL_EVERY_SECONDS } },
})
BATTLE:enemy(BNPC_LEADER, { group = "boss", attacks = BNPC_LEAD })

local over = false
local fail_in = nil
local phase = 1
-- Timers of their own, side by side (Battle:steps runs one queue, one step after another): { updates left, fn }.
local timers = {}

function after(updates, fn)
    table.insert(timers, { updates, fn })
end

function runTimers(director)
    -- Timers a firing one starts wait from the next update.
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

-- A wave's enemies, each after its ally (or you), described and spawned.
function arrive(director, wave)
    local ids = {}
    for enemy, target in pairs(wave) do
        BATTLE:enemy(enemy, { group = "adds", attacks = target or nil })
        table.insert(ids, enemy)
    end
    table.sort(ids)
    BATTLE:spawn(director, ids)
end

function addsLeft()
    return #BATTLE:living("adds")
end

function onSetup(director)
    director:set_sequence(1)
end

function onCommence(director)
    director:set_sequence(SEQUENCE_FIGHTING)
    director:spawn_bnpc(BNPC_SCOUT)
    BATTLE:spawn(director, ALLIES)
    BATTLE:spawn(director, { BNPC_LEADER })
    arrive(director, FIRST_WAVE)
    after(10, function(d) talk(d, SPEAKER_SCOUT, 202) end)
    after(20, function(d) talk(d, SPEAKER_LEAD, 203) end)
end

function secondWave(director)
    phase = 3
    director:set_sequence(4)
    arrive(director, SECOND_WAVE)
    talk(director, SPEAKER_SCOUT, 204)
    after(20, function(d) talk(d, SPEAKER_LEAD, 205) end)
end

function thirdWave(director)
    phase = 4
    director:set_sequence(5)
    arrive(director, THIRD_WAVE)
    after(20, function(d) talk(d, SPEAKER_LEAD, 206) end)
    after(30, function(d) talk(d, SPEAKER_SCOUT, 207) end)
    after(WON_AFTER, function(d) won(d) end)
end

function won(director)
    if over then
        return
    end
    over = true
    BATTLE:clearSteps()
    timers = {}
    director:set_sequence(SEQUENCE_WON)
    talk(director, SPEAKER_GLADIATOR, 210)
    -- No "Duty Complete" on retail: the cutscene at once.
    director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS, nil, false)
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
    BATTLE:update(director, health, player_health, info)
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    if LAST_WORDS[layout_id] ~= nil then
        talk(director, SPEAKERS[layout_id], LAST_WORDS[layout_id])
    end
    after(DEATH_COUNTS_AFTER, function(d) counted(d) end)
end

-- The waves move on by how many adds are left, once a death has played out.
function counted(director)
    if phase == 1 and addsLeft() <= SEQUENCE_3_AT then
        phase = 2
        director:set_sequence(3)
        after(SEQUENCE_4_AFTER, function(d) secondWave(d) end)
    elseif phase == 3 and addsLeft() <= SEQUENCE_5_AT then
        thirdWave(director)
    end
end

-- Losing sends you back to where you entered, still on the same step of the quest, so you can try again.
function onPlayerDeath(director)
    if over then
        return
    end
    over = true
    BATTLE:clearSteps()
    timers = {}
    director:set_sequence(SEQUENCE_FAILED)
    fail_in = FAIL_DELAY_UPDATES
end
