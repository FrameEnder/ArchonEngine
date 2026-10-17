-- Quest battle: Leia's Legacy, for the quest of the same name (SubFst055_00161.)
-- It takes place in a copy of the Central Shroud, so its BNpcs live in the fst_f1 drop-in (layer QB_SubFst055_001.)
--
-- Timed against Kawari Recorder's recording of retail (seconds from the fight starting, after the opening cutscene):
--   0     director sequence 2; Janremi: battle talk 101, Kuplu Kopo 104 five seconds later
--   Janremi at 70%: sequence 3, Janremi 102; Kuplu 105 ten seconds later
--   Janremi at 40%: sequence 4, Janremi 103
--   Janremi at 25%: sequence 5, Kuplu 106 three seconds later; Janremi uses Heartstopper from then on
--   everyone down: sequence 6, "Duty Complete", the quest's scene 6 and back out (the quest moves to sequence 4)
-- Sequence 1 is set before the opening cutscene, as retail does. The layout IDs, health, levels, where everyone stands
-- and the start position are the recording's too.
--
-- The battle talk window (DirectorPopupMessage) shows the content's own text by its ID, so these are the IDs retail
-- sent; the speaker's name and portrait are numbers from the same packets.

-- Where retail puts the player and which way they face: their SpawnPlayer in Bob's Chronofoil capture (2026-10-16).
START_POSITION = { x = -178.11, y = -27.43, z = 292.18 }
START_ROTATION = -1.288

BNPC_JANREMI = 2319770
BNPC_KUPLU = 4097283
-- The hedge knights: two with swords, and one who casts (and heals the others.)
BNPC_KNIGHT_CASTER = 2319772
ENEMIES = { BNPC_JANREMI, 2319771, 2319773, BNPC_KNIGHT_CASTER }
-- Who goes after whom, as on retail: Janremi after the player, the hedge knights after Kuplu Kopo. Kuplu takes the
-- caster first, then the swords, then Janremi. (How hard each hits the other, and heals, is in their timelines'
-- recorded_amounts, from the same recording, so Kuplu holds out as long as he did there.)
ENEMY_TARGETS = { [BNPC_JANREMI] = nil, [2319771] = BNPC_KUPLU, [2319773] = BNPC_KUPLU, [BNPC_KNIGHT_CASTER] = BNPC_KUPLU }
KUPLU_ORDER = { BNPC_KNIGHT_CASTER, 2319771, 2319773, BNPC_JANREMI }

-- Who's talking, by BNpcName, and their portrait in the battle talk window.
SPEAKER_JANREMI = { layout = BNPC_JANREMI, name = 185, icon = 0x11e1f }
SPEAKER_KUPLU = { layout = BNPC_KUPLU, name = 752, icon = 0x11e20 }

-- The quest moves to this sequence when the battle is won; retail then plays the quest battle's scene 6.
QUEST_SEQUENCE_AFTER_WIN = 4
FINISH_SCENE = 6
SEQUENCE_START = 1
SEQUENCE_FIGHTING = 2
SEQUENCE_WON = 6
SEQUENCE_FAILED = 0xFE
-- How many updates (half a second each) after the player falls before sending them out.
FAIL_DELAY_UPDATES = 12

-- Janremi's Heartstopper, once he's below 25%: retail used it every six seconds or so.
ACTION_HEARTSTOPPER = 866
HEARTSTOPPER_EVERY = 12

-- Who casts Cure on whom, below what health (in percent), at most this often (in updates.) On retail Kuplu Kopo healed
-- the player and himself (once he was down to about 40 %), and the casting hedge knight the other knights (down to
-- about 75 %.)
ACTION_CURE = 120
PLAYER = "player"
HEALERS = {
    { healer = BNPC_KUPLU, targets = { PLAYER, BNPC_KUPLU }, below = 50, every = 8 },
    { healer = BNPC_KNIGHT_CASTER, targets = { 2319771, 2319773, BNPC_KNIGHT_CASTER, BNPC_JANREMI }, below = 75, every = 10 },
}

-- How much the enemies hate the player to begin with; damage adds to this.
STARTING_ENMITY = 300
-- How much more Kuplu Kopo hates each new target, so he switches to it.
ALLY_ENMITY = 10000

-- Steps: { updates after the step before, kind, ... }
START_STEPS = {
    { 0, "sequence", SEQUENCE_FIGHTING },
    { 0, "talk", SPEAKER_JANREMI, 101 },
    { 10, "talk", SPEAKER_KUPLU, 104 },
}

