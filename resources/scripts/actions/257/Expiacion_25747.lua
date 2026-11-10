-- Expiacion (Lv. 86): 450 on the target, 60 % less round it; MP.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(60)
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, 450))
    effects:restore_mp(500)

    return effects
end
