-- Quest battle 76 (JobWhm500_01084, Heart of the Forest, the White Mage's level 50 quest.)
-- It takes place in territory 320, taken to be a copy of the Central Shroud (f1f1): its spots stand where the quest's
-- EOBJECT0 does there, 20 yalms from To Guard a Guardian's, so its NPCs are layer QB_JobWhm500_001 of fst_f1. Layout
-- IDs, levels, HP, positions, display and character data flags, looks and the start spot are retail's (Bob's
-- Chronofoil capture 667a68c5 of 2026-11-16, `questkit chronofoil --battle 0.20`, `0.22` and `0.24`: three runs, two
-- lost and one won; a level 50 White Mage.)
--
-- On your side: the one who prays (4290091, base 1702, 50688 HP), who never moves or acts and loses health all through
-- to status 324 (a tick every 3 s: 1763-1940 from the start, 2186-2413 from the second wave on), and three white
-- mages who guard them (4290072, 4290082, 4290083; bases 1708, 1706, 1707; 15145 HP), who Cure them and fight whoever
-- comes. Lost when the one who prays falls (both lost runs), or you.
--
-- Retail's flow (seconds from the fight starting, the director's scene 60000):
--   0       sequence 2; status 324 on the one who prays 0.1 s in; battle talks 107 and 108 (4290072) at 5.0 and 10.0.
--           The first wave is out from the start: three of base 1705 and three of 1701, one of each on each guard.
--   the first wave down: sequence 3 at once; 20.05 s later sequence 4, status 324 taken off and given again (the
--           larger tick), and the second wave 0.2-0.4 s after (three of base 1703); talk 109 (4290082) 5.0 s after 4.
--   the second wave down: sequence 5 at once; 20.05 s later sequence 6 and the third wave 0.3-0.5 s after (three of
--           base 1701); talk 110 (4290072) 5.05 s after 6, NpcYell 1173 (4290082) 10.07 s after. From 8.3 s after 6
--           one of base 1704 every 8.05 s, each at one of its spots (nine seen, two at once at the same one in the
--           won run), after you: it walks up and bursts on reaching you (1341.) Seven came in the won run (the last
--           56.6 s after 6), five in the lost one before it ended.
--   60.0 s after 6: sequence 7 (the won run; no more of 1704 came.)
--   everyone down after that: sequence 8 at once (0.05 s after the last one's health reached 0); 20.08 s later the
--           battle's scene 9 (0xF8402EFB) with sequence 9 and status 324 off, no "Duty Complete" before it; as it
--           ends the quest's LOGMESSAGE0 (3656), the quest to sequence 255, and out.
--   the one who prays under 35 % (34.2-37.6 % in the three runs' samples): talk 114 (4290083), once.
--   a guard under 20 % (2846-2853 of 15145 left by the hit that did it): it's down. ActorControl 32 [1] and 31 [1]
--           from it, untargetable (54 [0]), no more hits on it and nothing more from it, and its line: 112 for
--           4290083, 111 for 4290082, 113 for 4290072 (the second lost run, all three within 18 s.)
--   lost: sequence 0xFE as the one who prays reaches 0 (its death 2 s later); out about 6 s on.
-- The opening scene 1 is a cutscene (0xF841AEFB). Who fights how hard is in their timelines (QB76*.)
--
-- Not seen on retail, so guessed: everyone down before the minute is up (sequence 8 waits for 7 here); what a guard
-- that's down does when the battle is won; the spots of 1704 that no run used (4290121 and 4290123 are known of, not
-- where they stand.) Not played: the guards' Stoneskin (which kept the one who prays' ticks off for 6 s at a time),
-- Regen, Divine Seal and Holy; 1704's slow walk (about 1.9 yalms a second.)

-- Where retail puts the player and which way they face: their spawn in the runs.
START_POSITION = { x = -318.23, y = 21.2, z = 173.09 }
START_ROTATION = -0.772
OPENING_SCENE_FLAGS = 0xF841AEFB

BNPC_PRAYER = 4290091
GUARD_A = 4290072 -- base 1708, BNpcName 1452
GUARD_B = 4290082 -- base 1706, BNpcName 1422
GUARD_C = 4290083 -- base 1707, BNpcName 1423
GUARDS = { GUARD_A, GUARD_B, GUARD_C }

