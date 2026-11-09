-- The Navel (Titan)
--
-- From Bob's Chronofoil captures of 2026-11-06, during Lord of Crags (quest 857, which unlocks it): 650dbfd8 visit 0.1
-- (Duty Support, the player a level 36 arcanist) and 0fd8bce0 visit 0.3 and 48aaf6fe visit 0.2 (the Duty Finder), all
-- three won at the first attempt; `questkit chronofoil --battle` and the visits' packets, numbers only, with Cactbot's
-- titan-nm timeline for what the runs didn't show (docs/src/fork/dungeons.md).
--
-- Titan stands at the arena's north (0, 0, -16) facing south. From his pull he fights by his timeline
-- (TitanNavel_246: Tumult, Rock Buster). Below 80 % (once he isn't casting) he leaps: untargetable with ActionTimeline
-- 140, 2.1 s on his line 3804, put in the middle and Geocrush (651) round it; 9.0 s after the leap targetable again
-- and the next phase (Landslide joins in). Below 50 % he leaps again, and 9.1 s after it the Heart of the Mountain
-- (BNpcBase 1507) comes at his chest with his line 3801, ActorControl 31 and 30 on him and ActionTimeline 1439: he
-- stays untargetable and fights on (Rock Throw once, Landslide, Tumult, Rock Buster) while the party breaks it. As the
-- Heart falls he leaps to the middle again and, 7.3 s after it, his line 3802 and Earthen Fury (652) on everyone; 12.2
-- s after it targetable again with his line 3803, and his last phase: Weight of the Land, Rock Throw and the rest.
-- Rock Throw gaols whom it's aimed at (status 292 and a granite gaol round them until it's broken.) At 0 his last line
-- (3805), the closing cutscene and the duty complete. Everyone down is a party wipe (server/wipe.rs): it begins again.

-- Which way players face as they come in and after a wipe (the InstanceContent has no entrance rect): retail's
-- -3.142, toward Titan, all three runs.
ENTRANCE_ROTATION = -3.1416

TITAN = 4039257
-- The Heart of the Mountain: retail's had no layout ID (the director spawned it); here the fork's 99001507.
HEART = 99001507
GAOL = 4249552
HELPER = 4145900
-- Retail spawned four helpers with Titan (all one layout ID); two cast at once.
HELPER_COUNT = 4

NAME_TITAN = 1801
TALK_MS = 5000
TALK_PULL = 3800
TALK_GEOCRUSH = 3804
TALK_HEART = 3801
TALK_EARTHEN_FURY = 3802
TALK_FURY_OVER = 3803
TALK_DEFEATED = 3805

-- The Duty Support members' lines (NpcYell, the Duty Support run): the Marauder's 14361 at the pull, the Conjurer's
-- 14366 3.0 s after the Heart fell. (The Thaumaturge's 13975 three times and the Marauder's 13955 once were their
-- chatter, `party_npc`.)
LINE_PULL = 14361
LINE_HEART_BROKEN = 14366
LINE_HEART_BROKEN_AFTER = 3.0

-- His phases: the timeline's points `after` these (restart_timeline): his lines' text IDs.
PHASE_2 = TALK_GEOCRUSH
PHASE_HEART = TALK_HEART
PHASE_NONE = TALK_EARTHEN_FURY -- no points: nothing of his own while he leaps
PHASE_4 = TALK_FURY_OVER

