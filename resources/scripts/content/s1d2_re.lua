-- Brayflox's Longstop
--
-- From Bob's Chronofoil capture 44df02fc of 2026-11-03: one run with Duty Support (visit 0.20, 34 minutes, during The
-- Things We Do for Cheese, quest 832, which unlocks it); `questkit chronofoil --battle 0.20` and the visit's packets
-- (docs/src/fork/dungeons.md). Numbers only; names and text are the game's. Times below are from entering the duty.
--
-- A dragon perched by the entrance flies off as the duty begins. An event NPC by the first hall hands the party the
-- key to the first boss's pond; the pond's door opens with it and the boss comes with three adds, more in waves while
-- it lives. Its death opens the way on, where a goblin runs ahead; the second boss comes, and at 80 % a goblin joins
-- the fight against its add. The second boss's death brings the third, whose 979 leaves blobs behind; at 20 % the
-- dragon comes down into its room, and its room opens once both it and the dragon are gone. A door after it opens by
-- hand, and the last boss's cutscene plays as the party comes up to its terrace.

-- The director's variables (DirectorVars; retail's sequence was 1 throughout): data 0 the key held, data 2 and 3 the
-- party's progress (1 the key, 2 the first boss, 4 the second, 8 the door after the third; data 3 alone gets 16 as the
-- last boss falls): 1, 0, 1, 1 at the key; 0, 0, 1, 1 through the pond's door; then 3, 3; 7, 7; 15, 15; 15, 31.
VAR_KEY = 0
VAR_PROGRESS = 2
VAR_CLEARED = 3
PROGRESS_KEY = 1
PROGRESS_BOSS1 = 2
PROGRESS_BOSS2 = 4
PROGRESS_DOOR = 8
CLEARED_FINAL = 16

-- Log messages retail sent (LogMessage rows): an item obtained (the item), a door opened (a parameter of its own),
-- the first boss's waves (2052, no parameters), the dragon coming down and leaving (2055, 2056).
LOG_OBTAINED = 2031
LOG_OPENED = 2059
LOG_WAVE = 2052
LOG_DRAGON_LANDS = 2055
LOG_DRAGON_LEAVES = 2056

-- The key (EventItem) and the event NPC who hands it over (ENpcResident 1004346, layout 4000399: ContentNpc 1).
ITEM_KEY = 2000521
CONTENT_NPC_KEY = 1

-- The doors (EObj IDs; GimmickAccessor events 0x000F00A2-A6): the pond's (a prompt, then EventAction 24 for 2 s) opens
-- with the key; the first, second and third bosses' rooms' ways on open as their rooms do; the door after the third
-- (no prompt, EventAction 24 for 2 s) once its room has opened. Each with its line's parameter.
EOBJ_POND_DOOR = 2001462
LOG_POND_DOOR = 759
EOBJ_BOSS1_WAY_ON = 2001463
LOG_BOSS1_WAY_ON = 760
EOBJ_BOSS2_WAY_ON = 2001464
LOG_BOSS2_WAY_ON = 761
EOBJ_BOSS3_WAY_ON = 2001465
LOG_BOSS3_WAY_ON = 762
EOBJ_TERRACE_DOOR = 2001466
LOG_TERRACE_DOOR = 691
EVENT_ACTION_OPEN = 24
DOORS = { EOBJ_POND_DOOR, EOBJ_BOSS1_WAY_ON, EOBJ_BOSS2_WAY_ON, EOBJ_BOSS3_WAY_ON, EOBJ_TERRACE_DOOR }
EOBJ_SHORTCUT = 2000700

-- Map effects retail set as doors opened (state 1, timeline 2): the pond's door, then each of the first three rooms'
-- ways on.
MAP_EFFECT_POND_DOOR = 7
MAP_EFFECT_BOSS1 = 8
MAP_EFFECT_BOSS2 = 9
MAP_EFFECT_BOSS3 = 10
MAP_EFFECT_OPEN = 2

BGM_BATTLE = 37

-- The dragon perched by the entrance (BNpcBase 989, untargetable): ActionTimeline 1385 3.7 s after the duty began, gone
-- 10.3 s later.
BNPC_PERCHED_DRAGON = 3992822
PERCHED_DRAGON_FLIES_AFTER = 3.7
PERCHED_DRAGON_GONE_AFTER = 10.3
TIMELINE_DRAGON_FLIES = 1385

