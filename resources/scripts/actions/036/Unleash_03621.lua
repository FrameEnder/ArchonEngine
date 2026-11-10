-- Unleash (Lv. 6): 120 to all enemies near.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    drkDamage(effects, player, 120, DAMAGE_TYPE_MAGIC)
    drkWeaponskill(effects, player)

    return effects
end
