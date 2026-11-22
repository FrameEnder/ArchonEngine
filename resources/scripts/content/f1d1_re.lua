-- The Tam-Tara Deepcroft
--
-- From Bob's Chronofoil captures of 2026-10-24: one run with Duty Support (3ec702a8, visit 0.3, during Fire in the
-- Gloom) and two Duty Finder runs (cc26e0ea, visits 0.2 and 0.4); `questkit chronofoil --battle` and the visits'
-- packets (docs/src/fork/dungeons.md). Numbers only; names and text are the game's.
--
-- Three rooms, each with a pack standing around its leader: pulling the pack seals the room (15 s), the pack's last
-- death calls the room's boss (BNpcBase 567), and the boss's death opens the room again and leaves its coffers. A door
-- after each of the first two rooms opens once its boss is dead; the third room's pack waits behind a gate that the
-- key (found further on) opens, and they come at once. Past the third room's door, the seal at the last room plays
-- its cutscene and brings the last boss, who calls adds at 80, 60 and 30 % and takes no damage while one of its
-- shields (BNpcBase 125) stands.
--
-- On the dungeon framework (`content/lib/Dungeon.lua`, docs/src/fork/dungeons.md) since 2026-11-21: each room's pack
-- (its fall calls the boss) and the room's fight from the pack's pull to the boss's fall, and the last boss; the
-- gimmicks, the leaders' lines and the last boss's waves and shields are its own.

-- The rooms: the boss arena's wall and line (shown while sealed), its place name, the pack, its leader and its boss.
ROOMS = {
    {
        wall = 2001699, line = 2001700, place = 535,
        pack = { 1334953, 1334954, 1334952 }, leader = 1334952, boss = 3859972,
        -- The leader's line as the pack is pulled (NpcYell, under its BNpcName.)
        yell = 563,
    },
    {
        wall = 2001701, line = 2001702, place = 539,
        pack = { 1334956, 1334957, 1334959, 1334955 }, leader = 1334955, boss = 3859980,
        yell = 564,
    },
    {
        wall = 2001703, line = 2001704, place = 541,
        pack = { 1334961, 1334962, 1334963, 1334965, 1334960 }, leader = 1334960, boss = 3859984,
        yell = 565,
    },
}

-- The last room
EOBJ_FINAL_WALL = 2001741
EOBJ_FINAL_LINE = 2001085
PLACE_FINAL = 543
BNPC_FINAL_BOSS = 1332316
-- Its shields (BNpcBase 125) and adds (126, 127), in three waves as it falls below 80, 60 and 30 % (retail: one
-- shield at 79-80 %; a shield and three adds at 57-60 %; two shields and four adds at 28-29 %, in all three runs.)
FINAL_WAVES = {
    { health = 80, adds = { 2029064 } },
    { health = 60, adds = { 2029065, 1334967, 1334968, 1334975 } },
    { health = 30, adds = { 2029061, 2029067, 1334969, 1334970, 1334976, 2029118 } },
}
FINAL_SHIELDS = { 2029061, 2029064, 2029065, 2029067 }
-- Each shield's spawn tethers it to the boss: Channeling row 1, the line the client draws between them, which goes
-- when the shield dies (Bob's three runs: every shield's SpawnNpc had the boss as its tether target and tether 1.)
TETHER_SHIELD = 1

-- The gimmicks (EObj IDs)
EOBJ_DOOR1 = 2000061 -- after the first room: opens the way (2001697)
EOBJ_BARRIER1 = 2001697
EOBJ_DOOR2 = 2000062 -- after the second room (2001698)
EOBJ_BARRIER2 = 2001698
EOBJ_KEY = 2000057 -- the key to the gate
EOBJ_GATE = 2000060 -- the third room's gate (needs the key)
EOBJ_DOOR3 = 2000063 -- after the third room
EOBJ_FINAL_SEAL = 2000067 -- the last room's seal: its cutscene, then the last boss

EVENT_ACTION_INTERACT = 24

-- Log messages retail sent (LogMessage rows): a door or seal giving way, the last seal's second line, the key, the
-- gate opened with it (one parameter, 0), a room's boss appearing (its BNpcName, then 1) and gone.
LOG_OPENED = 1537
LOG_FINAL_SEAL = 1539
LOG_KEY = 2031
LOG_GATE = 1536
LOG_ROOM_CLEARED = 2006
LOG_BOSS_APPEARS = 2009
ROOM_BOSS_NAME = 455
ITEM_KEY = 2000244