-- The first boss (BNpcBase 423) and its three adds (425, all layout 4102269), spawned 0.4 s after the key: where
-- retail's stood.
BNPC_BOSS1 = 3992765
ADD_425 = 4102269
ADD_426 = 4102276
ADD_424 = 4102260
BOSS1_ADDS = {
    { layout = ADD_425, x = 117.0, y = -3.09, z = -25.0 },
    { layout = ADD_425, x = 113.5, y = -3.19, z = -25.0 },
    { layout = ADD_425, x = 110.0, y = -3.19, z = -25.0 },
}
BOSS1_ADDS_ROTATION = -0.149
BOSS1_WALL = 2001584
BOSS1_LINE = 2001585
PLACE_BOSS1 = 687
-- Its waves: three adds 60.1 s after the pull, then every 55 s (55.3, 54.8) while it lives, each after log 2052, going
-- for the party at once. Retail's three, where they came; after them the same three again (not seen: the boss fell
-- 31.8 s after the third).
BOSS1_FIRST_WAVE = 60.1
BOSS1_WAVE_EVERY = 55.0
BOSS1_WAVES = {
    { { ADD_425, 113.26, -3.18, -24.73 }, { ADD_425, 112.59, -3.18, -26.06 }, { ADD_426, 114.51, -3.18, -24.77 } },
    { { ADD_426, 112.83, -3.12, -23.1 }, { ADD_425, 106.8, -3.03, -25.26 }, { ADD_425, 115.92, -3.11, -23.93 } },
    { { ADD_425, 111.97, -3.13, -29.03 }, { ADD_424, 119.5, -2.97, -26.92 }, { ADD_426, 118.32, -3.06, -25.98 } },
}

