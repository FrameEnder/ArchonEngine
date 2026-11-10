-- Combust I I (Lv. 46): 60 over 30 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage_over_time(AST_EFFECT_COMBUST_II, 30.0, magicalDamage(player, levelPotency(player, 60, LEVEL_MAIM_AND_MEND)))

    return effects
end
