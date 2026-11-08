-- The Stone Vigil
--
-- From Bob's Chronofoil captures of 2026-11-07: 1cefedf4 visit 0.1 (39 minutes with Duty Support, during In Pursuit of
-- the Past, quest 952, which unlocks it at its sequence 2; three wipes: the first boss once, the last boss twice),
-- 341808b0 visit 0.2 (16 minutes, a tank through the Duty Finder with three other players, two of whom fell early) and
-- f842581d visit 0.1 (17 minutes, a conjurer through the Duty Finder; the client crashed at the exit);
-- `questkit chronofoil --battle` and the visits' packets (docs/src/fork/dungeons.md). Numbers only; names and text are
-- the game's. Where the runs differ, both are given. Public guides agree on the bosses' moves (Sources in the doc).
--
-- Perched dragons breathe down the halls as the party comes up to them; three ice walls let out ice sprites the same
-- way. The first boss runs to one of three spots at its room's edge before each Swinge; the second sends whirlwinds down
-- four lanes with each Typhoon; the last flies up as its health falls, diving across its room, and calls down Sheets of
-- Ice on everyone. A door before the last room opens by hand, and its cutscene plays as the party walks in.
--
-- Retail's Duty Support members went to set spots for the bosses' big moves (Swinge, Cauterize, the whirlwinds, the
-- Sheets of Ice), by role, every time (`director:party_spots`, `director:party_positions`; dodge.rs): the spots below
-- are theirs.

-- Rooms: the bosses, their walls and lines (EObj IDs), PlaceNames (logs 2012-2014), and the floors the party keeps to
-- while dodging (MapRanges of layer LVD_Maprange_01 of planmap.lgb, Bob's dump of 2026-11-07: unturned boxes round each
-- room; the walls close them anyway).
BNPC_BOSS1 = 4000969
BOSS1_WALL = 2001873
BOSS1_LINE = 2001872
PLACE_BOSS1 = 877
BOSS1_ARENA = 4322871
BNPC_BOSS2 = 9061121
BOSS2_WALL = 2002872
BOSS2_LINE = 2002735
PLACE_BOSS2 = 878
BOSS2_ARENA = 4322974
BNPC_BOSS3 = 4001014
BOSS3_WALL = 2001877
BOSS3_LINE = 2001876
PLACE_BOSS3 = 879
BOSS3_ARENA = 4322979

-- The ways on: opened as the first and second bosses fall (state 7, both runs), and the door before the last room,
-- opened by hand (GimmickAccessor 0x000F00DB: EventAction 24 for 2 s, 1.9 s in both runs).
EOBJ_BOSS1_WAY_ON = 2001878
EOBJ_BOSS2_WAY_ON = 2001879
EOBJ_LAST_DOOR = 2001880
EVENT_ACTION_OPEN = 24
-- Where the shortcut (2000700, by the entrance) takes the party: the pop ranges outside each room (layer LVD_Zone_01
-- of planmap.lgb), the next as each boss is pulled.
SHORTCUT_BOSS1 = 11438278
SHORTCUT_BOSS2 = 11438279
SHORTCUT_BOSS3 = 11438280

BGM_BATTLE = 37

