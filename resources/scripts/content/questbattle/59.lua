-- Quest battle: The Lominsan Way (GaiUsb004_00811.)
-- It takes place in territory 307, whose TerritoryType has Lower La Noscea's level (s1f2), so its NPCs live in the
-- sea_s1f2 drop-in (layer QB_GaiUsb004_001): the one who talks and fights (base 1769, 4295469, BNpcName 2073, 677 HP),
-- the enemy you defeat to win (1770, 4295470, 3900 HP), the NPC you protect (1771, 4295472, battalion 5, 2418 HP), a
-- bomb that comes out late (1772, 4295473, 897 HP) and an onlooker (an event NPC, 1007082, 4295474). Layout IDs,
-- levels, HP, positions, flags, looks and the start spot are retail's (Bob's Chronofoil capture 44df02fc of
-- 2026-11-03: six runs, five lost and the last won, `questkit chronofoil --battle 0.4` to `0.14`; capture c9145fdc of
-- 2026-11-03, `--battle 0.5`, won; a level 34-35 player.)
--
-- Retail's flow (seconds from the fight starting, the director's scene 60000):
--   the opening scene 1 (0x2001, over at once), then the director's scene 2 (0x2001: the client waits 16 s) with
--           sequence 2 and three lines: 1769's 92 and 93 at -13.0 and -8.0, the onlooker's 94 (kind 3) at -3.0; as
--           scene 2 ends, scene 60000 and sequence 3: the fight. Kawari has no scene 60000: onCommence plays scene 2
--           and the fight starts as it ends (onSceneFinished)
--   0       sequence 3: 1769 on 1771 the whole fight (it never turned on the player, even when they hit it), 1770 and
--           1771 stand and do nothing; 1769's NpcYells 1274-1278 at 3.1, 8.1, 13.1, 18.1, 23.1
--   1770 at 80 % (77.4-78.7 the first health seen under it, five runs): sequence 4 and 1769's yell 1285 (0.1 s
--           later); 1.7-3.6 s later (2.95 the median) 1769 casts Paralyze (1118, 3.7 s) on you, and 5.0 s after its
--           cast began (4.95-5.06) sequence 5, whatever 1770's health (59.9-70.1), and 1769's 95 and 96 3.0 and 8.0 s
--           later; 1770 at 50 %: sequence 6; at 40 %: sequence 7 and 1769's 97 at once, 3.0-3.1 s later sequence 8
--           and 0.3-0.4 s after that the bomb (every run that got there: 1770's health then 33-36 %). A run that never
--           got 1770 to 80 % had no Paralyze.
--   the bomb: it stands where it came out and 32.0 s after (31.8 in the other run) blows (1133, round itself: 1771
--           and the player beside it fell) unless killed first (three of five): either way sequence 9; killed, 1769's
--           yell 1279 3.0-3.1 s after and its 98 8.0-8.1 s after; blown, it falls 2.0 s later (Kill). While it's out
--           the director's log lines count it down: 3217, 3218 and 3219 ("15/10/5 seconds to self-destruct...") 14.6,
--           19.6 and 24.7 s after it came out, while it still stands (every run: the EventLogMessage from the director)
--   1769 at sequence 8 runs off to hide, (704.37, 66.09, -279.13), from beside 1771 (2.2 s at a run), turns to -129
--           degrees and does nothing while the bomb is out; 7.0-7.2 s after it's gone it runs back to (694.97, 66.45,
--           -286.64) and takes up 1771 again (four runs that got there)
--   1769's Rive (1299, a line in front of itself) is aimed at the player every time, whoever it's fighting (19
--           casts: within a degree of the player's bearing), its timeline's `facing`
--   1770's health at 0: "Duty Complete" (60001) and sequence 10 at once, the battle's scene 10 (0xF8402EFB) 3.0 s
--           later; the quest moves to sequence 255
--   lost, sequence 0xFE at once: 1771's health at 0 (three runs: the bomb twice, 1769 after 217 s once), 1769's (two
--           runs: 1769's 99 with it) or yours; out about 5 s later
-- How hard each hits is in their timelines (QB59*).

-- Where retail puts the player and which way they face: their SpawnPlayer in the capture.
START_POSITION = { x = 704.42, y = 65.78, z = -290.68 }
START_ROTATION = -2.07
-- Retail's opening scene 1 here: no cutscene, over at once (Bob's Chronofoil capture 44df02fc.)
OPENING_SCENE_FLAGS = 0x2001 -- HIDE_HOTBAR | NO_DEFAULT_CAMERA

BNPC_ATTACKER = 4295469
BNPC_FOE = 4295470
BNPC_PROTECTED = 4295472
BNPC_BOMB = 4295473
ENPC_ONLOOKER = 4295474

-- The director's scene before the fight (the client's Wait(480): 16 s) and its flags, retail's.
LEAD_IN_SCENE = 2
LEAD_IN_SCENE_FLAGS = 0x2001 -- HIDE_HOTBAR | NO_DEFAULT_CAMERA

