-- Quest battle: Feint and Strike, for the quest of the same name (ManSea007_00544.)
-- It takes place in territory 280, taken to be a copy of Western La Noscea (retail's positions in it are Western La
-- Noscea's, where the quest is), so its BNpcs live in the sea_s1f4 drop-in (layer QB_ManSea007_001). Layout IDs,
-- levels, HP, positions, display flags and the start position are retail's: Bob's Chronofoil capture of 2026-10-21,
-- four runs (`questkit chronofoil --battle 0.27`, `0.29`, `0.31`, lost as the player fell, and `0.33`, won; territory
-- 280, duty 334). It's Limsa Lominsa's Duty, Honor, Country (questbattle/38.lua) with its own numbers.
--
-- Timed against those runs (seconds from the fight starting, the director's scene 60000, Kawari's onCommence):
--   0      director sequence 2 (sequence 1 is set before the opening cutscene): the leader (4102995, level 26), a
--          marauder (4102998) and two elites (4103009/10) with you against the boss (4102991, level 20, 3717 HP),
--          three archers and three pugilists; five pairs further off fight each other (scenery: 26400 HP each,
--          display flags 0x4020B). The leader's battle talk 144 at 5 s, the marauder's 145 at 10
--   the boss at 70% (69.1-70.0% in all four runs): sequence 3, six more half a second later (two archers, two
--          pugilists, a conjurer and a thaumaturge), the leader's 147 eight seconds after the sequence (8.0 s in all)
--   the boss at 30%, or at 40% with no more than two of the others standing (29.3% in one run; 39.4% with two
--          standing in the won one): sequence 4 and the boss's 148
--   the boss at 35% after that (34.3% seen 1.9 s before): it casts Void Call (867, 2.7 s) on itself; 4 s after the
--          cast began, sequence 5 and its 149, and the blanga it called half a second later (after you); the
--          leader's 146 five seconds after the sequence. The blanga's Terror Eye (740) comes 16.2 s after it
--          appears, then every 15 s
--   the boss at 10%: the marauder's 154
--   the boss down: 7.8 s after its health reached 0 sequence 6, everyone else leaves (the blanga too) and the director's cutscene plays
--          (scene 6, 0xF8402EFB); retail then sends you on through a loading screen (PrepareZoning to the same
--          territory, then ActorSetPos 1.5 s later)
--   after it: sequence 7, a gargoyle (after you) and the masked mage (spawned not hostile, standing by until hit)
--          half a second later; the mage's 156 five seconds in
--   22 seconds in: sequence 8, the mage's 157 five seconds later
--   the gargoyle at 50%: the mage's 159
--   the gargoyle at 25% (27.4% -> 24.5%): sequence 9 and the late ally (4103188, level 50); ten seconds later
--          sequence 10 and his 160, the mage's 158 five seconds after that. He takes the gargoyle and then the mage
--   the mage at 40%: its 161, the late ally's 162 three seconds later
--   the mage down: at once sequence 11, "Duty Complete", the quest battle's scene 11 (0x2001) and back out; the
--          quest moves to sequence 3
-- The elites' falling only says their line (152, 153; one fell in the won run, both in a lost one); your falling
-- loses it: sequence 0xFE, out about 6 s later.
--
-- Who goes after whom (every run alike): the boss after the leader, the archers after the marauder, the pugilists
-- 4102999 and 4103168/69 after the elite 4103009 and 4103000/01 after the other, the conjurer and thaumaturge after
-- the elite 4103010; the leader stays on the boss, the marauder takes the archers, the elites the pugilists, then
-- everyone what's left, the boss last. The leader and the elite 4103009 Cure their side (and you) when low; the
-- enemy conjurer Cures its side. How hard everyone hits and heals is in their timelines' recorded_amounts.

-- Where retail puts the player and which way they face (the capture.)
START_POSITION = { x = 667.83, y = 8.27, z = 524.17 }
START_ROTATION = -2.74

BNPC_LEADER = 4102995
BNPC_MARAUDER = 4102998
BNPC_ELITE_1 = 4103009
BNPC_ELITE_2 = 4103010
BNPC_BOSS = 4102991
BNPC_BLANGA = 4103180
BNPC_GARGOYLE = 4103186
BNPC_MAGE = 4103187
BNPC_LATE_ALLY = 4103188

FIRST_ARCHERS = { 4102992, 4102993, 4102994 }
FIRST_PUGILISTS = { 4102999, 4103000, 4103001 }
SECOND_WAVE = { 4103160, 4103161, 4103168, 4103169, 4103170, 4103171 }
SECOND_ARCHERS = { 4103160, 4103161 }
SECOND_PUGILISTS = { 4103168, 4103169 }
BNPC_CONJURER = 4103170
BNPC_THAUMATURGE = 4103171
-- The fights in the background, each pair as retail had them fighting: { the player's side, the other side }.
BACKGROUND = { { 4103313, 4103314 }, { 4103316, 4103317 }, { 4103321, 4103322 }, { 4103325, 4103327 },
    { 4103323, 4103324 } }
ALLIES_FIRST_HALF = { BNPC_LEADER, BNPC_MARAUDER, BNPC_ELITE_1, BNPC_ELITE_2 }

ENEMY_TARGETS = {
    [BNPC_BOSS] = BNPC_LEADER,
    [4102992] = BNPC_MARAUDER, [4102993] = BNPC_MARAUDER, [4102994] = BNPC_MARAUDER,
    [4103160] = BNPC_MARAUDER, [4103161] = BNPC_MARAUDER,
    [4102999] = BNPC_ELITE_1, [4103168] = BNPC_ELITE_1, [4103169] = BNPC_ELITE_1,
    [4103000] = BNPC_ELITE_2, [4103001] = BNPC_ELITE_2,
    [BNPC_CONJURER] = BNPC_ELITE_2, [BNPC_THAUMATURGE] = BNPC_ELITE_2,
}

-- Who's talking, by BNpcName, and their portrait in the battle talk window (retail's DirectorPopupMessage.)
SPEAKER_LEADER = { layout = BNPC_LEADER, name = 982, icon = 73231 }
SPEAKER_MARAUDER = { layout = BNPC_MARAUDER, name = 836, icon = 73232 }
SPEAKER_BOSS = { layout = BNPC_BOSS, name = 1373, icon = 73233 }
SPEAKER_MAGE = { layout = BNPC_MAGE, name = 617, icon = 73234 }
SPEAKER_ELITE_1 = { layout = BNPC_ELITE_1, name = 981, icon = 73235 }
SPEAKER_ELITE_2 = { layout = BNPC_ELITE_2, name = 981, icon = 73236 }
SPEAKER_LATE_ALLY = { layout = BNPC_LATE_ALLY, name = 729, icon = 73187 }

-- The quest moves to this sequence when the battle is won; retail then plays the quest battle's scene 11.
QUEST_SEQUENCE_AFTER_WIN = 3
FINISH_SCENE = 11
CUTSCENE_SCENE = 6
CUTSCENE_FLAGS = 0xF8402EFB
-- Where the cutscene leaves you, as retail's ActorSetPos put it.
AFTER_CUTSCENE_POSITION = { x = 658.91, y = 9.56, z = 489.72 }
AFTER_CUTSCENE_ROTATION = -0.046
SEQUENCE_START = 1
SEQUENCE_FAILED = 0xFE
FAIL_DELAY_UPDATES = 12
-- The cutscene came 7.8 s after the boss's health reached 0 (its death 2 s after that, then 5.8 s.)
CUTSCENE_DELAY_UPDATES = 16

ACTION_VOID_CALL = 867
ACTION_TERROR_EYE = 740
ACTION_CURE = 120
BACKGROUND_ENMITY = 100000

-- Steps: { updates after the step before, kind, ... } (Battle:steps)
START_STEPS = {
    { 0, "sequence", 2 },
    { 10, "talk", SPEAKER_LEADER, 144 },
    { 10, "talk", SPEAKER_MARAUDER, 145 },
}
SECOND_WAVE_STEPS = {
    { 0, "sequence", 3 },
    { 1, "spawn", SECOND_WAVE },
    { 15, "talk", SPEAKER_LEADER, 147 },
}
SEQUENCE_4_STEPS = { { 0, "sequence", 4 }, { 0, "talk", SPEAKER_BOSS, 148 } }
VOID_CALL_STEPS = {
    { 0, "call", function(director)
        if BATTLE.alive[BNPC_BOSS] then
            director:use_action(BNPC_BOSS, ACTION_VOID_CALL, BNPC_BOSS) -- on itself, as retail
        end
    end },
    { 8, "sequence", 5 },
    { 0, "talk", SPEAKER_BOSS, 149 },
    { 1, "spawn", { BNPC_BLANGA } },
    { 9, "talk", SPEAKER_LEADER, 146 },
}
SEQUENCE_4_HEALTH = 30
SEQUENCE_4_HEALTH_FEW_LEFT = 40
FEW_LEFT = 2
VOID_CALL_HEALTH = 35
MARAUDER_LINE_HEALTH = 10
-- The blanga's Terror Eye, in updates: the first 16.2 s after it appears, then every 15 s (retail, the won run.)
TERROR_EYE_FIRST = 32
TERROR_EYE_EVERY = 30
AFTER_CUTSCENE_STEPS = {
    { 0, "sequence", 7 },
    { 1, "spawn", { BNPC_GARGOYLE, BNPC_MAGE } },
    { 9, "talk", SPEAKER_MAGE, 156 },
    { 34, "sequence", 8 },
    { 10, "talk", SPEAKER_MAGE, 157 },
}
GARGOYLE_PHASES = {
    { health = 50, steps = { { 0, "talk", SPEAKER_MAGE, 159 } } },
    {
        health = 25,
        steps = {
            { 0, "sequence", 9 },
            { 1, "spawn", { BNPC_LATE_ALLY } },
            { 19, "sequence", 10 },
            { 0, "talk", SPEAKER_LATE_ALLY, 160 },
            { 10, "talk", SPEAKER_MAGE, 158 },
        },
    },
}
MAGE_PHASES = {
    { health = 40, steps = { { 0, "talk", SPEAKER_MAGE, 161 }, { 6, "talk", SPEAKER_LATE_ALLY, 162 } } },
}
-- Allies whose fall says a line (as their health reaches 0); none of them loses the battle.
FALLING_LINES = {
    [BNPC_ELITE_1] = { speaker = SPEAKER_ELITE_1, text = 152 },
    [BNPC_ELITE_2] = { speaker = SPEAKER_ELITE_2, text = 153 },
}

-- The battle's outline (content/lib/Battle.lua), as retail ran it.
BATTLE = Battle.new()
-- The leader holds the boss the whole time (and the boss him), then helps with what's left.
BATTLE:ally(BNPC_LEADER, { focus = { "boss", "adds" }, range = 3 })
-- The marauder takes the archers, the elites the pugilists (the first the oldest, the second the next), then the rest.
BATTLE:ally(BNPC_MARAUDER, { focus = { "archers", "adds", "boss" }, order = "oldest", range = 3 })
BATTLE:ally(BNPC_ELITE_1, { role = "dps", focus = { "pugilists", "adds", "boss" }, order = "oldest", range = 3 })
BATTLE:ally(BNPC_ELITE_2, { role = "dps", focus = { "pugilists", "adds", "boss" }, order = "oldest", range = 3 })
-- The late ally takes the gargoyle and then the mage, from range (a conjurer.)
BATTLE:ally(BNPC_LATE_ALLY, { focus = { "adds", "boss" }, range = 15 })
-- Their Cures (retail, four runs: the leader 46 at 22-79% (median 41%), the most hurt of his side about half the
-- time, you too; the elite 4103009 24.)
BATTLE.roles[BNPC_LEADER].heals = {
    { action = ACTION_CURE, targets = { BNPC_MARAUDER, BNPC_ELITE_1, BNPC_ELITE_2, "self", Battle.PLAYER }, below = 70,
        every = 6 },
}
BATTLE.roles[BNPC_ELITE_1].heals = {
    { action = ACTION_CURE, targets = { BNPC_ELITE_2, BNPC_MARAUDER, "self", BNPC_LEADER }, below = 60, every = 8 },
}

BATTLE:enemy(BNPC_BOSS, { group = "boss", attacks = ENEMY_TARGETS[BNPC_BOSS] })
for _, list in ipairs({ FIRST_ARCHERS, SECOND_ARCHERS }) do
    for _, id in ipairs(list) do
        BATTLE:enemy(id, { group = { "archers", "adds" }, attacks = ENEMY_TARGETS[id] })
    end
end
for _, list in ipairs({ FIRST_PUGILISTS, SECOND_PUGILISTS }) do
    for _, id in ipairs(list) do
        BATTLE:enemy(id, { group = { "pugilists", "adds" }, attacks = ENEMY_TARGETS[id] })
    end
end
-- The conjurer Cures its side (29 Cures in four runs, at 3-50%.)
BATTLE:enemy(BNPC_CONJURER, { group = "adds", attacks = ENEMY_TARGETS[BNPC_CONJURER], range = 15,
    heals = { { action = ACTION_CURE, targets = "allies", below = 50, every = 6 } } })
BATTLE:enemy(BNPC_THAUMATURGE, { group = "adds", attacks = ENEMY_TARGETS[BNPC_THAUMATURGE], range = 15 })
BATTLE:enemy(BNPC_BLANGA, { group = "adds" })
BATTLE:enemy(BNPC_GARGOYLE, { group = "adds" })
-- The mage stands by until hit (retail: until the late ally reached it), then goes after whoever did.
BATTLE:enemy(BNPC_MAGE, { group = "boss", passive = true })

BATTLE:onHealth(BNPC_BOSS, 70, function()
    BATTLE:steps(SECOND_WAVE_STEPS)
end)
BATTLE:onHealth(BNPC_BOSS, MARAUDER_LINE_HEALTH, function()
    BATTLE:steps({ { 0, "talk", SPEAKER_MARAUDER, 154 } })
end)
for _, phases in ipairs({ { BNPC_GARGOYLE, GARGOYLE_PHASES }, { BNPC_MAGE, MAGE_PHASES } }) do
    for _, phase in ipairs(phases[2]) do
        BATTLE:onHealth(phases[1], phase.health, function()
            BATTLE:steps(phase.steps)
        end)
    end
end

local half = 1
local sequence_4 = false
local void_call = false
local cutscene_in = nil
local terror_eye_in = nil
local over = false
local fail_in = nil

function onSetup(director)
    director:set_sequence(SEQUENCE_START)
end

function onCommence(director)
    -- The background fights aren't part of the battle: nobody here goes after them.
    for _, pair in ipairs(BACKGROUND) do
        director:spawn_bnpc(pair[1])
        director:spawn_bnpc(pair[2])
        director:set_hate(pair[1], pair[2], BACKGROUND_ENMITY)
        director:set_hate(pair[2], pair[1], BACKGROUND_ENMITY)
    end
    BATTLE:spawn(director, ALLIES_FIRST_HALF)
    BATTLE:spawn(director, { BNPC_BOSS })
    BATTLE:spawn(director, FIRST_PUGILISTS)
    BATTLE:spawn(director, FIRST_ARCHERS)
    BATTLE:steps(START_STEPS)
end

-- Sequence 4: the boss at 30%, or at 40% with no more than two of the others standing (see the header); then, at
-- 35%, Void Call. Both come (in that order) if it falls before them.
function checkBoss(director, health)
    if half ~= 1 or BATTLE.arrival[BNPC_BOSS] == nil or void_call then
        return
    end
    local boss = 0
    if BATTLE.alive[BNPC_BOSS] then
        boss = health[BNPC_BOSS]
        if boss == nil then
            return -- not spawned yet (the server spawns the script's NPCs after the update that asked for them)
        end
    end
    if not sequence_4 then
        local others = BATTLE:enemiesLeft() - (BATTLE.alive[BNPC_BOSS] and 1 or 0)
        if boss <= SEQUENCE_4_HEALTH or (boss <= SEQUENCE_4_HEALTH_FEW_LEFT and others <= FEW_LEFT) then
            sequence_4 = true
            BATTLE:steps(SEQUENCE_4_STEPS)
        end
    end
    if sequence_4 and boss <= VOID_CALL_HEALTH and BATTLE.alive[BNPC_BOSS] then
        void_call = true
        BATTLE:steps(VOID_CALL_STEPS)
    end
end

-- Everyone from the first half leaves for the cutscene.
function clearFirstHalf(director)
    for _, pair in ipairs(BACKGROUND) do
        director:despawn_bnpc(pair[1])
        director:despawn_bnpc(pair[2])
    end
    local everyone = {}
    for layout, _ in pairs(BATTLE.alive) do
        table.insert(everyone, layout)
    end
    BATTLE:remove(director, everyone)
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
    if cutscene_in ~= nil then
        cutscene_in = cutscene_in - 1
        if cutscene_in <= 0 then
            cutscene_in = nil
            half = 0
            BATTLE:clearSteps()
            director:set_sequence(6)
            clearFirstHalf(director)
            director:play_scene(CUTSCENE_SCENE, CUTSCENE_FLAGS)
        end
        return
    end
    BATTLE:update(director, health, player_health, info)
    checkBoss(director, health or {})

    if BATTLE.alive[BNPC_BLANGA] then
        terror_eye_in = (terror_eye_in or TERROR_EYE_FIRST) - 1
        if terror_eye_in <= 0 then
            director:use_action(BNPC_BLANGA, ACTION_TERROR_EYE) -- at the player
            terror_eye_in = TERROR_EYE_EVERY
        end
    end
end

function lose(director)
    over = true
    BATTLE:clearSteps()
    director:set_sequence(SEQUENCE_FAILED)
    fail_in = FAIL_DELAY_UPDATES
end

function onActorDeath(director, layout_id, position)
    if over or half == 0 then
        return
    end
    local falling = FALLING_LINES[layout_id]
    if falling ~= nil and half == 1 and BATTLE.alive[layout_id] then
        BATTLE:died(director, layout_id)
        director:battle_talk(falling.speaker.layout, falling.speaker.name, falling.text, falling.speaker.icon)
        return
    end
    if not BATTLE:died(director, layout_id) then
        return
    end

    if half == 1 and layout_id == BNPC_BOSS then
        -- The boss down: the cutscene a few seconds later, whoever is still standing (the blanga was, on retail.)
        cutscene_in = CUTSCENE_DELAY_UPDATES
        return
    end

    if layout_id == BNPC_MAGE then
        -- retail: "Duty Complete" the moment the mage's health reached 0.
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
    -- Retail ends the cutscene with a loading screen into where the second half starts; the client waits for it.
    director:warp_players(AFTER_CUTSCENE_POSITION.x, AFTER_CUTSCENE_POSITION.y, AFTER_CUTSCENE_POSITION.z,
        AFTER_CUTSCENE_ROTATION)
    BATTLE:steps(AFTER_CUTSCENE_STEPS)
end

-- Losing sends you back to where you entered, still on the same step of the quest, so you can try again.
function onPlayerDeath(director)
    if over or half == 0 then
        return
    end
    lose(director)
end
