-- Quest battle: To Guard a Guardian, for the quest of the same name (ManFst007_00447.)
-- It takes place in a copy of the Central Shroud, so its BNpcs live in the fst_f1 drop-in (layer QB_ManFst007_001.)
--
-- Timed against Kawari Recorder's recording of retail (seconds from the fight starting, after the opening cutscene):
--   0      director sequence 2: Galfrid, Miraudont and the guards with you; Quzal Huatotl and six Ixali; more Ixali
--          and guards fighting each other further off. Galfrid: battle talk 161 at 5, Miraudont 162 at 10
--   the first six Ixali down: sequence 3, six more arrive; Galfrid 164 eight seconds later
--   Quzal at 33%: sequence 4, Quzal 165 (and Void Call)
--   Quzal at 25%: sequence 5, Quzal 166, a blanga arrives; Galfrid 163 six seconds later, Miraudont 168 three after
--   everyone down: sequence 6, everyone else leaves and the director's scene 6 plays (a cutscene)
--   after it: sequence 7, a lesser gargoyle and the masked mage; the mage 173 five seconds in
--   22 seconds in: sequence 8, the mage 174 five seconds later
--   gargoyle at 50%: the mage 176
--   gargoyle at 25%: sequence 9, Yda and Papalymo arrive; ten seconds later sequence 10, Papalymo 177, Yda 178 five
--          seconds later, the mage 175 five after that
--   the mage at 40%: the mage 179, Papalymo 180 7.5 seconds later, Yda 181 4.5 after that
--   the mage down: sequence 11, "Duty Complete", the quest battle's scene 11 and back out (the quest moves to
--          sequence 3)
-- Sequence 1 is set before the opening cutscene, as retail does. The layout IDs, health, levels, where everyone stands
-- and the start position are the recording's too. Which six Ixali come first and which in the second wave is a guess
-- from where they stand (the recording doesn't say which layout ID arrived when.)
--
-- The battle talk window (DirectorPopupMessage) shows the content's own text by its ID, so these are the IDs retail
-- sent; the speaker's name and portrait are numbers from the same packets.

START_POSITION = { x = -299.5, y = 21.2, z = 161.24 }
START_ROTATION = 1.88

BNPC_GALFRID = 4089300
BNPC_MIRAUDONT = 4089305
-- The two guards fighting with you.
BNPC_BOW = 3854581
BNPC_LANCE = 3854582
BNPC_QUZAL = 4089289
BNPC_BLANGA = 4089330
BNPC_GARGOYLE = 4089338
BNPC_MAGE = 3854784
BNPC_YDA = 4018837
BNPC_PAPALYMO = 4018839

FIRST_IXALI = { 4089292, 4089293, 4089294, 4089296, 4089297, 4089298 }
SECOND_IXALI = { 3803267, 3803268, 3803269, 4089325, 3803263, 3803264 }
-- Guards and Ixali fighting each other in the background: five fights, each a guard and the one or two Ixali standing
-- next to them. The groups are the layer's own (its NPCs are named INIT_NOTAR_P_POP_<n>ST_.. for the guard and
-- INIT_NOTAR_POP_<n>ST_.. for that fight's Ixali, as Sapphire's script has them), and retail placed each Ixal 2-9
-- yalms from its guard. Both sides have far too much health to win.
BACKGROUND = {
    { guard = 3854583, ixali = { 3854586 } },
    { guard = 3854588, ixali = { 3854584, 3854721 } },
    { guard = 3854673, ixali = { 3854720 } },
    { guard = 3854674, ixali = { 3854719, 3854724 } },
    { guard = 3854675, ixali = { 3854715, 3854723 } },
}
ALLIES_FIRST_HALF = { BNPC_GALFRID, BNPC_MIRAUDONT, BNPC_BOW, BNPC_LANCE }
ALLIES_SECOND_HALF = { BNPC_YDA, BNPC_PAPALYMO }

