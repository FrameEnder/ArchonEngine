-- The Praetorium (ContentFinderCondition 16, territory 1044, director 0x80030056), during The Ultimate Weapon (quest
-- 4522).
--
-- Five retail runs with Duty Support (the fourth set, level 50): Connor's Chronofoil capture af05916c (2026-11-09,
-- visit 0.10, a healer), Bob's 9ba7d981 (2026-11-17, visits 0.14, left at the last boss, and 0.16, a damage
-- dealer), 50e72631 (the same day, visit 0.5, a healer) and 6320ae56 (2026-11-18, visit 0.3, a Paladin, with Kawari
-- Witness); `questkit chronofoil --battle` and the visits' packets (docs/src/fork/dungeons.md). Numbers only. **No
-- layout dump**: the duty's event ranges aren't known, so what the party sets off by walking somewhere is set off
-- here by where they were when it happened in the runs (the WHERE_* lines below).
--
-- The way through:
--   * talk 32920 ten yalms in. The four at the entrance (9081944-9081947) dead: talk 32921, the lift
--     (EObj 2012811) works, five more below (9081963-9081967);
--   * the lift (event action 23, a second): everyone 30 yalms down; map effect 1 4.6 s on. The five dead: map effect 2
--     and ten more (9081969-9081978); map effect 4 further in; the ten dead: map effect 3. The second lift (EObj
--     2012813) to the first boss's floor;
--   * near its room: objective 0 and cutscene 280; the first boss (9054638) is there 2.2 s in. It dead: music 149,
--     objective 1, talk 32916, the sixteen of the way ahead (9082001-9082005, 9082022-9082032), and two objects
--     work: EObj 2012815 (everyone up to the hangar; talks 32917 and 32918 5.7 and 12.7 s on) and the armour (EObj
--     2012819: Mount 17, whose action 1128 does 4500 to all it hits);
--   * the ride: the last five of the sixteen dead (9082028-9082032): map effect 5, and the way's end takes
--     everyone below (still mounted) to seven more (9168161-9168164,
--     9168175-9168177); four of BNpcBase 14566 one after another (9082092-9082095: map effects 8 and 12, 9 and 13, 10
--     and 11, then 14 and 18) and BNpcBase 14616 (9089992); it dead: map effects 6 and 7 and cutscene 392, the mount
--     gone;
--   * further in: objective 2 and cutscene 283; the second boss (9054847) is there 2.3 s in. It dead: objective 3, map
--     effects 0 and 15, cutscene 284, and EObj 2012817 works: objective 4, map effect 16, cutscene 118, and as it ends
--     everyone before the last boss (9055066). It dead: the duty complete, cutscene 374, everyone out.
--
-- The fights (the bosses' own casts are their timelines, PraetoriumBoss1_14450, PraetoriumBoss2_14451 and
-- PraetoriumBoss3_14453; what goes with them is here):
--   * the first: 28467 takes it to the middle; with 28468 four helpers' 28469 6 yalms round the middle and four 28470
--     12 out (a cross the first time; the second time a cross or turned, at random); 5.9 s into 28471 a helper's
--     28472 on each of the party;
--   * the second: talks 32900 (4.1 s after the pull, with map effect 0), 32902 and 32901 9.2 and 29.6 s after it,
--     32903 below 20 %;
--     0.9 s after each 28479 lands one of BNpcBase 14452 comes (north, then south);
--   * the last: talks 32904 (the pull), 32906, 32905 and 32907 at 39.5, 62.1 and 80.5 s. 28496 takes it to the
--     middle; with 28486 a helper's 29779 and, 3.1 s on, three of BNpcBase 14456's 28487 across the middle the way it
--     faces; 3.1 s after each 28484 five of BNpcBase 14454 at the west edge (one of two sets of places), and five
--     14456s' 28488 from there 4.9 s later; with 29051 four rows of three helpers' 29052 2 s apart (north to south
--     or south to north); 3.9 s into 28491 a helper's 28492 on each of the party. **The second part** from 141 s
--     (139.7 to 142.9 in four runs, its health at 11 to 15 %): talk 32919; 5 s on it can't be targeted and four of
--     BNpcBase 14455 fight in its place (28498 on each of the party at 11.8, 34.9 and 63.0 s; 29051 at 27.8 with
--     the rows; 28491 at 49.9 with the 28492s), the 14454s and 28488s again at 16.8, 48.9 and 68.0 s, and a gauge
--     (the director's 0x8000000C [58, n], n up 111 a second from 7.5 s). At 77.9 s talk 32908, 28499 on everyone 2.1
--     s later, talk 32909 at 90.1 and it fights on (28495 7.1 s later; it fell within 12 s of it in the runs).
--
-- Not known, so guessed or left out: the event ranges (above); what ends the second part (77.9 s in three runs, 67.9
-- in one with its gauge stopped at 65.5 s: 77.9 here, the gauge to 67.6 s as in Bob's last run) and what the gauge
-- and 28499's amount (605 to 1650) have to do with the 14455s' deaths; where 28475 takes the second boss (it stays);
-- the bosses' clocks past what the runs show (the rounds go on); a fall at a boss (it wasn't seen: the party is
-- sent back to where the last lift or cutscene put it, `set_start`, and the fight begins again); whether the 14455s
-- attack between their casts (they do here); the members on the ride (they walk); ActorControls 20, 62, 63 and 70 on
-- the last boss (sent as retail did); the ten of BNpcBase 6358 that showed for a moment along the way (left out);
-- BNpcBase 14567 by the entrance (it stands there); coffers (none in a Duty Support run).

GUIDE = 9168666
NAME_GUIDE = 1486
TALK_MS = 6000

EVENT_ACTION_OBJECT = 23
EOBJ_LIFT1 = 2012811
EOBJ_LIFT2 = 2012813
EOBJ_HANGAR = 2012815
EOBJ_ARMOUR = 2012819
EOBJ_LAST = 2012817
EOBJ_AFTER_LAST = 2012818
EOBJ_EXIT = 2000848
-- Where retail put the player (its ActorSetPos, capture 6320ae56.)
LIFT1_TO = { x = 200.10, y = 156.50, z = 13.63, rotation = 0.522 }
LIFT2_TO = { x = 240.75, y = 71.50, z = 0.35, rotation = -1.571 }
HANGAR_TO = { x = 146.86, y = 102.00, z = -37.53, rotation = 2.654 }
BELOW_TO = { x = 108.73, y = -106.01, z = 0.45, rotation = -1.529 }
BOSS3_TO = { x = -550.22, y = -268.00, z = 220.35, rotation = -1.571 }
OUT_TO = { x = -764.70, y = -344.00, z = 0.45, rotation = -1.554 }
MOUNT_ARMOUR = 17

FOUR = { 9081944, 9081945, 9081946, 9081947 }
FIVE = { 9081963, 9081964, 9081965, 9081966, 9081967 }
TEN = { 9081969, 9081970, 9081971, 9081972, 9081973, 9081974, 9081975, 9081976, 9081977, 9081978 }
AHEAD = { 9082001, 9082002, 9082003, 9082004, 9082005 }
RIDE = { 9082022, 9082023, 9082024, 9082025, 9082026, 9082027, 9082028, 9082029, 9082030, 9082031, 9082032 }
LAST_FIVE = { 9082028, 9082029, 9082030, 9082031, 9082032 }
BELOW = { 9168161, 9168162, 9168163, 9168164, 9168175, 9168176, 9168177 }
BELOW_THREE = { 9168162, 9168163, 9168164 }
WALKERS = { 9082092, 9082093, 9082094, 9082095 }
WALKER_LAST = 9089992

-- Where the party was as retail set these off (no layout dump: see above.)
WHERE_START_X = 159.0 -- east of this at the entrance (ten yalms in): talk 32920 (2.1 to 54 s after the start)
WHERE_EFFECT_4 = { z = -51.5, x = 225.0 } -- north of z and east of x on the lift's floor: map effect 4
WHERE_BOSS1_X = 208.0 -- west of this on the second lift's floor: objective 0 and cutscene 280
-- The way's end, once map effect 5 is on: east of x on the ramp down between the two zs: everyone below
WHERE_BELOW = { x = 225.0, z_south = 76.0, z_north = 55.0 }
WHERE_EFFECT_8_Z = -29.5 -- a player north of this below: map effect 8 and the first of BNpcBase 14566
WHERE_EFFECT_9_X = 54.5 -- a player west of this with the three of 9168162-9168164 dead: map effect 9 and the second
WHERE_EFFECT_10_X = -16.5 -- and of this with all seven dead: map effect 10 and the third
WHERE_BOSS2_X = -130.0 -- objective 2 and cutscene 283

BOSS1 = 9054638
BOSS1_WALL = 2000828
BOSS1_LINE = 2000829
BOSS1_BEHIND = 2000789 -- shown a second after its pull
BOSS1_AHEAD = 2000876 -- gone as it falls
PLACE_BOSS1 = 451
BOSS1_MIDDLE = { x = 192.0, y = 76.0, z = 0.2 }
HELPER_BOSS1 = 1
HELPERS_BOSS1 = 8
BGM_AFTER_BOSS1 = 149
CUTSCENE_BOSS1 = 280
CUTSCENE_BOSS1_PARAMS = { 1, 37, 0, 0 }
BOSS1_AFTER_CUTSCENE = 2.2
-- Its 28467s (seconds from the pull), as its timeline has them: twice, then two a round of 52.7 s.
AWAY_FOR = 1.0
BOSS1_AWAY_ONCE = { 18.3, 38.6 }
BOSS1_AWAY_ROUND = { 71.0, 90.4 }
BOSS1_AWAY_EVERY = 52.7
ROUNDS = 10
-- (x, z) from the middle.
SPOTS_28469 = {
    { { -6.0, 0.0 }, { 0.0, -6.0 }, { 0.0, 6.0 }, { 6.0, 0.0 } },
    { { -5.2, -3.0 }, { -3.0, 5.2 }, { 3.0, -5.2 }, { 5.2, 3.0 } },
}
SPOTS_28470 = {
    { { -12.0, 0.0 }, { 0.0, -12.0 }, { 0.0, 12.0 }, { 12.0, 0.0 } },
    { { -10.4, -6.0 }, { -6.0, 10.3 }, { 6.0, -10.4 }, { 10.4, 6.0 } },
}
AFTER_28471 = 5.9

BOSS2 = 9054847
BOSS2_WALL = 2000822
BOSS2_LINE = 2000823
BOSS2_AHEAD = 2001135
PLACE_BOSS2 = 454
BOSS2_MIDDLE = { x = -164.0, y = -104.2, z = 0.0 }
NAME_BOSS2 = 2135
HELPER_BOSS2 = 2
HELPERS_BOSS2 = 8
CUTSCENE_RIDE = 392
CUTSCENE_RIDE_PARAMS = { 1, 0, 0, 0 }
CUTSCENE_BOSS2 = 283
CUTSCENE_BOSS2_PARAMS = { 1, 151, 0, 0 }
BOSS2_AFTER_CUTSCENE = 2.3
CUTSCENE_AFTER_BOSS2 = 284
CUTSCENE_AFTER_BOSS2_PARAMS = { 1, 149, 0, 0 }
-- Its talks, seconds from its pull (4.1 s in all five runs; then 9.2 and 29.6 s after the first.)
BOSS2_TALKS = { { 4.1, 32900 }, { 13.3, 32902 }, { 33.7, 32901 } }
BOSS2_TALK_BELOW = 20
BOSS2_ADD = 9054848
BOSS2_ADD_AFTER = 3.6 -- its 28479's 2.7 s and 0.9 more
BOSS2_ADD_AT = { { -164.0, -104.2, -20.0 }, { -164.0, -104.2, 20.0 } }

BOSS3 = 9055066
BOSS3_MIDDLE = { x = -562.0, y = -268.0, z = 220.0 }
NAME_BOSS3 = 2136
HELPER_BOSS3 = 3
HELPERS_BOSS3 = 16
CUTSCENE_BOSS3 = 118
CUTSCENE_BOSS3_PARAMS = { 0, 152, 1, 0 }
CUTSCENE_END = 374
CUTSCENE_END_PARAMS = { 0, 18, 0, 0 }
BOSS3_TALKS = { { 39.5, 32906 }, { 62.1, 32905 }, { 80.5, 32907 } }
BOSS3_AWAY = { 15.3, 60.9, 79.3, 109.2 }
BOSS3_SLASH_AT = 17.2 -- its 28486, from the timeline
SLASH_AFTER = 3.1
SLASH_Z = { -4.0, 0.0, 4.0 }
SWORD = 9055067 -- BNpcBase 14454, five
SWORDS = 5
PHANTOM = 9055068 -- BNpcBase 14455, four
BLADE = 9055069 -- BNpcBase 14456: each comes for one cast, 1.9 s before it, and goes 6 s after it began
BLADE_BEFORE = 1.9
BLADE_GONE_AFTER = 6.0
WAITING = { x = -575.0, z_phantoms = 228.0, z_swords = 232.0 } -- where retail kept them
SWORDS_AFTER_28484 = 3.1
SWORDS_X = -576.6
SWORDS_Z = { { 206.0, 214.0, 222.0, 230.0, 238.0 }, { 202.0, 210.0, 218.0, 226.0, 234.0 } }
SWORDS_FACING = 1.571
-- From the 14454s' going to their places: their 28486, ActionTimeline 7737, and the 14456s' 28488.
SWORDS_SHOW = 1.8
SWORDS_TIMELINE = 3.8
SWORDS_CAST = 4.9
SWORDS_CAST_PART2 = 4.2
ROWS_AFTER_29051 = 3.9
ROWS_APART = 2.0
ROWS = {
    { { -573.0, 203.0 }, { -562.0, 237.0 }, { -551.0, 203.0 } },
    { { -573.0, 214.5 }, { -562.0, 225.5 }, { -551.0, 214.5 } },
    { { -573.0, 225.5 }, { -562.0, 214.5 }, { -551.0, 225.5 } },
    { { -573.0, 237.0 }, { -562.0, 203.0 }, { -551.0, 237.0 } },
}
AFTER_28491 = 3.9
-- The second part, seconds from its talk.
-- 139.7 s twice and 142.2 and 142.8 s (7.0 or 10.0 s after its 28495 began; its health 11 to 15 %): between them.
PART2_AT = 141.2
PART_SECOND = 32919
PART_AFTER = 32909
PHANTOMS_AT = { { -554.5, 210.0 }, { -554.5, 230.0 }, { -569.5, 210.0 }, { -569.5, 230.0 } }
PART2_PHANTOMS = 5.0
PART2_ON_EACH = { 11.8, 34.9, 63.0 }
PART2_SWORDS = { 16.8, 48.9, 68.0 }
PART2_ROWS = 27.8
PART2_SPREAD = 49.9
PART2_ENDS = { 67.9, 77.9, 87.9 }
-- From its end (talk 32908): 28499 on everyone, the gauge gone, and the boss back (talk 32909.)
PART2_ALL = 2.1
PART2_GAUGE_GONE = 9.1
PART2_BACK = 12.2
PHANTOM_LOCK = 2.2
COMMAND_GAUGE = 0x8000000C
GAUGE = 58
GAUGE_FROM = 7.5
GAUGE_SECONDS = 80
GAUGE_FULL_SECONDS = 9.0 -- n is 1000 nine seconds in: 111 a second

ticks = 0
updating = false
later = {}
dead = {}
done = {}
helpers_of = { {}, {}, {} }
helpers = {}
next_helper = 0
party = {}
fight = nil -- 1, 2 or 3 while that boss is fought
crosses = 0
adds = 0
part2 = false
part2_over = false
part2_ending = false
floored = {}
phantoms_free_at = 0
last_boss_due = false
-- The cutscene under way: `onSceneFinished` says only that one ended (and says so once for each player: the first
-- to end it counts.)
playing = nil
swords = {}
blades = {}
phantoms = {}
gone = {}

-- Runs fn so many seconds from now (an update is half a second; a call made outside onUpdate waits for the next.)
function after(seconds, fn)
    local updates = math.max(1, math.floor(seconds * 2 + 0.5))
    if not updating then
        updates = updates + 1
    end
    table.insert(later, { at = ticks + updates, fn = fn })
end

function runLater(director)
    local due, waiting = {}, {}
    for _, entry in ipairs(later) do
        if entry.at <= ticks then
            table.insert(due, entry)
        else
            table.insert(waiting, entry)
        end
    end
    later = waiting
    table.sort(due, function(a, b) return a.at < b.at end)
    for _, entry in ipairs(due) do
        entry.fn(director)
    end
end

function once(name)
    if done[name] then
        return false
    end
    done[name] = true
    return true
end

function talk(director, text)
    director:battle_talk(GUIDE, NAME_GUIDE, text, 0, TALK_MS)
end

function cutscene(director, id, params)
    playing = id
    director:play_cutscene(id, params)
end

function onSetup(director)
    director:set_sequence(1)
    for _, object in ipairs({ EOBJ_LIFT1, EOBJ_HANGAR, EOBJ_ARMOUR, EOBJ_LAST, EOBJ_AFTER_LAST, EOBJ_EXIT }) do
        director:hide_eobj(object)
    end
end

function onCommence(director)
end

function bringHelpers(director, which, layout, count, at)
    helpers_of[which] = {}
    for i = 1, count do
        -- (The simulator's spawns return nothing: the layout ID stands in there.)
        helpers_of[which][i] = director:spawn_bnpc(layout, at.x, at.y, at.z, 0.0) or layout
        director:hold_bnpc(helpers_of[which][i])
    end
end

function useHelpers(which)
    helpers = helpers_of[which]
    next_helper = 0
end

function dropHelpers(director, which)
    for _, helper in ipairs(helpers_of[which]) do
        director:despawn_bnpc(helper)
    end
    helpers_of[which] = {}
    helpers = {}
end

-- The next helper in turn, put at a spot.
function helperAt(director, x, y, z, facing)
    if #helpers == 0 then
        return nil
    end
    next_helper = next_helper % #helpers + 1
    local helper = helpers[next_helper]
    director:warp_bnpc(helper, x, y, z, facing or 0.0)
    return helper
end

function helpersCast(director, action, spots, y)
    for _, spot in ipairs(spots) do
        local helper = helperAt(director, spot[1], y, spot[2], spot[3])
        if helper ~= nil then
            director:actor_use_action(helper, action)
        end
    end
end

-- A helper's cast on each of the party, from where they stand.
function helpersOnEach(director, action, y)
    for _, member in ipairs(party) do
        local helper = helperAt(director, member.x, y, member.z)
        if helper ~= nil then
            director:actor_use_action(helper, action, member.id)
        end
    end
end

function anyone(test)
    for _, member in ipairs(party) do
        if test(member) then
            return true
        end
    end
    return false
end

function within(member, at, yalms)
    local dx, dz = member.x - at.x, member.z - at.z
    return dx * dx + dz * dz <= yalms * yalms
end

function allDead(layouts)
    for _, layout in ipairs(layouts) do
        if not dead[layout] then
            return false
        end
    end
    return true
end

function comeOut(director, layouts)
    for _, layout in ipairs(layouts) do
        director:spawn_bnpc(layout)
    end
end

-- Whoever of the party has this actor ID (the one at an object), or the first of it.
function whoIs(actor_id)
    for _, member in ipairs(party) do
        if member.id == actor_id then
            return member
        end
    end
    return party[1]
end

function sendTo(director, to)
    director:warp_players(to.x, to.y, to.z, to.rotation)
    director:set_start(to.x, to.y, to.z, to.rotation)
end

-- What the party's steps set off.
function theWay(director)
    if not done.start and anyone(function(m) return m.y > 175 and m.x > WHERE_START_X end) then
        once("start")
        talk(director, 32920)
    end
    if done.five_dead and not done.effect_4
        and anyone(function(m) return m.y > 140 and m.y < 170 and m.z < WHERE_EFFECT_4.z and m.x > WHERE_EFFECT_4.x end) then
        once("effect_4")
        director:map_effect(4, 2, 1)
    end
    if done.lift2 and not done.boss1_room
        and anyone(function(m) return m.y > 60 and m.y < 90 and m.x < WHERE_BOSS1_X end) then
        once("boss1_room")
        director:objective(0, 1)
        cutscene(director, CUTSCENE_BOSS1, CUTSCENE_BOSS1_PARAMS)
        after(BOSS1_AFTER_CUTSCENE, function(d)
            d:spawn_boss(BOSS1, BOSS1_WALL, BOSS1_LINE, PLACE_BOSS1)
            bringHelpers(d, 1, HELPER_BOSS1, HELPERS_BOSS1, BOSS1_MIDDLE)
        end)
    end
    if done.effect_5 and not done.below
        and anyone(function(m)
            return m.y > 50 and m.y < 72 and m.x > WHERE_BELOW.x and m.z < WHERE_BELOW.z_south and m.z > WHERE_BELOW.z_north
        end) then
        once("below")
        sendTo(director, BELOW_TO)
        -- (There as the party is: 1.5 s after the fade begins.)
        after(1.5, function(d) comeOut(d, BELOW) end)
    end
    if done.below and not done.ride_over then
        -- A player, not a member: Connor's party's tank was past each of these before he set it off.
        local function west_of(x)
            return anyone(function(m) return m.player and m.y < -90 and m.x < x end)
        end
        if not done.effect_8
            and anyone(function(m) return m.player and m.y < -90 and m.x < 95 and m.z < WHERE_EFFECT_8_Z end) then
            once("effect_8")
            director:map_effect(8, 2, 1)
            director:spawn_bnpc(WALKERS[1])
        end
        if not done.effect_9 and allDead(BELOW_THREE) and west_of(WHERE_EFFECT_9_X) then
            once("effect_9")
            director:map_effect(9, 2, 1)
            director:spawn_bnpc(WALKERS[2])
        end
        if done.effect_9 and not done.effect_10 and allDead(BELOW) and west_of(WHERE_EFFECT_10_X) then
            once("effect_10")
            director:map_effect(10, 2, 1)
            director:spawn_bnpc(WALKERS[3])
        end
    end
    if done.ride_over and not done.boss2_room and anyone(function(m) return m.y < -90 and m.x < WHERE_BOSS2_X end) then
        once("boss2_room")
        director:objective(2, 3)
        cutscene(director, CUTSCENE_BOSS2, CUTSCENE_BOSS2_PARAMS)
        after(BOSS2_AFTER_CUTSCENE, function(d)
            d:spawn_boss(BOSS2, BOSS2_WALL, BOSS2_LINE, PLACE_BOSS2)
            bringHelpers(d, 2, HELPER_BOSS2, HELPERS_BOSS2, BOSS2_MIDDLE)
        end)
    end
end

function pull1(director)
    fight = 1
    crosses = 0
    after(1.0, function(d)
        if fight == 1 then
            d:show_eobj(BOSS1_BEHIND)
        end
    end)
    local function away(at)
        after(at + AWAY_FOR, function(d)
            if fight == 1 then
                d:jump_bnpc(BOSS1, BOSS1_MIDDLE.x, BOSS1_MIDDLE.y, BOSS1_MIDDLE.z, 1.571)
            end
        end)
    end
    for _, at in ipairs(BOSS1_AWAY_ONCE) do
        away(at)
    end
    for round = 0, ROUNDS - 1 do
        for _, at in ipairs(BOSS1_AWAY_ROUND) do
            away(at + round * BOSS1_AWAY_EVERY)
        end
    end
    useHelpers(1)
end

function pull2(director)
    fight = 2
    adds = 0
    for index, row in ipairs(BOSS2_TALKS) do
        after(row[1], function(d)
            if fight == 2 then
                d:battle_talk(BOSS2, NAME_BOSS2, row[2], 0, TALK_MS)
                if index == 1 then
                    d:map_effect(0, 2, 1)
                end
            end
        end)
    end
    useHelpers(2)
end

-- The way the last boss faces as it stands in the middle: at the first of the party.
function facingFromMiddle()
    local member = party[1]
    if member == nil then
        return 1.571
    end
    return math.atan(member.x - BOSS3_MIDDLE.x, member.z - BOSS3_MIDDLE.z)
end

function toTheMiddle(director)
    if fight == 3 and not part2 then
        director:jump_bnpc(BOSS3, BOSS3_MIDDLE.x, BOSS3_MIDDLE.y, BOSS3_MIDDLE.z, facingFromMiddle())
    end
end

-- With the last boss's 28486: a helper's 29779 in the middle and, 3.1 s on, three 14456s' 28487 across it.
function slash(director)
    local facing = facingFromMiddle()
    helpersCast(director, 29779, { { BOSS3_MIDDLE.x, BOSS3_MIDDLE.z, facing } }, BOSS3_MIDDLE.y)
    local spots = {}
    for _, z in ipairs(SLASH_Z) do
        table.insert(spots, { BOSS3_MIDDLE.x, BOSS3_MIDDLE.z + z, facing })
    end
    bladesCast(SLASH_AFTER, 28487, spots)
end

-- BNpcBase 14456s come to the spots, cast so many seconds from now, and go.
function bladesCast(cast_after, action, spots)
    after(math.max(0.0, cast_after - BLADE_BEFORE), function(d)
        if fight ~= 3 then
            return
        end
        local mine = {}
        for index, spot in ipairs(spots) do
            mine[index] = d:spawn_bnpc(BLADE, spot[1], BOSS3_MIDDLE.y, spot[2], spot[3]) or BLADE
            d:hold_bnpc(mine[index])
            table.insert(blades, mine[index])
        end
        after(BLADE_BEFORE, function(d2)
            if fight == 3 then
                for _, blade in ipairs(mine) do
                    d2:actor_use_action(blade, action)
                end
            end
        end)
        after(BLADE_BEFORE + BLADE_GONE_AFTER, function(d2)
            for _, blade in ipairs(mine) do
                d2:despawn_bnpc(blade)
            end
        end)
    end)
end

-- The five 14454s go to the west edge and show themselves; the 14456s' 28488 from their places follow.
function swordsCome(director, cast_after)
    local places = SWORDS_Z[math.random(#SWORDS_Z)]
    for index, sword in ipairs(swords) do
        director:warp_bnpc(sword, SWORDS_X, BOSS3_MIDDLE.y, places[index], SWORDS_FACING)
        director:actor_use_action(sword, 28485)
    end
    after(SWORDS_SHOW, function(d)
        if fight == 3 then
            for _, sword in ipairs(swords) do
                d:actor_use_action(sword, 28486)
            end
        end
    end)
    after(SWORDS_TIMELINE, function(d)
        if fight == 3 then
            for _, sword in ipairs(swords) do
                d:play_action_timeline(sword, 7737)
            end
        end
    end)
    local spots = {}
    for index = 1, SWORDS do
        table.insert(spots, { SWORDS_X, places[index], SWORDS_FACING })
    end
    bladesCast(cast_after, 28488, spots)
end

-- Four rows of three helpers' 29052, 2 s apart: north to south, or south to north.
function rows(director, first_after, reversed)
    for index = 1, #ROWS do
        local row = ROWS[reversed and (#ROWS + 1 - index) or index]
        after(first_after + (index - 1) * ROWS_APART, function(d)
            if fight == 3 then
                helpersCast(d, 29052, row, BOSS3_MIDDLE.y)
            end
        end)
    end
end

-- The last boss's second part: it leaves, four of BNpcBase 14455 fight in its place, and it comes back.
function secondPart(director)
    if fight ~= 3 or part2 or part2_over then
        return
    end
    part2 = true
    floored = {}
    director:restart_timeline(BOSS3, PART_SECOND)
    director:battle_talk(BOSS3, NAME_BOSS3, PART_SECOND, 0, 3000)
    local function at(seconds, fn)
        after(seconds, function(d)
            if fight == 3 and part2 and not part2_ending then
                fn(d)
            end
        end)
    end
    at(PART2_PHANTOMS, function(d)
        d:set_targetable(BOSS3, false)
        d:hold_bnpc(BOSS3)
        d:actor_use_action(BOSS3, 28496)
        for index, phantom in ipairs(phantoms) do
            local spot = PHANTOMS_AT[index]
            d:warp_bnpc(phantom, spot[1], BOSS3_MIDDLE.y, spot[2], 0.0)
            d:actor_use_action(phantom, 28485)
            d:set_targetable(phantom, true)
            d:release_bnpc(phantom)
        end
    end)
    at(PART2_PHANTOMS + 1.6, function(d)
        d:actor_use_action(BOSS3, 28497)
        d:bnpc_control(BOSS3, 20, { 2552, 388 })
        d:bnpc_control(BOSS3, 62, { 0, 1 })
        d:bnpc_control(BOSS3, 63, { 27 })
    end)
    -- The gauge, a second at a time while any of the four is above 1 HP.
    for second = 0, GAUGE_SECONDS do
        local n = math.floor(second * 1000 / GAUGE_FULL_SECONDS)
        at(GAUGE_FROM + second, function(d)
            if not allFloored() then
                d:command(COMMAND_GAUGE, { GAUGE, n })
            end
        end)
    end
    -- It ends at the first of these with all four at 1 HP, and at the last whatever they are (a guess: 67.9 s once,
    -- 77.9 three times; none was missed.)
    for index, seconds in ipairs(PART2_ENDS) do
        at(seconds, function(d)
            if allFloored() or index == #PART2_ENDS then
                secondPartEnds(d)
            end
        end)
    end
    for _, seconds in ipairs(PART2_ON_EACH) do
        at(seconds, function(d) phantomsUse(d, 28498, 4.7, true) end)
    end
    for _, seconds in ipairs(PART2_SWORDS) do
        at(seconds, function(d) swordsCome(d, SWORDS_CAST_PART2) end)
    end
    at(PART2_ROWS, function(d)
        if phantomsUse(d, 29051, 2.7, false) then
            rows(d, ROWS_AFTER_29051, true)
        end
    end)
    at(PART2_SPREAD, function(d)
        if phantomsUse(d, 28491, 2.7, false) then
            after(AFTER_28491, function(d2)
                if fight == 3 then
                    helpersOnEach(d2, 28492, BOSS3_MIDDLE.y)
                end
            end)
        end
    end)
end

function allFloored()
    for _, phantom in ipairs(phantoms) do
        if not floored[phantom] then
            return false
        end
    end
    return true
end

function secondPartEnds(director)
    part2_ending = true
    local function at(seconds, fn)
        after(seconds, function(d)
            if fight == 3 and part2 then
                fn(d)
            end
        end)
    end
    director:battle_talk(BOSS3, NAME_BOSS3, 32908, 0, 4000)
    director:bnpc_control(BOSS3, 70, { 8201036 })
    at(PART2_ALL, function(d) d:actor_use_action(BOSS3, 28499) end)
    at(PART2_ALL + 3.0, function(d) d:bnpc_control(BOSS3, 70, { 8201619 }) end)
    at(PART2_ALL + 5.3, function(d)
        d:bnpc_control(BOSS3, 62, { 0, 0 })
        d:bnpc_control(BOSS3, 63, { 0 })
    end)
    at(PART2_GAUGE_GONE, function(d) d:command(COMMAND_GAUGE, { GAUGE, 0xFFFFFFFF }) end)
    at(PART2_BACK, function(d)
        for _, phantom in ipairs(phantoms) do
            if not gone[phantom] then
                d:despawn_bnpc(phantom)
            end
        end
        phantoms = {}
        d:battle_talk(BOSS3, NAME_BOSS3, PART_AFTER, 0, 5000)
        d:set_targetable(BOSS3, true)
        d:release_bnpc(BOSS3)
        d:restart_timeline(BOSS3, PART_AFTER)
        part2_over = true
    end)
end

-- The four use something together, those still there; whether any did.
function phantomsUse(director, action, cast, on_each)
    local any = false
    for index, phantom in ipairs(phantoms) do
        if not gone[phantom] then
            local member = party[(index - 1) % math.max(1, #party) + 1]
            if not on_each then
                director:actor_use_action(phantom, action)
                any = true
            elseif member ~= nil then
                director:actor_use_action(phantom, action, member.id)
                any = true
            end
        end
    end
    phantoms_free_at = ticks + math.floor((cast + PHANTOM_LOCK) * 2 + 0.5)
    return any
end

-- The last boss and its four of BNpcBase 14455 at 1 HP: each of the four goes once it's free of its cast (at once, or
-- 2.2 s after the cast it was in: 17 times in four runs); the boss ends the duty.
function lastBossHealth(director, info)
    for _, phantom in ipairs(phantoms) do
        local it = info["actor" .. phantom]
        if it ~= nil and it.hp <= 1 then
            floored[phantom] = true
        end
        if floored[phantom] and not gone[phantom] and ticks >= phantoms_free_at then
            gone[phantom] = true
            director:set_targetable(phantom, false)
            director:hold_bnpc(phantom)
            director:play_action_timeline(phantom, 7737)
            after(1.5, function(d) d:despawn_bnpc(phantom) end)
        end
    end
    local boss = info[BOSS3]
    if boss ~= nil and boss.hp <= 1 and (not part2 or part2_over) then
        lastBossDown(director)
    end
end

function lastBossDown(director)
    fight = "over"
    later = {}
    for _, group in ipairs({ swords, blades, phantoms }) do
        for _, actor in ipairs(group) do
            director:despawn_bnpc(actor)
        end
    end
    swords, blades, phantoms = {}, {}, {}
    dropHelpers(director, 3)
    director:show_eobj(EOBJ_EXIT)
    cutscene(director, CUTSCENE_END, CUTSCENE_END_PARAMS)
    director:complete_duty()
    director:despawn_bnpc(BOSS3)
end

function pull3(director)
    fight = 3
    part2 = false
    part2_over = false
    part2_ending = false
    director:battle_talk(BOSS3, NAME_BOSS3, 32904, 0, TALK_MS)
    for _, row in ipairs(BOSS3_TALKS) do
        after(row[1], function(d)
            if fight == 3 and not part2 then
                d:battle_talk(BOSS3, NAME_BOSS3, row[2], 0, TALK_MS)
            end
        end)
    end
    for _, at in ipairs(BOSS3_AWAY) do
        after(at + AWAY_FOR, toTheMiddle)
    end
    after(BOSS3_SLASH_AT, function(d)
        if fight == 3 and not part2 then
            slash(d)
        end
    end)
    after(PART2_AT, secondPart)
    useHelpers(3)
end

function bringLastBoss(director)
    director:spawn_bnpc(BOSS3)
    -- It doesn't fall: at 1 HP the duty is done (its health was 1 as the director said so in every run.)
    director:set_health_floor(BOSS3, 1)
    bringHelpers(director, 3, HELPER_BOSS3, HELPERS_BOSS3, BOSS3_MIDDLE)
    -- Its four of BNpcBase 14455 and five of 14454 are there with it (retail spawned them as it came), waiting:
    -- they can't be targeted until their time.
    swords, blades, phantoms, gone, floored = {}, {}, {}, {}, {}
    for index = 1, #PHANTOMS_AT do
        phantoms[index] = director:spawn_bnpc(PHANTOM, WAITING.x, BOSS3_MIDDLE.y, WAITING.z_phantoms, SWORDS_FACING) or PHANTOM
        -- They don't fall either: at 1 HP each is gone (untargetable, ActionTimeline 7737.)
        director:set_health_floor(phantoms[index], 1)
        director:hold_bnpc(phantoms[index])
        director:set_targetable(phantoms[index], false)
    end
    for index = 1, SWORDS do
        swords[index] = director:spawn_bnpc(SWORD, WAITING.x, BOSS3_MIDDLE.y, WAITING.z_swords, SWORDS_FACING) or SWORD
        director:hold_bnpc(swords[index])
        director:set_targetable(swords[index], false)
    end
end

function dropLastBoss(director)
    director:despawn_bnpc(BOSS3)
    dropHelpers(director, 3)
    for _, group in ipairs({ swords, blades, phantoms }) do
        for _, actor in ipairs(group) do
            director:despawn_bnpc(actor)
        end
    end
    swords, blades, phantoms = {}, {}, {}
end

function onActorCast(director, layout_id, action_id, actor_id, x, y, z)
    if layout_id == BOSS1 and fight == 1 then
        if action_id == 28468 then
            crosses = crosses + 1
            -- A cross the first time (all five runs); the second, a cross twice and turned twice.
            local set = 1
            if crosses > 1 then
                set = math.random(2)
            end
            for _, action in ipairs({ { 28469, SPOTS_28469 }, { 28470, SPOTS_28470 } }) do
                local spots = {}
                for _, spot in ipairs(action[2][set]) do
                    table.insert(spots, { BOSS1_MIDDLE.x + spot[1], BOSS1_MIDDLE.z + spot[2], -3.142 })
                end
                helpersCast(director, action[1], spots, BOSS1_MIDDLE.y)
            end
        elseif action_id == 28471 then
            after(AFTER_28471, function(d)
                if fight == 1 then
                    helpersOnEach(d, 28472, BOSS1_MIDDLE.y)
                end
            end)
        end
    elseif layout_id == BOSS2 and fight == 2 then
        if action_id == 28479 then
            adds = adds + 1
            local at = BOSS2_ADD_AT[(adds - 1) % #BOSS2_ADD_AT + 1]
            after(BOSS2_ADD_AFTER, function(d)
                if fight == 2 then
                    d:spawn_bnpc(BOSS2_ADD, at[1], at[2], at[3], 0.0)
                end
            end)
        end
    elseif layout_id == BOSS3 and fight == 3 and not part2 then
        if action_id == 28484 then
            after(SWORDS_AFTER_28484, function(d)
                if fight == 3 then
                    swordsCome(d, SWORDS_CAST)
                end
            end)
        elseif action_id == 29051 then
            rows(director, ROWS_AFTER_29051, math.random(2) == 1)
        elseif action_id == 28491 then
            after(AFTER_28491, function(d)
                if fight == 3 then
                    helpersOnEach(d, 28492, BOSS3_MIDDLE.y)
                end
            end)
        end
    end
end

function onUpdate(director, health, player_health, info)
    ticks = ticks + 1
    updating = true
    if info ~= nil then
        party = {}
        for key, actor in pairs(info) do
            -- (Someone with no place yet, at the origin, isn't anywhere.)
            if (key == "player" or (type(key) == "string" and key:sub(1, 5) == "party"))
                and not (actor.x == 0 and actor.z == 0) then
                table.insert(party, { x = actor.x, y = actor.y or 0, z = actor.z, id = actor.id,
                                      player = key:sub(1, 6) == "player" })
            end
        end
        theWay(director)
        local function pulled(layout)
            local it = info[layout]
            return it ~= nil and (it.target ~= nil or (health[layout] ~= nil and health[layout] < 100))
        end
        if fight == nil then
            if done.boss1_room and not dead[BOSS1] and pulled(BOSS1) then
                pull1(director)
            elseif done.boss2_room and not dead[BOSS2] and pulled(BOSS2) then
                pull2(director)
            elseif done.last_room and not dead[BOSS3] and pulled(BOSS3) then
                pull3(director)
            end
        end
        if fight == 3 then
            lastBossHealth(director, info)
        end
        if fight == 2 and not done.talk_32903 and health[BOSS2] ~= nil and health[BOSS2] < BOSS2_TALK_BELOW then
            once("talk_32903")
            director:battle_talk(BOSS2, NAME_BOSS2, 32903, 0, 3000)
        end
    end
    runLater(director)
    updating = false
end

function onActorDeath(director, layout_id, position)
    dead[layout_id] = true
    if not done.four_dead and allDead(FOUR) then
        once("four_dead")
        talk(director, 32921)
        director:show_eobj(EOBJ_LIFT1)
        comeOut(director, FIVE)
    end
    if done.four_dead and not done.five_dead and allDead(FIVE) then
        once("five_dead")
        director:map_effect(2, 2, 1)
        comeOut(director, TEN)
    end
    if done.five_dead and not done.ten_dead and allDead(TEN) then
        once("ten_dead")
        director:map_effect(3, 2, 1)
    end
    if layout_id == BOSS1 then
        fight = nil
        director:set_bgm(BGM_AFTER_BOSS1)
        director:objective(1, 2)
        director:hide_eobj(BOSS1_AHEAD)
        director:show_eobj(EOBJ_HANGAR)
        director:show_eobj(EOBJ_ARMOUR)
        comeOut(director, AHEAD)
        comeOut(director, RIDE)
        dropHelpers(director, 1)
        after(1.0, function(d) talk(d, 32916) end)
    end
    if dead[BOSS1] and not done.effect_5 and allDead(LAST_FIVE) then
        once("effect_5")
        director:map_effect(5, 2, 1)
    end
    if layout_id == WALKERS[1] then
        director:map_effect(12, 1, 1)
    elseif layout_id == WALKERS[2] then
        director:map_effect(13, 1, 1)
    elseif layout_id == WALKERS[3] then
        director:map_effect(11, 2, 1)
        after(0.6, function(d) d:spawn_bnpc(WALKERS[4]) end)
    elseif layout_id == WALKERS[4] then
        director:map_effect(14, 1, 1)
        director:map_effect(18, 2, 1)
        director:spawn_bnpc(WALKER_LAST)
    elseif layout_id == WALKER_LAST then
        once("ride_ended")
        director:map_effect(6, 1, 4)
        cutscene(director, CUTSCENE_RIDE, CUTSCENE_RIDE_PARAMS)
        -- The mount goes 1.6 s into the cutscene (SetMode 1 and ActorControl 930 on retail.)
        after(1.6, function(d)
            for _, member in ipairs(party) do
                d:mount_player(member.id, 0)
            end
        end)
        after(2.0, function(d) d:map_effect(7, 4, 4) end)
    end
    if layout_id == BOSS2 then
        fight = nil
        director:objective(3, 4)
        director:map_effect(0, 8, 4)
        director:map_effect(15, 1, 4)
        director:hide_eobj(BOSS2_AHEAD)
        director:show_eobj(EOBJ_LAST)
        dropHelpers(director, 2)
        director:despawn_bnpc(BOSS2_ADD)
        cutscene(director, CUTSCENE_AFTER_BOSS2, CUTSCENE_AFTER_BOSS2_PARAMS)
    end
    if layout_id == BOSS3 and fight == 3 then
        lastBossDown(director)
    end
end

function onSceneFinished(director, scene)
    local ended = playing
    playing = nil
    if ended == CUTSCENE_RIDE then
        once("ride_over")
    elseif ended == CUTSCENE_BOSS3 then
        sendTo(director, BOSS3_TO)
        -- The boss and its own are there 1.7 to 3.9 s after the cutscene ends (as the party arrives.)
        after(1.5, function(d)
            once("last_room")
            bringLastBoss(d)
        end)
    elseif ended == CUTSCENE_END then
        director:warp_players(OUT_TO.x, OUT_TO.y, OUT_TO.z, OUT_TO.rotation)
    end
end

-- Everyone down: the fight they fell in is forgotten (the first two bosses' rooms reset themselves); the last boss
-- and everything of its fight goes, and comes back as the party does.
function onPartyWipe(director)
    if fight == "over" then
        return
    end
    local was = fight
    fight = nil
    later = {}
    helpers = {}
    if was == 2 then
        director:despawn_bnpc(BOSS2_ADD)
        done.talk_32903 = nil
    elseif was == 3 then
        part2 = false
        dropLastBoss(director)
        last_boss_due = true
    end
end

function onDutyRecommence(director)
    if last_boss_due then
        last_boss_due = false
        bringLastBoss(director)
    end
end

-- The lifts and the other objects: each works from its own floor once the way to it is open.
function objectWorks(id, member)
    local y = member ~= nil and member.y or nil
    local function floor(low, high)
        return y == nil or (y > low and y < high)
    end
    if id == EOBJ_LIFT1 then
        return done.four_dead == true and floor(175, 200)
    elseif id == EOBJ_LIFT2 then
        return done.ten_dead == true and floor(145, 170)
    elseif id == EOBJ_HANGAR then
        return dead[BOSS1] == true and floor(60, 90)
    elseif id == EOBJ_ARMOUR then
        return dead[BOSS1] == true and not done.ride_ended and floor(95, 110)
    elseif id == EOBJ_LAST then
        return dead[BOSS2] == true and not done.object_last and floor(-120, -90)
    end
    return false
end

function onGimmickAccessor(director, actor_id, id, params)
    if objectWorks(id, whoIs(actor_id)) then
        director:event_action(EVENT_ACTION_OBJECT, actor_id, id)
        return
    end
    director:finish_gimmick(actor_id)
end

function isGimmickFunctional(id)
    return objectWorks(id, nil)
end

function onEventActionCast(director, actor_id, target)
    director:finish_gimmick(actor_id)
    if not objectWorks(target, whoIs(actor_id)) then
        return
    end
    if target == EOBJ_LIFT1 then
        sendTo(director, LIFT1_TO)
        if once("lift1") then
            after(4.6, function(d) d:map_effect(1, 2, 1) end)
        end
    elseif target == EOBJ_LIFT2 then
        once("lift2")
        sendTo(director, LIFT2_TO)
    elseif target == EOBJ_HANGAR then
        sendTo(director, HANGAR_TO)
        if once("hangar") then
            after(5.7, function(d) talk(d, 32917) end)
            after(12.7, function(d) talk(d, 32918) end)
        end
    elseif target == EOBJ_ARMOUR then
        director:mount_player(actor_id, MOUNT_ARMOUR)
    elseif target == EOBJ_LAST then
        once("object_last")
        director:objective(4, 5)
        director:map_effect(16, 4096, 8192)
        director:hide_eobj(EOBJ_LAST)
        director:show_eobj(EOBJ_AFTER_LAST)
        cutscene(director, CUTSCENE_BOSS3, CUTSCENE_BOSS3_PARAMS)
    end
end

function getDebugShortcut(id)
    return 0
end
