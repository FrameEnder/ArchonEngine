-- Bloodwhetting (Lv. 82): 10 % less damage for 8 s, a heal of 400 with each weaponskill, Stem the Flow (4 s) and Stem the Tide (20 s).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(2678, 0, 8.0)
    effects:gain_effect_self(2679, 0, 4.0)
    effects:gain_effect_self(2680, 0, 20.0)

    return effects
end
