-- Unmend (Lv. 15): 150.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    drkDamage(effects, player, 150, DAMAGE_TYPE_MAGIC)
    drkWeaponskill(effects, player)

    return effects
end
