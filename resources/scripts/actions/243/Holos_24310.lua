-- Holos (Lv. 76): heals the party near 300 and 10 % less damage taken, 20 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    sgeHeal(effects, player, 300)
    effects:gain_effect(SGE_EFFECT_HOLOS, 0, 20.0)
    effects:gain_effect(SGE_EFFECT_HOLOSAKOS, 0, 30.0)

    return effects
end
