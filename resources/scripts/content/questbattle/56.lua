-- Quest battle: Like Mother, Like Daughter, for the conjurer quest of the same name (ClsCnj005_00440.)
-- It takes place in territory 291, a copy of the South Shroud (retail's positions in it are the South Shroud's), so its
-- NPCs live in the fst_f1f3 drop-in (layer QB_ClsCnj005_001): the thing at the middle (4250414, BNpcBase 1682, level
-- 25, 2138 HP), three conjurers on your side (Wulfiue 4249072, Nolanel 4249073, Joacin 4249074; bases 1688-1690,
-- levels 25-27, with retail's looks), Sylphie standing by (an event NPC, 4249177), the waves (bases 1683-1687) and
-- the elementals that come to help (4249117, 4249116, 4249119; bases 1691-1693). Layout IDs, levels, HP, positions,
-- display flags and the start spot are retail's (Bob's Chronofoil capture b23b91ef of 2026-10-26, `questkit
-- chronofoil --battle 0.29`, lost doing nothing, and `0.31`, won; a level 26 conjurer.)
--
-- Timed against both runs (seconds from the fight starting, the director's scene 60000):
--   0     sequence 2: the target, the conjurers and three 1683s, each on its own conjurer
--   5, 10 Sylphie's battle talk 103 and 104 (ENpcResident 1006738, no portrait; she's an event NPC)
--   15, 20, 25  the conjurers' yells (NpcYell 996 Wulfiue, 997 Nolanel, 998 Joacin)
--   the target at 80 % (84 -> 78 in the won run, 37.9 s in) or 40 s in (the lost run, it untouched): sequence 3,
--         a 1685 (on Nolanel) and a 1684 (on Joacin); Sylphie's 105 5 s later and 106 15 s later
--   the target at 65 % after that (69 -> 62, 43.2 s in): sequence 4, the elementals and a second 1684 (on Wulfiue);
--         Sylphie's 107 10 s later; sequence 5 6 s after 4, sequence 6 15 s after 5; then a 1687 5.3 s later and
--         Sylphie's 108 10 s later. Without it (the lost run), a 1686 came 35 s after sequence 3, and again 35 s on
--   each conjurer's last words as he falls: battle talk 110 (Wulfiue, BNpcName 1419), 109 (Nolanel, 1425), 111
--         (Joacin, 1426); the battle goes on without them
--   the target down: sequence 7 and "Duty Complete" at once, then the battle's scene 7 (0x2001); the quest moves to
--         sequence 4
--   you down: sequence 0xFE
-- The conjurers stand where they are, heal whoever of the side (you included) is lowest below 40 % (they healed at 11
-- to 74 %, median 21-32 %, about every 6 s), and fight their own 1683, then the target, then the 1684s. The elementals
-- heal too and fight the latest enemies. How hard each hits is in their timelines (QB56*).

-- Where retail puts the player and which way they face: their SpawnPlayer in the capture.
START_POSITION = { x = 33.69, y = 3.11, z = 34.99 }
START_ROTATION = 0.795
-- Retail's opening scene 1 here is no cutscene (0x2001, both runs).
OPENING_SCENE_FLAGS = 0x2001

BNPC_TARGET = 4250414
BNPC_SYLPHIE = 4249177
BNPC_WULFIUE = 4249072
BNPC_NOLANEL = 4249073
BNPC_JOACIN = 4249074
CONJURERS = { BNPC_WULFIUE, BNPC_NOLANEL, BNPC_JOACIN }
-- The first wave: each conjurer's own 1683.
FIRST_WAVE = { [BNPC_WULFIUE] = 4249088, [BNPC_NOLANEL] = 4249083, [BNPC_JOACIN] = 4249087 }
-- Sequence 3: a 1685 on Nolanel and a 1684 on Joacin.
SECOND_WAVE = { [4249106] = BNPC_NOLANEL, [4249097] = BNPC_JOACIN }
-- Sequence 4: the elementals, and a 1684 on Wulfiue.
ELEMENTALS = { 4249117, 4249116, 4249119 }
THIRD_WAVE = { [4250514] = BNPC_WULFIUE }
-- After sequence 6: a 1687 on Joacin.
LAST_ARRIVAL = { [4249132] = BNPC_JOACIN }
-- While the elementals haven't come: a 1686, every so often.
BNPC_STRAGGLER = 4249109

-- The quest moves to this sequence when the battle is won.
QUEST_SEQUENCE_AFTER_WIN = 4
SEQUENCE_FIGHTING = 2
SEQUENCE_WON = 7
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 7
FINISH_SCENE_FLAGS = 0x2001
FAIL_DELAY_UPDATES = 12

-- When things move on: the target's health (in percent) and the timers, in updates (half a second each.)
SECOND_WAVE_HEALTH = 80
SECOND_WAVE_LATEST = 80
THIRD_WAVE_HEALTH = 65
STRAGGLER_EVERY = 70
SEQUENCE_5_AFTER = 12
SEQUENCE_6_AFTER = 30
LAST_ARRIVAL_AFTER = 11

