-- Barrage (Lv. 38): the next Wide Volley is stronger, 10 s.
local EFFECT_BARRAGE = 128
local DURATION = 10.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_BARRAGE, 0, DURATION)

    return effects
end
