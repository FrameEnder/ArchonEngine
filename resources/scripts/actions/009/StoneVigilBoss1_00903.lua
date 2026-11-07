-- Swinge: the first boss's (BNpcBase 1461) 40-yalm cone (3.7 s; 60 degrees, `resources/data/action_areas.json`) from
-- one of three spots it runs to first, as the director says (r1d1_re.lua; 1239 to the player). From Bob's Chronofoil
-- capture 1cefedf4 (2026-11-07: the Stone Vigil with Duty Support, three wipes). Its area and cast are the Action
-- sheet's; its timeline's `recorded_amounts` scales the hit to retail's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
