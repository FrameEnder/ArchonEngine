-- Quest battle: Duty, Honor, Country, for the quest of the same name (ManWil007_00551.)
-- It takes place in a copy of Central Thanalan (territory 271, the same LGBs as 141), so its BNpcs live in the wil_w1
-- drop-in (layer QB_ManWil007_001). Layout IDs, levels, HP, positions, display flags and the start position are
-- retail's: Bob's Chronofoil capture of 2026-10-18, three runs (`questkit chronofoil --battle 0.36`, `0.38`, `0.40`:
-- two lost, as Papashan and then Owyne fell, and one won), with the Kawari Recorder's files of the same runs.
--
-- Timed against those runs (seconds from the fight starting, after the opening cutscene):
--   0      director sequence 2 (sequence 1 is set before the opening cutscene): Papashan, Owyne and two Sultansworn
--          elites with you against Garibald, three bowmen and three fistfighters; nine pairs of fighters further
--          off fight each other (scenery: 26400 HP each, display flags 0x40208). Papashan: battle talk 149 at 5,
--          Owyne 151 at 10
--   Garibald at 70%: sequence 3 and six more (two fistfighters, two bowmen, a spellweaver and a cursedealer) half
--          a second later; Papashan 155 eight seconds after the sequence (8.07 s in all three runs)
--   Garibald at 30%, or at 40% once no more than one of the others is standing: sequence 4, Garibald 157. (One run
--          had it as he went from 30.2% to 29.9% with six still standing, the other at 36.5% just as the second
--          last fell; the run lost at 50% never had it. A guess that fits all three.)
--   Garibald at about 27%: he casts Void Call (2.7 s; 0.7 s after he passed 27% in both runs); a second after it
--          lands, sequence 5 and his 158, and the blanga it called half a second later (it goes after you); Owyne
--          156 five seconds after the sequence
--   Garibald at 10%: Papashan 150
--   everyone down (the blanga last): sequence 6, everyone else leaves and the director's cutscene plays (scene 6,
--          0xF8402EFB); retail sends you on through a loading screen (PrepareZoning to the same territory, then
--          ActorSetPos 1.5 s later)
--   after it: sequence 7, a lesser gargoyle (after you) and the masked mage (spawned not hostile, standing by
--          until hit) half a second later; the mage 165 five seconds in
--   22 seconds in: sequence 8, the mage 166 five seconds later
--   the gargoyle at 50%: the mage 168
--   the gargoyle at 30% (it went from 30.5% to 28.0%): sequence 9 and Thancred; ten seconds later sequence 10 and
--          his 169, the mage 167 five seconds after that. He takes the gargoyle and then the mage
--   the mage at 40%: the mage 170, Thancred 171 five seconds later
--   the mage down: two seconds later (its death animation) sequence 11, "Duty Complete", the quest battle's scene 11
--          (0x2001) and back out; the quest moves to sequence 3
-- Losing: Papashan or Owyne falling loses the battle (the moment their health reaches 0: sequence 0xFE, with their
-- line, 160 or 161), as does your falling; out about 6 s later. A Sultansworn elite (4087459) falling only says 162.
--
-- Who goes after whom (every run alike): Garibald after Papashan, the bowmen after Owyne, the fistfighters 4087449
-- and 4087452 after one elite (4087459) and 4087451 after the other; of the second six, 4087504 goes after 4087460,
-- 4087502 after 4087459, the bowmen after Owyne and the two casters after Papashan. Papashan stays on Garibald, Owyne
-- takes the bowmen, the elites the fistfighters, then everyone what's left, Garibald last. Papashan, Owyne and
-- 4087459 Cure the others (never you) when they're down to about two thirds. How hard everyone hits and heals is in
-- their timelines' recorded_amounts.

-- Where retail puts the player and which way they face: their SpawnPlayer in the capture.
START_POSITION = { x = 247.74, y = -17.58, z = -131.33 }
START_ROTATION = 1.351

