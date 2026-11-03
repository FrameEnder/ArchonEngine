-- Haukke Manor
--
-- From Bob's Chronofoil captures of 2026-11-02: one run with Duty Support (5d9c7d7f, visit 0.2, during Skeletons in Her
-- Closet, quest 801, which unlocks it) and three Duty Finder runs (e7580e0d, visits 0.1, 0.3 and 0.5: twice as the
-- tank, once as the healer); `questkit chronofoil --battle` and the visits' packets (docs/src/fork/dungeons.md).
-- Numbers only; names and text are the game's.
--
-- The manor's doors open with its keys: the plain ones with any of the keys lying about (each used up), three with a
-- key of their own: the first boss's, the one lying in the cellar and the pair's. The first boss stands in its hall from
-- the start; the cellar's door brings the pair out; the pair's key opens the stairs up, and coming to the last room
-- plays its cutscene and brings the last boss with its two adds.

-- The director's variables (DirectorVars; retail's sequence was 1 throughout): data 0 counts the keys held, data 1 the
-- other keys found (1 the first boss's, 4 the cellar's, 2 the pair's.)
VAR_KEYS = 0
VAR_FOUND = 1
FOUND_BOSS1_KEY = 1
FOUND_PAIR_KEY = 2
FOUND_CELLAR_KEY = 4

-- Log messages retail sent (LogMessage rows): an item obtained (the item), a door opened with a key (a parameter of
-- its own), an enemy dropped an item (its BNpcName, the item), the stairs' door opened (0).
LOG_OBTAINED = 2031
LOG_OPENED = 2059
LOG_DROPPED = 2069
LOG_STAIRS = 2023

-- The items, by EventItem ID: the keys lying about, the first boss's, the cellar's and the pair's.
ITEM_KEY = 2000260
ITEM_BOSS1_KEY = 2000263
ITEM_CELLAR_KEY = 2000264
ITEM_PAIR_KEY = 2000259

-- The gimmicks (EObj IDs; each a GimmickAccessor event, 0x000F00xx.) Something to pick up has no prompt and a 2 s
-- interaction (EventAction 21); a door a prompt (scene 1), then 2 s (24). Retail's runs used the keys 2000302-2000305
-- and the doors 2000329, 2000337, 2000343 and 2000344; the rest of each series (keys to 2000308, doors 2000331,
-- 2000339 and 2000345, with the same kind of shared group) are taken to be the same. 2000338 stood open (retail spawned
-- it in state 7, every run.)
KEYS = { 2000302, 2000303, 2000304, 2000305, 2000306, 2000307, 2000308 }
KEY_DOORS = { 2000329, 2000331, 2000337, 2000339, 2000343, 2000344, 2000345 }
EOBJ_OPEN_DOOR = 2000338
LOG_KEY_DOOR = 757
EOBJ_BOSS1_KEY = 2000324 -- dropped by the first boss, where it fell (args2 1282 on retail)
EOBJ_BOSS1_KEY_ARGS = 1282
EOBJ_BOSS1_DOOR = 2000355 -- opens with it
LOG_BOSS1_DOOR = 655
EOBJ_CELLAR_KEY = 2000325
EOBJ_CELLAR_DOOR = 2000356 -- opens with it, and the pair come out
LOG_CELLAR_DOOR = 756
EOBJ_PAIR_KEY = 2001235 -- dropped by the last of the pair to fall, where it fell (args2 66818)
EOBJ_PAIR_KEY_ARGS = 66818
EOBJ_STAIRS = 2001233 -- the way up: locked (its own log 2004) until the pair's key
EOBJ_WAY_UP = 2012969 -- shown as the pair fall: back to the entrance (a shortcut of its own, by its GimmickAccessor)
-- If its GimmickAccessor isn't a shortcut the server knows, the script sends them: retail's line (from the gimmick's
-- own handler there) and the entrance's pop range (where retail's warps put them: 67.3-69.0, 0, 0-1.3.)
LOG_WAY_UP = 2049
POP_RANGE_ENTRANCE = 9228207

EVENT_ACTION_PICK_UP = 21
EVENT_ACTION_OPEN = 24

