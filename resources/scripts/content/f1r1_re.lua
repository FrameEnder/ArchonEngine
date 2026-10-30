-- The Thousand Maws of Toto-Rak
--
-- From Bob's Chronofoil captures of 2026-10-30: one run with Duty Support (caa5e908, visit 0.15, 27 minutes, during
-- Into the Beast's Maw) and one Duty Finder run with three other players (1fac2416, visit 0.15, 10.6 minutes);
-- `questkit chronofoil --battle` and the visits' packets (docs/src/fork/dungeons.md). Numbers only; names and text are
-- the game's.
--
-- Webs close the way: each burns away (a map effect) as the last of the pack guarding it falls, and two of them open
-- a room whose boss (BNpcBase 270) waits in it untargetable until then. Crossing into a room is one of the duty's
-- objectives; its boss's pull seals it (15 s), its death opens the way on and leaves coffers. The second room's boss
-- calls two adds at 50 %. Past the last web, crossing into the last room plays its cutscene and brings the last boss
-- (259), who calls two adds (260) at 75 and 30 %, sheds its tail (266, an add) at 55 % and drops eggs (572) that burst
-- where its 698 was aimed.

-- The webs: the map effect each is (shown burning away with timeline 2) and the pack whose last death burns it; two
-- leave a room's boss targetable (retail: ActorControl 54 with the map effect.) Retail, both runs: each map effect as
-- the pack's last member's health reached 0, whoever it was.
WEBS = {
    { index = 0, pack = { 9074838, 9074839, 9074840 } },
    { index = 1, pack = { 9074846, 9074847, 9074848, 9074849 }, boss = 3642541 },
    { index = 2, pack = { 9074872, 9074873, 9074874, 9074875 } },
    { index = 3, pack = { 9074881, 9074882, 9074883 }, boss = 3642543 },
    -- Coffer 25 came 3.7 and 4.0 s after this one burned.
    { index = 4, pack = { 9074888, 9074889, 9074890 }, coffer = 25, coffer_after = 3.8 },
    { index = 5, pack = { 9074898, 9074899, 9074900 } },
}
MAP_EFFECT_BURNT = 2

-- The rooms: their boss (and where it stands), the arena's wall and line (layout IDs: the walls and lines share their
-- EObj IDs, 2002872 and 2002735), the barrier on the far side that goes with the boss, the place name, and the line's
-- spot. Retail counted the room entered (its objective done) as the first player reached it, about the line: the
-- first position that did was 23.3 and 23.7 yalms from the first boss (one at 23.7 didn't), 22.3 and 22.1 from the
-- second (24.0 didn't), 41.7 and 41.0 from the last (43.0 and 43.6 didn't); the lines stand 23.4, 21.8 and 40.8 from
-- them. So here it's a player coming within `reach` of the boss. The objectives (InstanceContent's six,
-- ObjectiveStart 33300): entering a room is one, its boss another.
ROOMS = {
    {
        boss = 3642541, boss_at = { x = -104.66, y = -4.16, z = 111.89 },
        wall = 9089607, line = 9089609, barrier = 9087967, place = 523,
        line_at = { x = -128.09, y = -4.13, z = 112.1 }, reach = 23.5,
        web = 1, entered = 0, cleared = 1,
        coffers = { 22, 21 },
    },
    {
        boss = 3642543, boss_at = { x = -89.95, y = -8.17, z = -49.84 },
        wall = 9089610, line = 9089612, barrier = 9087976, place = 522,
        line_at = { x = -68.11, y = -8.02, z = -47.87 }, reach = 22.5,
        web = 3, entered = 2, cleared = 3,
        coffers = { 23 },
        -- Its two adds (BNpcBase 271, both layout 3656529) as it falls to 50 % (49.8 and 45.7 % in the two runs, the
        -- same two spots and facing.)
        adds_at = 50,
        adds = {
            { x = -81.93, y = -8.17, z = -56.37, rotation = -1.567 },
            { x = -72.86, y = -8.17, z = -42.17, rotation = -1.567 },
        },
    },
}
BNPC_ROOM_ADD = 3656529

