-- Lustrate (Lv. 45): heals 600; an Aetherflow; 10 Faerie Gauge from 70.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal(healAmount(player, classPotency(player, 600, TRAITS_ACN_MAIM_AND_MEND)))
    schSpendAetherflow(effects, player, false)

    return effects
end
