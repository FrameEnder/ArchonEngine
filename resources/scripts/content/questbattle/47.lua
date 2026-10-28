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
-- Its lines: the lead's 112, their leader's 113-116.
--
-- The three companies' battles (46, 47, 48) are one battle with each company's people, timed against all their
-- retail runs (seconds from the fight starting, the director's scene 60000):
--   before: sequence 1 and the opening cutscene (scene 1, 0xF841AEFB), everyone there
--   0     sequence 2; every enemy on the lead, the lead on their leader, the other two on the adds
--   5, 10 battle talk from the lead and from their leader (BNpcName 996, portrait 73215)
--   their leader below 50 % (the Twin Adders' three runs, the Flames'), or the first wave all down, 2.0 s after the
--         last one's health reached 0 (the Maelstrom's, the leader at 60 %): sequence 3 and his next line, another 5 s
--         later; sequence 4 and the magitek vanguard with two archers, all on the lead, 11 s after sequence 3 when the
--         leader's health brought it (11.08, 11.05, 11.06, 9.07) and 8 s after when the wave did (8.0); his last line
--         7 s after that (7.1 every run)
--   the vanguard below 70 % and 40 %: sequences 5 and 6; then each Drill Cannons it begins (525) moves on 2.9 s
--         later: sequence 7, and 8 with the vanguard below 30 % (27.1, 27.5 % at once; 32.2 % no, 17.3 % at the next
--         one); 15 s after 8 sequence 6 again (15.07 s twice), and the next Drill Cannons 7 (the Twin Adders' won run)
--   the vanguard down: 1.5 s after its health reached 0 (2.2, 2.05 and 0.4 s), sequence 9 and "Duty Complete", the
--         battle's scene 9 (a cutscene, 0xF8402EF3) 3.6 s later; the quest moves to 255 and the director to 0xFE with
--         its last byte 0x80; out back where you entered
--   an ally falling: its last words where recorded (the Twin Adders' conjurer, 119); the battle goes on
--   you down: sequence 0xFE (the Twin Adders' two losses)
-- Retail's sequences 5-8 came with no other change anyone could see. The lead and the second ally heal (Cure, 120)
-- whoever of the side is lowest below about 50 %; their leader heals his side (Cure, median 30-35 %). How hard each
-- hits is in their timelines (the foes' are QB46*, the same BNpcBases in all three).

-- Where retail puts the player and which way they face: their SpawnPlayer in the capture.
START_POSITION = { x = 172.03, y = 52.36, z = -72.12 }
START_ROTATION = 0.506
-- Retail's opening cutscene's flags here.
OPENING_SCENE_FLAGS = 0xF841AEFB

BNPC_LEAD = 4105366
BNPC_SECOND = 4105365
BNPC_THIRD = 4105364
BNPC_LEADER = 4105369
BNPC_VANGUARD = 4105368
ALLIES = { BNPC_LEAD, BNPC_SECOND, BNPC_THIRD }
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
-- Sequence 4 and the vanguard after sequence 3: 11 s when their leader's health brought it (11.08, 11.05, 11.06 and
-- 9.07 s in the Twin Adders' and the Flames' runs), 8 s when the first wave falling did (8.0 s, the Maelstrom's.)
SECOND_WAVE_AFTER = 22
SECOND_WAVE_AFTER_CLEARED = 16
-- The first wave's last one down: sequence 3 once its death animation has played (2.0 s, the Maelstrom's run.)
CLEARED_AFTER = 4
VANGUARD_TALK_AFTER = 14
-- Their leader's mark: below half; 49 so the battle-sim's health, drawn straight between retail's samples, crosses it
-- when retail's did.
LEADER_MARK = 49
-- The win: 1.5 s after the vanguard's health reached 0 (2.2, 2.05 and 0.4 s in the three battles' won runs.)
WON_AFTER_DEATH = 3

-- Who's talking (DirectorPopupMessage), by BNpcName, with retail's portrait. The content's text IDs, as retail sent
-- them.
SPEAKER_LEAD = { layout = BNPC_LEAD, name = 994, icon = 73222 }
SPEAKER_LEADER = { layout = BNPC_LEADER, name = 996, icon = 73215 }
TALK_LEAD = 112
TALK_LEADER = { 113, 114, 115, 116 }
SPEAKERS = {}
LAST_WORDS = {}

-- The vanguard's marks: below 70 % sequence 5, below 40 % sequence 6 (all three battles' won runs.)
VANGUARD_MARKS = { { 70, 5 }, { 40, 6 } }
-- After sequence 6 the vanguard's Drill Cannons move it on, 2.9 s after it begins casting them (2.85-2.9 s, every
-- time in the three runs): the first, sequence 7, and with the vanguard below LOW_MARK sequence 8 too (at once, or
-- at a later Drill Cannons: 27.1 and 27.5 % yes, 32.2 % no, then 17.3 % yes); 15 s after 8 sequence 6 again
-- (15.07 s twice), and the next Drill Cannons sequence 7 once more (the Twin Adders' run.)
ACTION_DRILL_CANNONS = 525
DRILL_AFTER = 6
LOW_MARK = 30
AGAIN_AFTER = 30

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
BATTLE:ally(BNPC_SECOND, {
    role = "dps",
    focus = { "adds", "boss" },
    split = false,
    order = "nearest",
    range = 10,
    keep_near = { within = 10 },
    heals = { { action = ACTION_CURE, targets = "allies", below = HEAL_BELOW, every = HEAL_EVERY_SECONDS } },
})
BATTLE:ally(BNPC_THIRD, {
    role = "dps",
    focus = { "adds", "boss" },
    split = false,
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
-- Sequence 3 has come (by their leader's health or the first wave's falling.)
leader_hurt = false
-- What the vanguard's next Drill Cannons does ("7", "8", "7 again", or nothing), the updates until it does, and the
-- updates until sequence 6 comes again.
drill_next = nil
drill_in = nil
again_in = nil
vanguard_health = 100

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
        { 10, "talk", SPEAKER_LEAD, TALK_LEAD },
        { 10, "talk", SPEAKER_LEADER, TALK_LEADER[1] },
    })
    BATTLE:onHealth(BNPC_LEADER, LEADER_MARK, function(d) leaderHurt(d, false) end)
    BATTLE:onCleared(FIRST_WAVE, function(d)
        BATTLE:steps({ { CLEARED_AFTER, "call", function(d2) leaderHurt(d2, true) end } })
    end)
end

-- Their leader below half, or the first wave down: sequence 3, his lines, then the vanguard and two archers.
function leaderHurt(director, cleared)
    if leader_hurt or over then
        return
    end
    leader_hurt = true
    director:set_sequence(3)
    talk(director, SPEAKER_LEADER, TALK_LEADER[2])
    local wave_after = cleared and SECOND_WAVE_AFTER_CLEARED or SECOND_WAVE_AFTER
    BATTLE:steps({
        { 10, "talk", SPEAKER_LEADER, TALK_LEADER[3] },
        { wave_after - 10, "call", function(d) secondWave(d) end },
        { VANGUARD_TALK_AFTER, "talk", SPEAKER_LEADER, TALK_LEADER[4] },
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
            if over then
                return
            end
            d:set_sequence(mark[2])
            if mark[2] == 6 then
                drill_next = "7"
            end
        end)
    end
end

-- The vanguard began an action: its Drill Cannons, after sequence 6, move the director on once it lands.
function onActorCast(director, layout_id, action_id, actor_id)
    if over or layout_id ~= BNPC_VANGUARD or action_id ~= ACTION_DRILL_CANNONS or drill_next == nil then
        return
    end
    if drill_in == nil then
        drill_in = DRILL_AFTER
    end
end

function drillLanded(director)
    local next = drill_next
    if next == "7" or next == "7 again" then
        director:set_sequence(7)
        drill_next = next == "7" and "8" or nil
    end
    if drill_next == "8" and vanguard_health < LOW_MARK then
        director:set_sequence(8)
        drill_next = nil
        again_in = AGAIN_AFTER
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
    if health ~= nil and health[BNPC_VANGUARD] ~= nil then
        vanguard_health = health[BNPC_VANGUARD]
        if vanguard_marks_due then
            vanguard_marks_due = false
            vanguardMarks(director)
        end
    end
    if drill_in ~= nil then
        drill_in = drill_in - 1
        if drill_in <= 0 then
            drill_in = nil
            drillLanded(director)
        end
    end
    if again_in ~= nil then
        again_in = again_in - 1
        if again_in <= 0 then
            again_in = nil
            director:set_sequence(6)
            drill_next = "7 again"
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
    if layout_id == BNPC_VANGUARD then
        drill_next, drill_in, again_in = nil, nil, nil
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
