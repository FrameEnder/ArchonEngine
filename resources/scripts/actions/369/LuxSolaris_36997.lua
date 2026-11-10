-- Lux Solaris (Lv. 100): heals the Summoner and the party near 500; only with Refulgent Lux.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal(healAmount(player, 500))
    effects:consume_effect(SMN_EFFECT_REFULGENT_LUX)

    return effects
end
