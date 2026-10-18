-- Chi (Lv. 35): a mudra, 6 s.
-- NOTE: mudras and Ninjutsu combinations (and the Ninki gauge) aren't implemented: it only grants Mudra.
local EFFECT_MUDRA = 496
local DURATION = 6.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_MUDRA, 0, DURATION)

    return effects
end
