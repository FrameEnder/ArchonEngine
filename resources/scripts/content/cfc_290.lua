-- The Jade Stoa (Byakko): ContentFinderCondition 290, territory 746, InstanceContent 20051 (director 0x80034E53).
--
-- From Bob's Chronofoil capture 84700114 (2026-11-24, visit 0.4: Bob a level 88 White Mage and a friend's level 86
-- Gunbreaker, unsynced, against the level 70 trial; won at the first attempt, 1235 s after the pull), with cactbot's
-- timeline for it (byakko.txt: the same order, 4.5 s off); `questkit chronofoil --battle` and the visit's packets,
-- numbers only (docs/src/fork/dungeons.md). Named `cfc_290` as its ShortCode isn't known here (the server falls back
-- to `content/cfc_<ContentFinderCondition>.lua`).
--
-- One clock from the pull, every part on it (Hakutei's health didn't end his: he stood at 54 % when it did):
--   0-107.6 s    Byakko: his raidwide, tank-buster, a seizing and throw, his knock-back, his orbs, three of 10796.
--   107.6-178    Hakutei in his place (BNpcBase 8440): the helpers' 10793 by each of the party and their puddles
--                (EObj 2002601: 273 to whoever stands in one), two 10802, his mark (87) and 10828, 10801, his 19.7 s
--                10827 on everyone.
--   176.9-290    the intermission: Byakko below, the party under 1479; chasing circles (10817 at each of the party,
--                10818 there 6.05 s on), 10822 twice from the middle, the leaper (8546) three times at the north edge,
--                10829 on everyone as he's back.
--   290.5-       Byakko's last part: a block once (183.1 s), then another every 205.9 s (205.1, 206.2 and 206.3 in the
--                run) till he falls: the eight circles (10808 and three steps of 10809), everyone marked (101) for
--                10811, 10800, two seizings, his raidwide twice (10799, 10830 6.2 s on), the orbs in threes.
-- Byakko's movements are his jumps (an ActorMove straight there): to the middle before his orbs and his 10796s there
-- (facing one of the party), to one of eight spots 19.6 yalms out before a 10796 from the edge (facing the middle),
-- and to one of a pair of spots 4 yalms out for his dash (10210, cast at the other one, where he comes down).
-- A seizing: 10208 (3.7 s) or 10070 (instant) stuns one of the party (201), 10209 seizes them (1477), a helper is put
-- 14 yalms out with marker 62 over it, he dashes, and 0.8 s after he's down the seized is thrown there (ActorControl
-- 220, GimmickJump row 10) and a helper's 10806 lands round them; the stun and the seizing go then.
-- The orbs (BNpcBase 8441) come out of the middle 2.5 s after his 10221, two (first part) or three (last part) 180 or
-- 120 degrees apart every 2 s, eleven times, the first three sets the same way and each after 18 degrees on (one way
-- or the other), at 2 yalms a second to 19.7 yalms out, where they fade; one goes off (10824) on whoever comes within
-- 1.5 yalms of it and falls 2 s later.
-- Not known, so guessed: the puddles' size (4 yalms here); whom Hakutei marks (one of the party at random; it was the
-- tank in the run); the eight circles' first set (diagonal or square, at random: both came first); where the leaper's
-- three spots come from (the run's, in its order); the director's 0x80000001 [522] and 0x80000008 at the intermission
-- and 0x80000009 (not sent); the two WalkInEvents at 184.1 s (not sent); what more parties than two change.

BYAKKO = 7231121
HAKUTEI = 7231122
ORB = 7231123
HELPER = 7231126
LEAPER = 7236115
HELPERS = 32

NAME_BYAKKO = 7092
TALK_MS = 4000
TALK_PULL = 18600

MIDDLE = { x = 0.0, y = 0.0, z = 0.0 }
-- Where Byakko stands as the duty begins (layout 7231121's spot), facing south toward the entrance.
BYAKKO_AT = { x = 0.0, y = 0.0, z = -7.05, rotation = 0.0 }
BELOW = { x = 0.0, y = -80.0, z = 0.0 }

-- His actions (the director's: content/cfc_290.lua drives them all; his timeline has his auto-attack).
RAIDWIDE = 10799
RAIDWIDE_AGAIN = 10830
RAIDWIDE_AGAIN_AFTER = 6.2
TANK_BUSTER = 10797
KNOCK_BACK = 10798
ROUND_HIM = 10800
CUT = 10796
BEFORE_JUMP = 10071
ORBS = 10221
STUN = 10208
STUN_NOW = 10070
SEIZE = 10209
DASH = 10210
CIRCLES_SHOWN = 10212
MARKED_SHOWN = 10215
LEAVES = 9681
BELOW_1 = 10222
BELOW_2 = 10821
BACK = 10794

HAKUTEI_COMES = 9679
CLAW = 10802
HERALD = 10828
LINE = 10801
HAKUTEI_9822 = 9822
HAKUTEI_10825 = 10825
ROAR = 10827

SPOTS = 10793
LANDING = 10806
CIRCLE = 10808
CIRCLE_STEP = 10809
ON_EACH = 10811
MARK = 10817
DROP = 10818
FROM_MIDDLE = 10822
AT_THE_END = 10829
ORB_BLAST = 10824
LEAPER_DROP = 10819

MARKER_EACH = 101
MARKER_LANDING = 62
MARKER_HERALD = 87
LOCKON = 34

STATUS_STUN = 201
STATUS_SEIZED = 1477
STATUS_ORBS = 1480
STATUS_PUDDLE = 273
STATUS_SKY = 1479
STATUS_SKY_ENDS = 1513
STATUS_SKY_ENDS_PARAM = 6146
STATUS_BACK = 783
STATUS_BACK_PARAM = 3783
STATUS_BACK_FOR = 5.0

PUDDLE = 2002601
PUDDLE_FOR = 7.6
PUDDLE_AFTER = 3.4
PUDDLE_RADIUS = 4.0

WEATHER_START = 92
WEATHER_HAKUTEI = 96
WEATHER_BELOW = 93
WEATHER_BACK = 94
WEATHER_FADE = 3.0

CUTSCENE_END = 1734
CUTSCENE_END_PARAMS = { 1, 18, 0, 0 }
-- After his fall (his health 0.18 % at the last look): the duty complete 2.1 s on, scene 3 1.6 s after that.
COMPLETE_AFTER = 2.1
CUTSCENE_AFTER = 3.6
TALK_LOW = 18613
LOW_AT = 10

-- The seizing (seconds from 10208's cast or 10070): 10209, his jump to the dash's start, the landing's marker, the
-- dash's cast, his coming down, the throw.
SEIZE_AFTER = { cast = 5.1, now = 1.1 }
JUMP_AFTER = 5.5
MARK_AFTER = { cast = 6.1, now = 2.8 }
DASH_AFTER = { cast = 6.3, now = 3.0 }
DASH_CAST = 4.7
THROW_AFTER_DOWN = 0.8
DASH_FROM_MIDDLE = 4.0
LANDING_FROM_MIDDLE = 14.0
TALK_SEIZE = { cast = 18601, now = 18602 }
TALK_SEIZE_AFTER = { cast = 4.9, now = 0.9 }

-- His jumps: 0.8 s after 10071, the cast 0.9 s after.
JUMP_AFTER_SIGN = 0.8
CAST_AFTER_JUMP = 0.9
EDGE = 19.6

-- The orbs.
ORBS_AFTER_CAST = 5.2
ORB_EVERY = 2.0
ORB_SETS = 11
ORB_SAME_FOR = 3
ORB_TURN = 18.0
ORB_SPEED = 2.0
ORB_GOES = 19.7
ORB_REACH = 1.5
ORB_FALLS_AFTER = 2.0
ORB_FIRST = { [2] = { -91.0, 91.0 }, [3] = { -62.0, -180.0, 60.0 } }

-- The eight circles: four at 5 yalms on the diagonals, four on the axes, a set 3.1 s after the other; each steps 5
-- yalms out three times, 6.15 s after its cast began and 1.07 s apart.
CIRCLE_SECOND_AFTER = 3.1
CIRCLE_STEPS_FROM = 6.15
CIRCLE_STEP_EVERY = 1.07
CIRCLE_STEPS = 3
CIRCLE_STEP_YALMS = 5.0
ON_EACH_AFTER = 8.1

-- Hakutei's part and the intermission, seconds after the pull.
HAKUTEI_FROM = 107.6
HAKUTEI_ROWS = {
    { 107.5, "talk", 18606 },
    { 111.7, "timeline", BYAKKO, 7737 },
    { 114.8, "weather", WEATHER_HAKUTEI },
    { 116.7, "spots" },
    { 121.8, "claw" },
    { 127.9, "claw" },
    { 134.0, "timeline", HAKUTEI, 7738 },
    { 134.6, "herald" },
    { 141.8, "line" },
    { 149.0, "visual", HAKUTEI_9822 },
    { 150.2, "visual", HAKUTEI_10825 },
    { 153.3, "roar" },
    { 176.5, "timeline", HAKUTEI, 7738 },
    { 178.5, "hakutei_goes" },
    { 178.5, "bgm", 522 },
    { 178.6, "timeline", BYAKKO, 4562 },
    { 176.9, "talk", 18607 },
    { 181.0, "talk", 18608, true },
    { 189.1, "weather", WEATHER_BELOW },
    { 198.7, "sky" },
    { 201.2, "talk", 18609 },
    { 204.3, "below", BELOW_1 },
    { 205.7, "marks" }, { 207.7, "marks" }, { 209.7, "marks" },
    { 213.4, "below", BELOW_2 }, { 213.4, "from_middle" },
    { 220.2, "leaper", 14.14, -14.14 },
    { 222.5, "below", BELOW_1 },
    { 223.9, "marks" }, { 225.9, "marks" }, { 227.9, "marks" }, { 229.9, "marks" }, { 231.9, "marks" },
    { 233.9, "marks" },
    { 234.2, "leaper", 0.0, -20.0 },
    { 237.7, "below", BELOW_2 }, { 237.7, "from_middle" },
    { 244.2, "leaper", -14.14, -14.14 },
    { 248.7, "talk", 18610 },
    { 254.2, "sky_ends" },
    { 252.3, "timeline", BYAKKO, 7738 },
    { 255.4, "byakko_up" },
    { 259.3, "back" },
    { 259.4, "timeline", HAKUTEI, 4576 },
    { 267.3, "bgm", 1 },
    { 277.9, "bgm", 523 },
    { 267.3, "targetable" },
    { 269.3, "talk", 18603, true },
    { 274.4, "at_the_end" },
    { 279.4, "talk", 18611, true },
    { 290.5, "fights" },
    { 290.5, "tank" },
}
MARK_DROPS_AFTER = 6.05
LEAPER_CASTS_AFTER = 1.1
ROAR_CAST = 19.7

-- Byakko's first part, seconds after the pull.
FIRST_ROWS = {
    { 6.9, "raidwide" },
    { 13.0, "tank" },
    { 24.3, "seize", "cast", true },
    { 38.3, "knock" },
    { 50.5, "tank" },
    { 61.6, "talk", 18612 },
    { 61.6, "orbs", 2 },
    { 68.1, "raidwide" },
    { 74.3, "cut_here" },
    { 81.8, "cut_here" },
    { 88.9, "cut_out" },
    { 105.8, "leaves" },
}

-- The last part's first block (once, from 300.7 s) and the one after it (over and over), seconds into the block.
LAST_FROM = 300.7
FIRST_BLOCK_FOR = 183.1
BLOCK_EVERY = 205.9
FIRST_BLOCK = {
    { 0.0, "circles" },
    { 8.2, "marked" },
    { 15.4, "round" },
    { 26.4, "circles" },
    { 28.7, "seize", "cast", true },
    { 42.6, "seize", "now", false },
    { 52.5, "knock" },
    { 58.5, "marked" },
    { 61.8, "cut_out" },
    { 78.7, "tank" },
    { 89.0, "knock" },
    { 99.0, "talk", 18612 },
    { 99.1, "orbs", 3 },
    { 105.4, "raidwide" },
    { 113.7, "cut_here" },
    { 120.8, "marked" },
    { 124.0, "cut_out" },
    { 141.1, "tank" },
    { 151.4, "raidwide" },
    { 159.8, "knock" },
    { 169.9, "raidwide" },
}
BLOCK = {
    { 0.0, "circles" },
    { 8.1, "marked" },
    { 15.3, "round" },
    { 22.4, "raidwide" },
    { 34.5, "circles" },
    { 36.9, "seize", "cast", true },
    { 50.7, "seize", "now", false },
    { 60.6, "knock" },
    { 66.6, "marked" },
    { 69.8, "cut_out" },
    { 84.6, "raidwide" },
    { 100.9, "tank" },
    { 111.1, "knock" },
    { 121.1, "talk", 18612 },
    { 121.2, "orbs", 3 },
    { 127.7, "raidwide" },
    { 136.1, "cut_here" },
    { 143.1, "marked" },
    { 146.4, "cut_out" },
    { 163.4, "tank" },
    { 173.6, "raidwide" },
    { 181.8, "knock" },
    { 191.9, "raidwide" },
}
-- How many later blocks are lined up at the pull (more than a fight lasts: retail's fell in the fifth.)
BLOCKS = 12

D = Dungeon.new()

attempt = 1
byakko = nil
hakutei = nil
leaper = nil
last_part = false
low_said = false
orbs = {}
puddles = {}
in_puddle = {}
dash_at = nil
seized = nil

function talk(director, text, from_nobody)
    director:battle_talk(from_nobody and 0 or BYAKKO, NAME_BYAKKO, text, 0, TALK_MS)
end

-- Byakko (`D`): brought by the script (at the start and after a wipe); pulled with his line; his fall ends the duty.
BYAKKO_BOSS = D:boss({ id = 1, layout = BYAKKO, middle = MIDDLE, helpers = { layout = HELPER, count = HELPERS },
    comes_later = true, eobjs_at_setup = false, bgm = false, last = true,
    talker = { name = NAME_BYAKKO, pull = TALK_PULL, ms = TALK_MS },
    on_pull = function(d)
        startTheClock(d)
    end,
    on_death = function(d)
        D:dropTimed()
        clearOrbs(d)
        D:after(COMPLETE_AFTER, function(d2) d2:complete_duty() end)
        D:after(CUTSCENE_AFTER, function(d2) d2:play_cutscene(CUTSCENE_END, CUTSCENE_END_PARAMS) end)
    end })

function onSetup(director)
    director:set_sequence(1)
    reset()
    bringByakko(director)
end

function reset()
    last_part = false
    low_said = false
    orbs = {}
    puddles = {}
    in_puddle = {}
    dash_at = nil
    seized = nil
    hakutei = nil
    leaper = nil
end

function bringByakko(director)
    byakko = director:spawn_bnpc(BYAKKO, BYAKKO_AT.x, BYAKKO_AT.y, BYAKKO_AT.z, BYAKKO_AT.rotation) or BYAKKO
    D:bringHelpers(director, BYAKKO_BOSS)
end

-- The clock from the pull: a row runs only while this attempt goes on.
function at(seconds, fn)
    local mine = attempt
    D:inSeconds(seconds, function(d, health, info)
        if mine == attempt and D.fight == 1 then
            fn(d, health, info)
        end
    end)
end

function startTheClock(director)
    for _, row in ipairs(FIRST_ROWS) do
        at(row[1], function(d) byakkoRow(d, row) end)
    end
    for _, row in ipairs(HAKUTEI_ROWS) do
        at(row[1], function(d) middleRow(d, row) end)
    end
    at(HAKUTEI_FROM, function(d) bringHakutei(d) end)
    local from = LAST_FROM
    for _, row in ipairs(FIRST_BLOCK) do
        at(from + row[1], function(d) byakkoRow(d, row) end)
    end
    from = from + FIRST_BLOCK_FOR
    for _ = 1, BLOCKS do
        for _, row in ipairs(BLOCK) do
            at(from + row[1], function(d) byakkoRow(d, row) end)
        end
        from = from + BLOCK_EVERY
    end
end

-- ---- Byakko's own ----

function byakkoRow(director, row)
    local kind = row[2]
    if kind == "raidwide" then
        director:actor_use_action(byakko, RAIDWIDE)
        if last_part then
            D:after(RAIDWIDE_AGAIN_AFTER, function(d) d:actor_use_action(byakko, RAIDWIDE_AGAIN) end)
        end
    elseif kind == "tank" then
        director:actor_use_action(byakko, TANK_BUSTER, theTank())
    elseif kind == "knock" then
        director:actor_use_action(byakko, KNOCK_BACK)
    elseif kind == "round" then
        director:actor_use_action(byakko, ROUND_HIM)
    elseif kind == "talk" then
        talk(director, row[3])
    elseif kind == "seize" then
        seizing(director, row[3], row[4])
    elseif kind == "orbs" then
        theOrbs(director, row[3])
    elseif kind == "cut_here" then
        local member = anyOfTheParty()
        if member ~= nil then
            director:warp_bnpc(byakko, MIDDLE.x, MIDDLE.y, MIDDLE.z, facing(MIDDLE, member))
        end
        director:actor_use_action(byakko, CUT)
    elseif kind == "cut_out" then
        director:actor_use_action(byakko, BEFORE_JUMP)
        local angle = math.random(0, 7) * math.pi / 4
        local spot = { x = EDGE * math.sin(angle), y = 0.0, z = EDGE * math.cos(angle) }
        D:after(JUMP_AFTER_SIGN, function(d)
            d:jump_bnpc(byakko, spot.x, spot.y, spot.z, facing(spot, MIDDLE))
            D:after(CAST_AFTER_JUMP, function(d2) d2:actor_use_action(byakko, CUT) end)
        end)
    elseif kind == "circles" then
        talk(director, 18604)
        director:actor_use_action(byakko, CIRCLES_SHOWN)
        theCircles(director)
    elseif kind == "marked" then
        talk(director, 18605)
        if not low_said and lowNow then
            low_said = true
            talk(director, TALK_LOW)
        end
        director:actor_use_action(byakko, MARKED_SHOWN)
        for _, member in ipairs(D.party) do
            director:actor_control(member.id, LOCKON, { MARKER_EACH, member.id })
        end
        D:after(ON_EACH_AFTER, function(d) D:helpersOnEach(d, ON_EACH, 0.0) end)
    elseif kind == "leaves" then
        byakkoLeaves(director)
    end
end

function theTank()
    local it = D.info and D.info[BYAKKO]
    if it ~= nil and it.target_id ~= nil then
        return it.target_id
    end
    local member = D.party[1]
    return member and member.id or nil
end

function anyOfTheParty(but)
    local choices = {}
    for _, member in ipairs(D.party) do
        if member.id ~= but then
            table.insert(choices, member)
        end
    end
    if #choices == 0 then
        choices = D.party
    end
    if #choices == 0 then
        return nil
    end
    return choices[math.random(#choices)]
end

-- The game's rotation from `from` toward `to` (0 is +z.)
function facing(from, to)
    return math.atan(to.x - from.x, to.z - from.z)
end

-- 10208 (a cast) or 10070 (at once) on one of the party, 10209, the dash and the throw. The first of a pair picks the
-- dash's spots (opposite, 4 yalms out, on a multiple of 45 degrees) and jumps to its start; the second dashes back.
function seizing(director, how, first)
    local target = anyOfTheParty(first and nil or seized)
    if target == nil then
        return
    end
    seized = target.id
    local who = target.id
    director:actor_use_action(byakko, how == "cast" and STUN or STUN_NOW, who)
    D:after(TALK_SEIZE_AFTER[how], function(d) talk(d, TALK_SEIZE[how]) end)
    D:after(SEIZE_AFTER[how], function(d) d:actor_use_action(byakko, SEIZE, who) end)
    local to
    if first or dash_at == nil then
        local angle = math.random(0, 7) * math.pi / 4
        to = { x = DASH_FROM_MIDDLE * math.sin(angle), y = 0.0, z = DASH_FROM_MIDDLE * math.cos(angle) }
        local start = { x = -to.x, y = 0.0, z = -to.z }
        D:after(JUMP_AFTER, function(d) d:jump_bnpc(byakko, start.x, start.y, start.z, facing(start, to)) end)
    else
        to = { x = -dash_at.x, y = 0.0, z = -dash_at.z }
    end
    dash_at = to
    local angle = math.random() * 2 * math.pi
    local landing = { x = LANDING_FROM_MIDDLE * math.sin(angle), y = 0.0, z = LANDING_FROM_MIDDLE * math.cos(angle) }
    local marked = nil
    D:after(MARK_AFTER[how], function(d)
        marked = D:helperAt(d, landing.x, landing.y, landing.z)
        if marked ~= nil then
            d:actor_control(marked, LOCKON, { MARKER_LANDING, marked })
        end
    end)
    D:after(DASH_AFTER[how], function(d)
        d:aim_action(byakko, DASH, to.x, to.y, to.z)
        d:actor_use_action(byakko, DASH)
        D:after(DASH_CAST, function(d2)
            d2:jump_bnpc(byakko, to.x, to.y, to.z, nil, nil, true)
            D:after(THROW_AFTER_DOWN, function(d3)
                d3:toss_player(who, landing.x, landing.y, landing.z)
                d3:lose_effect(who, STATUS_STUN)
                d3:lose_effect(who, STATUS_SEIZED)
                if marked ~= nil then
                    d3:actor_use_action(marked, LANDING)
                end
            end)
        end)
    end)
end

-- ---- The orbs ----

function theOrbs(director, each)
    director:actor_use_action(byakko, BEFORE_JUMP)
    D:after(JUMP_AFTER_SIGN, function(d)
        d:jump_bnpc(byakko, MIDDLE.x, MIDDLE.y, MIDDLE.z, nil, nil, true)
    end)
    D:after(JUMP_AFTER_SIGN + 0.4, function(d) d:actor_use_action(byakko, ORBS) end)
    local turn = (math.random(2) == 1 and 1 or -1) * ORB_TURN
    local first = ORB_FIRST[each]
    for set = 0, ORB_SETS - 1 do
        D:after(JUMP_AFTER_SIGN + 0.4 + ORBS_AFTER_CAST + set * ORB_EVERY, function(d)
            local shift = math.max(0, set - (ORB_SAME_FOR - 1)) * turn
            for _, degrees in ipairs(first) do
                spawnOrb(d, math.rad(degrees + shift))
            end
            if set == ORB_SETS - 1 then
                d:lose_effect(byakko, STATUS_ORBS)
            end
        end)
    end
end

function spawnOrb(director, angle)
    local orb = director:spawn_bnpc(ORB, MIDDLE.x, MIDDLE.y + 0.1, MIDDLE.z, angle)
    if orb == nil then
        return
    end
    director:hold_bnpc(orb)
    director:sweep_bnpc(orb, ORB_GOES * math.sin(angle), MIDDLE.y + 0.1, ORB_GOES * math.cos(angle), ORB_SPEED)
    table.insert(orbs, { id = orb, gone_at = D.clock + ORB_GOES / ORB_SPEED, blown = false })
end

-- Each update: an orb within reach of one of the party goes off and falls; one at the edge fades.
function watchOrbs(director, info)
    local left = {}
    for _, orb in ipairs(orbs) do
        local it = info["actor" .. orb.id]
        if not orb.blown and it ~= nil then
            for _, member in ipairs(D.party) do
                local dx, dz = member.x - it.x, member.z - it.z
                if dx * dx + dz * dz <= ORB_REACH * ORB_REACH then
                    orb.blown = true
                    director:sweep_bnpc(orb.id, it.x, it.y, it.z, ORB_SPEED)
                    director:actor_use_action(orb.id, ORB_BLAST)
                    local id = orb.id
                    D:after(ORB_FALLS_AFTER, function(d) d:kill_bnpc(id) end)
                    break
                end
            end
        end
        if not orb.blown and D.clock >= orb.gone_at then
            director:fade_out_bnpc(orb.id)
        elseif not orb.blown then
            table.insert(left, orb)
        end
    end
    orbs = left
end

function clearOrbs(director)
    for _, orb in ipairs(orbs) do
        director:fade_out_bnpc(orb.id)
    end
    orbs = {}
end

-- ---- The eight circles ----

function theCircles(director)
    local diagonal = {}
    local square = {}
    for i = 0, 3 do
        local a = (i * 2 + 1) * math.pi / 4
        table.insert(diagonal, a)
        table.insert(square, i * math.pi / 2)
    end
    local sets = math.random(2) == 1 and { diagonal, square } or { square, diagonal }
    for index, set in ipairs(sets) do
        D:after((index - 1) * CIRCLE_SECOND_AFTER, function(d)
            for _, angle in ipairs(set) do
                circle(d, angle)
            end
        end)
    end
end

function circle(director, angle)
    local spot = function(step)
        local r = CIRCLE_STEP_YALMS * (step + 1)
        return r * math.sin(angle), r * math.cos(angle)
    end
    local x, z = spot(0)
    local helper = D:helperAt(director, x, 0.0, z, angle)
    if helper == nil then
        return
    end
    director:actor_use_action(helper, CIRCLE)
    for step = 1, CIRCLE_STEPS do
        D:after(CIRCLE_STEPS_FROM + (step - 1) * CIRCLE_STEP_EVERY, function(d)
            local sx, sz = spot(step)
            d:warp_bnpc(helper, sx, 0.0, sz, angle)
            d:actor_use_action(helper, CIRCLE_STEP)
        end)
    end
end

-- ---- Byakko leaves; Hakutei; the intermission ----

function byakkoLeaves(director)
    director:actor_use_action(byakko, BEFORE_JUMP)
    D:after(JUMP_AFTER_SIGN, function(d) d:jump_bnpc(byakko, 0.0, 0.0, -16.0, nil, nil, true) end)
end

function bringHakutei(director)
    director:actor_use_action(byakko, LEAVES)
    director:set_targetable(byakko, false)
    director:hold_bnpc(byakko)
    hakutei = director:spawn_bnpc(HAKUTEI, MIDDLE.x, MIDDLE.y, MIDDLE.z, 0.0)
    if hakutei ~= nil then
        director:actor_use_action(hakutei, HAKUTEI_COMES)
        director:join_fight(hakutei, byakko)
    end
    at(114.8, function(d) d:warp_bnpc(byakko, BELOW.x, BELOW.y, BELOW.z, 0.0) end)
end

function middleRow(director, row)
    local kind = row[2]
    if kind == "talk" then
        talk(director, row[3], row[4])
    elseif kind == "timeline" then
        director:play_action_timeline(row[3], row[4])
    elseif kind == "bgm" then
        director:set_bgm(row[3])
    elseif kind == "weather" then
        director:set_weather(row[3], WEATHER_FADE, true)
    elseif kind == "spots" then
        -- At each of the party, a puddle where it lands that hurts (273) whoever stands in it.
        for _, member in ipairs(D.party) do
            local spot = { x = member.x, y = 0.0, z = member.z }
            local helper = D:helperAt(director, spot.x, spot.y, spot.z)
            if helper ~= nil then
                director:aim_action(helper, SPOTS, spot.x, spot.y, spot.z)
                director:actor_use_action(helper, SPOTS)
            end
            D:after(PUDDLE_AFTER, function(d)
                d:spawn_eobj(PUDDLE, spot)
                table.insert(puddles, { x = spot.x, z = spot.z, until_clock = D.clock + PUDDLE_FOR })
            end)
        end
        D:after(PUDDLE_AFTER + PUDDLE_FOR, function(d) d:delete_eobj(PUDDLE) end)
    elseif hakutei ~= nil and kind == "claw" then
        director:actor_use_action(hakutei, CLAW, hakuteiTarget())
    elseif hakutei ~= nil and kind == "herald" then
        local member = anyOfTheParty()
        if member ~= nil then
            director:actor_control(member.id, LOCKON, { MARKER_HERALD, member.id })
            local who = member.id
            D:after(5.1, function(d)
                if hakutei ~= nil then
                    d:actor_use_action(hakutei, HERALD, who)
                end
            end)
        end
    elseif hakutei ~= nil and kind == "line" then
        director:actor_use_action(hakutei, LINE)
    elseif hakutei ~= nil and kind == "visual" then
        director:actor_use_action(hakutei, row[3])
    elseif hakutei ~= nil and kind == "roar" then
        director:actor_use_action(hakutei, ROAR)
    elseif kind == "hakutei_goes" then
        -- Out of the fight and out of sight (retail's stayed: ActionTimeline 4576 as Byakko came back.)
        if hakutei ~= nil then
            director:set_targetable(hakutei, false)
            director:hold_bnpc(hakutei)
        end
    elseif kind == "sky" then
        for _, member in ipairs(D.party) do
            director:gain_effect(member.id, STATUS_SKY, 0, 0.0)
        end
    elseif kind == "below" then
        director:actor_use_action(byakko, row[3])
    elseif kind == "marks" then
        for _, member in ipairs(D.party) do
            local spot = { x = member.x, z = member.z }
            local helper = D:helperAt(director, spot.x, 0.0, spot.z)
            if helper ~= nil then
                director:aim_action(helper, MARK, spot.x, 0.0, spot.z)
                director:actor_use_action(helper, MARK)
                D:after(MARK_DROPS_AFTER, function(d)
                    d:aim_action(helper, DROP, spot.x, 0.0, spot.z)
                    d:actor_use_action(helper, DROP)
                end)
            end
        end
    elseif kind == "from_middle" then
        local helper = D:helperAt(director, MIDDLE.x, MIDDLE.y, MIDDLE.z)
        if helper ~= nil then
            director:actor_use_action(helper, FROM_MIDDLE)
        end
    elseif kind == "leaper" then
        local spot = { x = row[3], y = 0.0, z = row[4] }
        if leaper ~= nil then
            director:fade_out_bnpc(leaper)
        end
        leaper = director:spawn_bnpc(LEAPER, spot.x, spot.y, spot.z, facing(spot, MIDDLE))
        if leaper ~= nil then
            director:hold_bnpc(leaper)
            local it = leaper
            D:after(LEAPER_CASTS_AFTER, function(d) d:actor_use_action(it, LEAPER_DROP) end)
        end
    elseif kind == "sky_ends" then
        for _, member in ipairs(D.party) do
            director:gain_effect(member.id, STATUS_SKY_ENDS, STATUS_SKY_ENDS_PARAM, 5.1)
        end
    elseif kind == "byakko_up" then
        director:warp_bnpc(byakko, MIDDLE.x, MIDDLE.y, MIDDLE.z, 0.0)
        if leaper ~= nil then
            director:fade_out_bnpc(leaper)
            leaper = nil
        end
    elseif kind == "back" then
        director:set_weather(WEATHER_BACK, WEATHER_FADE, true)
        director:actor_use_action(byakko, BACK)
        for _, member in ipairs(D.party) do
            director:lose_effect(member.id, STATUS_SKY)
            director:gain_effect(member.id, STATUS_BACK, STATUS_BACK_PARAM, STATUS_BACK_FOR)
        end
    elseif kind == "targetable" then
        director:set_targetable(byakko, true)
    elseif kind == "at_the_end" then
        local helper = D:helperAt(director, MIDDLE.x, MIDDLE.y, MIDDLE.z)
        if helper ~= nil then
            director:actor_use_action(helper, AT_THE_END)
        end
    elseif kind == "fights" then
        last_part = true
        director:release_bnpc(byakko)
    elseif kind == "tank" then
        director:actor_use_action(byakko, TANK_BUSTER, theTank())
    end
end

function hakuteiTarget()
    local it = D.info and D.info[HAKUTEI]
    if it ~= nil and it.target_id ~= nil then
        return it.target_id
    end
    return theTank()
end

-- Who stands in a puddle has 273 while they do.
function watchPuddles(director)
    local live = {}
    for _, puddle in ipairs(puddles) do
        if D.clock < puddle.until_clock then
            table.insert(live, puddle)
        end
    end
    puddles = live
    for _, member in ipairs(D.party) do
        local inside = false
        for _, puddle in ipairs(puddles) do
            local dx, dz = member.x - puddle.x, member.z - puddle.z
            if dx * dx + dz * dz <= PUDDLE_RADIUS * PUDDLE_RADIUS then
                inside = true
            end
        end
        if inside and not in_puddle[member.id] then
            in_puddle[member.id] = true
            director:gain_effect(member.id, STATUS_PUDDLE, 0, 0.0)
        elseif not inside and in_puddle[member.id] then
            in_puddle[member.id] = nil
            director:lose_effect(member.id, STATUS_PUDDLE)
        end
    end
end

-- ---- The server's calls ----

function onActorCast(director, layout_id, action_id, actor_id, x, y, z, target_id)
    D:cast(director, layout_id, action_id, actor_id, x, y, z, target_id)
end

function onUpdate(director, health, player_health, info)
    D:update(director, health, player_health, info, function(d)
        if D.fight == 1 and pulled_clock == nil then
            pulled_clock = D.clock
        end
        lowNow = health[BYAKKO] ~= nil and health[BYAKKO] <= LOW_AT
        watchOrbs(d, info)
        watchPuddles(d)
    end)
end

function onActorDeath(director, bnpc_id, position)
    D:died(director, bnpc_id, position)
end

-- Everyone down: everything goes, the weather and the party's statuses as at the start, and Byakko comes again.
function onPartyWipe(director)
    clearOrbs(director)
    for _, member in ipairs(D.party) do
        for _, status in ipairs({ STATUS_STUN, STATUS_SEIZED, STATUS_PUDDLE, STATUS_SKY, STATUS_SKY_ENDS }) do
            director:lose_effect(member.id, status)
        end
    end
    director:delete_eobj(PUDDLE)
    if hakutei ~= nil then
        director:despawn_bnpc(HAKUTEI)
    end
    if leaper ~= nil then
        director:despawn_bnpc(LEAPER)
    end
    director:despawn_bnpc(BYAKKO)
    for _, helper in ipairs(D.helpers_of[BYAKKO_BOSS.id] or {}) do
        director:despawn_bnpc(helper)
    end
    D:wipe(director)
    director:set_weather(WEATHER_START, WEATHER_FADE, true)
    attempt = attempt + 1
    pulled_clock = nil
    reset()
    bringByakko(director)
end

-- No shortcuts in a trial.
function getDebugShortcut(id)
    return 0
end