-- The first boss (BNpcBase 272), standing in its hall from the start: the arena's wall and line, its PlaceName and its
-- BNpcName. Its death also takes down 2001236 (the way on, shown from the start.)
BNPC_BOSS1 = 3633770
BOSS1_WALL = 2001605
BOSS1_LINE = 2001604
PLACE_BOSS1 = 604
NAME_BOSS1 = 423
EOBJ_BOSS1_WAY_ON = 2001236

-- The pair (273 and 274) in the cellar, with their BNpcNames; the room opens as the last of them falls.
PAIR = { [3633631] = 426, [3633633] = 427 }
PAIR_WALL = 2001607
PAIR_LINE = 2001606
PLACE_PAIR = 609

-- The last boss (14504) and its adds: 14505 with it from the start, joining in 15.4 s after the boss is pulled; 14506
-- coming 34.0 s after it (all four runs, within 0.1 s.) Its arena's wall (closed from the start, opened by the
-- cutscene) and line.
BNPC_FINAL = 9058727
BNPC_FINAL_ADD = 9058728
BNPC_FINAL_ADD_JOINS = 15.4
BNPC_FINAL_ADD2 = 9058729
BNPC_FINAL_ADD2_COMES = 34.0
FINAL_WALL = 2000357
FINAL_LINE = 2001080
PLACE_FINAL = 608
-- The event ranges at the last room's door (one of them is the cutscene's rect: which isn't known.)
FINAL_RANGES = { [3943563] = true, [4105682] = true, [9393269] = true }

