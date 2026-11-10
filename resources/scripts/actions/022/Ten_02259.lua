-- Ten (Lv. 30): a mudra, 6 s.
-- Adds itself (1) to the mudras so far (the gauge's "mudras", Global.lua's ninMudra); Ninjutsu casts what they make.
local EFFECT_MUDRA = 496
local DURATION = 6.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    ninMudra(effects, player, 1)

    return effects
end
