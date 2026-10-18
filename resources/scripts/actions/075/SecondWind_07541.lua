-- Second Wind (Lv. 8) - role action: heals yourself.
-- 500 below level 94; 800 with Enhanced Second Wind (Lv. 94).
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 500
local ENHANCED_POTENCY = 800
local TRAIT_ENHANCED_SECOND_WIND = 642

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal(healAmount(player, traitPotency(player, TRAIT_ENHANCED_SECOND_WIND, POTENCY, ENHANCED_POTENCY)))

    return effects
end