-- The shortcut (2000700, by the entrance) is shown as the first boss is pulled; it takes you outside the first boss's
-- hall, then (from the pair's pull) outside the cellar's room (pop ranges 11438017 and 11438018, where retail's warps
-- put the party.) Outside the last room (11438019) from its cutscene: not seen on retail.
SHORTCUT_BOSS1 = 11438017
SHORTCUT_PAIR = 11438018
SHORTCUT_FINAL = 11438019

BGM_BATTLE = 37

CUTSCENE_FINAL = 170
CUTSCENE_END = 171
CUTSCENE_END_PARAMS = { 1, 18, 0, 0 }

-- What the coffers held on retail, one list per opening (item IDs): the first boss's (73), the pair's (77), the last
-- boss's (79; the Duty Support run's one item, then the Duty Finder runs'), and 68 (standing from the start.)
LOOT = {
    [73] = { { 31464 }, { 31464, 4122 }, { 31456, 4112 } },
    [77] = { { 31459, 4451 }, { 31451, 4445 } },
    [79] = { { 9198 }, { 31466, 2165 }, { 31460, 1844 } },
    [68] = { { 4116 } },
}
-- Two coffers stand from the start (66, 68); the first boss's death brings three (73 by it, 70 and 71 in the cellar),
-- the pair's two (77 by them, 69 upstairs), the last boss's one (79): every run.
START_COFFERS = { 66, 68 }
BOSS1_COFFERS = { 70, 71, 73 }
PAIR_COFFERS = { 69, 77 }
FINAL_COFFER = 79

-- The Duty Support members' story lines (NpcYell, `director:party_npc_yell(role, line, delay)`), when retail's said
-- them: the Marauder 24.7 s in and as the first boss is pulled (4 s before, as the party came into its hall), the
-- Conjurer 32.8 s in and 12.6 s after the last room's cutscene began, the Thaumaturge as the last of the pair fell.
LINE_START_TANK = 14339
LINE_START_TANK_AFTER = 24.7
LINE_START_HEALER = 14344
LINE_START_HEALER_AFTER = 32.8
LINE_BOSS1 = 14345
LINE_PAIR = 14347
LINE_FINAL = 14349
LINE_FINAL_AFTER = 12.6

-- How far the party has got
keys = 0
found = 0
boss1_pulled = false
pair_pulled = false
pair_left = 2
stairs_opened = false
final_started = false
final_pulled = false

function contains(list, id)
    for _, value in ipairs(list) do
        if value == id then
            return true
        end
    end
    return false
end

function sendVars(director)
    director:set_data(VAR_KEYS, keys)
    director:set_data(VAR_FOUND, found)
end

function onSetup(director)
    director:set_sequence(1)
    sendVars(director)

    director:boss_room(BNPC_BOSS1, BOSS1_WALL, BOSS1_LINE, PLACE_BOSS1)
    for id, _ in pairs(PAIR) do
        director:boss_room(id, PAIR_WALL, PAIR_LINE, PLACE_PAIR)
    end
    -- The walls stand open until a pull (retail: spawned in state 7); the way up isn't there yet.
    director:hide_eobj(BOSS1_WALL)
    director:hide_eobj(PAIR_WALL)
    director:hide_eobj(EOBJ_WAY_UP)
    director:hide_eobj(EOBJ_OPEN_DOOR)
    -- The doors standing closed (the keys' and the bosses'), the way on past the first boss and the last room's door:
    -- shown, so nobody walks through them before they open, bots and enemies included (Bob's bundle of 2026-11-02:
    -- the tank bot went up through the locked stairs the moment he was refused, as through any gimmick the director
    -- hadn't touched.) Retail spawned them all in state 0, as shown.
    for _, door in ipairs(KEY_DOORS) do
        director:show_eobj(door)
    end
    for _, door in ipairs({ EOBJ_BOSS1_DOOR, EOBJ_CELLAR_DOOR, EOBJ_STAIRS, EOBJ_BOSS1_WAY_ON, FINAL_WALL }) do
        director:show_eobj(door)
    end

    for coffer, openings in pairs(LOOT) do
        director:treasure_loot(coffer, openings)
    end
    for _, coffer in ipairs(START_COFFERS) do
        director:spawn_treasure(coffer)
    end

    director:party_npc_yell("tank", LINE_START_TANK, LINE_START_TANK_AFTER)
    director:party_npc_yell("healer", LINE_START_HEALER, LINE_START_HEALER_AFTER)
end

function pulled(info, id)
    return info ~= nil and info[id] ~= nil and info[id].target ~= nil
end

function onUpdate(director, health, player_health, info)
    if not boss1_pulled and pulled(info, BNPC_BOSS1) then
        boss1_pulled = true
        director:set_bgm(BGM_BATTLE)
        director:update_shortcut(SHORTCUT_BOSS1)
        director:party_npc_yell("tank", LINE_BOSS1)
    end

    if not pair_pulled then
        for id, _ in pairs(PAIR) do
            if pulled(info, id) then
                pair_pulled = true
                director:set_bgm(BGM_BATTLE)
                director:update_shortcut(SHORTCUT_PAIR)
                break
            end
        end
    end

    if final_started and not final_pulled and pulled(info, BNPC_FINAL) then
        final_pulled = true
        director:delay(BNPC_FINAL_ADD_JOINS)
        director:set_hate(BNPC_FINAL_ADD, nil, 300) -- the player
        director:delay(BNPC_FINAL_ADD2_COMES)
        director:spawn_bnpc(BNPC_FINAL_ADD2)
        director:set_hate(BNPC_FINAL_ADD2, nil, 300)
    end
end

function onActorDeath(director, bnpc_id, position)
    local where = { x = position.x, y = position.y, z = position.z }

    if bnpc_id == BNPC_BOSS1 then
        -- Its arena opens (the boss room's), and the way on; it drops its key where it fell.
        director:set_bgm(0)
        director:log_message(LOG_DROPPED, { NAME_BOSS1, ITEM_BOSS1_KEY })
        director:hide_eobj(EOBJ_BOSS1_WAY_ON)
        director:spawn_eobj(EOBJ_BOSS1_KEY, where, EOBJ_BOSS1_KEY_ARGS)
        for _, coffer in ipairs(BOSS1_COFFERS) do
            director:spawn_treasure(coffer)
        end
    elseif PAIR[bnpc_id] ~= nil then
        pair_left = pair_left - 1
        if pair_left == 0 then
            -- The last of them drops their key (the room opens: the boss rooms'), and the way up appears.
            director:set_bgm(0)
            director:log_message(LOG_DROPPED, { PAIR[bnpc_id], ITEM_PAIR_KEY })
            director:show_eobj(EOBJ_WAY_UP)
            director:spawn_eobj(EOBJ_PAIR_KEY, where, EOBJ_PAIR_KEY_ARGS)
            for _, coffer in ipairs(PAIR_COFFERS) do
                director:spawn_treasure(coffer)
            end
            director:party_npc_yell("ranged", LINE_PAIR)
        end
    elseif bnpc_id == BNPC_FINAL then
        director:despawn_bnpc(BNPC_FINAL_ADD)
        director:spawn_treasure(FINAL_COFFER)
        director:play_cutscene(CUTSCENE_END, CUTSCENE_END_PARAMS)
        director:complete_duty()
    end
end

function onGimmickAccessor(director, actor_id, id, params)
    if contains(KEYS, id) or id == EOBJ_BOSS1_KEY or id == EOBJ_CELLAR_KEY or id == EOBJ_PAIR_KEY then
        director:event_action(EVENT_ACTION_PICK_UP, actor_id, id)
        return
    elseif contains(KEY_DOORS, id) or id == EOBJ_BOSS1_DOOR or id == EOBJ_CELLAR_DOOR or id == EOBJ_STAIRS then
        director:event_action(EVENT_ACTION_OPEN, actor_id, id)
        return
    elseif id == EOBJ_WAY_UP then
        director:log_message(LOG_WAY_UP, {})
        director:finish_gimmick(actor_id)
        director:warp_to_pop_range(actor_id, POP_RANGE_ENTRANCE)
        return
    end

    director:finish_gimmick(actor_id)
end

function isGimmickFunctional(id)
    if contains(KEY_DOORS, id) then
        return keys > 0
    elseif id == EOBJ_BOSS1_DOOR then
        return found & FOUND_BOSS1_KEY ~= 0
    elseif id == EOBJ_CELLAR_DOOR then
        return found & FOUND_CELLAR_KEY ~= 0
    elseif id == EOBJ_STAIRS then
        return found & FOUND_PAIR_KEY ~= 0
    end

    return true
end

-- Something picked up or a door opened: its line, the object gone, the variables.
function onEventActionCast(director, actor_id, target)
    director:finish_gimmick(actor_id)

    if contains(KEYS, target) then
        keys = keys + 1
        director:log_message(LOG_OBTAINED, { ITEM_KEY })
    elseif target == EOBJ_BOSS1_KEY then
        found = found | FOUND_BOSS1_KEY
        director:log_message(LOG_OBTAINED, { ITEM_BOSS1_KEY })
    elseif target == EOBJ_CELLAR_KEY then
        found = found | FOUND_CELLAR_KEY
        director:log_message(LOG_OBTAINED, { ITEM_CELLAR_KEY })
    elseif target == EOBJ_PAIR_KEY then
        found = found | FOUND_PAIR_KEY
        director:log_message(LOG_OBTAINED, { ITEM_PAIR_KEY })
    elseif contains(KEY_DOORS, target) then
        if keys == 0 then
            return
        end
        keys = keys - 1
        director:log_message(LOG_OPENED, { LOG_KEY_DOOR })
    elseif target == EOBJ_BOSS1_DOOR then
        director:log_message(LOG_OPENED, { LOG_BOSS1_DOOR })
    elseif target == EOBJ_CELLAR_DOOR then
        director:log_message(LOG_OPENED, { LOG_CELLAR_DOOR })
        -- The pair come out at once (retail: 0.4 s later) and go for the party as it comes in.
        for id, _ in pairs(PAIR) do
            director:spawn_bnpc(id)
        end
    elseif target == EOBJ_STAIRS then
        stairs_opened = true
        director:log_message(LOG_STAIRS, { 0 })
    else
        return
    end

    director:hide_eobj(target)
    sendVars(director)
end

-- Coming to the last room: its cutscene, its wall opens, and the last boss and its first add are there.
function onGimmickRect(director, target)
    if final_started or not stairs_opened or not FINAL_RANGES[target] then
        return
    end
    final_started = true
    director:hide_eobj(FINAL_WALL)
    director:play_cutscene(CUTSCENE_FINAL)
    director:update_shortcut(SHORTCUT_FINAL)
    director:spawn_boss(BNPC_FINAL, FINAL_WALL, FINAL_LINE, PLACE_FINAL)
    director:spawn_bnpc(BNPC_FINAL_ADD)
    director:party_npc_yell("healer", LINE_FINAL, LINE_FINAL_AFTER)
end

function getDebugShortcut(id)
    if id == 0 then
        return SHORTCUT_BOSS1
    elseif id == 1 then
        return SHORTCUT_PAIR
    elseif id == 2 then
        return SHORTCUT_FINAL
    end

    return 0
end
