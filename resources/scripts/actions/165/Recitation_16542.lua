-- Recitation (Lv. 74): Adloquium, Succor/Concitation, Indomitability and Excogitation cost nothing and are critical, 15 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(SCH_EFFECT_RECITATION, 0, 15.0)

    return effects
end
