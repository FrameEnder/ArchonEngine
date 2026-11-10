-- Army's Paeon (Lv. 40): self and party members within 50 yalms, direct hit rate +3%, 45 s.
-- Repertoire comes with the Bard's weaponskills while it lasts (Global.lua's brdRepertoire); its haste isn't done.
-- Ends the other songs; Army's Coda from 90.
local EFFECT_ARMYS_PAEON = 2218
local DURATION = 45.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    brdStartSong(effects, player, EFFECT_ARMYS_PAEON, 2)

    return effects
end
