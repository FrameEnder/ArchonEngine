-- Quest battle 86 (GaiUsc208_00967, History Repeating, the main scenario's level 42 quest in Western La Noscea.)
-- It takes place in territory 330, whose ground is Western La Noscea's (s1f4: the run's spots stand on it, 13 yalms
-- from an open-world enemy of that zone's drop-in), so its NPCs are layer QB_GaiUsc208_001 of sea_s1f4. Layout IDs,
-- levels, HP, positions, flags, looks and the start spot are retail's (Bob's Chronofoil captures d7993cf0, 195176f8
-- and 0d10bf45 of 2026-11-08, `questkit chronofoil --battle 0.1` of each: three runs, all won, level 44 players.)
-- No ally fights beside you: one who talks (4294351, base 1632, level 50) stands on the rise behind the enemies,
-- untargetable, healing them; the director's helper (4317701, base 2201) stands unseen beside you.
--
-- Retail's flow, the same in every run:
--   the opening scene 1 (0xF8402EF3); then the director's scene 2 (0x2001, 10.1 s) and sequence 2, talk 81 5.1 s in
--           and the one who talks throwing 1329 at the helper 9.1 s in; scene 3 (0x2001, 10.1 s) and sequence 3, talk
--           82 5.1 s in; the fight 1.0-1.1 s after scene 3 ended. Kawari has no scene 60000: onCommence (the
--           opening's end) plays scene 2, and onSceneFinished plays scene 3 and starts the fight as it ends (the
--           second's wait left out: the waves' timers already run up to half a second late each, on the tick.)
-- Then (seconds from the fight starting, the director's scene 60000):
--   0       sequence 4; the first wave (a lancer, two marauders) 1.4-1.5 s later; NpcYell 1321 at 4.0-4.1
--   a wave's last one's health at 0: the next sequence at once, and
--     sequence 5: the second wave (lancer, thaumaturge, marauder) 2.2-2.3 s later, talk 83 at 5.1, NpcYell 1342 at 10.0
--     sequence 6: talk 84 at 5.1, NpcYell 1343 at 10.1, the third wave (lancer, marauder, two 2200s) at 15.3-15.5
--     sequence 7: talks 85 and 86 at 5.0 and 10.0, the bombs from 10.2-10.5 (one every 10 s), the fourth wave
--             (lancer, thaumaturge, marauder) at 15.4; the one who talks stops healing
--     sequence 8: talk 87 at 5.1, NpcYell 1344 at 10.0, the last wave (2202 and two who run in from behind) at
--             15.3-15.5; the bombs' clock starts again, one 8.3-8.5 s after sequence 8 and every 8 s after that
--     sequence 9 (the last wave down): talk 88 2.1 s later; "Duty Complete" (60001) and sequence 10 7.1 s after
--             sequence 9, the battle's scene 10 (0x2001) after it; the quest moves to sequence 4 with ITEM0 gone
--   the one who talks heals its side (1070, 5.7 s, at one of them, with Regen) every 14-38 s in sequences 4-6
-- The bombs: which of three comes is random (11, 11 and 16 in the three runs, each layout often). The two of base
-- 1633 (4317589, 4317648) wait 5 s, then run one fixed loop each round the fight at 9 yalms a second (a loop of the
-- layout's that the client never sees: retail's positions every 0.3 s, the same each time) and blow (1069) when they
-- pass within 3.2-3.8 yalms of you; 1634 (4317650) waits 5 s and comes for you at 1 yalm a second, blowing within
-- 3.9-5.3 yalms. Each falls 1.0 s after its blast (Kill, by itself); nobody can target them.
-- Lost (no lost run recorded): your health at 0, sequence 0xFE and out 5 s later.

-- Where retail puts the player and which way they face: their spawn in every run.
START_POSITION = { x = -305.5, y = -41.82, z = 697.34 }
START_ROTATION = 0.0
OPENING_SCENE_FLAGS = 0xF8402EF3

BNPC_TALKER = 4294351
BNPC_HELPER = 4317701
WAVES = {
    { 4294343, 4294347, 4294348 },
    { 4294439, 4294440, 4294441 },
    { 4294752, 4294754, 4294750, 4294751 },
    { 4294755, 4294756, 4294758 },
    { 4294766, 4294764, 4294763 },
}
-- The bombs: how each moves (a loop of spots, or after the player), how fast and how close it blows.
BOMB_CREEPER = 4317650
BOMBS = {
    [4317589] = {
        speed = 9.0,
        reach = 3.5,
        loop = {
            { -308.6, -41.9, 722.1 }, { -322.4, -41.9, 714.4 }, { -325.8, -41.7, 707.5 }, { -319.8, -41.6, 697.3 },
            { -308.1, -41.4, 693.8 }, { -289.6, -41.5, 695.6 }, { -286.1, -41.9, 708.5 }, { -289.1, -41.9, 715.8 },
            { -308.0, -41.9, 719.0 },
        },
    },
    [4317648] = {
        speed = 9.0,
        reach = 3.5,
        loop = {
            { -327.6, -41.8, 709.6 }, { -326.8, -40.2, 660.1 }, { -284.6, -40.0, 659.5 }, { -287.9, -40.8, 679.1 },
            { -285.2, -41.1, 692.4 }, { -294.5, -41.9, 711.6 }, { -310.3, -41.9, 722.3 },
        },
    },
    [BOMB_CREEPER] = { speed = 1.0, reach = 4.5 },
}
BOMB_LAYOUTS = { 4317589, 4317648, BOMB_CREEPER }
-- Where each bomb comes out (its layout's spot in the drop-in.)
BOMB_SPAWNS = {
    [4317589] = { -307.973, -42.0, 724.804 },
    [4317648] = { -305.066, -42.0, 725.071 },
    [BOMB_CREEPER] = { -302.829, -42.0, 725.61 },
}
ACTION_BLAST = 1069
-- In updates: how long a bomb waits before it moves, and how long after its blast it falls.
BOMB_SETS_OFF = 10
BOMB_FALLS_AFTER = 2

QUEST_SEQUENCE_AFTER_WIN = 4
-- ITEM0 goes with the win (retail's UpdateQuest to sequence 4 had every variable 0.)
QUEST_VARS_AFTER_WIN = { [1] = 0 }
SEQUENCE_LEAD_IN = 2
SEQUENCE_LEAD_IN_2 = 3
SEQUENCE_FIGHTING = 4
SEQUENCE_CLEARED = 9
SEQUENCE_WON = 10
SEQUENCE_FAILED = 0xFE
LEAD_IN_SCENE = 2
LEAD_IN_SCENE_2 = 3
LEAD_IN_SCENE_FLAGS = 0x2001 -- HIDE_HOTBAR | NO_DEFAULT_CAMERA
FINISH_SCENE = 10
FINISH_SCENE_FLAGS = 0x2001
FAIL_DELAY_UPDATES = 10

-- What each wave's sequence brings, in updates: a timer of n set at a death fires (n - 1) / 2 s after it at the
-- soonest, up to half a second more by the tick (retail's waves came 15.3-15.5 s after; 31 lands there.)
AFTER_CLEARED = {
    [5] = { wave = 2, wave_after = 5, talks = { { 11, 83 } }, yells = { { 21, 1342 } } },
    [6] = { wave = 3, wave_after = 31, talks = { { 11, 84 } }, yells = { { 21, 1343 } } },
    [7] = { wave = 4, wave_after = 31, talks = { { 11, 85 }, { 21, 86 } }, bombs = { first = 22, every = 20 } },
    [8] = {
        wave = 5,
        wave_after = 31,
        talks = { { 11, 87 } },
        yells = { { 21, 1344 } },
        bombs = { first = 18, every = 16 },
    },
}
-- From the fight starting (set as scene 3 ends: n updates are (n - 1) / 2 s.)
FIRST_WAVE_AFTER = 4
FIRST_YELL_AFTER = 9
FIRST_YELL = 1321
TALK_81_AFTER = 11
THROW_AFTER = 19
TALK_82_AFTER = 11
TALK_88_AFTER = 5
WON_AFTER = 15
ACTION_THROW = 1329
-- Won, retail let the player out in Western La Noscea (138) at its pop range 4294805, a few yalms from the campfire
-- where they went in, not where they went in from: within a yalm of this spot in all three of Bob's runs (captures
-- d7993cf0, 195176f8, 0d10bf45: ZoneInit at -306.00 -41.48 688.96, -305.78 -41.47 687.88, -306.36 -41.48 689.19),
-- facing 0 (each client's first report there). Before 2026-11-11 it was back where they went in (Bob's bundle
-- 20261111-072620.)
EXIT_AFTER_WIN = { territory = 138, x = -306.05, y = -41.48, z = 688.68, rotation = 0.0 }
-- The one who talks goes into the fight as the lead-in begins (sequence 2), after the director's helper, and out of it
-- 1.5-3.8 s after sequence 7 begins, as it stops healing: its weapon drawn and its nameplate red, then put away
-- (`director:engage_bnpc`; all three of Bob's runs, captures d7993cf0, 195176f8 and 0d10bf45.)
DISENGAGE_AFTER = 7

SPEAKER = { layout = BNPC_TALKER, name = 1417, icon = 73188 }

-- Retail's area heal on its side: 654 a heal, aimed at one of them, every 14 s at the most (sequences 4-6.)
ACTION_HEAL = 1070

BATTLE = Battle.new()
BATTLE:enemy(BNPC_TALKER, {
    group = "talker",
    passive = true,
    unaware = true,
    heals = { { action = ACTION_HEAL, targets = "allies", below = 80, every = 14.0 } },
})
-- Every wave goes for you (there's no one else.)
for _, wave in ipairs(WAVES) do
    for _, layout in ipairs(wave) do
        BATTLE:enemy(layout, { group = "wave", attacks = nil })
    end
end

local over = false
local fail_in = nil
local phase = 1
local wave_left = {}
local timers = {}
-- The bombs out now: { id, layout, at = {x, y, z}, next (its next spot), age (updates), blasted (the update) }.
local bombs = {}
local bomb_clock = 0 -- which bomb schedule is running (a new one stops the old)
local updates = 0

function after(n, fn)
    table.insert(timers, { n, fn })
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

function talk(director, text)
    director:battle_talk(SPEAKER.layout, SPEAKER.name, text, SPEAKER.icon, nil, SPEAKER.kind)
end

function sequence(director, number)
    phase = number
    director:set_sequence(number)
end

function bringOut(director, wave)
    for _, layout in ipairs(WAVES[wave]) do
        wave_left[layout] = true
    end
    BATTLE:spawn(director, WAVES[wave])
end

-- ---- The bombs ----

function spawnBomb(director)
    local layout = BOMB_LAYOUTS[math.random(#BOMB_LAYOUTS)]
    local id = director:spawn_bnpc(layout)
    local spot = BOMB_SPAWNS[layout]
    -- Only the director moves them, has them blow and has them fall.
    director:hold_bnpc(id or layout)
    table.insert(bombs, { id = id or layout, layout = layout, at = { spot[1], spot[2], spot[3] }, next = 1, age = 0 })
end

-- One bomb every `every` updates from `first` on, until the fight is over or another schedule starts.
function bombSchedule(first, every)
    bomb_clock = bomb_clock + 1
    local clock = bomb_clock
    local function bring(director)
        if over or clock ~= bomb_clock then
            return
        end
        spawnBomb(director)
        after(every, bring)
    end
    after(first, bring)
end

local function flatDistance(a, x, z)
    return math.sqrt((a[1] - x) ^ 2 + (a[3] - z) ^ 2)
end

-- Moves the estimate of where a looping bomb is by one update, and sends it on to its next spot when it reaches one.
local function moveAlongLoop(director, bomb, kind)
    local left = kind.speed * 0.5
    local turned = bomb.age == BOMB_SETS_OFF
    while left > 0 do
        local spot = kind.loop[bomb.next]
        local d = math.sqrt((spot[1] - bomb.at[1]) ^ 2 + (spot[3] - bomb.at[3]) ^ 2)
        if d <= left then
            bomb.at = { spot[1], spot[2], spot[3] }
            left = left - d
            bomb.next = bomb.next % #kind.loop + 1
            turned = true
        else
            local f = left / d
            bomb.at = {
                bomb.at[1] + (spot[1] - bomb.at[1]) * f,
                bomb.at[2] + (spot[2] - bomb.at[2]) * f,
                bomb.at[3] + (spot[3] - bomb.at[3]) * f,
            }
            left = 0
        end
    end
    if turned then
        local spot = kind.loop[bomb.next]
        director:sweep_bnpc(bomb.id, spot[1], spot[2], spot[3], kind.speed)
    end
end

function updateBombs(director, info)
    local player = info ~= nil and info.player or nil
    local still = {}
    for _, bomb in ipairs(bombs) do
        local kind = BOMBS[bomb.layout]
        bomb.age = bomb.age + 1
        local keep = true
        if bomb.blasted ~= nil then
            if updates - bomb.blasted >= BOMB_FALLS_AFTER then
                director:kill_bnpc(bomb.id)
                keep = false
            end
        elseif bomb.age >= BOMB_SETS_OFF then
            if kind.loop ~= nil then
                moveAlongLoop(director, bomb, kind)
            elseif player ~= nil and player.x ~= nil then
                -- The creeper: towards wherever you are now.
                local d = flatDistance(bomb.at, player.x, player.z)
                if d > 0 then
                    local f = math.min(kind.speed * 0.5, d) / d
                    bomb.at = {
                        bomb.at[1] + (player.x - bomb.at[1]) * f,
                        bomb.at[2] + (player.y - bomb.at[2]) * f,
                        bomb.at[3] + (player.z - bomb.at[3]) * f,
                    }
                end
                director:sweep_bnpc(bomb.id, player.x, player.y, player.z, kind.speed)
            end
            if player ~= nil and player.x ~= nil and flatDistance(bomb.at, player.x, player.z) <= kind.reach then
                -- Blown where it stands.
                director:sweep_bnpc(bomb.id, bomb.at[1], bomb.at[2], bomb.at[3], kind.speed)
                director:actor_use_action(bomb.id, ACTION_BLAST, player.id)
                bomb.blasted = updates
            end
        end
        if keep then
            table.insert(still, bomb)
        end
    end
    bombs = still
end

-- ---- The fight ----

function onSetup(director)
    director:set_sequence(1)
    -- Out from the moment you arrive (retail's spawns came before the opening.)
    BATTLE:spawn(director, { BNPC_TALKER })
    director:spawn_bnpc(BNPC_HELPER)
    director:hold_bnpc(BNPC_TALKER)
    director:hold_bnpc(BNPC_HELPER)
end

-- The opening has ended: the director's scene 2, its line and the throw at the helper.
function onCommence(director)
    sequence(director, SEQUENCE_LEAD_IN)
    director:engage_bnpc(BNPC_TALKER, BNPC_HELPER)
    director:play_scene(LEAD_IN_SCENE, LEAD_IN_SCENE_FLAGS)
    after(TALK_81_AFTER, function(d) talk(d, 81) end)
    after(THROW_AFTER, function(d) d:use_action(BNPC_TALKER, ACTION_THROW, BNPC_HELPER) end)
end

function onSceneFinished(director, scene)
    if over then
        return
    end
    if scene == LEAD_IN_SCENE and phase == SEQUENCE_LEAD_IN then
        sequence(director, SEQUENCE_LEAD_IN_2)
        director:play_scene(LEAD_IN_SCENE_2, LEAD_IN_SCENE_FLAGS)
        after(TALK_82_AFTER, function(d) talk(d, 82) end)
    elseif scene == LEAD_IN_SCENE_2 and phase == SEQUENCE_LEAD_IN_2 then
        startFight(director)
    end
end

-- The fight (retail's scene 60000 and sequence 4.)
function startFight(director)
    if over or phase ~= SEQUENCE_LEAD_IN_2 then
        return
    end
    sequence(director, SEQUENCE_FIGHTING)
    after(FIRST_WAVE_AFTER, function(d) bringOut(d, 1) end)
    after(FIRST_YELL_AFTER, function(d) d:yell(BNPC_TALKER, FIRST_YELL) end)
end

function cleared(director)
    local following = phase + 1
    if following == SEQUENCE_CLEARED then
        sequence(director, SEQUENCE_CLEARED)
        bomb_clock = bomb_clock + 1
        after(TALK_88_AFTER, function(d) talk(d, 88) end)
        after(WON_AFTER, won)
        return
    end
    sequence(director, following)
    local step = AFTER_CLEARED[following]
    after(step.wave_after, function(d) bringOut(d, step.wave) end)
    for _, line in ipairs(step.talks or {}) do
        after(line[1], function(d) talk(d, line[2]) end)
    end
    for _, yell in ipairs(step.yells or {}) do
        after(yell[1], function(d) d:yell(BNPC_TALKER, yell[2]) end)
    end
    if step.bombs ~= nil then
        bombSchedule(step.bombs.first, step.bombs.every)
    end
    if following == 7 then
        BATTLE:change(director, "the one who talks stops healing", { [BNPC_TALKER] = { heals = {} } })
        after(DISENGAGE_AFTER, function(d) d:engage_bnpc(BNPC_TALKER) end)
    end
end

function won(director)
    over = true
    timers = {}
    sequence(director, SEQUENCE_WON)
    director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS,
        QUEST_VARS_AFTER_WIN)
end

function lost(director)
    over = true
    timers = {}
    director:set_sequence(SEQUENCE_FAILED)
    fail_in = FAIL_DELAY_UPDATES
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
    updates = updates + 1
    runTimers(director)
    if over then
        return
    end
    updateBombs(director, info)
    BATTLE:update(director, health, player_health, info)
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    if wave_left[layout_id] then
        wave_left[layout_id] = nil
        if next(wave_left) == nil and phase >= SEQUENCE_FIGHTING and phase < SEQUENCE_CLEARED then
            cleared(director)
        end
    end
end

-- Losing sends you back to where you entered, still on the same step of the quest, so you can try again.
function onPlayerDeath(director)
    if over then
        return
    end
    lost(director)
end
