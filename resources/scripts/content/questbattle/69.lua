-- Quest battle 69: the Paladin's level 50 quest's battle (quest 1060), in territory 313, whose level isn't known, so
-- its NPCs live in a drop-in of its own (`territory_313.json`, layer QB_69). Layout IDs, levels, HP, positions, gear
-- and flags are retail's: Bob's Chronofoil captures 36c7d119 (2026-11-17: one run, won in 324 s, a Paladin of 50,
-- `questkit chronofoil --battle 0.1`) and b98a60bc (the same day: the first 97 s of another run, which the client
-- crashed in; `--battle 0.4`), ContentFinderCondition 362.
--
-- It has three parts, the two allies (BNpcNames 1421 and 1808, bases 1558 and 1557, level 50) a new NPC in each:
--
-- 1. Where you arrive: a manipulated ogre (4293477) and eye (4293479) against you and the two allies (4293474,
--    4293473), who both go for the ogre; the second ally Cures the first (six times, at 49-68 %), who Cures itself.
--      0      sequence 2; the first ally's talk 100 3.0 s in, the second's 101 at 8.1 (both runs alike)
--      the ogre down (its Defeated, 2 s after its health reached 0; the eye fell first both times): sequence 3;
--             talks 102, 103, 104 at 3.1, 8.1 and 13.1 s; at 18.1 s sequence 4 and the director's scene 4
--             (0xF8412FFF): the allies fade out and their second NPCs (4293788, 4293791) stand nearby
--      the scene over: a loading screen to (-768.71, 224.08, 33.80), sequence 5, and an event object (EObj 2002296,
--             layout 4293747) 124 yalms west
-- 2. The walk there. The second NPCs wander off (one 25 yalms east after 4.8 s, the other 13 south after 6.8 s) and
--    never fight; they went out of the player's sight 53 s later.
--      the object used (the director's event action 6, 2 s, no scene of the object's): sequence 6 and scene 6
--             (0xF8412FFF), with the allies' third NPCs (4302195, 4302196) and a helper (4323575, base 2201, 4400
--             HP, unseen) at the object; then sequence 7 and scene 7 (0x2001), which the client ended 17.1 s
--             later, with the allies' talks 105, 106 and 107 at 3.1, 9.1 and 15.1 s
--      scene 7 over: sequence 8 and, half a second later, the master (4293553, level 50, 11650 HP) on the height
--             above; 5.0 s on sequence 9 and talk 108, and the master casts Star Map (1116, 3.5 s) at the helper
--      7.4 s after sequence 9: sequence 10: the helper and the object fade out (ActorControl 607), and the fight
--             is on (the allies take the master on; it comes for you). Talk 109 at 6.1 s, the master's 110 at 11.1
-- 3. The master.
--      13.1 s after sequence 10: sequence 11 and two executioners (4293547, 4293548) and three assassins (4293534-36)
--             0.3 s later, who stand 5 s; the master's 111 at 3.0 s, 112 at 7.0, 113 at 12.1
--      the master at 60 % (0.5 s after the hit that took it there; its last add was still falling): sequence 12 and,
--             0.4 s later, an unholy grimoire (4293997, 4660 HP: it does nothing itself) and a vyraal (4293999); the
--             grimoire brought another vyraal 24.8 s after the first and a third 24.8 s after that. Talks 114, 115,
--             116 at 10.1, 15.1 and 20.1 s. The first ally went for the grimoire, the second for the vyraals, which
--             went for him
--      50.4 s after sequence 12: sequence 13 (with the third vyraal)
--      the master down (its Defeated): sequence 14, "Duty Complete", scene 14 (0xF8402EFB) 3.6 s later; the quest
--             moves to 255
-- Read from one run and so guesses: that 60 % is what brings the grimoire (the adds' falling fits less well: the
-- sequence came before the last one's Defeated), that sequence 13 is a clock, that the vyraals stop coming when the
-- grimoire is destroyed (Bob left it standing at 10 %), and where the second NPCs go. The object could be used again
-- before the helper went (Bob did, between sequences 8 and 9): its bar played and nothing more.

-- Where retail puts the player and which way they face: their arrival in the capture.
START_POSITION = { x = -766.28, y = 225.27, z = 26.3 }
START_ROTATION = -0.784
-- Retail's opening scene 1 here.
OPENING_SCENE_FLAGS = 0xF841AEFB

BNPC_OGRE = 4293477
BNPC_EYE = 4293479
-- The allies, a new NPC in each part: where you arrive, on the walk, and at the object.
ALLY_1 = { 4293474, 4293788, 4302195 }
ALLY_2 = { 4293473, 4293791, 4302196 }
BNPC_HELPER = 4323575
BNPC_MASTER = 4293553
EXECUTIONERS = { 4293547, 4293548 }
ASSASSINS = { 4293534, 4293535, 4293536 }
BNPC_GRIMOIRE = 4293997
BNPC_VYRAAL = 4293999

EOBJ_DESTINATION = 2002296
EOBJ_LAYOUT = 4293747
EOBJ_STATE = 1
EOBJ_POSITION = { x = -893.09, y = 229.54, z = -11.89 }
-- The director's event action as the object is used (retail's ActorControlTarget EventAction `[1, 6]`, 2.0 s.)
USE_BAR = 6

