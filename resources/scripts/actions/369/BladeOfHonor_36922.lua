-- Blade Of Honor (Lv. 100): 1,000 on the target, 60 % less round it; only with Blade of Honor Ready.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(60)
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, 1000))
    effects:consume_effect(PLD_EFFECT_BLADE_OF_HONOR_READY)

    return effects
end
