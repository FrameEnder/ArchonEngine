-- The Copperbell Mines
--
-- From Bob's Chronofoil captures of 2026-10-25: one run with Duty Support (bd298651, visit 0.5, during Into a Copper
-- Hell) and two Duty Finder runs (69bea0b9, visits 0.1 and 0.3); `questkit chronofoil --battle` and the visits'
-- packets (docs/src/fork/dungeons.md). Numbers only; names and text are the game's.
--
-- Three times the way is blocked: the first and third by a rock face that a charge brings down, the second and fourth
-- by a door that opens once two items are set by it. A charge is an item an enemy drops (or one lying past the first
-- boss); set by the rock, it's set off by the detonator beside it, and 10 s later the way is clear. The items for the
-- doors lie about or are dropped by the gigas (BNpcBase 1400). Past each door a boss appears at once, sealing its
-- arena as it goes for someone; past the last door, coming down into the last hall plays the last boss's cutscene and
-- brings it.

-- The director's variables (DirectorVars; retail's sequence was 1 throughout): data 1 is how far the party got (1 past
-- the first door, 3 the second, 7 the third), 2 whether it holds a charge, 3 how many of the doors' items it holds.
VAR_PROGRESS = 1
VAR_CHARGE = 2
VAR_ITEMS = 3

-- Log messages retail sent (LogMessage rows): an enemy dropped an item (its BNpcName, the item), an item obtained (the
-- item), a charge set (758), the items set by a door, and each door opening.
LOG_DROPPED = 2069
LOG_OBTAINED = 2031
LOG_CHARGE_SET = 2059
LOG_CHARGE_SET_PARAM = 758
LOG_ITEMS_SET = 2060
LOG_DOOR1 = 2061
LOG_DOOR2 = 2065
LOG_DOOR3 = 2062
-- The items, by EventItem ID: the charge and the doors' items
ITEM_CHARGE = 2000441
ITEM_DOOR = 2000442

-- The gimmicks (EObj IDs; each a GimmickAccessor event, 0x000F00xx)
EOBJ_CHARGE1 = 2000159 -- dropped by the last of the first pack (spawned where it fell)
EOBJ_ROCK1 = 2000160 -- where the first charge is set
EOBJ_DETONATOR1 = 2000163 -- the lever (layout 4055428); the blast goes off on 2000161 (layout 2790162)
EOBJ_ITEM_A = 2000169
EOBJ_ITEM_B = 2000172
EOBJ_DOOR1_SLOT = 2001536 -- where the first door's items are set
EOBJ_DOOR1 = 2000170
EOBJ_CHARGE2 = 2000178 -- shown as the first boss falls
EOBJ_ROCK2 = 2000173
EOBJ_DETONATOR2 = 2000175
EOBJ_ITEM_C = 2001531 -- dropped by the gigas 2790234
EOBJ_ITEM_D = 2000179 -- dropped by the gigas 2790237
EOBJ_DOOR2_SLOT = 2001537
EOBJ_DOOR2 = 2000180
EOBJ_ITEM_E = 2001532
EOBJ_ITEM_F = 2001533 -- shown as the second boss falls
EOBJ_DOOR3_SLOT = 2001538
EOBJ_DOOR3 = 2001534

EVENT_ACTION_INTERACT = 24

-- What each gimmick does to the scenery, by layout ID (EObj IDs repeat here: the boss walls share theirs with a line.)
-- A blast: ActorControl 413 (8, 2) on the charge, at once these shown and those hidden, 10 s later the rest.
BLASTS = {
    {
        charge = 2790162,
        show = { 4075277 },
        hide = { 4055428, 4055460 },
        later_show = { 2790163, 2790166 },
        later_hide = { 4075279 },
    },
    {
        charge = 4056538,
        show = { 4075280 },
        hide = { 2790185, 2790225 },
        later_show = { 2790184, 2790226 },
        later_hide = { 4075282 },
    },
}
AC_BLAST = 413
AC_BLAST_PARAMS = { 8, 2 }
BLAST_DELAY = 10
-- Each door's frame, hidden with the door as it opens
DOOR_FRAMES = { [EOBJ_DOOR1] = 2790178, [EOBJ_DOOR2] = 2790242, [EOBJ_DOOR3] = 4056716 }

