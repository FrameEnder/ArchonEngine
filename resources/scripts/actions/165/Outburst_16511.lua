-- Outburst (Lv. 26): the target and enemies near it.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, POTENCY, TRAITS_ACN_MAIM_AND_MEND)))

    return effects
end
