-- Quest battle: The Company You Keep (Immortal Flames)'s, for that quest (ManWil302_00682.)
-- It takes place in territory 278, whose Bg is Western Thanalan's (wil_w1/fld/w1f1, the same as territory 140), so its
-- NPCs are layer QB_ManWil302_001 of the wil_w1f1 drop-in: three Immortal Flames on your side (the gladiator who leads,
-- 4105366, BNpcBase 1024, BNpcName 994, 4956 HP; a conjurer, 4105365, 1025, BNpcName 1006; a thaumaturge, 4105364,
-- 1026, BNpcName 1005), the Garlean gladiator who leads the other side (4105369, 1020, BNpcName 996, level 22, 3204 HP)
-- with the first wave (two lancers 1021, a thaumaturge 1022, two archers 1023), then a magitek vanguard (4105368, 1016,
-- BNpcName 269, 4683 HP) with two more archers: the Twin Adders' battle (46) with the Flames, its foes the same
-- BNpcBases (their timelines are QB46's.) Layout IDs, levels, HP, positions, display flags and the start spot are
-- retail's (Bob's Chronofoil capture c99fe374 of 2026-10-28, `questkit chronofoil --battle 0.47`, won; Dags, a level
-- 24 gladiator.)
--
-- Timed against the run (seconds from the fight starting, the director's scene 60000):
--   before: sequence 1 and the opening cutscene (scene 1, 0xF841AEFB), everyone there
--   0     sequence 2; every enemy on the lead gladiator, the lead on their leader, the conjurer and the thaumaturge on
--         the lancers
--   5, 10 battle talk 112 (the lead, BNpcName 994, portrait 73222), 113 (their leader, 996, 73215)
--   their leader below 50 % (50.8 -> 46.8 %, 47.3 s in): sequence 3 and 114, 115 5 s later; 11 s after it (11.05)
--         sequence 4 and the magitek vanguard with two archers, all on the lead; 116 7 s after it (7.1)
--   the vanguard below 70, 40 and 27 % (71.6 -> 69.9, 40.5 -> 37.4, 28.4 -> 27.5): sequences 5, 6, then 7 and 8; 15 s
--         after 8 (15.1) sequence 6 again, and 6.3 s after that 7 (the Twin Adders' won run; here the vanguard was
--         down first)
--   the vanguard down: 2.0 s after its health reached 0 (its death animation), sequence 9 and "Duty Complete", the
--         battle's scene 9 (a cutscene, 0xF8402EF3) 3.6 s later; the quest moves to 255 and the director to 0xFE with
--         its last byte 0x80; out back where you entered (Western Thanalan, the capture's next visit)
--   you down: sequence 0xFE (the Twin Adders' battle's; no loss recorded here)
-- The lead and the conjurer heal (Cure, 120) whoever of the side is lowest (retail's at 35-55 %, medians 52 and 43 %);
-- their leader heals his side (Cure, median 35 %). How hard each hits is in their timelines (QB47* for the Flames,
-- QB46* for the others.)

-- Where retail puts the player and which way they face: their SpawnPlayer in the capture.
START_POSITION = { x = 172.03, y = 52.36, z = -72.12 }
START_ROTATION = 0.506
-- Retail's opening cutscene's flags here.
OPENING_SCENE_FLAGS = 0xF841AEFB

BNPC_LEAD = 4105366
BNPC_CONJURER = 4105365
BNPC_THAUMATURGE = 4105364
BNPC_LEADER = 4105369
BNPC_VANGUARD = 4105368
ALLIES = { BNPC_LEAD, BNPC_CONJURER, BNPC_THAUMATURGE }
FIRST_WAVE = { 4105370, 4105371, 4105372, 4105373, 4105375 }
SECOND_WAVE = { 4105390, 4105392 }

-- The quest moves to this sequence when the battle is won.
QUEST_SEQUENCE_AFTER_WIN = 255
SEQUENCE_FIGHTING = 2
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 9
FINISH_SCENE_FLAGS = 0xF8402EF3
FAIL_DELAY_UPDATES = 12

-- Timers, in updates (half a second each.)
SECOND_WAVE_AFTER = 22
VANGUARD_TALK_AFTER = 14
-- After the vanguard's third mark (sequence 8): sequence 6 15 s later, then 7 6.3 s after that (both runs of the two
-- battles: 15.07 and 15.21 s, then 6.28 s.)
VANGUARD_AFTER_8 = 30
VANGUARD_AFTER_6 = 13
-- Their leader's mark: below half; 49 so the battle-sim's health, drawn straight between retail's samples, crosses it
-- when retail's did.
LEADER_MARK = 49
-- The win came as the vanguard's death animation ended (2.0 s after its health reached 0.)
WON_AFTER_DEATH = 4

-- Who's talking (DirectorPopupMessage), by BNpcName, with retail's portrait. The content's text IDs, as retail sent
-- them.
SPEAKER_LEAD = { layout = BNPC_LEAD, name = 994, icon = 73222 }
SPEAKER_LEADER = { layout = BNPC_LEADER, name = 996, icon = 73215 }

-- The vanguard's marks: { percent, director sequence(s) }; the last one starts the timed ones.
VANGUARD_MARKS = {
    { 70, { 5 } },
    { 40, { 6 } },
    { 27, { 7, 8 } },
}

BATTLE = Battle.new()
ACTION_CURE = 120
HEAL_BELOW = 50
HEAL_EVERY_SECONDS = 6
BATTLE:ally(BNPC_LEAD, {
    role = "tank",
    focus = { "boss", "adds" },
    range = 3,
    heals = { { action = ACTION_CURE, targets = "allies", below = HEAL_BELOW, every = HEAL_EVERY_SECONDS } },
})
BATTLE:ally(BNPC_CONJURER, {
    role = "dps",
    split = false,
    focus = { "adds", "boss" },
    order = "nearest",
    range = 10,
    keep_near = { within = 10 },
    heals = { { action = ACTION_CURE, targets = "allies", below = HEAL_BELOW, every = HEAL_EVERY_SECONDS } },
})
BATTLE:ally(BNPC_THAUMATURGE, {
    role = "dps",
    split = false,
    focus = { "adds", "boss" },
    order = "nearest",
    range = 8,
    keep_near = { within = 15 },
})
BATTLE:enemy(BNPC_LEADER, {
    group = "boss",
    attacks = BNPC_LEAD,
    heals = { { action = ACTION_CURE, targets = "allies", below = 35, every = HEAL_EVERY_SECONDS } },
})
for _, enemy in ipairs(FIRST_WAVE) do
    BATTLE:enemy(enemy, { group = "adds", attacks = BNPC_LEAD })
end

local over = false
local fail_in = nil
vanguard_marks_due = false

function talk(director, speaker, text)
    director:battle_talk(speaker.layout, speaker.name, text, speaker.icon)
end

function onSetup(director)
    director:set_sequence(1)
end

function onCommence(director)
    director:set_sequence(SEQUENCE_FIGHTING)
    BATTLE:spawn(director, ALLIES)
    BATTLE:spawn(director, { BNPC_LEADER })
    BATTLE:spawn(director, FIRST_WAVE)
    BATTLE:steps({
        { 10, "talk", SPEAKER_LEAD, 112 },
        { 10, "talk", SPEAKER_LEADER, 113 },
    })
    BATTLE:onHealth(BNPC_LEADER, LEADER_MARK, function(d) leaderHurt(d) end)
end

-- Their leader below half: sequence 3, his lines, then the vanguard and two archers.
function leaderHurt(director)
    director:set_sequence(3)
    talk(director, SPEAKER_LEADER, 114)
    BATTLE:steps({
        { 10, "talk", SPEAKER_LEADER, 115 },
        { SECOND_WAVE_AFTER - 10, "call", function(d) secondWave(d) end },
        { VANGUARD_TALK_AFTER, "talk", SPEAKER_LEADER, 116 },
    })
end

function secondWave(director)
    director:set_sequence(4)
    BATTLE:enemy(BNPC_VANGUARD, { group = "boss", attacks = BNPC_LEAD })
    for _, enemy in ipairs(SECOND_WAVE) do
        BATTLE:enemy(enemy, { group = "adds", attacks = BNPC_LEAD })
    end
    BATTLE:spawn(director, { BNPC_VANGUARD })
    BATTLE:spawn(director, SECOND_WAVE)
    -- Its marks once its health is known (onUpdate): a mark with no health yet would count as passed.
    vanguard_marks_due = true
end

function vanguardMarks(director)
    for index, mark in ipairs(VANGUARD_MARKS) do
        BATTLE:onHealth(BNPC_VANGUARD, mark[1], function(d)
            if over then
                return
            end
            for _, sequence in ipairs(mark[2]) do
                d:set_sequence(sequence)
            end
            if index == #VANGUARD_MARKS then
                BATTLE:steps({
                    { VANGUARD_AFTER_8, "call", function(d2) if not over then d2:set_sequence(6) end end },
                    { VANGUARD_AFTER_6, "call", function(d2) if not over then d2:set_sequence(7) end end },
                })
            end
        end)
    end
end

function won(director)
    if over then
        return
    end
    over = true
    BATTLE:clearSteps()
    director:set_sequence(9)
    director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS)
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
    if vanguard_marks_due and health ~= nil and health[BNPC_VANGUARD] ~= nil then
        vanguard_marks_due = false
        vanguardMarks(director)
    end
    BATTLE:update(director, health, player_health, info)
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    if layout_id == BNPC_VANGUARD then
        BATTLE:clearSteps()
        BATTLE:steps({ { WON_AFTER_DEATH, "call", function(d) won(d) end } })
    end
end

-- Losing sends you back to where you entered, still on the same step of the quest, so you can try again.
function onPlayerDeath(director)
    if over then
        return
    end
    over = true
    BATTLE:clearSteps()
    director:set_sequence(SEQUENCE_FAILED)
    fail_in = FAIL_DELAY_UPDATES
end