-- As Janremi's health drops to each of these (in percent), these steps run.
PHASES = {
    { health = 70, steps = { { 0, "sequence", 3 }, { 0, "talk", SPEAKER_JANREMI, 102 }, { 20, "talk", SPEAKER_KUPLU, 105 } } },
    { health = 40, steps = { { 0, "sequence", 4 }, { 0, "talk", SPEAKER_JANREMI, 103 } } },
    { health = 25, steps = { { 0, "sequence", 5 }, { 0, "heartstopper" }, { 6, "talk", SPEAKER_KUPLU, 106 } } },
}

local remaining = {}
local next_phase = 1
local pending = {}
local over = false
local fail_in = nil
local heartstopper_in = nil
local ally_priority = 0

-- Queues steps after whatever's already waiting, each `delay` updates after the one before.
function queue(steps)
    local at = 0
    for _, step in ipairs(pending) do
        at = math.max(at, step[1])
    end
    for _, step in ipairs(steps) do
        at = at + step[1]
        local queued = { at }
        for i = 2, #step do
            queued[i] = step[i]
        end
        table.insert(pending, queued)
    end
end

function runStep(director, step)
    local kind = step[2]
    if kind == "sequence" then
        director:set_sequence(step[3])
    elseif kind == "talk" then
        director:battle_talk(step[3].layout, step[3].name, step[4], step[3].icon)
    elseif kind == "heartstopper" then
        heartstopper_in = 0
    end
end

function runPending(director)
    local still_pending = {}
    for _, step in ipairs(pending) do
        if step[1] <= 0 then
            runStep(director, step)
        else
            step[1] = step[1] - 1
            table.insert(still_pending, step)
        end
    end
    pending = still_pending
end

-- Each healer Cures whoever of theirs is lowest, if they're below its threshold.
function runHeals(director, health, player_health)
    for _, h in ipairs(HEALERS) do
        h.wait = (h.wait or 0) - 1
        if h.wait <= 0 and health[h.healer] ~= nil then
            local best, best_health = nil, h.below
            for _, target in ipairs(h.targets) do
                local current = target == PLAYER and player_health or health[target]
                if current ~= nil and current > 0 and current < best_health then
                    best, best_health = target, current
                end
            end
            if best ~= nil then
                if best == PLAYER then
                    director:use_action(h.healer, ACTION_CURE)
                else
                    director:use_action(h.healer, ACTION_CURE, best)
                end
                h.wait = h.every
            end
        end
    end
end

-- Kuplu Kopo goes after whoever's left, in order.
function allyAttacks(director)
    for _, id in ipairs(KUPLU_ORDER) do
        if remaining[id] then
            ally_priority = ally_priority + ALLY_ENMITY
            director:set_hate(BNPC_KUPLU, id, ally_priority)
            return
        end
    end
end

function onSetup(director)
    director:set_sequence(SEQUENCE_START)
end

function onCommence(director)
    director:spawn_bnpc(BNPC_KUPLU)
    for _, id in ipairs(ENEMIES) do
        remaining[id] = true
        director:spawn_bnpc(id)
        director:set_hate(id, ENEMY_TARGETS[id], STARTING_ENMITY) -- nil is the player
    end
    allyAttacks(director)
    queue(START_STEPS)
end

-- health: the health (in percent) of every living NPC, by layout ID
-- player_health: the player's health in percent (nil if they're not here)
function onUpdate(director, health, player_health)
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

    runPending(director)

    runHeals(director, health, player_health)

    local janremi_health = health[BNPC_JANREMI]
    if janremi_health == nil then
        return
    end
    while PHASES[next_phase] ~= nil and janremi_health <= PHASES[next_phase].health do
        queue(PHASES[next_phase].steps)
        next_phase = next_phase + 1
    end

    if heartstopper_in ~= nil then
        heartstopper_in = heartstopper_in - 1
        if heartstopper_in <= 0 then
            director:use_action(BNPC_JANREMI, ACTION_HEARTSTOPPER) -- at the player
            heartstopper_in = HEARTSTOPPER_EVERY
        end
    end
end

function onActorDeath(director, layout_id, position)
    if not remaining[layout_id] or over then
        return
    end
    remaining[layout_id] = nil
    if layout_id == BNPC_JANREMI then
        heartstopper_in = nil
    end

    if next(remaining) == nil then
        over = true
        pending = {}
        director:set_sequence(SEQUENCE_WON)
        director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE)
        return
    end
    allyAttacks(director)
end

-- Losing sends you back to where you entered, still on the same step of the quest, so you can try again.
function onPlayerDeath(director)
    if over then
        return
    end
    over = true
    pending = {}
    director:set_sequence(SEQUENCE_FAILED)
    fail_in = FAIL_DELAY_UPDATES
end
