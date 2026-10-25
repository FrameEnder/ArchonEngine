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
-- The second boss's adds (BNpcBase 14449, all six with layout 9040086 on retail): one here, see dungeons.md.
BNPC_BOSS2_ADD = 9040086
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
-- yalms (19.4-22 in Bob's three runs), when the pile goes and the gigas comes at them.
AMBUSHES = {
    { bnpc = 4023421, rocks = 2790172, x = -74.4, y = -6.0, z = -165.4 },
    { bnpc = 4023560, rocks = 2790295, x = -54.6, y = -37.9, z = 97.9 },
}
AMBUSH_RANGE = 22

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

function setData(director, index, value)
    director:set_data(index, value)
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

function spawnBoss(director, index)
    local boss = BOSSES[index]
    director:spawn_boss(boss.bnpc, boss.wall, boss.line, boss.place)
    if index == 2 then
        director:spawn_bnpc(BNPC_BOSS2_ADD)
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
    if index == 2 then
        director:despawn_bnpc(BNPC_BOSS2_ADD)
    elseif index == 3 then
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