-- Below these he leaps (retail: untargetable at 79.3, 75.5 and 79.9 %, then 49.4, 46.6 and 49.4 %), not while he
-- casts: 0.9-1.1 s after his health fell below (1.5 and 1.3 s for the second), or 2.1 s after a Landslide's hit when
-- one was being cast (the Duty Support run's second: 4.3 s.) Cactbot's 85 and 55 % aren't what the runs show.
LEAP_AT = { 80, 50 }
LEAP_AFTER_CROSSING = 1.0
LEAP_AFTER_CAST = 2.4
CAST_SECONDS = { [644] = 2.7, [650] = 2.7, [651] = 4.2 }

-- The leap (all three runs, from his going untargetable): ActionTimeline 140 at once; 2.1 s on his line, ActorSetPos
-- to the middle facing south and Geocrush (4.2 s, from the middle); 9.0 s on targetable again (the first), or 9.1 s on
-- the Heart (the second).
LEAP_TIMELINE = 140
GEOCRUSH_AFTER = 2.1
TARGETABLE_AFTER = 9.0
HEART_AFTER = 9.1
ACTION_GEOCRUSH = 651
MIDDLE = { x = 0.0, y = 0.0, z = 0.0 }
-- Where the Duty Support members went for both Geocrushes (Bob's capture 650dbfd8, 3-4 s into each cast): the arena's
-- edge, inside it but where it hurts least (it hurts less the farther out: 971-1017 at 4.7 yalms, 347-480 at 19-22.)
-- The tank (0, -19), the healer (0, 19-21), the dps (19-21, 0); a second dps takes the dps spot.
GEOCRUSH_SPOTS = { tank = { 0.0, -19.0 }, healer = { 0.0, 19.0 }, dps = { 19.0, 0.0 } }

-- The director's variables (DirectorVars, sequence 1 throughout, all three runs): 32, 0 at the start; 32, 1 with
-- the first Geocrush's line and 33, 0 4.9 s after it; 33, 4 with the second's and 39, 0 4.9 s after.
VARS_START = { 32, 0 }
VARS_AT_LINE = { { 32, 1 }, { 33, 4 } }
VARS_AFTER_LINE = { { 33, 0 }, { 39, 0 } }
VARS_AFTER = 4.9

-- The Heart (5139 HP), 2.3 yalms before him the way he faces, wherever he goes, a part of him (`spawn_bnpc_part`);
-- as it comes, on him ActorControl 31 (0, 0, 0, 1), 30 (0, 1, the Heart, 0) and ActionTimeline 1439; as it falls 30
-- (0, 2, the Heart, 1).
-- Kawari doesn't know what 30 and 31 do (the Thousand Maws of Toto-Rak's last boss had the same for its tail.)
HEART_DISTANCE = 2.3
HEART_TIMELINE = 1439
CONTROL_PARTS = 31
CONTROL_LINK = 30
-- Cactbot: Earthen Fury ("Enrage") 73.6 s after the second Geocrush began if the Heart still stands (it fell in 24.7,
-- 50.0 and 27 s in the runs); Kawari takes the Heart away then, and Earthen Fury comes all the same.
HEART_ENRAGE_AFTER = 66.6

-- Earthen Fury, from the Heart's fall (two runs to a tenth): ActionTimeline 140 at 3.2 s, ActorSetPos to the middle and
-- 141 at 5.3, his line 3802 and Earthen Fury (instant) at 7.3, ActorControl 31 (0, 0, 16, 1) at 9.7, and at 12.2 31
-- (0, 0, 16, 0), targetable, his line 3803 and his last phase.
FURY_TIMELINE_1 = 140
FURY_TIMELINE_1_AFTER = 3.2
FURY_TIMELINE_2 = 141
FURY_TO_MIDDLE_AFTER = 5.3
ACTION_EARTHEN_FURY = 652
FURY_AFTER = 7.3
FURY_PARTS_AFTER = 9.7
FURY_OVER_AFTER = 12.2

-- Rock Throw (645, instant, on someone other than his target): 2.8 s on status 292 on them, 4.0 s on the granite gaol
-- (571 HP) spawned hidden where they stand, brought into view 0.8 s later (Appear 142); as it breaks the status goes
-- (all six throws in the three runs.)
ACTION_ROCK_THROW = 645
STATUS_GAOL = 292
GAOLED_AFTER = 2.8
GAOL_AFTER = 4.0
GAOL_APPEAR_TIMELINE = 142
GAOL_APPEAR_DELAY = 0.8

-- Weight of the Land (644, cast on himself): two helpers' 973 (3.2 s, round them) under two of the party other than
-- the one he fights, as it begins (all eight casts; never his target.)
ACTION_WEIGHT = 644
ACTION_WEIGHT_HELPER = 973
WEIGHT_MARKS = 2
HELPER_Y = -0.02

-- The closing cutscene (scene 3: 245, then 1, 18, 0, 0, as retail sent it.)
CUTSCENE_END = 245
CUTSCENE_END_PARAMS = { 1, 18, 0, 0 }

-- onUpdate comes every server tick.
TICK = 0.5

titan = nil
heart = nil
helpers = {}
phase = 1
leaps = 0
leap_due = nil
pulled = false
attempts = 1
waiting_for_new_titan = false
-- The update count, when his cast ends (in updates), his last spot.
ticks = 0
cast_until = 0
titan_at = nil
titan_target = nil
-- Whom each granite gaol holds: { gaol = actor ID, prisoner = actor ID, at = { x, y, z } }
gaols = {}
-- The party (the player and their party's NPCs) as at the last update: { id, x, y, z }
party = {}
-- Things to do later: { at = an update count, fn = function(director) }
later = {}

function after(seconds, fn)
    table.insert(later, { at = ticks + math.max(1, math.ceil(seconds / TICK)), fn = fn })
end

function setVars(director, vars)
    for index, value in ipairs(vars) do
        director:set_data(index - 1, value)
    end
end

function bringTitan(director)
    titan = director:spawn_bnpc(TITAN)
    helpers = {}
    for i = 1, HELPER_COUNT do
        helpers[i] = director:spawn_bnpc(HELPER)
        director:hold_bnpc(helpers[i])
    end
end

function reset()
    phase = 1
    leaps = 0
    leap_due = nil
    pulled = false
    heart = nil
    cast_until = 0
    gaols = {}
    later = {}
end

function onSetup(director)
    director:set_sequence(1)
    setVars(director, VARS_START)
    reset()
    bringTitan(director)
    -- Inside Geocrush by design: there's no getting out of it.
    director:party_spots(ACTION_GEOCRUSH, MIDDLE.x, MIDDLE.z, GEOCRUSH_SPOTS, nil, nil, true)
end

function talk(director, text)
    director:battle_talk(TITAN, NAME_TITAN, text, 0, TALK_MS)
end

-- The leap and Geocrush: the first (`which` 1) ends with him targetable and his second phase, the second with the
-- Heart.
function leap(director, which)
    leaps = which
    phase = "leaping"
    director:restart_timeline(TITAN, PHASE_NONE)
    director:set_targetable(TITAN, false)
    director:set_invulnerable(TITAN, true)
    director:busy_bnpc(TITAN, which == 1 and TARGETABLE_AFTER or HEART_AFTER)
    director:play_action_timeline(TITAN, LEAP_TIMELINE)
    after(GEOCRUSH_AFTER, function(d)
        setVars(d, VARS_AT_LINE[which])
    end)
    after(GEOCRUSH_AFTER + VARS_AFTER, function(d)
        setVars(d, VARS_AFTER_LINE[which])
    end)
    director:delay(GEOCRUSH_AFTER)
    talk(director, TALK_GEOCRUSH)
    director:warp_bnpc(TITAN, MIDDLE.x, MIDDLE.y, MIDDLE.z, 0)
    director:actor_use_action(titan, ACTION_GEOCRUSH)
    if which == 1 then
        director:delay(TARGETABLE_AFTER)
        director:set_targetable(TITAN, true)
        director:set_invulnerable(TITAN, false)
        director:restart_timeline(TITAN, PHASE_2)
        director:delay(0)
        after(TARGETABLE_AFTER, function(d)
            phase = 2
        end)
    else
        director:delay(HEART_AFTER)
        bringHeart(director)
        director:delay(0)
    end
end

-- The Heart of the Mountain in his chest: spawned as a part of him (`spawn_bnpc_part`: the client draws and targets it
-- where his model has it), kept before him as he turns. Retail sent 31 and 30 on him and his ActionTimeline 1439 just
-- before the Heart's SpawnNpc (30 naming the Heart before it was out); here 30 and the timeline follow the spawn, as
-- the Heart's actor ID comes with it.
function bringHeart(director)
    phase = "heart"
    talk(director, TALK_HEART)
    director:bnpc_control(TITAN, CONTROL_PARTS, { 0, 0, 0, 1 })
    heart = director:spawn_bnpc_part(HEART, TITAN, HEART_DISTANCE)
    director:hold_bnpc(heart)
    director:set_hate(HEART, nil, 300) -- the player
    director:bnpc_control(TITAN, CONTROL_LINK, { 0, 1, heart, 0 })
    director:play_action_timeline(TITAN, HEART_TIMELINE)
    director:restart_timeline(TITAN, PHASE_HEART)
    local this_heart = heart
    after(HEART_AFTER + HEART_ENRAGE_AFTER, function(d)
        if heart == this_heart and phase == "heart" then
            d:despawn_bnpc(HEART)
            earthenFury(d, this_heart)
        end
    end)
end

-- The Heart fallen (or its time up): Earthen Fury from the middle, then his last phase.
function earthenFury(director, which_heart)
    phase = "fury"
    heart = nil
    director:bnpc_control(TITAN, CONTROL_LINK, { 0, 2, which_heart, 1 })
    director:restart_timeline(TITAN, PHASE_NONE)
    director:busy_bnpc(TITAN, FURY_OVER_AFTER)
    director:party_npc_yell("healer", LINE_HEART_BROKEN, LINE_HEART_BROKEN_AFTER)
    director:delay(FURY_TIMELINE_1_AFTER)
    director:play_action_timeline(TITAN, FURY_TIMELINE_1)
    director:delay(FURY_TO_MIDDLE_AFTER)
    director:warp_bnpc(TITAN, MIDDLE.x, MIDDLE.y, MIDDLE.z, 0)
    director:play_action_timeline(TITAN, FURY_TIMELINE_2)
    director:delay(FURY_AFTER)
    talk(director, TALK_EARTHEN_FURY)
    director:actor_use_action(titan, ACTION_EARTHEN_FURY)
    director:delay(FURY_PARTS_AFTER)
    director:bnpc_control(TITAN, CONTROL_PARTS, { 0, 0, 16, 1 })
    director:delay(FURY_OVER_AFTER)
    director:bnpc_control(TITAN, CONTROL_PARTS, { 0, 0, 16, 0 })
    director:set_targetable(TITAN, true)
    director:set_invulnerable(TITAN, false)
    talk(director, TALK_FURY_OVER)
    director:restart_timeline(TITAN, PHASE_4)
    director:delay(0)
    after(FURY_OVER_AFTER, function(d)
        phase = 4
    end)
end

-- Where someone of the party stands now, by actor ID.
function whereIs(id)
    for _, member in ipairs(party) do
        if member.id == id then
            return member
        end
    end
    return nil
end

-- Rock Throw's target gaoled; Weight of the Land's marks under two of the party.
function onActorCast(director, layout_id, action_id, actor_id, x, y, z, target_id)
    if layout_id ~= TITAN then
        return
    end
    if CAST_SECONDS[action_id] ~= nil then
        cast_until = ticks + math.ceil(CAST_SECONDS[action_id] / TICK)
    end
    if action_id == ACTION_ROCK_THROW and target_id ~= nil then
        local prisoner = target_id
        after(GAOLED_AFTER, function(d)
            d:gain_effect(prisoner, STATUS_GAOL, 0, 0)
        end)
        after(GAOL_AFTER, function(d)
            local spot = whereIs(prisoner) or { x = x, y = y, z = z }
            if spot.x == nil then
                return
            end
            local gaol = d:spawn_bnpc(GAOL, spot.x, spot.y, spot.z, 0)
            d:hold_bnpc(gaol)
            d:appear_bnpc(GAOL, GAOL_APPEAR_TIMELINE, GAOL_APPEAR_DELAY)
            d:set_hate(GAOL, nil, 300) -- the player
            table.insert(gaols, { gaol = gaol, prisoner = prisoner, at = spot })
        end)
    elseif action_id == ACTION_WEIGHT and #helpers > 0 then
        local his_target = titan_target
        local others = {}
        for _, member in ipairs(party) do
            if member.id ~= his_target then
                table.insert(others, member)
            end
        end
        for i = 1, math.min(WEIGHT_MARKS, #others) do
            local pick = table.remove(others, math.random(#others))
            local helper = helpers[i]
            director:warp_bnpc(helper, pick.x, HELPER_Y, pick.z, 0)
            director:actor_use_action(helper, ACTION_WEIGHT_HELPER)
        end
    end
end

function onUpdate(director, health, player_health, info)
    ticks = ticks + 1
    local due = {}
    local waiting = {}
    for _, entry in ipairs(later) do
        if entry.at <= ticks then
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
    party = {}
    for key, actor in pairs(info) do
        if type(key) == "string" and (key:match("^player%d+$") or key:match("^party%d+$")) then
            table.insert(party, { id = actor.id, x = actor.x, y = actor.y, z = actor.z })
        end
    end
    if info[TITAN] ~= nil then
        titan_at = { x = info[TITAN].x, y = info[TITAN].y, z = info[TITAN].z }
        titan_target = info[TITAN].target_id
    end

    local hp = health[TITAN]
    if hp == nil then
        return
    end
    -- Just after a wipe the old Titan's health is still there for an update (as the Bowl of Embers' Ifrit's): the new
    -- one comes at full health.
    if waiting_for_new_titan then
        if hp < 100 then
            return
        end
        waiting_for_new_titan = false
    end
    if not pulled and info[TITAN] ~= nil and info[TITAN].target ~= nil then
        pulled = true
        talk(director, TALK_PULL)
        if attempts == 1 then
            director:party_npc_yell("tank", LINE_PULL)
        end
    end
    -- His leaps, not while he casts.
    for which = 1, 2 do
        if phase == which and leap_due == nil and hp < LEAP_AT[which] then
            leap_due = math.max(ticks + math.ceil(LEAP_AFTER_CROSSING / TICK),
                cast_until + math.ceil(LEAP_AFTER_CAST / TICK))
        end
    end
    if leap_due ~= nil and ticks >= leap_due and (phase == 1 or phase == 2) then
        leap_due = nil
        leap(director, phase)
    end
end

-- Everyone gaoled is free, and the gaols go.
function freeGaoled(director)
    for _, held in ipairs(gaols) do
        director:lose_effect(held.prisoner, STATUS_GAOL)
        director:despawn_bnpc(held.gaol)
    end
    gaols = {}
end

function onActorDeath(director, bnpc_id, position)
    if bnpc_id == HEART and phase == "heart" then
        earthenFury(director, heart)
    elseif bnpc_id == GAOL then
        -- The gaol that broke: the one nearest where it fell (there was never more than one at once on retail.)
        local nearest, best = nil, nil
        for index, held in ipairs(gaols) do
            local dx, dz = held.at.x - position.x, held.at.z - position.z
            local distance = dx * dx + dz * dz
            if best == nil or distance < best then
                nearest, best = index, distance
            end
        end
        if nearest ~= nil then
            director:lose_effect(gaols[nearest].prisoner, STATUS_GAOL)
            table.remove(gaols, nearest)
        end
    elseif bnpc_id == TITAN then
        talk(director, TALK_DEFEATED)
        freeGaoled(director)
        for _, helper in ipairs(helpers) do
            director:despawn_bnpc(helper)
        end
        helpers = {}
        titan = nil
        director:play_cutscene(CUTSCENE_END, CUTSCENE_END_PARAMS)
        director:complete_duty()
    end
end

-- Everyone down: as they're sent back, Titan, his Heart, the gaols and his helpers go, and come back as at the start.
function onPartyWipe(director)
    freeGaoled(director)
    director:despawn_bnpc(TITAN)
    director:despawn_bnpc(HEART)
    for _, helper in ipairs(helpers) do
        director:despawn_bnpc(helper)
    end
    attempts = attempts + 1
    reset()
    setVars(director, VARS_START)
    waiting_for_new_titan = true
    bringTitan(director)
end

-- No shortcuts in a trial.
function getDebugShortcut(id)
    return 0
end
