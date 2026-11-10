-- Emergency Tactics (Lv. 100): Emergency Tactics with Seraphism's quick recast.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(SCH_EFFECT_EMERGENCY_TACTICS, 0, 15.0)

    return effects
end