-- Hidden from the start on retail (spawned in state 7) and shown later, or never.
HIDDEN_AT_START = {
    2790166, 4075277, 2790163, 2790171, 2790239, 2790184, 2790226, 4075280, 4056715, 4056736, 4056699,
}

-- The bosses: layout ID, arena wall and line (layout IDs), its PlaceName, and a second line hidden with them.
BOSSES = {
    { bnpc = 9053767, wall = 9085524, line = 9085538, place = 679, other_line = 9085522, music = true },
    { bnpc = 9040077, wall = 9089361, line = 9089385, place = 680, other_line = 9089366, music = true },
    { bnpc = 9060676, wall = 9082207, line = 9082214, place = 681 },
}
BGM_BATTLE = 37

-- Shown as the first and second bosses fall (by layout ID)
AFTER_BOSS = {
    { 2790239 },
    { 4056699, 4056715, 4056736 },
}

-- The last hall: coming down into it (retail: the cutscene began as the first player crossed z 55, 52-56 in two runs,
-- and not while they waited at 58-63) plays the last boss's cutscene; it comes 2.4 s later. Then ActorControl 409 (64,
-- the director) on the EObj 4217260.
FINAL_AREA = { min_x = -115, max_x = -75, max_y = -50, max_z = 55 }
CUTSCENE_FINAL = 178
CUTSCENE_END = 179
CUTSCENE_END_PARAMS = { 1, 18, 0, 0 }
EOBJ_FINAL_STAGE = 4217260
AC_FINAL_STAGE = 409
AC_FINAL_STAGE_PARAMS = { 64, 0x80030003 }

-- The ambushes: two bigger gigas (BNpcBase 1398, 1403) waiting in a pile of rock until someone comes within about 20
-- yalms (19.4-22 in Bob's three runs), when the pile goes and the gigas comes at them. Retail spawned it hidden (its
-- display flags, in the drop-in) and going for the player at once, then 0.6-0.9 s later brought it into view, fading
-- in and playing ActionTimeline 158, and made it targetable (all six ambushes).
AMBUSHES = {
    { bnpc = 4023421, rocks = 2790172, x = -74.4, y = -6.0, z = -165.4 },
    { bnpc = 4023560, rocks = 2790295, x = -54.6, y = -37.9, z = 97.9 },
}
AMBUSH_RANGE = 22
AMBUSH_TIMELINE = 158

-- Who drops what: the first pack's last (by layout ID, with its BNpcName for the log), and the two gigas.
FIRST_PACK = { [4021731] = 631, [4021732] = 629, [4021733] = 629 }
DROPS = {
    [2790234] = { eobj = EOBJ_ITEM_C, name = 1303, item = ITEM_DOOR },
    [2790237] = { eobj = EOBJ_ITEM_D, name = 1303, item = ITEM_DOOR },
}

-- What the coffers held on retail, one list per opening (item IDs; the three runs.) 59 and 65 were never opened and
-- hold one of these. See server/loot.rs.
LOOT = {
    [52] = { { 3554 }, { 3554, 4310 }, { 3555, 4310 } },
    [57] = { { 3554 } },
    [53] = { { 3034 }, { 3033 }, { 3035 } },
    [58] = { { 3033 }, { 3035 } },
    [51] = { { 31423 }, { 3786, 31428, 33130 }, { 3781, 31427 } },
}
-- What the coffers may hold besides what was recorded (`director:treasure_pool`): the FFXIV Console Games Wiki's loot
-- lists for the duty (each boss's coffer and each "Treasure Coffer" at its map position, matched to these base IDs by
-- the recorded items and by the coffers' positions in Bob's Chronofoil captures), as item IDs from the Item sheet. The
-- starting point: a recording adds what the wiki misses (`server/loot.rs`). The wiki's fixed-rate extras (a Triple
-- Triad card, an orchestrion material) aren't in these.
HEAD_HANDS = { 2684, 2683, 2679, 3556, 3554, 3555 }
ACCESSORIES = { 4206, 4207, 4310, 4096 }
BODY_LEGS = { 3033, 3034, 3035, 3331, 3329, 3330 }
WEAPONS = { 38467, 31422, 31423, 31421, 31425, 31424, 31427, 31428, 31426 }
FEET = { 3781, 3786, 3787 }
-- The bronze coffer by the weapons' list also has these (older pieces of the same levels.)
OLDER_PIECES = { 2235, 2682, 3031, 3030, 3032, 3783, 3782 }

