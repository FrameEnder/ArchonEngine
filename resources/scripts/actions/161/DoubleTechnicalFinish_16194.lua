-- Double Technical Finish (Lv. 70): 720 to the first enemy near, 60 % less to the rest; the dance over; Flourishing Finish (82) and Dance of the Dawn Ready (100). Technical Finish (2 % more damage, 20 s) and Esprit (the party's come with 33216, which the client sends after it).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    dncFinish(effects, player, DNC_EFFECT_TECHNICAL_STEP, 720)
    if player.level >= 82 then
        effects:gain_effect_self(DNC_EFFECT_FLOURISHING_FINISH, 0, 30.0)
    end
    if player.level >= 100 then
        effects:gain_effect_self(DNC_EFFECT_DANCE_OF_THE_DAWN_READY, 0, 30.0)
    end

    effects:gain_effect_self(DNC_EFFECT_TECHNICAL_FINISH, 0, 20.0)
    effects:gain_effect_self(DNC_EFFECT_TECHNICAL_ESPRIT, 0, 20.0)
    return effects
end
