-- Mage's Ballad (Lv. 30): self and party members within 50 yalms, damage dealt +1%, 45 s.
-- NOTE: Repertoire procs aren't implemented.
local EFFECT_MAGES_BALLAD = 2217
local DURATION = 45.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(EFFECT_MAGES_BALLAD, 0, DURATION)

    return effects
end