-- After scene 4 retail put the player here through a loading screen.
AFTER_SCENE_4 = { x = -768.71, y = 224.08, z = 33.80 }
AFTER_SCENE_4_ROTATION = -1.96
-- Where the second NPCs went, and how long they stood first (updates.)
WANDERS = {
    { layout = ALLY_1[2], after = 10, to = { x = -745.08, y = 226.86, z = 33.01 } },
    { layout = ALLY_2[2], after = 14, to = { x = -776.39, y = 224.17, z = 48.08 } },
}

SCENE_LEAVE = 4
SCENE_ARRIVE = 6
SCENE_TALK = 7
CUTSCENE_FLAGS = 0xF8412FFF
TALK_FLAGS = 0x2001

QUEST_SEQUENCE_AFTER_WIN = 255
SEQUENCE_START = 1
SEQUENCE_FIGHTING = 2
SEQUENCE_FIRST_WON = 3
SEQUENCE_LEAVING = 4
SEQUENCE_WALK = 5
SEQUENCE_ARRIVED = 6
SEQUENCE_TALKING = 7
SEQUENCE_MASTER = 8
SEQUENCE_STAR_MAP = 9
SEQUENCE_MASTER_FIGHT = 10
SEQUENCE_ADDS = 11
SEQUENCE_GRIMOIRE = 12
SEQUENCE_LATE = 13
SEQUENCE_WON = 14
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 14
FINISH_SCENE_FLAGS = 0xF8402EFB
FAIL_DELAY_UPDATES = 12

-- Timers, in updates (half a second each). A timer of n fires n / 2 s after it's set by another timer or in onUpdate,
-- and (n - 1) / 2 s after it's set before the next update (onCommence, onActorDeath, onSceneFinished,
-- onEventActionCast.)
DEATH_NOTICED_AFTER = 4
MASTER_AFTER = 1
STAR_MAP_AFTER = 10
MASTER_FIGHT_AFTER = 15
ADDS_SEQUENCE_AFTER = 26
ADDS_AFTER = 1
GRIMOIRE_AFTER = 1
VYRAAL_EVERY = 50
LATE_AFTER = 101
MASTER_HEALTH_FOR_GRIMOIRE = 60

ACTION_STAR_MAP = 1116
ACTION_CURE = 120

