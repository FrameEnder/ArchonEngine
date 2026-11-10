-- Wildfire (Lv. 45): 240 for each of the Machinist's next six weaponskills in 10 s (here as they land); the mark on the target.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(MCH_EFFECT_WILDFIRE, 6, 10.0)
    effects:gain_effect(MCH_EFFECT_WILDFIRE_TARGET, 0, 10.0)

    return effects
end
