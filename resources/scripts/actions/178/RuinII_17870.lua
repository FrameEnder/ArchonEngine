-- Ruin II (Lv. 38), Scholar: instant.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 220

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, POTENCY, TRAITS_ACN_MAIM_AND_MEND)))

    return effects
end
