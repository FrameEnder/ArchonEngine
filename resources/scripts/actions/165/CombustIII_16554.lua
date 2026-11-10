-- Combust I I I (Lv. 72): 70 over 30 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage_over_time(AST_EFFECT_COMBUST_III, 30.0, magicalDamage(player, levelPotency(player, 70, LEVEL_MAIM_AND_MEND)))

    return effects
end