-- Whom each went for first on retail (all three runs.)
WAVE_1 = {
    { 4290111, GUARD_B }, { 4290109, GUARD_A }, { 4290106, GUARD_C },
    { 4290095, GUARD_B }, { 4290094, GUARD_C }, { 4290093, GUARD_A },
}
WAVE_2 = { { 4292767, GUARD_B }, { 4292766, GUARD_A }, { 4292765, GUARD_C } }
WAVE_3 = { { 4292908, GUARD_B }, { 4292907, GUARD_C }, { 4292906, GUARD_A } }
-- Where the ones of base 1704 came out in the three runs (4290120, 4290127 and 4290137 twice.)
BOMB_SPOTS = { 4290117, 4290118, 4290120, 4290122, 4290126, 4290127, 4290130, 4290132, 4290137 }

QUEST_SEQUENCE_AFTER_WIN = 255
-- Heart of the Forest's LOGMESSAGE0, the quest's line as the battle's closing scene ends.
QUEST_LOG = 3656
SEQUENCE_FIGHTING = 2
SEQUENCE_FIRST_DOWN = 3
SEQUENCE_SECOND_WAVE = 4
SEQUENCE_SECOND_DOWN = 5
SEQUENCE_THIRD_WAVE = 6
SEQUENCE_NO_MORE = 7
SEQUENCE_ALL_DOWN = 8
SEQUENCE_WON = 9
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 9
FINISH_SCENE_FLAGS = 0xF8402EFB
FAIL_DELAY_UPDATES = 12

-- The one who prays' status and its tick (retail's ranges' middles: 1763-1940, 2186-2413; each tick ±5 %.)
STATUS_PRAYING = 324
TICK_FIRST = 1850
TICK_LATER = 2300
-- The marks (percent of health): the one who prays' for talk 114, a guard's for going down.
PRAYER_TALK = 35
GUARD_DOWN = 20
CONTROL_DOWN_1 = 32
CONTROL_DOWN_2 = 31

