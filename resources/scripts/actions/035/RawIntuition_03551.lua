-- Raw Intuition (Lv. 56): 10 % less damage for 6 s, a heal of 400 with each weaponskill.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(735, 0, 6.0)

    return effects
end
