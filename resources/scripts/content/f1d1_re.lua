-- The Tam-Tara Deepcroft
--
-- From Bob's Chronofoil captures of 2026-10-24: one run with Duty Support (3ec702a8, visit 0.3, during Fire in the
-- Gloom) and two Duty Finder runs (cc26e0ea, visits 0.2 and 0.4); `questkit chronofoil --battle` and the visits'
-- packets (docs/src/fork/dungeons.md). Numbers only; names and text are the game's.
--
-- Three rooms, each with a pack standing around its leader: pulling the pack seals the room (15 s), the leader's death
-- calls the room's boss (BNpcBase 567), and the boss's death opens the room again and leaves its coffers. A door
-- after each of the first two rooms opens once its boss is dead; the third room's pack waits behind a gate that the
-- key (found further on) opens, and they come at once. Past the third room's door, the seal at the last room plays
-- its cutscene and brings the last boss, who calls adds at 80, 60 and 30 % and takes no damage while one of its
-- shields (BNpcBase 125) stands.

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

-- What the coffers held on retail, one list per opening (item IDs; the three runs.) A coffer without a recording
-- (15, 119) holds one of these. See server/loot.rs.
LOOT = {
    [16] = { { 3556 }, { 2679 } },
    [118] = { { 3330 } },
    [17] = { { 3555 }, { 3555, 4310 } },
    [62] = { { 3033 }, { 3331 }, { 3033 } },
    [11] = { { 31426 }, { 3787, 31425 }, { 31427 } },
}
-- The coffers each room's boss leaves (retail: the same in every run), and the last boss's.
ROOM_COFFERS = { { 118, 15, 16 }, { 17 }, { 62 } }
FINAL_COFFER = 11
-- One coffer stands by the key from the start (119, every run.)
KEY_COFFER = 119

-- The Duty Support members' story lines (NpcYell, `director:party_npc_yell(role, line, delay)`), when retail's said
-- them: the Lancer 34.5 s after the duty began and as the first room's leader fell, the Thaumaturge as the first room's
-- boss fell, the tank 26.5 s after the last seal was broken. Nobody says them without Duty Support.
PARTY_NPC_LINE_START = 14307
PARTY_NPC_LINE_START_AFTER = 34.5
PARTY_NPC_LINE_LEADER1 = 14309
PARTY_NPC_LINE_BOSS1 = 14311
PARTY_NPC_LINE_FINAL = 14312
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
rooms_pulled = {}
rooms_cleared = {}
has_key = false
gate_opened = false
door3_opened = false
final_started = false
final_waves = 0
shielded = false

function setVars(director, vars)
    for index, value in ipairs(vars) do
        director:set_data(index - 1, value)
    end
end

function onSetup(director)
    director:set_sequence(1)
    setVars(director, VARS_START)

    for index, room in ipairs(ROOMS) do
        director:boss_room(room.boss, room.wall, room.line, room.place, room.pack)
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
    director:spawn_treasure(KEY_COFFER)

    director:party_npc_yell("melee", PARTY_NPC_LINE_START, PARTY_NPC_LINE_START_AFTER)
end

function roomOf(layout_id)
    for index, room in ipairs(ROOMS) do
        if room.leader == layout_id then
            return index, "leader"
        elseif room.boss == layout_id then
            return index, "boss"
        end
    end
    return nil
end

-- A room's pack going for someone: battle music and its leader's line (the seal itself is the boss room's.)
function pullRoom(director, index)
    if rooms_pulled[index] then
        return
    end
    rooms_pulled[index] = true
    local room = ROOMS[index]
    director:set_bgm(BGM_BATTLE)
    director:yell(room.leader, room.yell)
end

function onUpdate(director, health, player_health, info)
    for index, room in ipairs(ROOMS) do
        if not rooms_pulled[index] and info ~= nil then
            for _, id in ipairs(room.pack) do
                if info[id] ~= nil and info[id].target ~= nil then
                    pullRoom(director, index)
                    break
                end
            end
        end
    end

    if final_started and health[BNPC_FINAL_BOSS] ~= nil then
        local wave = FINAL_WAVES[final_waves + 1]
        if wave ~= nil and health[BNPC_FINAL_BOSS] < wave.health then
            final_waves = final_waves + 1
            for _, id in ipairs(wave.adds) do
                director:spawn_bnpc(id)
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
    local index, what = roomOf(bnpc_id)
    if what == "leader" then
        -- Its death calls the room's boss, who goes for the party at once
        director:log_message(LOG_BOSS_APPEARS, { ROOM_BOSS_NAME, 1 })
        director:spawn_bnpc(ROOMS[index].boss)
        director:set_hate(ROOMS[index].boss, nil, 300) -- the player
        if index == 1 then
            director:party_npc_yell("melee", PARTY_NPC_LINE_LEADER1)
        end
    elseif what == "boss" then
        -- The room opens again (the boss room's), the music stops, its coffers appear
        rooms_cleared[index] = true
        director:set_bgm(0)
        director:log_message(LOG_ROOM_CLEARED, {})
        for _, coffer in ipairs(ROOM_COFFERS[index]) do
            director:spawn_treasure(coffer)
        end
        if index == 1 then
            director:party_npc_yell("ranged", PARTY_NPC_LINE_BOSS1)
        end
    elseif bnpc_id == BNPC_FINAL_BOSS then
        director:spawn_treasure(FINAL_COFFER)
        director:play_cutscene(CUTSCENE_END, CUTSCENE_END_PARAMS)
        director:complete_duty()
    end
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
        pullRoom(director, 3)
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
        director:spawn_boss(BNPC_FINAL_BOSS, EOBJ_FINAL_WALL, EOBJ_FINAL_LINE, PLACE_FINAL)
        director:party_npc_yell("tank", PARTY_NPC_LINE_FINAL, PARTY_NPC_LINE_FINAL_AFTER)
    end
end

-- No shortcuts yet (their pop ranges aren't known without the game's layout files.)
function getDebugShortcut(id)
    return 0
end
