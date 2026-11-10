-- Quest battle 60: In the Eyes of Gods and Men (GaiUsb803_00912, the main scenario's level 40 quest in Coerthas.)
-- It takes place in territory 301, whose TerritoryType has Coerthas Central Highlands' level (r1f1), so its NPCs live
-- in the roc_r1f1 drop-in (layer QB_GaiUsb803_001). Layout IDs, levels, HP, positions, flags, looks and the start spot
-- are retail's (Bob's Chronofoil captures 130088ec and 3c56e09f of 2026-11-06, `questkit chronofoil --battle 0.9` and
-- `--battle 0.7`: one run on each of his two characters, both won; level 40.) On your side: a conjurer (4261257, base
-- 1739, 3787 HP) from the start and a gladiator (4261300, base 1738, 4257 HP) who joins at sequence 3. Against you:
-- the one you fight (4261262, base 1735, 9150 HP) with an archer (1733) and a conjurer (1732); then a lancer (1736)
-- and a pugilist (1734); then a marauder (1731), another conjurer (1732, 4287237) and a wyvern (4261305, base 1737,
-- 15150 HP, battalion 5: at war with both sides, and invulnerable: every hit on it did nothing, effect 7.) An onlooker
-- (event NPC 1007071, 4261248) talks and yells; talking to him gets EVENT_QIB_NOT_TALK (scene 10001.)
--
-- Retail's flow (seconds from the fight starting, the director's scene 60000), the same in both runs:
--   0       sequence 2, the onlooker's yell 1030; battle talk 85 (the conjurer) at 5.0, 86 (the boss) at 10.0
--   the boss under 80 % (78.7, 78.4): sequence 3; the gladiator 4.2-4.4 s later; sequence 4 and talk 87 (the
--           conjurer) 5.0 s after sequence 3; the second wave 6.4-6.5 s after sequence 4; sequence 5 10.0 s after
--           sequence 4, with talk 88 (the gladiator) at once, 89 (the onlooker) 10.1 s later, yells 1031 (the
--           conjurer) and 1032 (the onlooker) 15.1 and 20.1 s later
--   the boss under 50 % (48.3, 49.4): sequence 6; talk 95 (the boss) 1.0 s and its yell 1033 6.0 s later; the third
--           wave 8.3-8.4 s later; talks 90 (the gladiator) and 91 (the onlooker) 18.0 and 23.0 s later
--   the boss under 20 % (19.8, 18.5): the gladiator's yell 1034
--   the boss's health at 0: sequence 7 and its talk 92 at once; a rosary (EObj 2002279) appears 0.2-0.5 s later where
--           it always does, the director the handler; the boss's death 2.0 s later; the director's log 3206 ("...
--           drops a draconian rosary") 5.0 s after sequence 7; the wyvern runs off, fades (607) 6.4 s after and goes 1.35 s
--           later; log 3209 ("The wyvern takes to the skies") 10.05 s after; if the rosary is still there then,
--           sequences 8 and 9 at once and the gladiator's yell 1371 12.1 s later
--   lost (no lost run recorded): your health at 0, or Haurchefant's (the gladiator: Bob, from retail); sequence 0xFE
--           and out 5 s later
--   talking to the rosary (4.5 s after sequence 7 in one run, 24.8 in the other): no scene, the director's event
--           action 22 (5 s); as it ends the event finishes, sequence 10, the rosary fades (607) and goes 1.1 s later,
--           log 3208 ("You obtain a draconian rosary"); 3.0 s later "Duty Complete" (60001) and sequence 11, then the
--           battle's scene 11 (0xF8402FFF); the quest moves to sequence 255
-- The opening scene 1 is a cutscene (0xF8412FFF.) Who fights whom and how hard is in their timelines (QB60*.)

-- Haurchefant rides in (retail's SpawnNpc had him on mount 10, the drop-in's `mount`): at a run (6 yalms a second, the
-- ActorMoves' speed byte 60) from where he appears along his facing to HAURCHEFANT_TURN in 7.6-7.8 s, then on towards
-- the fight to HAURCHEFANT_ARRIVES, getting off his mount on the way as he says talk 88 (ActorControl 930, 10.8-11.0 s
-- after he appeared), and fights from 13.2-13.4 s; the same path to a yalm in all three runs (Bob's Chronofoil
-- captures 3c56e09f, 130088ec, 21f967d2.) Held meanwhile, so Battle leaves him alone until he arrives.
HAURCHEFANT_SPEED = 6
HAURCHEFANT_TURN = { x = 463.1, y = 304.34, z = -275.1 }
HAURCHEFANT_ARRIVES = { x = 459.46, y = 304.46, z = -307.0 }
-- In updates from when he appears: the turn (7.66 s at his speed from his spot) and his arrival (5.35 s more.)
HAURCHEFANT_TURN_AFTER = 15
HAURCHEFANT_FIGHTS_AFTER = 26

-- Where retail puts the player and which way they face: their SpawnPlayer in both captures.
START_POSITION = { x = 457.38, y = 303.03, z = -264.67 }
START_ROTATION = 2.673
OPENING_SCENE_FLAGS = 0xF8412FFF

BNPC_CONJURER_ALLY = 4261257
BNPC_GLADIATOR_ALLY = 4261300
BNPC_BOSS = 4261262
BNPC_WYVERN = 4261305
ENPC_ONLOOKER = 4261248
WAVE_1 = { 4261276, 4261275 }
WAVE_2 = { 4261303, 4261302 }
WAVE_3 = { 4287237, 4287236, BNPC_WYVERN }
-- The enemies' conjurers, who heal their side.
ENEMY_CONJURERS = { [4261275] = true, [4287237] = true }

-- The rosary the boss drops, and where it appears (retail's SpawnObject, the same in both runs.)
EOBJ_ROSARY = 2002279
ROSARY_POSITION = { x = 458.884, y = 304.806, z = -279.591 }
-- Picking it up: the event action (its EventAction row; 5 s.)
PICK_UP_BAR = 22

QUEST_SEQUENCE_AFTER_WIN = 255
SEQUENCE_FIGHTING = 2
SEQUENCE_GLADIATOR = 3
SEQUENCE_SECOND_WAVE = 4
SEQUENCE_TALK = 5
SEQUENCE_THIRD_WAVE = 6
SEQUENCE_BOSS_DOWN = 7
SEQUENCE_WYVERN_GONE = 8
SEQUENCE_WAITING = 9
SEQUENCE_PICKED_UP = 10
SEQUENCE_WON = 11
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 11
FINISH_SCENE_FLAGS = 0xF8402FFF
FAIL_DELAY_UPDATES = 10

-- The boss's health (percent) for sequences 3 and 6, and the gladiator's yell.
BOSS_GLADIATOR = 80
BOSS_HALF = 50
BOSS_LOW = 20

-- Timers, in updates (half a second each). A timer of n fires n / 2 s after it's set by another timer or in onUpdate
-- (after that update's timers have run), and (n - 1) / 2 s after it's set before the next update (onCommence,
-- onActorDeath, onEventActionCast.)
TALK_85_AFTER = 11
TALK_86_AFTER = 21
GLADIATOR_AFTER = 9
SEQUENCE_4_AFTER = 10
SECOND_WAVE_AFTER = 13
SEQUENCE_5_AFTER = 20
TALK_89_AFTER = 20
YELL_1031_AFTER = 30
YELL_1032_AFTER = 40
TALK_95_AFTER = 2
YELL_1033_AFTER = 12
THIRD_WAVE_AFTER = 17
TALK_90_AFTER = 36
TALK_91_AFTER = 46
ROSARY_AFTER = 1
DROPPED_LOG_AFTER = 11
WYVERN_FADES_AFTER = 14
-- The wyvern runs off as the boss falls: turning at once (0.2 s after sequence 7) and running in a straight line at
-- 5.9 yalms a second from wherever it was to the same spot in all three runs (within half a yalm), where it fades
-- (6.4-7.0 s after sequence 7) and goes. Kawari faded it where it stood (Bob's bundle 20261110-102239.)
WYVERN_FLEES_TO = { x = 442.25, y = 302.42, z = -324.64 }
WYVERN_SPEED = 6
WYVERN_GOES_AFTER = 3
WYVERN_GONE_AFTER = 21
YELL_1371_AFTER = 24
ROSARY_GOES_AFTER = 3
WON_AFTER = 7

-- Who's talking (DirectorPopupMessage), with retail's names and portraits; the onlooker as an EventNpc (kind 3.)
SPEAKER_CONJURER = { layout = BNPC_CONJURER_ALLY, name = 2081, icon = 73194 }
SPEAKER_BOSS = { layout = BNPC_BOSS, name = 1454, icon = 73195 }
SPEAKER_GLADIATOR = { layout = BNPC_GLADIATOR_ALLY, name = 2082, icon = 73124 }
SPEAKER_ONLOOKER = { layout = ENPC_ONLOOKER, name = 1007071, icon = 73193, kind = 3 }
-- The director's log lines (LogMessage rows.)
LOG_DROPPED = 3206
LOG_OBTAINED = 3208
LOG_WYVERN_GONE = 3209

ACTION_CURE = 120
ACTION_CURE_II = 135

BATTLE = Battle.new()
-- The conjurer went for the first wave's archer first (the one hitting it), then its conjurer, the boss, and the second
-- wave's lancer as it came; the gladiator for that lancer, then the boss; both for the wyvern once it was out, until it
-- left (all three runs.)
BATTLE:ally(BNPC_CONJURER_ALLY, {
    role = "dps",
    focus = { "wyvern", "archer", "adds", "boss" },
    order = "newest",
    range = 10,
    keep_near = { within = 15 },
    -- Retail's healed the gladiator, you and itself at 31-79 % (median 40-47), every 16.7-23.4 s, and once Cure II on
    -- the gladiator at 25 % (capture 21f967d2.)
    heals = {
        { action = ACTION_CURE_II, targets = { BNPC_GLADIATOR_ALLY, Battle.PLAYER, "self" }, below = 26, every = 19.0 },
        { action = ACTION_CURE, targets = { BNPC_GLADIATOR_ALLY, Battle.PLAYER, "self" }, below = 45, every = 16.7 },
    },
})
-- He Provoked (18) the boss once, 2.0-9.6 s after he began fighting (two runs 2.0 and 2.7), and never pulled anything
-- else: the second wave's archer and the wyvern stayed on the player (all three runs.) Battle's `protect` pulled an
-- enemy off the player every 10 s, so the archer and the wyvern fought him, and he fell (Bob's bundle 20261110-095652.)
BATTLE:ally(BNPC_GLADIATOR_ALLY, {
    role = "tank",
    focus = { "wyvern", "adds", "boss" },
    order = "newest",
    protect = false,
})
ACTION_PROVOKE = 18
HAURCHEFANT_PROVOKES_AFTER = 5
-- The enemies' conjurers: one heal every 5.6-8.3 s at most, whichever it is (`heal_gap`; never two closer in the three
-- runs: Cure, then Cure II 1.5 s later was Kawari's, Bob's bundle 20261110-091329): Cure II on one of their side
-- under about 23 % (6-22 %; at 24-26 % Cure came), else Cure under 50 % (31-50 %). Cure healed 444-468, Cure II
-- 695-730, as Kawari's do.
-- (A new table for each: Battle keeps each rule's wait in it.)
ENEMY_HEAL_GAP = 5.6
function enemyHeals()
    return {
        { action = ACTION_CURE_II, targets = "allies", below = 23, every = ENEMY_HEAL_GAP },
        { action = ACTION_CURE, targets = "allies", below = 50, every = ENEMY_HEAL_GAP },
    }
end
-- On retail every enemy went for you first but the first wave's archer, who shot the conjurer from the start (5-12
-- shots before it turned to you, all three runs; Kawari's shot you), and the boss turned to the gladiator once he
-- Provoked it.
BNPC_FIRST_ARCHER = 4261276
for _, layout in ipairs(WAVE_1) do
    local role = { group = "adds" }
    if layout == BNPC_FIRST_ARCHER then
        role.group = { "archer", "adds" }
        role.attacks = BNPC_CONJURER_ALLY
    end
    if ENEMY_CONJURERS[layout] then
        role.heals = enemyHeals()
        role.heal_gap = ENEMY_HEAL_GAP
    end
    BATTLE:enemy(layout, role)
end
for _, layout in ipairs(WAVE_2) do
    BATTLE:enemy(layout, { group = "adds" })
end
BATTLE:enemy(4287237, { group = "adds", heals = enemyHeals(), heal_gap = ENEMY_HEAL_GAP })
BATTLE:enemy(4287236, { group = "adds" })
BATTLE:enemy(BNPC_WYVERN, { group = "wyvern" })
-- Its own Cure once it was under about 15 % (its first 0.7-8 s after it crossed 15 %, three runs), 3.6-7.6 s apart.
BATTLE:enemy(BNPC_BOSS, { group = "boss", heals = { { action = ACTION_CURE, targets = { "self" }, below = 15,
    every = 3.6 } } })

local over = false
local fail_in = nil
local phase = 1
local boss_low = false
local picked_up = false
local rosary = nil
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

function talk(director, speaker, text)
    director:battle_talk(speaker.layout, speaker.name, text, speaker.icon, nil, speaker.kind)
end

function sequence(director, number)
    phase = number
    director:set_sequence(number)
end

function onSetup(director)
    director:set_sequence(1)
end

function onCommence(director)
    sequence(director, SEQUENCE_FIGHTING)
    director:spawn_bnpc(ENPC_ONLOOKER)
    BATTLE:spawn(director, { BNPC_CONJURER_ALLY, BNPC_BOSS, WAVE_1[1], WAVE_1[2] })
    director:yell(ENPC_ONLOOKER, 1030)
    after(TALK_85_AFTER, function(d) talk(d, SPEAKER_CONJURER, 85) end)
    after(TALK_86_AFTER, function(d) talk(d, SPEAKER_BOSS, 86) end)
end

function haurchefantRidesIn(director)
    BATTLE:spawn(director, { BNPC_GLADIATOR_ALLY })
    director:hold_bnpc(BNPC_GLADIATOR_ALLY)
    local turn, arrives = HAURCHEFANT_TURN, HAURCHEFANT_ARRIVES
    director:sweep_bnpc(BNPC_GLADIATOR_ALLY, turn.x, turn.y, turn.z, HAURCHEFANT_SPEED)
    after(HAURCHEFANT_TURN_AFTER, function(d)
        d:sweep_bnpc(BNPC_GLADIATOR_ALLY, arrives.x, arrives.y, arrives.z, HAURCHEFANT_SPEED)
    end)
    after(HAURCHEFANT_FIGHTS_AFTER, function(d)
        d:hold_bnpc(BNPC_GLADIATOR_ALLY, false)
        after(HAURCHEFANT_PROVOKES_AFTER, function(d2)
            BATTLE:provoke(d2, BNPC_GLADIATOR_ALLY, BNPC_BOSS, ACTION_PROVOKE)
        end)
    end)
end

function gladiatorJoins(director)
    sequence(director, SEQUENCE_GLADIATOR)
    after(GLADIATOR_AFTER, haurchefantRidesIn)
    after(SEQUENCE_4_AFTER, function(d)
        sequence(d, SEQUENCE_SECOND_WAVE)
        talk(d, SPEAKER_CONJURER, 87)
        after(SECOND_WAVE_AFTER, function(d2) BATTLE:spawn(d2, WAVE_2) end)
        after(SEQUENCE_5_AFTER, function(d2)
            sequence(d2, SEQUENCE_TALK)
            d2:dismount_bnpc(BNPC_GLADIATOR_ALLY)
            talk(d2, SPEAKER_GLADIATOR, 88)
            after(TALK_89_AFTER, function(d3) talk(d3, SPEAKER_ONLOOKER, 89) end)
            after(YELL_1031_AFTER, function(d3) d3:yell(BNPC_CONJURER_ALLY, 1031) end)
            after(YELL_1032_AFTER, function(d3) d3:yell(ENPC_ONLOOKER, 1032) end)
        end)
    end)
end

function thirdWave(director)
    sequence(director, SEQUENCE_THIRD_WAVE)
    after(TALK_95_AFTER, function(d) talk(d, SPEAKER_BOSS, 95) end)
    after(YELL_1033_AFTER, function(d) d:yell(BNPC_BOSS, 1033) end)
    after(THIRD_WAVE_AFTER, function(d)
        BATTLE:spawn(d, WAVE_3)
        d:set_invulnerable(BNPC_WYVERN, true)
    end)
    after(TALK_90_AFTER, function(d) talk(d, SPEAKER_GLADIATOR, 90) end)
    after(TALK_91_AFTER, function(d) talk(d, SPEAKER_ONLOOKER, 91) end)
end

-- The boss is down: it drops the rosary, the wyvern flies off, and the battle waits for you to pick it up.
function bossDown(director)
    sequence(director, SEQUENCE_BOSS_DOWN)
    talk(director, SPEAKER_BOSS, 92)
    after(ROSARY_AFTER, function(d) d:spawn_eobj(EOBJ_ROSARY, ROSARY_POSITION) end)
    after(DROPPED_LOG_AFTER, function(d) d:log_message(LOG_DROPPED, {}) end)
    director:hold_bnpc(BNPC_WYVERN)
    local flees_to = WYVERN_FLEES_TO
    director:sweep_bnpc(BNPC_WYVERN, flees_to.x, flees_to.y, flees_to.z, WYVERN_SPEED)
    after(WYVERN_FADES_AFTER, function(d)
        d:fade_out_bnpc(BNPC_WYVERN)
        after(WYVERN_GOES_AFTER, function(d2) d2:despawn_bnpc(BNPC_WYVERN) end)
    end)
    after(WYVERN_GONE_AFTER, function(d)
        d:log_message(LOG_WYVERN_GONE, {})
        if not picked_up then
            sequence(d, SEQUENCE_WYVERN_GONE)
            sequence(d, SEQUENCE_WAITING)
            after(YELL_1371_AFTER, function(d2)
                if not picked_up then
                    d2:yell(BNPC_GLADIATOR_ALLY, 1371)
                end
            end)
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
    if over then
        return
    end
    runTimers(director)
    local boss = health[BNPC_BOSS]
    if boss ~= nil and phase < SEQUENCE_BOSS_DOWN then
        if phase == SEQUENCE_FIGHTING and boss < BOSS_GLADIATOR then
            gladiatorJoins(director)
        elseif phase == SEQUENCE_TALK and boss < BOSS_HALF then
            thirdWave(director)
        end
        if phase == SEQUENCE_THIRD_WAVE and not boss_low and boss < BOSS_LOW then
            boss_low = true
            director:yell(BNPC_GLADIATOR_ALLY, 1034)
        end
    end
    BATTLE:update(director, health, player_health, info)
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    -- Haurchefant falling loses the battle, as your own death does (Bob, from retail, bundle 20261110-091329: Kawari
    -- fought on and the player fell 6 s later. No retail run lost, so it goes as the player's: sequence 0xFE at once
    -- and out after FAIL_DELAY_UPDATES, Fool Me Twice's way.)
    if layout_id == BNPC_GLADIATOR_ALLY then
        lost(director)
        return
    end
    if layout_id == BNPC_BOSS and phase < SEQUENCE_BOSS_DOWN then
        bossDown(director)
    end
end

-- Talking to the rosary: the director's event action, no scene (retail's.)
function onObjectTalk(director, actor_id, object_id, base_id)
    if base_id ~= EOBJ_ROSARY or picked_up or over then
        director:finish_gimmick(actor_id)
        return
    end
    rosary = object_id
    director:event_action(PICK_UP_BAR, actor_id, object_id)
end

function onEventActionCast(director, actor_id, target)
    director:finish_gimmick(actor_id)
    if picked_up or over then
        return
    end
    picked_up = true
    sequence(director, SEQUENCE_PICKED_UP)
    director:eobj_control(EOBJ_ROSARY, 607, { rosary or target, 1, 0, 100 })
    director:log_message(LOG_OBTAINED, {})
    after(ROSARY_GOES_AFTER, function(d) d:delete_eobj(EOBJ_ROSARY) end)
    after(WON_AFTER, function(d)
        over = true
        timers = {}
        sequence(d, SEQUENCE_WON)
        d:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS)
    end)
end

function lost(director)
    over = true
    timers = {}
    director:set_sequence(SEQUENCE_FAILED)
    fail_in = FAIL_DELAY_UPDATES
end

-- Losing sends you back to where you entered, still on the same step of the quest, so you can try again.
function onPlayerDeath(director)
    if over then
        return
    end
    lost(director)
end
