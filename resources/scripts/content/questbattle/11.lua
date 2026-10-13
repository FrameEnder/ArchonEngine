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
ALLIES = { BNPC_IDA, BNPC_PAPALYMO }

-- The "B" enemies go after Ida and Papalymo, the "A" enemies after you.
INITIAL_ENEMIES = { a = {}, b = { 4094218, 4094220 } }
-- What arrives as the boss's health drops to 75%, 50% and 25%.
WAVES = {
    { health = 75, a = { 4094238, 4094239 }, b = { 4094232, 4094233 } },
    { health = 50, a = { 4094240, 4094471 }, b = { 4094474, 4094475 } },
    { health = 25, a = { 4094472, 4094473 }, b = {} },
}

-- The quest moves to this sequence when the battle is won.
QUEST_SEQUENCE_AFTER_WIN = 2

-- Enemies still standing, by layout ID, and the order they arrived in.
local remaining = {}
local arrived = {}
local next_wave = 1
local won = false

function spawnEnemy(director, id)
    remaining[id] = true
    table.insert(arrived, id)
    director:spawn_bnpc(id)
end

function spawnEnemies(director, enemies)
    for _, id in ipairs(enemies.a) do
        spawnEnemy(director, id)
        director:set_hate(id) -- the player
    end
    -- Split them between the two allies.
    for i, id in ipairs(enemies.b) do
        spawnEnemy(director, id)
        director:set_hate(id, ALLIES[(i - 1) % #ALLIES + 1])
    end
end

-- The allies go for the boss first, then whatever's left in the order it arrived.
function alliesAttack(director)
    local target = nil
    if remaining[BNPC_BOSS] then
        target = BNPC_BOSS
    else
        for _, id in ipairs(arrived) do
            if remaining[id] then
                target = id
                break
            end
        end
    end

    if target ~= nil then
        for _, ally in ipairs(ALLIES) do
            director:set_hate(ally, target)
        end
    end
end

function onSetup(director)
end

function onCommence(director)
    director:spawn_bnpc(BNPC_IDA)
    director:spawn_bnpc(BNPC_PAPALYMO)

    spawnEnemy(director, BNPC_BOSS)
    director:set_hate(BNPC_BOSS, BNPC_IDA)
    spawnEnemies(director, INITIAL_ENEMIES)

    alliesAttack(director)
end

-- health: the health (in percent) of every living NPC, by layout ID
function onUpdate(director, health)
    local boss_health = health[BNPC_BOSS]
    if boss_health == nil then
        return
    end

    while WAVES[next_wave] ~= nil and boss_health <= WAVES[next_wave].health do
        spawnEnemies(director, WAVES[next_wave])
        next_wave = next_wave + 1
    end
end

function onActorDeath(director, layout_id, position)
    if not remaining[layout_id] then
        return
    end
    remaining[layout_id] = nil

    if next(remaining) == nil then
        if not won then
            won = true
            director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN)
        end
        return
    end

    -- Whoever the allies were fighting may be gone, so point them at the next one.
    alliesAttack(director)
end
