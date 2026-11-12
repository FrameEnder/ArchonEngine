-- Quest battle 52: Pincer Maneuver (ClsAcn200_00459, the Arcanists' level 20 quest.)
-- It takes place in territory 287, whose level is Lower La Noscea's (s1f2), so its NPCs are layer QB_ClsAcn200_001 of
-- sea_s1f2: Thubyrgeim the Transpicuous on your side (4256744, base 2061, 2401 HP) with his Topaz Carbuncle (4256749,
-- 2062), and Aersthota Morningcalm (4256747, 2064) and Geissfryn Eveningcalm (4256746, 2063), who come from behind
-- Kikiroon's mercenaries; against you three waves of mercenaries (menacing 2060, gladiators; mindful 2059, archers;
-- Mamool Ja 2065, conjurers, and 2066, lancers; a melancholy one 2067, a thaumaturge.) Then K'lyhia's grimoire is there
-- to be picked up. Layout IDs, levels, HP, positions, display and character data flags, looks and the start spot are
-- retail's (Bob's Kawari Recorder run of 2026-11-12, `questkit battle`: one run, won, a level 24 arcanist.)
--
-- Retail's flow (seconds from the director's scene 60000):
--   0       sequence 2 (0.1 s after): three menacing mercenaries on Thubyrgeim, two mindful ones (one on him, one on
--           you); battle talk 95 (the Qiqirn trader, an event NPC), 96 and 97 (Thubyrgeim) at 3.1, 8.1 and 13.1;
--           Aersthota and Geissfryn at 15.3
--   Thubyrgeim's Summon II (1307, a 5.7 s cast 5.1 s in): his Topaz Carbuncle 7.3 s after it began
--   three of the first five down (29.2): sequence 3 at once; the second wave 0.5 and 0.7 s later (three menacing, two
--           Mamool Ja, three mindful); talks 98, 99, 100 5.1, 15.1 and 20.1 s after sequence 3
--   six of the second wave's eight down (95.4): sequence 4 at once, the third wave 0.7 s later (two menacing, two
--           mindful, two Mamool Ja, a melancholy one); talks 101, 102, 103 5.1, 15.1 and 20.1 s after it
--   every mercenary down (134.9): sequence 5 at once, the grimoire (EObj 2002521) 0.7 s later, talk 107 1.1 s after
--   the grimoire picked up (a 2 s bar, 31): sequence 6, ActorControl 607 on the grimoire, "You obtain K'lyhia's
--           grimoire!" (LogMessage 3223) from the director, the grimoire gone 1.3 s later; "Duty Complete" (60001) and
--           sequence 7 5.1 s after sequence 6, the battle's scene 7 (0xF8402EFB) 3.1 s after that; the quest to 255
-- What each one does is in their timelines (QB52*); the Mamool Ja of the second wave Cure the others below 40 %.
-- Retail faded another grimoire (EObj 2002520, there from the start, the zone's own) with the one picked up; it isn't
-- one of the battle's, so it's left alone.

-- Where retail put the player and which way they faced.
START_POSITION = { x = 63.9, y = 71.29, z = 369.74 }
START_ROTATION = -0.87
-- Retail's opening scene 1 here.
OPENING_SCENE_FLAGS = 0xF8412EFB

BNPC_THUBYRGEIM = 4256744
BNPC_CARBUNCLE = 4256749
BNPC_GEISSFRYN = 4256746
BNPC_AERSTHOTA = 4256747
ENPC_TRADER = 4257118
-- The first wave and whom each goes for (nil: you.)
WAVE_1 = { [4256753] = BNPC_THUBYRGEIM, [4256754] = BNPC_THUBYRGEIM, [4256755] = BNPC_THUBYRGEIM,
    [4256750] = BNPC_THUBYRGEIM, [4256751] = false }
WAVE_2_FIRST = { 4256763, 4256766, 4256772 }
WAVE_2_SECOND = { 4256762, 4256761, 4256760, 4256759, 4256758 }
WAVE_2 = { [4256763] = BNPC_THUBYRGEIM, [4256762] = BNPC_THUBYRGEIM, [4256761] = BNPC_THUBYRGEIM,
    [4256772] = BNPC_THUBYRGEIM, [4256766] = BNPC_GEISSFRYN, [4256760] = BNPC_AERSTHOTA, [4256759] = BNPC_GEISSFRYN,
    [4256758] = BNPC_GEISSFRYN }
WAVE_2_HEALERS = { [4256766] = true, [4256772] = true }
WAVE_3 = { [4256788] = BNPC_THUBYRGEIM, [4304848] = BNPC_THUBYRGEIM, [4256782] = BNPC_THUBYRGEIM,
    [4256783] = BNPC_THUBYRGEIM, [4256777] = BNPC_AERSTHOTA, [4256776] = BNPC_GEISSFRYN, [4256789] = BNPC_GEISSFRYN }
WAVE_1_DOWN_FOR_WAVE_2 = 3
WAVE_2_DOWN_FOR_WAVE_3 = 6

EOBJ_GRIMOIRE = 2002521
GRIMOIRE_POSITION = { x = -10.21, y = 67.5, z = 416.09 }
PICK_UP_BAR = 31
LOG_OBTAINED = 3223

QUEST_SEQUENCE_AFTER_WIN = 255
SEQUENCE_FIGHTING = 2
SEQUENCE_SECOND_WAVE = 3
SEQUENCE_THIRD_WAVE = 4
SEQUENCE_GRIMOIRE = 5
SEQUENCE_PICKED_UP = 6
SEQUENCE_WON = 7
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 7
FINISH_SCENE_FLAGS = 0xF8402EFB
FAIL_DELAY_UPDATES = 12

ACTION_SUMMON_II = 1307
ACTION_PHYSICK = 190
ACTION_CURE = 120

-- Who's talking (DirectorPopupMessage), with retail's names; the trader as an EventNpc (kind 3.)
SPEAKER_THUBYRGEIM = { layout = BNPC_THUBYRGEIM, name = 1496, icon = 0 }
SPEAKER_TRADER = { layout = ENPC_TRADER, name = 1005185, icon = 0, kind = 3 }

-- Timers, in updates (half a second each). A timer of n fires (n - 1) / 2 s after it's set before the next update
-- (onCommence, onActorDeath, onEventActionCast.)
TALKS_FIGHTING = { { 7, SPEAKER_TRADER, 95 }, { 17, SPEAKER_THUBYRGEIM, 96 }, { 27, SPEAKER_THUBYRGEIM, 97 } }
TALKS_SECOND_WAVE = { { 11, SPEAKER_TRADER, 98 }, { 31, SPEAKER_THUBYRGEIM, 99 }, { 41, SPEAKER_THUBYRGEIM, 100 } }
TALKS_THIRD_WAVE = { { 11, SPEAKER_TRADER, 101 }, { 31, SPEAKER_THUBYRGEIM, 102 }, { 41, SPEAKER_THUBYRGEIM, 103 } }
TALKS_GRIMOIRE = { { 3, SPEAKER_THUBYRGEIM, 107 } }
PINCER_AFTER = 31
WAVE_2_FIRST_AFTER = 2
WAVE_2_SECOND_AFTER = 2
WAVE_3_AFTER = 2
GRIMOIRE_AFTER = 2
CARBUNCLE_AFTER_CAST = 15
GRIMOIRE_GOES_AFTER = 4
WON_AFTER = 11

BATTLE = Battle.new()
-- Thubyrgeim casts from range and heals Aersthota and Geissfryn (retail's Physick on them); Aersthota holds the
-- mercenaries, Geissfryn heals her (Cure); the carbuncle takes the oldest up close.
BATTLE:ally(BNPC_THUBYRGEIM, {
    role = "dps",
    focus = { "mercenaries" },
    range = 15,
    heals = { { action = ACTION_PHYSICK, targets = { BNPC_AERSTHOTA, BNPC_GEISSFRYN }, below = 50, every = 5 } },
})
BATTLE:ally(BNPC_CARBUNCLE, { role = "dps", focus = { "mercenaries" }, order = "oldest", range = 3 })
BATTLE:ally(BNPC_AERSTHOTA, { role = "tank", focus = { "mercenaries" }, range = 3 })
BATTLE:ally(BNPC_GEISSFRYN, {
    role = "dps",
    focus = { "mercenaries" },
    range = 15,
    heals = { { action = ACTION_CURE, targets = { BNPC_AERSTHOTA, BNPC_GEISSFRYN }, below = 50, every = 6 } },
})
for _, wave in ipairs({ WAVE_1, WAVE_2, WAVE_3 }) do
    for layout, ally in pairs(wave) do
        local role = { group = "mercenaries", attacks = ally or nil }
        if WAVE_2_HEALERS[layout] then
            role.heals = { { action = ACTION_CURE, targets = { "allies" }, below = 40, every = 5 } }
        end
        BATTLE:enemy(layout, role)
    end
end

local over = false
local winning = false
local fail_in = nil
local phase = 1
local wave_1_down = 0
local wave_2_down = 0
local carbuncle_out = false
local picked_up = false
local grimoire = nil
local standing = {}
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

function talks(lines)
    for _, line in ipairs(lines) do
        local speaker, text = line[2], line[3]
        after(line[1], function(d)
            d:battle_talk(speaker.layout, speaker.name, text, speaker.icon, nil, speaker.kind)
        end)
    end
end

function bringOut(director, layouts)
    for _, layout in ipairs(layouts) do
        standing[layout] = true
    end
    BATTLE:spawn(director, layouts)
end

function keys(wave)
    local list = {}
    for layout, _ in pairs(wave) do
        table.insert(list, layout)
    end
    table.sort(list)
    return list
end

function sequence(director, value)
    phase = value
    director:set_sequence(value)
end

function onSetup(director)
    director:set_sequence(1)
end

function onCommence(director)
    sequence(director, SEQUENCE_FIGHTING)
    BATTLE:spawn(director, { BNPC_THUBYRGEIM })
    bringOut(director, keys(WAVE_1))
    talks(TALKS_FIGHTING)
    after(PINCER_AFTER, function(d) BATTLE:spawn(d, { BNPC_AERSTHOTA, BNPC_GEISSFRYN }) end)
end

-- His Summon II brings his Topaz Carbuncle.
function onActorCast(director, layout_id, action_id)
    if over or layout_id ~= BNPC_THUBYRGEIM or action_id ~= ACTION_SUMMON_II or carbuncle_out then
        return
    end
    carbuncle_out = true
    after(CARBUNCLE_AFTER_CAST, function(d) BATTLE:spawn(d, { BNPC_CARBUNCLE }) end)
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
    BATTLE:update(director, health, player_health, info)
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    if not standing[layout_id] then
        return
    end
    standing[layout_id] = nil
    if WAVE_1[layout_id] ~= nil then
        wave_1_down = wave_1_down + 1
        if wave_1_down == WAVE_1_DOWN_FOR_WAVE_2 and phase == SEQUENCE_FIGHTING then
            sequence(director, SEQUENCE_SECOND_WAVE)
            after(WAVE_2_FIRST_AFTER, function(d) bringOut(d, WAVE_2_FIRST) end)
            after(WAVE_2_SECOND_AFTER, function(d) bringOut(d, WAVE_2_SECOND) end)
            talks(TALKS_SECOND_WAVE)
        end
    elseif WAVE_2[layout_id] ~= nil then
        wave_2_down = wave_2_down + 1
        if wave_2_down == WAVE_2_DOWN_FOR_WAVE_3 and phase == SEQUENCE_SECOND_WAVE then
            sequence(director, SEQUENCE_THIRD_WAVE)
            after(WAVE_3_AFTER, function(d) bringOut(d, keys(WAVE_3)) end)
            talks(TALKS_THIRD_WAVE)
        end
    end
    -- Every mercenary down once the third wave has come: the grimoire.
    if phase == SEQUENCE_THIRD_WAVE and next(standing) == nil then
        timers = {}
        sequence(director, SEQUENCE_GRIMOIRE)
        after(GRIMOIRE_AFTER, function(d) d:spawn_eobj(EOBJ_GRIMOIRE, GRIMOIRE_POSITION) end)
        talks(TALKS_GRIMOIRE)
    end
end

-- Picking the grimoire up: the director's event action, no scene (retail's.)
function onObjectTalk(director, actor_id, object_id, base_id)
    if base_id ~= EOBJ_GRIMOIRE or phase ~= SEQUENCE_GRIMOIRE or picked_up or over then
        director:finish_gimmick(actor_id)
        return
    end
    grimoire = object_id
    director:event_action(PICK_UP_BAR, actor_id, object_id)
end

function onEventActionCast(director, actor_id, target)
    director:finish_gimmick(actor_id)
    if picked_up or over or phase ~= SEQUENCE_GRIMOIRE then
        return
    end
    picked_up = true
    winning = true
    sequence(director, SEQUENCE_PICKED_UP)
    director:eobj_control(EOBJ_GRIMOIRE, 607, { grimoire or target, 1, 0, 100 })
    director:log_message(LOG_OBTAINED, {})
    after(GRIMOIRE_GOES_AFTER, function(d)
        d:delete_eobj(EOBJ_GRIMOIRE)
    end)
    after(WON_AFTER, function(d)
        over = true
        timers = {}
        sequence(d, SEQUENCE_WON)
        d:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS)
    end)
end

-- Losing sends you back to where you entered, still on the same step of the quest, so you can try again.
function onPlayerDeath(director)
    if over or winning then
        return
    end
    over = true
    timers = {}
    director:set_sequence(SEQUENCE_FAILED)
    fail_in = FAIL_DELAY_UPDATES
end
