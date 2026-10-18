-- Lustrate (Lv. 45).
-- NOTE: Scholar's Aetherflow and Faerie gauges and the faerie aren't implemented: it doesn't spend Aetherflow.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 600

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal(healAmount(player, classPotency(player, POTENCY, TRAITS_ACN_MAIM_AND_MEND)))

    return effects
end
