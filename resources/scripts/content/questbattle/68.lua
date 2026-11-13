-- Quest battle 68 (JobPld451_01059, Parley in the Sagolii, the Paladin's level 45 quest.)
-- It takes place in territory 312, taken to be a copy of Southern Thanalan (w1f4): retail's start spot is a few yalms
-- from where the player stood in Southern Thanalan using the quest's EOBJECT0 (-406.8, 7.2, 522.9 there; -407.3, 7.2,
-- 525.8 here), so its NPCs are layer QB_JobPld451_001 of wil_w1f4. Layout IDs, levels, HP, positions, display flags,
-- looks and the start spot are retail's (Bob's Chronofoil capture da424662 of 2026-11-12, `questkit chronofoil --battle
-- 0.17`: one run, won, a level 45 Paladin). No allies. Against you, all out from the start: the boss (4296937, base
-- 1663, 5068 HP, a paladin), a gladiator (4296945, base 1664, 851 HP) and three more (4296942-44, base 1665, 640 HP).
--
-- Retail's flow (seconds from the fight starting, the director's scene 60000):
--   0       sequence 2; the boss's battle talk 82 at 5.1
--   all four others down (the last at 13.5, by their deaths' ActorControl 2): sequence 3 at once, talk 83 2.1 s later
--   the boss under 50 % (23.0: 50.1 → 48.3): sequence 4, talk 84 1.1 s later
--   the boss down to 10 % (36.6: 504 HP): he takes no more damage (three more hits left him at 504); 1.8 s later "Duty
--           Complete" (60001) and sequence 5, and the director kills him; the battle's scene 5 (0xF8402EFB) 3.1 s after
--           that; the quest moves to sequence 255, and when the player is back the quest's own LOGMESSAGE0 (3615)
--   lost (no lost run recorded): your health at 0, sequence 0xFE and out 5 s later
-- The battle's client script (`JobPld451Btl_01059`, Bob's extracted scripts of 2026-11-12) has scenes 1 (the opening
-- cutscene, 0xF841AEFB) and 5 (the finish) and OnSequence001-002 only, as played. What each one does is in their
-- timelines (QB68*): the boss's 1098 and 1099 showed no effect, his 1056 (at about 20 %) gave him statuses 293 and 294.

-- Where retail puts the player and which way they face: their spawn in the run.
START_POSITION = { x = -407.28, y = 7.21, z = 525.83 }
START_ROTATION = 0.0
OPENING_SCENE_FLAGS = 0xF841AEFB

BNPC_BOSS = 4296937
OTHERS = { 4296945, 4296944, 4296943, 4296942 }

QUEST_SEQUENCE_AFTER_WIN = 255
-- Parley in the Sagolii's LOGMESSAGE0, the quest's line as the battle's closing scene ends and the player is back
-- (LogMessage 3615 from quest 1059's handler, Bob's Chronofoil capture da424662.)
QUEST_LOG = 3615
SEQUENCE_FIGHTING = 2
SEQUENCE_OTHERS_DOWN = 3
SEQUENCE_BOSS_HALF = 4
SEQUENCE_WON = 5
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 5
FINISH_SCENE_FLAGS = 0xF8402EFB
FAIL_DELAY_UPDATES = 10

-- The boss's health marks (percent).
BOSS_HALF = 50
BOSS_YIELDS = 10

-- Timers, in updates (half a second each). A timer of n fires n / 2 s after it's set by another timer or in onUpdate
-- (after that update's timers have run), and (n - 1) / 2 s after it's set before the next update (onCommence,
-- onActorDeath.)
TALK_82_AFTER = 11
TALK_83_AFTER = 5
TALK_84_AFTER = 2
WON_AFTER_YIELDING = 4

-- Who's talking (DirectorPopupMessage): the boss, by his BNpcName.
SPEAKER_BOSS = { layout = BNPC_BOSS, name = 1421, icon = 0 }

BATTLE = Battle.new()
-- Every one of them went for you on retail.
BATTLE:enemy(BNPC_BOSS, { group = "boss", attacks = nil })
for _, layout in ipairs(OTHERS) do
    BATTLE:enemy(layout, { group = "adds", attacks = nil })
end

local over = false
local fail_in = nil
local phase = 1
local others_left = {}
local yielded = false
local timers = {}

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

function talk(director, text)
    director:battle_talk(SPEAKER_BOSS.layout, SPEAKER_BOSS.name, text, SPEAKER_BOSS.icon)
end

function sequence(director, number)
    phase = number
    director:set_sequence(number)
end

function onSetup(director)
    director:set_sequence(1)
    -- Everyone is out before the opening, as on retail.
    for _, layout in ipairs(OTHERS) do
        others_left[layout] = true
    end
    BATTLE:spawn(director, { BNPC_BOSS })
    BATTLE:spawn(director, OTHERS)
end

function onCommence(director)
    sequence(director, SEQUENCE_FIGHTING)
    after(TALK_82_AFTER, function(d) talk(d, 82) end)
end

-- The win: "Duty Complete", and the boss, who stopped at 10 %, goes.
function won(director)
    over = true
    timers = {}
    sequence(director, SEQUENCE_WON)
    director:kill_bnpc(BNPC_BOSS)
    director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS, nil, true,
        QUEST_LOG)
end

function lost(director)
    over = true
    timers = {}
    director:set_sequence(SEQUENCE_FAILED)
    fail_in = FAIL_DELAY_UPDATES
end

-- The boss stops taking damage at 10 %; the win comes 1.8 s later.
function yields(director)
    if yielded then
        return
    end
    yielded = true
    director:set_invulnerable(BNPC_BOSS, true)
    after(WON_AFTER_YIELDING, won)
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
    local boss = health[BNPC_BOSS]
    if boss ~= nil then
        if phase < SEQUENCE_BOSS_HALF and boss < BOSS_HALF then
            sequence(director, SEQUENCE_BOSS_HALF)
            after(TALK_84_AFTER, function(d) talk(d, 84) end)
        end
        if boss <= BOSS_YIELDS then
            yields(director)
        end
    end
    BATTLE:update(director, health, player_health, info)
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    -- Should he fall before the update sees him at 10 %, it's the same.
    if layout_id == BNPC_BOSS then
        yields(director)
        return
    end
    if not others_left[layout_id] then
        return
    end
    others_left[layout_id] = nil
    if next(others_left) == nil then
        if phase < SEQUENCE_OTHERS_DOWN then
            sequence(director, SEQUENCE_OTHERS_DOWN)
        end
        after(TALK_83_AFTER, function(d) talk(d, 83) end)
    end
end

-- Losing sends you back to where you entered, still on the same step of the quest, so you can try again.
function onPlayerDeath(director)
    if over then
        return
    end
    lost(director)
end
