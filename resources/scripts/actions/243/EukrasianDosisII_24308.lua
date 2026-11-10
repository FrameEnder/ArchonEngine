-- Eukrasian Dosis I I (Lv. 72): 60 over 30 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage_over_time(SGE_EFFECT_EUKRASIAN_DOSIS_II, 30.0, magicalDamage(player, levelPotency(player, 60, LEVEL_MAIM_AND_MEND)))
    sgeEukrasian(effects, player)

    return effects
end
