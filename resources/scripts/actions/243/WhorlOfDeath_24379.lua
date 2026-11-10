-- Whorl Of Death (Lv. 35): 100 to all enemies near; Death's Design, 30 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rprDamage(effects, player, 100)
    effects:gain_effect(RPR_EFFECT_DEATHS_DESIGN, 0, 30.0)

    return effects
end
