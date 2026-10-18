-- Low Blow (Lv. 12) - role action: stuns for 5 s.
local STUN_DURATION = 5.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(EFFECT_STUN, 0, STUN_DURATION)

    return effects
end
