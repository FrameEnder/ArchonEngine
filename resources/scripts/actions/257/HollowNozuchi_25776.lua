-- Hollow Nozuchi (Lv. 86): 70 more to enemies in Doton (here: 70 to the target).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, 70))

    return effects
end
