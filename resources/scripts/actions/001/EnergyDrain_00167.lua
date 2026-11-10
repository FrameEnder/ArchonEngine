-- Energy Drain (Lv. 45): 100, some of it drained as HP (not done); an Aetherflow; 10 Faerie Gauge from 70.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, 100, TRAITS_ACN_MAIM_AND_MEND)))
    schSpendAetherflow(effects, player, false)

    return effects
end
