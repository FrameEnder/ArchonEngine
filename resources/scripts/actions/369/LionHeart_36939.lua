-- Lion Heart (Lv. 100): 1,000 to the target, 60 % less round it, after Noble Blood.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(60)
    gnbDamage(effects, player, 1000)

    return effects
end
