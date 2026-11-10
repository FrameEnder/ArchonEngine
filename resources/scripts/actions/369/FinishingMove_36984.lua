-- Finishing Move (Lv. 96): 850 to the first enemy near, 60 % less to the rest; Standard Finish (5 %, 60 s) and Esprit; Last Dance Ready; Finishing Move Ready spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(60)
    dncDamage(effects, player, 850)
    effects:consume_effect(DNC_EFFECT_FINISHING_MOVE_READY)
    effects:gain_effect_self(DNC_EFFECT_STANDARD_FINISH, 0, 60.0)
    effects:gain_effect_self(DNC_EFFECT_STANDARD_ESPRIT, 0, 60.0)
    if player.level >= 92 then
        effects:gain_effect_self(DNC_EFFECT_LAST_DANCE_READY, 0, 30.0)
    end

    return effects
end