-- The event ranges the director watches, from planmap.lgb (Bob's dump of 2026-11-07): their box's middle, its turn
-- about the vertical (radians) and its half-sizes, as the layout has them.
EVENT_RANGES = {
    -- LVD_gmmick_wyvern: the perched dragons' (each the one breathing across it).
    [4240064] = { x = 31.96, y = 7.95, z = 241.84, rotation = 0.0, sx = 8.0, sy = 6.0, sz = 2.0 },
    [4240075] = { x = -23.9, y = 8.0, z = 167.87, rotation = 1.5705, sx = 8.0, sy = 6.0, sz = 2.0 },
    [4240078] = { x = -24.46, y = 0.0, z = 47.94, rotation = 1.5705, sx = 8.0, sy = 6.0, sz = 2.0 },
    [4240080] = { x = -32.86, y = -0.91, z = 23.74, rotation = 1.5705, sx = 6.0, sy = 6.0, sz = 2.0 },
    -- LVD_gmmick_ice: in front of each ice wall.
    [4240100] = { x = 24.0, y = -0.05, z = -4.38, rotation = 0.0, sx = 6.0, sy = 6.0, sz = 2.0 },
    [4240136] = { x = 24.78, y = -0.05, z = -112.05, rotation = 0.0, sx = 8.0, sy = 6.0, sz = 2.0 },
    [4240137] = { x = 23.97, y = 0.0, z = -159.15, rotation = 0.0, sx = 6.0, sy = 6.0, sz = 2.0 },
    -- LVD_gimmick: just past the last door, the last room's cutscene.
    [4239894] = { x = 0.0, y = 7.42, z = -226.05, rotation = 0.0, sx = 10.0, sy = 8.0, sz = 2.0 },
    -- LVD_Faith (Duty Support's): the second boss's way in, and the last room's floor before its boss.
    [9394127] = { x = 31.01, y = 4.0, z = -80.13, rotation = 0.0, sx = 3.0, sy = 5.0, sz = 7.69 },
    [9394128] = { x = 0.16, y = 0.07, z = -236.07, rotation = 0.0, sx = 26.91, sy = 5.0, sz = 6.35 },
}

-- The perched dragons (BNpcBase 1460, not hostile): as a player first comes into the range ahead of one, ActionTimeline
-- 7749, its 1021 (instant) 1.96 s later, ActionTimeline 7739 2.09 s after that (all four in the first run, each once).
DRAGONS = {
    { layout = 4240190, range = 4240064 },
    { layout = 4240192, range = 4240075 },
    { layout = 4240197, range = 4240078 },
    { layout = 4240201, range = 4240080 },
}
TIMELINE_DRAGON_RISES = 7749
TIMELINE_DRAGON_SETTLES = 7739
ACTION_BLAZING_TRAIL = 1021
BREATH_AFTER = 1.96
SETTLES_AFTER = 2.09

-- The ice walls (EObj IDs; layer LVD_gmmick_ice): as a player first comes into the range in front of one, its
-- ActorControl 106 with 0, and three ice sprites (1465) 0.2-0.4 s later at it, going for the party.
ICE = {
    { eobj = 2001881, range = 4240100, sprites = { 4005250, 4004499, 4005249 } },
    { eobj = 2001882, range = 4240136, sprites = { 4002766, 4002764, 4002759 } },
    { eobj = 2001883, range = 4240137, sprites = { 4240155, 4002769, 4002767 } },
}
ICE_BREAKS = 106
SPRITES_AFTER = 0.3

-- The first boss (1461): Swinge (903, 3.7 s, a 40-yalm cone) 14.4 s after the pull (14.6, 14.3, 14.2) and then about
-- every 31 s (32.4, 28.7; 33.4, 26.6, 36.6; 32.3); before each it runs (9 yalms a second: ActorMoves every 0.31 s,
-- 2.8 yalms apart, speed byte 90) to one of three spots at the room's edge, never the one it's at, arriving as it
-- begins (`sweep_bnpc`: one ActorMove there snapped it to the spot on the client, Bob's bundle 20261107-212704); below half its health it Swinges
-- again from there 6.4 s later (6.3-6.4), turned to its target (every double was at 45-49 % or below, every single at
-- 68 % or above). The first faces into the room: its target, if that's within 0.72 rad of the room's middle (0, 116),
-- else the nearer edge of that (all 13 first Swinges in the three runs were 0.02-0.72 off the middle; the second ones
-- 1.95-2.41 off, at the tank who had stepped round it). Facing its target wherever it stood turned it to the wall
-- (Bob's bundle 20261108-145830).
ACTION_SWINGE = 903
SWINGE_FIRST = 14.4
SWINGE_EVERY = 31.0
SWINGE_RUN = 2.4
SWINGE_RUN_SPEED = 9.0
SWINGE_ROOM_MIDDLE = { x = 0.0, z = 116.0 }
SWINGE_OFF_MIDDLE = 0.72
SWINGE_AGAIN_AFTER = 6.4
SWINGE_TWICE_BELOW = 50
SWINGE_CAST = 3.7
RUN_SPEED = 90
SWINGE_SPOTS = {
    { x = 0.0, y = 0.14, z = 130.0 },
    { x = 15.0, y = 0.05, z = 116.0 },
    { x = -15.03, y = 0.44, z = 115.98 },
}
-- Where retail's Duty Support members went as it Swinged from each spot (all ten casts: the same spots each time; the
-- tank's was in the second Swinge, aimed at it, so it stepped out of that by the cone).
SWINGE_PARTY = {
    { tank = { -5.0, 133.0 }, dps = { 5.0, 132.0 }, healer = { 6.0, 134.0 } },
    { tank = { 17.0, 111.0 }, dps = { 16.0, 121.0 }, healer = { 19.0, 120.0 } },
    { tank = { -17.0, 111.0 }, dps = { -19.0, 120.0 }, healer = { -18.0, 123.0 } },
}

-- The second boss (14535): Typhoon (28730, its timeline) brings whirlwinds (14536, layout 9061122) 1.5 s after the cast
-- ends, at x = 57 in lanes z = -72, -77.5, -82.5 and -88, each with a helper (14544, layout 9061123) riding with it
-- using 28731 every 0.58 s from when the first set off until it arrives. Two set off 3.9 s after they came, the others
-- 7.1 s after them, at 3.0 yalms a second down to x = 30.4 (8.9 s), and are gone 23.5 and 30.5 s after they came. The
-- first Typhoon's were three: -72 and -82.5, then -88 (the first two runs), or -77.5 and -88, then -82.5 (the
-- conjurer's); after it four, in pairs: -72 with -82.5 and -77.5 with -88 (either first), or -72 with -77.5 and -82.5
-- with -88 (either first): all four orders seen over the three runs.
NPC_WHIRLWIND = 9061122
NPC_WHIRLWIND_HELPER = 9061123
ACTION_TYPHOON = 28730
ACTION_WHIRLWIND = 28731
WHIRLWINDS_AFTER = 4.2
WHIRLWIND_FROM_X = 57.0
WHIRLWIND_TO_X = 30.4
WHIRLWIND_Y = 4.5
WHIRLWIND_TO_Y = 4.0
WHIRLWIND_SPEED = 3.0
WHIRLWIND_EVERY = 0.58
FIRST_PAIR_GOES = 3.9
SECOND_PAIR_GOES = 11.0
FIRST_PAIR_GONE = 23.5
SECOND_PAIR_GONE = 30.5
LANES = { -72.0, -77.5, -82.5, -88.0 }
FIRST_TYPHOONS = {
    { { -72.0, -82.5 }, { -88.0 } },
    { { -77.5, -88.0 }, { -82.5 } },
}
TYPHOONS = {
    { { -72.0, -82.5 }, { -77.5, -88.0 } },
    { { -77.5, -88.0 }, { -72.0, -82.5 } },
    { { -72.0, -77.5 }, { -82.5, -88.0 } },
    { { -82.5, -88.0 }, { -72.0, -77.5 } },
}
-- Retail's Duty Support members (all five Typhoons): waiting by the boss as the whirlwinds came, then into the middle
-- lane the first pair left alone (-77.5's or -82.5's), then into the other one, behind the first pair, as the second
-- set off.
TYPHOON_WAIT = { tank = { 47.0, -80.0 }, dps = { 41.0, -81.0 }, healer = { 40.0, -80.0 } }
TYPHOON_LANE = {
    [-77.5] = { tank = { 47.0, -77.0 }, dps = { 40.0, -77.0 }, healer = { 41.0, -76.0 } },
    [-82.5] = { tank = { 47.0, -83.0 }, dps = { 41.0, -84.0 }, healer = { 40.0, -83.0 } },
}

-- The last boss (1455), spawned 2.2 s after its room's cutscene began (2.1-2.4 s in Brayflox's), with eight helpers
-- (BNpcBase 9020, no layout ID on retail: the fork's 99009020). Retail's own cutscenes: 249 as the party walked in past
-- the door (params 1, 38, 1, 0), 300 at the clear (1, 18, 0, 0), both runs.
CUTSCENE_LAST_ROOM = 249
CUTSCENE_CLEAR = 300
CUTSCENE_CLEAR_PARAMS = { 1, 18, 0, 0 }
LAST_ROOM_RANGE = 4239894
BOSS3_COMES_AFTER = 2.2
HELPER = 99009020
HELPERS = 8

-- Its rounds on the ground (retail's clock, all four pulls): Rime Wreath (1025, on itself), Frost Breath (1022, at its
-- target) 9.2 s later, Sheet of Ice (1023, at one of the party) 16.4-17.5 s after the Rime Wreath, and 26.0 s after it
-- (30.1 in the round after the pull) whether to fly; the first round's Rime Wreath 8.1-8.2 s after the pull, the next
-- one's 4.1 s after it lands or decides not to fly.
ACTION_RIME_WREATH = 1025
ACTION_FROST_BREATH = 1022
ACTION_SHEET = 1023
ACTION_SHEETS = 1024
ACTION_CAUTERIZE = 1026
ACTION_TOUCHDOWN = 1027
FIRST_ROUND_AFTER = 8.1
FROST_BREATH_AFTER = 9.2
SHEET_AFTER = 16.8
DECIDE_AFTER = 26.0
FIRST_DECIDE_AFTER = 30.1
NEXT_ROUND_AFTER = 4.1
-- When it flies (its health, in percent, in the six pulls): the first time as soon as it's below 85 % while its round's
-- Frost Breath or Sheet of Ice is still to come, once it isn't casting (0 and 1.8 s after it fell below in the last two
-- runs), else at the round's end (2.8-7.1 s after, three pulls); once more between 50 and 70 % at a round's end (59.7-
-- 66.0 % three times; at 48-55 % it didn't, three times); then below 40 % at every round's end, with the Sheets of Ice
-- (34.6-35.7 %, 16.3 %, 12.3 %, 23.0 %). Falling below 50 % with a Frost Breath or Sheet of Ice to come, it begins a new
-- round instead at that move's time (the conjurer's run: 4.1 s later, as after a round's end; in the other pulls it
-- fell below 50 % after its round's last move).
FLY_FIRST_BELOW = 85
NEW_ROUND_BELOW = 50
-- How long after a move it doesn't take off (the longest of its casts, Rime Wreath's 2.7 s, and a moment.)
CASTING_FOR = 3.0
FLY_SECOND = { 50, 70 }
FLY_SHEETS_BELOW = 40
-- Its flights (from SetTargetable off and ActionTimeline 1385): over the spot of each Cauterize 4.1 s later (an
-- ActorSetPos there, ActionTimeline 142 0.1 s later, the cast 0.7 s after it), the next 7.8 s after; home (0, -248)
-- 7.8 s after the last, Touchdown (1027) as it lands, targetable 2.2 s later. With the Sheets of Ice, its helpers
-- cast 1024 (2.7 s) at each of the party 5.8 s in, and again where they stand 3.0 s later, and the dives come
-- 13.7 s in, two of them.
TIMELINE_FLY = 1385
TIMELINE_DIVE = 142
DIVE_AFTER = 4.1
SHEETS_DIVE_AFTER = 13.7
DIVE_CAST_AFTER = 0.7
DIVE_EVERY = 7.8
LANDS_TARGETABLE_AFTER = 2.2
SHEETS_AFTER = 5.8
SHEETS_AGAIN_AFTER = 3.0
HOME = { x = 0.0, y = 0.0, z = -248.0, rotation = 3.14159 }
-- Its dives' spots and facings (all of them over the five pulls: the same five) and where retail's Duty Support members
-- went for each (the tank's 0, -253 its spot by the boss; they were the same every time).
CAUTERIZE = {
    { x = 23.73, y = 0.0, z = -262.5, rotation = -1.5708, party = { tank = { 0.0, -251.0 }, dps = { 4.0, -247.0 }, healer = { 0.0, -246.0 } } },
    { x = 23.73, y = 0.0, z = -233.51, rotation = -1.5708, party = { tank = { 0.0, -251.0 }, dps = { 2.0, -247.0 }, healer = { 0.0, -246.0 } } },
    { x = 0.0, y = 0.0, z = -271.78, rotation = 0.0, party = { tank = { -11.0, -253.0 }, dps = { 11.0, -245.0 }, healer = { 11.0, -242.0 } } },
    { x = 14.48, y = 0.0, z = -271.78, rotation = 0.0, party = { tank = { 0.0, -253.0 }, dps = { 2.0, -247.0 }, healer = { 2.0, -243.0 } } },
    { x = -14.51, y = 0.0, z = -271.78, rotation = 0.0, party = { tank = { 0.0, -253.0 }, dps = { 4.0, -247.0 }, healer = { 2.0, -243.0 } } },
}
-- Before the Sheets of Ice (all three times, Duty Support): spread out from 1.5 s into the flight, then a step aside
-- as the first ones were cast (the second were cast where they then stood, which they left by the circles).
SHEETS_SPREAD_AFTER = 1.5
SHEETS_SPREAD = { tank = { 0.0, -267.0 }, dps = { 17.75, -253.44 }, healer = { 0.0, -229.0 } }
SHEETS_ASIDE = { tank = { -7.0, -267.0 }, dps = { 19.0, -263.0 }, healer = { 7.0, -229.0 } }

-- What the coffers held on retail, one list per opening seen (item IDs; the Duty Support run's, then the tank's and the
-- conjurer's where its coffer is known). 179, 180 and 182 come as the first boss falls, 181, 183 and 184 as the second does, 185 at the
-- clear.
LOOT = {
    [179] = { { 7033 } },
    [180] = { { 31530 } },
    [182] = { { 10500 } },
    [181] = { { 31533 } },
    [183] = { { 31518 } },
    [184] = { { 31525 } },
    [185] = { { 10438, 7033 }, { 7033, 2176 }, { 7033, 31528, 1927 } },
}
COFFERS_BOSS1 = { 179, 182, 180 }
COFFERS_BOSS2 = { 181, 184, 183 }
COFFER_CLEAR = 185

-- The Duty Support members' story lines (NpcYell, `director:party_npc_yell(role, line, delay)`), when retail's said
-- them: the Thaumaturge 4.3 s after the duty began, the tank as the first boss fell, the Conjurer as the party came up
-- to the second boss's room (3.0 s before its pull), the Thaumaturge as it fell, the tank as the party walked into the
-- last room after its cutscene (7.2 s after it ended).
LINE_START = 14368
LINE_START_AFTER = 4.3
LINE_BOSS1 = 14371
LINE_BOSS2_ROOM = 14374
LINE_BOSS2 = 14375
LINE_BOSS3_ROOM = 14376

-- onUpdate comes every logic tick (500 ms): the script's own clock.
TICK = 0.5

clock = 0
later = {}
boss1_pulled = false
boss1_pulled_at = nil
boss1_dead = false
swinges = 0
swinge_spot = nil
boss2_pulled = false
boss2_dead = false
typhoons = 0
whirlwinds = {}
whirlwind_helpers = {}
boss2_line = false
door_open = false
boss3_started = false
boss3_line = false
boss3_pulled = false
boss3_dead = false
boss3_helpers = {}
-- The last boss's state: its next move (`step`), whether it's flying, and which flights it has made.
isgebind = nil
dragons_breathed = {}
ice_broken = {}

function after(seconds, fn)
    table.insert(later, { at = clock + seconds, fn = fn })
end

function pulled(info, id)
    return info ~= nil and info[id] ~= nil and info[id].target ~= nil
end

-- Every player in the instance ("player1", "player2"...; "player" when there's only the one key.)
function players(info)
    local list = {}
    if info == nil then
        return list
    end
    local index = 1
    while info["player" .. index] ~= nil do
        table.insert(list, info["player" .. index])
        index = index + 1
    end
    if #list == 0 and info.player ~= nil then
        table.insert(list, info.player)
    end
    return list
end

-- The players and the party's NPCs (Duty Support members, bots), living.
function party(info)
    local list = players(info)
    if info ~= nil then
        for index = 1, 8 do
            if info["party" .. index] ~= nil then
                table.insert(list, info["party" .. index])
            end
        end
    end
    return list
end

-- Whether a position is inside one of EVENT_RANGES' boxes (turned into the box's own frame.)
function inRange(position, id)
    local range = EVENT_RANGES[id]
    local dx, dy, dz = position.x - range.x, position.y - range.y, position.z - range.z
    local c, s = math.cos(range.rotation), math.sin(range.rotation)
    local lx, lz = dx * c - dz * s, dx * s + dz * c
    return math.abs(lx) <= range.sx and math.abs(dy) <= range.sy and math.abs(lz) <= range.sz
end

function anyoneIn(list, id)
    for _, who in ipairs(list) do
        if inRange(who, id) then
            return true
        end
    end
    return false
end

-- Where an actor (by actor ID) stands, among the party and the NPCs onUpdate reports.
function positionOf(info, actor_id)
    if info == nil or actor_id == nil then
        return nil
    end
    for _, entry in pairs(info) do
        if type(entry) == "table" and entry.id == actor_id then
            return entry
        end
    end
    return nil
end

-- The way from one spot to another, as the game turns (0 is +z.)
function facing(from, to)
    return math.atan(to.x - from.x, to.z - from.z)
end

function onSetup(director)
    director:set_sequence(1)

    director:boss_room(BNPC_BOSS1, BOSS1_WALL, BOSS1_LINE, PLACE_BOSS1, {}, BOSS1_ARENA)
    director:boss_room(BNPC_BOSS2, BOSS2_WALL, BOSS2_LINE, PLACE_BOSS2, {}, BOSS2_ARENA)
    -- The walls stand open until a pull; the ways on and the rooms' lines stand closed.
    for _, wall in ipairs({ BOSS1_WALL, BOSS2_WALL, BOSS3_WALL }) do
        director:hide_eobj(wall)
    end
    for _, door in ipairs({ EOBJ_BOSS1_WAY_ON, EOBJ_BOSS2_WAY_ON, EOBJ_LAST_DOOR }) do
        director:show_eobj(door)
    end
    for _, line in ipairs({ BOSS1_LINE, BOSS2_LINE, BOSS3_LINE }) do
        director:show_eobj(line)
    end

    for coffer, openings in pairs(LOOT) do
        director:treasure_loot(coffer, openings)
    end

    -- Retail's Duty Support members' spots for the bosses' big casts.
    for index, spot in ipairs(SWINGE_SPOTS) do
        director:party_spots(ACTION_SWINGE, spot.x, spot.z, SWINGE_PARTY[index])
    end
    for _, dive in ipairs(CAUTERIZE) do
        director:party_spots(ACTION_CAUTERIZE, dive.x, dive.z, dive.party)
    end

end

function onCommence(director)
    director:party_npc_yell("ranged", LINE_START, LINE_START_AFTER)
end

function onUpdate(director, health, player_health, info)
    clock = clock + TICK
    local due = {}
    local rest = {}
    for _, task in ipairs(later) do
        if task.at <= clock then
            table.insert(due, task)
        else
            table.insert(rest, task)
        end
    end
    later = rest
    for _, task in ipairs(due) do
        task.fn(director, health, info)
    end

    local everyone = players(info)
    local all = party(info)

    for _, dragon in ipairs(DRAGONS) do
        if not dragons_breathed[dragon.layout] and anyoneIn(everyone, dragon.range) then
            dragons_breathed[dragon.layout] = true
            breathe(director, info, dragon.layout)
        end
    end
    for index, ice in ipairs(ICE) do
        if not ice_broken[index] and anyoneIn(everyone, ice.range) then
            ice_broken[index] = true
            breakIce(director, ice)
        end
    end

    -- The first boss.
    if not boss1_pulled and pulled(info, BNPC_BOSS1) then
        boss1_pulled = true
        boss1_pulled_at = clock
        swinges = 0
        director:set_bgm(BGM_BATTLE)
        director:update_shortcut(SHORTCUT_BOSS1)
    end
    if boss1_pulled and not boss1_dead and info[BNPC_BOSS1] ~= nil
        and clock - boss1_pulled_at >= SWINGE_FIRST - SWINGE_RUN + swinges * SWINGE_EVERY then
        swinges = swinges + 1
        swinge(director, health, info)
    end

    -- The second boss.
    if not boss2_line and anyoneIn(all, 9394127) then
        boss2_line = true
        director:party_npc_yell("healer", LINE_BOSS2_ROOM)
    end
    if not boss2_pulled and pulled(info, BNPC_BOSS2) then
        boss2_pulled = true
        director:set_bgm(BGM_BATTLE)
        director:update_shortcut(SHORTCUT_BOSS2)
    end

    -- The last room.
    if door_open and not boss3_started and anyoneIn(everyone, LAST_ROOM_RANGE) then
        startLastRoom(director)
    end
    if boss3_started and not boss3_line and anyoneIn(all, 9394128) then
        boss3_line = true
        director:party_npc_yell("tank", LINE_BOSS3_ROOM)
    end
    if boss3_started and not boss3_pulled and not boss3_dead and pulled(info, BNPC_BOSS3) then
        boss3_pulled = true
        director:update_shortcut(SHORTCUT_BOSS3)
        isgebind = {
            flights = 0,
            second = false,
            flying = false,
            next = clock + FIRST_ROUND_AFTER,
            step = 1,
            first = true,
            new_round = false,
            moved_at = -CASTING_FOR,
        }
    end
    if isgebind ~= nil and not boss3_dead and not isgebind.flying then
        local boss = info ~= nil and info[BNPC_BOSS3] or nil
        local hp = health ~= nil and health[BNPC_BOSS3] or 100
        -- Below 85 % with its Frost Breath or Sheet of Ice to come: up at once, once it isn't casting.
        if boss ~= nil and isgebind.flights == 0 and hp < FLY_FIRST_BELOW and (isgebind.step == 2 or isgebind.step == 3)
            and clock - isgebind.moved_at >= CASTING_FOR then
            fly(director, false, boss.id)
        elseif clock >= isgebind.next then
            isgebindStep(director, health, info)
        end
    end
end

-- A perched dragon breathes down the hall.
function breathe(director, info, layout)
    local dragon = info[layout]
    if dragon == nil then
        return
    end
    director:play_action_timeline(layout, TIMELINE_DRAGON_RISES)
    director:delay(BREATH_AFTER)
    director:actor_use_action(dragon.id, ACTION_BLAZING_TRAIL)
    director:delay(BREATH_AFTER + SETTLES_AFTER)
    director:play_action_timeline(layout, TIMELINE_DRAGON_SETTLES)
    director:delay(0)
end

function breakIce(director, ice)
    director:eobj_control(ice.eobj, ICE_BREAKS, { 0 })
    director:delay(SPRITES_AFTER)
    for _, sprite in ipairs(ice.sprites) do
        director:spawn_bnpc(sprite)
        director:set_hate(sprite, nil, 300) -- the player
    end
    director:delay(0)
end

-- The first boss runs to a spot at its room's edge (not the one it's at) and Swinges from there, turned to its
-- target; below half its health, again 6.4 s later.
function swinge(director, health, info)
    local boss = info[BNPC_BOSS1]
    local choices = {}
    for index = 1, #SWINGE_SPOTS do
        if index ~= swinge_spot then
            table.insert(choices, index)
        end
    end
    swinge_spot = choices[math.random(#choices)]
    local spot = SWINGE_SPOTS[swinge_spot]
    local twice = health ~= nil and health[BNPC_BOSS1] ~= nil and health[BNPC_BOSS1] < SWINGE_TWICE_BELOW
    local run = math.max(TICK, math.sqrt((spot.x - boss.x) ^ 2 + (spot.z - boss.z) ^ 2) / SWINGE_RUN_SPEED)
    director:busy_bnpc(BNPC_BOSS1, run + SWINGE_CAST + (twice and SWINGE_AGAIN_AFTER or 0))
    director:sweep_bnpc(BNPC_BOSS1, spot.x, spot.y, spot.z, SWINGE_RUN_SPEED)
    -- There, it turns to its target and Swinges.
    after(run, function(d, _, now_info)
        local now_boss = now_info ~= nil and now_info[BNPC_BOSS1] or nil
        if now_boss == nil then
            return
        end
        local target = positionOf(now_info, now_boss.target_id)
        d:move_bnpc(BNPC_BOSS1, spot.x, spot.y, spot.z, intoTheRoom(spot, target), RUN_SPEED)
        d:actor_use_action(now_boss.id, ACTION_SWINGE)
    end)
    if twice then
        after(run + SWINGE_AGAIN_AFTER, function(d, _, now_info)
            local now_boss = now_info ~= nil and now_info[BNPC_BOSS1] or nil
            if now_boss == nil then
                return
            end
            local now_target = positionOf(now_info, now_boss.target_id)
            if now_target ~= nil then
                d:move_bnpc(BNPC_BOSS1, spot.x, spot.y, spot.z, facing(spot, now_target), RUN_SPEED)
            end
            d:actor_use_action(now_boss.id, ACTION_SWINGE)
        end)
    end
end

-- The first Swinge's facing from `spot`: at its target, kept within SWINGE_OFF_MIDDLE of the room's middle.
function intoTheRoom(spot, target)
    local middle = facing(spot, SWINGE_ROOM_MIDDLE)
    if target == nil then
        return middle
    end
    local off = facing(spot, target) - middle
    off = (off + math.pi) % (2 * math.pi) - math.pi
    off = math.max(-SWINGE_OFF_MIDDLE, math.min(SWINGE_OFF_MIDDLE, off))
    return middle + off
end

-- The bosses' actions the director answers.
function onActorCast(director, layout_id, action_id, actor_id, x, y, z)
    if layout_id == BNPC_BOSS2 and action_id == ACTION_TYPHOON then
        -- Its helpers, one for each lane's whirlwind (held: they act as the director says; retail's stood by the boss
        -- from the start, at (50.1, 4.0, -79.3).)
        if #whirlwind_helpers == 0 then
            for _ = 1, #LANES do
                local helper = director:spawn_bnpc(NPC_WHIRLWIND_HELPER, 50.1, 4.0, -79.3, 0.0)
                director:hold_bnpc(helper)
                table.insert(whirlwind_helpers, helper)
            end
        end
        typhoons = typhoons + 1
        local pairs_now = typhoons == 1 and FIRST_TYPHOONS[math.random(#FIRST_TYPHOONS)]
            or TYPHOONS[math.random(#TYPHOONS)]
        after(WHIRLWINDS_AFTER, function(d)
            if not boss2_dead then
                typhoon(d, pairs_now)
            end
        end)
    end
end

-- Which lane's helper rides with a whirlwind.
function laneIndex(lane)
    for index, z in ipairs(LANES) do
        if z == lane then
            return index
        end
    end
    return 1
end

function contains(list, value)
    for _, item in ipairs(list) do
        if item == value then
            return true
        end
    end
    return false
end

-- The whirlwinds come, wait and go down their lanes in two pairs; the party waits by the boss, then goes into the
-- middle lane the first pair leaves alone, then into the other behind the first pair.
function typhoon(director, lanes)
    director:party_positions(TYPHOON_WAIT, FIRST_PAIR_GOES)
    for pair, set in ipairs(lanes) do
        local goes = pair == 1 and FIRST_PAIR_GOES or SECOND_PAIR_GOES
        local gone = pair == 1 and FIRST_PAIR_GONE or SECOND_PAIR_GONE
        for _, lane in ipairs(set) do
            local whirlwind = director:spawn_bnpc(NPC_WHIRLWIND, WHIRLWIND_FROM_X, WHIRLWIND_Y, lane, -1.5708)
            director:hold_bnpc(whirlwind)
            director:set_targetable(whirlwind, false)
            table.insert(whirlwinds, whirlwind)
            director:sweep_bnpc(whirlwind, WHIRLWIND_TO_X, WHIRLWIND_TO_Y, lane, WHIRLWIND_SPEED, goes,
                whirlwind_helpers[laneIndex(lane)], ACTION_WHIRLWIND, WHIRLWIND_EVERY, FIRST_PAIR_GOES)
            after(gone, function(d)
                d:despawn_bnpc(whirlwind)
            end)
        end
    end
    local first_free = contains(lanes[1], -77.5) and -82.5 or -77.5
    local second_free = first_free == -77.5 and -82.5 or -77.5
    director:delay(FIRST_PAIR_GOES)
    director:party_positions(TYPHOON_LANE[first_free], SECOND_PAIR_GOES - FIRST_PAIR_GOES)
    director:delay(SECOND_PAIR_GOES)
    -- Out of the lane the second set goes down, if they're in it, into the other, behind the first set (the first
    -- Typhoon's second "set" is one lane: an outer one leaves them where they are.)
    local stay = contains(lanes[2], first_free) and second_free or first_free
    director:party_positions(TYPHOON_LANE[stay], FIRST_PAIR_GONE - SECOND_PAIR_GOES)
    director:delay(0)
end

-- The last boss's next move on the ground, or its flight.
function isgebindStep(director, health, info)
    local boss = info[BNPC_BOSS3]
    if boss == nil then
        return
    end
    local hp = health ~= nil and health[BNPC_BOSS3] or 100
    local state = isgebind
    -- Below 85 % for the first time, at any of its moves: it flies instead.
    if state.flights == 0 and hp < FLY_FIRST_BELOW then
        fly(director, false, boss.id)
        return
    end
    -- Below 50 % for the first time with a move of its round to come: a new round instead.
    if not state.new_round and hp < NEW_ROUND_BELOW and (state.step == 2 or state.step == 3) then
        state.new_round = true
        state.step = 1
        state.next = clock + NEXT_ROUND_AFTER
        return
    end
    if state.step ~= 4 then
        state.moved_at = clock
    end
    if state.step == 1 then
        director:actor_use_action(boss.id, ACTION_RIME_WREATH)
        state.round_at = clock
        state.step = 2
        state.next = state.round_at + FROST_BREATH_AFTER
    elseif state.step == 2 then
        local target = positionOf(info, boss.target_id)
        director:actor_use_action(boss.id, ACTION_FROST_BREATH, target ~= nil and target.id or nil)
        state.step = 3
        state.next = state.round_at + SHEET_AFTER
    elseif state.step == 3 then
        local all = party(info)
        local victim = all[math.random(math.max(#all, 1))]
        director:actor_use_action(boss.id, ACTION_SHEET, victim ~= nil and victim.id or nil)
        state.step = 4
        state.next = state.round_at + (state.first and FIRST_DECIDE_AFTER or DECIDE_AFTER)
    else
        state.first = false
        if hp < NEW_ROUND_BELOW then
            state.new_round = true
        end
        if hp < FLY_SHEETS_BELOW then
            fly(director, true, boss.id)
        elseif not state.second and state.flights >= 1 and hp >= FLY_SECOND[1] and hp < FLY_SECOND[2] then
            state.second = true
            fly(director, false, boss.id)
        else
            state.step = 1
            state.next = clock + NEXT_ROUND_AFTER
        end
    end
end

-- Up it goes: untargetable, over its dives' spots (two with the Sheets of Ice, which come first), home again.
function fly(director, sheets, boss_id)
    local state = isgebind
    state.flights = state.flights + 1
    state.flying = true
    state.first = false
    local first = math.random(#CAUTERIZE)
    local dives = { first }
    if sheets then
        local second = first
        while second == first do
            second = math.random(#CAUTERIZE)
        end
        table.insert(dives, second)
    end
    local dive_at = sheets and SHEETS_DIVE_AFTER or DIVE_AFTER
    local home_at = dive_at + (#dives - 1) * DIVE_EVERY + DIVE_EVERY
    local landed_at = home_at + LANDS_TARGETABLE_AFTER

    director:set_targetable(BNPC_BOSS3, false)
    director:busy_bnpc(BNPC_BOSS3, landed_at)
    director:play_action_timeline(BNPC_BOSS3, TIMELINE_FLY)
    if sheets then
        director:delay(SHEETS_SPREAD_AFTER)
        director:party_positions(SHEETS_SPREAD, SHEETS_AFTER - SHEETS_SPREAD_AFTER)
        director:delay(0)
        after(SHEETS_AFTER, function(d, _, now_info)
            d:party_positions(SHEETS_ASIDE, SHEETS_AGAIN_AFTER)
            sheetsOfIce(d, now_info, 0)
        end)
        after(SHEETS_AFTER + SHEETS_AGAIN_AFTER, function(d, _, now_info)
            sheetsOfIce(d, now_info, 4)
        end)
    end
    for index, dive in ipairs(dives) do
        local spot = CAUTERIZE[dive]
        local at = dive_at + (index - 1) * DIVE_EVERY
        director:delay(at)
        director:warp_bnpc(BNPC_BOSS3, spot.x, spot.y, spot.z, spot.rotation)
        director:delay(at + 0.1)
        director:play_action_timeline(BNPC_BOSS3, TIMELINE_DIVE)
        director:delay(0)
        after(at + DIVE_CAST_AFTER, function(d, _, now_info)
            local boss = now_info ~= nil and now_info[BNPC_BOSS3] or nil
            if boss ~= nil then
                d:actor_use_action(boss.id, ACTION_CAUTERIZE)
            end
        end)
    end
    director:delay(home_at)
    director:warp_bnpc(BNPC_BOSS3, HOME.x, HOME.y, HOME.z, HOME.rotation)
    director:delay(home_at + 0.1)
    director:actor_use_action(boss_id, ACTION_TOUCHDOWN)
    director:delay(landed_at)
    director:set_targetable(BNPC_BOSS3, true)
    director:delay(0)
    after(landed_at, function()
        if isgebind ~= nil then
            isgebind.flying = false
            isgebind.step = 1
            isgebind.next = clock + NEXT_ROUND_AFTER
        end
    end)
end

-- Four helpers each cast a Sheet of Ice at one of the party, where they stand (the first four helpers, then the others.)
function sheetsOfIce(director, info, from)
    local all = party(info)
    for index, who in ipairs(all) do
        local helper = boss3_helpers[from + index]
        if helper ~= nil and index <= 4 then
            director:actor_use_action(helper, ACTION_SHEETS, who.id)
        end
    end
end

function startLastRoom(director)
    boss3_started = true
    director:update_shortcut(SHORTCUT_BOSS3)
    director:play_cutscene(CUTSCENE_LAST_ROOM)
    director:delay(BOSS3_COMES_AFTER)
    director:spawn_boss(BNPC_BOSS3, BOSS3_WALL, BOSS3_LINE, PLACE_BOSS3, BOSS3_ARENA)
    director:delay(0)
    after(BOSS3_COMES_AFTER, function(d)
        bringHelpers(d)
    end)
end

function bringHelpers(director)
    for _, helper in ipairs(boss3_helpers) do
        director:despawn_bnpc(helper)
    end
    boss3_helpers = {}
    for _ = 1, HELPERS do
        local helper = director:spawn_bnpc(HELPER, HOME.x, HOME.y + 0.1, HOME.z, 0.0)
        director:hold_bnpc(helper)
        table.insert(boss3_helpers, helper)
    end
end

function onActorDeath(director, bnpc_id, position)
    if bnpc_id == BNPC_BOSS1 then
        boss1_dead = true
        director:set_bgm(0)
        director:hide_eobj(EOBJ_BOSS1_WAY_ON)
        director:objective(0, 1)
        for _, coffer in ipairs(COFFERS_BOSS1) do
            director:spawn_treasure(coffer)
        end
        director:party_npc_yell("tank", LINE_BOSS1)
    elseif bnpc_id == BNPC_BOSS2 then
        boss2_dead = true
        for _, whirlwind in ipairs(whirlwinds) do
            director:despawn_bnpc(whirlwind)
        end
        whirlwinds = {}
        for _, helper in ipairs(whirlwind_helpers) do
            director:despawn_bnpc(helper)
        end
        whirlwind_helpers = {}
        director:party_positions(nil)
        director:set_bgm(0)
        director:hide_eobj(EOBJ_BOSS2_WAY_ON)
        director:objective(1, 2)
        for _, coffer in ipairs(COFFERS_BOSS2) do
            director:spawn_treasure(coffer)
        end
        director:party_npc_yell("ranged", LINE_BOSS2)
    elseif bnpc_id == BNPC_BOSS3 then
        boss3_dead = true
        isgebind = nil
        director:party_positions(nil)
        for _, helper in ipairs(boss3_helpers) do
            director:despawn_bnpc(helper)
        end
        boss3_helpers = {}
        director:objective(2)
        director:spawn_treasure(COFFER_CLEAR)
        director:play_cutscene(CUTSCENE_CLEAR, CUTSCENE_CLEAR_PARAMS)
        director:complete_duty()
    end
end

-- Everyone back at the start: the fight's own state goes (the bosses reset as ever); the last boss comes back to its
-- room at once if the party had reached it (retail: as they were sent back, 1.9 s, both times).
function onPartyWipe(director)
    later = {}
    swinges = 0
    boss1_pulled = boss1_dead
    boss2_pulled = boss2_dead
    for _, whirlwind in ipairs(whirlwinds) do
        director:despawn_bnpc(whirlwind)
    end
    whirlwinds = {}
    director:party_positions(nil)
    if boss3_started and not boss3_dead then
        director:despawn_bnpc(BNPC_BOSS3)
        boss3_pulled = false
        isgebind = nil
        director:spawn_boss(BNPC_BOSS3, BOSS3_WALL, BOSS3_LINE, PLACE_BOSS3, BOSS3_ARENA)
        bringHelpers(director)
    end
end

function onGimmickAccessor(director, actor_id, id, params)
    if id == EOBJ_LAST_DOOR and not door_open then
        director:event_action(EVENT_ACTION_OPEN, actor_id, id)
        return
    end
    director:finish_gimmick(actor_id)
end

function isGimmickFunctional(id)
    if id == EOBJ_LAST_DOOR then
        return boss2_dead
    end
    return true
end

function onEventActionCast(director, actor_id, target)
    director:finish_gimmick(actor_id)
    if target == EOBJ_LAST_DOOR and boss2_dead and not door_open then
        door_open = true
        director:hide_eobj(EOBJ_LAST_DOOR)
    end
end

function getDebugShortcut(id)
    if id == 0 then
        return SHORTCUT_BOSS1
    elseif id == 1 then
        return SHORTCUT_BOSS2
    elseif id == 2 then
        return SHORTCUT_BOSS3
    end
    return 0
end
