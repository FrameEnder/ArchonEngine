-- Acceleration (Lv. 50): the next Verthunder (III), Veraero (III), Scatter or Impact instant and sure of its proc, 20 s; Grand Impact Ready (30 s) from 96.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(RDM_EFFECT_ACCELERATION, 0, 20.0)
    if player.level >= 96 then
        effects:gain_effect_self(RDM_EFFECT_GRAND_IMPACT_READY, 0, 30.0)
    end

    return effects
end
