-- Sonic Break (Lv. 54): 340 and 120 over 15 s; Ready to Break spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    gnbDamage(effects, player, 340)
    effects:damage_over_time(GNB_EFFECT_SONIC_BREAK, 15.0, physicalDamage(player, 120))
    effects:consume_effect(GNB_EFFECT_READY_TO_BREAK)

    return effects
end
