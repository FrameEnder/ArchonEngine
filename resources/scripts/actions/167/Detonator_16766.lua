-- Detonator (Lv. 45): ends Wildfire.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:consume_effect(MCH_EFFECT_WILDFIRE)

    return effects
end
