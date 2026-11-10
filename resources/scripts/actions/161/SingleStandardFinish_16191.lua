-- Single Standard Finish (Lv. 15): 540 to the first enemy near, 60 % less to the rest; Standard Finish (2 % more damage, 60 s) and Esprit; Last Dance Ready from 92.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    dncFinish(effects, player, DNC_EFFECT_STANDARD_STEP, 540)
    effects:gain_effect_self(DNC_EFFECT_STANDARD_FINISH, 0, 60.0)
    effects:gain_effect_self(DNC_EFFECT_STANDARD_ESPRIT, 0, 60.0)
    if player.level >= 92 then
        effects:gain_effect_self(DNC_EFFECT_LAST_DANCE_READY, 0, 30.0)
    end

    return effects
end