BNPC_PAPASHAN = 4087433
BNPC_OWYNE = 4087434
BNPC_ELITE_1 = 4087459
BNPC_ELITE_2 = 4087460
BNPC_GARIBALD = 4087437
BNPC_BLANGA = 4087563
BNPC_GARGOYLE = 4087572
BNPC_MAGE = 4087573
BNPC_THANCRED = 4087577

FIRST_BOWMEN = { 4087440, 4087443, 4087445 }
FIRST_FISTFIGHTERS = { 4087449, 4087451, 4087452 }
SECOND_WAVE = { 4087493, 4087494, 4087498, 4087499, 4087502, 4087504 }
SECOND_BOWMEN = { 4087498, 4087499 }
SECOND_FISTFIGHTERS = { 4087502, 4087504 }
-- The fights in the background, each pair as the layer has them (INIT_NOTAR_P_POP_1ST_n against INIT_NOTAR_POP_1ST_n,
-- and as retail had them fighting): { the player's side, the other side }.
BACKGROUND = {
    { 4089037, 4089052 }, { 4089038, 4089055 }, { 4089039, 4089053 }, { 4089040, 4089054 }, { 4089041, 4089050 },
    { 4089043, 4089051 }, { 4089044, 4089048 }, { 4089045, 4089049 }, { 4089046, 4089047 },
}
ALLIES_FIRST_HALF = { BNPC_PAPASHAN, BNPC_OWYNE, BNPC_ELITE_1, BNPC_ELITE_2 }

ENEMY_TARGETS = {
    [BNPC_GARIBALD] = BNPC_PAPASHAN,
    [4087440] = BNPC_OWYNE, [4087443] = BNPC_OWYNE, [4087445] = BNPC_OWYNE,
    [4087449] = BNPC_ELITE_1, [4087452] = BNPC_ELITE_1, [4087451] = BNPC_ELITE_2,
    [4087504] = BNPC_ELITE_2, [4087502] = BNPC_ELITE_1, [4087498] = BNPC_OWYNE, [4087499] = BNPC_OWYNE,
    [4087493] = BNPC_PAPASHAN, [4087494] = BNPC_PAPASHAN,
}

-- Who's talking, by BNpcName, and their portrait in the battle talk window (retail's DirectorPopupMessage.)
SPEAKER_PAPASHAN = { layout = BNPC_PAPASHAN, name = 935, icon = 0x11e21 }
SPEAKER_OWYNE = { layout = BNPC_OWYNE, name = 936, icon = 0x11e22 }
SPEAKER_GARIBALD = { layout = BNPC_GARIBALD, name = 949, icon = 0x11e23 }
SPEAKER_ELITE = { layout = BNPC_ELITE_1, name = 970, icon = 0x11e24 }
SPEAKER_MAGE = { layout = BNPC_MAGE, name = 617, icon = 0x11e12 }
SPEAKER_THANCRED = { layout = BNPC_THANCRED, name = 713, icon = 0x11dcd }

-- The quest moves to this sequence when the battle is won; retail then plays the quest battle's scene 11.
QUEST_SEQUENCE_AFTER_WIN = 3
FINISH_SCENE = 11
CUTSCENE_SCENE = 6
CUTSCENE_FLAGS = 0xF8402EFB
-- Where the cutscene leaves you, as retail's ActorSetPos put it.
AFTER_CUTSCENE_POSITION = { x = 258.39, y = -19.72, z = -129.58 }
AFTER_CUTSCENE_ROTATION = 1.288
SEQUENCE_START = 1
SEQUENCE_FAILED = 0xFE
FAIL_DELAY_UPDATES = 12
WON_DELAY_UPDATES = 4

ACTION_VOID_CALL = 867
ACTION_CURE = 120
BACKGROUND_ENMITY = 100000

