-- Eukrasian Dosis I I I (Lv. 82): 90 over 30 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage_over_time(SGE_EFFECT_EUKRASIAN_DOSIS_III, 30.0, magicalDamage(player, levelPotency(player, 90, LEVEL_MAIM_AND_MEND)))
    sgeEukrasian(effects, player)

    return effects
end
