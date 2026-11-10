-- Battle Litany (Lv. 52): critical hits 10 % more likely for the party near, 20 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(DRG_EFFECT_BATTLE_LITANY, 0, 20.0)

    return effects
end
