-- Zwerchhau (Lv. 35): 100, 150 after Riposte.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, in_combo and 150 or 100))

    return effects
end
