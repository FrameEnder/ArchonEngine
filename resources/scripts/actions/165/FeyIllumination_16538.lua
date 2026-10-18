-- Fey Illumination (Lv. 40): party members' healing magic potency +10%, magic damage taken -5%, 20 s.
-- NOTE: Scholar's Aetherflow and Faerie gauges and the faerie aren't implemented: the caster gives it.
local EFFECT_FEY_ILLUMINATION = 317
local DURATION = 20.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(EFFECT_FEY_ILLUMINATION, 0, DURATION)

    return effects
end
