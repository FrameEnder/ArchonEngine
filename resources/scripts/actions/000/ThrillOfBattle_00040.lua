-- Thrill of Battle (Lv. 30): max HP +20% and healing received +20%, 10 s.
-- NOTE: the extra HP isn't given (the script can't see max HP); the status is applied.
local EFFECT_THRILL_OF_BATTLE = 87
local DURATION = 10.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_THRILL_OF_BATTLE, 0, DURATION)

    return effects
end