-- Timers, in updates (half a second each). A timer of n fires n / 2 s after it's set by another timer or in onUpdate
-- (after that update's timers have run), and (n - 1) / 2 s after it's set before the next update (onCommence,
-- onActorDeath.)
TALK_107_AFTER = 11
TALK_108_AFTER = 21
NEXT_WAVE_AFTER = 41
WAVE_TALK_AFTER = 10
YELL_AFTER = 20
BOMB_FIRST_AFTER = 16
BOMB_EVERY = 16
NO_MORE_AFTER = 120
WON_AFTER = 40
-- 1704 bursts within this of the player, and dies 0.8 s later (retail's Kill after its 1341); the script counts it
-- gone two updates on, should the server not have said so by then.
BOMB_REACH = 3.5
BOMB_DIES_AFTER = 0.8
BOMB_GONE_AFTER = 2
BURSTING = 2
ACTION_BURST = 1341
YELL_THIRD_WAVE = 1173

SPEAKER_A = { layout = GUARD_A, name = 1452, icon = 0 }
SPEAKER_B = { layout = GUARD_B, name = 1422, icon = 0 }
SPEAKER_C = { layout = GUARD_C, name = 1423, icon = 0 }
DOWN_TALK = { [GUARD_A] = { SPEAKER_A, 113 }, [GUARD_B] = { SPEAKER_B, 111 }, [GUARD_C] = { SPEAKER_C, 112 } }

ACTION_CURE = 120
ACTION_CURE_II = 135

BATTLE = Battle.new()
BATTLE:ally(BNPC_PRAYER, { still = true })
-- Each guard Cures the one who prays every 8-10 s and Cure IIs them about every 20 s, but only once they're below
-- about 65 %: in the three runs no guard healed them above 64 % (the first Cure came 30.6 to 33.4 s in, at 60 to
-- 63 %, the first Cure II at 58 to 61 %; their health rebuilt from every tick, heal and hit, since the samples of it
-- are 6 s apart), so retail's are never kept topped up and drop a third in the first half-minute. The script Cured
-- from 100 % (the first write-up read "at 19-100 %" off the sparse samples), which held them above 46 % all through
-- Bob's run of 2026-11-18 with 8,300 of his own healing, where retail's lost run needed 26,000 and still fell ("the
-- balance of this battle seemed a tad different then in retail".) Between heals they also Stoneskin (129) one of
-- themselves or the player about every 45 s each (15 uses in the 240 s run, the lowest of them here) and Regen
-- (137) a hurt one about every two minutes, which takes those casts from the one who prays as retail's did. Divine Seal
-- (138) and Holy (139) aren't played.
ACTION_STONESKIN = 129
ACTION_REGEN = 137
HEAL_PRAYER_BELOW = 65
for _, guard in ipairs(GUARDS) do
    BATTLE:ally(guard, {
        role = "dps",
        split = false,
        focus = { "mine", "any" },
        order = "oldest",
        range = 15,
        heals = {
            { action = ACTION_STONESKIN, targets = { GUARD_A, GUARD_B, GUARD_C, "player" }, below = 101, every = 45 },
            { action = ACTION_REGEN, targets = { GUARD_A, GUARD_B, GUARD_C, "player" }, below = 90, every = 120 },
            { action = ACTION_CURE_II, targets = { BNPC_PRAYER }, below = HEAL_PRAYER_BELOW, every = 20 },
            { action = ACTION_CURE, targets = { BNPC_PRAYER }, below = HEAL_PRAYER_BELOW, every = 9 },
        },
    })
end
for _, wave in ipairs({ WAVE_1, WAVE_2, WAVE_3 }) do
    for _, enemy in ipairs(wave) do
        BATTLE:enemy(enemy[1], { group = "adds", attacks = enemy[2] })
    end
end

local over = false
local fail_in = nil
local phase = 1
-- The enemies out and not yet down, by layout ID (how many, for 1704's spots.)
local standing = {}
local prayer_talked = false
local down = {}
local timers = {}
-- How many of 1704 have come out, to tell one at a spot from the next there.
local bombs = 0
local bomb_at = {}

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

function bringOut(director, wave)
    local layouts = {}
    for _, enemy in ipairs(wave) do
        standing[enemy[1]] = (standing[enemy[1]] or 0) + 1
        table.insert(layouts, enemy[1])
    end
    BATTLE:spawn(director, layouts)
end

function anyStanding()
    for _, count in pairs(standing) do
        if count > 0 then
            return true
        end
    end
    return false
end

function pray(director, tick)
    director:lose_effect(BNPC_PRAYER, STATUS_PRAYING)
    director:gain_effect(BNPC_PRAYER, STATUS_PRAYING, 0, 0, tick)
end

-- One of base 1704 at a spot none of them stands at (the script tells them apart by layout ID), after the player.
function bringBomb(director)
    if over or phase ~= SEQUENCE_THIRD_WAVE then
        return
    end
    local free = {}
    for _, spot in ipairs(BOMB_SPOTS) do
        if (standing[spot] or 0) == 0 then
            table.insert(free, spot)
        end
    end
    if #free > 0 then
        local spot = free[math.random(#free)]
        standing[spot] = 1
        bombs = bombs + 1
        bomb_at[spot] = bombs
        local bomb = director:spawn_bnpc(spot)
        director:set_hate(bomb, nil, Battle.STARTING_ENMITY)
        director:root_bnpc(bomb, Battle.ARRIVAL_STAND_SECONDS)
    end
    after(BOMB_EVERY, bringBomb)
end

function nextWave(director)
    if over then
        return
    end
    if phase == SEQUENCE_FIRST_DOWN then
        phase = SEQUENCE_SECOND_WAVE
        director:set_sequence(SEQUENCE_SECOND_WAVE)
        pray(director, TICK_LATER)
        bringOut(director, WAVE_2)
        after(WAVE_TALK_AFTER, function(d) talk(d, SPEAKER_B, 109) end)
    elseif phase == SEQUENCE_SECOND_DOWN then
        phase = SEQUENCE_THIRD_WAVE
        director:set_sequence(SEQUENCE_THIRD_WAVE)
        bringOut(director, WAVE_3)
        after(WAVE_TALK_AFTER, function(d) talk(d, SPEAKER_A, 110) end)
        after(YELL_AFTER, function(d) d:yell(GUARD_B, YELL_THIRD_WAVE) end)
        after(BOMB_FIRST_AFTER, bringBomb)
        after(NO_MORE_AFTER, function(d)
            if over then
                return
            end
            phase = SEQUENCE_NO_MORE
            d:set_sequence(SEQUENCE_NO_MORE)
            allDown(d)
        end)
    end
end

function won(director)
    if over then
        return
    end
    over = true
    timers = {}
    director:lose_effect(BNPC_PRAYER, STATUS_PRAYING)
    director:set_sequence(SEQUENCE_WON)
    director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS, nil, false,
        QUEST_LOG)
end

-- A wave's last one down: the next step.
function allDown(director)
    if over or anyStanding() then
        return
    end
    if phase == SEQUENCE_FIGHTING then
        phase = SEQUENCE_FIRST_DOWN
        director:set_sequence(SEQUENCE_FIRST_DOWN)
        after(NEXT_WAVE_AFTER, nextWave)
    elseif phase == SEQUENCE_SECOND_WAVE then
        phase = SEQUENCE_SECOND_DOWN
        director:set_sequence(SEQUENCE_SECOND_DOWN)
        after(NEXT_WAVE_AFTER, nextWave)
    elseif phase == SEQUENCE_NO_MORE then
        phase = SEQUENCE_ALL_DOWN
        director:set_sequence(SEQUENCE_ALL_DOWN)
        after(WON_AFTER, won)
    end
end

function onSetup(director)
    director:set_sequence(1)
    BATTLE:spawn(director, { BNPC_PRAYER, GUARD_A, GUARD_B, GUARD_C })
    bringOut(director, WAVE_1)
end

function onCommence(director)
    phase = SEQUENCE_FIGHTING
    director:set_sequence(SEQUENCE_FIGHTING)
    director:gain_effect(BNPC_PRAYER, STATUS_PRAYING, 0, 0, TICK_FIRST)
    after(TALK_107_AFTER, function(d) talk(d, SPEAKER_A, 107) end)
    after(TALK_108_AFTER, function(d) talk(d, SPEAKER_A, 108) end)
end

function lost(director)
    over = true
    timers = {}
    director:set_sequence(SEQUENCE_FAILED)
    fail_in = FAIL_DELAY_UPDATES
end

function near(a, b, yalms)
    if a == nil or b == nil then
        return false
    end
    local dx, dz = a.x - b.x, a.z - b.z
    return dx * dx + dz * dz <= yalms * yalms
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
    local prayer = health[BNPC_PRAYER]
    if not prayer_talked and prayer ~= nil and prayer < PRAYER_TALK then
        prayer_talked = true
        talk(director, SPEAKER_C, 114)
    end
    -- A guard under its mark is down: out of the fight where it stands, and says so.
    for _, guard in ipairs(GUARDS) do
        local left = health[guard]
        if not down[guard] and left ~= nil and left > 0 and left < GUARD_DOWN then
            down[guard] = true
            BATTLE:died(director, guard)
            director:hold_bnpc(guard)
            director:set_invulnerable(guard, true)
            director:bnpc_control(guard, CONTROL_DOWN_1, { 1 })
            director:bnpc_control(guard, CONTROL_DOWN_2, { 1 })
            director:set_targetable(guard, false)
            talk(director, DOWN_TALK[guard][1], DOWN_TALK[guard][2])
        end
    end
    -- One of 1704 that has reached the player bursts, and dies of it.
    if info ~= nil then
        for _, spot in ipairs(BOMB_SPOTS) do
            if standing[spot] == 1 and near(info[spot], info.player, BOMB_REACH) then
                standing[spot] = BURSTING
                director:use_action(spot, ACTION_BURST)
                director:delay(BOMB_DIES_AFTER)
                director:kill_bnpc(spot)
                director:delay(0)
                local which = bomb_at[spot]
                after(BOMB_GONE_AFTER, function(d)
                    if standing[spot] == BURSTING and bomb_at[spot] == which then
                        standing[spot] = 0
                        allDown(d)
                    end
                end)
            end
        end
    end
    BATTLE:update(director, health, player_health, info)
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    if layout_id == BNPC_PRAYER then
        lost(director)
        return
    end
    BATTLE:died(director, layout_id)
    if (standing[layout_id] or 0) > 0 then
        standing[layout_id] = 0
        allDown(director)
    end
end

-- Losing sends you back to where you entered, still on the same step of the quest, so you can try again.
function onPlayerDeath(director)
    if over then
        return
    end
    lost(director)
end
