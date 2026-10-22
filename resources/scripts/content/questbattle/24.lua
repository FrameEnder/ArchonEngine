-- Quest battle: Victory in Peril, for the quest of the same name (SubSea117_00414.)
-- It takes place in territory 249, taken to be a copy of Lower La Noscea (retail's positions in it are Lower La
-- Noscea's, where the quest is), so its BNpcs live in the sea_s1f2 drop-in (layer QB_SubSea117_001). Layout IDs,
-- levels, HP, positions and the start position are retail's: Bob's Chronofoil capture of 2026-10-21, two runs
-- (`questkit chronofoil --battle 0.20`, lost as the player fell, and `0.22`, won; territory 249, duty 313).
--
-- Timed against those runs (seconds from the fight starting, the director's scene 60000, Kawari's onCommence):
--   0      director sequence 2 (sequence 1 is set before the opening cutscene). The boss (3970651) and four corsairs
--          are there from the start: two archers (4167390/91) and, further off, an archer and a conjurer
--          (4167378/79); none of them fights yet
--   you within about 26 yalms of the first two (the won run: sequence 3 as you came within 25.6 of the nearer, the
--          first arrow 1.3 s later; the lost run: the first arrow as you came within 22-27), or one of them hurt: they
--          come at you, sequence 3
--   both down, and you within 26.7 yalms of the other two's conjurer (both runs: you walked over from where you'd
--          fought the first pair, 50 yalms off; sequence 4 came 0.1-0.2 s after you were 26.6 and 25.4 yalms from it,
--          not at 26.9-28.2, 4.2 and 5.3 s after the first pair fell): sequence 4, and those two come at you (they
--          don't notice you on their own until then)
--   those two down and you within 15 yalms of the boss (both runs: sequence 5 came 0.1-0.2 s after you were 13.8 and
--          14.4 yalms from it, not at 15.7-16.4; the client then stops you where you stand, 13.1-13.3 off): sequence 5
--          and the cutscene (the director's scene 5, 0xF8402EFB); the allied marauder (3970427, level 30) and conjurer
--          (3970428) appear. Nothing fights during it: the boss's first blow is 1 s after sequence 6 (both runs)
-- (Distances and times from the capture's own clock: the player's positions are in it, the NPCs' in the runs.)
--   the cutscene over: sequence 6; the boss goes after the marauder, and three more corsairs come (two archers
--          after the conjurer, a conjurer after you); battle talk 105, 106, 107, 108 every 5 s from 4 s in
--   the boss at 55% (56.1% -> 54.9% and 55.1% -> 51.2%): sequence 7 and five more (two lancers, two thaumaturges
--          and a marauder, all after the allied marauder); the boss turns on you; 5 s later sequence 8, then 109, 110
--          and 111 every 5 s from 3 s in
--   the boss at 25% (25.2% -> 23.2% and 25.7% -> 20.8%): sequence 9 and the boss's 112; sequence 10 about 5.5 s
--          later (6.1 and 4.9 s), 115 3 s after it and 113 6-7 s after that; 114 5 s later (the lost run)
--   the boss down: at once the allies sheathe and leave battle, and so do you (the won run had no adds left; Kawari
--          stops any still standing too); 2 s later (as its death animation ended) sequence 11, "Duty Complete", 114
--          (if not said yet) 3 s later, and the quest battle's scene 11 (0xF8402FFF); the quest moves to sequence 4
--   the player down (the lost run): sequence 0xFE, and out 6 s later
-- Who goes after whom (both runs): the allied marauder on the boss, then the last five as they come (newest
-- first), then the boss again; the allied conjurer on the boss, healing you, the marauder and itself (Cure at 7-53%,
-- the most hurt 19 times in 31); the corsair conjurers heal their side. How hard everyone hits and heals is in their
-- timelines' recorded_amounts (QB24*_<base>.json).

-- The opening cutscene's scene flags, as retail sent them (Bob's Chronofoil capture of 2026-10-21, both runs); quest battles' differ.
OPENING_SCENE_FLAGS = 0xF8412EFB

-- Where retail puts the player and which way they face (the capture.)
START_POSITION = { x = 165.01, y = 14.1, z = 676.28 }
START_ROTATION = -2.727

BNPC_BOSS = 3970651
BNPC_ALLY_MARAUDER = 3970427
BNPC_ALLY_CONJURER = 3970428
FIRST_PAIR = { 4167390, 4167391 }
SECOND_PAIR = { 4167378, 4167379 }
SECOND_WAVE = { 3970655, 3970671, 3970673 }
THIRD_WAVE = { 4103658, 3970675, 3970686, 3970685, 3970684 }
ALLIES = { BNPC_ALLY_MARAUDER, BNPC_ALLY_CONJURER }

-- Who's talking, by name (BNpcName, or the ENpcResident for the one not in the fight) and portrait, as retail sent
-- them. The watcher isn't in the fight, so retail names it as an EventNpc (kind 3; the client crashes looking an
-- ENpcResident up as a BNpcName, Bob's bundle of 2026-10-22.)
SPEAKER_WATCHER = { layout = 0, name = 1002702, icon = 73237, kind = 3 }
SPEAKER_BOSS = { layout = BNPC_BOSS, name = 811, icon = 73238 }
SPEAKER_MARAUDER = { layout = BNPC_ALLY_MARAUDER, name = 815, icon = 73239 }
SPEAKER_CORSAIR = { layout = 3970684, name = 814, icon = 73240 }
SPEAKER_CONJURER = { layout = BNPC_ALLY_CONJURER, name = 816, icon = 73241 }

-- The quest moves to this sequence when the battle is won; retail then plays the quest battle's scene 11.
QUEST_SEQUENCE_AFTER_WIN = 4
FINISH_SCENE = 11
FINISH_SCENE_FLAGS = 0xF8402FFF
CUTSCENE_SCENE = 5
CUTSCENE_FLAGS = 0xF8402EFB
SEQUENCE_START = 1
SEQUENCE_FAILED = 0xFE
FAIL_DELAY_UPDATES = 12
SEVENTH_PHASE_HEALTH = 55
LAST_PHASE_HEALTH = 25
ACTION_CURE = 120
ALLY_ENMITY = 100000

-- Steps: { updates after the step before, kind, ... } (Battle:steps)
-- The first pair down: the second pair waits for you to come over (see onUpdate.)
FIRST_PAIR_DOWN_STEPS = {
    { 0, "call", function(director)
        second_pair_ready = true
    end },
}
SECOND_PAIR_DOWN_STEPS = {
    { 0, "sequence", 5 },
    { 0, "call", function(director)
        director:play_scene(CUTSCENE_SCENE, CUTSCENE_FLAGS)
    end },
    -- They appear during the cutscene but wait for it to end (retail: nothing fights until sequence 6.)
    { 0, "call", function(director)
        BATTLE:holdBack(ALLIES)
    end },
    { 1, "spawn", ALLIES },
}
SIXTH_STEPS = {
    { 0, "sequence", 6 },
    { 0, "call", function(director)
        BATTLE:release(director, ALLIES)
        BATTLE:engage(director, { BNPC_BOSS }, BNPC_ALLY_MARAUDER, ALLY_ENMITY)
    end },
    { 1, "spawn", SECOND_WAVE },
    { 8, "talk", SPEAKER_WATCHER, 105 },
    { 10, "talk", SPEAKER_BOSS, 106 },
    { 10, "talk", SPEAKER_WATCHER, 107 },
    { 10, "talk", SPEAKER_MARAUDER, 108 },
}
SEVENTH_STEPS = {
    { 0, "sequence", 7 },
    { 1, "spawn", THIRD_WAVE },
    { 0, "call", function(director)
        director:set_hate(BNPC_BOSS, nil, ALLY_ENMITY * 2) -- it turns on the player
    end },
    { 9, "sequence", 8 },
    { 6, "talk", SPEAKER_CORSAIR, 109 },
    { 10, "talk", SPEAKER_CONJURER, 110 },
    { 10, "talk", SPEAKER_MARAUDER, 111 },
}
LAST_STEPS = {
    { 0, "sequence", 9 },
    { 0, "talk", SPEAKER_BOSS, 112 },
    { 11, "sequence", 10 },
    { 6, "talk", SPEAKER_CONJURER, 115 },
    { 12, "talk", SPEAKER_BOSS, 113 },
    { 10, "call", function(director)
        sayLast(director)
    end },
}

-- The boss down: 2 s later (its death animation, as retail waits) sequence 11 and "Duty Complete"; 114 3 s after that.
WON_STEPS = {
    { 4, "sequence", 11 },
    { 0, "call", function(director)
        director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS)
    end },
    { 6, "call", function(director)
        sayLast(director)
    end },
}

-- The battle's outline (content/lib/Battle.lua), as retail ran it.
BATTLE = Battle.new()
BATTLE:ally(BNPC_ALLY_MARAUDER, { role = "tank", focus = { "boss" }, order = "newest", range = 3, protect = false })
BATTLE:ally(BNPC_ALLY_CONJURER, {
    role = "support",
    focus = { "boss", "adds" },
    range = 15,
    heals = {
        { action = ACTION_CURE, targets = { Battle.PLAYER, BNPC_ALLY_MARAUDER, "self" }, below = 55, every = 7 },
    },
})
-- The boss and the second pair don't notice anyone on their own: the boss waits out the cutscene (it would take on
-- the allies appearing beside it), the second pair for the first to fall.
BATTLE:enemy(BNPC_BOSS, { group = "boss", passive = true, unaware = true })
for _, id in ipairs(FIRST_PAIR) do
    BATTLE:enemy(id, { group = "first", passive = true })
