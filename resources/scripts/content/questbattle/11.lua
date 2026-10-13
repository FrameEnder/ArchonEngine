-- Quest battle: Chasing Shadows, for the quest of the same name (ManFst005_00445)
-- It takes place in a copy of the Central Shroud, so its BNpcs live in the fst_f1 drop-in (layer QB_ManFst005_001.)
-- Their layout IDs, health, the start position, when each wave arrives and who goes after whom come from Sapphire's
-- script for this battle.
--
-- TODO: The enemies don't use their skills yet, only auto-attacks.

START_POSITION = { x = 323.0, y = -1.28, z = -320.0 }
START_ROTATION = 3.0

BNPC_BOSS = 3802315
BNPC_IDA = 3802271
BNPC_PAPALYMO = 3802272

-- Ida takes on the boss and Papalymo the adds. The boss and the "B" enemies go after them, the "A" enemies after you.
-- How much enemies hate their first target. Damage adds to this, so hitting an enemy a few times pulls it to you.
STARTING_ENMITY = 300
-- How much more Ida and Papalymo hate each new target they're given, so they switch to it and stay on it.
ALLY_ENMITY = 10000
INITIAL_ENEMIES = { a = {}, b = { 4094218, 4094220 } }
-- What arrives as the boss's health drops to 75%, 50% and 25%.
WAVES = {
    { health = 75, a = { 4094238, 4094239 }, b = { 4094232, 4094233 } },
    { health = 50, a = { 4094240, 4094471 }, b = { 4094474, 4094475 } },
    { health = 25, a = { 4094472, 4094473 }, b = {} },
}

-- The quest moves to this sequence when the battle is won.
QUEST_SEQUENCE_AFTER_WIN = 2
-- How many updates (half a second each) to wait after the player falls before sending them out.
FAIL_DELAY_UPDATES = 6

-- Enemies still standing, by layout ID, and the order they arrived in.
local remaining = {}
local arrived = {}
local next_wave = 1
-- Set once the battle is won or lost.
local over = false
-- Counts down after the player falls, see onUpdate.
local fail_in = nil

function spawnEnemy(director, id)
    remaining[id] = true
    table.insert(arrived, id)
    director:spawn_bnpc(id)
end

function spawnEnemies(director, enemies)
    for _, id in ipairs(enemies.a) do
        spawnEnemy(director, id)
        director:set_hate(id, nil, 1) -- the player
    end
    for _, id in ipairs(enemies.b) do
        spawnEnemy(director, id)
        director:set_hate(id, BNPC_PAPALYMO, STARTING_ENMITY)
    end
end

-- The oldest enemy still standing, leaving out the boss if asked.
function oldestRemaining(skip_boss)
    for _, id in ipairs(arrived) do
        if remaining[id] and not (skip_boss and id == BNPC_BOSS) then
            return id
        end
    end
    return nil
end

-- Ida goes for the boss and Papalymo for the adds, oldest first. Either takes whatever's left once theirs are gone.
local ally_priority = 0
function alliesAttack(director)
    local ida_target = nil
    if remaining[BNPC_BOSS] then
        ida_target = BNPC_BOSS
    else
        ida_target = oldestRemaining(false)
    end
    local papalymo_target = oldestRemaining(true) or ida_target

    ally_priority = ally_priority + ALLY_ENMITY
    if ida_target ~= nil then
        director:set_hate(BNPC_IDA, ida_target, ally_priority)
    end
    if papalymo_target ~= nil then
        director:set_hate(BNPC_PAPALYMO, papalymo_target, ally_priority)
    end
end

function onSetup(director)
end

function onCommence(director)
    director:spawn_bnpc(BNPC_IDA)
    director:spawn_bnpc(BNPC_PAPALYMO)

    spawnEnemy(director, BNPC_BOSS)
    director:set_hate(BNPC_BOSS, BNPC_IDA, STARTING_ENMITY)
    spawnEnemies(director, INITIAL_ENEMIES)

    alliesAttack(director)
end

-- health: the health (in percent) of every living NPC, by layout ID
function onUpdate(director, health)
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

    local boss_health = health[BNPC_BOSS]
    if boss_health == nil then
        return
    end

    while WAVES[next_wave] ~= nil and boss_health <= WAVES[next_wave].health do
        spawnEnemies(director, WAVES[next_wave])
        next_wave = next_wave + 1
        -- Papalymo turns to the new adds if he'd run out.
        alliesAttack(director)
    end
end

function onActorDeath(director, layout_id, position)
    if not remaining[layout_id] then
        return
    end
    remaining[layout_id] = nil

    if over then
        return
    end

    if next(remaining) == nil then
        over = true
        director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN)
        return
    end

    -- Whoever the allies were fighting may be gone, so point them at the next one.
    alliesAttack(director)
end

-- Losing sends you back to where you entered, still on the same step of the quest, so you can try again.
function onPlayerDeath(director)
    if over then
        return
    end
    over = true
    fail_in = FAIL_DELAY_UPDATES
end
