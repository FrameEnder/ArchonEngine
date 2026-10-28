-- Quest battle: The Company You Keep (Twin Adder)'s, for that quest (ManFst302_00680.)
-- It takes place in territory 277, whose Bg is the East Shroud's (fst_f1/fld/f1f2, the same as territory 152), so its
-- NPCs are layer QB_ManFst302_001 of the fst_f1f2 drop-in: three Twin Adders on your side (the gladiator who leads,
-- 4103768, BNpcBase 1017, BNpcName 1001, 4956 HP; a conjurer, 4103769, 1018, BNpcName 1176; an archer, 4103770, 1019,
-- BNpcName 999), the Garlean gladiator who leads the other side (4103775, 1020, BNpcName 996, level 22, 3204 HP) with
-- the first wave (two lancers 1021, a thaumaturge 1022, two archers 1023), then a magitek vanguard (4103788, 1016,
-- BNpcName 269, 4683 HP) with two more archers. Layout IDs, levels, HP, positions, looks, display flags and the start
-- spot are retail's (Bob's Chronofoil capture b8432613 of 2026-10-27, `questkit chronofoil --battle 0.30` and `0.32`
-- lost, `0.34` won; a level 24 conjurer.)
--
-- Timed against the three runs (seconds from the fight starting, the director's scene 60000):
--   before: sequence 1 and the opening cutscene (scene 1, 0xF841AEFB), everyone there
--   0     sequence 2; every enemy on the lead gladiator (all three runs), the lead on their leader, the conjurer and
--         the archer on the lancers
--   5, 10 battle talk 113 (the lead, BNpcName 1001, portrait 73216), 114 (their leader, 996, 73215)
--   their leader below 50 % (49, 49, 47 %; 87.8-90.1 s in the lost runs, 57.5 s in the won one): sequence 3 and 115,
--         116 5 s later; 11 s after it (11.0, 11.1; 9.1 once) sequence 4 and the magitek vanguard with two archers,
--         all on the lead; 117 7 s after it (7.1, 7.1, 7.1)
--   the vanguard at 70, 40, 27, 14 and 8 % (68, 39, 27, 14, 8 % as they came in the won run): sequences 5, 6, 7 then
--         8, 6, 7
--   the vanguard down: 2.2 s after its health reached 0 (its death animation), sequence 9 and "Duty Complete", the
--         battle's scene 9 (a cutscene, 0xF8402EF3) 3.6 s later; the quest moves to 255 and the director to 0xFE with its last byte 0x80
--   the conjurer falling: its last words (119, BNpcName 1176, portrait 73217; a lost run); the battle goes on
--   you down: sequence 0xFE (both losses)
-- Retail's sequences 5-8 came with no other change anyone could see; their marks are the won run's, one run.
-- The lead and the conjurer heal (Cure, 120) whoever of the side is lowest below about 50 % (medians 45-47 %); their
-- leader heals his side (Cure, median 30 %). How hard each hits is in their timelines (QB46*).

-- Where retail puts the player and which way they face: their SpawnPlayer in the capture.
START_POSITION = { x = 51.32, y = 7.89, z = 450.57 }
START_ROTATION = 0.0
-- Retail's opening cutscene's flags here (all three runs.)
OPENING_SCENE_FLAGS = 0xF841AEFB

BNPC_LEAD = 4103768
BNPC_CONJURER = 4103769
BNPC_ARCHER = 4103770
BNPC_LEADER = 4103775
BNPC_VANGUARD = 4103788
ALLIES = { BNPC_LEAD, BNPC_CONJURER, BNPC_ARCHER }
FIRST_WAVE = { 4103777, 4103779, 4103781, 4103782, 4103793 }
SECOND_WAVE = { 4103786, 4103787 }

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
-- Their leader's mark: below half (retail's came as his health went from 50.1 / 50.3 / 50.8 % to 48.7 / 48.7 /
-- 46.7 %); 49 so the battle-sim's health, drawn straight between retail's samples, crosses it when retail's did.
LEADER_MARK = 49
-- The win came as the vanguard's death animation ended (2.2 s after its health reached 0.)
WON_AFTER_DEATH = 4

-- Who's talking (DirectorPopupMessage), by BNpcName, with retail's portrait. The content's text IDs, as retail sent
-- them.
SPEAKER_LEAD = { layout = BNPC_LEAD, name = 1001, icon = 73216 }
SPEAKER_LEADER = { layout = BNPC_LEADER, name = 996, icon = 73215 }
SPEAKERS = {
    [BNPC_CONJURER] = { layout = BNPC_CONJURER, name = 1176, icon = 73217 },
}
LAST_WORDS = { [BNPC_CONJURER] = 119 }

-- The vanguard's marks: { percent, director sequence(s) }.
VANGUARD_MARKS = {
    { 70, { 5 } },
    { 40, { 6 } },
    { 27, { 7, 8 } },
    { 14, { 6 } },
    { 8, { 7 } },
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
BATTLE:ally(BNPC_ARCHER, {
    role = "dps",
    split = false,
    focus = { "adds", "boss" },
    order = "nearest",
    range = 10,
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
        { 10, "talk", SPEAKER_LEAD, 113 },
        { 10, "talk", SPEAKER_LEADER, 114 },
    })
    BATTLE:onHealth(BNPC_LEADER, LEADER_MARK, function(d) leaderHurt(d) end)
end

-- Their leader below half: sequence 3, his lines, then the vanguard and two archers.
function leaderHurt(director)
    director:set_sequence(3)
    talk(director, SPEAKER_LEADER, 115)
    BATTLE:steps({
        { 10, "talk", SPEAKER_LEADER, 116 },
        { SECOND_WAVE_AFTER - 10, "call", function(d) secondWave(d) end },
        { VANGUARD_TALK_AFTER, "talk", SPEAKER_LEADER, 117 },
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
    for _, mark in ipairs(VANGUARD_MARKS) do
        BATTLE:onHealth(BNPC_VANGUARD, mark[1], function(d)
            if not over then
                for _, sequence in ipairs(mark[2]) do
                    d:set_sequence(sequence)
                end
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
    if LAST_WORDS[layout_id] ~= nil then
        talk(director, SPEAKERS[layout_id], LAST_WORDS[layout_id])
    end
    if layout_id == BNPC_VANGUARD then
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
