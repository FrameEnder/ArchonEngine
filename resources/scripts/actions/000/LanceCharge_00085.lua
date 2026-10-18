-- Lance Charge (Lv. 30): damage dealt +10% for 20 s.
local EFFECT_LANCE_CHARGE = 1864
local DURATION = 20.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_LANCE_CHARGE, 0, DURATION)

    return effects
end
