-- Salted Earth (Lv. 52): 50 a tick for 15 s to the enemies in it (here: the target).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage_over_time(DRK_EFFECT_SALTED_EARTH, 15.0, physicalDamage(player, 50))

    return effects
end