-- Steps: { updates after the step before, kind, ... } (Battle:steps)
START_STEPS = {
    { 0, "sequence", 2 },
    { 10, "talk", SPEAKER_PAPASHAN, 149 },
    { 10, "talk", SPEAKER_OWYNE, 151 },
}
SECOND_WAVE_STEPS = {
    { 0, "sequence", 3 },
    { 1, "spawn", SECOND_WAVE },
    { 15, "talk", SPEAKER_PAPASHAN, 155 },
}
SEQUENCE_4_STEPS = { { 0, "sequence", 4 }, { 0, "talk", SPEAKER_GARIBALD, 157 } }
VOID_CALL_STEPS = {
    { 0, "call", function(director)
        if BATTLE.alive[BNPC_GARIBALD] then
            director:use_action(BNPC_GARIBALD, ACTION_VOID_CALL)
        end
    end },
    { 8, "sequence", 5 },
    { 0, "talk", SPEAKER_GARIBALD, 158 },
    { 1, "spawn", { BNPC_BLANGA } },
    { 9, "talk", SPEAKER_OWYNE, 156 },
}
SEQUENCE_4_HEALTH = 30
SEQUENCE_4_HEALTH_FEW_LEFT = 40
FEW_LEFT = 1
VOID_CALL_HEALTH = 27
PAPASHAN_LINE_HEALTH = 10
AFTER_CUTSCENE_STEPS = {
    { 0, "sequence", 7 },
    { 1, "spawn", { BNPC_GARGOYLE, BNPC_MAGE } },
    { 9, "talk", SPEAKER_MAGE, 165 },
    { 34, "sequence", 8 },
    { 10, "talk", SPEAKER_MAGE, 166 },
}
GARGOYLE_PHASES = {
    { health = 50, steps = { { 0, "talk", SPEAKER_MAGE, 168 } } },
    {
        health = 30,
        steps = {
            { 0, "sequence", 9 },
            { 1, "spawn", { BNPC_THANCRED } },
            { 19, "sequence", 10 },
            { 0, "talk", SPEAKER_THANCRED, 169 },
            { 10, "talk", SPEAKER_MAGE, 167 },
        },
    },
}
MAGE_PHASES = {
    { health = 40, steps = { { 0, "talk", SPEAKER_MAGE, 170 }, { 10, "talk", SPEAKER_THANCRED, 171 } } },
}
-- Allies whose fall loses the battle, and the line each says as it does; the elite's only says it.
FALLING_LINES = {
    [BNPC_PAPASHAN] = { speaker = SPEAKER_PAPASHAN, text = 160, loses = true },
    [BNPC_OWYNE] = { speaker = SPEAKER_OWYNE, text = 161, loses = true },
    [BNPC_ELITE_1] = { speaker = SPEAKER_ELITE, text = 162 },
}

-- The battle's outline (content/lib/Battle.lua), as retail ran it.
BATTLE = Battle.new()
-- Papashan holds Garibald the whole time (and Garibald him), then helps with what's left.
BATTLE:ally(BNPC_PAPASHAN, { focus = { "boss", "adds" }, range = 3 })
-- Owyne takes the bowmen, the elites the fistfighters (the first the oldest, the second the next), then the rest.
BATTLE:ally(BNPC_OWYNE, { focus = { "bowmen", "adds", "boss" }, order = "oldest", range = 3 })
BATTLE:ally(BNPC_ELITE_1, { role = "dps", focus = { "fistfighters", "adds", "boss" }, order = "oldest", range = 3 })
BATTLE:ally(BNPC_ELITE_2, { role = "dps", focus = { "fistfighters", "adds", "boss" }, order = "oldest", range = 3 })
-- Thancred takes the gargoyle and then the mage.
BATTLE:ally(BNPC_THANCRED, { focus = { "adds", "boss" }, range = 3 })
-- Their Cures (retail: on each other at 35-70% health, about 10 each a run, never on the player or, but for the
-- first elite, themselves.)
HEALS = {
    [BNPC_PAPASHAN] = { targets = { BNPC_OWYNE, BNPC_ELITE_1, BNPC_ELITE_2 }, below = 66 },
    [BNPC_OWYNE] = { targets = { BNPC_ELITE_1, BNPC_PAPASHAN, BNPC_ELITE_2 }, below = 69 },
    [BNPC_ELITE_1] = { targets = { BNPC_OWYNE, BNPC_ELITE_2, "self", BNPC_PAPASHAN }, below = 70 },
}
HEAL_EVERY_SECONDS = 8
for healer, h in pairs(HEALS) do
    BATTLE.roles[healer].heals = {
        { action = ACTION_CURE, targets = h.targets, below = h.below, every = HEAL_EVERY_SECONDS },
    }
