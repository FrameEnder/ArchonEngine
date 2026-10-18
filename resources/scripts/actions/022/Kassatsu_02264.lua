-- Kassatsu (Lv. 50): the next ninjutsu needs no mudra charges, 15 s.
local EFFECT_KASSATSU = 497
local DURATION = 15.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_KASSATSU, 0, DURATION)

    return effects
end
