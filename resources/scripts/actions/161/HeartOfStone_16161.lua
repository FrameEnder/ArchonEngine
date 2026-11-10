-- Heart Of Stone (Lv. 68): 15 % less damage taken, 7 s (Brutal Shell too, 30 s, if the Gunbreaker has it).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(GNB_EFFECT_HEART_OF_STONE, 0, 7.0)

    return effects
end
