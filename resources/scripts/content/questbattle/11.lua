-- Quest battle: Chasing Shadows, for the quest of the same name (ManFst005_00445)
-- It takes place in a copy of the Central Shroud, so its BNpcs live in the fst_f1 drop-in (layer QB_ManFst005_001.)
-- Their layout IDs, the start position and when each wave arrives come from Sapphire's script for this battle.
--
-- TODO: On retail, Ida and Papalymo fight alongside you (the "B" enemies go after them), and the enemies use their
-- skills. Kawari's NPCs can't fight each other or use skills yet, so for now the allies stand by and everything
-- attacks you.

START_POSITION = { x = 323.0, y = -1.28, z = -320.0 }
START_ROTATION = 3.0

BNPC_BOSS = 3802315
BNPC_IDA = 3802271
BNPC_PAPALYMO = 3802272

-- What arrives at the start, and as the boss's health drops to 75%, 50% and 25%.
INITIAL_ENEMIES = { BNPC_BOSS, 4094218, 4094220 }
WAVES = {
    { health = 75, enemies = { 4094232, 4094233, 4094238, 4094239 } },
    { health = 50, enemies = { 4094474, 4094475, 4094240, 4094471 } },
    { health = 25, enemies = { 4094472, 4094473 } },
}

-- The quest moves to this sequence when the battle is won.
QUEST_SEQUENCE_AFTER_WIN = 2

-- Enemies still standing, by layout ID.
local remaining = {}
local next_wave = 1
local won = false

function spawnEnemies(director, enemies)
    for _, id in ipairs(enemies) do
        remaining[id] = true
        director:spawn_bnpc(id)
    end
end

function onSetup(director)
end

function onCommence(director)
    director:spawn_bnpc(BNPC_IDA)
    director:spawn_bnpc(BNPC_PAPALYMO)

    spawnEnemies(director, INITIAL_ENEMIES)
end

-- health: the health (in percent) of every living NPC, by layout ID
function onUpdate(director, health)
    local boss_health = health[BNPC_BOSS]
    if boss_health == nil then
        return
    end

    while WAVES[next_wave] ~= nil and boss_health <= WAVES[next_wave].health do
        spawnEnemies(director, WAVES[next_wave].enemies)
        next_wave = next_wave + 1
    end
end

function onActorDeath(director, layout_id, position)
    remaining[layout_id] = nil

    if not won and next(remaining) == nil then
        won = true
        director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN)
    end
end
