-- Technical Finish (Lv. 70): 350 to the first enemy near, 60 % less to the rest, with no step done; the dance over.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    dncFinish(effects, player, DNC_EFFECT_TECHNICAL_STEP, 350)
    if player.level >= 82 then
        effects:gain_effect_self(DNC_EFFECT_FLOURISHING_FINISH, 0, 30.0)
    end
    if player.level >= 100 then
        effects:gain_effect_self(DNC_EFFECT_DANCE_OF_THE_DAWN_READY, 0, 30.0)
    end

    return effects
end
