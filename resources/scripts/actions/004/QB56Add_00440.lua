-- Like Mother, Like Daughter's 1684s (quest battle 56, BNpcBase 1684): a 2.2 s cast about every 21 s that caught 1-4
-- of the conjurers, the sprites and you (Bob's Chronofoil capture b23b91ef, 2026-10-26, two runs: 322 on an NPC, 267
-- on you; area from the Action sheet). Its timeline's `recorded_amounts` scales the hit to retail's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PHYSICAL, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
