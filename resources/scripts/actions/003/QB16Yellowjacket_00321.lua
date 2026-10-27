-- Trial by Water's yellowjackets (quest battle 16, BNpcBase 298): every 7.9 s between their auto-attacks, no cast
-- (Bob's Chronofoil capture b23b91ef, 2026-10-26, two runs: 33 on the Wood Wailers, the median of 73, 26 on players.)
-- Its timeline's `recorded_amounts` scales the hit to retail's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PHYSICAL, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
