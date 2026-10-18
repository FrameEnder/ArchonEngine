-- Leg Sweep (Lv. 10) - role action: stuns for 3 s. (Action 82 is the older Lancer one.)
local STUN_DURATION = 3.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(EFFECT_STUN, 0, STUN_DURATION)

    return effects
end
