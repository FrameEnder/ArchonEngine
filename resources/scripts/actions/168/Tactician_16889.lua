-- Tactician (Lv. 56): the party near takes 15 % less damage, 15 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(MCH_EFFECT_TACTICIAN, 0, 15.0)

    return effects
end
