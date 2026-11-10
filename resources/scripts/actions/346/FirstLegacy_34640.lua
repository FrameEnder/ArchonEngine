-- First Legacy (Lv. 100): 320 to the target, 75 % less round it.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(75)
    vprDamage(effects, player, 320)

    return effects
end
