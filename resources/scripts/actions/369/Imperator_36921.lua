-- Imperator (Lv. 96): 580 on the target, 60 % less round it; four stacks of Requiescat and Confiteor Ready (30 s).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(60)
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, 580))
    effects:gain_effect_self(PLD_EFFECT_REQUIESCAT, 4, 30.0)
    effects:gain_effect_self(PLD_EFFECT_CONFITEOR_READY, 0, 30.0)

    return effects
end