function joined(...)
    local all = {}
    for _, list in ipairs({ ... }) do
        for _, item in ipairs(list) do
            table.insert(all, item)
        end
    end
    return all
end

-- The wiki's lists: Kottos's silver coffer (52), Ichorous Ire's (53), Gyges the Great's gold one (51), and Treasure
-- Coffers 1-4 (65, 57, 58, 59.)
WIKI_LOOT = {
    [52] = joined(HEAD_HANDS, ACCESSORIES),
    [53] = BODY_LEGS,
    [51] = joined(WEAPONS, FEET, { 33130 }),
    [65] = joined(FEET, ACCESSORIES),
    [57] = HEAD_HANDS,
    [58] = BODY_LEGS,
    [59] = joined(WEAPONS, OLDER_PIECES),
}
STANDING_COFFERS = { 65, 57, 58 }
BOSS_COFFERS = { { 52 }, { 53, 59 }, { 51 } }

-- The Duty Support members' story lines (NpcYell), when retail's said them (the Thaumaturge 40.3 s in, the tank 6.8 s
-- after the first boss appeared, the Lancer 4.8 s after the second blast cleared the way, the Thaumaturge as the second
-- boss was pulled, the Lancer 29.4 s after the last boss's cutscene began.)
LINE_START = 14315
LINE_START_AFTER = 40.3
LINE_BOSS1 = 14318
LINE_BOSS1_AFTER = 6.8
LINE_BLAST2 = 14320
LINE_BLAST2_AFTER = BLAST_DELAY + 4.8
LINE_BOSS2 = 14322
LINE_FINAL = 14324
LINE_FINAL_AFTER = 29.4

-- How far the party has got
charges_set = {}
blasted = {}
doors_set = {}
doors_open = {}
bosses_pulled = {}
ambushed = {}
first_pack_left = 3
final_started = false
-- Things to do later: { at = os.time(), fn = function(director) }
later = {}

-- DirectorVars' data is 0-based (the first byte after the flag and branch): data 1 is `set_data(0, ...)`. Sent one
-- higher, the Duty Information window counted nothing (Bob's bundle of 2026-10-25: the charge went to the items' byte.)
function setData(director, index, value)
    director:set_data(index - 1, value)
end

function after(seconds, fn)
    table.insert(later, { at = os.time() + seconds, fn = fn })
end

function onSetup(director)
    director:set_sequence(1)

    for _, layout in ipairs(HIDDEN_AT_START) do
        director:hide_eobj(layout)
    end
    for _, boss in ipairs(BOSSES) do
        director:hide_eobj(boss.wall)
    end

    for coffer, openings in pairs(LOOT) do
        director:treasure_loot(coffer, openings)
    end
    for coffer, items in pairs(WIKI_LOOT) do
        director:treasure_pool(coffer, items)
    end
    for _, coffer in ipairs(STANDING_COFFERS) do
        director:spawn_treasure(coffer)
    end

    director:party_npc_yell("ranged", LINE_START, LINE_START_AFTER)
end

function bossIndex(layout_id)
    for index, boss in ipairs(BOSSES) do
        if boss.bnpc == layout_id then
            return index
        end
    end
    return nil
end

-- Kottos (the first boss)'s leaps, as retail played them in all three runs (Bob's Chronofoil captures of 2026-10-25):
-- three invisible helpers (BNpcBase 9020, all layout 9054923) come with it. As it begins 28543 (7.7 s, no target)
-- they're put on three of four spots 7 yalms north, east, south and west of the arena's middle (ActorSetPos) and each
-- casts 28544 there (8.7 s), 2 s apart: the first on north or south, the second on east or west, the third opposite
-- the first. As each one goes off, Kottos jumps onto its spot (ActorControl 239, then an ActorMove there, facing the
-- way it leapt), 9.1, 11.2 and 13.3 s after 28543 began, and uses 28549 (instant, round itself) 1.0 s after the first
-- two landings (10.1 and 12.2 s); its timeline's 28546 comes 1.0 s after the third (14.3 s). From 28543 until then it
-- does nothing else: no auto-attacks, no walking (`busy_bnpc`), as on retail (its first auto-attack after came as
-- 28546 landed.)
KOTTOS = 9053767
KOTTOS_LEAP = 28543
KOTTOS_HELPER = 9054923
KOTTOS_MARK = 28544
KOTTOS_MIDDLE = { x = 43.0, z = -89.56 }
KOTTOS_SPOT_OFFSET = 7.0
KOTTOS_HELPER_Y = -9.266 -- where the helpers stood
KOTTOS_LANDING_Y = -9.78 -- where Kottos landed
KOTTOS_MARK_EVERY = 2.0
KOTTOS_JUMPS = { 9.1, 11.2, 13.3 }
KOTTOS_SLAM = 28549
KOTTOS_SLAMS = { 10.1, 12.2 }
KOTTOS_BUSY = 14.2 -- until its timeline's 28546 (14.3 s after 28543)
kottos_helpers = {}

