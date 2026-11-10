-- Protraction (Lv. 86): 10 % more maximum HP and healing taken, 10 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(SCH_EFFECT_PROTRACTION, 0, 10.0)

    return effects
end
