-- Full Metal Field (Lv. 100): a sure critical direct hit of 900 to the first enemy, 25 % less round it; Full Metal Machinist spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:guarantee_critical()
    effects:guarantee_direct_hit()
    effects:aoe_falloff(25)
    mchShot(effects, player, 900, true)
    effects:consume_effect(MCH_EFFECT_FULL_METAL_MACHINIST)

    return effects
end