BGM_BATTLE = 37

-- The cutscenes: the last seal's (26) and the last boss's death (27), with retail's scene parameters after them.
CUTSCENE_FINAL = 26
CUTSCENE_END = 27
CUTSCENE_END_PARAMS = { 1, 18, 0, 0 }

-- What the coffers held on retail, one list per opening (item IDs; the three runs, and a fourth through Duty Support
-- as a gladiator, Bob's Chronofoil capture c99fe374 of 2026-10-28: not chosen for the class, an Acolyte's Skirt and a
-- Plundered Grimoire among them; and a fifth with a Duty Finder party, cc62ad94 of the same day, the last of 16, 62 and
-- 11.) A coffer without a recording (15, 119) holds one of these. See server/loot.rs.
LOOT = {
    [16] = { { 3556 }, { 2679 }, { 2684 }, { 3555 } },
    [118] = { { 3330 }, { 3330 } },
    [17] = { { 3555 }, { 3555, 4310 }, { 2683 } },
    [62] = { { 3033 }, { 3331 }, { 3033 }, { 3330 }, { 3331 } },
    [11] = { { 31426 }, { 3787, 31425 }, { 31427 }, { 31428 }, { 3786, 31422 } },
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

-- The wiki's lists: the three Void Soulcounters' silver coffers (16, 17, 62), Galvanth's gold one (11), and Treasure
-- Coffers 1-3 (15, 118, 119.)
WIKI_LOOT = {
    [16] = HEAD_HANDS,
    [17] = joined(HEAD_HANDS, ACCESSORIES),
    [62] = BODY_LEGS,
    [11] = joined(WEAPONS, FEET),
    [15] = joined(FEET, ACCESSORIES),
    [118] = BODY_LEGS,
    [119] = joined(WEAPONS, OLDER_PIECES),
}
-- The coffers each room's boss leaves (retail: the same in every run), and the last boss's.
ROOM_COFFERS = { { 118, 15, 16 }, { 17 }, { 62 } }
FINAL_COFFER = 11
-- One coffer stands by the key from the start (119, every run.)
KEY_COFFER = 119

-- The Duty Support members' story lines (NpcYell, `director:party_npc_yell(role, line, delay)`), when retail's said
-- them: the Lancer 34.5 s after the duty began and as the first room's leader fell, the Thaumaturge as the first room's
-- boss fell, the tank 26.5 s after the last seal was broken (when the player is the tank, the Conjurer's 14313 then,
-- 26.8 s after: Bob's Chronofoil capture c99fe374 of 2026-10-28.) Nobody says them without Duty Support.
PARTY_NPC_LINE_START = 14307
PARTY_NPC_LINE_START_AFTER = 34.5
PARTY_NPC_LINE_LEADER1 = 14309
PARTY_NPC_LINE_BOSS1 = 14311
PARTY_NPC_LINE_FINAL = 14312
PARTY_NPC_LINE_FINAL_HEALER = 14313
PARTY_NPC_LINE_FINAL_AFTER = 26.5

-- The director's variables as retail sent them (DirectorVars), at the start and as each gimmick was used. Retail's
-- sequence was 1 throughout.
VARS_START = { 62, 0, 0, 0, 0 }
VARS_DOOR1 = { 60, 0, 0, 1, 0 }
VARS_DOOR2 = { 56, 0, 0, 2, 1 }
VARS_KEY = { 56, 1, 0, 2, 3 }
VARS_GATE = { 56, 1, 1, 2, 3 }
VARS_DOOR3 = { 48, 1, 1, 3, 3 }
VARS_FINAL = { 1, 1, 1, 4, 7 }

-- How far the party has got
rooms_cleared = {}
leaders_dead = {}
final_dead = false
has_key = false
gate_opened = false
door3_opened = false
final_started = false
final_waves = 0

function isShield(id)
    for _, shield in ipairs(FINAL_SHIELDS) do
        if shield == id then
            return true
        end
    end
    return false
end
shielded = false

function setVars(director, vars)
    for index, value in ipairs(vars) do
        director:set_data(index - 1, value)
    end
end

DUNGEON = Dungeon.new()
D = DUNGEON

-- Each room: its pack, whose last to fall calls the boss, who goes for the party at once. (Fifteen rooms in five of
-- Bob's retail runs: it appeared 0.1-0.5 s after the pack's last death in twelve, whichever of them that was, the
-- leader's own death up to 7.2 s before; in three it came with one of the pack still standing at 1-5% of its health,
-- 1.5-1.8 s before that one died. What decided those isn't known.) The room's fight is from the pack's pull (the seal
-- is the boss room's, its pack its triggers) to the boss's fall: battle music, and the leader's line while there's a
-- leader to say it (not on a pull after a wipe that it didn't live to); the boss's fall opens the room again, the
-- music stops and its coffers appear.
for index, room in ipairs(ROOMS) do
    D:pack({ name = "room" .. index, layouts = room.pack,
        on_fall = function(d)
            d:log_message(LOG_BOSS_APPEARS, { ROOM_BOSS_NAME, 1 })
            d:spawn_bnpc(room.boss)
            d:set_hate(room.boss, nil, 300) -- the player
        end })
    room.fight = D:boss({ id = index, layout = room.boss, wall = room.wall, line = room.line, place = room.place,
        triggers = room.pack, pulled_by = room.pack, eobjs_at_setup = false, bgm = BGM_BATTLE, after_bgm = 0,
        on_pull = function(d)
            if not leaders_dead[index] then
                d:yell(room.leader, room.yell)
            end
        end,
        on_death = function(d)
            rooms_cleared[index] = true
            d:log_message(LOG_ROOM_CLEARED, {})
            for _, coffer in ipairs(ROOM_COFFERS[index]) do
                d:spawn_treasure(coffer)
            end
            if index == 1 then
                d:party_npc_yell("ranged", PARTY_NPC_LINE_BOSS1)
            end
        end })
end

-- The last boss: brought by the last seal; no music of its own (none on retail.) A wipe takes away the adds of the
-- waves it had reached, and it can be hurt again: its waves come again as its health falls from full. Not seen on
-- retail (no wipe here in Bob's captures); found by `questkit duty-sim`'s rehearsed wipes (2026-11-10.)
FINAL = D:boss({ id = 4, layout = BNPC_FINAL_BOSS, wall = EOBJ_FINAL_WALL, line = EOBJ_FINAL_LINE, place = PLACE_FINAL,
    comes_later = true, eobjs_at_setup = false, bgm = false, last = true,
    on_wipe = function(d)
        for wave = 1, final_waves do
            for _, id in ipairs(FINAL_WAVES[wave].adds) do
                d:despawn_bnpc(id)
            end
        end
        final_waves = 0
        if shielded then
            shielded = false
            d:set_invulnerable(BNPC_FINAL_BOSS, false)
        end
    end,
    on_death = function(d)
        final_dead = true
        d:spawn_treasure(FINAL_COFFER)
        d:play_cutscene(CUTSCENE_END, CUTSCENE_END_PARAMS)
        d:complete_duty()
    end })

function onSetup(director)
    director:set_sequence(1)
    setVars(director, VARS_START)

    D:setup(director)
    for _, room in ipairs(ROOMS) do
        -- The walls stand open until a pull (retail: spawned in state 7)
        director:hide_eobj(room.wall)
    end
    director:hide_eobj(EOBJ_FINAL_WALL)

    -- The third room's pack stays where it is, behind the gate, until it opens
    for _, id in ipairs(ROOMS[3].pack) do
        director:set_notice(id, false)
    end

    for coffer, openings in pairs(LOOT) do
        director:treasure_loot(coffer, openings)
    end
    for coffer, items in pairs(WIKI_LOOT) do
        director:treasure_pool(coffer, items)
    end
    director:spawn_treasure(KEY_COFFER)

    director:party_npc_yell("melee", PARTY_NPC_LINE_START, PARTY_NPC_LINE_START_AFTER)
end

-- The rooms' and the last boss's pulls (`D`); the last boss's waves and shields.
function onUpdate(director, health, player_health, info)
    D:update(director, health, player_health, info, function(d) finalWaves(d, health) end)
end

function finalWaves(director, health)
    if final_started and health[BNPC_FINAL_BOSS] ~= nil then
        local wave = FINAL_WAVES[final_waves + 1]
        if wave ~= nil and health[BNPC_FINAL_BOSS] < wave.health then
            final_waves = final_waves + 1
            for _, id in ipairs(wave.adds) do
                if isShield(id) then
                    director:spawn_tethered_bnpc(id, BNPC_FINAL_BOSS, TETHER_SHIELD)
                else
                    director:spawn_bnpc(id)
                end
                director:set_hate(id, nil, 300) -- the player
            end
        end

        -- No damage to the boss while a shield stands
        local shield_up = false
        for _, id in ipairs(FINAL_SHIELDS) do
            if health[id] ~= nil then
                shield_up = true
            end
        end
        if shield_up ~= shielded then
            shielded = shield_up
            director:set_invulnerable(BNPC_FINAL_BOSS, shield_up)
        end
    end
end

function onActorDeath(director, bnpc_id, position)
    for index, room in ipairs(ROOMS) do
        if bnpc_id == room.leader then
            leaders_dead[index] = true
            if index == 1 then
                director:party_npc_yell("melee", PARTY_NPC_LINE_LEADER1)
            end
        end
    end
    D:died(director, bnpc_id, position)
end

-- Everyone back at the start (a wipe; the server sends the enemies home whole): the fight under way is forgotten, so
-- the next pull is a first pull again (`D`): a room not yet cleared is pulled anew (its music starts with it; its boss
-- stays out if its leader had fallen), and the last boss's waves go (its `on_wipe`.)
function onPartyWipe(director)
    D:wipe(director)
end

function onGimmickAccessor(director, actor_id, id, params)
    if id == EOBJ_DOOR1 or id == EOBJ_DOOR2 or id == EOBJ_KEY or id == EOBJ_GATE or id == EOBJ_DOOR3
        or id == EOBJ_FINAL_SEAL then
        director:event_action(EVENT_ACTION_INTERACT, actor_id, id)
        return
    end

    director:finish_gimmick(actor_id)
end

function isGimmickFunctional(id)
    if id == EOBJ_DOOR1 then
        return rooms_cleared[1] == true
    elseif id == EOBJ_DOOR2 then
        return rooms_cleared[2] == true
    elseif id == EOBJ_GATE then
        return has_key
    elseif id == EOBJ_DOOR3 then
        return rooms_cleared[3] == true
    elseif id == EOBJ_FINAL_SEAL then
        return door3_opened
    end

    return true
end

function onEventActionCast(director, actor_id, target)
    director:finish_gimmick(actor_id)

    if target == EOBJ_DOOR1 then
        director:hide_eobj(EOBJ_BARRIER1)
        director:log_message(LOG_OPENED, {})
        director:hide_eobj(EOBJ_DOOR1)
        setVars(director, VARS_DOOR1)
    elseif target == EOBJ_DOOR2 then
        director:hide_eobj(EOBJ_BARRIER2)
        director:log_message(LOG_OPENED, {})
        director:hide_eobj(EOBJ_DOOR2)
        setVars(director, VARS_DOOR2)
    elseif target == EOBJ_KEY then
        has_key = true
        director:log_message(LOG_KEY, { ITEM_KEY })
        director:hide_eobj(EOBJ_KEY)
        setVars(director, VARS_KEY)
    elseif target == EOBJ_GATE and not gate_opened then
        gate_opened = true
        director:log_message(LOG_GATE, { 0 })
        director:hide_eobj(EOBJ_GATE)
        setVars(director, VARS_GATE)

        -- The pack behind it comes at once (retail: 0.4 s later)
        for _, id in ipairs(ROOMS[3].pack) do
            director:set_notice(id, true)
            director:set_hate(id, nil, 300) -- the player
        end
        -- (Its fight begins with it: the music, the leader's line.)
        if D.fight == nil then
            D:pull(director, ROOMS[3].fight)
        end
    elseif target == EOBJ_DOOR3 then
        door3_opened = true
        director:log_message(LOG_OPENED, {})
        director:hide_eobj(EOBJ_DOOR3)
        setVars(director, VARS_DOOR3)
    elseif target == EOBJ_FINAL_SEAL and not final_started then
        final_started = true
        director:log_message(LOG_OPENED, {})
        director:log_message(LOG_FINAL_SEAL, {})
        director:hide_eobj(EOBJ_FINAL_SEAL)
        setVars(director, VARS_FINAL)
        director:play_cutscene(CUTSCENE_FINAL)
        D:bringBoss(director, FINAL)
        director:party_npc_yell("tank", PARTY_NPC_LINE_FINAL, PARTY_NPC_LINE_FINAL_AFTER, "healer",
            PARTY_NPC_LINE_FINAL_HEALER)
    end
end

-- No shortcuts yet (their pop ranges aren't known without the game's layout files.)
function getDebugShortcut(id)
    return 0
end
