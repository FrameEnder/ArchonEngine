-- Flamethrower (Lv. 70): 120 a tick to the enemies in a cone while it's held, 10 s (here: the target, as damage over time).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage_over_time(MCH_EFFECT_FLAMETHROWER, 10.0, physicalDamage(player, levelPotency(player, 120, LEVEL_MCH_ACTION_DAMAGE)))

    return effects
end
