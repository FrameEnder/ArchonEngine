-- Noble Blood (Lv. 100): 900 to the target, 60 % less round it, after Reign of Beasts.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(60)
    gnbDamage(effects, player, 900)

    return effects
end
