-- Second Generation (Lv. 90): 480 to the target, 75 % less round it, 680 after the step before; an Anguine Tribute.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(75)
    vprDamage(effects, player, in_combo and 680 or 480)
    effects:gauge_add("tribute", -1)

    return effects
end
