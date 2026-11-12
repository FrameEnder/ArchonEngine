-- Quest battle 80: Austerities of Earth (JobSmn350_01092, the summoners' level 35 quest.)
-- It takes place in territory 324, whose level is the North Shroud's (f1f4), so its NPCs are layer QB_JobSmn350_001
-- of fst_f1f4: Y'mhitra the Learned on your side (4301637, base 2208, 4270 HP, a conjurer); against you Titan-Egi
-- (4301700, 2203, 7183 HP) with a stone shiver (4301671, 2204) and a moraine golem (4301681, 2205), then, after a
-- cutscene, the dark summoner (4301698, 2206, 5224 HP) and his Ifrit-Egi (4301699, 2207, 8540 HP.) Layout IDs, levels,
-- HP, positions, display and character data flags, looks and the start spot are retail's (Bob's Kawari Recorder run
-- of 2026-11-12, `questkit battle`: one run, won, a level 39 arcanist.)
--
-- Retail's flow (seconds from the director's scene 60000):
--   0       sequence 2 (0.2 s after); Titan-Egi on Y'mhitra, then you (9 s in); her battle talk 81 at 3.1
--   Titan-Egi under 80 % (15.5): sequence 3, the stone shiver 0.4 s later, talk 82 6.1 s after
--   Titan-Egi under 50 % (34.1): sequence 4
--   Titan-Egi under 25 % (50.5): sequence 5, the moraine golem 0.3 s later, talk 83 3.1 s after
--   Titan-Egi down (64.7): sequence 6 and the battle's scene 6 (0xF8402EFB, the dark summoner) at once; whoever was
--           still out went. As it ended (3.3 s): sequence 7 (the client's OnSequence007 sets the new objective), and
--           0.3 s later the dark summoner, his Ifrit-Egi (on Y'mhitra) and Y'mhitra where she began; the summoner's
--           talk 84 and Y'mhitra's 85 3.1 and 8.1 s after sequence 7
--   Ifrit-Egi under 75 % (86.3): sequence 8, talks 86 (the summoner) and 87 (Y'mhitra) 3.1 and 8.1 s after
--   Ifrit-Egi under 30 % (123.1): sequence 9; 5.1 s later (Ifrit-Egi at 20-21 %, not a health mark) sequence 10 and
--           the battle's scene 10 (0xF8402EFB) at once, with no "Duty Complete": the quest moves to sequence 2.
--           Neither the summoner nor his Ifrit-Egi falls.
-- What each one does is in their timelines (QB80*).

function won(director)
    over = true
    timers = {}
    sequence(director, SEQUENCE_WON)
    director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS, nil, false)
end

-- Where retail put the player and which way they faced.
START_POSITION = { x = -273.6, y = -80.82, z = 478.47 }
START_ROTATION = 0.63
-- Retail's opening scene 1 here.
OPENING_SCENE_FLAGS = 0xF841AEFB

BNPC_YMHITRA = 4301637
BNPC_TITAN = 4301700
BNPC_SHIVER = 4301671
BNPC_GOLEM = 4301681
BNPC_SUMMONER = 4301698
BNPC_IFRIT = 4301699

QUEST_SEQUENCE_AFTER_WIN = 2
SEQUENCE_FIGHTING = 2
SEQUENCE_SHIVER = 3
SEQUENCE_TITAN_HALF = 4
SEQUENCE_GOLEM = 5
SEQUENCE_CUTSCENE = 6
SEQUENCE_SUMMONER = 7
SEQUENCE_IFRIT_75 = 8
SEQUENCE_IFRIT_30 = 9
SEQUENCE_WON = 10
SEQUENCE_FAILED = 0xFE
CUTSCENE = 6
CUTSCENE_FLAGS = 0xF8402EFB
FINISH_SCENE = 10
FINISH_SCENE_FLAGS = 0xF8402EFB
FAIL_DELAY_UPDATES = 12

-- Health marks (percent): Titan-Egi's, then Ifrit-Egi's.
SHIVER_HEALTH = 80
TITAN_HALF = 50
GOLEM_HEALTH = 25
IFRIT_75 = 75
IFRIT_30 = 30

ACTION_CURE = 120

SPEAKER_YMHITRA = { layout = BNPC_YMHITRA, name = 1640, icon = 0 }
SPEAKER_SUMMONER = { layout = BNPC_SUMMONER, name = 2211, icon = 0 }

-- Timers, in updates (half a second each). A timer of n fires n / 2 s after it's set in onUpdate, (n - 1) / 2 s after
-- it's set before the next update (onCommence, onActorDeath, onSceneFinished.)
TALKS_FIGHTING = { { 7, SPEAKER_YMHITRA, 81 } }
TALKS_SHIVER = { { 12, SPEAKER_YMHITRA, 82 } }
TALKS_GOLEM = { { 6, SPEAKER_YMHITRA, 83 } }
TALKS_SUMMONER = { { 7, SPEAKER_SUMMONER, 84 }, { 17, SPEAKER_YMHITRA, 85 } }
TALKS_IFRIT_75 = { { 6, SPEAKER_SUMMONER, 86 }, { 16, SPEAKER_YMHITRA, 87 } }
ADD_AFTER = 1
SECOND_PART_AFTER = 2
WON_AFTER = 10

BATTLE = Battle.new()
BATTLE:ally(BNPC_YMHITRA, {
    role = "dps",
    focus = { "adds", "boss" },
    range = 15,
    heals = { { action = ACTION_CURE, targets = { "player" }, below = 50, every = 12 } },
})
BATTLE:enemy(BNPC_TITAN, { group = "boss", attacks = BNPC_YMHITRA })
BATTLE:enemy(BNPC_SHIVER, { group = "adds" })
BATTLE:enemy(BNPC_GOLEM, { group = "adds" })
BATTLE:enemy(BNPC_IFRIT, { group = "boss", attacks = BNPC_YMHITRA })
BATTLE:enemy(BNPC_SUMMONER, { group = "summoner" })

local over = false
local fail_in = nil
local phase = 1
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

function talks(lines)
    for _, line in ipairs(lines) do
        local speaker, text = line[2], line[3]
        after(line[1], function(d)
            d:battle_talk(speaker.layout, speaker.name, text, speaker.icon)
        end)
    end
end

function sequence(director, value)
    phase = value
    director:set_sequence(value)
end

function onSetup(director)
    director:set_sequence(1)
end

function onCommence(director)
    sequence(director, SEQUENCE_FIGHTING)
    BATTLE:spawn(director, { BNPC_YMHITRA, BNPC_TITAN })
    talks(TALKS_FIGHTING)
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
    local titan, ifrit = health[BNPC_TITAN], health[BNPC_IFRIT]
    if titan ~= nil then
        if phase == SEQUENCE_FIGHTING and titan < SHIVER_HEALTH then
            sequence(director, SEQUENCE_SHIVER)
            after(ADD_AFTER, function(d) BATTLE:spawn(d, { BNPC_SHIVER }) end)
            talks(TALKS_SHIVER)
        end
        if phase == SEQUENCE_SHIVER and titan < TITAN_HALF then
            sequence(director, SEQUENCE_TITAN_HALF)
        end
        if phase == SEQUENCE_TITAN_HALF and titan < GOLEM_HEALTH then
            sequence(director, SEQUENCE_GOLEM)
            after(ADD_AFTER, function(d) BATTLE:spawn(d, { BNPC_GOLEM }) end)
            talks(TALKS_GOLEM)
        end
    end
    if ifrit ~= nil then
        if phase == SEQUENCE_SUMMONER and ifrit < IFRIT_75 then
            sequence(director, SEQUENCE_IFRIT_75)
            talks(TALKS_IFRIT_75)
        end
        -- Under 30 %: won 5 s later, its closing scene at once, no fanfare.
        if (phase == SEQUENCE_SUMMONER or phase == SEQUENCE_IFRIT_75) and ifrit < IFRIT_30 then
            sequence(director, SEQUENCE_IFRIT_30)
            after(WON_AFTER, won)
        end
    end
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    if layout_id ~= BNPC_TITAN or phase >= SEQUENCE_CUTSCENE then
        return
    end
    -- Titan-Egi down: the dark summoner's cutscene; whoever's still out goes, Y'mhitra comes back where she began.
    timers = {}
    sequence(director, SEQUENCE_CUTSCENE)
    for _, layout in ipairs({ BNPC_SHIVER, BNPC_GOLEM, BNPC_YMHITRA }) do
        director:despawn_bnpc(layout)
        BATTLE:died(director, layout)
    end
    director:play_scene(CUTSCENE, CUTSCENE_FLAGS)
end

function onSceneFinished(director, scene)
    if scene ~= CUTSCENE or phase ~= SEQUENCE_CUTSCENE or over then
        return
    end
    sequence(director, SEQUENCE_SUMMONER)
    after(SECOND_PART_AFTER, function(d) BATTLE:spawn(d, { BNPC_YMHITRA, BNPC_IFRIT, BNPC_SUMMONER }) end)
    talks(TALKS_SUMMONER)
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
