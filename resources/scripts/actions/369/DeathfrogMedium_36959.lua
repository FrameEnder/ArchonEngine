-- Deathfrog Medium (Lv. 96): 400 to the target and round it; 50 Ninki; only with Higi.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, 400))
    effects:gauge_add("ninki", -50)
    effects:consume_effect(NIN_EFFECT_HIGI)

    return effects
end
