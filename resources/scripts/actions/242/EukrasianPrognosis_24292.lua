-- Eukrasian Prognosis (Lv. 30): heals the party near 100 and a barrier of 320 % of it, 30 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    sgeHeal(effects, player, 100, true)
    effects:gain_effect(SGE_EFFECT_EUKRASIAN_PROGNOSIS, 0, 30.0)
    sgeEukrasian(effects, player)

    return effects
end
