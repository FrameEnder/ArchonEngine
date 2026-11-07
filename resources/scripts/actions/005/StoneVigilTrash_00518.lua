-- An action of the Stone Vigil's 1463s: a hit on its target about every 21 s (1094 to the player). From Bob's
-- Chronofoil capture 1cefedf4 (2026-11-07: the Stone Vigil with Duty Support, three wipes). Its area and cast are the
-- Action sheet's; its timeline's `recorded_amounts` scales the hit to retail's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