-- 1770's health (percent) for sequences 4, 6 and 7; 5 follows 4's Paralyze.
SEQUENCE_HEALTH = { [4] = 80, [6] = 50, [7] = 40 }
ACTION_PARALYZE = 1118
-- In updates: Paralyze 3.0 s after sequence 4, sequence 5 5.0 s after it began.
PARALYZE_AFTER = 7
SEQUENCE_5_AFTER_PARALYZE = 10
-- The bomb's blast (1133) and when (in updates: 32.0 s after it came out.)
ACTION_LAST_RESORT = 1133
LAST_RESORT_AFTER = 64
-- It falls 2.0 s after its blast (retail's Kill.)
BOMB_FALLS_AFTER = 4

-- How much 1769 hates 1771: it never turns away.
ATTACKER_ON_PROTECTED = 1000000
-- Where 1769 hides while the bomb is out, the way it faces there, where it comes back to, and how near counts as
-- there (retail's, every run.)
ATTACKER_HIDES_AT = { x = 704.37, y = 66.09, z = -279.13 }
ATTACKER_HIDING_ROTATION = -2.25
ATTACKER_COMES_BACK_TO = { x = 694.97, y = 66.45, z = -286.64 }
ATTACKER_THERE = 1.0
-- In updates: it comes back 7.0 s after the bomb's gone.
ATTACKER_BACK_AFTER = 14
-- The bomb's countdown: the director's log lines and when (in updates after it came out: 14.6, 19.6, 24.7 s.)
BOMB_COUNTDOWN = { { 29, 3217 }, { 39, 3218 }, { 49, 3219 } }

QUEST_SEQUENCE_AFTER_WIN = 255
SEQUENCE_FIGHTING = 3
SEQUENCE_BOMB = 8
SEQUENCE_BOMB_GONE = 9
SEQUENCE_WON = 10
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 10
FINISH_SCENE_FLAGS = 0xF8402EFB
FAIL_DELAY_UPDATES = 10

-- Who's talking (DirectorPopupMessage), with retail's names and portraits: 1769 as a BattleNpc, the onlooker as an
-- EventNpc (kind 3, its ENpcResident.)
SPEAKER_ATTACKER = { layout = BNPC_ATTACKER, name = 2073, icon = 73196 }
SPEAKER_ONLOOKER = { layout = ENPC_ONLOOKER, name = 1007082, icon = 73197, kind = 3 }
-- 1769's NpcYells: through the fight's first 25 s, at 80 % and after the bomb is killed.
FIGHT_YELLS = { 1274, 1275, 1276, 1277, 1278 }
YELL_FOE_HURT = 1285
YELL_BOMB_KILLED = 1279

local phase = 1
local over = false
local fail_in = nil
local bomb_actor = nil
local bomb_blown = false
-- 1769: nil, "hiding" (on its way), "hidden" (held there), "coming back".
local attacker_away = nil
-- Timers of their own, side by side: { updates left, fn }.
local timers = {}

-- A timer of n updates fires n / 2 s after it's set by another timer, and (n - 1) / 2 s after it's set outside them
-- (onCommence, onSceneFinished, a death, onUpdate: counted down from that same update.)
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

function onSetup(director)
    director:set_sequence(1)
end

-- The opening has ended: everyone is out (retail had them from the moment you arrived), standing, and the director's
-- scene 2 plays with its three lines.
function onCommence(director)
    phase = 2
    director:set_sequence(2)
    director:spawn_bnpc(BNPC_PROTECTED)
    director:spawn_bnpc(BNPC_FOE)
    director:spawn_bnpc(BNPC_ATTACKER)
    director:spawn_bnpc(ENPC_ONLOOKER)
    for _, layout in ipairs({ BNPC_PROTECTED, BNPC_FOE, BNPC_ATTACKER }) do
        director:hold_bnpc(layout)
    end
    director:play_scene(LEAD_IN_SCENE, LEAD_IN_SCENE_FLAGS)
    after(7, function(d) talk(d, SPEAKER_ATTACKER, 92) end)
    after(17, function(d) talk(d, SPEAKER_ATTACKER, 93) end)
    after(27, function(d) talk(d, SPEAKER_ONLOOKER, 94) end)
end

-- Scene 2 over: the fight (retail's scene 60000 and sequence 3.)
function onSceneFinished(director, scene)
    if scene ~= LEAD_IN_SCENE or phase ~= 2 or over then
        return
    end
    phase = SEQUENCE_FIGHTING
    director:set_sequence(SEQUENCE_FIGHTING)
    director:hold_bnpc(BNPC_ATTACKER, false)
    director:set_hate(BNPC_ATTACKER, BNPC_PROTECTED, ATTACKER_ON_PROTECTED)
    for i, yell in ipairs(FIGHT_YELLS) do
        after(7 + 10 * (i - 1), function(d) d:yell(BNPC_ATTACKER, yell) end)
    end
end

-- Sequences 4, 6 and 7, as 1770's health falls (one a tick, in order); 5 comes with 4's Paralyze.
function advance(director, foe)
    local next = phase + 1
    local below = SEQUENCE_HEALTH[next]
    if below == nil or foe > below then
        return
    end
    phase = next
    director:set_sequence(next)
    if next == 4 then
        director:yell(BNPC_ATTACKER, YELL_FOE_HURT)
        after(PARALYZE_AFTER, function(d)
            d:use_action(BNPC_ATTACKER, ACTION_PARALYZE)
            after(SEQUENCE_5_AFTER_PARALYZE, sequence5)
        end)
    elseif next == 7 then
        talk(director, SPEAKER_ATTACKER, 97)
        after(7, bombComes)
    end
end

function sequence5(director)
    if over or phase ~= 4 then
        return
    end
    phase = 5
    director:set_sequence(5)
    after(6, function(d) talk(d, SPEAKER_ATTACKER, 95) end)
    after(16, function(d) talk(d, SPEAKER_ATTACKER, 96) end)
end

function bombComes(director)
    if over then
        return
    end
    phase = SEQUENCE_BOMB
    director:set_sequence(SEQUENCE_BOMB)
    -- 1769 runs off to hide as its bomb comes out.
    attacker_away = "hiding"
    director:retreat_bnpc(BNPC_ATTACKER, ATTACKER_HIDES_AT.x, ATTACKER_HIDES_AT.y, ATTACKER_HIDES_AT.z)
    after(1, function(d)
        bomb_actor = d:spawn_bnpc(BNPC_BOMB)
        d:hold_bnpc(BNPC_BOMB)
        after(LAST_RESORT_AFTER, blowBomb)
        for _, step in ipairs(BOMB_COUNTDOWN) do
            after(step[1], function(d2)
                if not over and phase == SEQUENCE_BOMB then
                    d2:log_message(step[2], {})
                end
            end)
        end
    end)
end

-- The bomb's gone (killed, or fallen after its blast): 1769 comes back to 1771 7 s later.
function bombGone(director)
    after(ATTACKER_BACK_AFTER, function(d)
        if over or attacker_away == nil then
            return
        end
        attacker_away = "coming back"
        d:hold_bnpc(BNPC_ATTACKER, false)
        d:retreat_bnpc(BNPC_ATTACKER, ATTACKER_COMES_BACK_TO.x, ATTACKER_COMES_BACK_TO.y, ATTACKER_COMES_BACK_TO.z)
    end)
end

function flatDistance(a, b)
    return math.sqrt((a.x - b.x) ^ 2 + (a.z - b.z) ^ 2)
end

-- 1769 getting where it was going: held where it hides, facing retail's way; back beside 1771, after it again.
function attackerArrives(director, info)
    local attacker = info ~= nil and info[BNPC_ATTACKER] or nil
    if attacker == nil or attacker_away == nil then
        return
    end
    if attacker_away == "hiding" and flatDistance(attacker, ATTACKER_HIDES_AT) <= ATTACKER_THERE then
        attacker_away = "hidden"
        director:hold_bnpc(BNPC_ATTACKER)
        director:move_bnpc(BNPC_ATTACKER, attacker.x, attacker.y, attacker.z, ATTACKER_HIDING_ROTATION)
    elseif attacker_away == "coming back" and flatDistance(attacker, ATTACKER_COMES_BACK_TO) <= ATTACKER_THERE then
        attacker_away = nil
        director:set_hate(BNPC_ATTACKER, BNPC_PROTECTED, ATTACKER_ON_PROTECTED)
    end
end

-- 32 s after it came out, unless it was killed first.
function blowBomb(director)
    if over or phase ~= SEQUENCE_BOMB then
        return
    end
    bomb_blown = true
    phase = SEQUENCE_BOMB_GONE
    director:actor_use_action(bomb_actor, ACTION_LAST_RESORT)
    director:set_sequence(SEQUENCE_BOMB_GONE)
    after(BOMB_FALLS_AFTER, function(d)
        d:kill_bnpc(BNPC_BOMB)
        bombGone(d)
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
    attackerArrives(director, info)
    local foe = health[BNPC_FOE]
    if foe ~= nil and phase >= SEQUENCE_FIGHTING and phase < 7 then
        advance(director, foe)
    end
end

function lose(director)
    over = true
    timers = {}
    director:set_sequence(SEQUENCE_FAILED)
    fail_in = FAIL_DELAY_UPDATES
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    if layout_id == BNPC_FOE then
        over = true
        timers = {}
        director:set_sequence(SEQUENCE_WON)
        director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS)
    elseif layout_id == BNPC_PROTECTED then
        lose(director)
    elseif layout_id == BNPC_ATTACKER then
        talk(director, SPEAKER_ATTACKER, 99)
        lose(director)
    elseif layout_id == BNPC_BOMB and not bomb_blown then
        phase = SEQUENCE_BOMB_GONE
        director:set_sequence(SEQUENCE_BOMB_GONE)
        bombGone(director)
        after(7, function(d) d:yell(BNPC_ATTACKER, YELL_BOMB_KILLED) end)
        after(17, function(d) talk(d, SPEAKER_ATTACKER, 98) end)
    end
end

-- Losing sends you back to where you entered, still on the same step of the quest, so you can try again.
function onPlayerDeath(director)
    if over then
        return
    end
    lose(director)
end