function kottosSpot(name, y)
    local m, o = KOTTOS_MIDDLE, KOTTOS_SPOT_OFFSET
    local spots = {
        north = { x = m.x, z = m.z - o },
        south = { x = m.x, z = m.z + o },
        east = { x = m.x + o, z = m.z },
        west = { x = m.x - o, z = m.z },
    }
    local spot = spots[name]
    return { x = spot.x, y = y, z = spot.z }
end

function kottosLeaps(director, kottos)
    local first = (math.random(2) == 1) and "north" or "south"
    local middle = (math.random(2) == 1) and "east" or "west"
    local last = (first == "north") and "south" or "north"
    local order = { first, middle, last }
    -- The helpers in the order retail used them: the last spawned first.
    local helpers = { kottos_helpers[3], kottos_helpers[2], kottos_helpers[1] }
    for i, name in ipairs(order) do
        local spot = kottosSpot(name, KOTTOS_HELPER_Y)
        director:warp_bnpc(helpers[i], spot.x, spot.y, spot.z, 0)
    end
    for i = 1, 3 do
        director:delay((i - 1) * KOTTOS_MARK_EVERY)
        director:actor_use_action(helpers[i], KOTTOS_MARK)
    end
    director:delay(0)
    director:busy_bnpc(kottos, KOTTOS_BUSY)
    for i, name in ipairs(order) do
        local spot = kottosSpot(name, KOTTOS_LANDING_Y)
        director:delay(KOTTOS_JUMPS[i])
        director:jump_bnpc(kottos, spot.x, spot.y, spot.z) -- facing the way it leapt
        if KOTTOS_SLAMS[i] ~= nil then
            director:delay(KOTTOS_SLAMS[i])
            director:actor_use_action(kottos, KOTTOS_SLAM)
        end
    end
    director:delay(0)
end

-- The second boss (14448)'s six helpers (BNpcBase 14449, all layout 9040086; never fought, never killed), as retail
-- played them in all three runs: as the boss's 28463 (2.7 s, on itself) lands, the boss grows a size (ActorControl 48:
-- ModelScale 16, 17, 18 in the three rounds) and its six helpers are put on its middle (ActorSetPos), each use 28464
-- (instant, hitting nobody) facing the way it will cast, leap to their spots (ActorControl 239 and an ActorMove) and
-- cast 28465 (5.7 s, a line, 121-127 to the players caught) facing outward. Two stand on a diagonal (one of two) and
-- four in a row or a column 12 yalms apart through the middle (each round picks both at random: in the runs A+row,
-- B+row, B+column, A+column). Times from 28463's cast beginning: 4.0 s (grow, put on the middle), 4.1 (28464), 4.7
-- (leap), 8.7 (28465).
BOSS2 = 9040077
BOSS2_GROW = 28463
BOSS2_HELPER = 9040086
BOSS2_TURN = 28464
BOSS2_LINE = 28465
BOSS2_MIDDLE = { x = 27.0, y = -37.958, z = 114.0 }
BOSS2_Y = -37.98 -- where the helpers landed
BOSS2_FIRST_SCALE = 16
BOSS2_DIAGONALS = {
    { { x = 14.27, z = 126.73, r = -0.785 }, { x = 39.72, z = 101.24, r = 2.356 } },
    { { x = 39.72, z = 126.73, r = 0.785 }, { x = 14.27, z = 101.24, r = -2.356 } },
}
BOSS2_LINES = {
    { { x = 45.0, z = 113.97, r = 1.571 }, { x = 32.97, z = 113.97, r = 1.571 },
      { x = 20.98, z = 113.97, r = -1.571 }, { x = 8.99, z = 113.97, r = -1.571 } }, -- a row
    { { x = 26.99, z = 131.98, r = 0.0 }, { x = 26.99, z = 119.98, r = 0.0 },
      { x = 26.99, z = 107.99, r = 3.142 }, { x = 26.99, z = 95.99, r = 3.142 } }, -- a column
}
boss2_helpers = {}
boss2_rounds = 0