-- Who goes after whom, as the recording shows (who took whose hits): Quzal after Galfrid, the lostwings after
-- Miraudont, the slowbeaks after the two guards, the sincallers after Galfrid and Miraudont; the blanga, the gargoyle
-- and the masked mage after the player. Anyone not listed goes after the player. Hitting them pulls them over, as
-- enmity does. How hard each hits and heals is in their timelines' recorded_amounts.
LOSTWINGS = { [4089296] = true, [4089297] = true, [4089298] = true, [3803269] = true, [4089325] = true }
SLOWBEAKS = { [4089292] = true, [4089293] = true, [4089294] = true, [3803267] = true, [3803268] = true }
ENEMY_TARGETS = {
    [BNPC_QUZAL] = BNPC_GALFRID,
    [4089296] = BNPC_MIRAUDONT, [4089297] = BNPC_MIRAUDONT, [4089298] = BNPC_MIRAUDONT,
    [3803269] = BNPC_MIRAUDONT, [4089325] = BNPC_MIRAUDONT,
    [4089292] = BNPC_BOW, [4089293] = BNPC_LANCE, [4089294] = BNPC_BOW, [3803267] = BNPC_LANCE, [3803268] = BNPC_BOW,
    [3803263] = BNPC_GALFRID, [3803264] = BNPC_MIRAUDONT,
}

