-- Mage's Ballad (Lv. 30): self and party members within 50 yalms, damage dealt +1%, 45 s.
-- Repertoire comes with the Bard's weaponskills while it lasts (Global.lua's brdRepertoire); its Bloodletter resets
-- aren't done. Ends the other songs; Mage's Coda from 90.
local EFFECT_MAGES_BALLAD = 2217
local DURATION = 45.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    brdStartSong(effects, player, EFFECT_MAGES_BALLAD, 1)

    return effects
end
