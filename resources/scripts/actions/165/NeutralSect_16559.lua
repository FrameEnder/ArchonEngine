-- Neutral Sect (Lv. 80): 20 % more healing, 20 s; Suntouched (30 s) from 100.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(AST_EFFECT_NEUTRAL_SECT, 0, 20.0)
    if player.level >= 100 then
        effects:gain_effect_self(AST_EFFECT_SUNTOUCHED, 0, 30.0)
    end

    return effects
end
