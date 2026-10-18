-- Mantra (Lv. 42): HP recovery via healing actions +10% for self and nearby party members, 15 s.
local EFFECT_MANTRA = 102
local DURATION = 15.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(EFFECT_MANTRA, 0, DURATION)

    return effects
end
