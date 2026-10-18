-- Quest battle: Underneath the Sultantree, for the quest of the same name (ManWil005_00550.)
-- It takes place in a copy of Central Thanalan (territory 270), so its BNpcs live in the wil_w1 drop-in (layer
-- QB_ManWil005_001): the blanga (4077147, level 12, 13608 HP), the handsome stranger fighting beside you (4077150,
-- level 50, 2589 HP) and three pairs of lesser blangas (level 4, 44 HP). Layout IDs, levels, HP, positions and the
-- start position are retail's (Bob's Chronofoil capture of 2026-10-17, `questkit chronofoil --battle 4.4`); the
-- Kawari Recorder export of the same run agrees.
--
-- Timed against that run (seconds from the fight starting, after the opening cutscene):
--   0      director sequence 2 (sequence 1 is set before the opening cutscene)
--   5      the stranger: battle talk 114
--   the blanga at 70%: sequence 3, two lesser blangas (they go after you); the stranger 115 seven seconds later and
--          116 five after that
--   those two down: the stranger 117 six seconds later
--   the blanga at 40%: sequence 4, two more; the stranger 118 ten seconds later
--   the blanga at about 23%: sequence 5, the last two (it went from 24.2% to 22.1% around then; one run can't tell
--          whether this goes by its health or comes 15 s after the second pair, as it also did)
--   the blanga at 10%: the stranger 119
--   everyone down: sequence 6, "Duty Complete", the quest battle's scene 6 (0x2001) and back out; the quest moves
--          to sequence 2
-- The stranger stays on the blanga the whole time, and the blanga on him; the lesser
-- blangas go after you. How hard each hits is in their timelines' recorded_amounts.

-- Where retail puts the player and which way they face: their SpawnPlayer in the capture.
START_POSITION = { x = 198.3, y = 14.24, z = 538.25 }
START_ROTATION = -1.24

BNPC_BLANGA = 4077147
BNPC_STRANGER = 4077150
PAIRS = {
    { health = 70, sequence = 3, adds = { 4077151, 4077152 } },
    { health = 40, sequence = 4, adds = { 4077427, 4077428 } },
    { health = 23, sequence = 5, adds = { 4077429, 4077430 } },
}

-- The quest moves to this sequence when the battle is won.
QUEST_SEQUENCE_AFTER_WIN = 2
SEQUENCE_START = 1
SEQUENCE_FIGHTING = 2
SEQUENCE_WON = 6
SEQUENCE_FAILED = 0xFE
-- How many updates (half a second each) to wait after the player falls before sending them out.
FAIL_DELAY_UPDATES = 12

-- The stranger's battle talk (DirectorPopupMessage): his BNpcName and portrait here, and the content's text IDs, as
-- retail sent them.
SPEAKER_STRANGER = { layout = BNPC_STRANGER, name = 1377, icon = 0x11dcd }
-- Steps: { updates after the step before, kind, ... } (Battle:steps)
START_STEPS = {
    { 10, "talk", SPEAKER_STRANGER, 114 },
}
PAIR_STEPS = {
    [1] = { { 14, "talk", SPEAKER_STRANGER, 115 }, { 10, "talk", SPEAKER_STRANGER, 116 } },
    [2] = { { 20, "talk", SPEAKER_STRANGER, 118 } },
}
FIRST_PAIR_CLEARED_STEPS = { { 12, "talk", SPEAKER_STRANGER, 117 } }
LOW_HEALTH = 10
LOW_HEALTH_STEPS = { { 0, "talk", SPEAKER_STRANGER, 119 } }

-- The battle's outline (content/lib/Battle.lua): the stranger (a gladiator) plays the tank, up close on the blanga
-- (it on him) and only then on whatever's left; he doesn't pull the lesser blangas off you.
BATTLE = Battle.new()
BATTLE:ally(BNPC_STRANGER, { role = "tank", protect = false, focus = { "boss", "adds" }, range = 3 })
BATTLE:enemy(BNPC_BLANGA, { group = "boss", attacks = BNPC_STRANGER })
for n, pair in ipairs(PAIRS) do
    for _, id in ipairs(pair.adds) do
        BATTLE:enemy(id, { group = "adds" })
    end
    BATTLE:onHealth(BNPC_BLANGA, pair.health, function(director)
        director:set_sequence(pair.sequence)
        BATTLE:spawn(director, pair.adds)
        if PAIR_STEPS[n] ~= nil then
            BATTLE:steps(PAIR_STEPS[n])
        end
    end)
end
BATTLE:onCleared(PAIRS[1].adds, function()
    BATTLE:steps(FIRST_PAIR_CLEARED_STEPS)
end)
BATTLE:onHealth(BNPC_BLANGA, LOW_HEALTH, function()
    BATTLE:steps(LOW_HEALTH_STEPS)
end)

local over = false
local fail_in = nil

function onSetup(director)
    director:set_sequence(SEQUENCE_START)
end

function onCommence(director)
    director:set_sequence(SEQUENCE_FIGHTING)
    BATTLE:spawn(director, { BNPC_STRANGER })
    BATTLE:spawn(director, { BNPC_BLANGA })
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
    BATTLE:update(director, health, player_health, info)
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    if BATTLE:enemiesLeft() == 0 and BATTLE.alive[BNPC_BLANGA] == nil then
        over = true
        BATTLE:clearSteps()
        director:set_sequence(SEQUENCE_WON)
        director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN)
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
