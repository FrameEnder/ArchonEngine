-- Eukrasian Dyskrasia (Lv. 82): 40 over 30 s to all enemies near.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage_over_time(SGE_EFFECT_EUKRASIAN_DYSKRASIA, 30.0, magicalDamage(player, levelPotency(player, 40, LEVEL_MAIM_AND_MEND)))
    sgeEukrasian(effects, player)

    return effects
end
