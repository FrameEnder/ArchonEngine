-- Army's Paeon (Lv. 40): self and party members within 50 yalms, direct hit rate +3%, 45 s.
-- NOTE: Repertoire procs aren't implemented.
local EFFECT_ARMYS_PAEON = 2218
local DURATION = 45.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(EFFECT_ARMYS_PAEON, 0, DURATION)

    return effects
end