-- The battle talk (the content's text IDs and the speakers' BNpcNames, retail's; no portrait.)
NAME_ALLY_1 = 1421
NAME_ALLY_2 = 1808
NAME_MASTER = 1894

ADDS = "adds"
MASTER = "master"
SUMMONED = "summoned"
GRIMOIRE = "grimoire"

BATTLE = Battle.new()
for part = 1, 3 do
    if part ~= 2 then
        -- The first ally takes the hits and Cures itself (twice, at 45-53 %); the second Cures it (at 49-68 %, 5.9
        -- to 10.6 s apart.)
        BATTLE:ally(ALLY_1[part], {
            role = "tank",
            focus = { GRIMOIRE, ADDS, MASTER, "any" },
            heals = { { action = ACTION_CURE, targets = { ALLY_1[part] }, below = 50, every = 24 } },
        })
        BATTLE:ally(ALLY_2[part], {
            role = "dps",
            focus = { SUMMONED, ADDS, MASTER, "any" },
            heals = { { action = ACTION_CURE, targets = { ALLY_1[part] }, below = 68, every = 6 } },
        })
    end
end
for _, layout in ipairs(WANDERS) do
    BATTLE:ally(layout.layout, { still = true })
end
BATTLE:ally(BNPC_HELPER, { still = true })
BATTLE:enemy(BNPC_OGRE, { group = "first" })
BATTLE:enemy(BNPC_EYE, { group = "first" })
BATTLE:enemy(BNPC_MASTER, { group = MASTER, passive = true, unaware = true })
for _, layout in ipairs(EXECUTIONERS) do
    BATTLE:enemy(layout, { group = ADDS })
end
for _, layout in ipairs(ASSASSINS) do
    BATTLE:enemy(layout, { group = ADDS, attacks = ALLY_2[3] })
end
BATTLE:enemy(BNPC_GRIMOIRE, { group = GRIMOIRE, still = true, passive = true })
BATTLE:enemy(BNPC_VYRAAL, { group = SUMMONED, attacks = ALLY_2[3], stands_for = 0 })

local over = false
local fail_in = nil
local phase = SEQUENCE_START
local timers = {}
-- The object as the client knows it, and whether it has brought the allies yet.
local destination = nil
local arrived = false
-- How many of the grimoire's vyraals stand, and the updates until the next.
local vyraals = 0
local vyraal_in = nil
local grimoire_out = false

-- Timers of their own (not Battle:steps, which queues each after the last.)
function after(updates, fn)
    table.insert(timers, { updates, fn })
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

local function sequence(director, n)
    phase = n
    director:set_sequence(n)
end

local function talk(director, part, ally, name, text)
    director:battle_talk(ally and ally[part] or BNPC_MASTER, name, text, 0)
end

-- Talks, each so many updates from now: { updates, the ally (nil: the master), its name, the text }.
local function talks(part, list)
    for _, entry in ipairs(list) do
        after(entry[1], function(d) talk(d, part, entry[2], entry[3], entry[4]) end)
    end
end

function onSetup(director)
    director:set_sequence(SEQUENCE_START)
end

function onCommence(director)
    sequence(director, SEQUENCE_FIGHTING)
    BATTLE:spawn(director, { ALLY_1[1], ALLY_2[1] })
    BATTLE:spawn(director, { BNPC_OGRE, BNPC_EYE })
    talks(1, { { 7, ALLY_1, NAME_ALLY_1, 100 }, { 17, ALLY_2, NAME_ALLY_2, 101 } })
end

-- The first part is won: the talk, then the scene that takes everyone off.
local function firstWon(director)
    sequence(director, SEQUENCE_FIRST_WON)
    talks(1, { { 6, ALLY_1, NAME_ALLY_1, 102 }, { 16, ALLY_2, NAME_ALLY_2, 103 }, { 26, ALLY_2, NAME_ALLY_2, 104 } })
    after(36, function(d)
        sequence(d, SEQUENCE_LEAVING)
        d:play_scene(SCENE_LEAVE, CUTSCENE_FLAGS)
        d:fade_out_bnpc(ALLY_1[1])
        d:fade_out_bnpc(ALLY_2[1])
        after(1, function(d2)
            BATTLE:spawn(d2, { ALLY_1[2], ALLY_2[2] })
        end)
        after(3, function(d2)
            BATTLE:remove(d2, { ALLY_1[1], ALLY_2[1] })
        end)
    end)
end

-- The master's part begins: scene 7 is over.
local function masterComes(director)
    sequence(director, SEQUENCE_MASTER)
    after(MASTER_AFTER, function(d)
        BATTLE:spawn(d, { BNPC_MASTER })
        d:hold_bnpc(BNPC_MASTER)
    end)
    after(STAR_MAP_AFTER, function(d)
        sequence(d, SEQUENCE_STAR_MAP)
        talk(d, 3, ALLY_2, NAME_ALLY_2, 108)
        d:use_action(BNPC_MASTER, ACTION_STAR_MAP, BNPC_HELPER)
        after(MASTER_FIGHT_AFTER, function(d2)
            sequence(d2, SEQUENCE_MASTER_FIGHT)
            d2:fade_out_bnpc(BNPC_HELPER)
            if destination ~= nil then
                d2:eobj_control(EOBJ_DESTINATION, 607, { destination, 1, 0, 100 })
            end
            after(3, function(d3)
                BATTLE:remove(d3, { BNPC_HELPER })
                d3:delete_eobj(EOBJ_DESTINATION)
                destination = nil
            end)
            d2:hold_bnpc(BNPC_MASTER, false)
            BATTLE:release(d2, { ALLY_1[3], ALLY_2[3] })
            BATTLE:engage(d2, { BNPC_MASTER }, nil)
            talks(3, { { 12, ALLY_1, NAME_ALLY_1, 109 }, { 22, nil, NAME_MASTER, 110 } })
            after(ADDS_SEQUENCE_AFTER, function(d3)
                sequence(d3, SEQUENCE_ADDS)
                after(ADDS_AFTER, function(d4)
                    BATTLE:spawn(d4, EXECUTIONERS)
                    BATTLE:spawn(d4, ASSASSINS)
                end)
                talks(3, { { 6, nil, NAME_MASTER, 111 }, { 14, ALLY_2, NAME_ALLY_2, 112 },
                    { 24, ALLY_2, NAME_ALLY_2, 113 } })
            end)
        end)
    end)
end

-- A vyraal from the grimoire, after the second ally (they share one layout ID, so each is known by its actor.)
local function bringVyraal(director)
    if vyraals == 0 then
        BATTLE:spawn(director, { BNPC_VYRAAL })
    else
        local actor = director:spawn_bnpc(BNPC_VYRAAL)
        director:set_hate(actor, ALLY_2[3], 1)
    end
    vyraals = vyraals + 1
    vyraal_in = VYRAAL_EVERY
end

local function grimoireComes(director)
    sequence(director, SEQUENCE_GRIMOIRE)
    after(GRIMOIRE_AFTER, function(d)
        BATTLE:spawn(d, { BNPC_GRIMOIRE })
        grimoire_out = true
        bringVyraal(d)
    end)
    talks(3, { { 20, ALLY_1, NAME_ALLY_1, 114 }, { 30, ALLY_2, NAME_ALLY_2, 115 }, { 40, ALLY_2, NAME_ALLY_2, 116 } })
    after(LATE_AFTER, function(d) sequence(d, SEQUENCE_LATE) end)
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
    runTimers(director)
    if over then
        return
    end
    BATTLE:update(director, health, player_health, info)
    if phase == SEQUENCE_ADDS then
        local master = (health or {})[BNPC_MASTER]
        if master ~= nil and master <= MASTER_HEALTH_FOR_GRIMOIRE then
            grimoireComes(director)
        end
    end
    -- The grimoire brings another vyraal every so often while it stands.
    if grimoire_out and BATTLE.alive[BNPC_GRIMOIRE] and vyraal_in ~= nil then
        vyraal_in = vyraal_in - 1
        if vyraal_in <= 0 then
            bringVyraal(director)
        end
    end
end

function onSceneFinished(director, scene)
    if over then
        return
    end
    if scene == SCENE_LEAVE and phase == SEQUENCE_LEAVING then
        -- Retail ends the scene with a loading screen to where the walk starts; the object is there from then.
        director:warp_players(AFTER_SCENE_4.x, AFTER_SCENE_4.y, AFTER_SCENE_4.z, AFTER_SCENE_4_ROTATION)
        sequence(director, SEQUENCE_WALK)
        after(1, function(d)
            d:spawn_eobj(EOBJ_DESTINATION, EOBJ_POSITION, EOBJ_LAYOUT, EOBJ_STATE)
        end)
        for _, wander in ipairs(WANDERS) do
            after(wander.after, function(d)
                d:move_bnpc(wander.layout, wander.to.x, wander.to.y, wander.to.z)
            end)
        end
    elseif scene == SCENE_ARRIVE and phase == SEQUENCE_ARRIVED then
        sequence(director, SEQUENCE_TALKING)
        director:play_scene(SCENE_TALK, TALK_FLAGS)
        -- Its talk is the director's: 105, 106 and 107 at 3.1, 9.1 and 15.1 s; the scene ended 17.1 s in.
        talks(3, { { 7, ALLY_1, NAME_ALLY_1, 105 }, { 19, ALLY_2, NAME_ALLY_2, 106 }, { 31, ALLY_2, NAME_ALLY_2, 107 } })
    elseif scene == SCENE_TALK and phase == SEQUENCE_TALKING then
        masterComes(director)
    end
end

-- Using the object: the director's event action, no scene of the object's (retail's.)
function onObjectTalk(director, actor_id, object_id, base_id)
    if base_id ~= EOBJ_DESTINATION or over then
        director:finish_gimmick(actor_id)
        return
    end
    destination = object_id
    director:event_action(USE_BAR, actor_id, object_id)
end

function onEventActionCast(director, actor_id, target)
    director:finish_gimmick(actor_id)
    -- Used again before the helper goes, the bar plays and nothing more (retail's, between sequences 8 and 9.)
    if arrived or over or phase ~= SEQUENCE_WALK then
        return
    end
    arrived = true
    sequence(director, SEQUENCE_ARRIVED)
    director:play_scene(SCENE_ARRIVE, CUTSCENE_FLAGS)
    BATTLE:remove(director, { ALLY_1[2], ALLY_2[2] })
    BATTLE:holdBack({ ALLY_1[3], ALLY_2[3] })
    after(1, function(d)
        BATTLE:spawn(d, { ALLY_1[3], ALLY_2[3], BNPC_HELPER })
    end)
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    if layout_id == BNPC_VYRAAL then
        -- Another of them may still stand: the allies keep to it.
        vyraals = math.max(vyraals - 1, 0)
        BATTLE.alive[BNPC_VYRAAL] = vyraals > 0 or nil
    end
    if phase == SEQUENCE_FIGHTING and (layout_id == BNPC_OGRE or layout_id == BNPC_EYE)
        and not BATTLE.alive[BNPC_OGRE] and not BATTLE.alive[BNPC_EYE] then
        after(DEATH_NOTICED_AFTER, firstWon)
    elseif layout_id == BNPC_GRIMOIRE then
        grimoire_out = false
    elseif layout_id == BNPC_MASTER then
        after(DEATH_NOTICED_AFTER, function(d)
            over = true
            timers = {}
            d:set_sequence(SEQUENCE_WON)
            d:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS)
        end)
    end
end

-- Losing sends you back to where you entered, still on the same step of the quest, so you can try again.
function onPlayerDeath(director)
    if over then
        return
    end
    over = true
    timers = {}
    director:set_sequence(SEQUENCE_FAILED)
    fail_in = FAIL_DELAY_UPDATES
end
