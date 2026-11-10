-- Abyssal Drain (Lv. 56): 240 to the target and round it; heals 500.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    drkDamage(effects, player, 240, DAMAGE_TYPE_MAGIC)
    effects:heal_self(healAmount(player, 500))

    return effects
end
