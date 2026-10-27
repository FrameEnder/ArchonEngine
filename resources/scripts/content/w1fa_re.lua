-- The Bowl of Embers (Ifrit)
--
-- From Bob's Chronofoil capture b8432613 of 2026-10-27, visit 0.16: the trial with Duty Support during Lord of the
-- Inferno (quest 343, which unlocks it), two attempts at Ifrit (the first wiped as the Infernal Nail stood, the second
-- won), the player a level 22 Conjurer; `questkit chronofoil --battle 0.16` and the visit's packets, with Cactbot's
-- ifrit-nm timeline for the loops neither attempt finished (docs/src/fork/dungeons.md). Numbers only.
--
-- Ifrit stands at the arena's east (15, 0, 0) facing west. From his pull he fights by his timeline
-- (IfritBowlOfEmbers_207: Incinerate, Vulcan Burst); at 80 % he speaks (battle talk 2601) and Eruption joins in; at 50 %
-- he speaks again (2602), summons (ActionTimeline 1398, 4.1 s on) and the Infernal Nail appears in the middle 8.0 s on.
-- As the Nail falls: status 775 on him, his line 2603 and ActionTimeline 140, 2.6 s later he's put in the middle (and
-- 141), and 3.4 s after the Nail he casts Hellfire (458) round the arena; 9.8 s after it 775 goes with his line 2604 and
-- he fights on with Eruption and his Radiant Plumes. At 0 his last line (2605), the closing cutscene and the duty is
-- complete. Everyone down is a party wipe (server/wipe.rs): it all begins again.

IFRIT = 4126276
NAIL = 4126281
HELPER = 4126284
-- Retail spawned twenty helpers with Ifrit (all one layout ID); at most nine cast at once (eight plumes.)
HELPER_COUNT = 9

NAME_IFRIT = 1185
TALK_MS = 5000
TALK_PULL = 2600
TALK_80 = 2601
TALK_50 = 2602
TALK_NAIL_FALLEN = 2603
TALK_HELLFIRE_OVER = 2604
TALK_DEFEATED = 2605

-- The Duty Support members' lines (NpcYell): at the pull the Thaumaturge's 14326 (the first attempt) and the Lancer's
-- 14327 (the second), the Thaumaturge's 13693 0.3 s before the Nail appears (both), and 13981 1.3 s after Ifrit fell
-- below 4 % (4.32 to 3.6 %; he had been below 5 % for 5 s.)
LINE_PULL_FIRST = 14326
LINE_PULL_AGAIN = 14327
LINE_NAIL = 13693
LINE_LOW = 13981
LOW_AT = 4

-- His phases: the timeline's points `after` these (restart_timeline): the lines' text IDs, and Hellfire's.
PHASE_2 = TALK_80
PHASE_3 = TALK_50
PHASE_NONE = TALK_NAIL_FALLEN -- no points: nothing of his own from the Nail's fall to Hellfire
ACTION_HELLFIRE = 458
PHASE_4 = ACTION_HELLFIRE

