-- Rain of Death (Lv. 45): the target and enemies near it.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, classPotency(player, POTENCY, TRAITS_ARC_ACTION_DAMAGE)))

    return effects
end
