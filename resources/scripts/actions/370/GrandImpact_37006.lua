-- Grand Impact (Lv. 96): 600 to the first enemy, 55 % less round it; 3 White and 3 Black Mana; Grand Impact Ready spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(55)
    rdmMagic(effects, player, 600)
    rdmMana(effects, 3, 3)
    effects:consume_effect(RDM_EFFECT_GRAND_IMPACT_READY)

    return effects
end
