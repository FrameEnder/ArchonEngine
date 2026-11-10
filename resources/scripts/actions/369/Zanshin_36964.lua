-- Zanshin (Lv. 96): 940 to the first enemy in a cone, 40 % less to the rest; 50 Kenki; Zanshin Ready spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(40)
    samDamage(effects, player, 940)
    effects:gauge_add("kenki", -50)
    effects:consume_effect(SAM_EFFECT_ZANSHIN_READY)

    return effects
end