-- Who casts Cure on whom, below what health (in percent), at most this often (in updates), from the recording: the
-- bow, Miraudont and Galfrid keep the two guards (and themselves) up, the sincallers heal Quzal, and Papalymo the
-- player in the second half.
PLAYER = "player"
-- On retail (the Recorder's count of each one's Cures, and who they landed on): the bow 9 and Miraudont 9 (about 60
-- each) and Galfrid 3 (about 200) all on the two guards, never on themselves or the player; the sincallers 12 (about
-- 88), mostly on Quzal; Papalymo once, a big one on the player (660). (The ~100 Cures on the guards and the player in
-- that recording were the player's own.) The thresholds and how often are fitted to those counts.
GUARDS = { BNPC_BOW, BNPC_LANCE }
HEALERS = {
    { healer = BNPC_BOW, targets = GUARDS, below = 50, every = 12 },
    { healer = BNPC_MIRAUDONT, targets = GUARDS, below = 50, every = 12 },
    { healer = BNPC_GALFRID, targets = GUARDS, below = 30, every = 40 },
    { healer = 3803263, targets = { BNPC_QUZAL }, below = 50, every = 12 },
    { healer = 3803264, targets = { BNPC_QUZAL }, below = 50, every = 12 },
    { healer = BNPC_PAPALYMO, targets = { PLAYER }, below = 40, every = 60 },
}

-- Who's talking, by BNpcName, and their portrait in the battle talk window.
SPEAKER_GALFRID = { layout = BNPC_GALFRID, name = 948, icon = 0x11e1a }
SPEAKER_MIRAUDONT = { layout = BNPC_MIRAUDONT, name = 517, icon = 0x11e1b }
SPEAKER_QUZAL = { layout = BNPC_QUZAL, name = 946, icon = 0x11e1c }
SPEAKER_MAGE = { layout = BNPC_MAGE, name = 617, icon = 0x11e12 }
SPEAKER_YDA = { layout = BNPC_YDA, name = 619, icon = 0x11dc4 }
SPEAKER_PAPALYMO = { layout = BNPC_PAPALYMO, name = 620, icon = 0x11dcc }

-- The quest moves to this sequence when the battle is won; retail then plays the quest battle's scene 11.
QUEST_SEQUENCE_AFTER_WIN = 3
FINISH_SCENE = 11
-- The cutscene halfway through, and its scene flags as retail sent them.
CUTSCENE_SCENE = 6
CUTSCENE_FLAGS = 0xF8402EFB
-- Where the cutscene leaves you, as retail's ActorSetPos put it.
AFTER_CUTSCENE_POSITION = { x = -299.54, y = 20.99, z = 161.21 }
AFTER_CUTSCENE_ROTATION = -0.64
SEQUENCE_START = 1
SEQUENCE_FAILED = 0xFE
FAIL_DELAY_UPDATES = 12

ACTION_VOID_CALL = 867
ACTION_CURE = 120

STARTING_ENMITY = 300
ALLY_ENMITY = 10000
-- How much the background fighters hate each other.
BACKGROUND_ENMITY = 100000

-- Steps: { updates after the step before, kind, ... } (Battle:steps)
START_STEPS = {
    { 0, "sequence", 2 },
    { 10, "talk", SPEAKER_GALFRID, 161 },
    { 10, "talk", SPEAKER_MIRAUDONT, 162 },
}
SECOND_WAVE_STEPS = {
    { 0, "sequence", 3 },
    { 0, "spawn", SECOND_IXALI },
    { 16, "talk", SPEAKER_GALFRID, 164 },
}
-- As Quzal's health drops to each of these (in percent).
QUZAL_PHASES = {
    { health = 33, steps = { { 0, "sequence", 4 }, { 0, "talk", SPEAKER_QUZAL, 165 }, { 0, "action", BNPC_QUZAL, ACTION_VOID_CALL } } },
    {
        health = 25,
        steps = {
            { 0, "sequence", 5 },
            { 0, "talk", SPEAKER_QUZAL, 166 },
            { 0, "spawn", { BNPC_BLANGA } },
            { 12, "talk", SPEAKER_GALFRID, 163 },
            { 5, "talk", SPEAKER_MIRAUDONT, 168 },
        },
    },
}
AFTER_CUTSCENE_STEPS = {
    { 0, "sequence", 7 },
    { 0, "spawn", { BNPC_GARGOYLE, BNPC_MAGE } },
    { 10, "talk", SPEAKER_MAGE, 173 },
    { 34, "sequence", 8 },
    { 10, "talk", SPEAKER_MAGE, 174 },
}
GARGOYLE_PHASES = {
    { health = 50, steps = { { 0, "talk", SPEAKER_MAGE, 176 } } },
    {
        health = 25,
        steps = {
            { 0, "sequence", 9 },
            { 0, "spawn", ALLIES_SECOND_HALF },
            { 20, "sequence", 10 },
            { 0, "talk", SPEAKER_PAPALYMO, 177 },
            { 10, "talk", SPEAKER_YDA, 178 },
            { 10, "talk", SPEAKER_MAGE, 175 },
        },
    },
}
MAGE_PHASES = {
    {
        health = 40,
        steps = {
            { 0, "talk", SPEAKER_MAGE, 179 },
            { 15, "talk", SPEAKER_PAPALYMO, 180 },
            { 9, "talk", SPEAKER_YDA, 181 },
        },
    },
}

-- The battle's outline (content/lib/Battle.lua), as retail ran it: Galfrid, and later Yda and Papalymo, take the boss
-- (Quzal, then the masked mage) while it's there; Miraudont clears the lostwings and the guards the slowbeaks that are
-- after them, then the other adds, and only then turn on the boss. Whoever runs out of theirs takes the oldest left.
BATTLE = Battle.new()
for _, id in ipairs({ BNPC_GALFRID, BNPC_YDA, BNPC_PAPALYMO }) do
    BATTLE:ally(id, { focus = { "boss", "adds" } })
end
BATTLE:ally(BNPC_MIRAUDONT, { focus = { "lostwings", "adds", "boss" } })
BATTLE:ally(BNPC_BOW, { focus = { "slowbeaks", "adds", "boss" } })
BATTLE:ally(BNPC_LANCE, { focus = { "slowbeaks", "adds", "boss" } })
for _, id in ipairs({ BNPC_QUZAL, BNPC_MAGE }) do
    BATTLE:enemy(id, { group = "boss", attacks = ENEMY_TARGETS[id] })
end
for _, id in ipairs({ BNPC_BLANGA, BNPC_GARGOYLE }) do
    BATTLE:enemy(id, { group = "adds" })
end
for _, wave in ipairs({ FIRST_IXALI, SECOND_IXALI }) do
    for _, id in ipairs(wave) do
        local group = LOSTWINGS[id] and { "lostwings", "adds" } or SLOWBEAKS[id] and { "slowbeaks", "adds" } or "adds"
        BATTLE:enemy(id, { group = group, attacks = ENEMY_TARGETS[id] })
    end
end
-- The heals, on whichever of their targets is lowest (HEALERS has the evidence.)
for _, h in ipairs(HEALERS) do
    local role = BATTLE.roles[h.healer]
    local heal = { action = ACTION_CURE, targets = {}, below = h.below, every = h.every / 2 }
    for _, target in ipairs(h.targets) do
        table.insert(heal.targets, target == PLAYER and Battle.PLAYER or target)
    end
    role.heals = role.heals or {}
    table.insert(role.heals, heal)
end

-- Phases: as each one's health drops to these, its steps (or as soon as it falls, if it falls before.)
for _, phases in ipairs({ { BNPC_QUZAL, QUZAL_PHASES }, { BNPC_GARGOYLE, GARGOYLE_PHASES }, { BNPC_MAGE, MAGE_PHASES } }) do
    for _, phase in ipairs(phases[2]) do
        BATTLE:onHealth(phases[1], phase.health, function()
            BATTLE:steps(phase.steps)
        end)
    end
end
-- The first six Ixali down bring six more.
BATTLE:onCleared(FIRST_IXALI, function()
    BATTLE:steps(SECOND_WAVE_STEPS)
end)

local half = 1
local over = false
local fail_in = nil

function onSetup(director)
    director:set_sequence(SEQUENCE_START)
end

function onCommence(director)
    -- The background fights aren't part of the battle: nobody here goes after them.
    for _, fight in ipairs(BACKGROUND) do
        director:spawn_bnpc(fight.guard)
        for _, ixal in ipairs(fight.ixali) do
            director:spawn_bnpc(ixal)
            director:set_hate(ixal, fight.guard, BACKGROUND_ENMITY)
        end
        director:set_hate(fight.guard, fight.ixali[1], BACKGROUND_ENMITY)
    end
    BATTLE:spawn(director, ALLIES_FIRST_HALF)
    BATTLE:spawn(director, { BNPC_QUZAL })
    BATTLE:spawn(director, FIRST_IXALI)
    BATTLE:steps(START_STEPS)
end

function onUpdate(director, health, player_health, info)
    if fail_in ~= nil then
        fail_in = fail_in - 1
        if fail_in <= 0 then
            fail_in = nil
            director:fail_quest_battle()
        end
        return
    end
    if over then
        return
    end
    BATTLE:update(director, health, player_health, info)
end

-- Everyone from the first half leaves for the cutscene.
function clearFirstHalf(director)
    for _, fight in ipairs(BACKGROUND) do
        director:despawn_bnpc(fight.guard)
        for _, ixal in ipairs(fight.ixali) do
            director:despawn_bnpc(ixal)
        end
    end
    BATTLE:remove(director, ALLIES_FIRST_HALF)
end

function onActorDeath(director, layout_id, position)
    if over or not BATTLE:died(director, layout_id) then
        return
    end

    if half == 1 and BATTLE:enemiesLeft() == 0 and BATTLE.arrival[BNPC_BLANGA] and BATTLE.arrival[SECOND_IXALI[1]] then
        -- Quzal, both waves and the blanga are down.
        half = 0
        BATTLE:clearSteps()
        director:set_sequence(6)
        clearFirstHalf(director)
        director:play_scene(CUTSCENE_SCENE, CUTSCENE_FLAGS)
        return
    end

    if layout_id == BNPC_MAGE then
        over = true
        BATTLE:clearSteps()
        director:set_sequence(11)
        director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE)
    end
end

function onSceneFinished(director, scene)
    if scene ~= CUTSCENE_SCENE or half ~= 0 then
        return
    end
    half = 2
    -- Retail ends the cutscene with a loading screen into where the second half starts (PrepareZoning to the same
    -- territory, then ActorSetPos, from the recording); the client waits for it.
    director:warp_players(AFTER_CUTSCENE_POSITION.x, AFTER_CUTSCENE_POSITION.y, AFTER_CUTSCENE_POSITION.z,
        AFTER_CUTSCENE_ROTATION)
    BATTLE:steps(AFTER_CUTSCENE_STEPS)
end

-- Losing sends you back to where you entered, still on the same step of the quest, so you can try again.
function onPlayerDeath(director)
    if over or half == 0 then
        return
    end
    over = true
    BATTLE:clearSteps()
    director:set_sequence(SEQUENCE_FAILED)
    fail_in = FAIL_DELAY_UPDATES
end
