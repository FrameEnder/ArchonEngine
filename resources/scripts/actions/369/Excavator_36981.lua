-- Excavator (Lv. 96): 660 to the first enemy, 25 % less round it; 20 Battery; Excavator Ready spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(25)
    mchShot(effects, player, 660)
    mchHeat(effects, player, 0, 20)
    effects:consume_effect(MCH_EFFECT_EXCAVATOR_READY)

    return effects
end
