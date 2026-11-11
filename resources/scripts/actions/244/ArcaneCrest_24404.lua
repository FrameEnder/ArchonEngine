-- Arcane Crest (Lv. 40): a barrier of 10 % of the Reaper's HP, 5 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(RPR_EFFECT_CREST_OF_TIME_BORROWED, 0, 5.0)
    effects:barrier_percent_self(RPR_EFFECT_CREST_OF_TIME_BORROWED, 10)

    return effects
end