end
for _, id in ipairs(SECOND_PAIR) do
    BATTLE:enemy(id, { group = "second", passive = true, unaware = true })
end
BATTLE:enemy(3970655, { group = "adds", attacks = BNPC_ALLY_CONJURER })
BATTLE:enemy(3970671, { group = "adds", attacks = BNPC_ALLY_CONJURER })
for _, id in ipairs(THIRD_WAVE) do
    BATTLE:enemy(id, { group = { "third", "adds" }, attacks = BNPC_ALLY_MARAUDER })
end
-- The corsair conjurers Cure their side (retail: 34 Cures of 81-86 HP, 5.5-7.7 s apart, at 2-51%, mostly the most
-- hurt.) The first one only ever Cured itself and the archer beside it, never the first pair (it isn't fighting yet
-- then); the second one the boss and the adds.
BATTLE:enemy(4167379, {
    group = "second",
    passive = true,
    unaware = true,
    heals = { { action = ACTION_CURE, targets = { 4167378, "self" }, below = 50, every = 7 } },
})
BATTLE:enemy(3970673, {
    group = "adds",
    heals = { { action = ACTION_CURE, targets = "allies", below = 50, every = 7 } },
})

-- How close you come before each part starts (see the timing above; distances from the nearest of the group.)
FIRST_PAIR_NOTICE_YALMS = 26
local engaged = false
SECOND_PAIR_NOTICE_YALMS = 26.7
second_pair_ready = false
local second_engaged = false
CUTSCENE_NEAR_YALMS = 15
cutscene_ready = false
local cutscene_started = false

