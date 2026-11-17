-- Quest battle: Lurkers in the Grotto, for the quest of the same name (ManSea005_00543.)
-- It takes place in a copy of Middle La Noscea (territory 252: retail's positions in it are Middle La Noscea's), so its
-- BNpcs live in the sea_s1f1 drop-in (layer QB_ManSea005_001): the ally fighting beside you (4083622, level 50, 1372
-- HP, a conjurer), the boss (4083604, level 12, 6196 HP) and three waves of two (4083606/07 and 4083610/11, level 5,
-- 56 HP; 4083608/09, level 6, 75 HP). Layout IDs, levels, HP, positions, display flags and the start position are
-- retail's (Bob's Chronofoil capture of 2026-10-21, `questkit chronofoil --battle 0.7` and `0.9`, territory 252, duty
-- 332: Bob lost the first run and won the second.)
--
-- Timed against those runs (seconds from the fight starting, the director's scene 60000, Kawari's onCommence):
--   0      director sequence 2 (sequence 1 is set before the opening cutscene)
--   10     the ally: battle talk 82 (10.0 s both runs)
--   the boss at 70%: sequence 3 and the first wave (4083606/07); the ally's 83 seven seconds later (7.1, 7.0, 7.1)
--          and 84 22 s after the sequence (22.07 and 22.06 s in the two won runs)
--   the boss at 40%: sequence 4 and the second wave (4083608/09)
--   10 s after sequence 4 (10.0, 10.1, 10.1): sequence 5 and the third wave (4083610/11) with the ally's 85
--   the boss at 15%: the ally's 86, and 87 13.0 s after it (one run)
-- The health marks and the lines' clocks are from a third run, a Marauder's (Bob's Chronofoil capture 0cafbf61 of
-- 2026-11-17, `--battle 0.7`, won in 79 s: `battles/35-0cafbf61.json`), which showed the boss's health far more
-- often: 72.2 % 1.6 s before sequence 3 and 69.0 % 1.1 s after it; 40.4 % 0.3 s before sequence 4 (then a tick of the
-- ally's Aero: 37.6 %); 86 in the moment a hit took it from about 18 % to 13.3 %. The October runs' samples were 6 to
-- 10 s apart, hence the 65 % and 41 % here before, and 84, 86 and 87 were read as the player's health (below 80, 40
-- and 15 %: 78 and 79 % at 84 in those two): the Marauder was at 97 % at 84 and 83 % at 86. The boss was down 7.3 s
-- after 86 in that run, and 87 never came; in the October run that won, 86 and 87 were 13.0 s apart, with the boss's
-- health seen at 20.5 % 3.7 s before 86 and not again until it fell.
--   everyone down: 2 s later (the last one's death animation, as retail waits) sequence 6, "Duty Complete", the quest
--          battle's scene 6 (0x2001) and back out; the quest moves to sequence 2 with UI8BH = 1 (ITEM0) in the same
--          update
--   the player down (the lost run): sequence 0xFE, and out 6 s later; the quest stays as it was
-- As on retail (both runs): the boss fights the ally the whole time and the waves go after you; the ally stays on the
-- boss, then takes the adds one by one, and casts Cure (120) on you (at 17-48%, every 12-20 s.) It goes to the boss,
-- which hardly moves (at most 4.1 yalms in both runs): it was within 3 yalms of you 90% of the time on retail because
-- you were by the boss too. Kept within 3 of you (the first version), it stayed by you when you hung back, the boss came
-- over to it and everyone ended up on top of you (Bob's 2026-10-21 bundle).
-- How hard each hits is in their timelines' recorded_amounts (QB35*_<base>.json).

-- Where retail puts the player and which way they face: their SpawnPlayer in the capture.
START_POSITION = { x = -58.21, y = 25.26, z = -135.32 }
START_ROTATION = 0.122

BNPC_ALLY = 4083622
BNPC_BOSS = 4083604
WAVES = {
    { health = 70, sequence = 3, adds = { 4083606, 4083607 } },
    { health = 40, sequence = 4, adds = { 4083608, 4083609 } },
}
-- The third wave comes on a timer after the second, with sequence 5.
LAST_WAVE = { sequence = 5, adds = { 4083610, 4083611 } }
LAST_WAVE_AFTER_UPDATES = 20 -- 10 s

-- The quest moves to this sequence when the battle is won, with these quest variables (index: value) in the same
-- update: UI8BH = 1, ITEM0 (retail's UpdateQuest on winning.)
QUEST_SEQUENCE_AFTER_WIN = 2
QUEST_VARS_AFTER_WIN = { [1] = 0x10 }
SEQUENCE_START = 1
SEQUENCE_FIGHTING = 2
SEQUENCE_WON = 6
SEQUENCE_FAILED = 0xFE
-- The scene retail plays after "Duty Complete", and its flags; the player leaves when it ends.
FINISH_SCENE = 6
FINISH_SCENE_FLAGS = 0x2001
-- How many updates (half a second each) to wait after the player falls before sending them out.
FAIL_DELAY_UPDATES = 12
-- "Duty Complete" came 2 s after the last enemy's health reached 0, as its death animation ended.
WON_DELAY_UPDATES = 4

-- The ally's battle talk (DirectorPopupMessage): their BNpcName and portrait here, and the content's text IDs, as
-- retail sent them.
SPEAKER_ALLY = { layout = BNPC_ALLY, name = 1374, icon = 73187 }
-- Steps: { updates after the step before, kind, ... } (Battle:steps)
START_STEPS = { { 20, "talk", SPEAKER_ALLY, 82 } }
FIRST_WAVE_STEPS = { { 14, "talk", SPEAKER_ALLY, 83 } }
-- The ally's other lines: 84 this long after sequence 3 (22 s), 86 as the boss falls to 15 %, 87 this long after 86
-- (13 s), in updates.
TALK_84 = 84
TALK_84_AFTER_UPDATES = 44
TALK_86 = 86
TALK_86_BOSS_HEALTH = 15
TALK_87 = 87
TALK_87_AFTER_UPDATES = 26
LAST_WAVE_TALK = 85

-- The battle's outline (content/lib/Battle.lua): the ally, a conjurer, plays the support: on the boss first, then the
-- adds, from about 5 yalms, near you, healing you when you're low. The boss goes after the ally, the waves after you.
BATTLE = Battle.new()
ACTION_CURE = 120
HEAL_BELOW_HEALTH = 50 -- retail's six heals came at 17-48%
HEAL_EVERY_SECONDS = 12
BATTLE:ally(BNPC_ALLY, {
    role = "support",
    focus = { "boss", "adds" },
    range = 5,
    -- The support role's own keep_near (near the tank and you, within 20) still applies.
    heals = {
        { action = ACTION_CURE, targets = { Battle.PLAYER }, below = HEAL_BELOW_HEALTH, every = HEAL_EVERY_SECONDS },
    },
})
BATTLE:enemy(BNPC_BOSS, { group = "boss", attacks = BNPC_ALLY })
for _, wave in ipairs(WAVES) do
    for _, id in ipairs(wave.adds) do
        BATTLE:enemy(id, { group = "adds" })
    end
end
for _, id in ipairs(LAST_WAVE.adds) do
    BATTLE:enemy(id, { group = "adds" })
end

-- Updates until each of the ally's timed lines, by its text.
talk_in = {}

local function lastWave(director)
    director:set_sequence(LAST_WAVE.sequence)
    BATTLE:spawn(director, LAST_WAVE.adds)
    director:battle_talk(SPEAKER_ALLY.layout, SPEAKER_ALLY.name, LAST_WAVE_TALK, SPEAKER_ALLY.icon)
end

for n, wave in ipairs(WAVES) do
    BATTLE:onHealth(BNPC_BOSS, wave.health, function(director)
        director:set_sequence(wave.sequence)
        BATTLE:spawn(director, wave.adds)
        if n == 1 then
            BATTLE:steps(FIRST_WAVE_STEPS)
            talk_in[TALK_84] = TALK_84_AFTER_UPDATES
        else
            BATTLE:steps({ { LAST_WAVE_AFTER_UPDATES, "call", lastWave } })
        end
    end)
end

BATTLE:onHealth(BNPC_BOSS, TALK_86_BOSS_HEALTH, function(director)
    director:battle_talk(SPEAKER_ALLY.layout, SPEAKER_ALLY.name, TALK_86, SPEAKER_ALLY.icon)
    talk_in[TALK_87] = TALK_87_AFTER_UPDATES
end)

local over = false
local fail_in = nil
local won_in = nil

function onSetup(director)
    director:set_sequence(SEQUENCE_START)
end

function onCommence(director)
    director:set_sequence(SEQUENCE_FIGHTING)
    BATTLE:spawn(director, { BNPC_ALLY })
    BATTLE:spawn(director, { BNPC_BOSS })
    BATTLE:steps(START_STEPS)
end

-- player_health: the player's health in percent (nil if they're not here)
function onUpdate(director, health, player_health, info)
    if fail_in ~= nil then
        fail_in = fail_in - 1
        if fail_in <= 0 then
            fail_in = nil
            director:fail_quest_battle()
        end
        return
    end
    if won_in ~= nil then
        won_in = won_in - 1
        if won_in <= 0 then
            won_in = nil
            director:set_sequence(SEQUENCE_WON)
            director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS,
                QUEST_VARS_AFTER_WIN)
        end
        return
    end
    if over then
        return
    end
    -- The lines on their own clocks (not the steps': the waves' come after whatever waits there.)
    for text, updates in pairs(talk_in) do
        if updates <= 1 then
            talk_in[text] = nil
            director:battle_talk(SPEAKER_ALLY.layout, SPEAKER_ALLY.name, text, SPEAKER_ALLY.icon)
        else
            talk_in[text] = updates - 1
        end
    end
    BATTLE:update(director, health, player_health, info)
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    -- Every wave has to have come (the last one is on a timer after the second.)
    local waves_done = BATTLE.alive[LAST_WAVE.adds[1]] ~= nil or BATTLE.alive[LAST_WAVE.adds[2]] ~= nil
    for _, id in ipairs(LAST_WAVE.adds) do
        for _, arrived in ipairs(BATTLE.arrived) do
            if arrived == id then
                waves_done = true
            end
        end
    end
    if waves_done and BATTLE:enemiesLeft() == 0 and BATTLE.alive[BNPC_BOSS] == nil then
        over = true
        BATTLE:clearSteps()
        won_in = WON_DELAY_UPDATES
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
