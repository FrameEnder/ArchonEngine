-- Hammer Stamp (Lv. 50): a sure critical direct hit of 560 to the target, 70 % less round it; a Hammer Time stack.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:guarantee_critical()
    effects:guarantee_direct_hit()
    pctMagic(effects, player, tierPotency(player, { { 94, 560 }, { 84, 540 }, { 74, 500 }, { 54, 420 }, { 1, 380 } }), 70)
    effects:consume_effect_stack(PCT_EFFECT_HAMMER_TIME)

    return effects
end
