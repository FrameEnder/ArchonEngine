-- Quest battle: Trial by Water, for the conjurer quest of the same name (ClsCnj003_00092.)
-- It takes place in territory 234, a copy of the East Shroud (retail's positions in it are the East Shroud's), so its
-- BNpcs live in the fst_f1f2 drop-in (layer QB_ClsCnj003_002): three Wood Wailer lancers (base 1423, level 15,
-- 1244 HP) and an archer (base 1422, 995 HP) on your side, three waves of yellowjackets (base 298, level 15, 606 HP)
-- and water sprites (base 1681, level 14, 146 HP) on the other. Layout IDs, levels, HP, positions, display flags and
-- the start spot are retail's (Bob's Chronofoil capture b23b91ef of 2026-10-26, `questkit chronofoil --battle 0.18`,
-- lost doing nothing, and `0.20`, won; a level 19 conjurer); the layout IDs are Sapphire's too (INIT_POP_*.)
--
-- Timed against both runs (seconds from the fight starting, the director's scene 60000, after the opening cutscene):
--   0     sequence 2; everyone is there, each yellowjacket on its own lancer
--   5.1   battle talk 127 (BNpcName 628, no portrait, from no one in the battle) and sequence 3
--   one enemy left (30.4 s in the won run, 49.1 s in the lost one): sequence 4, the second wave (three yellowjackets,
--         one for each lancer, and a water sprite, on you); the archer's battle talk 128 (BNpcName 621) 5 s later
--   one enemy left again (72.5 s, 108.7 s): sequence 5, the third wave (three yellowjackets and two water sprites);
--         the archer's 129 5 s later
--   all of them down: sequence 6, a second later "Duty Complete" (60001) and sequence 7, then the battle's scene 7
--         (a cutscene, 0xF8402FFF); the quest moves to sequence 7
--   you down: sequence 0xFE (the lancers falling doesn't lose it: two of them fell in the lost run)
-- Each lancer yells once as his health drops to 43% (4263914 NpcYell 967, the other two 966, half a second after the
-- hit: 43.0% yelled, 43.2% didn't, in five hits across the runs.) The waves come by how many are left, not by time:
-- the runs were 18 and 36 s apart by the second wave, and both had exactly one enemy standing.
-- The lancers fight the yellowjacket that came for them and then help the others (the nearest); the archer shoots
-- what 3905949's yellowjackets are, then the nearest. How hard each hits is in their timelines (QB16*.)

-- Where retail puts the player and which way they face: their SpawnPlayer in the capture.
START_POSITION = { x = -51.95, y = -9.75, z = 232.86 }
START_ROTATION = -3.142
-- Retail's opening cutscene's flags here (both runs.)
OPENING_SCENE_FLAGS = 0xF841AFFF

BNPC_LANCER_A = 3905949
BNPC_LANCER_B = 4263914
BNPC_LANCER_C = 4263915
BNPC_ARCHER = 3914511
-- Each wave: the yellowjacket for each lancer (by the lancer's group), and the water sprites (after you.)
WAVES = {
    { a = 3875540, b = 3879109, c = 3905947, sprites = {} },
    { a = 3905948, b = 4263917, c = 4263918, sprites = { 4263967 }, sequence = 4, talk = 128 },
    { a = 4263934, b = 4263935, c = 4263936, sprites = { 4263969, 4263970 }, sequence = 5, talk = 129 },
}
-- The next wave comes when this many enemies are left standing.
NEXT_WAVE_AT = 1

-- The quest moves to this sequence when the battle is won.
QUEST_SEQUENCE_AFTER_WIN = 7
SEQUENCE_START = 1
SEQUENCE_FIGHTING = 2
SEQUENCE_FIRST_LINE = 3
SEQUENCE_CLEARED = 6
SEQUENCE_WON = 7
SEQUENCE_FAILED = 0xFE
-- After the last one falls, "Duty Complete" and sequence 7 a second later (two updates.)
WON_DELAY_UPDATES = 2
FINISH_SCENE = 7
FINISH_SCENE_FLAGS = 0xF8402FFF
FAIL_DELAY_UPDATES = 12

-- Battle talk (DirectorPopupMessage): BNpcName and the content's text IDs, as retail sent them. The first line's
-- speaker isn't in the battle (no actor, BNPC_NAME_01 in Sapphire's script); the others are the archer's.
SPEAKER_START = { layout = 0, name = BNPC_NAME_01 or 628, icon = 0 }
SPEAKER_ARCHER = { layout = BNPC_ARCHER, name = 621, icon = 0 }
START_STEPS = {
    { 10, "talk", SPEAKER_START, 127 },
    { 0, "sequence", SEQUENCE_FIRST_LINE },
}
LINE_DELAY_UPDATES = 10

-- The lancers' yells (NpcYell, a speech bubble) as their health drops to this, half a second later.
YELL_BELOW = 43
YELLS = { [BNPC_LANCER_A] = 966, [BNPC_LANCER_B] = 967, [BNPC_LANCER_C] = 966 }

BATTLE = Battle.new()
local LANCER_GROUPS = { [BNPC_LANCER_A] = "a", [BNPC_LANCER_B] = "b", [BNPC_LANCER_C] = "c" }
for lancer, group in pairs(LANCER_GROUPS) do
    BATTLE:ally(lancer, { role = "dps", split = false, focus = { group, "adds" }, order = "nearest", range = 3 })
    BATTLE:onHealth(lancer, YELL_BELOW, function()
        BATTLE:steps({ { 1, "yell", lancer, YELLS[lancer] } })
    end)
end
BATTLE:ally(BNPC_ARCHER, { role = "dps", split = false, focus = { "a", "adds" }, order = "nearest", range = 25 })

local next_wave = 1
local over = false
local fail_in = nil

-- A wave's enemies, described and spawned: each yellowjacket after its lancer, the sprites after you.
function spawnWave(director, wave)
    local ids = {}
    for group, lancer in pairs({ a = BNPC_LANCER_A, b = BNPC_LANCER_B, c = BNPC_LANCER_C }) do
        BATTLE:enemy(wave[group], { group = { group, "adds" }, attacks = lancer })
        table.insert(ids, wave[group])
    end
    table.sort(ids)
    for _, id in ipairs(wave.sprites) do
        BATTLE:enemy(id, { group = "adds" })
        table.insert(ids, id)
    end
    BATTLE:spawn(director, ids)
    next_wave = next_wave + 1
end

function onSetup(director)
    director:set_sequence(SEQUENCE_START)
end

function onCommence(director)
    director:set_sequence(SEQUENCE_FIGHTING)
    BATTLE:spawn(director, { BNPC_LANCER_A, BNPC_LANCER_B, BNPC_LANCER_C, BNPC_ARCHER })
    spawnWave(director, WAVES[1])
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
        BATTLE:runPending(director)
        return
    end
    BATTLE:update(director, health, player_health, info)
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    local left = BATTLE:enemiesLeft()
    local wave = WAVES[next_wave]
    if wave ~= nil and left <= NEXT_WAVE_AT then
        director:set_sequence(wave.sequence)
        spawnWave(director, wave)
        BATTLE:steps({ { LINE_DELAY_UPDATES, "talk", SPEAKER_ARCHER, wave.talk } })
    elseif wave == nil and left == 0 then
        over = true
        BATTLE:clearSteps()
        director:set_sequence(SEQUENCE_CLEARED)
        BATTLE:steps({
            { WON_DELAY_UPDATES, "sequence", SEQUENCE_WON },
            { 0, "call", function(d)
                d:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS)
            end },
        })
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