-- Who's talking (DirectorPopupMessage): Sylphie by her ENpcResident (an event NPC: kind 3), the conjurers by their
-- BNpcName; no portraits. The content's text IDs, as retail sent them.
SPEAKER_SYLPHIE = { layout = BNPC_SYLPHIE, name = 1006738, icon = 0, kind = 3 }
SPEAKERS = {
    [BNPC_WULFIUE] = { layout = BNPC_WULFIUE, name = 1419, icon = 0 },
    [BNPC_NOLANEL] = { layout = BNPC_NOLANEL, name = 1425, icon = 0 },
    [BNPC_JOACIN] = { layout = BNPC_JOACIN, name = 1426, icon = 0 },
}
LAST_WORDS = { [BNPC_WULFIUE] = 110, [BNPC_NOLANEL] = 109, [BNPC_JOACIN] = 111 }
START_STEPS = {
    { 10, "talk", SPEAKER_SYLPHIE, 103 },
    { 10, "talk", SPEAKER_SYLPHIE, 104 },
    { 10, "yell", BNPC_WULFIUE, 996 },
    { 10, "yell", BNPC_NOLANEL, 997 },
    { 10, "yell", BNPC_JOACIN, 998 },
}

BATTLE = Battle.new()
ACTION_CURE = 120
HEAL_BELOW = 40
HEAL_EVERY_SECONDS = 6
for _, conjurer in ipairs(CONJURERS) do
    BATTLE:ally(conjurer, {
        role = "dps",
        split = false,
        focus = { "mine:" .. conjurer, "boss", "adds" },
        order = "oldest",
        range = 25,
        heals = { { action = ACTION_CURE, targets = "allies", below = HEAL_BELOW, every = HEAL_EVERY_SECONDS } },
    })
end
for _, elemental in ipairs(ELEMENTALS) do
    BATTLE:ally(elemental, {
        role = "dps",
        split = false,
        focus = { "adds", "boss" },
        order = "newest",
        range = 15,
        heals = { { action = ACTION_CURE, targets = "allies", below = HEAL_BELOW, every = HEAL_EVERY_SECONDS } },
    })
end
-- The target stands there: the director holds it (no noticing, chasing or timeline), so it only takes hits.
BATTLE:enemy(BNPC_TARGET, { group = "boss", passive = true })

local over = false
local fail_in = nil
local phase = 1
-- Updates since the fight started, and since sequence 3.
local elapsed = 0
local since_second = 0
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

-- Enemies sent after a conjurer each ({ [enemy] = conjurer }), that conjurer going for them first.
function arrive(director, wave)
    local ids = {}
    for enemy, conjurer in pairs(wave) do
        BATTLE:enemy(enemy, { group = { "mine:" .. conjurer, "adds" }, attacks = conjurer })
        table.insert(ids, enemy)
    end
    table.sort(ids)
    BATTLE:spawn(director, ids)
end

function secondWave(director)
    phase = 2
    since_second = 0
    director:set_sequence(3)
    arrive(director, SECOND_WAVE)
    after(10, function(d) talk(d, SPEAKER_SYLPHIE, 105) end)
    after(30, function(d) talk(d, SPEAKER_SYLPHIE, 106) end)
end

function thirdWave(director)
    phase = 3
    director:set_sequence(4)
    BATTLE:spawn(director, ELEMENTALS)
    arrive(director, THIRD_WAVE)
    after(20, function(d) talk(d, SPEAKER_SYLPHIE, 107) end)
    after(SEQUENCE_5_AFTER, function(d)
        d:set_sequence(5)
        after(SEQUENCE_6_AFTER, function(d6)
            d6:set_sequence(6)
            after(LAST_ARRIVAL_AFTER, function(d7) arrive(d7, LAST_ARRIVAL) end)
            after(20, function(d8) talk(d8, SPEAKER_SYLPHIE, 108) end)
        end)
    end)
end

function onSetup(director)
    director:set_sequence(1)
end

function onCommence(director)
    director:set_sequence(SEQUENCE_FIGHTING)
    director:spawn_bnpc(BNPC_SYLPHIE)
    BATTLE:spawn(director, CONJURERS)
    BATTLE:spawn(director, { BNPC_TARGET })
    director:hold_bnpc(BNPC_TARGET)
    arrive(director, FIRST_WAVE)
    BATTLE:steps(START_STEPS)
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
    elapsed = elapsed + 1
    local target = health[BNPC_TARGET]
    if phase == 1 and (elapsed >= SECOND_WAVE_LATEST or (target ~= nil and target <= SECOND_WAVE_HEALTH)) then
        secondWave(director)
    elseif phase == 2 then
        since_second = since_second + 1
        if target ~= nil and target <= THIRD_WAVE_HEALTH then
            thirdWave(director)
        elseif since_second % STRAGGLER_EVERY == 0 then
            -- The same layout ID each time (retail's second came while the first stood), so by actor ID.
            local actor = director:spawn_bnpc(BNPC_STRAGGLER)
            director:set_hate(actor, BNPC_NOLANEL, Battle.STARTING_ENMITY)
        end
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
    if layout_id == BNPC_TARGET then
        over = true
        BATTLE:clearSteps()
        timers = {}
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
    BATTLE:clearSteps()
    timers = {}
    director:set_sequence(SEQUENCE_FAILED)
    fail_in = FAIL_DELAY_UPDATES
end
