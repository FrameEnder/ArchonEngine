-- Heart Of Light (Lv. 64): the party near takes 5 % less physical and 10 % less magic damage, 15 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(GNB_EFFECT_HEART_OF_LIGHT, 0, 15.0)

    return effects
end