local function playerNear(info, layouts, yalms)
    local player = info and info.player
    if player == nil then
        return false
    end
    for _, id in ipairs(layouts) do
        local npc = info[id]
        if npc ~= nil then
            local dx, dy, dz = npc.x - player.x, npc.y - player.y, npc.z - player.z
            if dx * dx + dy * dy + dz * dz <= yalms * yalms then
                return true
            end
        end
    end
    return false
end
BATTLE:onCleared(FIRST_PAIR, function(director)
    BATTLE:steps(FIRST_PAIR_DOWN_STEPS)
end)
-- The second pair down: the cutscene waits for you to come over to the boss (see onUpdate.)
BATTLE:onCleared(SECOND_PAIR, function(director)
    cutscene_ready = true
end)
-- The allied marauder takes the last five as they come, then the boss again.
BATTLE:onHealth(BNPC_BOSS, SEVENTH_PHASE_HEALTH, function(director)
    BATTLE:steps(SEVENTH_STEPS)
    BATTLE:change(director, "the last five", { [BNPC_ALLY_MARAUDER] = { focus = { "third", "boss" } } })
end)
BATTLE:onHealth(BNPC_BOSS, LAST_PHASE_HEALTH, function(director)
    BATTLE:steps(LAST_STEPS)
end)

local over = false
local fail_in = nil
local said_last = false

function sayLast(director)
    if not said_last then
        said_last = true
        director:battle_talk(SPEAKER_WATCHER.layout, SPEAKER_WATCHER.name, 114, SPEAKER_WATCHER.icon, nil,
            SPEAKER_WATCHER.kind)
    end
end

function onSetup(director)
    director:set_sequence(SEQUENCE_START)
end

function onCommence(director)
    director:set_sequence(2)
    BATTLE:spawn(director, { BNPC_BOSS })
    BATTLE:spawn(director, FIRST_PAIR)
    BATTLE:spawn(director, SECOND_PAIR)
end

-- The cutscene over: the second half starts.
function onSceneFinished(director, scene)
    if scene == CUTSCENE_SCENE and not over then
        BATTLE:steps(SIXTH_STEPS)
    end
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
    if over then
        BATTLE:runPending(director)
        return
    end
    if not engaged then
        for _, id in ipairs(FIRST_PAIR) do
            if health[id] ~= nil and health[id] < 100 then
                engaged = true
            end
        end
        if playerNear(info, FIRST_PAIR, FIRST_PAIR_NOTICE_YALMS) then
            engaged = true
        end
        if engaged then
            director:set_sequence(3)
            for _, id in ipairs(FIRST_PAIR) do
                if BATTLE.alive[id] then
                    director:set_hate(id, nil, ALLY_ENMITY)
                end
            end
        end
    end
    if second_pair_ready and not second_engaged then
        local hurt = false
        for _, id in ipairs(SECOND_PAIR) do
            if health[id] ~= nil and health[id] < 100 then
                hurt = true
            end
        end
        if hurt or playerNear(info, SECOND_PAIR, SECOND_PAIR_NOTICE_YALMS) then
            second_engaged = true
            director:set_sequence(4)
            BATTLE:engage(director, SECOND_PAIR, nil, ALLY_ENMITY) -- the player
        end
    end
    if cutscene_ready and not cutscene_started and playerNear(info, { BNPC_BOSS }, CUTSCENE_NEAR_YALMS) then
        cutscene_started = true
        BATTLE:steps(SECOND_PAIR_DOWN_STEPS)
    end
    BATTLE:update(director, health, player_health, info)
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    if layout_id == BNPC_BOSS then
        over = true
        -- The fight ends as it falls: everyone sheathes and leaves battle (retail, the same moment), adds still
        -- standing too.
        director:end_fight()
        BATTLE:clearSteps()
        BATTLE:steps(WON_STEPS)
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