-- Below these (retail's lines came at 79.9 and 79.5 %, 49.9 and 50.0 %.)
PHASE_2_AT = 80
PHASE_3_AT = 50

-- The summoning (both attempts, from his 50 % line): ActionTimeline 1398 at 4.0 and 4.3 s, the Nail at 7.8 and 8.2 s,
-- appearing (ActorControl 36: 1, 142, then targetable) 0.7-0.8 s later.
SUMMON_TIMELINE = 1398
SUMMON_AFTER = 4.1
NAIL_AFTER = 8.0
NAIL_APPEAR_TIMELINE = 142
NAIL_APPEAR_DELAY = 0.7
NAIL_LINE_AFTER = NAIL_AFTER - 0.3
-- Cactbot: Hellfire 44.3 s after the 50 % line if the Nail still stands (it fell in 29 s in the second attempt; the
-- first wiped first), so then too, the Nail gone.
HELLFIRE_ANYWAY_AFTER = 44.3

-- As the Nail falls (the second attempt, from its health reaching 0): status 775 (9.8 s) and ActionTimeline 140 at
-- once, put in the middle facing north at 2.6 s with ActionTimeline 141 0.1 s later, Hellfire's cast at 3.4 s; no
-- auto-attack until 10.8 s.
STATUS_HELLFIRE = 775
STATUS_HELLFIRE_SECONDS = 9.8
HELLFIRE_TIMELINE_1 = 140
HELLFIRE_TIMELINE_2 = 141
TO_MIDDLE_AFTER = 2.6
HELLFIRE_TIMELINE_2_AFTER = 2.7
HELLFIRE_AFTER = 3.4
HELLFIRE_BUSY = 10.8
MIDDLE = { x = 0.0, y = 0.0, z = 0.0 }

-- Eruption (455, cast on himself): his helper's 733 under someone of the party as it begins (0.1 s on in seven of the
-- nine; the Lancer, the Thaumaturge or the Marauder in the capture, never the player, which may be chance.) Radiant
-- Plume (456): the first ring four helpers' 734 8 yalms from the middle (north, south, east, west), the next eight 20
-- yalms out (those and the diagonals), in turn.
ACTION_ERUPTION = 455
ACTION_ERUPTION_HELPER = 733
ACTION_PLUME = 456
ACTION_PLUME_HELPER = 734
HELPER_Y = -0.02
INNER_RING = 8.0
OUTER_RING = 20.0

-- The closing cutscene (scene 3: 87, then 1, 18, 0, 0, as retail sent it.)
CUTSCENE_END = 87
CUTSCENE_END_PARAMS = { 1, 18, 0, 0 }

ifrit = nil
helpers = {}
phase = 1
pulled = false
nail_up = false
nail_fallen = false
said_low = false
attempts = 1
plumes = 0
-- The party where it stood at the last update: the player and their party's NPCs.
party = {}
-- Things to do later: { at = os.time(), fn = function(director) }
later = {}

function after(seconds, fn)
    table.insert(later, { at = os.time() + seconds, fn = fn })
end

function bringIfrit(director)
    ifrit = director:spawn_bnpc(IFRIT)
    helpers = {}
    for i = 1, HELPER_COUNT do
        helpers[i] = director:spawn_bnpc(HELPER)
        director:hold_bnpc(helpers[i])
    end
end

function reset()
    phase = 1
    pulled = false
    nail_up = false
    nail_fallen = false
    said_low = false
    plumes = 0
    later = {}
end

function onSetup(director)
    director:set_sequence(1)
    reset()
    bringIfrit(director)
end

function talk(director, text)
    director:battle_talk(IFRIT, NAME_IFRIT, text, 0, TALK_MS)
end

function summonNail(director)
    director:delay(SUMMON_AFTER)
    director:play_action_timeline(IFRIT, SUMMON_TIMELINE)
    director:delay(NAIL_LINE_AFTER)
    director:party_npc_yell("ranged", LINE_NAIL)
    director:delay(NAIL_AFTER)
    director:spawn_bnpc(NAIL)
    director:hold_bnpc(NAIL)
    director:appear_bnpc(NAIL, NAIL_APPEAR_TIMELINE, NAIL_APPEAR_DELAY)
    director:delay(0)
    nail_up = true
    after(HELLFIRE_ANYWAY_AFTER, function(director)
        if not nail_fallen and phase == 3 then
            director:despawn_bnpc(NAIL)
            hellfire(director)
        end
    end)
end

-- The Nail fallen (or its time up): Hellfire from the middle, then his last phase.
function hellfire(director)
    nail_fallen = true
    nail_up = false
    phase = 4
    director:restart_timeline(IFRIT, PHASE_NONE)
    director:busy_bnpc(IFRIT, HELLFIRE_BUSY)
    director:set_invulnerable(IFRIT, true)
    if ifrit ~= nil then
        director:gain_effect(ifrit, STATUS_HELLFIRE, 0, STATUS_HELLFIRE_SECONDS)
    end
    talk(director, TALK_NAIL_FALLEN)
    director:play_action_timeline(IFRIT, HELLFIRE_TIMELINE_1)
    director:delay(TO_MIDDLE_AFTER)
    director:warp_bnpc(IFRIT, MIDDLE.x, MIDDLE.y, MIDDLE.z, 0)
    director:delay(HELLFIRE_TIMELINE_2_AFTER)
    director:play_action_timeline(IFRIT, HELLFIRE_TIMELINE_2)
    director:delay(HELLFIRE_AFTER)
    director:actor_use_action(ifrit, ACTION_HELLFIRE)
    director:restart_timeline(IFRIT, PHASE_4)
    director:delay(STATUS_HELLFIRE_SECONDS)
    director:set_invulnerable(IFRIT, false)
    talk(director, TALK_HELLFIRE_OVER)
    director:delay(0)
