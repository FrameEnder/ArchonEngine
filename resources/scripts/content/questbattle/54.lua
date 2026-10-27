-- Quest battle: Trial by Wind, for the conjurer quest of the same name (ClsCnj002_00091.)
-- It takes place in territory 289, a copy of the East Shroud (retail's positions in it are the East Shroud's, by the
-- corrupted air the quest sends you to), so its BNpcs live in the fst_f1f2 drop-in (layer QB_ClsCnj002_002): the
-- thing at the middle you're there to destroy (4261180, base 1662, level 10, 768 HP), the wind sprites (base 1659,
-- level 8, 31 HP), the conjurer who comes to help (4248519, base 1661, level 15, 638 HP) and a last arrival (4248516,
-- base 1660, level 10, 172 HP). Layout IDs, levels, HP, positions, display flags and the start spot are retail's (Bob's
-- Chronofoil capture b23b91ef of 2026-10-26, `questkit chronofoil --battle 0.11`, one run, won, a level 14 conjurer.)
--
-- Timed against that run (seconds from entering; the fight starts at 1.1, the director's scene 60000):
--   0.7   director sequence 1, then scene 1 (flags 0x2001)
--   1.1   the fight starts: sequence 2; the target and one wind sprite (4248510) are there
--   15.0  the target at 75% (it went from 78% to 71%): sequence 3, three more wind sprites (4261182, 4261181 and
--         another from 4248510's layout ID: retail spawned that one twice, both standing until the end)
--   25.1  the target at 40% (46% to 39%): sequence 4, the conjurer arrives
--   30.0  the target at 25% (31% to 24%): sequence 5, 4248516 arrives
--   33.2  the conjurer's battle talk 107 (BNpcName 1419, no portrait), 38.2 his 108
--   44.3  the target at 0: sequence 6; 45.3 "Duty Complete" (60001) and sequence 7; it falls, and every enemy still
--         standing is gone with it
--   48.9  the battle's scene 7 (a cutscene, 0xF8402EFB); the quest moves to sequence 5
-- The target never acts or moves; the wind sprites stay where they appear and cast 969 at you every 3.1 s. The
-- conjurer goes for the wind sprite nearest him (Aero every 6 s), then the target. How hard each hits is in their
-- timelines (WindSprite_1659, TrialByWindAdd_1660, TrialByWindConjurer_1661.)

-- Where retail puts the player and which way they face: their SpawnPlayer in the capture.
START_POSITION = { x = -509.7, y = 6.58, z = 39.45 }
START_ROTATION = -3.021
-- Retail's opening scene 1 here is no cutscene: HIDE_HOTBAR and NO_DEFAULT_CAMERA (0x2001), over in 0.1 s.
OPENING_SCENE_FLAGS = 0x2001

BNPC_TARGET = 4261180
BNPC_CONJURER = 4248519
BNPC_SPRITE = 4248510
BNPC_LAST = 4248516
WAVES = {
    -- `again`: layout IDs spawned a second time (another actor from the same layout ID, as retail did.)
    { health = 75, sequence = 3, enemies = { 4261182, 4261181 }, again = { BNPC_SPRITE } },
    { health = 40, sequence = 4, allies = { BNPC_CONJURER } },
    { health = 25, sequence = 5, enemies = { BNPC_LAST } },
}

-- The quest moves to this sequence when the battle is won.
QUEST_SEQUENCE_AFTER_WIN = 5
SEQUENCE_START = 1
SEQUENCE_FIGHTING = 2
SEQUENCE_DESTROYED = 6
SEQUENCE_WON = 7
SEQUENCE_FAILED = 0xFE
-- After the target falls, "Duty Complete" and sequence 7 a second later (two updates.)
WON_DELAY_UPDATES = 2
-- The scene retail plays after "Duty Complete", and its flags (a cutscene); the player leaves when it ends.
FINISH_SCENE = 7
FINISH_SCENE_FLAGS = 0xF8402EFB
-- How many updates (half a second each) to wait after the player falls before sending them out.
FAIL_DELAY_UPDATES = 12

-- The conjurer's battle talk (DirectorPopupMessage): his BNpcName, no portrait, the content's text IDs, as retail sent
-- them, 3 s after sequence 5 and 5 s apart.
SPEAKER_CONJURER = { layout = BNPC_CONJURER, name = 1419, icon = 0 }
LAST_WAVE_STEPS = {
    { 6, "talk", SPEAKER_CONJURER, 107 },
    { 10, "talk", SPEAKER_CONJURER, 108 },
}

-- The battle's outline (content/lib/Battle.lua). The conjurer plays a ranged DPS: the wind sprites first, the nearest
-- one, then the target.
BATTLE = Battle.new()
BATTLE:ally(BNPC_CONJURER, { role = "dps", split = false, focus = { "adds", "boss" }, order = "nearest", range = 12 })
-- The target stands there: the director holds it (no noticing, chasing or timeline), so it only takes hits.
BATTLE:enemy(BNPC_TARGET, { group = "boss", passive = true })

local over = false
local fail_in = nil
-- The wind sprites spawned a second time from a layout ID, by actor ID (spawn_bnpc), to send away at the end.
local again = {}

function onSetup(director)
    director:set_sequence(SEQUENCE_START)
end

function onCommence(director)
    director:set_sequence(SEQUENCE_FIGHTING)
    BATTLE:spawn(director, { BNPC_TARGET, BNPC_SPRITE })
    director:hold_bnpc(BNPC_TARGET)
end

for _, wave in ipairs(WAVES) do
    BATTLE:onHealth(BNPC_TARGET, wave.health, function(director)
        if over then
            return
        end
        director:set_sequence(wave.sequence)
        if wave.allies ~= nil then
            BATTLE:spawn(director, wave.allies)
        end
        if wave.enemies ~= nil then
            BATTLE:spawn(director, wave.enemies)
        end
        for _, layout in ipairs(wave.again or {}) do
            local actor = director:spawn_bnpc(layout)
            director:set_hate(actor, nil, Battle.STARTING_ENMITY)
            table.insert(again, actor)
        end
        if wave.sequence == 5 then
            BATTLE:steps(LAST_WAVE_STEPS)
        end
    end)
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
    -- After the win only the closing steps run.
    if over then
        BATTLE:runPending(director)
        return
    end
    BATTLE:update(director, health, player_health, info)
end

-- Everyone on the other side still standing goes when the target falls, as on retail.
function removeEnemies(director)
    local standing = {}
    for layout, role in pairs(BATTLE.roles) do
        if role.side == "enemy" and BATTLE.alive[layout] then
            table.insert(standing, layout)
        end
    end
    BATTLE:remove(director, standing)
    for _, actor in ipairs(again) do
        director:despawn_bnpc(actor)
    end
    again = {}
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    if layout_id == BNPC_TARGET then
        over = true
        BATTLE:clearSteps()
        director:set_sequence(SEQUENCE_DESTROYED)
        removeEnemies(director)
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
