-- Hammer Brush (Lv. 86): a sure critical direct hit of 580 to the target, 70 % less round it; a Hammer Time stack.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:guarantee_critical()
    effects:guarantee_direct_hit()
    pctMagic(effects, player, 580, 70)
    effects:consume_effect_stack(PCT_EFFECT_HAMMER_TIME)

    return effects
end
