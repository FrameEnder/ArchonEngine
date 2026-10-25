-- The Copperbell Mines' last boss's helpers (BNpcBase 9020, layout 9060684): a 5.7 s line through the arena's middle,
-- five at a time 3 s into the boss's 28760 (Bob's Chronofoil captures of the Copperbell Mines, 2026-10-25: 127 to the
-- one player caught). Its timeline's `recorded_amounts` (CopperbellHelper_9020) scales it; its area is the Action
-- sheet's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
