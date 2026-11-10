-- Vice Of Thorns (Lv. 92): 950 to the first enemy, 55 % less round it; Thorned Flourish spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(55)
    rdmMagic(effects, player, 950)
    effects:consume_effect(RDM_EFFECT_THORNED_FLOURISH)

    return effects
end
