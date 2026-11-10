-- Meikyo Shisui (Lv. 50): three combo steps without their combo, 20 s; Tendo (30 s) from 100.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(SAM_EFFECT_MEIKYO_SHISUI, 3, 20.0)
    if player.level >= 100 then
        effects:gain_effect_self(SAM_EFFECT_TENDO, 0, 30.0)
    end

    return effects
end
