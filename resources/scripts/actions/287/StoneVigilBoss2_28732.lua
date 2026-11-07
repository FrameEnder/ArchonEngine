-- Spiked Tail: the second boss's (BNpcBase 14535) hit on its target (4.7 s; 366 to the Duty Support tank). From Bob's
-- Chronofoil capture 1cefedf4 (2026-11-07: the Stone Vigil with Duty Support, three wipes). Its area and cast are the
-- Action sheet's; its timeline's `recorded_amounts` scales the hit to retail's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
