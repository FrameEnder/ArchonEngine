-- Life Surge (Lv. 6): the next weaponskill is a critical hit, 5 s.
local EFFECT_LIFE_SURGE = 116
local DURATION = 5.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_LIFE_SURGE, 0, DURATION)

    return effects
end
