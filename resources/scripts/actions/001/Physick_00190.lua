-- Physick (Lv. 4), Scholar.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 450

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal(healAmount(player, classPotency(player, POTENCY, TRAITS_ACN_MAIM_AND_MEND)))

    return effects
end
