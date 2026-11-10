-- Phantom Kamaitachi (Lv. 82): the shadow's 700 to the target and round it; 10 Ninki; only with Phantom Kamaitachi Ready.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, 700))
    effects:consume_effect(NIN_EFFECT_PHANTOM_KAMAITACHI_READY)
    ninGainNinki(effects, player, 10)

    return effects
end
