-- Touchdown: the last boss's (BNpcBase 1455) landing, instant, round itself, as the director says (312 to the player).
-- From Bob's Chronofoil capture 1cefedf4 (2026-11-07: the Stone Vigil with Duty Support, three wipes). Its area and
-- cast are the Action sheet's; its timeline's `recorded_amounts` scales the hit to retail's.
-- Its hits knocked back (the Knockback sheet's row 1 after the damage, all 14).
local POTENCY = 100
local KNOCKBACK = 1

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, player.parameters:calc_physical_damage(POTENCY))
    effects:knockback(KNOCKBACK)

    return effects
end
