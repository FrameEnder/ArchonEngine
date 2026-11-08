-- The Howling Eye (Garuda)
--
-- From Bob's Chronofoil captures of 2026-11-08, during Lady of the Vortex (quest 519, which unlocks it): 6a5244ec
-- visit 0.6 (Duty Support, the player a level 44 conjurer synced to 44), 09ecf8e7 visit 0.2 and 6c495fc4 visit 0.4
-- (the Duty Finder), all three won at the first attempt; `questkit chronofoil --battle` and the visits' packets,
-- numbers only (docs/src/fork/dungeons.md). Cactbot has no timeline for it.
--
-- Garuda starts at the arena's north (1.14, -1.95, -18.0) and fights by her timeline (TrialGaruda_239) from her pull.
-- Each of her big moves begins with her vanishing (untargetable, ActionTimeline 1415) and, 3.6 s later, coming back
-- (targetable, 1416) where it's cast from, the cast 0.7 s after:
--   below 80 %: Mistral Shriek (667, 3.7 s) from the north, which the four monoliths (BNpcBase 237) take for whoever
--               hides behind one (action_areas.json's `sheltered_by`); 13.0 s after it began her 1412 and six razor
--               plumes (BNpcBase 238) where she stands, two at a time a second apart, each going for a monolith; and
--               again 64.8 s after it vanished, while she's above 50 % (the Duty Support run).
--   below 50 %: her line 3701, 1.0 s on she vanishes; back, 1412 0.7 s on and eight razor plumes at the eight spots
--               round the arena, two every 2 s from 1.3 s on.
--   below 40 %: back in the middle, her line 3702 and Mistral Song (662, 3.7 s, everyone); her line 3703 11.0 s
--               after it began, and from 16.1 s after it her 1411 and Feather Rain (664, her helper's, every 5.2 s to the
--               end); 33.0 s after it began Aerial Blast (660, 3.7 s) from the middle; 11.4 s after that 1412 and eight
--               satin plumes (BNpcBase 688) at the eight spots, a second apart, each going for the player; 51.1 s after
--               Aerial Blast began 661 (3.7 s) from the north (the Duty Support run, near its end; what comes after
--               isn't recorded.)
-- Plumes left standing blow (665) 24.7 s after they came and fall 2.3 s later. Each time she's back her timeline
-- starts again (anchor 1416). Duty Support's members gather behind the monolith at (-8, -4) for Mistral Shriek, the
-- Conjurer says 14381 at the pull and the Thaumaturge 14382 as the first Shriek begins. At 0 her last line (3705),
-- the closing cutscene and the duty complete. Everyone down is a party wipe (server/wipe.rs): it begins again.

-- Which way players face as they come in and after a wipe: retail's -3.14, toward her, all three runs.
ENTRANCE_ROTATION = -3.1416

GARUDA = 4246177
HELPER = 4246419
MONOLITHS = { 4246138, 4246139, 4246140, 4246141 }
RAZOR_PLUME = 4246290
SATIN_PLUME = 4246291

NAME_GARUDA = 1644
TALK_MS = 5000
TALK_PULL = 3700
TALK_HALF = 3701
TALK_SONG = 3702
TALK_AFTER_SONG = 3703
TALK_DEFEATED = 3705
TALK_AFTER_SONG_AFTER = 11.0

-- Duty Support's lines (NpcYell, the Duty Support run): the Conjurer's at the pull, the Thaumaturge's as the first
-- Mistral Shriek began. (The Marauder's 13955 and the Thaumaturge's 13978 and 13985 were their chatter, `party_npc`.)
LINE_PULL = 14381
LINE_SHRIEK = 14382

-- Her ActionTimelines (ActorControl 407 from her, all three runs): vanishing, coming back, the plumes, Feather Rain.
TIMELINE_VANISH = 1415
TIMELINE_BACK = 1416
TIMELINE_PLUMES = 1412
TIMELINE_FEATHER_RAIN = 1411
-- Her timeline's anchors: its points `after` 1416 start as she can act again; 1415 has none (nothing of her own
-- while she's away.)
PHASE_READY = TIMELINE_BACK
PHASE_NONE = TIMELINE_VANISH

-- Away 3.6 s (all twelve times), the cast 0.7 s after she's back (0.6-0.8).
AWAY = 3.6
CAST_AFTER_BACK = 0.7
NORTH = { x = -0.02, y = -1.85, z = -21.71, rotation = 0.0 }
MIDDLE = { x = 0.02, y = -1.82, z = -0.08, rotation = 0.0 }

ACTION_SHRIEK = 667
ACTION_SONG = 662
ACTION_AERIAL_BLAST = 660
ACTION_LAST = 661
ACTION_FEATHER_RAIN = 664
ACTION_PLUME_BLAST = 665
CAST_SECONDS = { [659] = 2.2, [660] = 3.7, [661] = 3.7, [662] = 3.7, [667] = 3.7 }

-- When she vanishes: her health below these, not while she casts (retail: 1.7-3.4 s after it fell below 80 %,
-- 0.6-3.0 s after 40 %; 2.4 s after a cast's end at the soonest.)
SHRIEK_BELOW = 80
HALF_BELOW = 50
SONG_BELOW = 40
SHRIEK_AFTER_CROSSING = 2.0
SONG_AFTER_CROSSING = 0.7
VANISH_AFTER_CAST = 2.4
SHRIEK_AGAIN_AFTER = 64.8
HALF_VANISH_AFTER_LINE = 1.0

-- The plumes. The first wave where she stands, 13.0 s after Mistral Shriek began: 1412, then pairs 0.4, 1.4 and 2.4 s
-- after it. The second at the eight spots, 1412 0.7 s after she's back, pairs 1.3, 3.3, 5.3 and 7.3 s after it. The
-- satin plumes 11.4 s after Aerial Blast began: 1412, then one a second from 1.0 s on, one at each spot.
PLUMES_AFTER_SHRIEK = 13.0
FIRST_WAVE = { 0.4, 1.4, 2.4 }
PLUMES_AFTER_BACK = 0.7
SECOND_WAVE = { 1.3, 3.3, 5.3, 7.3 }
SATIN_AFTER_BLAST = 11.4
SATIN_EVERY = 1.0
PLUME_FALLS_AFTER = 2.3
SPOTS = {
    { x = -13.1, y = -1.9, z = -14.6, rotation = 0.61 },
    { x = 14.8, y = -1.9, z = -12.8, rotation = -0.84 },
    { x = 1.2, y = -2.1, z = -19.5, rotation = 0.0 },
    { x = -2.6, y = -2.0, z = 19.4, rotation = 3.1 },
    { x = 22.3, y = -2.0, z = -0.1, rotation = -1.44 },
    { x = -19.5, y = -2.0, z = -2.3, rotation = 1.42 },
    { x = -19.1, y = -1.9, z = 10.1, rotation = 2.12 },
    { x = 12.4, y = -1.8, z = 15.2, rotation = -2.5 },
}
PLUME_ENMITY = 300

-- After Mistral Song began (all three runs to a tenth or two): Feather Rain from 16.1 s, every 5.2 s; Aerial Blast's
-- vanishing at 33.0 s; 661's vanishing 51.1 s after Aerial Blast began (one run.)
FEATHER_RAIN_AFTER_SONG = 16.1
FEATHER_RAIN_EVERY = 5.2
BLAST_AFTER_SONG = 33.0
LAST_AFTER_BLAST = 51.1

-- Mistral Shriek: Duty Support's members gathered behind the monolith at (-6.37, -6.36), at (-8, -4) (all three
-- Shrieks; Bob's capture 6a5244ec.)
SHRIEK_SHELTER = { x = -8.0, z = -4.0 }

-- The closing cutscene (scene 3: 253, then 1, 18, 0, 0, as retail sent it.)
CUTSCENE_END = 253
CUTSCENE_END_PARAMS = { 1, 18, 0, 0 }

TICK = 0.5

garuda = nil
helper = nil
monoliths = {}
plumes = {}
phase = 1
vanish_due = nil
next_vanish = nil
pulled = false
attempts = 1
waiting_for_new_garuda = false
away = false
ticks = 0
cast_until = 0
shrieks = 0
garuda_at = nil
feather_rain = false
later = {}

function after(seconds, fn)
    table.insert(later, { at = ticks + math.max(1, math.ceil(seconds / TICK)), fn = fn })
end

function talk(director, text)
    director:battle_talk(GARUDA, NAME_GARUDA, text, 0, TALK_MS)
end

function bringGaruda(director)
    garuda = director:spawn_bnpc(GARUDA)
    helper = director:spawn_bnpc(HELPER)
    director:hold_bnpc(helper)
end

function onSetup(director)
    director:set_sequence(1)
    for _, layout in ipairs(MONOLITHS) do
        monoliths[layout] = director:spawn_bnpc(layout)
        director:hold_bnpc(layout)
    end
    local behind = { SHRIEK_SHELTER.x, SHRIEK_SHELTER.z }
    director:party_spots(ACTION_SHRIEK, NORTH.x, NORTH.z, { tank = behind, healer = behind, dps = behind })
    reset()
    bringGaruda(director)
end

function reset()
    phase = 1
    vanish_due = nil
    next_vanish = nil
    pulled = false
    away = false
    cast_until = 0
    shrieks = 0
    feather_rain = false
    plumes = {}
    later = {}
end

-- She vanishes, and 3.6 s later comes back at `spot` (where she was if nil), casting `action` 0.7 s after; `back` (if
-- given) runs as she's back, `cast` as the cast begins.
function vanish(director, spot, action, back, cast)
    away = true
    director:restart_timeline(GARUDA, PHASE_NONE)
    director:set_targetable(GARUDA, false)
    director:set_invulnerable(GARUDA, true)
    local cast_seconds = action ~= nil and (CAST_AFTER_BACK + CAST_SECONDS[action]) or 0
    director:busy_bnpc(GARUDA, AWAY + cast_seconds)
    director:play_action_timeline(GARUDA, TIMELINE_VANISH)
    after(AWAY, function(d)
        if spot ~= nil then
            d:warp_bnpc(GARUDA, spot.x, spot.y, spot.z, spot.rotation)
        end
        d:set_targetable(GARUDA, true)
        d:set_invulnerable(GARUDA, false)
        d:play_action_timeline(GARUDA, TIMELINE_BACK)
        if back ~= nil then
            back(d)
        end
        if action == nil then
            away = false
            d:restart_timeline(GARUDA, PHASE_READY)
            return
        end
        after(CAST_AFTER_BACK, function(d2)
            if cast ~= nil then
                cast(d2)
            end
            d2:actor_use_action(garuda, action)
            cast_until = ticks + math.ceil(CAST_SECONDS[action] / TICK)
            after(CAST_SECONDS[action], function(d3)
                away = false
                d3:restart_timeline(GARUDA, PHASE_READY)
            end)
        end)
    end)
end

-- A plume at `spot`, going for the nearest monolith (razor) or the player (satin.)
function bringPlume(director, layout, spot)
    local id = director:spawn_bnpc(layout, spot.x, spot.y, spot.z, spot.rotation or 0.0)
    table.insert(plumes, id)
    if layout == SATIN_PLUME then
        director:set_hate(id, nil, PLUME_ENMITY)
        return
    end
    local nearest, best = nil, nil
    for _, rock in ipairs(MONOLITHS) do
        local at = ROCK_AT[rock]
        local d = (at.x - spot.x) ^ 2 + (at.z - spot.z) ^ 2
        if best == nil or d < best - 0.01 or (math.abs(d - best) <= 0.01 and math.random(2) == 1) then
            nearest, best = rock, d
        end
    end
    director:set_hate(id, nearest, PLUME_ENMITY)
end

-- Where the monoliths stand (the drop-in's spots.)
ROCK_AT = {
    [4246138] = { x = -4.867, z = 5.02 },
    [4246139] = { x = 7.907, z = 6.31 },
    [4246140] = { x = -6.369, z = -6.36 },
    [4246141] = { x = 5.828, z = -5.6 },
}

function shuffledSpots()
    local spots = {}
    for i, spot in ipairs(SPOTS) do
        spots[i] = spot
    end
    for i = #spots, 2, -1 do
        local j = math.random(i)
        spots[i], spots[j] = spots[j], spots[i]
    end
    return spots
end

function shriek(director)
    shrieks = shrieks + 1
    local first = shrieks == 1
    next_vanish = ticks + math.ceil(SHRIEK_AGAIN_AFTER / TICK)
    vanish(director, NORTH, ACTION_SHRIEK, nil, function(d)
        if first and attempts == 1 then
            d:party_npc_yell("ranged", LINE_SHRIEK)
        end
        after(PLUMES_AFTER_SHRIEK, function(d2)
            if phase ~= 1 then
                return
            end
            d2:play_action_timeline(GARUDA, TIMELINE_PLUMES)
            local at = garuda_at or NORTH
            for _, when in ipairs(FIRST_WAVE) do
                after(when, function(d3)
                    bringPlume(d3, RAZOR_PLUME, at)
                    bringPlume(d3, RAZOR_PLUME, at)
                end)
            end
        end)
    end)
end

function half(director)
    phase = 2
    next_vanish = nil
    talk(director, TALK_HALF)
    after(HALF_VANISH_AFTER_LINE, function(d)
        vanish(d, nil, nil, function(d2)
            after(PLUMES_AFTER_BACK, function(d3)
                d3:play_action_timeline(GARUDA, TIMELINE_PLUMES)
                local spots = shuffledSpots()
                for i, when in ipairs(SECOND_WAVE) do
                    after(when, function(d4)
                        bringPlume(d4, RAZOR_PLUME, spots[2 * i - 1])
                        bringPlume(d4, RAZOR_PLUME, spots[2 * i])
                    end)
                end
            end)
        end)
    end)
end

function song(director)
    phase = 3
    vanish(director, MIDDLE, ACTION_SONG, nil, function(d)
        talk(d, TALK_SONG)
        after(TALK_AFTER_SONG_AFTER, function(d2)
            talk(d2, TALK_AFTER_SONG)
        end)
        after(FEATHER_RAIN_AFTER_SONG, function(d2)
            feather_rain = true
            d2:play_action_timeline(GARUDA, TIMELINE_FEATHER_RAIN)
            featherRain(d2)
        end)
        after(BLAST_AFTER_SONG, aerialBlast)
    end)
end

function featherRain(director)
    if not feather_rain or helper == nil then
        return
    end
    director:actor_use_action(helper, ACTION_FEATHER_RAIN)
    after(FEATHER_RAIN_EVERY, featherRain)
end

function aerialBlast(director)
    if phase ~= 3 then
        return
    end
    phase = 4
    vanish(director, MIDDLE, ACTION_AERIAL_BLAST, nil, function(d)
        after(SATIN_AFTER_BLAST, function(d2)
            d2:play_action_timeline(GARUDA, TIMELINE_PLUMES)
            local spots = shuffledSpots()
            for i, spot in ipairs(spots) do
                after(SATIN_EVERY * i, function(d3)
                    bringPlume(d3, SATIN_PLUME, spot)
                end)
            end
        end)
        after(LAST_AFTER_BLAST, function(d2)
            if phase == 4 then
                phase = 5
                vanish(d2, NORTH, ACTION_LAST)
            end
        end)
    end)
end

-- Her casts from her timeline (659), and the plumes' blasts: the plume falls 2.3 s later.
function onActorCast(director, layout_id, action_id, actor_id, x, y, z, target_id)
    if layout_id == GARUDA and CAST_SECONDS[action_id] ~= nil then
        cast_until = math.max(cast_until, ticks + math.ceil(CAST_SECONDS[action_id] / TICK))
    elseif action_id == ACTION_PLUME_BLAST and (layout_id == RAZOR_PLUME or layout_id == SATIN_PLUME) then
        after(PLUME_FALLS_AFTER, function(d)
            d:kill_bnpc(actor_id)
        end)
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
    if info[GARUDA] ~= nil then
        garuda_at = { x = info[GARUDA].x, y = info[GARUDA].y, z = info[GARUDA].z, rotation = 0.0 }
    end
    local hp = health[GARUDA]
    if hp == nil then
        return
    end
    -- Just after a wipe the old Garuda's health is still there for an update: the new one comes at full health.
    if waiting_for_new_garuda then
        if hp < 100 then
            return
        end
        waiting_for_new_garuda = false
    end
    if not pulled and info[GARUDA] ~= nil and info[GARUDA].target ~= nil then
        pulled = true
        talk(director, TALK_PULL)
        if attempts == 1 then
            director:party_npc_yell("healer", LINE_PULL)
        end
    end
    if away then
        return
    end
    local soonest = cast_until + math.ceil(VANISH_AFTER_CAST / TICK)
    if phase == 1 and shrieks == 0 and vanish_due == nil and hp < SHRIEK_BELOW then
        vanish_due = math.max(ticks + math.ceil(SHRIEK_AFTER_CROSSING / TICK), soonest)
    elseif phase == 1 and hp < HALF_BELOW then
        vanish_due = nil
        half(director)
        return
    elseif phase == 2 and hp < SONG_BELOW and vanish_due == nil then
        vanish_due = math.max(ticks + math.ceil(SONG_AFTER_CROSSING / TICK), soonest)
    end
    if phase == 1 and next_vanish ~= nil and ticks >= next_vanish and ticks >= soonest then
        shriek(director)
    elseif vanish_due ~= nil and ticks >= vanish_due then
        vanish_due = nil
        if phase == 1 then
            shriek(director)
        elseif phase == 2 then
            song(director)
        end
    end
end

function onActorDeath(director, bnpc_id, position)
    if bnpc_id ~= GARUDA then
        return
    end
    talk(director, TALK_DEFEATED)
    feather_rain = false
    later = {}
    for _, plume in ipairs(plumes) do
        director:despawn_bnpc(plume)
    end
    plumes = {}
    director:despawn_bnpc(HELPER)
    helper = nil
    garuda = nil
    director:play_cutscene(CUTSCENE_END, CUTSCENE_END_PARAMS)
    director:complete_duty()
end

-- Everyone down: as they're sent back, Garuda, her helper and the plumes go, and come back as at the start.
function onPartyWipe(director)
    for _, plume in ipairs(plumes) do
        director:despawn_bnpc(plume)
    end
    director:despawn_bnpc(GARUDA)
    director:despawn_bnpc(HELPER)
    attempts = attempts + 1
    reset()
    waiting_for_new_garuda = true
    bringGaruda(director)
end

-- No shortcuts in a trial.
function getDebugShortcut(id)
    return 0
end
