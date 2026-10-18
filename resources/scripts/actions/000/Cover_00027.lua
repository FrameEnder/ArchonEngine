-- Cover (Lv. 45): the target is Covered, the paladin Cover, 12 s.
-- NOTE: redirecting the damage isn't implemented, and it doesn't cost Oath (50).
local EFFECT_COVER = 80
local EFFECT_COVERED = 81
local DURATION = 12.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_COVER, 20, DURATION) -- retail's param 20
    effects:gain_effect(EFFECT_COVERED, 0, DURATION)

    return effects
end