end

BATTLE:enemy(BNPC_GARIBALD, { group = "boss", attacks = ENEMY_TARGETS[BNPC_GARIBALD] })
for _, id in ipairs(FIRST_BOWMEN) do
    BATTLE:enemy(id, { group = { "bowmen", "adds" }, attacks = ENEMY_TARGETS[id] })
end
for _, id in ipairs(SECOND_BOWMEN) do
    BATTLE:enemy(id, { group = { "bowmen", "adds" }, attacks = ENEMY_TARGETS[id] })
end
for _, id in ipairs(FIRST_FISTFIGHTERS) do
    BATTLE:enemy(id, { group = { "fistfighters", "adds" }, attacks = ENEMY_TARGETS[id] })
end
for _, id in ipairs(SECOND_FISTFIGHTERS) do
    BATTLE:enemy(id, { group = { "fistfighters", "adds" }, attacks = ENEMY_TARGETS[id] })
end
-- The spellweaver Cures its side (itself at 2% once; 15 Cures in three runs, at 2-50%.)
BATTLE:enemy(4087493, { group = "adds", attacks = ENEMY_TARGETS[4087493], range = 15,
    heals = { { action = ACTION_CURE, targets = "allies", below = 50, every = 6 } } })
BATTLE:enemy(4087494, { group = "adds", attacks = ENEMY_TARGETS[4087494], range = 15 })
BATTLE:enemy(BNPC_BLANGA, { group = "adds" })
BATTLE:enemy(BNPC_GARGOYLE, { group = "adds" })
-- The mage stands by until hit (retail: until Thancred reached it, a minute in), then goes after whoever did.
BATTLE:enemy(BNPC_MAGE, { group = "boss", passive = true })

BATTLE:onHealth(BNPC_GARIBALD, 70, function()
    BATTLE:steps(SECOND_WAVE_STEPS)
end)
BATTLE:onHealth(BNPC_GARIBALD, PAPASHAN_LINE_HEALTH, function()
    BATTLE:steps({ { 0, "talk", SPEAKER_PAPASHAN, 150 } })
end)
for _, phases in ipairs({ { BNPC_GARGOYLE, GARGOYLE_PHASES }, { BNPC_MAGE, MAGE_PHASES } }) do
    for _, phase in ipairs(phases[2]) do
        BATTLE:onHealth(phases[1], phase.health, function()
            BATTLE:steps(phase.steps)
        end)
    end
end

local half = 1
local sequence_4 = false
local void_call = false
local won_in = nil
local over = false
local fail_in = nil

function onSetup(director)
    director:set_sequence(SEQUENCE_START)
end

function onCommence(director)
    -- The background fights aren't part of the battle: nobody here goes after them.
    for _, pair in ipairs(BACKGROUND) do
        director:spawn_bnpc(pair[1])
        director:spawn_bnpc(pair[2])
        director:set_hate(pair[1], pair[2], BACKGROUND_ENMITY)
        director:set_hate(pair[2], pair[1], BACKGROUND_ENMITY)
    end
    BATTLE:spawn(director, ALLIES_FIRST_HALF)
    BATTLE:spawn(director, { BNPC_GARIBALD })
    BATTLE:spawn(director, FIRST_FISTFIGHTERS)
    BATTLE:spawn(director, FIRST_BOWMEN)
    BATTLE:steps(START_STEPS)
