-- Bioblaster (Lv. 72): 50 to the enemies in a cone and 50 over 15 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    mchShot(effects, player, 50, true)
    effects:damage_over_time(MCH_EFFECT_BIOBLASTER, 15.0, physicalDamage(player, levelPotency(player, 50, LEVEL_MCH_ACTION_DAMAGE)))

    return effects
end
