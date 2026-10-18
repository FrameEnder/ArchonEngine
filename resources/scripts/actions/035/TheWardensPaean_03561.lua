-- The Warden's Paean (Lv. 35): removes a detrimental effect, or a barrier against the next one (30 s).
local EFFECT_THE_WARDENS_PAEAN = 866
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:cleanse()
    effects:gain_effect(EFFECT_THE_WARDENS_PAEAN, 0, DURATION)

    return effects
end