-- After the first boss: an enemy (1046) on the way on, and a goblin (BNpcBase 420, on the party's side) running at it,
-- its line 143 5.0 s later (twice, a second apart) and 148 with ActionTimeline 957 2.1 s after the enemy fell. The
-- second boss (990) comes 13.4 s after the first fell.
BNPC_AFTER_BOSS1 = 3992769
BNPC_GOBLIN_RUNNER = 4102288
YELL_RUNNER_GO = 143
YELL_RUNNER_GO_AFTER = { 5.0, 6.0 }
YELL_CHEER = 148
YELL_CHEER_AFTER = 2.1
TIMELINE_CHEER = 957
BOSS2_COMES_AFTER = 13.4

-- The goblins standing about (BNpcBase 421, on the party's side): the one on the ledge calls out (145) every 15 s from
-- 18.9 s after the first boss fell until the second boss is pulled; two by the ambushes below.
BNPC_GOBLIN_LEDGE = 4155136
YELL_HELP = 145
YELL_HELP_FIRST = 18.9
YELL_HELP_EVERY = 15.0
YELL_HELP_AGAIN = 146

-- The ambushes: three 1044s by one goblin and two by another, appearing as the party comes within about 28 and 25
-- yalms of their goblin (where retail's player was when they appeared) and going for the goblin; the goblin calls out
-- as they come (145; the first 146 15.8 s later), the second cheers as the last of its two falls (148, ActionTimeline
-- 957, 1.4 s after), and that drops coffer 106.
AMBUSHES = {
    { goblin = 4000414, at = { x = 19.35, z = -62.77 }, within = 28.0, enemies = { 3977699, 3977700, 3977701 },
        again = 15.8 },
    { goblin = 4000410, at = { x = 65.58, z = -59.71 }, within = 25.0, enemies = { 3977704, 3977705 }, cheer = 1.4,
        coffer = 106 },
}

-- The second boss (990) and, as it falls to 80 %, its add (991) and the goblin (422, level 20, on the party's side) who
-- takes it on; the boss's 984 starts 3.7 s after the add falls (its timeline's points after 991). The goblin's lines:
-- 150 18 s after it came, 151 with its 991 and 156 4 s later, 155 with its 990 and 157 6 s later, 158 6.3 s after the
-- boss's room opened.
BNPC_BOSS2 = 4102301
BNPC_BOSS2_ADD = 4102303
BNPC_GOBLIN_FIGHTER = 4102292
BOSS2_ADD_BELOW = 80
BOSS2_WALL = 2001586
BOSS2_LINE = 2001587
PLACE_BOSS2 = 688
YELL_FIGHTER_COMES = 150
YELL_FIGHTER_COMES_AFTER = 18.0
YELL_BRAVADO = 151
YELL_BRAVADO_AFTER = 156
YELL_BRAVADO_AFTER_DELAY = 4.0
YELL_BANG_TOSS = 155
YELL_BANG_TOSS_AFTER = 157
YELL_BANG_TOSS_AFTER_DELAY = 6.0
YELL_FIGHTER_DONE = 158
YELL_FIGHTER_DONE_AFTER = 6.3
ACTION_BRAVADO = 991
ACTION_BANG_TOSS = 990
-- The boss's timeline's phase once its add has fallen (its points `after` this: the add's BNpcBase.)
BOSS2_ADD_GONE = 991

-- The third boss (427): 2.6-2.9 s after each of its 979s a 1064 (layout 4107077) where its target stood, going for the
-- player; at 20 % the dragon (429), spawned unseen, comes down 0.8 s later with 564, logs 2055 4.8 s in, becomes
-- untargetable 24.2 s in, flies off (ActionTimeline 1385) at 25.6, logs 2056 at 31.2 and is gone at 36.3; the room
-- opens then, or as the boss falls if the dragon has gone.
BNPC_BOSS3 = 3997806
BNPC_BOSS3_BLOB = 4107077
BNPC_DRAGON = 3977798
BOSS3_WALL = 2001588
BOSS3_LINE = 2001589
PLACE_BOSS3 = 689
ACTION_EFFLUVIUM = 979
BLOB_AFTER = 2.7
DRAGON_BELOW = 20
DRAGON_APPEARS_AFTER = 0.8
ACTION_TOUCHDOWN = 564
DRAGON_LANDS_LOG_AFTER = 4.8
DRAGON_UNTARGETABLE_AFTER = 24.2
DRAGON_FLIES_AFTER = 25.6
DRAGON_LEAVES_LOG_AFTER = 31.2
DRAGON_GONE_AFTER = 36.3

-- The last boss (14533) and its terrace. Its cutscene (184) played as retail's player came up the slope at about
-- x -73, z -233 (the layout's event range isn't known yet); the boss and its helpers came 2.4 s later.
BNPC_FINAL = 9059658
FINAL_WALL = 2002872
FINAL_LINE = 2002735
PLACE_FINAL = 690
CUTSCENE_FINAL = 184
FINAL_TRIGGER = { x = -74.0, y = 30.0, z = -225.0 }
CUTSCENE_END = 185
CUTSCENE_END_PARAMS = { 1, 18, 0, 0 }
-- The event NPCs standing on the terrace 3.1 s after the clear (ENpcResidents 1043371, 1043373, 1043374).
AFTER_CLEAR_NPCS = { 9303244, 9303246, 9303247 }
AFTER_CLEAR_NPCS_AFTER = 3.1

-- The puddles: retail's 14 helpers (BNpcBase 9020, no layout ID; the fork's 99009020), two on each of seven spots, a
-- map effect for each (0-6). With each of the boss's 28656 (on itself), four or three spots' helpers cast 28657 there
-- at once; 5.6 s later their map effects go to state 1 (timeline 2) and the spots' other helpers cast 28658 (9.0 s);
-- 8.9 s after that the map effects go to state 4 (timeline 8). The first 28656 marks the middle and set A; after it the
-- boss casts two 5.1 s apart, the first marking the middle and one of the sets (A in two of four pairs), the second
-- the other set. Which map effect is which spot: the middle is 0 (it came with both sets); A's three are 2, 4, 6 and
-- B's 1, 3, 5, but which of them is which is a guess until the layout's map effects are read.
HELPER = 99009020
ACTION_TOXIC_VOMIT = 28656
ACTION_MARK = 28657
ACTION_BURST = 28658
BURST_AFTER = 5.6
BURST_LANDS_AFTER = 8.9
SPOTS = {
    [0] = { x = -26.44, y = 34.99, z = -236.01, rotation = 0.0 },
    [1] = { x = -40.45, y = 34.47, z = -227.99, rotation = 2.059 },
    [2] = { x = -40.45, y = 34.99, z = -244.01, rotation = 1.065 },
    [3] = { x = -12.47, y = 35.45, z = -227.99, rotation = -2.042 },
    [4] = { x = -26.44, y = 35.23, z = -219.99, rotation = -3.142 },
    [5] = { x = -26.44, y = 35.2, z = -252.0, rotation = 0.0 },
    [6] = { x = -12.47, y = 35.39, z = -244.01, rotation = -1.012 },
}
SET_A = { 2, 4, 6 }
SET_B = { 1, 3, 5 }
MAP_EFFECT_MARKED = { timeline = 2, state = 1 }
MAP_EFFECT_BURST = { timeline = 8, state = 4 }

-- What the coffers held on retail (one opening, the Duty Support run's one item each): the first boss's (109), the
-- enemy after it (105, as it fell), the second ambush (106, as the last of the two fell), the second boss's (110), the
-- third room's (111) and the last boss's (112).
LOOT = {
    [109] = { { 31476 } },
    [105] = { { 4462 } },
    [106] = { { 3608 } },
    [110] = { { 3610 } },
    [111] = { { 3382 } },
    [112] = { { 31472 } },
}
COFFER_BOSS1 = 109
COFFER_AFTER_BOSS1 = 105
COFFER_BOSS2 = 110
COFFER_BOSS3 = 111
COFFER_FINAL = 112

-- The Duty Support members' story lines (NpcYell, `director:party_npc_yell(role, line, delay)`), when retail's said
-- them: the Marauder 19.8 s in and 8.4 s after the first boss fell, the Conjurer as the second boss fell and as the
-- third room opened, the Thaumaturge 32.4 s after the last boss's cutscene began.
LINE_START = 14350
LINE_START_AFTER = 19.8
LINE_BOSS1 = 14354
LINE_BOSS1_AFTER = 8.4
LINE_BOSS2 = 14357
LINE_BOSS3 = 14359
LINE_FINAL = 14360
LINE_FINAL_AFTER = 32.4

-- onUpdate comes every logic tick (500 ms): the script's own clock, for what depends on how the fight goes.
TICK = 0.5

-- How far the party has got
clock = 0
later = {}
has_key = false
key_given = false
progress = 0
cleared = 0
boss1_pulled = false
boss1_pulled_at = nil
boss1_waves = 0
boss1_dead = false
runner_target_dead = false
ledge_calls = false
ambushed = {}
ambush_left = {}
boss2_pulled = false
boss2_add_came = false
boss2_dead = false
boss3_pulled = false
boss3_dead = false
dragon_came = false
dragon_actor = nil
dragon_gone = false
boss3_room_open = false
final_started = false
-- The last fight: how many 28656s, which set the pair's first marked, the helpers by spot.
vomits = 0
pair_first = nil
helpers = {}

function contains(list, id)
    for _, value in ipairs(list) do
        if value == id then
            return true
        end
    end
    return false
end

function after(seconds, fn)
    table.insert(later, { at = clock + seconds, fn = fn })
end

function sendVars(director)
    director:set_data(VAR_KEY, has_key and 1 or 0)
    director:set_data(VAR_PROGRESS, progress)
    director:set_data(VAR_CLEARED, progress | cleared)
end

function pulled(info, id)
    return info ~= nil and info[id] ~= nil and info[id].target ~= nil
end

function flat_distance(a, b)
    return math.sqrt((a.x - b.x) ^ 2 + (a.z - b.z) ^ 2)
end

function onSetup(director)
    director:set_sequence(1)
    sendVars(director)

    director:boss_room(BNPC_BOSS1, BOSS1_WALL, BOSS1_LINE, PLACE_BOSS1, { ADD_425 })
    director:boss_room(BNPC_BOSS2, BOSS2_WALL, BOSS2_LINE, PLACE_BOSS2)
    -- The dragon holds the third room with its boss: it opens when both have gone (`leave_boss_room` as it flies off.)
    director:boss_room(BNPC_BOSS3, BOSS3_WALL, BOSS3_LINE, PLACE_BOSS3)
    director:boss_room(BNPC_DRAGON, BOSS3_WALL, BOSS3_LINE, PLACE_BOSS3)
    -- The walls stand open until a pull (retail: spawned in state 7); the doors and the rooms' lines stand closed
    -- (state 0), so nobody walks through them before they open, bots and enemies included.
    for _, wall in ipairs({ BOSS1_WALL, BOSS2_WALL, BOSS3_WALL, FINAL_WALL }) do
        director:hide_eobj(wall)
    end
    for _, door in ipairs(DOORS) do
        director:show_eobj(door)
    end
    for _, line in ipairs({ BOSS1_LINE, BOSS2_LINE, BOSS3_LINE, FINAL_LINE }) do
        director:show_eobj(line)
    end

    for coffer, openings in pairs(LOOT) do
        director:treasure_loot(coffer, openings)
    end

    director:party_npc_yell("tank", LINE_START, LINE_START_AFTER)
end

-- The duty began (DutyCommence): the dragon by the entrance flies off.
function onCommence(director)
    director:delay(PERCHED_DRAGON_FLIES_AFTER)
    director:play_action_timeline(BNPC_PERCHED_DRAGON, TIMELINE_DRAGON_FLIES)
    director:delay(PERCHED_DRAGON_FLIES_AFTER + PERCHED_DRAGON_GONE_AFTER)
    director:despawn_bnpc(BNPC_PERCHED_DRAGON)
    director:delay(0)
end

-- The event NPC's talk: the key (once), and the first boss and its adds are in the pond.
function onContentNpc(director, actor_id, id, npc, results)
    if id ~= CONTENT_NPC_KEY or key_given then
        director:finish_gimmick(actor_id)
        return
    end
    key_given = true
    has_key = true
    progress = progress | PROGRESS_KEY
    director:log_message(LOG_OBTAINED, { ITEM_KEY })
    director:finish_gimmick(actor_id)
    sendVars(director)

    director:spawn_bnpc(BNPC_BOSS1)
    for _, add in ipairs(BOSS1_ADDS) do
        director:spawn_bnpc(add.layout, add.x, add.y, add.z, BOSS1_ADDS_ROTATION)
    end
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
        task.fn(director)
    end

    if not boss1_pulled and (pulled(info, BNPC_BOSS1) or (key_given and pulled(info, ADD_425))) then
        boss1_pulled = true
        boss1_pulled_at = clock
        director:set_bgm(BGM_BATTLE)
        director:show_eobj(EOBJ_SHORTCUT)
    end
    if boss1_pulled and not boss1_dead
        and clock - boss1_pulled_at >= BOSS1_FIRST_WAVE + boss1_waves * BOSS1_WAVE_EVERY then
        sendWave(director, BOSS1_WAVES[boss1_waves % #BOSS1_WAVES + 1])
        boss1_waves = boss1_waves + 1
    end

    local player = info ~= nil and info.player or nil
    if player ~= nil then
        for index, ambush in ipairs(AMBUSHES) do
            if boss1_dead and not ambushed[index] and flat_distance(player, ambush.at) <= ambush.within then
                springAmbush(director, index)
            end
        end
        if not final_started and progress & PROGRESS_DOOR ~= 0 and player.x > FINAL_TRIGGER.x
            and player.y > FINAL_TRIGGER.y and player.z < FINAL_TRIGGER.z then
            startFinal(director)
        end
    end

    if not boss2_pulled and pulled(info, BNPC_BOSS2) then
        boss2_pulled = true
        director:set_bgm(BGM_BATTLE)
    end
    if boss2_pulled and not boss2_add_came and health ~= nil and health[BNPC_BOSS2] ~= nil
        and health[BNPC_BOSS2] < BOSS2_ADD_BELOW then
        boss2_add_came = true
        director:spawn_bnpc(BNPC_BOSS2_ADD)
        director:spawn_bnpc(BNPC_GOBLIN_FIGHTER)
        director:set_hate(BNPC_BOSS2_ADD, BNPC_GOBLIN_FIGHTER, 300)
        director:set_hate(BNPC_GOBLIN_FIGHTER, BNPC_BOSS2_ADD, 300)
        after(YELL_FIGHTER_COMES_AFTER, function(d)
            d:yell(BNPC_GOBLIN_FIGHTER, YELL_FIGHTER_COMES)
        end)
    end

    if not boss3_pulled and pulled(info, BNPC_BOSS3) then
        boss3_pulled = true
        director:set_bgm(BGM_BATTLE)
    end
    if boss3_pulled and not dragon_came and not boss3_dead and health ~= nil and health[BNPC_BOSS3] ~= nil
        and health[BNPC_BOSS3] < DRAGON_BELOW then
        bringDragon(director)
    end
end

-- Three adds, after the line, going for the party.
function sendWave(director, wave)
    director:log_message(LOG_WAVE, {})
    for _, add in ipairs(wave) do
        local actor = director:spawn_bnpc(add[1], add[2], add[3], add[4], 0.0)
        director:set_hate(actor, nil, 300) -- the player
    end
end

function springAmbush(director, index)
    local ambush = AMBUSHES[index]
    ambushed[index] = true
    ambush_left[index] = #ambush.enemies
    director:yell(ambush.goblin, YELL_HELP)
    for _, id in ipairs(ambush.enemies) do
        director:spawn_bnpc(id)
        director:set_hate(id, ambush.goblin, 300)
    end
    if ambush.again ~= nil then
        after(ambush.again, function(d)
            d:yell(ambush.goblin, YELL_HELP_AGAIN)
        end)
    end
end

function ledgeCall(director)
    if boss2_pulled then
        return
    end
    director:yell(BNPC_GOBLIN_LEDGE, YELL_HELP)
    after(YELL_HELP_EVERY, ledgeCall)
end

function bringDragon(director)
    dragon_came = true
    dragon_actor = director:spawn_bnpc(BNPC_DRAGON)
    director:appear_bnpc(BNPC_DRAGON, 0, DRAGON_APPEARS_AFTER)
    director:delay(DRAGON_APPEARS_AFTER)
    director:actor_use_action(dragon_actor, ACTION_TOUCHDOWN)
    director:set_hate(dragon_actor, nil, 300) -- the player
    director:delay(DRAGON_LANDS_LOG_AFTER)
    director:log_message(LOG_DRAGON_LANDS, {})
    director:delay(0)
    after(DRAGON_UNTARGETABLE_AFTER, function(d)
        if dragon_gone then
            return
        end
        d:set_targetable(BNPC_DRAGON, false)
        d:hold_bnpc(BNPC_DRAGON)
        d:delay(DRAGON_FLIES_AFTER - DRAGON_UNTARGETABLE_AFTER)
        d:play_action_timeline(BNPC_DRAGON, TIMELINE_DRAGON_FLIES)
        d:delay(DRAGON_LEAVES_LOG_AFTER - DRAGON_UNTARGETABLE_AFTER)
        d:log_message(LOG_DRAGON_LEAVES, {})
        d:delay(0)
    end)
    after(DRAGON_GONE_AFTER, function(d)
        if dragon_gone then
            return
        end
        d:despawn_bnpc(BNPC_DRAGON)
        dragonGone(d)
    end)
end

-- The dragon has gone (flown off, or fallen): the third room opens if its boss has too.
function dragonGone(director)
    dragon_gone = true
    director:leave_boss_room(BNPC_DRAGON)
    if boss3_dead then
        openBoss3Room(director)
    end
end

function openBoss3Room(director)
    if boss3_room_open then
        return
    end
    boss3_room_open = true
    director:set_bgm(0)
    director:log_message(LOG_OPENED, { LOG_BOSS3_WAY_ON })
    director:hide_eobj(EOBJ_BOSS3_WAY_ON)
    director:map_effect(MAP_EFFECT_BOSS3, MAP_EFFECT_OPEN)
    director:spawn_treasure(COFFER_BOSS3)
    director:party_npc_yell("healer", LINE_BOSS3)
end

-- The bosses' and their adds' actions the director answers.
function onActorCast(director, layout_id, action_id, actor_id, x, y, z)
    if layout_id == BNPC_BOSS3 and action_id == ACTION_EFFLUVIUM then
        director:delay(BLOB_AFTER)
        local blob = director:spawn_bnpc(BNPC_BOSS3_BLOB, x, y, z, 0.0)
        director:set_hate(blob, nil, 300) -- the player
        director:delay(0)
    elseif layout_id == BNPC_GOBLIN_FIGHTER and action_id == ACTION_BRAVADO then
        director:yell(BNPC_GOBLIN_FIGHTER, YELL_BRAVADO)
        director:delay(YELL_BRAVADO_AFTER_DELAY)
        director:yell(BNPC_GOBLIN_FIGHTER, YELL_BRAVADO_AFTER)
        director:delay(0)
    elseif layout_id == BNPC_GOBLIN_FIGHTER and action_id == ACTION_BANG_TOSS then
        director:yell(BNPC_GOBLIN_FIGHTER, YELL_BANG_TOSS)
        director:delay(YELL_BANG_TOSS_AFTER_DELAY)
        director:yell(BNPC_GOBLIN_FIGHTER, YELL_BANG_TOSS_AFTER)
        director:delay(0)
    elseif layout_id == BNPC_FINAL and action_id == ACTION_TOXIC_VOMIT then
        vomits = vomits + 1
        local marked
        if vomits == 1 then
            marked = { 0, SET_A[1], SET_A[2], SET_A[3] }
        elseif vomits % 2 == 0 then
            pair_first = math.random(2) == 1 and SET_A or SET_B
            marked = { 0, pair_first[1], pair_first[2], pair_first[3] }
        else
            local other = pair_first == SET_A and SET_B or SET_A
            marked = { other[1], other[2], other[3] }
        end
        puddles(director, marked)
    end
end

-- The spots' first helpers mark them, then the others burst them, the map effects following.
function puddles(director, marked)
    for _, spot in ipairs(marked) do
        if helpers[spot] ~= nil then
            director:actor_use_action(helpers[spot].mark, ACTION_MARK)
        end
    end
    director:delay(BURST_AFTER)
    for _, spot in ipairs(marked) do
        director:map_effect(spot, MAP_EFFECT_MARKED.timeline, MAP_EFFECT_MARKED.state)
        if helpers[spot] ~= nil then
            director:actor_use_action(helpers[spot].burst, ACTION_BURST)
        end
    end
    director:delay(BURST_AFTER + BURST_LANDS_AFTER)
    for _, spot in ipairs(marked) do
        director:map_effect(spot, MAP_EFFECT_BURST.timeline, MAP_EFFECT_BURST.state)
    end
    director:delay(0)
end

function onActorDeath(director, bnpc_id, position)
    if bnpc_id == BNPC_BOSS1 then
        -- Its room opens (the boss room's), and the way on; the goblin and an enemy ahead; the second boss later.
        boss1_dead = true
        progress = progress | PROGRESS_BOSS1
        sendVars(director)
        director:set_bgm(0)
        director:log_message(LOG_OPENED, { LOG_BOSS1_WAY_ON })
        director:hide_eobj(EOBJ_BOSS1_WAY_ON)
        director:map_effect(MAP_EFFECT_BOSS1, MAP_EFFECT_OPEN)
        director:spawn_treasure(COFFER_BOSS1)
        director:spawn_bnpc(BNPC_AFTER_BOSS1)
        director:spawn_bnpc(BNPC_GOBLIN_RUNNER)
        director:set_hate(BNPC_GOBLIN_RUNNER, BNPC_AFTER_BOSS1, 300)
        for _, delay in ipairs(YELL_RUNNER_GO_AFTER) do
            director:delay(delay)
            director:yell(BNPC_GOBLIN_RUNNER, YELL_RUNNER_GO)
        end
        director:delay(BOSS2_COMES_AFTER)
        director:spawn_bnpc(BNPC_BOSS2)
        director:delay(0)
        director:party_npc_yell("tank", LINE_BOSS1, LINE_BOSS1_AFTER)
        if not ledge_calls then
            ledge_calls = true
            after(YELL_HELP_FIRST, ledgeCall)
        end
    elseif bnpc_id == BNPC_AFTER_BOSS1 and not runner_target_dead then
        runner_target_dead = true
        director:spawn_treasure(COFFER_AFTER_BOSS1)
        director:delay(YELL_CHEER_AFTER)
        director:yell(BNPC_GOBLIN_RUNNER, YELL_CHEER)
        director:play_action_timeline(BNPC_GOBLIN_RUNNER, TIMELINE_CHEER)
        director:delay(0)
    elseif bnpc_id == BNPC_BOSS2_ADD then
        director:restart_timeline(BNPC_BOSS2, BOSS2_ADD_GONE)
    elseif bnpc_id == BNPC_BOSS2 then
        boss2_dead = true
        progress = progress | PROGRESS_BOSS2
        sendVars(director)
        director:set_bgm(0)
        director:log_message(LOG_OPENED, { LOG_BOSS2_WAY_ON })
        director:hide_eobj(EOBJ_BOSS2_WAY_ON)
        director:map_effect(MAP_EFFECT_BOSS2, MAP_EFFECT_OPEN)
        director:spawn_treasure(COFFER_BOSS2)
        director:party_npc_yell("healer", LINE_BOSS2)
        director:spawn_bnpc(BNPC_BOSS3)
        if boss2_add_came then
            director:delay(YELL_FIGHTER_DONE_AFTER)
            director:yell(BNPC_GOBLIN_FIGHTER, YELL_FIGHTER_DONE)
            director:delay(0)
        end
    elseif bnpc_id == BNPC_BOSS3 then
        boss3_dead = true
        if not dragon_came then
            -- Burst down before the dragon came: there's no one else to wait for.
            dragon_gone = true
            director:leave_boss_room(BNPC_DRAGON)
        end
        if dragon_gone then
            openBoss3Room(director)
        end
    elseif bnpc_id == BNPC_DRAGON and not dragon_gone then
        dragonGone(director)
    elseif bnpc_id == BNPC_FINAL then
        cleared = CLEARED_FINAL
        sendVars(director)
        for _, spot in pairs(helpers) do
            director:despawn_bnpc(spot.mark)
            director:despawn_bnpc(spot.burst)
        end
        helpers = {}
        director:spawn_treasure(COFFER_FINAL)
        director:play_cutscene(CUTSCENE_END, CUTSCENE_END_PARAMS)
        director:complete_duty()
        director:delay(AFTER_CLEAR_NPCS_AFTER)
        for _, id in ipairs(AFTER_CLEAR_NPCS) do
            director:spawn_bnpc(id)
        end
        director:delay(0)
    else
        for index, ambush in ipairs(AMBUSHES) do
            if ambush_left[index] ~= nil and ambush_left[index] > 0 and contains(ambush.enemies, bnpc_id) then
                ambush_left[index] = ambush_left[index] - 1
                if ambush_left[index] == 0 then
                    if ambush.coffer ~= nil then
                        director:spawn_treasure(ambush.coffer)
                    end
                    if ambush.cheer ~= nil then
                        director:delay(ambush.cheer)
                        director:yell(ambush.goblin, YELL_CHEER)
                        director:play_action_timeline(ambush.goblin, TIMELINE_CHEER)
                        director:delay(0)
                    end
                end
            end
        end
    end
end

-- The last boss's terrace: its cutscene, the boss and its helpers (held: they act when the director says).
function startFinal(director)
    if final_started then
        return
    end
    final_started = true
    director:play_cutscene(CUTSCENE_FINAL)
    director:spawn_boss(BNPC_FINAL, FINAL_WALL, FINAL_LINE, PLACE_FINAL)
    for spot, at in pairs(SPOTS) do
        local mark = director:spawn_bnpc(HELPER, at.x, at.y, at.z, at.rotation)
        local burst = director:spawn_bnpc(HELPER, at.x, at.y, at.z, at.rotation)
        director:hold_bnpc(mark)
        director:hold_bnpc(burst)
        helpers[spot] = { mark = mark, burst = burst }
    end
    director:party_npc_yell("ranged", LINE_FINAL, LINE_FINAL_AFTER)
end

function onGimmickAccessor(director, actor_id, id, params)
    if id == EOBJ_POND_DOOR or id == EOBJ_TERRACE_DOOR then
        director:event_action(EVENT_ACTION_OPEN, actor_id, id)
        return
    end

    director:finish_gimmick(actor_id)
end

function isGimmickFunctional(id)
    if id == EOBJ_POND_DOOR then
        return has_key
    elseif id == EOBJ_TERRACE_DOOR then
        return boss3_room_open
    end

    return true
end

-- A door opened: its line, the door gone, the variables.
function onEventActionCast(director, actor_id, target)
    director:finish_gimmick(actor_id)

    if target == EOBJ_POND_DOOR then
        if not has_key then
            return
        end
        has_key = false
        director:log_message(LOG_OPENED, { LOG_POND_DOOR })
        director:map_effect(MAP_EFFECT_POND_DOOR, MAP_EFFECT_OPEN)
    elseif target == EOBJ_TERRACE_DOOR then
        if not boss3_room_open then
            return
        end
        progress = progress | PROGRESS_DOOR
        director:log_message(LOG_OPENED, { LOG_TERRACE_DOOR })
    else
        return
    end

    director:hide_eobj(target)
    sendVars(director)
end

function getDebugShortcut(id)
    return 0
end
