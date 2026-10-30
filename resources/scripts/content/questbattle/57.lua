-- Quest battle: Brotherly Love (GaiUsa201_00724.)
-- It takes place in territory 303, a copy of the East Shroud (retail's positions in it are the East Shroud's, where
-- Komuxio's quests are), so its NPCs live in the fst_f1f2 drop-in (layer QB_GaiUsa201_001): three thaumaturges (bases
-- 1584, 4255869-4255871) and their gladiator leader (1582, 4255868, BNpcName 583), Yda (1586, 4255892) and Papalymo
-- (1587, 4255894) on your side at level 50, Claxio watching (an event NPC, 4256371), and the second wave: a beast that
-- isn't to be killed (1585, 4255903, 49300 HP), a gladiator (1658, 4255897) and a conjurer (1583, 4255896). Layout
-- IDs, levels, HP, positions, display flags, looks and the start spot are retail's (Bob's Chronofoil capture 5fcc970c
-- of 2026-10-30, `questkit chronofoil --battle 0.7`, one run, won; a level 26 player.)
--
-- Timed against that run (seconds from the fight starting, the director's scene 60000):
--   0       sequence 2: the first wave fights. Two thaumaturges go for Yda and Papalymo, one and the leader for you
--           (the leader on Papalymo); Yda and Papalymo each take the thaumaturge on them, then the oldest left
--   5, 10   battle talk 78 (the leader) and 79 (Claxio, ENpcResident 1006684: an event NPC, kind 3)
--   26.4    the first thaumaturge down, as its death animation ended (2 s after its health reached 0): sequence 3,
--           the leader's 80, and the second wave (the beast on Yda, the other two on you)
--   +10.1   sequence 4: Yda and Papalymo leave the rest to you and fight the beast. Papalymo's 81 2 s later, the
--           beast's 604 at them 3.9 s later, Claxio's 82 7 s later, Yda's 85 and Papalymo's 86 8 s later (as the 604
--           left them at 80 %)
--   +10.1   sequence 5: the leader turns on you; Yda's 83 1.9 s later
--   the leader at 35 % (43 -> 35, 26.2 s after sequence 5): sequence 6, Papalymo's 84
--   every enemy but the beast down (the conjurer last, 99.9 s): sequence 7 at once (its health at 0, not its death
--           animation's end); 2.5 s later the beast runs back the way it came, Yda and Papalymo after it; 9.1 s
--           after sequence 7 "Duty Complete" (60001) and sequence 8, the beast and Claxio fade out (taken away 1.4 s
--           later), then the battle's scene 8 (0xF8402EFB); the quest moves to sequence 255
--   you down: sequence 0xFE
-- How hard each hits is in their timelines (QB57*).

-- Where retail puts the player and which way they face: their SpawnPlayer in the capture.
START_POSITION = { x = -288.22, y = 12.51, z = -52.01 }
START_ROTATION = -2.229
-- Retail's opening scene 1 here (Bob's Chronofoil capture 5fcc970c.)
OPENING_SCENE_FLAGS = 0xF8412EFB

BNPC_YDA = 4255892
BNPC_PAPALYMO = 4255894
BNPC_CLAXIO = 4256371
BNPC_LEADER = 4255868
BNPC_BEAST = 4255903
-- The first wave: { [thaumaturge] = the ally it goes for (nil: you) }.
THAUMATURGES = { [4255869] = BNPC_YDA, [4255870] = BNPC_PAPALYMO, [4255871] = false }
-- The second wave besides the beast, on you.
SECOND_WAVE = { 4255896, 4255897 }
BNPC_CONJURER = 4255896

QUEST_SEQUENCE_AFTER_WIN = 255
SEQUENCE_FIGHTING = 2
SEQUENCE_ADDS_DOWN = 7
SEQUENCE_WON = 8
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 8
FINISH_SCENE_FLAGS = 0xF8402EFB
FAIL_DELAY_UPDATES = 12

-- Timers, in updates (half a second each): a timer of n updates fires n / 2 s after it's set by another timer, and
-- (n - 1) / 2 s after it's set outside them (onCommence, a death: counted down from that same update.)
SEQUENCE_4_AFTER = 21
SEQUENCE_5_AFTER = 20
WON_AFTER = 19
-- The beast gives up and runs back the way it came 2.5 s after sequence 7 (retail's last hit at 1.8 s, its first step
-- at 2.5 s), and is faded out as "Duty Complete" plays, then taken away 1.4 s later, with Claxio (ActorControl 607
-- from each, then their DeleteActors 1.4 s on; Bob's Chronofoil capture 5fcc970c.)
RETREAT_AFTER = 6
FADE_SECONDS = 1.4
-- Retail's director answered the first thaumaturge's death 2 s after its health reached 0, as the death animation
-- ended; the last add's at once (sequence 7 came with its health at 0, its Defeated 2 s later.)
DEATH_ANIMATION_UPDATES = 4
-- The leader's health (percent) that brings sequence 6, after sequence 5: retail's came with the hit that took it from
-- 43 to 35, so the mark is somewhere in between.
LEADER_HEALTH = 35
-- The beast's area attack the director has it use 3.9 s into sequence 4 (on itself.)
ACTION_BEAST = 604

-- Who's talking (DirectorPopupMessage), with retail's portraits; Claxio by his ENpcResident (an event NPC: kind 3.)
SPEAKER_LEADER = { layout = BNPC_LEADER, name = 583, icon = 73207 }
SPEAKER_CLAXIO = { layout = BNPC_CLAXIO, name = 1006684, icon = 73208, kind = 3 }
SPEAKER_YDA = { layout = BNPC_YDA, name = 619, icon = 73156 }
SPEAKER_PAPALYMO = { layout = BNPC_PAPALYMO, name = 620, icon = 73164 }

BATTLE = Battle.new()
ACTION_CURE = 120
for _, ally in ipairs({ BNPC_YDA, BNPC_PAPALYMO }) do
    BATTLE:ally(ally, {
        role = "dps",
        split = false,
        focus = { "mine:" .. ally, "thaumaturges" },
        order = "oldest",
        range = ally == BNPC_PAPALYMO and 10 or nil,
        heals = ally == BNPC_PAPALYMO
            and { { action = ACTION_CURE, targets = { Battle.PLAYER }, below = 62, every = 14 } } or nil,
    })
end
for thaumaturge, ally in pairs(THAUMATURGES) do
    BATTLE:enemy(thaumaturge, {
        group = ally and { "mine:" .. ally, "thaumaturges" } or "thaumaturges",
        attacks = ally or nil,
    })
end
BATTLE:enemy(BNPC_LEADER, { group = "leader", attacks = BNPC_PAPALYMO })
BATTLE:enemy(BNPC_BEAST, { group = "boss", attacks = BNPC_YDA })
for _, add in ipairs(SECOND_WAVE) do
    BATTLE:enemy(add, {
        group = "adds",
        heals = add == BNPC_CONJURER and { { action = ACTION_CURE, targets = "allies", below = 35, every = 13 } } or nil,
    })
end

local over = false
local fail_in = nil
local phase = 1
local adds_left = 0
-- Timers of their own, side by side: { updates left, fn }.
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

function sortedKeys(t)
    local keys = {}
    for key in pairs(t) do
        table.insert(keys, key)
    end
    table.sort(keys)
    return keys
end

function secondWave(director)
    if phase < 7 then
        phase = 3
    end
    director:set_sequence(3)
    talk(director, SPEAKER_LEADER, 80)
    BATTLE:spawn(director, { BNPC_BEAST, SECOND_WAVE[1], SECOND_WAVE[2] })
    after(SEQUENCE_4_AFTER, function(d)
        if phase >= 7 then
            return -- everything but the beast already down
        end
        phase = 4
        d:set_sequence(4)
        -- Yda and Papalymo leave the rest to you.
        BATTLE:change(d, "the beast", {
            [BNPC_YDA] = { focus = { "boss" } },
            [BNPC_PAPALYMO] = { focus = { "boss" } },
        })
        after(4, function(d2) talk(d2, SPEAKER_PAPALYMO, 81) end)
        after(8, function(d2) d2:use_action(BNPC_BEAST, ACTION_BEAST, BNPC_BEAST) end)
        after(14, function(d2) talk(d2, SPEAKER_CLAXIO, 82) end)
        after(16, function(d2)
            talk(d2, SPEAKER_YDA, 85)
            talk(d2, SPEAKER_PAPALYMO, 86)
        end)
        after(SEQUENCE_5_AFTER, function(d2)
            if phase >= 7 then
                return
            end
            phase = 5
            d2:set_sequence(5)
            BATTLE:engage(d2, { BNPC_LEADER }, nil)
            after(4, function(d3) talk(d3, SPEAKER_YDA, 83) end)
        end)
    end)
end

function onSetup(director)
    director:set_sequence(1)
end

function onCommence(director)
    director:set_sequence(SEQUENCE_FIGHTING)
    director:spawn_bnpc(BNPC_CLAXIO)
    BATTLE:spawn(director, { BNPC_YDA, BNPC_PAPALYMO })
    local first = sortedKeys(THAUMATURGES)
    table.insert(first, BNPC_LEADER)
    BATTLE:spawn(director, first)
    adds_left = #first
    after(11, function(d) talk(d, SPEAKER_LEADER, 78) end)
    after(21, function(d) talk(d, SPEAKER_CLAXIO, 79) end)
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
    if phase == 5 then
        local leader = health[BNPC_LEADER]
        if leader == nil or leader <= LEADER_HEALTH then
            phase = 6
            director:set_sequence(6)
            talk(director, SPEAKER_PAPALYMO, 84)
        end
    end
    BATTLE:update(director, health, player_health, info)
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    if layout_id == BNPC_BEAST or layout_id == BNPC_YDA or layout_id == BNPC_PAPALYMO then
        return
    end
    adds_left = adds_left - 1
    local first_thaumaturge = phase == 1 and THAUMATURGES[layout_id] ~= nil
    if first_thaumaturge then
        phase = 2
        adds_left = adds_left + #SECOND_WAVE -- coming
    end
    local all_down = adds_left <= 0 and phase >= 2 and phase < 7
    if all_down then
        phase = 7
    end
    if first_thaumaturge then
        after(DEATH_ANIMATION_UPDATES, secondWave)
    end
    if all_down then
        allDown(director)
    end
end

-- Every enemy but the beast down.
function allDown(director)
    director:set_sequence(SEQUENCE_ADDS_DOWN)
    -- Yda and Papalymo chase it as it goes (retail's kept after it until the closing scene.)
    after(RETREAT_AFTER, function(d) d:retreat_bnpc(BNPC_BEAST) end)
    after(WON_AFTER, function(d)
        over = true
        timers = {}
        d:set_sequence(SEQUENCE_WON)
        d:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS)
        d:fade_out_bnpc(BNPC_BEAST)
        d:fade_out_bnpc(BNPC_CLAXIO)
        d:delay(FADE_SECONDS)
        d:despawn_bnpc(BNPC_BEAST)
        d:despawn_bnpc(BNPC_CLAXIO)
    end)
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
