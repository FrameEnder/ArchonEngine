-- Dismantle (Lv. 62): the target deals 10 % less damage, 10 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(MCH_EFFECT_DISMANTLED, 0, 10.0)

    return effects
end
