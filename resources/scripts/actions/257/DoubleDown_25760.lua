-- Double Down (Lv. 90): 1,000 to the first enemy near, 15 % less to the rest; two Cartridges.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(15)
    gnbDamage(effects, player, 1000)
    effects:gauge_add("cartridges", -2)

    return effects
end