FINAL = {
    boss = 3446839, boss_at = { x = 231.99, y = -39.29, z = -144.3 },
    wall = 9089456, line = 9089470, place = 612,
    line_at = { x = 191.24, y = -39.53, z = -143.99 }, reach = 42.0,
    entered = 4, cleared = 5,
}
-- Retail's cutscenes: the last room's (95, with the default 1, 38, 1, 0) and the last boss's death (173.) The boss came
-- 2.5 and 2.8 s after the first began.
CUTSCENE_FINAL = 95
CUTSCENE_END = 173
CUTSCENE_END_PARAMS = { 1, 18, 0, 0 }
FINAL_BOSS_AFTER = 2.5
-- The last room's weather: 19 (forced, faded in over 20 s) from its cutscene until the boss's death, then 2 again
-- over 4 s (retail's WeatherId, both runs.)
WEATHER_FINAL = 19
WEATHER_FINAL_FADE = 20.0
WEATHER_AFTER = 2
WEATHER_AFTER_FADE = 4.0

-- The last boss's adds (BNpcBase 260, all layout 3862492): two as it falls to 75 % (74.7 and 73.9 % in the two runs)
-- and two at 30 % (29.5, 28.6 %), at the same spots every time.
BNPC_FINAL_ADD = 3862492
FINAL_ADDS = {
    {
        health = 75,
        spots = {
            { x = 230.27, y = -39.02, z = -156.42, rotation = 0 },
            { x = 235.42, y = -39.7, z = -133.61, rotation = 0 },
        },
    },
    {
        health = 30,
        spots = {
            { x = 234.98, y = -39.29, z = -137.12, rotation = 0 },
            { x = 219.52, y = -39.29, z = -131.16, rotation = 0 },
        },
    },
}

-- Its tail (BNpcBase 266, an add with no layout ID on retail: here the fork's 99000266.) As the boss falls to 55 %
-- (54.2 and 47.1 % as it came: retail waited for what it was doing) it uses 704 on itself, then 702 2.7 s later; as
-- 702 begins the tail comes off where it stands (its facing), the boss gets ActorControl 31 (0, 0, 0, 1) and 30 (0, 1,
-- the tail's actor ID, 0), and log 2016 comes. While the tail stands the boss uses 702 again every 30.8 s (the Duty
-- Support run's second, 30.9 s later.) The tail's death: ActorControl 20 (64), 30 (0, 2, the tail, 1) and 31 (0, 0, 0,
-- 0) on the boss. Not here: 4.0 s into each 702 retail spawned an event object near the boss (EObj 2000404, the
-- director's, not in the layout, hidden as the tail died and deleted 3.4 s later), which Kawari can't spawn.
BNPC_TAIL = 99000266
TAIL_AT = 55
ACTION_TAIL_FIRST = 704
ACTION_TAIL = 702
TAIL_AFTER_FIRST = 2.7
TAIL_AGAIN_EVERY = 30.8
LOG_TAIL = 2016

-- Its eggs (BNpcBase 572, all layout 3656521): each lands where the boss's 698 was aimed, 4.7 s after the cast began
-- (both runs, every egg); 5.0 s later it casts 730 round itself (2.7 s) and it dies 2.1 s after that.
BNPC_EGG = 3656521
ACTION_EGG_THROWN = 698
ACTION_EGG = 730
EGG_LANDS_AFTER = 4.7
EGG_BURSTS_AFTER = 5.0
EGG_DIES_AFTER = 4.8

BGM_BATTLE = 37

-- Coffers (retail's, one list per opening; `server/loot.rs`): 22 and 21 as the first room's boss falls, 23 the
-- second's, 27 the last boss's, 25 after the fourth web; 19 and 29 stand from the start. 21 was opened in one run.
LOOT = {
    [22] = { { 7035 }, { 31446 } },
    [21] = { { 2749 } },
    [19] = { { 3584 }, { 3584 } },
    [29] = { { 3093 }, { 3095 } },
    [23] = { { 3093 }, { 3095 } },
    [25] = { { 2159 }, { 1696 } },
    [27] = { { 38470 }, { 31443, 1907 } },
}
STANDING_COFFERS = { 19, 29 }
FINAL_COFFER = 27

-- Event objects retail spawned hidden that nothing showed (2001005, 2001152, by the last room.)
HIDDEN_EOBJS = { 2001005, 2001152 }

-- The Duty Support members' story lines (NpcYell, Bob's capture caa5e908): the tank's 14331 18.1 s in, the
-- Conjurer's 14336 as the first room was entered, the tank's 14337 10.6 s after the last room's cutscene began.
PARTY_NPC_LINE_START = 14331
PARTY_NPC_LINE_START_AFTER = 18.1
PARTY_NPC_LINE_ROOM1 = 14336
PARTY_NPC_LINE_FINAL = 14337
PARTY_NPC_LINE_FINAL_AFTER = 10.6

-- How far the party has got
webs_burnt = {}
dead = {}
rooms_entered = {}
rooms_pulled = {}
room_adds_called = {}
final_started = false
final_adds_called = 0
tail_started = false
tail_actor = nil
tail_alive = false
tail_next_at = nil

function distance(a, b)
    local dx, dy, dz = a.x - b.x, a.y - b.y, a.z - b.z
    return math.sqrt(dx * dx + dy * dy + dz * dz)
end

-- Whether any player has come within `room.reach` of its boss.
function crossed(info, room)
    for key, actor in pairs(info) do
        if type(key) == "string" and key:match("^player%d+$") and distance(actor, room.boss_at) <= room.reach then
            return true
        end
    end
    return false
end

function onSetup(director)
    director:set_sequence(1)

    for _, room in ipairs(ROOMS) do
        director:boss_room(room.boss, room.wall, room.line, room.place)
        -- The walls stand open until a pull (retail: spawned in state 7)
        director:hide_eobj(room.wall)
    end
    director:hide_eobj(FINAL.wall)
    for _, eobj in ipairs(HIDDEN_EOBJS) do
        director:hide_eobj(eobj)
    end

    for coffer, openings in pairs(LOOT) do
        director:treasure_loot(coffer, openings)
    end
    for _, coffer in ipairs(STANDING_COFFERS) do
        director:spawn_treasure(coffer)
    end

    director:party_npc_yell("tank", PARTY_NPC_LINE_START, PARTY_NPC_LINE_START_AFTER)
end

function burnWeb(director, web)
    webs_burnt[web.index] = true
    director:map_effect(web.index, MAP_EFFECT_BURNT)
    if web.boss ~= nil then
        director:set_targetable(web.boss, true)
    end
    if web.coffer ~= nil then
        director:delay(web.coffer_after)
        director:spawn_treasure(web.coffer)
        director:delay(0)
    end
end

function onUpdate(director, health, player_health, info)
    if info == nil then
        return
    end

    for index, room in ipairs(ROOMS) do
        -- Crossing into the room
        if not rooms_entered[index] and webs_burnt[room.web] and crossed(info, room) then
            rooms_entered[index] = true
            director:objective(room.entered, room.entered + 1)
            if index == 1 then
                director:party_npc_yell("healer", PARTY_NPC_LINE_ROOM1)
            end
        end
        -- Its boss going for someone: the battle music (the seal is the boss room's)
        if not rooms_pulled[index] and info[room.boss] ~= nil and info[room.boss].target ~= nil then
            rooms_pulled[index] = true
            director:set_bgm(BGM_BATTLE)
        end
        -- Its adds
        if room.adds ~= nil and not room_adds_called[index] and health[room.boss] ~= nil
            and health[room.boss] <= room.adds_at then
            room_adds_called[index] = true
            for _, spot in ipairs(room.adds) do
                local add = director:spawn_bnpc(BNPC_ROOM_ADD, spot.x, spot.y, spot.z, spot.rotation)
                director:set_hate(add, nil, 300) -- the player
            end
        end
    end

    -- Crossing into the last room: its cutscene, then its boss
    if not final_started and webs_burnt[5] and crossed(info, FINAL) then
        final_started = true
        director:objective(FINAL.entered, FINAL.entered + 1)
        director:play_cutscene(CUTSCENE_FINAL)
        director:set_weather(WEATHER_FINAL, WEATHER_FINAL_FADE, true)
        director:party_npc_yell("tank", PARTY_NPC_LINE_FINAL, PARTY_NPC_LINE_FINAL_AFTER)
        director:delay(FINAL_BOSS_AFTER)
        director:spawn_boss(FINAL.boss, FINAL.wall, FINAL.line, FINAL.place)
        return
    end

    local boss = health[FINAL.boss]
    if not final_started or boss == nil then
        return
    end

    local adds = FINAL_ADDS[final_adds_called + 1]
    if adds ~= nil and boss <= adds.health then
        final_adds_called = final_adds_called + 1
        for _, spot in ipairs(adds.spots) do
            local add = director:spawn_bnpc(BNPC_FINAL_ADD, spot.x, spot.y, spot.z, spot.rotation)
            director:set_hate(add, nil, 300) -- the player
        end
    end

    if not tail_started and boss <= TAIL_AT then
        tail_started = true
        director:use_action(FINAL.boss, ACTION_TAIL_FIRST, FINAL.boss)
        director:delay(TAIL_AFTER_FIRST)
        useTail(director, info[FINAL.boss])
        return
    end

    if tail_alive and tail_next_at ~= nil and os.time() >= tail_next_at then
        useTail(director, info[FINAL.boss])
    end
end

-- The boss's 702: the tail comes off with the first.
function useTail(director, boss)
    tail_next_at = os.time() + TAIL_AGAIN_EVERY
    director:use_action(FINAL.boss, ACTION_TAIL, FINAL.boss)
    local at = (boss ~= nil and boss.x ~= nil) and boss or FINAL.boss_at
    if tail_actor == nil then
        tail_actor = director:spawn_bnpc(BNPC_TAIL, at.x, at.y, at.z)
        tail_alive = true
        director:set_hate(tail_actor, nil, 300) -- the player
        director:bnpc_control(FINAL.boss, 31, { 0, 0, 0, 1 })
        director:bnpc_control(FINAL.boss, 30, { 0, 1, tail_actor, 0 })
        director:log_message(LOG_TAIL, {})
    end
end

-- The boss's 698 aimed at someone: an egg lands where they stood, bursts, and dies.
function onActorCast(director, layout_id, action_id, actor_id, x, y, z)
    if layout_id ~= FINAL.boss or action_id ~= ACTION_EGG_THROWN or x == nil then
        return
    end
    director:delay(EGG_LANDS_AFTER)
    local egg = director:spawn_bnpc(BNPC_EGG, x, y, z, 0)
    director:hold_bnpc(egg)
    director:delay(EGG_LANDS_AFTER + EGG_BURSTS_AFTER)
    director:actor_use_action(egg, ACTION_EGG, egg)
    director:delay(EGG_LANDS_AFTER + EGG_BURSTS_AFTER + EGG_DIES_AFTER)
    director:kill_bnpc(egg)
end

function onActorDeath(director, layout_id, position)
    dead[layout_id] = (dead[layout_id] or 0) + 1

    for _, web in ipairs(WEBS) do
        if not webs_burnt[web.index] then
            local standing = false
            for _, id in ipairs(web.pack) do
                if dead[id] == nil then
                    standing = true
                end
            end
            if not standing then
                burnWeb(director, web)
            end
        end
    end

    for index, room in ipairs(ROOMS) do
        if layout_id == room.boss then
            -- Its walls go (the boss room's), the music stops, the way on opens and its coffers appear
            director:set_bgm(0)
            director:hide_eobj(room.barrier)
            director:objective(room.cleared, room.cleared + 1)
            for _, coffer in ipairs(room.coffers) do
                director:spawn_treasure(coffer)
            end
        end
    end

    if layout_id == BNPC_TAIL then
        tail_alive = false
        director:bnpc_control(FINAL.boss, 20, { 64 })
        director:bnpc_control(FINAL.boss, 30, { 0, 2, tail_actor, 1 })
        director:bnpc_control(FINAL.boss, 31, { 0, 0, 0, 0 })
    elseif layout_id == FINAL.boss then
        director:objective(FINAL.cleared, nil)
        director:complete_duty()
        director:play_cutscene(CUTSCENE_END, CUTSCENE_END_PARAMS)
        director:set_weather(WEATHER_AFTER, WEATHER_AFTER_FADE, false)
        director:spawn_treasure(FINAL_COFFER)
    end
end

function onGimmickAccessor(director, actor_id, id, params)
    director:finish_gimmick(actor_id)
end

function isGimmickFunctional(id)
    return true
end

-- No shortcut yet (its pop ranges aren't known without the game's layout files.)
function getDebugShortcut(id)
    return 0
end
