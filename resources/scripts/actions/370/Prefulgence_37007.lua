-- Prefulgence (Lv. 100): 1,200 to the first enemy, 55 % less round it; Prefulgence Ready spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(55)
    rdmMagic(effects, player, 1200)
    effects:consume_effect(RDM_EFFECT_PREFULGENCE_READY)

    return effects
end
