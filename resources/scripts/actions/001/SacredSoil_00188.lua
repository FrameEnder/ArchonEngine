-- Sacred Soil (Lv. 50): an area where party members take 10% less damage, 15 s.
-- NOTE: the ground area isn't implemented: the caster gets the status. It doesn't spend Aetherflow.
local EFFECT_SACRED_SOIL = 298
local DURATION = 15.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_SACRED_SOIL, 0, DURATION)

    return effects
end