end

-- Where someone of the party stands, at random.
function someoneOfTheParty()
    if #party == 0 then
        return nil
    end
    return party[math.random(#party)]
end

function ring(radius, diagonals)
    local spots = {}
    local step = diagonals and 45 or 90
    for degrees = 0, 359, step do
        local angle = math.rad(degrees)
        table.insert(spots, { x = MIDDLE.x + radius * math.sin(angle), z = MIDDLE.z + radius * math.cos(angle) })
    end
    return spots
end

function onActorCast(director, layout_id, action_id, actor_id)
    if layout_id ~= IFRIT or #helpers == 0 then
        return
    end
    if action_id == ACTION_ERUPTION then
        local spot = someoneOfTheParty()
        if spot ~= nil then
            director:warp_bnpc(helpers[1], spot.x, HELPER_Y, spot.z, 0)
            director:actor_use_action(helpers[1], ACTION_ERUPTION_HELPER)
        end
    elseif action_id == ACTION_PLUME then
        plumes = plumes + 1
        local spots = (plumes % 2 == 1) and ring(INNER_RING, false) or ring(OUTER_RING, true)
        for i, spot in ipairs(spots) do
            local helper = helpers[i + 1]
            if helper ~= nil then
                director:warp_bnpc(helper, spot.x, HELPER_Y, spot.z, 0)
                director:actor_use_action(helper, ACTION_PLUME_HELPER)
            end
        end
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
    party = {}
    for key, actor in pairs(info) do
        if key == "player" or (type(key) == "string" and key:sub(1, 5) == "party") then
            table.insert(party, { x = actor.x, z = actor.z })
        end
    end

    local hp = health[IFRIT]
    if hp == nil then
        return
    end
    if not pulled and info[IFRIT] ~= nil and info[IFRIT].target ~= nil then
        pulled = true
        talk(director, TALK_PULL)
        if attempts == 1 then
            director:party_npc_yell("ranged", LINE_PULL_FIRST)
        else
            director:party_npc_yell("melee", LINE_PULL_AGAIN)
        end
    end
    if phase == 1 and hp < PHASE_2_AT then
        phase = 2
        talk(director, TALK_80)
        director:restart_timeline(IFRIT, PHASE_2)
    end
    if phase == 2 and hp < PHASE_3_AT then
        phase = 3
        talk(director, TALK_50)
        director:restart_timeline(IFRIT, PHASE_3)
        summonNail(director)
    end
    if not said_low and hp < LOW_AT then
        said_low = true
        director:party_npc_yell("ranged", LINE_LOW)
    end
end

function onActorDeath(director, bnpc_id, position)
    if bnpc_id == NAIL and not nail_fallen then
        hellfire(director)
    elseif bnpc_id == IFRIT then
        talk(director, TALK_DEFEATED)
        for _, helper in ipairs(helpers) do
            director:despawn_bnpc(helper)
        end
        helpers = {}
        ifrit = nil
        director:play_cutscene(CUTSCENE_END, CUTSCENE_END_PARAMS)
        director:complete_duty()
    end
end

-- Everyone down: as they're sent back, Ifrit, the Nail and his helpers go, and come back as at the start (retail
-- spawned him and his helpers again as the players arrived.)
function onPartyWipe(director)
    director:despawn_bnpc(IFRIT)
    director:despawn_bnpc(NAIL)
    for _, helper in ipairs(helpers) do
        director:despawn_bnpc(helper)
    end
    attempts = attempts + 1
    reset()
    bringIfrit(director)
end

-- No shortcuts in a trial.
function getDebugShortcut(id)
    return 0
end
