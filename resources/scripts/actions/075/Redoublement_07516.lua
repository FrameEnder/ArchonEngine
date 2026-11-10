-- Redoublement (Lv. 50): 100, 230 after Zwerchhau.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, in_combo and 230 or 100))

    return effects
end