function boss2Lines(director)
    boss2_rounds = boss2_rounds + 1
    local spots = {}
    for _, spot in ipairs(BOSS2_DIAGONALS[math.random(2)]) do
        table.insert(spots, spot)
    end
    for _, spot in ipairs(BOSS2_LINES[math.random(2)]) do
        table.insert(spots, spot)
    end
    local m = BOSS2_MIDDLE
    director:delay(4.0)
    director:set_model_scale(BOSS2, BOSS2_FIRST_SCALE + boss2_rounds - 1)
    for i, helper in ipairs(boss2_helpers) do
        director:warp_bnpc(helper, m.x, m.y, m.z, spots[i].r)
    end
    director:delay(4.1)
    for _, helper in ipairs(boss2_helpers) do
        director:actor_use_action(helper, BOSS2_TURN)
    end
    director:delay(4.7)
    for i, helper in ipairs(boss2_helpers) do
        director:jump_bnpc(helper, spots[i].x, BOSS2_Y, spots[i].z, spots[i].r)
    end
    director:delay(8.7)
    for _, helper in ipairs(boss2_helpers) do
        director:actor_use_action(helper, BOSS2_LINE)
    end
    director:delay(0)
end

-- The last boss (14537)'s five helpers (BNpcBase 9020, all layout 9060684), as retail played them in all three runs: as
-- the boss begins 28760 (5.7 s, on no one, at the arena's middle, where it walks first) they're put (ActorSetPos) on
-- five spots facing the middle: four 14 yalms out, at an axis (north-south or east-west) and each of its ends turned
-- 60 degrees the same way, and one 7 yalms out across the axis, on the side the turn leaves open (in the runs: 0, 60,
-- 180, 240 with 270; 90, 150, 270, 330 with 0; 0, 300, 180, 120 with 90, degrees from north). 3.0 s later they cast
-- 28761 (5.7 s, a line through the middle, 127 to the player caught). 1.1 s after 28760 lands the boss gets ActorControl
-- 239 where it stands, and goes back to its target.
BOSS3 = 9060676
BOSS3_PULL = 28760
BOSS3_HELPER = 9060684
BOSS3_LINE = 28761
BOSS3_MIDDLE = { x = -100.41, y = -57.79, z = 6.67 }
BOSS3_HELPER_Y = -58.27
BOSS3_OUTER = 14.0
BOSS3_INNER = 7.0
BOSS3_LINE_AFTER = 3.0
BOSS3_SETTLE_AFTER = 7.1
boss3_helpers = {}

function boss3Spot(degrees, distance)
    local angle = math.rad(degrees)
    local x = BOSS3_MIDDLE.x + distance * math.sin(angle)
    local z = BOSS3_MIDDLE.z + distance * math.cos(angle)
    -- Facing the middle
    return { x = x, z = z, r = math.atan(BOSS3_MIDDLE.x - x, BOSS3_MIDDLE.z - z) }
end

function boss3Lines(director)
    local axis = (math.random(2) == 1) and 0 or 90
    local turn = (math.random(2) == 1) and 60 or -60
    local spots = { boss3Spot(axis - 1.5 * turn, BOSS3_INNER) }
    for _, degrees in ipairs({ axis, axis + turn, axis + 180, axis + 180 + turn }) do
        table.insert(spots, boss3Spot(degrees, BOSS3_OUTER))
    end
    local m = BOSS3_MIDDLE
    director:move_bnpc(BOSS3, m.x, m.y, m.z, 0)
    for i, helper in ipairs(boss3_helpers) do
        director:warp_bnpc(helper, spots[i].x, BOSS3_HELPER_Y, spots[i].z, spots[i].r)
    end
    director:delay(BOSS3_LINE_AFTER)
    for _, helper in ipairs(boss3_helpers) do
        director:actor_use_action(helper, BOSS3_LINE)
    end
    director:delay(BOSS3_SETTLE_AFTER)
    director:jump_bnpc(BOSS3, m.x, m.y, m.z, 0)
    director:delay(0)
end

function spawnHelpers(director, layout, count)
    local helpers = {}
    for i = 1, count do
        helpers[i] = director:spawn_bnpc(layout)
        director:hold_bnpc(helpers[i])
    end
    return helpers
end

-- The helpers go with their boss: retail let Kottos' and the last boss's go (ActorControl 417 with no one) the moment
-- it fell; the second boss's simply went.
function dismissHelpers(director, helpers, release)
    for _, helper in ipairs(helpers) do
        if release then
            director:release_bnpc(helper)
        end
        director:despawn_bnpc(helper)
    end
end

function onActorCast(director, layout_id, action_id, actor_id)
    if layout_id == KOTTOS and action_id == KOTTOS_LEAP and #kottos_helpers == 3 then
        kottosLeaps(director, actor_id)
    elseif layout_id == BOSS2 and action_id == BOSS2_GROW and #boss2_helpers == 6 then
        boss2Lines(director)
    elseif layout_id == BOSS3 and action_id == BOSS3_PULL and #boss3_helpers == 5 then
        boss3Lines(director)
    end
end

function spawnBoss(director, index)
    local boss = BOSSES[index]
    director:spawn_boss(boss.bnpc, boss.wall, boss.line, boss.place)
    if index == 1 then
        kottos_helpers = spawnHelpers(director, KOTTOS_HELPER, 3)
    elseif index == 2 then
        boss2_helpers = spawnHelpers(director, BOSS2_HELPER, 6)
        boss2_rounds = 0
    elseif index == 3 then
        boss3_helpers = spawnHelpers(director, BOSS3_HELPER, 5)
    end
end

function onUpdate(director, health, player_health, info)
    local now = os.time()
    local due = {}
    local waiting = {}
    for _, entry in ipairs(later) do
        if entry.at <= now then
            table.insert(due, entry)
        else
            table.insert(waiting, entry)
        end
    end
    later = waiting
    for _, entry in ipairs(due) do
        entry.fn(director)
    end

    if info == nil then
        return
    end

    -- A boss going for someone: the battle music (the first two), the second's line
    for index, boss in ipairs(BOSSES) do
        if not bosses_pulled[index] and info[boss.bnpc] ~= nil and info[boss.bnpc].target ~= nil then
            bosses_pulled[index] = true
            if boss.music then
                director:set_bgm(BGM_BATTLE)
            end
            if index == 2 then
                director:party_npc_yell("ranged", LINE_BOSS2)
            end
        end
    end

    local player = info.player
    if player == nil then
        return
    end

    for index, ambush in ipairs(AMBUSHES) do
        if not ambushed[index] then
            local dx, dy, dz = player.x - ambush.x, player.y - ambush.y, player.z - ambush.z
            if dx * dx + dy * dy + dz * dz <= AMBUSH_RANGE * AMBUSH_RANGE then
                ambushed[index] = true
                director:hide_eobj(ambush.rocks)
                director:spawn_bnpc(ambush.bnpc)
                director:set_hate(ambush.bnpc, nil, 300) -- the player
                director:appear_bnpc(ambush.bnpc, AMBUSH_TIMELINE)
            end
        end
    end

    if not final_started and doors_open[EOBJ_DOOR3] and player.x >= FINAL_AREA.min_x
        and player.x <= FINAL_AREA.max_x and player.y <= FINAL_AREA.max_y and player.z <= FINAL_AREA.max_z then
        final_started = true
        director:play_cutscene(CUTSCENE_FINAL)
        spawnBoss(director, 3)
        director:eobj_control(EOBJ_FINAL_STAGE, AC_FINAL_STAGE, AC_FINAL_STAGE_PARAMS)
        director:party_npc_yell("melee", LINE_FINAL, LINE_FINAL_AFTER)
    end
end

function onActorDeath(director, bnpc_id, position)
    local where = { x = position.x, y = position.y, z = position.z }

    if FIRST_PACK[bnpc_id] ~= nil then
        first_pack_left = first_pack_left - 1
        if first_pack_left == 0 then
            director:log_message(LOG_DROPPED, { FIRST_PACK[bnpc_id], ITEM_CHARGE })
            director:spawn_eobj(EOBJ_CHARGE1, where)
        end
        return
    end

    local drop = DROPS[bnpc_id]
    if drop ~= nil then
        director:log_message(LOG_DROPPED, { drop.name, drop.item })
        director:spawn_eobj(drop.eobj, where)
        return
    end

    local index = bossIndex(bnpc_id)
    if index == nil then
        return
    end
    -- spawn_boss opens the arena (wall and line hidden, log 2014); the music stops and the coffers appear.
    local boss = BOSSES[index]
    if boss.other_line ~= nil then
        director:hide_eobj(boss.other_line)
    end
    if boss.music then
        director:set_bgm(0)
    end
    for _, coffer in ipairs(BOSS_COFFERS[index]) do
        director:spawn_treasure(coffer)
    end
    for _, layout in ipairs(AFTER_BOSS[index] or {}) do
        director:show_eobj(layout)
    end
    if index == 1 then
        dismissHelpers(director, kottos_helpers, true)
        kottos_helpers = {}
    elseif index == 2 then
        dismissHelpers(director, boss2_helpers, false)
        boss2_helpers = {}
    elseif index == 3 then
        dismissHelpers(director, boss3_helpers, true)
        boss3_helpers = {}
        director:play_cutscene(CUTSCENE_END, CUTSCENE_END_PARAMS)
        director:complete_duty()
    end
end

GIMMICKS = {
    [EOBJ_CHARGE1] = true, [EOBJ_ROCK1] = true, [EOBJ_DETONATOR1] = true, [EOBJ_ITEM_A] = true,
    [EOBJ_ITEM_B] = true, [EOBJ_DOOR1_SLOT] = true, [EOBJ_DOOR1] = true, [EOBJ_CHARGE2] = true,
    [EOBJ_ROCK2] = true, [EOBJ_DETONATOR2] = true, [EOBJ_ITEM_C] = true, [EOBJ_ITEM_D] = true,
    [EOBJ_DOOR2_SLOT] = true, [EOBJ_DOOR2] = true, [EOBJ_ITEM_E] = true, [EOBJ_ITEM_F] = true,
    [EOBJ_DOOR3_SLOT] = true, [EOBJ_DOOR3] = true,
}

function onGimmickAccessor(director, actor_id, id, params)
    if GIMMICKS[id] then
        director:event_action(EVENT_ACTION_INTERACT, actor_id, id)
        return
    end

    director:finish_gimmick(actor_id)
end

-- Which slot comes before each door, and which rock before each detonator
SLOT_OF_DOOR = { [EOBJ_DOOR1] = EOBJ_DOOR1_SLOT, [EOBJ_DOOR2] = EOBJ_DOOR2_SLOT, [EOBJ_DOOR3] = EOBJ_DOOR3_SLOT }
ROCK_OF_DETONATOR = { [EOBJ_DETONATOR1] = EOBJ_ROCK1, [EOBJ_DETONATOR2] = EOBJ_ROCK2 }

has_charge = false
items_held = 0

-- Retail refused a door's slot without both items (log 2030) and a door before its slot was used (log 2029); the logs
-- are the gimmick's own (the server sends them when this says no.)
function isGimmickFunctional(id)
    if id == EOBJ_ROCK1 or id == EOBJ_ROCK2 then
        return has_charge
    elseif ROCK_OF_DETONATOR[id] ~= nil then
        return charges_set[ROCK_OF_DETONATOR[id]] == true and not blasted[id]
    elseif id == EOBJ_DOOR1_SLOT or id == EOBJ_DOOR2_SLOT or id == EOBJ_DOOR3_SLOT then
        return items_held >= 2 and not doors_set[id]
    elseif SLOT_OF_DOOR[id] ~= nil then
        return doors_set[SLOT_OF_DOOR[id]] == true and not doors_open[id]
    end

    return true
end

function blast(director, index, detonator)
    blasted[detonator] = true
    local blast = BLASTS[index]
    director:eobj_control(blast.charge, AC_BLAST, AC_BLAST_PARAMS)
    for _, layout in ipairs(blast.show) do
        director:show_eobj(layout)
    end
    for _, layout in ipairs(blast.hide) do
        director:hide_eobj(layout)
    end
    after(BLAST_DELAY, function(director)
        for _, layout in ipairs(blast.later_show) do
            director:show_eobj(layout)
        end
        for _, layout in ipairs(blast.later_hide) do
            director:hide_eobj(layout)
        end
    end)
end

function onEventActionCast(director, actor_id, target)
    director:finish_gimmick(actor_id)

    if (target == EOBJ_CHARGE1 or target == EOBJ_CHARGE2) and not has_charge then
        has_charge = true
        director:log_message(LOG_OBTAINED, { ITEM_CHARGE })
        director:hide_eobj(target)
        setData(director, VAR_CHARGE, 1)
    elseif (target == EOBJ_ROCK1 or target == EOBJ_ROCK2) and has_charge and not charges_set[target] then
        has_charge = false
        charges_set[target] = true
        director:log_message(LOG_CHARGE_SET, { LOG_CHARGE_SET_PARAM })
        director:hide_eobj(target)
        setData(director, VAR_CHARGE, 0)
    elseif target == EOBJ_DETONATOR1 and charges_set[EOBJ_ROCK1] and not blasted[target] then
        blast(director, 1, target)
    elseif target == EOBJ_DETONATOR2 and charges_set[EOBJ_ROCK2] and not blasted[target] then
        blast(director, 2, target)
        director:party_npc_yell("melee", LINE_BLAST2, LINE_BLAST2_AFTER)
    elseif target == EOBJ_ITEM_A or target == EOBJ_ITEM_B or target == EOBJ_ITEM_C or target == EOBJ_ITEM_D
        or target == EOBJ_ITEM_E or target == EOBJ_ITEM_F then
        items_held = items_held + 1
        director:log_message(LOG_OBTAINED, { ITEM_DOOR })
        director:hide_eobj(target)
        setData(director, VAR_ITEMS, items_held)
    elseif (target == EOBJ_DOOR1_SLOT or target == EOBJ_DOOR2_SLOT or target == EOBJ_DOOR3_SLOT)
        and items_held >= 2 and not doors_set[target] then
        items_held = items_held - 2
        doors_set[target] = true
        director:log_message(LOG_ITEMS_SET, {})
        director:hide_eobj(target)
        setData(director, VAR_ITEMS, items_held)
    elseif target == EOBJ_DOOR1 and doors_set[EOBJ_DOOR1_SLOT] and not doors_open[target] then
        openDoor(director, target, 1, LOG_DOOR1)
        spawnBoss(director, 1)
        director:party_npc_yell("tank", LINE_BOSS1, LINE_BOSS1_AFTER)
    elseif target == EOBJ_DOOR2 and doors_set[EOBJ_DOOR2_SLOT] and not doors_open[target] then
        openDoor(director, target, 3, LOG_DOOR2)
        spawnBoss(director, 2)
    elseif target == EOBJ_DOOR3 and doors_set[EOBJ_DOOR3_SLOT] and not doors_open[target] then
        openDoor(director, target, 7, LOG_DOOR3)
    end
end

function openDoor(director, door, progress, log)
    doors_open[door] = true
    director:hide_eobj(DOOR_FRAMES[door])
    director:hide_eobj(door)
    setData(director, VAR_PROGRESS, progress)
    director:log_message(log, {})
end

-- No shortcuts yet (their pop ranges aren't known without the game's layout files.)
function getDebugShortcut(id)
    return 0
end
