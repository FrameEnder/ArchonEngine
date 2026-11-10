-- Forked Raiju (Lv. 90): 700; 5 Ninki; a Raiju Ready stack spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, 700))
    effects:consume_effect_stack(NIN_EFFECT_RAIJU_READY)
    ninGainNinki(effects, player, 5)

    return effects
end
