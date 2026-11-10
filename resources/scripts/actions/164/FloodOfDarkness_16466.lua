-- Flood Of Darkness (Lv. 30): 100 to the enemies in a line; Darkside 30 s more (up to 60).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    drkDamage(effects, player, 100, DAMAGE_TYPE_MAGIC)
    drkDarkside(effects, player)

    return effects
end
