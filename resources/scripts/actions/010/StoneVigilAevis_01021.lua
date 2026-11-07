-- Blazing Trail: a perched dragon's (BNpcBase 1460) breath ahead of it, instant, as the director says (656 to the
-- player, 594 to the Duty Support members). From Bob's Chronofoil capture 1cefedf4 (2026-11-07: the Stone Vigil with
-- Duty Support, three wipes). Its area and cast are the Action sheet's; its timeline's `recorded_amounts` scales the
-- hit to retail's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))

    return effects
end
