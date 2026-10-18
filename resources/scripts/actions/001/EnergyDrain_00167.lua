-- Energy Drain (Lv. 45), Scholar.
-- NOTE: Scholar's Aetherflow and Faerie gauges and the faerie aren't implemented: it doesn't spend Aetherflow or absorb HP.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, POTENCY, TRAITS_ACN_MAIM_AND_MEND)))

    return effects
end
