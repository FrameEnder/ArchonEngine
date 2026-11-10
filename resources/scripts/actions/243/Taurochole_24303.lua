-- Taurochole (Lv. 62): heals 700 and 10 % less damage taken, 15 s; an Addersgall.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    sgeHeal(effects, player, 700)
    effects:gain_effect(SGE_EFFECT_TAUROCHOLE, 0, 15.0)
    sgeAddersgall(effects, player)

    return effects
end
