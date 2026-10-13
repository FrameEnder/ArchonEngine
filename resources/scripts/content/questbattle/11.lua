-- Quest battle: Chasing Shadows, for the quest of the same name (ManFst005_00445)
-- It takes place in a copy of the Central Shroud, so its BNpcs live in the fst_f1 drop-in (layer QB_ManFst005_001.)
-- Their layout IDs and the start position come from Sapphire's script for this battle.
--
-- TODO: On retail, Ida and Papalymo fight alongside you, and more enemies arrive as the boss drops to 75%, 50% and
-- 25% health. Kawari's NPCs can't fight each other yet and directors can't watch an enemy's health, so for now only
-- the boss and its first two adds fight you, and the allies stand by.

START_POSITION = { x = 323.0, y = -1.28, z = -320.0 }
START_ROTATION = 3.0

BNPC_BOSS = 3802315
BNPC_IDA = 3802271
BNPC_PAPALYMO = 3802272
BNPC_ADD_1 = 4094218
BNPC_ADD_2 = 4094220

-- The quest moves to this sequence when the battle is won.
QUEST_SEQUENCE_AFTER_WIN = 2

-- Enemies still standing, by layout ID.
local remaining = {}
local won = false

function onSetup(director)
end

function onCommence(director)
    director:spawn_bnpc(BNPC_IDA)
    director:spawn_bnpc(BNPC_PAPALYMO)

    for _, id in ipairs({ BNPC_BOSS, BNPC_ADD_1, BNPC_ADD_2 }) do
        remaining[id] = true
        director:spawn_bnpc(id)
    end
end

function onActorDeath(director, layout_id, position)
    remaining[layout_id] = nil

    if not won and next(remaining) == nil then
        won = true
        director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN)
    end
end
