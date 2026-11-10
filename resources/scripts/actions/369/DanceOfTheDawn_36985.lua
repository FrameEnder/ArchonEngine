-- Dance Of The Dawn (Lv. 100): 1,000 to the target, 60 % less round it; 50 Esprit; Dance of the Dawn Ready spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(60)
    dncDamage(effects, player, 1000)
    effects:gauge_add("esprit", -50)
    effects:consume_effect(DNC_EFFECT_DANCE_OF_THE_DAWN_READY)

    return effects
end
