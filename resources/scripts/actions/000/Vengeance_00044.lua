-- Vengeance (Lv. 38): damage taken -30% for 15 s.
-- NOTE: the counter-attack (55 potency per physical hit) isn't implemented.
local EFFECT_VENGEANCE = 89
local DURATION = 15.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_VENGEANCE, 0, DURATION)

    return effects
end