end

-- Sequence 4: Garibald at 30%, or at 40% with no more than one of the others standing (see the header); then, at
-- 27%, Void Call. Both come (in that order) if he falls before them.
function checkGaribald(director, health)
    if half ~= 1 or BATTLE.arrival[BNPC_GARIBALD] == nil or void_call then
        return
    end
    local garibald = BATTLE.alive[BNPC_GARIBALD] and health[BNPC_GARIBALD] or 0
    if not sequence_4 then
        local others = BATTLE:enemiesLeft() - (BATTLE.alive[BNPC_GARIBALD] and 1 or 0)
        if garibald <= SEQUENCE_4_HEALTH or (garibald <= SEQUENCE_4_HEALTH_FEW_LEFT and others <= FEW_LEFT) then
            sequence_4 = true
            BATTLE:steps(SEQUENCE_4_STEPS)
        end
    end
    if sequence_4 and garibald <= VOID_CALL_HEALTH then
        void_call = true
        BATTLE:steps(VOID_CALL_STEPS)
    end
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
    if won_in ~= nil then
        won_in = won_in - 1
        if won_in <= 0 then
            won_in = nil
            director:set_sequence(11)
            director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE)
        end
        return
    end
    if over then
        return
    end
    BATTLE:update(director, health, player_health, info)
    checkGaribald(director, health or {})
end

-- Everyone from the first half leaves for the cutscene.
function clearFirstHalf(director)
    for _, pair in ipairs(BACKGROUND) do
        director:despawn_bnpc(pair[1])
        director:despawn_bnpc(pair[2])
    end
    BATTLE:remove(director, ALLIES_FIRST_HALF)
end

function lose(director)
    over = true
    BATTLE:clearSteps()
    director:set_sequence(SEQUENCE_FAILED)
    fail_in = FAIL_DELAY_UPDATES
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    local falling = FALLING_LINES[layout_id]
    if falling ~= nil and half == 1 and BATTLE.alive[layout_id] then
        BATTLE:died(director, layout_id)
        director:battle_talk(falling.speaker.layout, falling.speaker.name, falling.text, falling.speaker.icon)
        if falling.loses then
            lose(director)
        end
        return
    end
    if not BATTLE:died(director, layout_id) then
        return
    end

    if half == 1 and BATTLE:enemiesLeft() == 0 and BATTLE.arrival[BNPC_BLANGA] and BATTLE.arrival[SECOND_WAVE[1]] then
        -- Garibald, both waves and the blanga are down.
        half = 0
        BATTLE:clearSteps()
        director:set_sequence(6)
        clearFirstHalf(director)
        director:play_scene(CUTSCENE_SCENE, CUTSCENE_FLAGS)
        return
    end

    if layout_id == BNPC_MAGE then
        over = true
        BATTLE:clearSteps()
        -- retail: "Duty Complete" came 2 s after the mage's health reached 0, as its death animation ended.
        won_in = WON_DELAY_UPDATES
    end
end

function onSceneFinished(director, scene)
    if scene ~= CUTSCENE_SCENE or half ~= 0 then
        return
    end
    half = 2
    -- Retail ends the cutscene with a loading screen into where the second half starts; the client waits for it.
    director:warp_players(AFTER_CUTSCENE_POSITION.x, AFTER_CUTSCENE_POSITION.y, AFTER_CUTSCENE_POSITION.z,
        AFTER_CUTSCENE_ROTATION)
    BATTLE:steps(AFTER_CUTSCENE_STEPS)
end

-- Losing sends you back to where you entered, still on the same step of the quest, so you can try again.
function onPlayerDeath(director)
    if over or half == 0 then
        return
    end
    lose(director)
end
