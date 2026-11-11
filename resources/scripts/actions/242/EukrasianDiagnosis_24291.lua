-- Eukrasian Diagnosis (Lv. 30): heals 300 and a barrier of 180 % of it, 30 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local healed = sgeHeal(effects, player, 300, true)
    effects:gain_effect(SGE_EFFECT_EUKRASIAN_DIAGNOSIS, 0, 30.0)
    effects:barrier(SGE_EFFECT_EUKRASIAN_DIAGNOSIS, math.floor(healed * 180 / 100))
    sgeEukrasian(effects, player)

    return effects
end
