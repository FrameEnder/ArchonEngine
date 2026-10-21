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
--   the boss at 75% (75.3% and 73.5% the last health seen before it): sequence 3 and the first wave (4083606/07); the
--          ally's 83 seven seconds later (7.1, 7.0)
--   the boss at 45% (45.5% seen 5 s before it in the won run; 75.3% then 30.1% in the lost one): sequence 4 and the
--          second wave (4083608/09)
--   10 s after sequence 4 (10.0, 10.1): sequence 5 and the third wave (4083610/11) with the ally's 85
--   the player below 80% (78%, 79%): the ally's 84; below 40%: 86 (0.5 s and 4 s after it in the two runs, so a
--          guess); below 15% (14%, 2%): 87; once each
--   everyone down: 2 s later (the last one's death animation, as retail waits) sequence 6, "Duty Complete", the quest
--          battle's scene 6 (0x2001) and back out; the quest moves to sequence 2 with UI8BH = 1 (ITEM0) in the same
--          update
--   the player down (the lost run): sequence 0xFE, and out 6 s later; the quest stays as it was
-- As on retail (both runs): the boss fights the ally the whole time and the waves go after you; the ally stays on the
-- boss, then takes the adds one by one, within 3 yalms of you, and casts Cure (120) on you (at 17-48%, every 12-20 s.)
-- How hard each hits is in their timelines' recorded_amounts (QB35*_<base>.json).

-- Where retail puts the player and which way they face: their SpawnPlayer in the capture.
START_POSITION = { x = -58.21, y = 25.26, z = -135.32 }
START_ROTATION = 0.122

BNPC_ALLY = 4083622
BNPC_BOSS = 4083604
WAVES = {
    { health = 65, sequence = 3, adds = { 4083606, 4083607 } },
    { health = 41, sequence = 4, adds = { 4083608, 4083609 } },
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
-- The ally's lines as the player's health drops: below this percent, once each.
PLAYER_HEALTH_TALK = {
    { below = 80, text = 84 },
    { below = 40, text = 86 },
    { below = 15, text = 87 },
}
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
    keep_near = { within = 3 },
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
        else
            BATTLE:steps({ { LAST_WAVE_AFTER_UPDATES, "call", lastWave } })
        end
    end)
end

local over = false
local fail_in = nil
local won_in = nil
local said = {}

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
    if player_health ~= nil then
        for i, line in ipairs(PLAYER_HEALTH_TALK) do
            if not said[i] and player_health < line.below then
                said[i] = true
                director:battle_talk(SPEAKER_ALLY.layout, SPEAKER_ALLY.name, line.text, SPEAKER_ALLY.icon)
            end
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
